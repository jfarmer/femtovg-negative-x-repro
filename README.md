# femtovg: with the swash feature, filled text at a negative x is drawn with the wrong glyph mask

The programs that made the two comparison images for a pull request to
[femtovg](https://github.com/femtovg/femtovg).

Left: upstream master at
[`eb4fe53`](https://github.com/femtovg/femtovg/commit/eb4fe53d51a6a274754a787a55179140bc7d6c77).
Right: the same program built against the commit of the pull request. Both are drawn at device
pixel ratio 1 and nothing is rescaled. The typeface, the font size and the content of both
scenes were chosen by a search to make the effect as large as possible (see below).

An About panel, Roboto Flex 10 px:

![About panel on master and with the fix](images/about-side-by-side.png)

A menu card, New York 12 px (the top 680 px of a 912 px render):

![Menu card on master and with the fix](images/menu-side-by-side.png)

`images/` also has each whole render and a 4x enlargement of part of each scene.

## The bug

With the `swash` cargo feature, femtovg rasterizes a filled glyph into a coverage bitmap (a
mask), keeps it in the glyph atlas and reuses it. It keeps one mask per tenth of a pixel of the
fraction of the glyph's x. `GlyphAtlas::render_atlas` in `src/text.rs` computes that tenth as
`quantize(glyph.x.fract(), 0.1) * 10.0` and casts it to `u8` for the cache key. For a negative
x the value is zero or negative, and the cast gives 0. So every negative x of a glyph shares
one cache entry with the glyph's whole-pixel positions, and the glyph is drawn with whichever
mask was rasterized first, up to 1 px from its place. Neighbouring glyphs get different errors,
so the gap between two letters can be wrong by almost 2 px.

`glyph.x` does not include the canvas translation. Both scenes draw every line like this, which
gives the left half of the line a negative x:

```rust
canvas.save();
canvas.translate(cx, baseline);
canvas.fill_text(0.0, 0.0, text, &paint)?; // paint has Align::Center
canvas.restore();
```

## What was chosen to make the effect large

[`search/about_names.py`](search/about_names.py) chose the 14 names and their order.
[`search/menu_climb.py`](search/menu_climb.py) chose which dishes each course lists, their
order, each price within 2 of a base price, and the order of the ingredients. Of the typefaces
and sizes tried, the one with the most wrong gaps was kept for each scene.

The search does not make any single error larger. It raises the number of neighbouring glyphs
that get different errors:

| gaps between neighbouring glyphs wrong by 1 px or more | searched | 1000 random choices: mean (range) |
| ------------------------------------------------------ | -------- | --------------------------------- |
| About panel (398 gaps)                                 | 78       | 17.6 (6 to 36)                    |
| Menu card (897 gaps)                                   | 119      | 32.9 (10 to 70)                   |

[`src/replay.rs`](src/replay.rs) computes these counts from the glyph positions that
`fill_text` returned. They are not measured from pixels. The random choices were counted, not
rendered.

## Running it

```sh
./run.sh
```

This needs git, cargo, a GPU that wgpu can use, and network access. The script checks out two
femtovg revisions, builds the scenes against each, draws and composes the images into `out/`,
and reports how many pixels differ from the files in `images/`. The committed images were made
on macOS 26 with wgpu on Metal. Another GPU or driver may give a few different pixels.

| variable      | default                                    | meaning                               |
| ------------- | ------------------------------------------ | ------------------------------------- |
| `FEMTOVG_URL` | `https://github.com/femtovg/femtovg`       | where the revision with the bug is    |
| `MASTER_REV`  | `eb4fe53d51a6a274754a787a55179140bc7d6c77` | upstream master when this was written |
| `FIXED_URL`   | `https://github.com/jfarmer/femtovg`       | where the revision with the fix is    |
| `FIXED_REV`   | `97171befe2f7a7afe9e60259ed59a29dd633a5fa` | the commit of the pull request        |

A URL may be the path of a local clone. A revision is a full commit hash, or the name of a
branch or tag. If the pull request has changed since this was written, run
`FIXED_REV=glyph-mask-phase-keys ./run.sh`.

No font file is in this repository. Roboto Flex is read from femtovg's `examples/assets`.
New York is an Apple system font, read from `/System/Library/Fonts` or from `NEW_YORK_DIR`.
Where it is missing, only the About panel is drawn.
