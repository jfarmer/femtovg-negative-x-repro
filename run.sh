#!/bin/sh
# Draws five scenes on two femtovg revisions each and four more on one revision each, composes
# the comparison images, and compares the result with the files in images/.
#
# usage: ./run.sh
#
# Environment, with the defaults in parentheses:
#   FEMTOVG_URL   where the revision with the bug is fetched from (https://github.com/femtovg/femtovg)
#   MASTER_REV    that revision (eb4fe53, upstream master when this was written)
#   FIXED_URL     where the revision with the fix is fetched from (https://github.com/jfarmer/femtovg)
#   FIXED_REV     that revision (b064093, the commit of the pull request)
#   NEW_YORK_DIR  the directory with NewYork.ttf and NewYorkItalic.ttf (/System/Library/Fonts)
# The letter-spacing specimen is for another bug and has its own pair of revisions. Their
# default hashes are below.
#   SPACING_FEMTOVG_URL  where the revision with the bug is fetched from (https://github.com/femtovg/femtovg)
#   SPACING_MASTER_REV   that revision (upstream master when the specimen was written)
#   SPACING_FIXED_URL    where the revision with the fix is fetched from (https://github.com/jfarmer/femtovg)
#   SPACING_FIXED_REV    that revision (the commit of the pull request)
# The two gradient scenes are for a third bug and have their own pair of revisions too.
#   GRADIENT_FEMTOVG_URL  where the revision with the bug is fetched from (https://github.com/femtovg/femtovg)
#   GRADIENT_MASTER_REV   that revision (upstream master when the scenes were written)
#   GRADIENT_FIXED_URL    where the revision with the fix is fetched from (https://github.com/jfarmer/femtovg)
#   GRADIENT_FIXED_REV    that revision (the commit of the pull request)
# The shadow scene is for a fourth bug, which has no fix yet, so it has one revision.
#   SHADOW_FEMTOVG_URL  where that revision is fetched from (https://github.com/femtovg/femtovg)
#   SHADOW_MASTER_REV   that revision (upstream master when the scene was written)
# The three cut-off scenes are for a fifth bug, which has no fix yet either.
#   CUTOFF_FEMTOVG_URL  where that revision is fetched from (https://github.com/femtovg/femtovg)
#   CUTOFF_MASTER_REV   that revision (upstream master when the scenes were written)
# A URL may be the path of a local clone, and then nothing is downloaded from it. A revision is
# a full commit hash, or the name of a branch or tag.
#
# Needs git, cargo and a GPU that wgpu can use. Everything is written to femtovg/, target/ and
# out/, which git ignores. images/ is only read.
set -eu

