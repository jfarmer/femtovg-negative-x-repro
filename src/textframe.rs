//! Per-frame CPU cost of drawing the text of a Slint-like UI frame: LABELS labels (default 200) of 10 to 40
//! characters at 12 to 16 px, each under its own save/translate/restore, drawn every frame with a warm glyph atlas.
//!
//!   textframe-<master|fixed> <mode> <frames> [void|gl]
//!
//! Modes:
//!   glyph_run  fill_glyph_run with glyph ids and positions shaped once at startup (Slint's path; build with
//!              `--no-default-features --features swash` to match Slint's femtovg features)
//!   fill_text  fill_text with femtovg's own shaping and caches (needs the default `textlayout` feature)
//!
//! Environment: FONT (a .ttf path), LABELS (label count), SCISSOR=1 (intersect a scissor around each label),
//! SAME_PAINT=1 (reuse one Paint per label instead of building it per draw as Slint does), COORDS (comma-separated
//! normalized variation coordinates for glyph_run, e.g. 0,0,0 for a variable font), EXACT_SIZE_HINT=1 (pass a
//! slice iterator; by default glyph_run passes an iterator whose size_hint is (0, None), like the flat_map behind
//! parley's positioned_glyphs()), ONE_RUN=1 (glyph_run only: every label's glyphs in one fill_glyph_run call with
//! one paint, an upper bound for what merging the labels' draw calls could save), SAME_COLOR=1 (every label in
//! one color), PIXEL_HASH=1 (gl only: print a hash of the image target after the last frame).
//!
//! The `void` renderer discards the commands, so it measures femtovg's own work up to the renderer: shaping and
//! cache lookups, atlas lookups, quad and vertex building, command recording and the end-of-frame bookkeeping.
//! `gl` adds the OpenGL backend's command processing and the driver's CPU work, drawing into a 1920 x 1080 image
//! target of a hidden window. It prints the mean frame time and the median, p10 and p90 of the frame times.
use std::time::Instant;

use femtovg::{renderer::Void, Canvas, Color, FontId, Paint, PositionedGlyph, Renderer, TextContext};

const W: u32 = 1920;
const H: u32 = 1080;

const WORDS: &[&str] = &[
    "Settings", "Network", "Volume", "Brightness", "Display", "the", "of", "and", "Bluetooth", "Battery", "Save",
    "Cancel", "Apply", "Open", "file", "folder", "Recent", "items", "Wi-Fi", "connected", "Account", "Profile",
    "Language", "Region", "Date", "Time", "Updates", "available", "Search", "results", "Notifications", "enabled",
    "Privacy", "Security", "Storage", "used", "of", "128", "GB", "Printer", "ready", "Temperature", "21.5", "°C",
];

struct Label {
    x: f32,
    y: f32,
    size: f32,
    // Only fill_text reads the text. glyph_run uses the glyphs shaped from it at startup.
    #[cfg_attr(not(feature = "textlayout"), allow(dead_code))]
    text: String,
    glyphs: Vec<PositionedGlyph>,
    color: Color,
}

