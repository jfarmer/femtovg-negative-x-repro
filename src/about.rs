//! An About panel in which every line of text is centred, drawn with femtovg on
//! `renderer::WGPURenderer` and saved as a PNG.
//!
//! usage: about-<revision> <RobotoFlex-VariableFont.ttf> <out.png> <glyphs.tsv>
//!
//! `Panel::centered_text` draws every line the same way: it translates the canvas to the centre
//! of the panel and the baseline of the line and calls `fill_text(0.0, 0.0, ..)` with
//! `Align::Center`. femtovg lays the line out from -width / 2 to +width / 2, so the glyphs in the
//! left half of every line have a negative x.
//!
//! The frame is drawn once, top to bottom, at device pixel ratio 1. glyphs.tsv and the lines
//! printed on stdout come from replay.rs.

mod offscreen;
mod replay;

use femtovg::{renderer::WGPURenderer, Align, Canvas, Color, FontId, Paint, Path, TextContext};

const WIDTH: u32 = 392;
const HEIGHT: u32 = 608;

/// The contributors, one per line, in the order they are drawn. search/about_names.py chose
/// them, and chose the version line and the "Developed by" heading from a few alternatives.
const NAMES: &str = include_str!("about-names.txt");

const LICENCE: [&str; 4] = [
    "This program is free software; you can redistribute it and/or",
    "modify it under the terms of the GNU General Public License.",
    "It is distributed in the hope that it will be useful, but",
    "without any warranty.",
];

struct Panel {
    canvas: Canvas<WGPURenderer>,
    font: FontId,
    replay: replay::Replay,
}

impl Panel {
    /// Draws `text` centred on `cx`, with its baseline at `baseline`.
    fn centered_text(&mut self, cx: f32, baseline: f32, text: &str, paint: &Paint) {
        self.canvas.save();
        self.canvas.translate(cx, baseline);
        let metrics = self.canvas.fill_text(0.0, 0.0, text, paint).expect("fill_text");
        self.canvas.restore();
        // Not part of the drawing: keeps the glyph positions fill_text used, for glyphs.tsv.
        self.replay.record(text, paint.font_size(), &metrics);
    }

    fn text_paint(&self, size: f32, color: Color) -> Paint {
        Paint::color(color)
            .with_font(&[self.font])
            .with_font_size(size)
            .with_text_align(Align::Center)
    }

    fn rule(&mut self, cx: f32, y: f32) {
        let mut rule = Path::new();
        rule.rect(cx - 30.0, y, 60.0, 1.0);
        self.canvas.fill_path(&rule, &Paint::color(Color::rgb(214, 214, 220)));
    }

    fn draw(&mut self) {
        self.canvas.clear_rect(0, 0, WIDTH, HEIGHT, Color::rgb(236, 236, 240));

        let (card_x, card_y, card_w, card_h) = (16.0, 20.0, 360.0, 568.0);
        let cx = card_x + card_w / 2.0;
        let mut card = Path::new();
        card.rounded_rect(card_x, card_y, card_w, card_h, 8.0);
        self.canvas.fill_path(&card, &Paint::color(Color::white()));
        let mut border = Path::new();
        border.rounded_rect(card_x + 0.5, card_y + 0.5, card_w - 1.0, card_h - 1.0, 8.0);
        self.canvas
            .stroke_path(&border, &Paint::color(Color::rgb(200, 200, 204)).with_line_width(1.0));

        let mut icon = Path::new();
        icon.rounded_rect(cx - 24.0, 40.0, 48.0, 48.0, 11.0);
        self.canvas.fill_path(&icon, &Paint::color(Color::rgb(28, 78, 160)));

        let ink = Color::rgb(20, 20, 24);
        let grey = Color::rgb(96, 96, 104);
        let name = self.text_paint(20.0, ink);
        let secondary = self.text_paint(10.0, grey);
        let body = self.text_paint(10.0, ink);

        self.centered_text(cx, 116.0, "Carillon", &name);
        self.centered_text(cx, 136.0, "Version 1.11 (1111)", &secondary);
        self.rule(cx, 152.0);

        self.centered_text(cx, 176.0, "Developed by", &secondary);
        let mut baseline = 196.0;
        for contributor in NAMES.lines() {
            self.centered_text(cx, baseline, contributor, &body);
            baseline += 20.0;
        }
        self.rule(cx, 472.0);

        let mut baseline = 496.0;
        for line in LICENCE {
            self.centered_text(cx, baseline, line, &secondary);
            baseline += 16.0;
        }
        self.centered_text(cx, 568.0, "\u{a9} 2026 The Carillon Contributors", &secondary);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, font_path, out_png, out_tsv] = args.as_slice() else {
        eprintln!(
            "usage: {} <RobotoFlex-VariableFont.ttf> <out.png> <glyphs.tsv>",
            args[0]
        );
        std::process::exit(2);
    };
    let font_data = std::fs::read(font_path).unwrap_or_else(|error| panic!("cannot read {font_path}: {error}"));

    let gpu = offscreen::Gpu::new();
    eprintln!("{}: drawing on {}", args[0], gpu.adapter_name);
    let text_context = TextContext::default();
    let font = text_context.add_font_mem(&font_data).expect("add_font_mem");
    let mut panel = Panel {
        canvas: gpu.canvas(text_context, WIDTH, HEIGHT),
        font,
        replay: replay::Replay::default(),
    };
    panel.draw();

    gpu.read_frame(&mut panel.canvas, WIDTH, HEIGHT)
        .save(out_png)
        .unwrap_or_else(|error| panic!("cannot write {out_png}: {error}"));
    std::fs::write(out_tsv, panel.replay.table()).unwrap_or_else(|error| panic!("cannot write {out_tsv}: {error}"));
    print!("{}", panel.replay.summary());
}