FEMTOVG_URL=${FEMTOVG_URL:-https://github.com/femtovg/femtovg}
MASTER_REV=${MASTER_REV:-eb4fe53d51a6a274754a787a55179140bc7d6c77}
FIXED_URL=${FIXED_URL:-https://github.com/jfarmer/femtovg}
FIXED_REV=${FIXED_REV:-b064093e871bfc08404d86fb0ca3dc52b43d2e79}
NEW_YORK_DIR=${NEW_YORK_DIR:-/System/Library/Fonts}
SPACING_FEMTOVG_URL=${SPACING_FEMTOVG_URL:-https://github.com/femtovg/femtovg}
SPACING_MASTER_REV=${SPACING_MASTER_REV:-485c66566bac9b4a580d2afb8d8230122d6f8457}
SPACING_FIXED_URL=${SPACING_FIXED_URL:-https://github.com/jfarmer/femtovg}
SPACING_FIXED_REV=${SPACING_FIXED_REV:-2bd54638cb7d5d9e2a5337cbf9856d2638eadfdd}
GRADIENT_FEMTOVG_URL=${GRADIENT_FEMTOVG_URL:-https://github.com/femtovg/femtovg}
GRADIENT_MASTER_REV=${GRADIENT_MASTER_REV:-6dd55434177690c845fc5e6f8e9d5a2fe6f1b277}
GRADIENT_FIXED_URL=${GRADIENT_FIXED_URL:-https://github.com/jfarmer/femtovg}
GRADIENT_FIXED_REV=${GRADIENT_FIXED_REV:-38d649c699b339f040a2e8f8a13ed28e1644baf7}
SHADOW_FEMTOVG_URL=${SHADOW_FEMTOVG_URL:-https://github.com/femtovg/femtovg}
SHADOW_MASTER_REV=${SHADOW_MASTER_REV:-d70ffeb79658606fe220801115ccefff33897e86}
CUTOFF_FEMTOVG_URL=${CUTOFF_FEMTOVG_URL:-https://github.com/femtovg/femtovg}
CUTOFF_MASTER_REV=${CUTOFF_MASTER_REV:-d70ffeb79658606fe220801115ccefff33897e86}

die() {
    echo "run.sh: $*" >&2
    exit 1
}

# git fetch runs inside femtovg/<name>, where a relative path would point somewhere else.
absolute() {
    if [ -d "$1" ]; then (cd "$1" && pwd); else echo "$1"; fi
}
FEMTOVG_URL=$(absolute "$FEMTOVG_URL")
FIXED_URL=$(absolute "$FIXED_URL")
SPACING_FEMTOVG_URL=$(absolute "$SPACING_FEMTOVG_URL")
SPACING_FIXED_URL=$(absolute "$SPACING_FIXED_URL")
GRADIENT_FEMTOVG_URL=$(absolute "$GRADIENT_FEMTOVG_URL")
GRADIENT_FIXED_URL=$(absolute "$GRADIENT_FIXED_URL")
SHADOW_FEMTOVG_URL=$(absolute "$SHADOW_FEMTOVG_URL")
CUTOFF_FEMTOVG_URL=$(absolute "$CUTOFF_FEMTOVG_URL")

cd "$(dirname "$0")"

# checkout <name> <url> <revision>: makes femtovg/<name> a checkout of that revision.
checkout() {
    dir=femtovg/$1
    [ -d "$dir/.git" ] || git init --quiet "$dir"
    # Skips the fetch when the checkout is already at this commit, so that a second run needs
    # no network. This only matches when the revision is given as a full hash.
    if [ "$(git -C "$dir" rev-parse --quiet --verify HEAD || true)" != "$3" ]; then
        git -C "$dir" fetch --quiet --depth 1 "$2" "$3" || die "cannot fetch $3 from $2"
        git -C "$dir" checkout --quiet --detach FETCH_HEAD
    fi
    # The scenes are built against these files, so they have to be exactly the revision.
    [ -z "$(git -C "$dir" status --porcelain)" ] || die "$dir has local changes; delete it and run again"
    echo "$dir is at $(git -C "$dir" rev-parse HEAD)"
}
checkout master "$FEMTOVG_URL" "$MASTER_REV"
checkout fixed "$FIXED_URL" "$FIXED_REV"
checkout spacing-master "$SPACING_FEMTOVG_URL" "$SPACING_MASTER_REV"
checkout spacing-fixed "$SPACING_FIXED_URL" "$SPACING_FIXED_REV"
checkout gradient-master "$GRADIENT_FEMTOVG_URL" "$GRADIENT_MASTER_REV"
checkout gradient-fixed "$GRADIENT_FIXED_URL" "$GRADIENT_FIXED_REV"
checkout shadow-master "$SHADOW_FEMTOVG_URL" "$SHADOW_MASTER_REV"
checkout cutoff-master "$CUTOFF_FEMTOVG_URL" "$CUTOFF_MASTER_REV"

# All builds read the fonts from the same files.
roboto=femtovg/master/examples/assets/RobotoFlex-VariableFont.ttf
[ -f "$roboto" ] || die "$roboto is missing. The About panel, the specimen and the captions are set in Roboto Flex, which femtovg keeps in examples/assets."
scenes="about menu"
for font in "$NEW_YORK_DIR/NewYork.ttf" "$NEW_YORK_DIR/NewYorkItalic.ttf"; do
    if [ ! -f "$font" ]; then
        echo "run.sh: $font is missing. The menu is set in New York, which comes with macOS; set NEW_YORK_DIR if it is somewhere else. The menu card is not drawn." >&2
        scenes=about
        break
    fi
done

# One target directory for the eight crates, so that the dependencies are compiled once.
CARGO_TARGET_DIR=$(pwd)/target
export CARGO_TARGET_DIR
for revision in master fixed spacing-master spacing-fixed gradient-master gradient-fixed shadow-master cutoff-master; do
    (cd "crates/$revision" && cargo build --release --locked) || die "the build against femtovg/$revision failed"
done
bin=target/release
mkdir -p out

# draw <scene> <font file>...
draw() {
    scene=$1
    shift
    "$bin/$scene-master" "$@" "out/$scene-master.png" "out/$scene-glyphs.tsv" >"out/$scene-summary.txt"
    "$bin/$scene-fixed" "$@" "out/$scene-fixed.png" "out/$scene-glyphs-fixed.tsv" >/dev/null
    # replay.rs computes the table from the glyph positions alone. If the two builds wrote
    # different tables, the fix would have changed the layout and not only the masks.
    cmp "out/$scene-glyphs.tsv" "out/$scene-glyphs-fixed.tsv" || die "the two revisions place the glyphs of the $scene scene differently"
    rm "out/$scene-glyphs-fixed.tsv"
    echo "$scene-master.png and $scene-fixed.png: $("$bin/compare" "out/$scene-master.png" "out/$scene-fixed.png")" >>"out/$scene-summary.txt"
}

# compose <side|zoom> <scene> <x> <y> <w> <h> <output name>: the same rectangle of both renders.
# $master_checkout is the checkout that the scene's master render was built against.
compose() {
    "$bin/compose" "$1" "$roboto" "out/$2-master.png" "master ($(git -C "femtovg/$master_checkout" rev-parse --short=7 HEAD))" "out/$2-fixed.png" "with the fix" "$3" "$4" "$5" "$6" "out/$2-$7.png"
}

master_checkout=master
draw about "$roboto"
# The whole panel.
compose side about 0 0 392 608 side-by-side
# The left 200 px of the four lines of the licence notice.
compose zoom about 56 484 200 68 zoom

if [ "$scenes" = "about menu" ]; then
    draw menu "$NEW_YORK_DIR/NewYork.ttf" "$NEW_YORK_DIR/NewYorkItalic.ttf"
    # The title and the first three of the four courses. The whole card is 912 px tall.
    compose side menu 0 0 392 680 side-by-side
    # The third and fourth dish of the first course.
    compose zoom menu 64 218 200 74 zoom
fi

# The specimen prints one row per sample, from what fill_text returned. The two revisions lay
# the samples out differently, so the summary has the rows of both.
for revision in master fixed; do
    echo "spacing-$revision:"
    "$bin/spacing-$revision" "$roboto" "out/spacing-$revision.png"
done >out/spacing-summary.txt
echo "spacing-master.png and spacing-fixed.png: $("$bin/compare" out/spacing-master.png out/spacing-fixed.png)" >>out/spacing-summary.txt
master_checkout=spacing-master
# The whole specimen.
compose side spacing 0 0 392 428 side-by-side

# The gradient scenes print nothing, so their summaries have only the pixel comparison.
for scene in text shadow; do
    for revision in master fixed; do
        "$bin/gradient-$revision" "$roboto" "$scene" "out/gradient-$scene-$revision.png"
    done
    echo "gradient-$scene-master.png and gradient-$scene-fixed.png: $("$bin/compare" "out/gradient-$scene-master.png" "out/gradient-$scene-fixed.png")" >"out/gradient-$scene-summary.txt"
done
master_checkout=gradient-master
# The whole of both scenes.
compose side gradient-text 0 0 720 250 side-by-side
compose side gradient-shadow 0 0 720 250 side-by-side

# The shadow scene is drawn on one revision, once with fill_text and once with fill_glyph_run.
# Without the shadow the two calls draw the same pixels, and the summary has that comparison too.
for call in text run; do
    "$bin/shadow-master" "$roboto" "$call" shadow "out/shadow-$call.png"
    "$bin/shadow-master" "$roboto" "$call" plain "out/shadow-$call-plain.png"
done
{
    echo "without the shadow, fill_text and fill_glyph_run: $("$bin/compare" out/shadow-text-plain.png out/shadow-run-plain.png)"
    echo "shadow-text.png and shadow-run.png: $("$bin/compare" out/shadow-text.png out/shadow-run.png)"
} >out/shadow-summary.txt
shadow_rev=$(git -C femtovg/shadow-master rev-parse --short=7 HEAD)
# The whole scene, with the fill_text render above the fill_glyph_run render.
"$bin/compose" stack "$roboto" out/shadow-text.png "fill_text(), master ($shadow_rev)" out/shadow-run.png "fill_glyph_run(), master ($shadow_rev)" 0 0 540 264 out/shadow-stacked.png
# The a, the d and the o of the first line.
"$bin/compose" zoom "$roboto" out/shadow-text.png "fill_text()" out/shadow-run.png "fill_glyph_run()" 144 24 200 88 out/shadow-zoom.png

# The cut-off scenes are drawn on one revision too. Each is drawn once under a canvas shadow, and
# once with no canvas shadow and the stroke drawn a second time in black where its shadow belongs.
cutoff_rev=$(git -C femtovg/cutoff-master rev-parse --short=7 HEAD)
# cutoff <scene> <side|stack> <w> <h> <output name>: both renders of a scene, whole, in one picture.
cutoff() {
    "$bin/cutoff-master" "$1" moved "out/cutoff-$1-moved.png"
    "$bin/cutoff-master" "$1" shadow "out/cutoff-$1-shadow.png"
    echo "cutoff-$1-moved.png and cutoff-$1-shadow.png: $("$bin/compare" "out/cutoff-$1-moved.png" "out/cutoff-$1-shadow.png")" >"out/cutoff-$1-summary.txt"
    "$bin/compose" "$2" "$roboto" "out/cutoff-$1-moved.png" "stroke drawn twice, master ($cutoff_rev)" "out/cutoff-$1-shadow.png" "canvas shadow, master ($cutoff_rev)" 0 0 "$3" "$4" "out/cutoff-$1-$5.png"
}
cutoff joins stack 520 290 stacked
cutoff caps stack 520 290 stacked
cutoff star side 320 320 side-by-side

echo
for scene in $scenes; do
    echo "$scene, as femtovg/master draws it:"
    sed 's/^/  /' "out/$scene-summary.txt"
done
echo "spacing, as femtovg/spacing-master and femtovg/spacing-fixed draw it:"
sed 's/^/  /' out/spacing-summary.txt
echo "the gradient scenes, as femtovg/gradient-master and femtovg/gradient-fixed draw them:"
sed 's/^/  /' out/gradient-text-summary.txt out/gradient-shadow-summary.txt
echo "the shadow scene, as femtovg/shadow-master draws it:"
sed 's/^/  /' out/shadow-summary.txt
echo "the cut-off scenes, as femtovg/cutoff-master draws them:"
sed 's/^/  /' out/cutoff-joins-summary.txt out/cutoff-caps-summary.txt out/cutoff-star-summary.txt

echo
echo "out/ compared with images/:"
files=0
different=0
# check <file>: compares out/<file> with images/<file> and prints the result.
check() {
    files=$((files + 1))
    if [ "${1##*.}" = png ]; then
        result=$("$bin/compare" "out/$1" "images/$1") || different=$((different + 1))
    elif cmp -s "out/$1" "images/$1"; then
        result=identical
    else
        result=differs
        different=$((different + 1))
    fi
    printf '  %-32s %s\n' "$1" "$result"
}
for scene in $scenes spacing gradient-text gradient-shadow; do
    # Of these scenes, an enlargement is made of the About panel and of the menu card only.
    case $scene in
    about | menu) zoom=$scene-zoom.png ;;
    *) zoom= ;;
    esac
    for file in "$scene-master.png" "$scene-fixed.png" "$scene-side-by-side.png" $zoom "$scene-summary.txt"; do
        check "$file"
    done
done
for file in shadow-text.png shadow-run.png shadow-stacked.png shadow-zoom.png shadow-summary.txt; do
    check "$file"
done
for scene in joins caps star; do
    # The star is one shape, so its two renders fit side by side.
    case $scene in
    star) composed=side-by-side ;;
    *) composed=stacked ;;
    esac
    for file in "cutoff-$scene-moved.png" "cutoff-$scene-shadow.png" "cutoff-$scene-$composed.png" "cutoff-$scene-summary.txt"; do
        check "$file"
    done
done
[ "$scenes" = "about menu" ] || echo "  the menu images were not made (see above)"
if [ "$different" -eq 0 ]; then
    echo "All $files files are identical to the committed ones."
else
    echo "$different of $files files differ from the committed ones. The scenes are drawn on the GPU, so a few"
    echo "differing pixels are expected on another GPU or driver. Look at the pictures in out/."
fi
