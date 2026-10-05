"""Shared by the two search scripts: glyph positions from the `measure` program, and the cache-key
rule of femtovg master applied to them without drawing anything.

The rule is the one src/replay.rs describes. The arithmetic is done in single precision, as
`GlyphAtlas::render_atlas` does it, so that a gap error next to a threshold is counted the same
way here and there.

run.sh builds target/release/measure and checks out the femtovg revision that has Roboto Flex.
"""
import math
import os
import struct
import subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MEASURE = os.path.join(ROOT, "target", "release", "measure")
ROBOTO_FLEX = os.path.join(ROOT, "femtovg", "master", "examples", "assets", "RobotoFlex-VariableFont.ttf")

_single = struct.Struct("f")


def f32(value):
    """`value` rounded to single precision."""
    return _single.unpack(_single.pack(value))[0]


def measure(font, size, variations, lines):
    """Lays out each line centred on x = 0. Returns, for each line, a list of
    (glyph id, x, whether the glyph is drawn) in the order of the glyphs."""
    out = subprocess.run([MEASURE, font, str(size), *variations], input="\n".join(lines) + "\n",
                         capture_output=True, text=True, check=True).stdout
    glyphs = [[] for _ in lines]
    for row in out.splitlines():
        number, glyph_id, x, drawn = row.split("\t")
        glyphs[int(number)].append((int(glyph_id), float(x), drawn == "1"))
    return glyphs


def prepare(glyphs):
    """What `gaps` needs of each drawn glyph of a line: (cache key, the offset the mask is
    rasterized at if this glyph is the first to use the key, trunc(x), x, whether a space comes
    before the glyph). All lines given to one cache must be in the same font and size."""
    out, after_space = [], False
    for glyph_id, x, drawn in glyphs:
        if not drawn:
            after_space = True
            continue
        # geometry::quantize(x.fract(), 0.1) * 10.0
        tenths = math.trunc(f32(f32(math.fmod(x, 1.0) / f32(0.1)) + 0.5))
        subpixel_location = f32(f32(tenths * f32(0.1)) * 10.0)
        # `subpixel_location as u8` is 0 for a negative value.
        key = glyph_id * 16 + max(0, int(subpixel_location))
        out.append((key, f32(subpixel_location / 10.0), float(math.trunc(x)), x, after_space))
        after_space = False
    return out


def gaps(line, cache):
    """Draws one prepared line with the masks in `cache`, adding the masks it is first to use.
    Returns (gap error, whether a space is between the two) for each pair of neighbouring drawn
    glyphs. The gap error is how much wider (+) or narrower (-) than intended master draws the
    gap, in pixels."""
    out, previous = [], None
    for key, offset, whole, x, after_space in line:
        offset = cache.setdefault(key, offset)
        error = f32(f32(whole + offset) - x)
        if previous is not None:
            out.append((f32(error - previous), after_space))
        previous = error
    return out
