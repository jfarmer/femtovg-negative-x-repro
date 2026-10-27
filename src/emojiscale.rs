//! Two emoji and two letters under a canvas scale, for the blocky-emoji bug.
//!
//! usage: emojiscale-<revision> <emoji font> <text font.ttf> <out.png>
//!
//! The emoji font is Apple Color Emoji, whose glyphs are PNGs. The top half draws the glyphs at
//! 20 px under a 6x scale with a gradient paint. The bottom half draws them with a solid paint,
//! turned by 0.2 radians and scaled 5x. femtovg draws both as outlines (`render_direct()`), and it
//! draws the PNG glyphs of such a run from its ephemeral atlas.

mod offscreen;

use femtovg::{Color, Paint, PositionedGlyph, TextContext};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, emoji_path, text_path, out] = args.as_slice() else {
        panic!("usage: emojiscale <emoji font> <text font.ttf> <out.png>");
    };
    let emoji_data = std::fs::read(emoji_path).expect("cannot read the emoji font");
    let text_data = std::fs::read(text_path).expect("cannot read the text font");
    let emoji_face = ttf_parser::Face::parse(&emoji_data, 0).unwrap();
    let text_face = ttf_parser::Face::parse(&text_data, 0).unwrap();
    let emoji_ids: Vec<u16> = ['😀', '🎉'].iter().map(|c| emoji_face.glyph_index(*c).unwrap().0).collect();
    let text_ids: Vec<u16> = ['A', 'a'].iter().map(|c| text_face.glyph_index(*c).unwrap().0).collect();

    let (width, height) = (720, 560);
    let gpu = offscreen::Gpu::new();
    let text_context = TextContext::default();
    let emoji = text_context.add_shared_font_with_index(emoji_data.clone(), 0).unwrap();
    let text = text_context.add_font_mem(&text_data).unwrap();
    let mut canvas = gpu.canvas(text_context, width, height);
    canvas.clear_rect(0, 0, width, height, Color::white());

    let run = |ids: &[u16], x0: f32, advance: f32| -> Vec<PositionedGlyph> {
        ids.iter().enumerate().map(|(i, id)| PositionedGlyph { x: x0 + i as f32 * advance, y: 20.0, glyph_id: *id }).collect()
    };
    let gradient = Paint::linear_gradient(0.0, 0.0, 100.0, 0.0, Color::rgb(200, 0, 80), Color::rgb(0, 60, 200)).with_font_size(20.0);
    let solid = Paint::color(Color::black()).with_font_size(20.0);

    canvas.save();
    canvas.translate(20.0, 20.0);
    canvas.scale(6.0, 6.0);
    canvas.fill_glyph_run(emoji, &[], run(&emoji_ids, 0.0, 24.0), &gradient).unwrap();
    canvas.fill_glyph_run(text, &[], run(&text_ids, 52.0, 14.0), &gradient).unwrap();
    canvas.restore();

    canvas.save();
    canvas.translate(60.0, 320.0);
    canvas.rotate(-0.2);
    canvas.scale(5.0, 5.0);
    canvas.fill_glyph_run(emoji, &[], run(&emoji_ids, 0.0, 24.0), &solid).unwrap();
    canvas.fill_glyph_run(text, &[], run(&text_ids, 52.0, 14.0), &solid).unwrap();
    canvas.restore();

    gpu.read_frame(&mut canvas, width, height).save(out).expect("cannot write the frame");
}
