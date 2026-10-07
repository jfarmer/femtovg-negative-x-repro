//! Text filled with a gradient, drawn with femtovg on `renderer::WGPURenderer` and saved as a PNG.
//!
//! usage: gradient-<revision> <RobotoFlex-VariableFont.ttf> text|shadow <out.png>
//!
//! femtovg draws a glyph from the glyph atlas or as an outline. `draw_glyph_run` in femtovg's
//! `src/lib.rs` chooses the outline when the font size is above 92 px, and then
//! `text::render_direct` fills the outline of each glyph as a path. Every line here is drawn at
//! device pixel ratio 1 with no canvas transform, so only the font size decides.
//!
//! `text` draws the same words twice with one linear gradient that runs from the left edge of
//! the canvas to the right. The first line is 90 px and comes from the glyph atlas. The second is
//! 100 px and is drawn as outlines.
//!
//! `shadow` draws one 100 px line with a gradient that goes from opaque to transparent, under a
//! canvas shadow. A shadow has the shadow colour and the alpha of what was drawn, so it fades the
//! way the text does.

mod offscreen;

use femtovg::{renderer::WGPURenderer, Align, Baseline, Canvas, Color, FontId, Paint, TextContext};

const WIDTH: u32 = 720;
const HEIGHT: u32 = 250;
const WORDS: &str = "Gradient text";
/// The largest font size that `draw_glyph_run` draws from the glyph atlas is 92 px. One line is
/// set just below it and the others just above.
const ATLAS_SIZE: f32 = 90.0;
const OUTLINE_SIZE: f32 = 100.0;
/// The gradient starts and ends 40 px inside the canvas, which is about where the lines do.
const GRADIENT_START: f32 = 40.0;
const GRADIENT_END: f32 = WIDTH as f32 - 40.0;
/// The shadow is drawn this far below the text, which is further than the line is tall, so the
/// two do not overlap.
const SHADOW_OFFSET_Y: f32 = 110.0;

/// A horizontal gradient from red at `GRADIENT_START` to `end` at `GRADIENT_END`, for text
/// centred on the x and y it is drawn at.
fn gradient(font: FontId, end: Color) -> Paint {
    let red = Color::rgb(230, 40, 40);
    Paint::linear_gradient(GRADIENT_START, 0.0, GRADIENT_END, 0.0, red, end)
        .with_font(&[font])
        .with_text_align(Align::Center)
        .with_text_baseline(Baseline::Middle)
}

fn draw_text(canvas: &mut Canvas<WGPURenderer>, font: FontId) {
    let centre = WIDTH as f32 / 2.0;
    let blue = Color::rgb(40, 60, 230);
    let paint = gradient(font, blue);
    canvas
        .fill_text(centre, 65.0, WORDS, &paint.clone().with_font_size(ATLAS_SIZE))
        .expect("fill_text");
    canvas
        .fill_text(centre, 180.0, WORDS, &paint.with_font_size(OUTLINE_SIZE))
        .expect("fill_text");
}

fn draw_shadow(canvas: &mut Canvas<WGPURenderer>, font: FontId) {
    let transparent_red = Color::rgba(230, 40, 40, 0);
    let paint = gradient(font, transparent_red).with_font_size(OUTLINE_SIZE);
    canvas.set_shadow_color(Color::black());
    canvas.set_shadow_offset(0.0, SHADOW_OFFSET_Y);
    canvas
        .fill_text(WIDTH as f32 / 2.0, 65.0, WORDS, &paint)
        .expect("fill_text");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, font_path, scene, out_png] = args.as_slice() else {
        eprintln!("usage: {} <RobotoFlex-VariableFont.ttf> text|shadow <out.png>", args[0]);
        std::process::exit(2);
    };
    let font_data = std::fs::read(font_path).unwrap_or_else(|error| panic!("cannot read {font_path}: {error}"));

    let gpu = offscreen::Gpu::new();
    eprintln!("{}: drawing on {}", args[0], gpu.adapter_name);
    let text_context = TextContext::default();
    let font = text_context.add_font_mem(&font_data).expect("add_font_mem");
    let mut canvas = gpu.canvas(text_context, WIDTH, HEIGHT);
    canvas.clear_rect(0, 0, WIDTH, HEIGHT, Color::white());
    match scene.as_str() {
        "text" => draw_text(&mut canvas, font),
        "shadow" => draw_shadow(&mut canvas, font),
        other => panic!("unknown scene {other}"),
    }

    gpu.read_frame(&mut canvas, WIDTH, HEIGHT)
        .save(out_png)
        .unwrap_or_else(|error| panic!("cannot write {out_png}: {error}"));
}
