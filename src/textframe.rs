//! Per-frame CPU cost of drawing the text of a Slint-like UI frame, drawn every frame with a warm glyph atlas.
//!
//!   textframe-<master|fixed> <mode> <frames> [void|gl|wgpu] [labels|table]
//!
//! Scenes:
//!   labels  LABELS labels (default 200) of 10 to 40 characters at 12 to 16 px, each under its own
//!           save/translate/restore, at fixed places.
//!   table   a table of 5,000 rows and 8 columns of 13 px text under a header row, scrolled down 7.25 px a frame.
//!           Each frame draws the rows in view (about 48 rows, 390 cells), fills a new rectangle path behind every
//!           other row and clips the rows to the area below the header, the way a Slint ListView of rows is drawn.
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
//! parley's positioned_glyphs()), ONE_RUN=1 (labels with glyph_run only: every label's glyphs in one
//! fill_glyph_run call with one paint, an upper bound for what merging the labels' draw calls could save),
//! SAME_COLOR=1 (every label in one color), EMOJI=1 (every third word of a label is an emoji; with
//! FONT=/System/Library/Fonts/Apple Color Emoji.ttc, whose digits and .notdef glyph are outlines and whose emoji are
//! PNG images, each label then mixes outline and PNG glyphs in one run), STROKE=1 (stroke_glyph_run or stroke_text,
//! with the default 1 px line width, instead of the fill calls), ROTATE=<radians> (rotate each label and table cell
//! by that angle after its translate, so that draw_glyph_run draws it as outlines rather than from the atlas),
//! PIXEL_HASH=1 (gl and wgpu: print a hash of the frame after the last one), SCREENSHOT=<file.ppm> (gl and wgpu:
//! save that frame as a binary PPM).
//!
//! The `void` renderer discards the commands, so it measures femtovg's own work up to the renderer: shaping and
//! cache lookups, atlas lookups, quad and vertex building, command recording and the end-of-frame bookkeeping.
//! `gl` adds the OpenGL backend's command processing and the driver's CPU work, drawing into a 1920 x 1080 image
//! target of a hidden window. `wgpu` draws into a 1920 x 1080 texture with femtovg's WGPU renderer, submits each
//! frame and waits for the GPU to finish it, so a frame's time includes the GPU's work. It prints the mean frame
//! time and the median, p10 and p90 of the frame times.
use std::cell::Cell;
use std::time::Instant;

use femtovg::{renderer::Void, Canvas, Color, FontId, Paint, Path, PositionedGlyph, Renderer, TextContext};

const W: u32 = 1920;
const H: u32 = 1080;

const WORDS: &[&str] = &[
    "Settings", "Network", "Volume", "Brightness", "Display", "the", "of", "and", "Bluetooth", "Battery", "Save",
    "Cancel", "Apply", "Open", "file", "folder", "Recent", "items", "Wi-Fi", "connected", "Account", "Profile",
    "Language", "Region", "Date", "Time", "Updates", "available", "Search", "results", "Notifications", "enabled",
    "Privacy", "Security", "Storage", "used", "of", "128", "GB", "Printer", "ready", "Temperature", "21.5", "°C",
];

const EMOJI: &[&str] = &["😀", "🎉", "👍", "🔋", "📶", "🔒", "📁", "🌐"];

/// Advances the seed of a linear congruential generator and returns its next value, so that every run draws
/// the same labels and table.
fn next(seed: &mut u32) -> u32 {
    *seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
    *seed >> 8
}

/// Places the glyphs of `text` on one line from x = 0 with their baseline at y = `size`, by the font's character
/// map and advances (no kerning or ligatures), once at startup.
fn shape(font_ref: swash::FontRef<'_>, text: &str, size: f32) -> Vec<PositionedGlyph> {
    let charmap = font_ref.charmap();
    let metrics = font_ref.glyph_metrics(&[]).scale(size);
    let mut pen = 0.0;
    text.chars()
        .map(|c| {
            let glyph_id = charmap.map(c);
            let glyph = PositionedGlyph { x: pen, y: size, glyph_id };
            pen += metrics.advance_width(glyph_id);
            glyph
        })
        .collect()
}

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

