#!/bin/sh
# Draws the two scenes on two femtovg revisions, composes the comparison images, and compares
# the result with the files in images/.
#
# usage: ./run.sh
#
# Environment, with the defaults in parentheses:
#   FEMTOVG_URL   where the revision with the bug is fetched from (https://github.com/femtovg/femtovg)
#   MASTER_REV    that revision (eb4fe53, upstream master when this was written)
#   FIXED_URL     where the revision with the fix is fetched from (https://github.com/jfarmer/femtovg)
#   FIXED_REV     that revision (97171be, the commit of the pull request)
#   NEW_YORK_DIR  the directory with NewYork.ttf and NewYorkItalic.ttf (/System/Library/Fonts)
# A URL may be the path of a local clone, and then nothing is downloaded from it. A revision is
# a full commit hash, or the name of a branch or tag.
#
# Needs git, cargo and a GPU that wgpu can use. Everything is written to femtovg/, target/ and
# out/, which git ignores. images/ is only read.
set -eu

FEMTOVG_URL=${FEMTOVG_URL:-https://github.com/femtovg/femtovg}
MASTER_REV=${MASTER_REV:-eb4fe53d51a6a274754a787a55179140bc7d6c77}
FIXED_URL=${FIXED_URL:-https://github.com/jfarmer/femtovg}
FIXED_REV=${FIXED_REV:-97171befe2f7a7afe9e60259ed59a29dd633a5fa}
NEW_YORK_DIR=${NEW_YORK_DIR:-/System/Library/Fonts}

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

# Both builds read the fonts from the same files.
roboto=femtovg/master/examples/assets/RobotoFlex-VariableFont.ttf
[ -f "$roboto" ] || die "$roboto is missing. The About panel and the captions are set in Roboto Flex, which femtovg keeps in examples/assets."
scenes="about menu"
for font in "$NEW_YORK_DIR/NewYork.ttf" "$NEW_YORK_DIR/NewYorkItalic.ttf"; do
    if [ ! -f "$font" ]; then
        echo "run.sh: $font is missing. The menu is set in New York, which comes with macOS; set NEW_YORK_DIR if it is somewhere else. Drawing the About panel only." >&2
        scenes=about
        break
    fi
done

# One target directory for both crates, so that the dependencies are compiled once.
CARGO_TARGET_DIR=$(pwd)/target
export CARGO_TARGET_DIR
for revision in master fixed; do
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
master_caption="master ($(git -C femtovg/master rev-parse --short=7 HEAD))"
compose() {
    "$bin/compose" "$1" "$roboto" "out/$2-master.png" "$master_caption" "out/$2-fixed.png" "with the fix" "$3" "$4" "$5" "$6" "out/$2-$7.png"
}

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

echo
for scene in $scenes; do
    echo "$scene, as femtovg/master draws it:"
    sed 's/^/  /' "out/$scene-summary.txt"
done

echo
echo "out/ compared with images/:"
files=0
different=0
for scene in $scenes; do
    for file in "$scene-master.png" "$scene-fixed.png" "$scene-side-by-side.png" "$scene-zoom.png" "$scene-summary.txt"; do
        files=$((files + 1))
        if [ "${file##*.}" = png ]; then
            result=$("$bin/compare" "out/$file" "images/$file") || different=$((different + 1))
        elif cmp -s "out/$file" "images/$file"; then
            result=identical
        else
            result=differs
            different=$((different + 1))
        fi
        printf '  %-26s %s\n' "$file" "$result"
    done
done
[ "$scenes" = "about menu" ] || echo "  the menu images were not made (see above)"
if [ "$different" -eq 0 ]; then
    echo "All $files files are identical to the committed ones."
else
    echo "$different of $files files differ from the committed ones. The scenes are drawn on the GPU, so a few"
    echo "differing pixels are expected on another GPU or driver. Look at the pictures in out/."
fi
