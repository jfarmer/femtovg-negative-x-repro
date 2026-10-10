//! Two lines of text under a canvas shadow, drawn with femtovg on `renderer::WGPURenderer` and
//! saved as a PNG.
//!
//! usage: shadow-<revision> <RobotoFlex-VariableFont.ttf> text|run shadow|plain <out.png>
//!
//! `text` draws each line with `fill_text`. `run` draws each line with `fill_glyph_run`, from the
//! glyphs and positions that `measure_text` returns for that line, so both draw every glyph at
//! the same place. `plain` leaves the shadow off, and then both draw the same pixels.
//!
//! The lines are 100 px. `draw_glyph_run` in femtovg's `src/lib.rs` draws text above 92 px as
//! outlines: `text::render_direct` fills the outline of each glyph with `fill_path_internal`,
//! which casts a shadow of what it fills. `draw_text`, which `fill_text` calls, casts one shadow
//! for the line and turns the shadow off while the glyphs are drawn. `fill_glyph_run` calls
//! `draw_glyph_run` directly, so every glyph casts its own shadow, and that shadow is drawn over
//! the glyphs that were drawn before it.

mod offscreen;

use femtovg::{renderer::WGPURenderer, Canvas, Color, FontId, Paint, PositionedGlyph, TextContext};

const WIDTH: u32 = 540;
const HEIGHT: u32 = 264;
/// Most of these letters have a round side, which comes closer to the next letter than a
/// straight stem does.
const LINES: [&str; 2] = ["Shadowed", "typography"];
const BASELINES: [f32; 2] = [100.0, 212.0];
/// The x of the first glyph of each line. The shadow starts 14 px left of it.
const LEFT: f32 = 36.0;
/// Above the 92 px limit of the glyph atlas, so every glyph is drawn as an outline.
const FONT_SIZE: f32 = 100.0;
/// Tighter than the font's own spacing. The closer two letters are, the more of the left one
/// the shadow of the right one covers.
const LETTER_SPACING: f32 = -4.0;
/// The shadow is to the left of the text. A line is drawn from left to right, so a shadow to the
/// left lands on glyphs that are already drawn. A shadow to the right would land where the next
/// glyph is drawn afterwards.
const SHADOW_OFFSET: (f32, f32) = (-14.0, 8.0);
const SHADOW_BLUR: f32 = 6.0;

/// Draws `line` with `fill_glyph_run`, with the glyphs that `fill_text` would draw at the places
/// where it would draw them.
fn fill_run(canvas: &mut Canvas<WGPURenderer>, font: FontId, line: &str, baseline: f32, paint: &Paint) {
    let glyphs: Vec<PositionedGlyph> = canvas
        .measure_text(LEFT, baseline, line, paint)
        .expect("measure_text")
        .glyphs
        .iter()
        .map(|glyph| PositionedGlyph {
            x: glyph.x,
            y: glyph.y,
            glyph_id: glyph.glyph_id,
        })
        .collect();
    // No variation coordinates: the paint sets none either, so both calls use the font's default
    // instance.
    canvas.fill_glyph_run(font, &[], glyphs, paint).expect("fill_glyph_run");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, font_path, call, shadow, out_png] = args.as_slice() else {
        eprintln!(
            "usage: {} <RobotoFlex-VariableFont.ttf> text|run shadow|plain <out.png>",
            args[0]
        );
        std::process::exit(2);
    };
    let glyph_run = match call.as_str() {
        "text" => false,
        "run" => true,
        other => panic!("unknown call {other}"),
    };
    let shadow = match shadow.as_str() {
        "shadow" => true,
        "plain" => false,
        other => panic!("unknown shadow setting {other}"),
    };
    let font_data = std::fs::read(font_path).unwrap_or_else(|error| panic!("cannot read {font_path}: {error}"));

    let gpu = offscreen::Gpu::new();
    eprintln!("{}: drawing on {}", args[0], gpu.adapter_name);
    let text_context = TextContext::default();
    let font = text_context.add_font_mem(&font_data).expect("add_font_mem");
    let mut canvas = gpu.canvas(text_context, WIDTH, HEIGHT);
    canvas.clear_rect(0, 0, WIDTH, HEIGHT, Color::rgb(52, 110, 214));

    let paint = Paint::color(Color::white())
        .with_font(&[font])
        .with_font_size(FONT_SIZE)
        .with_letter_spacing(LETTER_SPACING);
    if shadow {
        canvas.set_shadow_color(Color::black());
        canvas.set_shadow_offset(SHADOW_OFFSET.0, SHADOW_OFFSET.1);
        canvas.set_shadow_blur(SHADOW_BLUR);
    }
    for (line, baseline) in LINES.into_iter().zip(BASELINES) {
        if glyph_run {
            fill_run(&mut canvas, font, line, baseline, &paint);
        } else {
            canvas.fill_text(LEFT, baseline, line, &paint).expect("fill_text");
        }
    }

    gpu.read_frame(&mut canvas, WIDTH, HEIGHT)
        .save(out_png)
        .unwrap_or_else(|error| panic!("cannot write {out_png}: {error}"));
}
