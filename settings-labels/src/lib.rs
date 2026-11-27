use femtovg::{Canvas, Color, FontId, Paint, Path, Renderer, TextContext};
use std::time::Instant;

pub struct Scene {
    font: FontId,
    paint: Paint,
    labels: Vec<Vec<(f32, u16)>>,
}
impl Scene {
    pub fn new(ctx: &TextContext) -> Self {
        let bytes = include_bytes!(env!("BENCH_FONT"));
        let font = ctx.add_shared_font_with_index(&bytes[..], 0).unwrap();
        let face = ttf_parser::Face::parse(bytes, 0).unwrap();
        let paint = Paint::color(Color::white()).with_font_size(14.).with_font(&[font]);
        let labels: Vec<Vec<_>> = [
            "Appearance and accessibility",
            "Background notifications",
            "Keyboard shortcuts",
            "Preferred language",
            "Automatic software updates",
            "Battery and charging",
        ]
        .into_iter()
        .map(|label| {
            label
                .chars()
                .enumerate()
                .map(|(i, ch)| (i as f32 * 9., face.glyph_index(ch).unwrap().0))
                .collect()
        })
        .collect();
        Self { font, paint, labels }
    }
    fn draw<T: Renderer>(&self, canvas: &mut Canvas<T>) {
        canvas.clear_rect(0, 0, 900, 900, Color::rgb(41, 43, 48));
        for row in 0..20 {
            for col in 0..3 {
                let x = (col * 295 + 3) as f32 + 0.25;
                let y = (row * 43 + 3) as f32 + 0.5;
                let mut p = Path::new();
                p.rounded_rect(x, y, 284., 39., 0.);
                let paint =
                    Paint::color(Color::rgba((row * 10) as u8, (col * 17) as u8, 120, 190)).with_anti_alias(false);
                canvas.fill_path(&p, &paint);
                canvas
                    .fill_glyph_run(
                        self.font,
                        &[],
                        self.labels[(row + col) % self.labels.len()]
                            .iter()
                            .map(|&(dx, id)| femtovg::PositionedGlyph {
                                x: x + 6. + dx,
                                y: y + 23.,
                                glyph_id: id,
                            }),
                        &self.paint,
                    )
                    .unwrap();
            }
        }
    }
}

pub fn measure<T: Renderer, C>(
    canvas: &mut Canvas<T>,
    scene: &Scene,
    mut flush: impl FnMut(&mut Canvas<T>) -> C,
    mut complete: impl FnMut(C),
) {
    let frames: usize = std::env::args().nth(1).expect("frame count").parse().unwrap();
    assert!(frames >= 2);
    let mut encode = Vec::with_capacity(frames - 1);
    let mut wall = Vec::with_capacity(frames - 1);
    for frame in 0..frames {
        let start = Instant::now();
        scene.draw(canvas);
        let commands = flush(canvas);
        let cpu = start.elapsed().as_nanos();
        complete(commands);
        if frame > 0 {
            encode.push(cpu);
            wall.push(start.elapsed().as_nanos());
        }
    }
    encode.sort_unstable();
    wall.sort_unstable();
    println!("{} {}", encode[encode.len() / 2], wall[wall.len() / 2]);
}