fn labels(font_ref: swash::FontRef<'_>, count: usize) -> Vec<Label> {
    let mut seed: u32 = 12345;
    let emoji = std::env::var("EMOJI").is_ok_and(|v| v == "1");
    (0..count)
        .map(|i| {
            let target_len = 10 + (next(&mut seed) % 31) as usize;
            let mut text = String::new();
            let mut words = 0;
            while text.chars().count() < target_len {
                if !text.is_empty() {
                    text.push(' ');
                }
                let pick = next(&mut seed) as usize;
                text.push_str(if emoji && words % 3 == 1 {
                    EMOJI[pick % EMOJI.len()]
                } else {
                    WORDS[pick % WORDS.len()]
                });
                words += 1;
            }
            let text: String = text.chars().take(target_len).collect::<String>().trim_end().to_owned();
            let size = 12.0 + (i % 5) as f32;
            let glyphs = shape(font_ref, &text, size);
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

const TABLE_ROWS: usize = 5000;
const TABLE_FONT_SIZE: f32 = 13.0;
const ROW_HEIGHT: f32 = 22.0;
const HEADER_HEIGHT: f32 = 26.0;
/// Pixels the table moves up each frame. Not a whole number, so the rows land at fractional y positions as they
/// do when a list scrolls smoothly.
const SCROLL_STEP: f32 = 7.25;
const COLUMNS: [(&str, f32); 8] = [
    ("Order", 100.0),
    ("Customer", 210.0),
    ("Email", 330.0),
    ("City", 150.0),
    ("Status", 120.0),
    ("Amount", 120.0),
    ("Date", 120.0),
    ("Note", 770.0),
];
const FIRST_NAMES: &[&str] = &[
    "Ada", "Grace", "Alan", "Edsger", "Barbara", "Donald", "Margaret", "Ken", "Dennis", "Frances", "John", "Radia",
    "Leslie", "Tim", "Sophie", "Linus", "Guido", "Bjarne", "Anders", "Niklaus",
];
const LAST_NAMES: &[&str] = &[
    "Lovelace", "Hopper", "Turing", "Dijkstra", "Liskov", "Knuth", "Hamilton", "Thompson", "Ritchie", "Allen",
    "Backus", "Perlman", "Lamport", "Berners-Lee", "Wilson", "Torvalds", "Rossum", "Stroustrup", "Hejlsberg", "Wirth",
];
const CITIES: &[&str] = &[
    "Berlin", "Oslo", "Lisbon", "Montréal", "Kyoto", "Nairobi", "Lima", "Austin", "Tallinn", "Wellington", "Porto",
    "Seoul",
];
const STATUSES: &[(&str, Color)] = &[
    ("Shipped", Color::rgbf(0.13, 0.45, 0.25)),
    ("Pending", Color::rgbf(0.6, 0.4, 0.0)),
    ("Delivered", Color::rgbf(0.13, 0.45, 0.25)),
    ("Cancelled", Color::rgbf(0.7, 0.15, 0.15)),
    ("Refunded", Color::rgbf(0.35, 0.35, 0.6)),
    ("On hold", Color::rgbf(0.6, 0.4, 0.0)),
];
const TEXT_COLOR: Color = Color::rgbf(0.16, 0.16, 0.18);

struct TableCell {
    x: f32,
    // Only fill_text reads the text. glyph_run uses the glyphs shaped from it at startup.
    #[cfg_attr(not(feature = "textlayout"), allow(dead_code))]
    text: String,
    glyphs: Vec<PositionedGlyph>,
    color: Color,
}

fn table_cells(font_ref: swash::FontRef<'_>, texts: Vec<(String, Color)>) -> Vec<TableCell> {
    let mut x = 0.0;
    texts
        .into_iter()
        .zip(COLUMNS)
        .map(|((text, color), (_, width))| {
            let cell = TableCell { x: x + 8.0, glyphs: shape(font_ref, &text, TABLE_FONT_SIZE), text, color };
            x += width;
            cell
        })
        .collect()
}

fn table(font_ref: swash::FontRef<'_>) -> (Vec<TableCell>, Vec<Vec<TableCell>>) {
    let header =
        table_cells(font_ref, COLUMNS.iter().map(|(title, _)| ((*title).to_owned(), Color::rgb(20, 20, 24))).collect());
    let mut seed: u32 = 4242;
    let rows = (0..TABLE_ROWS)
        .map(|row| {
            let first = FIRST_NAMES[next(&mut seed) as usize % FIRST_NAMES.len()];
            let last = LAST_NAMES[next(&mut seed) as usize % LAST_NAMES.len()];
            let (status, status_color) = STATUSES[next(&mut seed) as usize % STATUSES.len()];
            let cents = next(&mut seed) % 5_000_000;
            let dollars = cents / 100;
            let amount = if dollars >= 1000 {
                format!("${},{:03}.{:02}", dollars / 1000, dollars % 1000, cents % 100)
            } else {
                format!("${dollars}.{:02}", cents % 100)
            };
            let mut note = String::new();
            for _ in 0..3 + next(&mut seed) % 6 {
                if !note.is_empty() {
                    note.push(' ');
                }
                note.push_str(WORDS[next(&mut seed) as usize % WORDS.len()]);
            }
            let texts = vec![
                (format!("#{:06}", 104_729 + row * 7), TEXT_COLOR),
                (format!("{first} {last}"), TEXT_COLOR),
                (format!("{}.{}@example.com", first.to_lowercase(), last.to_lowercase()), TEXT_COLOR),
                (CITIES[next(&mut seed) as usize % CITIES.len()].to_owned(), TEXT_COLOR),
                (status.to_owned(), status_color),
                (amount, TEXT_COLOR),
                (format!("2026-{:02}-{:02}", 1 + next(&mut seed) % 12, 1 + next(&mut seed) % 28), TEXT_COLOR),
                (note, TEXT_COLOR),
            ];
            table_cells(font_ref, texts)
        })
        .collect();
    (header, rows)
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    GlyphRun,
    FillText,
}

#[derive(Clone, Copy, PartialEq)]
enum Scene {
    Labels,
    Table,
}

struct Bench {
    mode: Mode,
    scene: Scene,
    labels: Vec<Label>,
    header: Vec<TableCell>,
    rows: Vec<Vec<TableCell>>,
    font_id: FontId,
    scissor: bool,
    same_paint: bool,
    coords: Vec<i16>,
    exact_size_hint: bool,
    stroke: bool,
    rotate: f32,
    one_run: Vec<PositionedGlyph>,
    paints: Vec<Paint>,
    frame: Cell<usize>,
}

impl Bench {
    // Registers the font the way Slint's font cache does, on a TextContext the canvas is created with.
    fn new(text_context: &TextContext, mode: Mode, scene: Scene) -> Self {
        let path = std::env::var("FONT").unwrap_or_else(|_| "fonts/Roboto-Regular.ttf".to_owned());
        let data = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let font_id = text_context.add_shared_font_with_index(data.clone(), 0).unwrap();
        let font_ref = swash::FontRef::from_index(&data, 0).expect("FONT is no font swash can read");
        let count = std::env::var("LABELS").ok().and_then(|v| v.parse().ok()).unwrap_or(200);
        let labels = if scene == Scene::Labels { labels(font_ref, count) } else { Vec::new() };
        let (header, rows) = if scene == Scene::Table { table(font_ref) } else { (Vec::new(), Vec::new()) };
        let paints = labels.iter().map(|label| Self::paint(font_id, label.color, label.size)).collect();
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
            scene,
            labels,
            header,
            rows,
            font_id,
            scissor: std::env::var("SCISSOR").is_ok_and(|v| v == "1"),
            same_paint: std::env::var("SAME_PAINT").is_ok_and(|v| v == "1"),
            coords: std::env::var("COORDS")
                .map(|v| v.split(',').map(|c| c.trim().parse().unwrap()).collect())
                .unwrap_or_default(),
            exact_size_hint: std::env::var("EXACT_SIZE_HINT").is_ok_and(|v| v == "1"),
            stroke: std::env::var("STROKE").is_ok_and(|v| v == "1"),
            rotate: std::env::var("ROTATE").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0),
            one_run,
            paints,
            frame: Cell::new(0),
        }
    }

    fn paint(font_id: FontId, color: Color, size: f32) -> Paint {
        let mut paint = Paint::color(color);
        paint.set_font(&[font_id]);
        paint.set_font_size(size);
        paint
    }

    #[cfg_attr(not(feature = "textlayout"), allow(unused_variables))]
    fn draw_text<T: Renderer>(
        &self,
        canvas: &mut Canvas<T>,
        text: &str,
        glyphs: &[PositionedGlyph],
        size: f32,
        paint: &Paint,
    ) {
        match self.mode {
            Mode::GlyphRun if self.exact_size_hint => self.draw_glyphs(canvas, glyphs.iter().cloned(), paint),
            Mode::GlyphRun => self.draw_glyphs(canvas, glyphs.iter().cloned().flat_map(std::iter::once), paint),
            #[cfg(feature = "textlayout")]
            Mode::FillText if self.stroke => {
                canvas.stroke_text(0.0, size, text, paint).unwrap();
            }
            #[cfg(feature = "textlayout")]
            Mode::FillText => {
                canvas.fill_text(0.0, size, text, paint).unwrap();
            }
            #[cfg(not(feature = "textlayout"))]
            Mode::FillText => unreachable!(),
        }
    }

    fn draw_glyphs<T: Renderer>(
        &self,
        canvas: &mut Canvas<T>,
        glyphs: impl IntoIterator<Item = PositionedGlyph>,
        paint: &Paint,
    ) {
        if self.stroke {
            canvas.stroke_glyph_run(self.font_id, &self.coords, glyphs, paint).unwrap();
        } else {
            canvas.fill_glyph_run(self.font_id, &self.coords, glyphs, paint).unwrap();
        }
    }

    fn draw<T: Renderer>(&self, canvas: &mut Canvas<T>) {
        canvas.clear_rect(0, 0, W, H, Color::white());
        match self.scene {
            Scene::Labels => self.draw_labels(canvas),
            Scene::Table => self.draw_table(canvas),
        }
        self.frame.set(self.frame.get() + 1);
    }

    fn draw_labels<T: Renderer>(&self, canvas: &mut Canvas<T>) {
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
            if self.rotate != 0.0 {
                canvas.rotate(self.rotate);
            }
            if self.scissor {
                canvas.intersect_scissor(0.0, 0.0, 460.0, 20.0);
            }
            let built;
            let paint = if self.same_paint {
                kept_paint
            } else {
                built = Self::paint(self.font_id, label.color, label.size);
                &built
            };
            self.draw_text(canvas, &label.text, &label.glyphs, label.size, paint);
            canvas.restore();
        }
    }

    fn draw_row<T: Renderer>(&self, canvas: &mut Canvas<T>, cells: &[TableCell], y: f32) {
        for cell in cells {
            canvas.save();
            canvas.translate(cell.x, y + (ROW_HEIGHT - TABLE_FONT_SIZE) / 2.0 - 2.0);
            if self.rotate != 0.0 {
                canvas.rotate(self.rotate);
            }
            let paint = Self::paint(self.font_id, cell.color, TABLE_FONT_SIZE);
            self.draw_text(canvas, &cell.text, &cell.glyphs, TABLE_FONT_SIZE, &paint);
            canvas.restore();
        }
    }

    fn fill_rect<T: Renderer>(canvas: &mut Canvas<T>, y: f32, height: f32, color: Color) {
        let mut path = Path::new();
        path.rect(0.0, y, W as f32, height);
        canvas.fill_path(&path, &Paint::color(color));
    }

    fn draw_table<T: Renderer>(&self, canvas: &mut Canvas<T>) {
        let body_height = H as f32 - HEADER_HEIGHT;
        let total_height = self.rows.len() as f32 * ROW_HEIGHT;
        let offset = (self.frame.get() as f32 * SCROLL_STEP) % (total_height - body_height);
        canvas.save();
        canvas.intersect_scissor(0.0, HEADER_HEIGHT, W as f32, body_height);
        let first = (offset / ROW_HEIGHT) as usize;
        let last = (((offset + body_height) / ROW_HEIGHT).ceil() as usize).min(self.rows.len());
        for (index, cells) in self.rows.iter().enumerate().take(last).skip(first) {
            let y = HEADER_HEIGHT + index as f32 * ROW_HEIGHT - offset;
            if index % 2 == 1 {
                Self::fill_rect(canvas, y, ROW_HEIGHT, Color::rgb(244, 245, 247));
            }
            self.draw_row(canvas, cells, y);
        }
        canvas.restore();
        Self::fill_rect(canvas, 0.0, HEADER_HEIGHT, Color::rgb(228, 230, 235));
        self.draw_row(canvas, &self.header, 2.0);
    }
}

