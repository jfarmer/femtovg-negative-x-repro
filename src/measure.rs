//! Prints the glyph positions of centred lines of text without drawing them. The scripts in
//! search/ use it to try content.
//!
//! usage: measure <font file> <font size> [<axis>=<value>...] < lines.txt
//!
//! Every line of stdin is laid out the way the scenes lay text out: at (0, 0) with
//! `Align::Center`. One row is printed per glyph, tab separated: the number of the line, the
//! glyph id, x, and 0 for whitespace or 1 for a glyph that is drawn. x is printed with enough
//! digits to read back the same `f32`.

use femtovg::{Align, Color, Paint, TextContext};
use std::fmt::Write as _;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "usage: {} <font file> <font size> [<axis>=<value>...] < lines.txt",
            args[0]
        );
        std::process::exit(2);
    }
    let font_data = std::fs::read(&args[1]).unwrap_or_else(|error| panic!("cannot read {}: {error}", args[1]));
    let text_context = TextContext::default();
    let font = text_context.add_font_mem(&font_data).expect("add_font_mem");
    let mut paint = Paint::color(Color::black())
        .with_font(&[font])
        .with_font_size(args[2].parse().expect("font size"))
        .with_text_align(Align::Center);
    for setting in &args[3..] {
        let (axis, value) = setting.split_once('=').expect("<axis>=<value>");
        let axis: &[u8; 4] = axis.as_bytes().try_into().expect("an axis tag has four letters");
        paint.set_font_variation(axis, value.parse().expect("axis value"));
    }

    let mut out = String::new();
    for (number, line) in std::io::stdin().lines().enumerate() {
        let line = line.expect("stdin");
        let metrics = text_context
            .measure_text(0.0, 0.0, &line, &paint)
            .expect("measure_text");
        for glyph in &metrics.glyphs {
            let drawn = u8::from(!glyph.c.is_whitespace());
            writeln!(out, "{number}\t{}\t{}\t{drawn}", glyph.glyph_id, glyph.x).unwrap();
        }
    }
    print!("{out}");
}
