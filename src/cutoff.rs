//! Strokes under a canvas shadow, drawn with femtovg on `renderer::WGPURenderer` and saved as a
//! PNG.
//!
//! usage: cutoff-<revision> joins|caps|star shadow|moved <out.png>
//!
//! `shadow` draws each stroke once, under a canvas shadow that has an offset and no blur. `moved`
//! sets no canvas shadow and draws each stroke twice: first in the shadow colour, moved by the
//! shadow offset, and then in its own colour. A shadow with no blur is the shape in the shadow
//! colour, moved by the offset, so the two should draw the same pixels.
//!
//! They do not. femtovg draws the stroke into an offscreen image to make the shadow, and it sizes
//! that image before it draws. `stroke_path_internal` in femtovg's `src/lib.rs` takes the bounding
//! box of the path's points and grows it by half the line width on each side. A miter join or a
//! square cap that reaches further than that is outside the image, and that part of the stroke
//! has no shadow.
//!
//! `joins` and `caps` draw one shape twice: upright on the left, and turned by 45 degrees on the
//! right. Upright, the join or cap reaches half the line width past the box and no further, so
//! its shadow is whole. Turned, it reaches further.

mod offscreen;

use femtovg::{renderer::WGPURenderer, Canvas, Color, LineCap, LineJoin, Paint, Path};

/// The shadow is this far right of the stroke and this far below it, and is not blurred.
const SHADOW_OFFSET: (f32, f32) = (22.0, 22.0);

struct Scene {
    width: u32,
    height: u32,
    paths: Vec<Path>,
    /// The line width, the join and the cap. The colour is set when the scene is drawn.
    stroke: Paint,
}

fn polyline(points: &[(f32, f32)], closed: bool) -> Path {
    let mut path = Path::new();
    path.move_to(points[0].0, points[0].1);
    for point in &points[1..] {
        path.line_to(point.0, point.1);
    }
    if closed {
        path.close();
    }
    path
}

/// The outline of a square, 40 px wide with miter joins, upright and turned by 45 degrees. A
/// miter at a right angle reaches 28 px from the corner. Upright that is 20 px along x and 20 px
/// along y, which is half the line width. Turned it is 28 px along x or y.
fn joins() -> Scene {
    // Half the side of the square, and half its diagonal.
    const HALF: f32 = 55.0;
    let diagonal = HALF * std::f32::consts::SQRT_2;
    let upright = (125.0, 135.0);
    let turned = (365.0, 135.0);
    Scene {
        width: 520,
        height: 290,
        paths: vec![
            polyline(
                &[
                    (upright.0 - HALF, upright.1 - HALF),
                    (upright.0 + HALF, upright.1 - HALF),
                    (upright.0 + HALF, upright.1 + HALF),
                    (upright.0 - HALF, upright.1 + HALF),
                ],
                true,
            ),
            polyline(
                &[
                    (turned.0, turned.1 - diagonal),
                    (turned.0 + diagonal, turned.1),
                    (turned.0, turned.1 + diagonal),
                    (turned.0 - diagonal, turned.1),
                ],
                true,
            ),
        ],
        stroke: Paint::default().with_line_width(40.0).with_line_join(LineJoin::Miter),
    }
}

/// A line 110 px long, 60 px wide with square caps, level and turned by 45 degrees. The corner
/// of a square cap is 42 px from the end of the line. Level that is 30 px along x and 30 px along
/// y, which is half the line width. Turned it is 42 px along x or y.
fn caps() -> Scene {
    // Half the length of the line, and how far that is along x and along y once it is turned.
    const HALF: f32 = 55.0;
    let turned_half = HALF * std::f32::consts::FRAC_1_SQRT_2;
    let level = (125.0, 135.0);
    let turned = (365.0, 135.0);
    Scene {
        width: 520,
        height: 290,
        paths: vec![
            polyline(&[(level.0 - HALF, level.1), (level.0 + HALF, level.1)], false),
            polyline(
                &[
                    (turned.0 - turned_half, turned.1 + turned_half),
                    (turned.0 + turned_half, turned.1 - turned_half),
                ],
                false,
            ),
        ],
        stroke: Paint::default().with_line_width(60.0).with_line_cap(LineCap::Square),
    }
}

/// The outline of a five-pointed star, 24 px wide with miter joins. The two edges at each point
/// meet at 36 degrees, so the miter reaches 39 px past the point, where half the line width is
/// 12 px.
fn star() -> Scene {
    const CENTRE: (f32, f32) = (150.0, 150.0);
    const OUTER: f32 = 76.0;
    // The inner radius of a regular five-pointed star, as a part of the outer radius.
    const INNER: f32 = OUTER * 0.381_966;
    let corners: Vec<(f32, f32)> = (0..10)
        .map(|corner| {
            let radius = if corner % 2 == 0 { OUTER } else { INNER };
            // The first point is straight up, and the corners go round in steps of 36 degrees.
            let angle = (corner as f32 * 36.0 - 90.0).to_radians();
            (CENTRE.0 + radius * angle.cos(), CENTRE.1 + radius * angle.sin())
        })
        .collect();
    Scene {
        width: 320,
        height: 320,
        paths: vec![polyline(&corners, true)],
        stroke: Paint::default().with_line_width(24.0).with_line_join(LineJoin::Miter),
    }
}

/// Strokes every path of `scene` in `color`, moved by `offset`.
fn stroke(canvas: &mut Canvas<WGPURenderer>, scene: &Scene, color: Color, offset: (f32, f32)) {
    let paint = scene.stroke.clone().with_color(color);
    canvas.save();
    canvas.translate(offset.0, offset.1);
    for path in &scene.paths {
        canvas.stroke_path(path, &paint);
    }
    canvas.restore();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, scene, how, out_png] = args.as_slice() else {
        eprintln!("usage: {} joins|caps|star shadow|moved <out.png>", args[0]);
        std::process::exit(2);
    };
    let scene = match scene.as_str() {
        "joins" => joins(),
        "caps" => caps(),
        "star" => star(),
        other => panic!("unknown scene {other}"),
    };
    let canvas_shadow = match how.as_str() {
        "shadow" => true,
        "moved" => false,
        other => panic!("unknown way to draw the shadow {other}"),
    };

    let gpu = offscreen::Gpu::new();
    eprintln!("{}: drawing on {}", args[0], gpu.adapter_name);
    let mut canvas = gpu.canvas(femtovg::TextContext::default(), scene.width, scene.height);
    canvas.clear_rect(0, 0, scene.width, scene.height, Color::rgb(52, 110, 214));

    if canvas_shadow {
        canvas.set_shadow_color(Color::black());
        canvas.set_shadow_offset(SHADOW_OFFSET.0, SHADOW_OFFSET.1);
        canvas.set_shadow_blur(0.0);
    } else {
        stroke(&mut canvas, &scene, Color::black(), SHADOW_OFFSET);
    }
    stroke(&mut canvas, &scene, Color::white(), (0.0, 0.0));

    gpu.read_frame(&mut canvas, scene.width, scene.height)
        .save(out_png)
        .unwrap_or_else(|error| panic!("cannot write {out_png}: {error}"));
}