fn parse_args() -> (Mode, usize, String, Scene) {
    let args: Vec<String> = std::env::args().collect();
    let mode = match args.get(1).map(String::as_str) {
        Some("glyph_run") => Mode::GlyphRun,
        Some("fill_text") if cfg!(feature = "textlayout") => Mode::FillText,
        Some("fill_text") => panic!("fill_text needs the textlayout feature"),
        _ => panic!("usage: textframe-<master|fixed> glyph_run|fill_text <frames> [void|gl|wgpu] [labels|table]"),
    };
    let frames = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(1000);
    let renderer = args.get(3).cloned().unwrap_or_else(|| "void".to_owned());
    let scene = match args.get(4).map(String::as_str) {
        None | Some("labels") => Scene::Labels,
        Some("table") => Scene::Table,
        Some(other) => panic!("unknown scene {other}"),
    };
    (mode, frames, renderer, scene)
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

/// Prints a hash of the frame with PIXEL_HASH=1 and saves it with SCREENSHOT=<file.ppm>. `rgba` holds `width`
/// times `height` pixels of four bytes each, row by row.
fn report_pixels(width: usize, height: usize, rgba: &[u8]) {
    if std::env::var("PIXEL_HASH").is_ok_and(|v| v == "1") {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let mut inked = 0usize;
        for p in rgba.chunks_exact(4) {
            for &byte in p {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3);
            }
            inked += usize::from(p[0] < 250 || p[1] < 250 || p[2] < 250);
        }
        println!("pixels={width}x{height} hash={hash:016x} non_white={inked}");
    }
    if let Ok(path) = std::env::var("SCREENSHOT") {
        let mut ppm = format!("P6\n{width} {height}\n255\n").into_bytes();
        ppm.extend(rgba.chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]));
        std::fs::write(&path, ppm).unwrap_or_else(|e| panic!("{path}: {e}"));
    }
}

