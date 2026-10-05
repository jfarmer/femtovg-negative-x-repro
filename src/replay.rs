//! Computes where femtovg master draws each glyph of a frame, from the glyph positions that
//! `Canvas::fill_text` returned.
//!
//! `GlyphAtlas::render_atlas` in src/text.rs keeps one rasterized mask per glyph and per
//! `subpixel_location as u8`, where
//!
//! ```text
//! let subpixel_location = crate::geometry::quantize(glyph.x.fract(), 0.1) * 10.0;
//! ```
//!
//! The mask is rasterized with the outline moved right by `subpixel_location / 10.0` px and is
//! drawn with its origin at `glyph.x.trunc()`. For a negative `glyph.x` the fraction is negative
//! and the cast to `u8` gives 0. Every negative x of a glyph therefore has key 0, which is also
//! the key of the glyph on a whole pixel, and all of them are drawn with the mask of whichever
//! was drawn first.
//!
//! `Replay` applies that rule to the glyphs in the order they were drawn. It uses only the
//! positions, so the result is the same whichever femtovg revision the scene is built against:
//! it describes master, also when the fixed build prints it.
//!
//! The real cache key (`RenderedGlyphId`) also holds the line width, the render mode and a hash
//! of the font variation settings. They are left out here because in both scenes two paints that
//! have the same font and font size never differ in any of them.

use femtovg::{FontId, TextMetrics};
use std::collections::HashMap;
use std::fmt::Write as _;

/// `geometry::quantize` in femtovg.
fn quantize(a: f32, d: f32) -> f32 {
    (a / d + 0.5).trunc() * d
}

/// The largest absolute value.
fn largest(values: impl Iterator<Item = f32>) -> f32 {
    values.fold(0.0, |largest, value| largest.max(value.abs()))
}

struct Glyph {
    line: usize,
    c: char,
    glyph_id: u16,
    font_size: f32,
    /// The x `render_atlas` receives: relative to the origin of the `fill_text` call, before the
    /// canvas translation.
    x: f32,
    key: u8,
    /// How far right of `x.trunc()` the outline is in the mask that master draws for this glyph.
    mask_offset: f32,
    /// `x.trunc() + mask_offset - x`: how far right (+) or left (-) of its place master draws
    /// the glyph.
    error: f32,
    /// `error` minus the `error` of the glyph drawn before this one on the same line: how much
    /// wider (+) or narrower (-) than intended the gap between the two is. `None` for the first
    /// glyph of a line.
    gap_error: Option<f32>,
    /// Whether no space separates this glyph from the one before it.
    in_word: bool,
}

#[derive(Default)]
pub struct Replay {
    /// (font, `RenderedGlyphId::size`, glyph id, `subpixel_location as u8`) -> the offset the
    /// mask under that key was rasterized at.
    cache: HashMap<(FontId, u32, u16, u8), f32>,
    lines: Vec<String>,
    glyphs: Vec<Glyph>,
}

impl Replay {
    /// Adds the glyphs of one `fill_text` call. `metrics` is what the call returned.
    pub fn record(&mut self, text: &str, font_size: f32, metrics: &TextMetrics) {
        let line = self.lines.len();
        self.lines.push(text.to_owned());
        let mut previous_error = None;
        let mut after_space = false;
        for glyph in &metrics.glyphs {
            if glyph.c.is_whitespace() {
                after_space = true;
                continue;
            }
            let subpixel_location = quantize(glyph.x.fract(), 0.1) * 10.0;
            let key = subpixel_location as u8;
            let size_key = (font_size * 10.0).trunc() as u32;
            let mask_offset = *self
                .cache
                .entry((glyph.font_id, size_key, glyph.glyph_id, key))
                .or_insert(subpixel_location / 10.0);
            let error = glyph.x.trunc() + mask_offset - glyph.x;
            self.glyphs.push(Glyph {
                line,
                c: glyph.c,
                glyph_id: glyph.glyph_id,
                font_size,
                x: glyph.x,
                key,
                mask_offset,
                error,
                gap_error: previous_error.map(|previous| error - previous),
                in_word: previous_error.is_some() && !after_space,
            });
            previous_error = Some(error);
            after_space = false;
        }
    }

    /// One row per drawn glyph, tab separated, in draw order. The comments on `Glyph` describe
    /// the columns.
    pub fn table(&self) -> String {
        let mut out = String::from(
            "line\ttext\tchar\tglyph_id\tfont_size\tx\tkey\tmask_offset\terror_px\tgap_error_px\tin_word\n",
        );
        for glyph in &self.glyphs {
            writeln!(
                out,
                "{}\t{}\t{}\t{}\t{}\t{:.3}\t{}\t{:.1}\t{:+.3}\t{}\t{}",
                glyph.line,
                self.lines[glyph.line],
                glyph.c,
                glyph.glyph_id,
                glyph.font_size,
                glyph.x,
                glyph.key,
                glyph.mask_offset,
                glyph.error,
                glyph.gap_error.map_or(String::new(), |gap| format!("{gap:+.3}")),
                u8::from(glyph.in_word),
            )
            .unwrap();
        }
        out
    }

    /// The counts that the README quotes.
    pub fn summary(&self) -> String {
        let pairs = |in_word_only: bool| {
            self.glyphs
                .iter()
                .filter(move |glyph| glyph.in_word || !in_word_only)
                .filter_map(|glyph| glyph.gap_error)
        };
        let mut out = String::new();
        writeln!(out, "glyphs drawn: {}", self.glyphs.len()).unwrap();
        writeln!(
            out,
            "glyphs drawn 0.5 px or more from their place: {} (largest distance {:.3} px)",
            self.glyphs.iter().filter(|glyph| glyph.error.abs() >= 0.5).count(),
            largest(self.glyphs.iter().map(|glyph| glyph.error)),
        )
        .unwrap();
        for (label, in_word_only) in [
            ("adjacent pairs of drawn glyphs", false),
            ("of those, pairs with no space between", true),
        ] {
            writeln!(
                out,
                "{label}: {}; gap wrong by 1 px or more: {}; largest gap error {:.3} px",
                pairs(in_word_only).count(),
                pairs(in_word_only).filter(|gap| gap.abs() >= 1.0).count(),
                largest(pairs(in_word_only)),
            )
            .unwrap();
        }
        out
    }
}
