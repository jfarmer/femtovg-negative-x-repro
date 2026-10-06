//! A letter-spacing specimen: one sentence at six letter spacings, every line right-aligned to
//! the same x, drawn with femtovg on `renderer::WGPURenderer` and saved as a PNG.
//!
//! usage: spacing-<revision> <RobotoFlex-VariableFont.ttf> <out.png>
//!
//! `Specimen::right_aligned_text` draws every line the same way: `fill_text(RIGHT, baseline, ..)`
//! with `Align::Right`. The six samples differ only in `Paint::letter_spacing`, and the sample at
//! letter spacing 0 is drawn first.
//!
//! femtovg keeps shaped text in two caches whose key is a `ShapingId` (src/text/textlayout.rs in
//! femtovg). On a revision where `ShapingId::new` gives every negative letter spacing the key 0,
//! `shape` finds the entry of the first sample for the other five and passes it to `layout`.
//! `layout` subtracts the width in that entry, which was computed with letter spacing 0, from
//! `RIGHT` to get the x of the first glyph. It then places the glyphs with the sample's own
//! letter spacing, so the line ends left of `RIGHT`.
//!
//! The frame is drawn once, top to bottom, at device pixel ratio 1. Every glyph is at a positive
//! x. One row per sample is printed on stdout, computed from the `TextMetrics` that `fill_text`
//! returned.

mod offscreen;

use femtovg::{renderer::WGPURenderer, Align, Canvas, Color, FontId, Paint, Path, TextContext, TextMetrics};
use std::fmt::Write as _;

const WIDTH: u32 = 392;
const HEIGHT: u32 = 428;
/// The x that every line is right-aligned to.
const RIGHT: f32 = 352.0;

const SAMPLE: &str = "Still legible, a little tighter";
const SAMPLE_SIZE: f32 = 28.0;
/// The letter spacing of each sample in pixels, in the order the samples are drawn.
const LETTER_SPACINGS: [f32; 6] = [0.0, -0.5, -1.0, -1.5, -2.0, -2.5];

/// One row of the table on stdout.
struct Row {
    letter_spacing: f32,
    glyphs: usize,
    /// `TextMetrics::width()`: the width that `layout` subtracted from `RIGHT` to get `start`.
    width: f32,
    /// `TextMetrics::x`: where the line starts.
    start: f32,
    /// Where the line ends: the x of the last glyph without its `offset_x`, plus its advance and
    /// the letter spacing. That is where `layout` leaves its cursor after the last glyph.
    end: f32,
}

struct Specimen {
    canvas: Canvas<WGPURenderer>,
    font: FontId,
    rows: Vec<Row>,
}

impl Specimen {
    /// Draws `text` with its right end at `RIGHT` and its baseline at `baseline`.
    fn right_aligned_text(&mut self, baseline: f32, text: &str, paint: &Paint) -> TextMetrics {
        self.canvas.fill_text(RIGHT, baseline, text, paint).expect("fill_text")
    }

    fn text_paint(&self, size: f32, color: Color) -> Paint {
        Paint::color(color)
            .with_font(&[self.font])
            .with_font_size(size)
            .with_text_align(Align::Right)
    }