fn run<T: Renderer>(
    canvas: &mut Canvas<T>,
    text_context: &TextContext,
    mode: Mode,
    scene: Scene,
    frames: usize,
    flush: impl Fn(&mut Canvas<T>),
) {
    let bench = Bench::new(text_context, mode, scene);
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
    let (mode, frames, renderer, scene) = parse_args();
    match renderer.as_str() {
        "void" => {
            let text_context = TextContext::default();
            let mut canvas = Canvas::new_with_text_context(Void, text_context.clone()).unwrap();
            canvas.set_size(W, H, 1.0);
            run(&mut canvas, &text_context, mode, scene, frames, |canvas| canvas.flush_to_output(()));
        }
        "gl" => gl::run_gl(mode, scene, frames),
        "wgpu" => wgpu_renderer::run_wgpu(mode, scene, frames),
        other => panic!("unknown renderer {other}"),
    }
}

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

    use super::{Mode, Scene, H, W};

    struct App {
        mode: Mode,
        scene: Scene,
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
            super::run(&mut canvas, &text_context, self.mode, self.scene, self.frames, |canvas: &mut Canvas<OpenGl>| {
                canvas.flush_to_output(());
            });
            // Wait for the GPU so that a run's instruction count includes the driver work its frames queued.
            let shot = canvas.screenshot().unwrap();
            let rgba: Vec<u8> = shot.pixels().flat_map(|p| [p.r, p.g, p.b, p.a]).collect();
            super::report_pixels(shot.width(), shot.height(), &rgba);
            event_loop.exit();
        }

        fn window_event(&mut self, _: &ActiveEventLoop, _: winit::window::WindowId, _: WindowEvent) {}
    }

    pub(super) fn run_gl(mode: Mode, scene: Scene, frames: usize) {
        let event_loop = EventLoop::new().unwrap();
        event_loop
            .run_app(&mut App {
                mode,
                scene,
                frames,
                done: false,
            })
            .unwrap();
    }
}

