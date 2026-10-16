#!/bin/sh
# Measures the CPU cost of drawing a Slint-like frame of text (src/textframe.rs) on two femtovg
# revisions, and checks that both revisions draw the same pixels.
#
# usage: ./textframe.sh
#
# Environment, with the defaults in parentheses:
#   TEXTFRAME_FEMTOVG_URL  where the revision before the change is fetched from (https://github.com/femtovg/femtovg)
#   TEXTFRAME_MASTER_REV   that revision (upstream master when the benchmark was written)
#   TEXTFRAME_FIXED_URL    where the revision with the change is fetched from (https://github.com/jfarmer/femtovg)
#   TEXTFRAME_FIXED_REV    that revision (the commit of the pull request)
#   REPEATS                how many times each count is measured; the median is printed (3)
# A URL may be the path of a local clone, and then nothing is downloaded from it.
#
# Instructions per frame are the difference between a 2,200-frame and a 200-frame run, read from
# `/usr/bin/time -l`, so the setup and the first frames cancel. That needs macOS. The `gl` rows
# and the pixel check open a hidden window and need OpenGL. Everything is written to femtovg/,
# target/ and out/textframe/, which git ignores.
set -eu

TEXTFRAME_FEMTOVG_URL=${TEXTFRAME_FEMTOVG_URL:-https://github.com/femtovg/femtovg}
TEXTFRAME_MASTER_REV=${TEXTFRAME_MASTER_REV:-9f5ad2614e922677a467e2d6f66fecba14d3a30a}
TEXTFRAME_FIXED_URL=${TEXTFRAME_FIXED_URL:-https://github.com/jfarmer/femtovg}
TEXTFRAME_FIXED_REV=${TEXTFRAME_FIXED_REV:-5e4b6e486146b28d8439ef419eef86e5d433916b}
REPEATS=${REPEATS:-3}

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

# frame_cost <binary> <mode> <renderer>: prints the median instructions per frame and the median
# frame time of the long run, over REPEATS pairs of runs.
frame_cost() {
    i=0
    while [ "$i" -lt "$REPEATS" ]; do
        short=$( { /usr/bin/time -l "$1" "$2" 200 "$3" >/dev/null; } 2>&1 | awk '/instructions retired/ { print $1 }')
        long=$( { /usr/bin/time -l "$1" "$2" 2200 "$3"; } 2>&1)
        [ -n "$short" ] || die "/usr/bin/time -l printed no instruction count; this needs macOS"
        echo "$long" | awk -v short="$short" '
            /instructions retired/ { instructions = ($1 - short) / 2000 }
            /median_us=/ { split($0, f, "median_us="); split(f[2], g, " "); us = g[1] }
            END { printf "%.0f %.1f\n", instructions, us }'
        i=$((i + 1))
    done | sort -n | awk '{ n++; ins[n] = $1; us[n] = $2 } END {
        m = int((n + 1) / 2); printf "%.2f M instructions, %.0f us\n", ins[m] / 1e6, us[m] }'
}

echo
echo "Per frame, median of $REPEATS:"
for row in "swash glyph_run void" "default fill_text void" "swash glyph_run gl"; do
    set -- $row
    echo "  $2, $1 build, $3 renderer"
    echo "    master: $(frame_cost "$out/master-$1" "$2" "$3")"
    echo "    fixed:  $(frame_cost "$out/fixed-$1" "$2" "$3")"
done

# The same frame drawn on OpenGL by both revisions, hashed after the last frame.
echo
echo "OpenGL pixel hashes:"
emoji=/System/Library/Fonts/Apple\ Color\ Emoji.ttc
differ=0
for row in "swash glyph_run" "default fill_text"; do
    set -- $row
    for scene in plain same-color emoji; do
        font=fonts/Roboto-Regular.ttf
        same=0
        case $scene in
            same-color) same=1 ;;
            emoji) [ -f "$emoji" ] || continue; font=$emoji ;;
        esac
        a=$(PIXEL_HASH=1 SAME_COLOR=$same FONT="$font" "$out/master-$1" "$2" 3 gl | grep -o 'hash=[0-9a-f]*')
        b=$(PIXEL_HASH=1 SAME_COLOR=$same FONT="$font" "$out/fixed-$1" "$2" 3 gl | grep -o 'hash=[0-9a-f]*')
        if [ "$a" = "$b" ]; then result=same; else result=DIFFERENT; differ=1; fi
        echo "  $2, $1 build, $scene: master $a, fixed $b, $result"
    done
done
[ "$differ" = 0 ] || die "the two revisions drew different pixels"