fn labels(font_data: &[u8], count: usize) -> Vec<Label> {
    let font_ref = swash::FontRef::from_index(font_data, 0).expect("FONT is no font swash can read");
    let charmap = font_ref.charmap();
    let mut seed: u32 = 12345;
    let mut next = move || {
        seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
        seed >> 8
    };
    (0..count)
        .map(|i| {
            let target_len = 10 + (next() % 31) as usize;
            let mut text = String::new();
            while text.chars().count() < target_len {
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(WORDS[next() as usize % WORDS.len()]);
            }
            let text: String = text.chars().take(target_len).collect::<String>().trim_end().to_owned();
            let size = 12.0 + (i % 5) as f32;
            let metrics = font_ref.glyph_metrics(&[]).scale(size);
            let mut pen = 0.0;
            let glyphs = text
                .chars()
                .map(|c| {
                    let glyph_id = charmap.map(c);
                    let glyph = PositionedGlyph { x: pen, y: size, glyph_id };
                    pen += metrics.advance_width(glyph_id);
                    glyph
                })
                .collect();
            Label {
                x: 8.0 + (i % 4) as f32 * 470.0,
                y: 8.0 + (i / 4) as f32 * 21.0,
                size,
                text,
                glyphs,
                color: if std::env::var("SAME_COLOR").is_ok_and(|v| v == "1") {
                    Color::rgb(30, 30, 30)
                } else {
                    Color::rgb((i * 37 % 200) as u8, (i * 11 % 200) as u8, 60)
                },
            }
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    GlyphRun,
    FillText,
}

struct Bench {
    mode: Mode,
    labels: Vec<Label>,
    font_id: FontId,
    scissor: bool,
    same_paint: bool,
    coords: Vec<i16>,
    exact_size_hint: bool,
    one_run: Vec<PositionedGlyph>,
    paints: Vec<Paint>,
}

impl Bench {
    // Registers the font the way Slint's font cache does, on a TextContext the canvas is created with.
    fn new(text_context: &TextContext, mode: Mode) -> Self {
        let path = std::env::var("FONT")
            .unwrap_or_else(|_| "fonts/Roboto-Regular.ttf".to_owned());
        let data = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let font_id = text_context.add_shared_font_with_index(data.clone(), 0).unwrap();
        let count = std::env::var("LABELS").ok().and_then(|v| v.parse().ok()).unwrap_or(200);
        let labels = labels(&data, count);
        let paints = labels
            .iter()
            .map(|label| {
                let mut paint = Paint::color(label.color);
                paint.set_font(&[font_id]);
                paint.set_font_size(label.size);
                paint
            })
            .collect();
        let one_run = if std::env::var("ONE_RUN").is_ok_and(|v| v == "1") {
            labels
                .iter()
                .flat_map(|label| {
                    label.glyphs.iter().map(|glyph| PositionedGlyph {
                        x: glyph.x + label.x,
                        y: glyph.y + label.y,
                        glyph_id: glyph.glyph_id,
                    })
                })
                .collect()
        } else {
            Vec::new()
        };
        Self {
            mode,
            labels,
            font_id,
            scissor: std::env::var("SCISSOR").is_ok_and(|v| v == "1"),
            same_paint: std::env::var("SAME_PAINT").is_ok_and(|v| v == "1"),
            coords: std::env::var("COORDS")
                .map(|v| v.split(',').map(|c| c.trim().parse().unwrap()).collect())
                .unwrap_or_default(),
            exact_size_hint: std::env::var("EXACT_SIZE_HINT").is_ok_and(|v| v == "1"),
            one_run,
            paints,
        }
    }

    fn draw<T: Renderer>(&self, canvas: &mut Canvas<T>) {
        canvas.clear_rect(0, 0, W, H, Color::white());
        if !self.one_run.is_empty() {
            let mut paint = Paint::color(self.labels[0].color);
            paint.set_font_size(14.0);
            canvas
                .fill_glyph_run(self.font_id, &self.coords, self.one_run.iter().cloned(), &paint)
                .unwrap();
            return;
        }
        for (label, kept_paint) in self.labels.iter().zip(&self.paints) {
            canvas.save();
            canvas.translate(label.x, label.y);
            if self.scissor {
                canvas.intersect_scissor(0.0, 0.0, 460.0, 20.0);
            }
            let built;
            let paint = if self.same_paint {
                kept_paint
            } else {
                let mut paint = Paint::color(label.color);
                paint.set_font(&[self.font_id]);
                paint.set_font_size(label.size);
                built = paint;
                &built
            };
            match self.mode {
                Mode::GlyphRun if self.exact_size_hint => canvas
                    .fill_glyph_run(self.font_id, &self.coords, label.glyphs.iter().cloned(), paint)
                    .unwrap(),
                Mode::GlyphRun => canvas
                    .fill_glyph_run(
                        self.font_id,
                        &self.coords,
                        label.glyphs.iter().cloned().flat_map(std::iter::once),
                        paint,
                    )
                    .unwrap(),
                #[cfg(feature = "textlayout")]
                Mode::FillText => {
                    canvas.fill_text(0.0, label.size, &label.text, paint).unwrap();
                }
                #[cfg(not(feature = "textlayout"))]
                Mode::FillText => unreachable!(),
            }
            canvas.restore();
        }
    }
}

fn parse_args() -> (Mode, usize, String) {
    let args: Vec<String> = std::env::args().collect();
    let mode = match args.get(1).map(String::as_str) {
        Some("glyph_run") => Mode::GlyphRun,
        Some("fill_text") if cfg!(feature = "textlayout") => Mode::FillText,
        Some("fill_text") => panic!("fill_text needs the textlayout feature"),
        _ => panic!("usage: textframe-<master|fixed> glyph_run|fill_text <frames> [void|gl]"),
    };
    let frames = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(1000);
    let renderer = args.get(3).cloned().unwrap_or_else(|| "void".to_owned());
    (mode, frames, renderer)
}

fn report(times: &mut [f64], total: f64, frames: usize) {
    times.sort_by(f64::total_cmp);
    let pick = |q: f64| times[((times.len() - 1) as f64 * q) as usize] * 1e6;
    println!(
        "frames={frames} mean_us={:.2} median_us={:.2} p10_us={:.2} p90_us={:.2}",
        total * 1e6 / frames as f64,
        pick(0.5),
        pick(0.1),
        pick(0.9)
    );
}

fn run<T: Renderer>(
    canvas: &mut Canvas<T>,
    text_context: &TextContext,
    mode: Mode,
    frames: usize,
    flush: impl Fn(&mut Canvas<T>),
) {
    let bench = Bench::new(text_context, mode);
    // Two frames fill the atlas and the shaping caches.
    for _ in 0..2 {
        bench.draw(canvas);
        flush(canvas);
    }
    let mut times = Vec::with_capacity(frames);
    let started = Instant::now();
    for _ in 0..frames {
        let t = Instant::now();
        bench.draw(canvas);
        flush(canvas);
        times.push(t.elapsed().as_secs_f64());
    }
    let total = started.elapsed().as_secs_f64();
    if frames > 0 {
        report(&mut times, total, frames);
    }
}

fn main() {
    let (mode, frames, renderer) = parse_args();
    match renderer.as_str() {
        "void" => {
            let text_context = TextContext::default();
            let mut canvas = Canvas::new_with_text_context(Void, text_context.clone()).unwrap();
            canvas.set_size(W, H, 1.0);
            run(&mut canvas, &text_context, mode, frames, |canvas| canvas.flush_to_output(()));
        }
        #[cfg(not(target_arch = "wasm32"))]
        "gl" => gl::run_gl(mode, frames),
        other => panic!("unknown renderer {other}"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod gl {
    use std::num::NonZeroU32;

    use femtovg::{renderer::OpenGl, Canvas, ImageFlags, PixelFormat, RenderTarget};
    use glutin::{
        config::ConfigTemplateBuilder,
        context::{ContextApi, ContextAttributesBuilder},
        display::GetGlDisplay,
        prelude::*,
        surface::SurfaceAttributesBuilder,
    };
    use glutin_winit::DisplayBuilder;
    use raw_window_handle::HasWindowHandle;
    use winit::application::ApplicationHandler;
    use winit::event::WindowEvent;
    use winit::event_loop::{ActiveEventLoop, EventLoop};
    use winit::window::Window;

    use super::{Mode, H, W};

    struct App {
        mode: Mode,
        frames: usize,
        done: bool,
    }

    impl ApplicationHandler for App {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.done {
                return;
            }
            self.done = true;
            let window_attrs = Window::default_attributes()
                .with_inner_size(winit::dpi::PhysicalSize::new(64, 64))
                .with_visible(false);
            let template = ConfigTemplateBuilder::new().with_alpha_size(8).with_stencil_size(8);
            let (window, gl_config) = DisplayBuilder::new()
                .with_window_attributes(Some(window_attrs))
                .build(event_loop, template, |mut configs| configs.next().unwrap())
                .unwrap();
            let window = window.unwrap();
            let raw = window.window_handle().unwrap().as_raw();
            let gl_display = gl_config.display();
            let attributes = ContextAttributesBuilder::new().build(Some(raw));
            let fallback = ContextAttributesBuilder::new()
                .with_context_api(ContextApi::Gles(None))
                .build(Some(raw));
            let context = unsafe {
                gl_display
                    .create_context(&gl_config, &attributes)
                    .unwrap_or_else(|_| gl_display.create_context(&gl_config, &fallback).unwrap())
            };
            let attrs = SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new().build(
                raw,
                NonZeroU32::new(64).unwrap(),
                NonZeroU32::new(64).unwrap(),
            );
            let surface = unsafe { gl_display.create_window_surface(&gl_config, &attrs).unwrap() };
            let _context = context.make_current(&surface).unwrap();
            let renderer =
                unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s).cast()) }.unwrap();
            let text_context = femtovg::TextContext::default();
            let mut canvas = Canvas::new_with_text_context(renderer, text_context.clone()).unwrap();
            canvas.set_size(W, H, 1.0);
            let image = canvas
                .create_image_empty(W as usize, H as usize, PixelFormat::Rgba8, ImageFlags::empty())
                .unwrap();
            canvas.set_render_target(RenderTarget::Image(image));
            super::run(&mut canvas, &text_context, self.mode, self.frames, |canvas: &mut Canvas<OpenGl>| {
                canvas.flush_to_output(());
            });
            // Wait for the GPU so that a run's instruction count includes the driver work its frames queued.
            let shot = canvas.screenshot().unwrap();
            if std::env::var("PIXEL_HASH").is_ok_and(|v| v == "1") {
                let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
                let mut inked = 0usize;
                for p in shot.pixels() {
                    for byte in [p.r, p.g, p.b, p.a] {
                        hash = (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3);
                    }
                    inked += usize::from(p.r < 250 || p.g < 250 || p.b < 250);
                }
                println!("pixels={}x{} hash={hash:016x} non_white={inked}", shot.width(), shot.height());
            }
            event_loop.exit();
        }

        fn window_event(&mut self, _: &ActiveEventLoop, _: winit::window::WindowId, _: WindowEvent) {}
    }

    pub(super) fn run_gl(mode: Mode, frames: usize) {
        let event_loop = EventLoop::new().unwrap();
        event_loop
            .run_app(&mut App {
                mode,
                frames,
                done: false,
            })
            .unwrap();
    }
}
