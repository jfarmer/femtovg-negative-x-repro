#!/bin/sh
# Measures the cost of drawing a Slint-like frame of text (src/textframe.rs) on two femtovg
# revisions, in two scenes (200 labels, and a large table that scrolls every frame) and on three
# renderers (none, OpenGL and WGPU), and checks that both revisions draw the same pixels. The labels
# are also measured stroked, and rotated so that femtovg draws them as outlines.
#
# usage: ./textframe.sh
#
# Environment, with the defaults in parentheses:
#   TEXTFRAME_FEMTOVG_URL  where the revision before the change is fetched from (https://github.com/femtovg/femtovg)
#   TEXTFRAME_MASTER_REV   that revision (upstream master when the benchmark was written)
#   TEXTFRAME_FIXED_URL    where the revision with the change is fetched from (https://github.com/jfarmer/femtovg)
#   TEXTFRAME_FIXED_REV    that revision (the commit of the pull request)
#   REPEATS                how many pairs of runs each row measures; the medians are printed (5)
# A URL may be the path of a local clone, and then nothing is downloaded from it.
#
# Instructions per frame are the difference between a 2,200-frame and a 200-frame run, read from
# `/usr/bin/time -l`, so the setup and the first frames cancel. That needs macOS. The `gl` rows
# open a hidden window and need OpenGL. The `wgpu` rows wait for the GPU after every frame, so
# their times include the GPU's work. Everything is written to femtovg/, target/ and
# out/textframe/, which git ignores.
set -eu

TEXTFRAME_FEMTOVG_URL=${TEXTFRAME_FEMTOVG_URL:-https://github.com/femtovg/femtovg}
TEXTFRAME_MASTER_REV=${TEXTFRAME_MASTER_REV:-9f5ad2614e922677a467e2d6f66fecba14d3a30a}
TEXTFRAME_FIXED_URL=${TEXTFRAME_FIXED_URL:-https://github.com/jfarmer/femtovg}
TEXTFRAME_FIXED_REV=${TEXTFRAME_FIXED_REV:-12a7b961b9ac3f5c19905dd256dbdcf0c097101e}
REPEATS=${REPEATS:-5}

die() {
    echo "textframe.sh: $*" >&2
    exit 1
}

# git fetch runs inside femtovg/<name>, where a relative path would point somewhere else.
absolute() {
    if [ -d "$1" ]; then (cd "$1" && pwd); else echo "$1"; fi
}
TEXTFRAME_FEMTOVG_URL=$(absolute "$TEXTFRAME_FEMTOVG_URL")
TEXTFRAME_FIXED_URL=$(absolute "$TEXTFRAME_FIXED_URL")

cd "$(dirname "$0")"

# checkout <name> <url> <revision>: makes femtovg/<name> a checkout of that revision.
checkout() {
    dir=femtovg/$1
    [ -d "$dir/.git" ] || git init --quiet "$dir"
    if [ "$(git -C "$dir" rev-parse --quiet --verify HEAD || true)" != "$3" ]; then
        git -C "$dir" fetch --quiet --depth 1 "$2" "$3" || die "cannot fetch $3 from $2"
        git -C "$dir" checkout --quiet --detach FETCH_HEAD
    fi
    [ -z "$(git -C "$dir" status --porcelain)" ] || die "$dir has local changes; delete it and run again"
    echo "$dir is at $(git -C "$dir" rev-parse HEAD)"
}
checkout textframe-master "$TEXTFRAME_FEMTOVG_URL" "$TEXTFRAME_MASTER_REV"
checkout textframe-fixed "$TEXTFRAME_FIXED_URL" "$TEXTFRAME_FIXED_REV"

# Each revision is built twice: with femtovg's default features, and with `swash` only, which is
# what Slint uses.
CARGO_TARGET_DIR=$(pwd)/target
export CARGO_TARGET_DIR
out=out/textframe
mkdir -p "$out"
for side in master fixed; do
    (cd "crates/textframe-$side" && cargo build --release --locked) || die "the default build against femtovg/textframe-$side failed"
    cp "target/release/textframe-$side" "$out/$side-default"
    (cd "crates/textframe-$side" && cargo build --release --locked --no-default-features --features swash) ||
        die "the swash build against femtovg/textframe-$side failed"
    cp "target/release/textframe-$side" "$out/$side-swash"
done

# The environment of the binary for each way of drawing the text: fill, stroke, or fill rotated by
# 0.05 radians, which draw_glyph_run draws as outlines.
variant_env() {
    case $1 in
        fill) echo "" ;;
        stroke) echo STROKE=1 ;;
        outlines) echo ROTATE=0.05 ;;
        *) die "unknown variant $1" ;;
    esac
}