mod wgpu_renderer {
    use femtovg::{renderer::WGPURenderer, Canvas, TextContext};

    use super::{Mode, Scene, H, W};

    /// Copies `target` into a buffer and returns its pixels as RGBA bytes, row by row.
    fn readback(device: &wgpu::Device, queue: &wgpu::Queue, target: &wgpu::Texture) -> Vec<u8> {
        let row = (W * 4).div_ceil(256) * 256;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: u64::from(row * H),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(H),
                },
            },
            target.size(),
        );
        queue.submit(Some(encoder.finish()));
        buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let data = buffer.slice(..).get_mapped_range().unwrap();
        (0..H as usize)
            .flat_map(|y| data[y * row as usize..y * row as usize + W as usize * 4].iter().copied())
            .collect()
    }

    pub(super) fn run_wgpu(mode: Mode, scene: Scene, frames: usize) {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            ..Default::default()
        }))
        .unwrap();
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: W,
                height: H,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let text_context = TextContext::default();
        let renderer = WGPURenderer::new(device.clone(), queue.clone());
        let mut canvas = Canvas::new_with_text_context(renderer, text_context.clone()).unwrap();
        canvas.set_size(W, H, 1.0);
        super::run(&mut canvas, &text_context, mode, scene, frames, |canvas: &mut Canvas<WGPURenderer>| {
            let commands = canvas.flush_to_output(&target);
            queue.submit(commands);
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        });
        super::report_pixels(W as usize, H as usize, &readback(&device, &queue, &target));
    }
}