    fn draw(&mut self) {
        self.canvas.clear_rect(0, 0, WIDTH, HEIGHT, Color::rgb(236, 236, 240));

        let (card_x, card_y, card_w, card_h) = (16.0, 20.0, 360.0, 388.0);
        let mut card = Path::new();
        card.rounded_rect(card_x, card_y, card_w, card_h, 8.0);
        self.canvas.fill_path(&card, &Paint::color(Color::white()));
        let mut border = Path::new();
        border.rounded_rect(card_x + 0.5, card_y + 0.5, card_w - 1.0, card_h - 1.0, 8.0);
        self.canvas
            .stroke_path(&border, &Paint::color(Color::rgb(200, 200, 204)).with_line_width(1.0));

        let ink = Color::rgb(20, 20, 24);
        let grey = Color::rgb(96, 96, 104);
        let heading = self.text_paint(13.0, ink);
        let label = self.text_paint(10.0, grey);
        let sample = self.text_paint(SAMPLE_SIZE, ink);

        let text = format!("Roboto Flex {SAMPLE_SIZE} px, right-aligned");
        self.right_aligned_text(52.0, &text, &heading);

        // A rule beside the right margin, from above the first label to below the last sample.
        // It is 4 px right of RIGHT because the width of a line includes the letter spacing after
        // its last glyph: when that width is right, the advance of the last glyph of the sample
        // at -2.5 px ends at RIGHT + 2.5.
        let mut rule = Path::new();
        rule.rect(RIGHT + 4.0, 68.0, 1.0, 320.0);
        self.canvas.fill_path(&rule, &Paint::color(Color::rgb(226, 80, 64)));

        let mut baseline = 84.0;
        for letter_spacing in LETTER_SPACINGS {
            // U+2212 is the minus sign.
            let text = format!("letter spacing {letter_spacing} px").replace('-', "\u{2212}");
            self.right_aligned_text(baseline, &text, &label);
            let paint = sample.clone().with_letter_spacing(letter_spacing);
            let metrics = self.right_aligned_text(baseline + 30.0, SAMPLE, &paint);
            // Not part of the drawing: keeps what fill_text returned, for the table.
            let last = metrics.glyphs.last().expect("the sample has glyphs");
            self.rows.push(Row {
                letter_spacing,
                glyphs: metrics.glyphs.len(),
                width: metrics.width(),
                start: metrics.x,
                end: last.x - last.offset_x + last.advance_x + letter_spacing,
            });
            baseline += 52.0;
        }
    }

    /// The table that the README quotes.
    fn table(&self) -> String {
        // How far left of RIGHT a line ends, rounded to the 0.001 px that is printed. Adding 0.0
        // turns -0.0 into 0.0.
        let shortfall = |row: &Row| ((RIGHT - row.end) * 1000.0).round() / 1000.0 + 0.0;
        let mut out = String::new();
        writeln!(
            out,
            "letter spacing  glyphs  TextMetrics::width()  starts at x  ends at x  ends left of x = {RIGHT} by"
        )
        .unwrap();
        for row in &self.rows {
            writeln!(
                out,
                "{:>11.1} px  {:>6}  {:>20.3}  {:>11.3}  {:>9.3}  {:>20.3} px",
                row.letter_spacing,
                row.glyphs,
                row.width,
                row.start,
                row.end,
                shortfall(row),
            )
            .unwrap();
        }
        writeln!(
            out,
            "last column, sum over the {} lines: {:.3} px; largest: {:.3} px",
            self.rows.len(),
            self.rows.iter().map(shortfall).sum::<f32>(),
            self.rows.iter().map(shortfall).fold(0.0, f32::max),
        )
        .unwrap();
        out
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, font_path, out_png] = args.as_slice() else {
        eprintln!("usage: {} <RobotoFlex-VariableFont.ttf> <out.png>", args[0]);
        std::process::exit(2);
    };
    let font_data = std::fs::read(font_path).unwrap_or_else(|error| panic!("cannot read {font_path}: {error}"));

    let gpu = offscreen::Gpu::new();
    eprintln!("{}: drawing on {}", args[0], gpu.adapter_name);
    let text_context = TextContext::default();
    let font = text_context.add_font_mem(&font_data).expect("add_font_mem");
    let mut specimen = Specimen {
        canvas: gpu.canvas(text_context, WIDTH, HEIGHT),
        font,
        rows: Vec::new(),
    };
    specimen.draw();

    gpu.read_frame(&mut specimen.canvas, WIDTH, HEIGHT)
        .save(out_png)
        .unwrap_or_else(|error| panic!("cannot write {out_png}: {error}"));
    print!("{}", specimen.table());
}