# measure <binary> <mode> <renderer> <scene> <variant>: prints the instructions per frame and the
# median frame time of a 2,200-frame run, with a 200-frame run subtracted from the instructions.
measure() {
    vars=$(variant_env "$5")
    short=$( { /usr/bin/time -l env $vars "$1" "$2" 200 "$3" "$4" >/dev/null; } 2>&1 | awk '/instructions retired/ { print $1 }')
    [ -n "$short" ] || die "/usr/bin/time -l printed no instruction count; this needs macOS"
    { /usr/bin/time -l env $vars "$1" "$2" 2200 "$3" "$4"; } 2>&1 | awk -v short="$short" '
        /instructions retired/ { instructions = ($1 - short) / 2000 }
        /median_us=/ { split($0, f, "median_us="); split(f[2], g, " "); us = g[1] }
        END { printf "%.0f %.1f\n", instructions, us }'
}

# compare <scene> <build> <mode> <renderer> <variant>: measures master and then the change, REPEATS times, so
# that a change in the machine's load reaches both, and prints the medians of each and the median
# over the pairs of the change's frame time divided by master's.
compare() {
    i=0
    while [ "$i" -lt "$REPEATS" ]; do
        echo "$(measure "$out/master-$2" "$3" "$4" "$1" "$5") $(measure "$out/fixed-$2" "$3" "$4" "$1" "$5")"
        i=$((i + 1))
    done | awk '
        function median(a, n,   i, j, t) {
            for (i = 2; i <= n; i++) for (j = i; j > 1 && a[j - 1] > a[j]; j--) { t = a[j]; a[j] = a[j - 1]; a[j - 1] = t }
            return n % 2 ? a[(n + 1) / 2] : (a[n / 2] + a[n / 2 + 1]) / 2
        }
        { n++; mi[n] = $1; mt[n] = $2; fi[n] = $3; ft[n] = $4; ratio[n] = $4 / $2 }
        END {
            printf "    master: %.2f M instructions, %.0f us\n", median(mi, n) / 1e6, median(mt, n)
            printf "    fixed:  %.2f M instructions, %.0f us\n", median(fi, n) / 1e6, median(ft, n)
            printf "    fixed frame time / master frame time, median of %d pairs: %.2f\n", n, median(ratio, n)
        }'
}

echo
echo "Load before: $(uptime | sed 's/.*load averages*: //')"
echo "Per frame, median of $REPEATS:"
for row in "labels swash glyph_run void fill" "labels default fill_text void fill" "labels swash glyph_run gl fill" \
    "labels swash glyph_run wgpu fill" "table swash glyph_run void fill" "table default fill_text void fill" \
    "table swash glyph_run gl fill" "table swash glyph_run wgpu fill" "table default fill_text wgpu fill" \
    "labels swash glyph_run void stroke" "labels default fill_text void stroke" "labels swash glyph_run gl stroke" \
    "labels swash glyph_run wgpu stroke" "labels swash glyph_run void outlines" \
    "labels default fill_text void outlines" "labels swash glyph_run gl outlines" \
    "labels swash glyph_run wgpu outlines"; do
    set -- $row
    echo "  $1, $3, $2 build, $4 renderer, $5"
    compare "$1" "$2" "$3" "$4" "$5"
done
echo "Load after: $(uptime | sed 's/.*load averages*: //')"

# The same frames drawn by both revisions, hashed after the last frame.
echo
echo "Pixel hashes:"
emoji=/System/Library/Fonts/Apple\ Color\ Emoji.ttc
differ=0
for row in "swash glyph_run" "default fill_text"; do
    set -- $row
    for renderer in gl wgpu; do
        for variant in fill stroke outlines; do
            vars=$(variant_env $variant)
            for scene in labels same-color emoji table; do
                font=fonts/Roboto-Regular.ttf
                same=0
                mixed=0
                drawn=labels
                case $scene in
                    same-color) same=1 ;;
                    emoji) [ -f "$emoji" ] || continue; font=$emoji; mixed=1 ;;
                    table) drawn=table ;;
                esac
                a=$(env PIXEL_HASH=1 SAME_COLOR=$same EMOJI=$mixed FONT="$font" $vars "$out/master-$1" "$2" 3 $renderer $drawn | grep -o 'hash=[0-9a-f]*')
                b=$(env PIXEL_HASH=1 SAME_COLOR=$same EMOJI=$mixed FONT="$font" $vars "$out/fixed-$1" "$2" 3 $renderer $drawn | grep -o 'hash=[0-9a-f]*')
                if [ "$a" = "$b" ]; then result=same; else result=DIFFERENT; differ=1; fi
                echo "  $2, $1 build, $renderer, $variant, $scene: master $a, fixed $b, $result"
            done
        done
    done
done
[ "$differ" = 0 ] || die "the two revisions drew different pixels"
