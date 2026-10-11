//! One `Path` that grows by one verb per frame and is drawn every frame, drawn with femtovg on
//! `renderer::WGPURenderer`, with the last frame saved as a PNG.
//!
//! usage: pathcache-<revision> trace|bars <out.png>
//!
//! A `Path` keeps one flattened copy of itself, built the first time it is drawn or hit-tested
//! and keyed on the canvas transform. On femtovg master the builder methods (`line_to`, `rect`
//! and the rest) do not drop that copy, so a path that is drawn, extended and drawn again under
//! the same transform is drawn as it was at its first draw. Both scenes here keep one path,
//! append to it every frame and draw it every frame, the way an app that plots live data or
//! records a pen stroke does.
//!
//! `trace` appends one sample to a polyline each frame and strokes it. `bars` appends one
//! rectangle each frame and fills the path. Each frame also draws a marker at the newest sample
//! or bar from a new `Path`, which is right on every revision, so the marker shows how far the
//! drawing should have come.

mod offscreen;

use femtovg::{renderer::WGPURenderer, Canvas, Color, LineCap, LineJoin, Paint, Path};

const WIDTH: u32 = 520;
const HEIGHT: u32 = 240;
const BACKGROUND: Color = Color::rgbf(52.0 / 255.0, 110.0 / 255.0, 214.0 / 255.0);
const MARKER: Color = Color::rgbf(1.0, 0.82, 0.25);

/// The samples of the trace: one every 10 px from x = 20 to x = 500, on a curve with two
/// harmonics, so that the trace is not a straight line.
fn samples() -> Vec<(f32, f32)> {
    (0..49)
        .map(|i| {
            let t = i as f32 * 0.3;
            (
                20.0 + i as f32 * 10.0,
                120.0 - 60.0 * t.sin() - 25.0 * (2.7 * t + 1.0).sin(),
            )
        })
        .collect()
}

/// The heights of the bars, in px.
const BAR_HEIGHTS: [f32; 12] = [
    70.0, 110.0, 90.0, 150.0, 130.0, 170.0, 120.0, 160.0, 100.0, 140.0, 80.0, 180.0,
];
const BAR_WIDTH: f32 = 30.0;
const BAR_GAP: f32 = 10.0;
const BASELINE: f32 = 210.0;

/// The left edge of bar `i`. The twelve bars and the gaps between them are centred on the canvas.
fn bar_x(i: usize) -> f32 {
    let chart_width = BAR_HEIGHTS.len() as f32 * (BAR_WIDTH + BAR_GAP) - BAR_GAP;
    (WIDTH as f32 - chart_width) / 2.0 + i as f32 * (BAR_WIDTH + BAR_GAP)
}

/// A filled circle of radius 5 px at `centre`.
fn dot(canvas: &mut Canvas<WGPURenderer>, centre: (f32, f32)) {
    let mut path = Path::new();
    path.circle(centre.0, centre.1, 5.0);
    canvas.fill_path(&path, &Paint::color(MARKER));
}

/// Strokes the trace with one more sample each frame. Returns the last frame.
fn trace(gpu: &offscreen::Gpu, canvas: &mut Canvas<WGPURenderer>) -> image::RgbaImage {
    let samples = samples();
    let paint = Paint::color(Color::white())
        .with_line_width(3.0)
        .with_line_join(LineJoin::Round)
        .with_line_cap(LineCap::Round);
    let mut path = Path::new();
    path.move_to(samples[0].0, samples[0].1);
    let mut frame = None;
    for &sample in &samples[1..] {
        path.line_to(sample.0, sample.1);

        canvas.clear_rect(0, 0, WIDTH, HEIGHT, BACKGROUND);
        canvas.stroke_path(&path, &paint);
        // The cursor: a line down the canvas at the newest sample, and a dot on the sample.
        let mut cursor = Path::new();
        cursor.move_to(sample.0, 0.0);
        cursor.line_to(sample.0, HEIGHT as f32);
        canvas.stroke_path(&cursor, &Paint::color(MARKER).with_line_width(1.0));
        dot(canvas, sample);
        frame = Some(gpu.read_frame(canvas, WIDTH, HEIGHT));
    }
    frame.expect("at least one frame")
}

/// Fills the bar chart with one more bar each frame. Returns the last frame.
fn bars(gpu: &offscreen::Gpu, canvas: &mut Canvas<WGPURenderer>) -> image::RgbaImage {
    let mut path = Path::new();
    let mut frame = None;
    for (i, &height) in BAR_HEIGHTS.iter().enumerate() {
        let x = bar_x(i);
        path.rect(x, BASELINE - height, BAR_WIDTH, height);

        canvas.clear_rect(0, 0, WIDTH, HEIGHT, BACKGROUND);
        canvas.fill_path(&path, &Paint::color(Color::white()));
        // The axis, and a dot under the newest bar.
        let mut axis = Path::new();
        axis.move_to(bar_x(0) - BAR_GAP, BASELINE + 0.5);
        axis.line_to(bar_x(BAR_HEIGHTS.len() - 1) + BAR_WIDTH + BAR_GAP, BASELINE + 0.5);
        canvas.stroke_path(&axis, &Paint::color(Color::white()).with_line_width(1.0));
        dot(canvas, (x + BAR_WIDTH / 2.0, BASELINE + 14.0));
        frame = Some(gpu.read_frame(canvas, WIDTH, HEIGHT));
    }
    frame.expect("at least one frame")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, scene, out_png] = args.as_slice() else {
        eprintln!("usage: {} trace|bars <out.png>", args[0]);
        std::process::exit(2);
    };

    let gpu = offscreen::Gpu::new();
    eprintln!("{}: drawing on {}", args[0], gpu.adapter_name);
    let mut canvas = gpu.canvas(femtovg::TextContext::default(), WIDTH, HEIGHT);
    let frame = match scene.as_str() {
        "trace" => trace(&gpu, &mut canvas),
        "bars" => bars(&gpu, &mut canvas),
        other => panic!("unknown scene {other}"),
    };
    frame
        .save(out_png)
        .unwrap_or_else(|error| panic!("cannot write {out_png}: {error}"));
}
