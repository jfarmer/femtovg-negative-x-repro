//! CPU command generation for #389, with no GPU, window, or platform font dependency.
//! Run --help for options. Python owns build provenance and process measurements.
use femtovg::{
    renderer::Void, Canvas, Color, FontId, ImageFlags, Paint, Path, PixelFormat, PositionedGlyph,
};
use std::{hint::black_box, time::Instant};
mod stress;

const CASES: &[&str] = &[
    "gradient-fill",
    "gradient-stroke",
    "glyph-fill",
    "glyph-stroke",
    "dashed-stroke",
    "image-fill",
    "shadow-fill",
    "transformed-fill",
    "atlas-fill",
    "path-fill",
    "path-stroke",
    "tiny-path-fill",
    "tiny-path-stroke",
    "cache-glyph-fill",
    "mixed-fill",
    "mixed-stroke",
    "best-fill",
    "best-stroke",
];

#[cfg(feature = "allocations")]
mod allocations {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
    pub struct Counting;
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    static BYTES: AtomicUsize = AtomicUsize::new(0);
    static LIVE: AtomicUsize = AtomicUsize::new(0);
    static PEAK: AtomicUsize = AtomicUsize::new(0);
    fn add(size: usize) {
        CALLS.fetch_add(1, Relaxed);
        BYTES.fetch_add(size, Relaxed);
        let live = LIVE.fetch_add(size, Relaxed) + size;
        PEAK.fetch_max(live, Relaxed);
    }
    unsafe impl GlobalAlloc for Counting {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let ptr = System.alloc(layout);
            if !ptr.is_null() {
                add(layout.size());
            }
            ptr
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            let ptr = System.alloc_zeroed(layout);
            if !ptr.is_null() {
                add(layout.size());
            }
            ptr
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            LIVE.fetch_sub(layout.size(), Relaxed);
            System.dealloc(ptr, layout);
        }
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            let result = System.realloc(ptr, layout, size);
            if !result.is_null() {
                LIVE.fetch_sub(layout.size(), Relaxed);
                add(size);
            }
            result
        }
    }
    pub fn begin() -> usize {
        CALLS.store(0, Relaxed);
        BYTES.store(0, Relaxed);
        let live = LIVE.load(Relaxed);
        PEAK.store(live, Relaxed);
        live
    }
    pub fn result(initial: usize) -> String {
        format!("{{\"calls\":{},\"requested_bytes\":{},\"live_bytes_start\":{},\"live_bytes_end\":{},\"peak_live_bytes\":{}}}",
            CALLS.load(Relaxed), BYTES.load(Relaxed), initial, LIVE.load(Relaxed), PEAK.load(Relaxed))
    }
}
#[cfg(feature = "allocations")]
#[global_allocator]
static ALLOCATOR: allocations::Counting = allocations::Counting;

#[cfg(all(target_os = "macos", feature = "signposts"))]
extern "C" {
    fn repro_measure_init();
    fn repro_measure_begin() -> i32;
    fn repro_measure_end();
}

struct Options {
    font: String,
    case: String,
    motion: String,
    text: String,
    measure: String,
    frames: usize,
    warmup: usize,
    lines: usize,
    seconds: f64,
    qos: String,
    samples: String,
    stress: stress::Options,
}
fn options() -> Options {
    let mut opt = Options {
        font: String::new(),
        case: "gradient-fill".into(),
        motion: "stationary".into(),
        text: "sentence".into(),
        measure: "bulk".into(),
        frames: 3000,
        warmup: 300,
        lines: 1,
        seconds: 0.0,
        qos: "default".into(),
        samples: String::new(),
        stress: stress::Options::default(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(key) = args.next() {
        if key == "--help" {
            println!(
                "--font FILE --case {} --motion stationary|moving --text alphabet|sentence|diverse \
                --measure bulk|stages --frames N --warmup N --lines N --seconds SECONDS --qos default|background --samples FILE\n\
                --seconds runs whole 100-frame batches for at least that long; otherwise --frames is exact.\n\
                stages times every frame's draw and flush; bulk has one clock around the whole loop.\n\
                Stress: --draws N --working-set N --glyph-percent N --order grouped|shuffled --shape triangle|rectangle|line --paint solid|gradient --aa true|false --seed N\n\
                Best: --batch true|false --visible-percent 0..100 --font-size SIZE --rotation RADIANS --layout overlap|grid|labels",
                CASES.join("|")
            );
            std::process::exit(0);
        }
        let value = args
            .next()
            .unwrap_or_else(|| panic!("missing value for {key}"));
        match key.as_str() {
            "--font" => opt.font = value,
            "--case" => opt.case = value,
            "--motion" => opt.motion = value,
            "--text" => opt.text = value,
            "--measure" => opt.measure = value,
            "--frames" => opt.frames = value.parse().expect("frames"),
            "--warmup" => opt.warmup = value.parse().expect("warmup"),
            "--lines" => opt.lines = value.parse().expect("lines"),
            "--seconds" => opt.seconds = value.parse().expect("seconds"),
            "--qos" => opt.qos = value,
            "--samples" => opt.samples = value,
            _ => opt.stress.set(&key, &value),
        }
    }
    assert!(!opt.font.is_empty(), "--font is required");
    assert!(CASES.contains(&opt.case.as_str()), "unknown case");
    assert!(
        ["stationary", "moving"].contains(&opt.motion.as_str()),
        "unknown motion"
    );
    assert!(
        ["alphabet", "sentence", "diverse"].contains(&opt.text.as_str()),
        "unknown text"
    );
    assert!(
        ["bulk", "stages"].contains(&opt.measure.as_str()),
        "unknown measurement"
    );
    assert!(
        ["default", "background"].contains(&opt.qos.as_str()),
        "unknown QoS"
    );
    assert!(
        opt.frames > 0 && opt.lines > 0 && opt.lines <= 64,
        "frames > 0; lines 1..64"
    );
    assert!(
        opt.seconds.is_finite() && opt.seconds >= 0.0,
        "seconds must be finite and nonnegative"
    );
    assert!(
        opt.seconds == 0.0 || opt.measure == "bulk",
        "duration mode requires bulk"
    );
    opt.stress.validate();
    opt
}

fn set_qos(qos: &str) {
    if qos == "background" {
        #[cfg(target_os = "macos")]
        {
            extern "C" {
                fn pthread_set_qos_class_self_np(class: u32, relative_priority: i32) -> i32;
            }
            // Background QoS encourages efficiency-core placement; it does not pin a core.
            assert_eq!(
                unsafe { pthread_set_qos_class_self_np(0x09, 0) },
                0,
                "set background QoS"
            );
        }
        #[cfg(not(target_os = "macos"))]
        panic!("background QoS is macOS-only; use the default on this platform");
    }
}

struct Scene {
    canvas: Canvas<Void>,
    font: FontId,
    paints: Vec<Paint>,
    texts: Vec<String>,
    glyphs: Vec<Vec<PositionedGlyph>>,
    path: Path,
    stress: Option<stress::Workload>,
}
impl Scene {
    fn new(opt: &Options, font_data: &[u8]) -> Self {
        let mut canvas = Canvas::new(Void).expect("canvas");
        canvas.set_size(16384, 16384, 1.0);
        let font = canvas.add_font_mem(font_data).expect("font");
        if stress::is_case(&opt.case) {
            let stress =
                stress::Workload::new(&mut canvas, font, &opt.case, &opt.stress, font_data);
            return Self {
                canvas,
                font,
                stress: Some(stress),
                paints: Vec::new(),
                texts: Vec::new(),
                glyphs: Vec::new(),
                path: Path::new(),
            };
        }
        let image = canvas
            .create_image_empty(64, 64, PixelFormat::Rgba8, ImageFlags::empty())
            .expect("image");
        let words = match opt.text.as_str() {
            "alphabet" => "abcdefghijklmnopqrstuvwxyz".to_owned(),
            "sentence" => "the quick brown fox jumps over".to_owned(),
            _ => (33u8..=126).map(char::from).collect(),
        };
        let mut paints = Vec::new();
        let mut texts = Vec::new();
        for line in 0..opt.lines {
            let size = match opt.case.as_str() {
                "atlas-fill" => 90.0,
                "transformed-fill" => 48.0,
                _ => {
                    100.0
                        + if opt.lines > 1 {
                            (line % 16) as f32
                        } else {
                            0.0
                        }
                }
            };
            let paint = if opt.case == "image-fill" {
                Paint::image(image, 0.0, 0.0, 64.0, 64.0, 0.0, 1.0)
            } else {
                Paint::linear_gradient(
                    0.0,
                    0.0,
                    2048.0,
                    0.0,
                    Color::rgb(230, 40, 40),
                    if opt.case == "shadow-fill" {
                        Color::rgba(230, 40, 40, 0)
                    } else {
                        Color::rgb(40, 60, 230)
                    },
                )
            }
            .with_font(&[font])
            .with_font_size(size)
            .with_line_width(2.0);
            paints.push(if opt.case == "dashed-stroke" {
                paint.with_line_dash(&[300.0, 200.0])
            } else {
                paint
            });
            // Rotate printable ASCII to vary shaping and outline access order in the large-working-set case.
            let text = if opt.text == "diverse" {
                let shift = line % words.len();
                format!("{}{}", &words[shift..], &words[..shift])
            } else {
                words.clone()
            };
            texts.push(text);
        }
        let glyphs = if opt.case.starts_with("glyph-") {
            texts
                .iter()
                .zip(&paints)
                .enumerate()
                .map(|(line, (text, paint))| {
                    canvas
                        .measure_text(10.0, 120.0 + line as f32 * 150.0, text, paint)
                        .expect("prepare glyphs")
                        .glyphs
                        .into_iter()
                        .map(|g| PositionedGlyph {
                            x: g.x,
                            y: g.y,
                            glyph_id: g.glyph_id,
                        })
                        .collect()
                })
                .collect()
        } else {
            Vec::new()
        };
        if opt.case == "shadow-fill" {
            canvas.set_shadow_color(Color::black());
            canvas.set_shadow_offset(0.0, 18.0);
            canvas.set_shadow_blur(14.0);
        }
        if opt.case == "transformed-fill" {
            canvas.scale(2.0, 2.0);
        }
        let mut path = Path::new();
        path.move_to(0.0, 0.0);
        path.bezier_to(50.0, -20.0, 90.0, 60.0, 100.0, 100.0);
        path.line_to(25.0, 75.0);
        path.close();
        Self {
            canvas,
            font,
            paints,
            texts,
            glyphs,
            path,
            stress: None,
        }
    }
    fn draw(&mut self, opt: &Options, frame: usize) {
        if let Some(stress) = &self.stress {
            stress.draw(&mut self.canvas, self.font, &opt.case, &opt.motion, frame);
            black_box(&mut self.canvas);
            return;
        }
        // Bounded motion changes the last transform used by Path::cache every frame.
        let offset = if opt.motion == "moving" {
            (frame % 128) as f32 * 0.25
        } else {
            0.0
        };
        for (line, (paint, text)) in self.paints.iter().zip(&self.texts).enumerate() {
            let y = 120.0 + line as f32 * 150.0;
            match opt.case.as_str() {
                "path-fill" | "path-stroke" => {
                    self.canvas.save();
                    self.canvas.translate(10.0 + offset, y);
                    if opt.case == "path-fill" {
                        self.canvas.fill_path(&self.path, paint);
                    } else {
                        self.canvas.stroke_path(&self.path, paint);
                    }
                    self.canvas.restore();
                }
                "gradient-stroke" | "dashed-stroke" => {
                    black_box(
                        self.canvas
                            .stroke_text(10.0 + offset, y, text, paint)
                            .expect("stroke"),
                    );
                }
                "glyph-fill" | "glyph-stroke" => {
                    let glyphs = self.glyphs[line].iter().map(|g| PositionedGlyph {
                        x: g.x + offset,
                        y: g.y,
                        glyph_id: g.glyph_id,
                    });
                    if opt.case == "glyph-fill" {
                        self.canvas
                            .fill_glyph_run(self.font, &[], glyphs, paint)
                            .expect("fill glyphs");
                    } else {
                        self.canvas
                            .stroke_glyph_run(self.font, &[], glyphs, paint)
                            .expect("stroke glyphs");
                    }
                }
                _ => {
                    black_box(
                        self.canvas
                            .fill_text(10.0 + offset, y, text, paint)
                            .expect("fill"),
                    );
                }
            }
        }
        black_box(&mut self.canvas);
    }
    fn flush(&mut self) {
        self.canvas.flush_to_output(());
        black_box(&mut self.canvas);
    }
}

fn distribution(samples: &mut [u64]) -> String {
    samples.sort_unstable();
    let at = |q: usize| samples[((samples.len() - 1) * q).div_ceil(100)];
    let total: u128 = samples.iter().map(|n| *n as u128).sum();
    format!(
        "{{\"total_ns\":{total},\"p50_ns\":{},\"p95_ns\":{},\"p99_ns\":{}}}",
        at(50),
        at(95),
        at(99)
    )
}

fn main() {
    let opt = options();
    #[cfg(all(target_os = "macos", feature = "signposts"))]
    unsafe {
        repro_measure_init();
    }
    set_qos(&opt.qos);
    let font_data = std::fs::read(&opt.font).expect("read font");
    let setup = Instant::now();
    let mut scene = Scene::new(&opt, &font_data);
    let setup_ns = setup.elapsed().as_nanos();
    // First frame includes cold shaping/outlines; it is reported independently of steady state.
    let first = Instant::now();
    scene.draw(&opt, 0);
    scene.flush();
    let first_frame_ns = first.elapsed().as_nanos();
    for frame in 0..opt.warmup {
        scene.draw(&opt, frame);
        scene.flush();
    }
    let mut draw_times = if opt.measure == "stages" {
        vec![0u64; opt.frames]
    } else {
        Vec::new()
    };
    let mut flush_times = draw_times.clone();
    let mut frame_times = draw_times.clone();
    // Determine work outside the measured region (counts include spaces and missing-glyph substitutions).
    let glyphs_per_frame: usize = if let Some(stress) = &scene.stress {
        stress.glyphs_per_frame()
    } else if opt.case.starts_with("path-") {
        0
    } else {
        scene
            .texts
            .iter()
            .zip(&scene.paints)
            .map(|(text, paint)| {
                scene
                    .canvas
                    .measure_text(0.0, 0.0, text, paint)
                    .expect("measure")
                    .glyphs
                    .len()
            })
            .sum()
    };
    black_box(scene.font);
    #[cfg(all(target_os = "macos", feature = "signposts"))]
    unsafe {
        assert_ne!(
            repro_measure_begin(),
            0,
            "signpost logging did not become enabled"
        );
    }
    #[cfg(feature = "allocations")]
    let initial_live = allocations::begin();
    let start = Instant::now();
    let mut frames = 0usize;
    loop {
        let batch = if opt.seconds > 0.0 { 100 } else { opt.frames };
        for _ in 0..batch {
            if opt.measure == "stages" {
                let frame_start = Instant::now();
                scene.draw(&opt, opt.warmup + frames);
                let draw_end = Instant::now();
                scene.flush();
                let frame_end = Instant::now();
                draw_times[frames] = draw_end.duration_since(frame_start).as_nanos() as u64;
                flush_times[frames] = frame_end.duration_since(draw_end).as_nanos() as u64;
                frame_times[frames] = frame_end.duration_since(frame_start).as_nanos() as u64;
            } else {
                scene.draw(&opt, opt.warmup + frames);
                scene.flush();
            }
            frames += 1;
        }
        if opt.seconds == 0.0 || start.elapsed().as_secs_f64() >= opt.seconds {
            break;
        }
    }
    let elapsed_ns = start.elapsed().as_nanos();
    #[cfg(feature = "allocations")]
    let allocation_result = allocations::result(initial_live);
    #[cfg(not(feature = "allocations"))]
    let allocation_result = "null";
    #[cfg(all(target_os = "macos", feature = "signposts"))]
    unsafe {
        repro_measure_end();
    }
    let stages = if opt.measure == "stages" {
        if !opt.samples.is_empty() {
            use std::io::Write;
            let mut file =
                std::io::BufWriter::new(std::fs::File::create(&opt.samples).expect("samples file"));
            writeln!(
                file,
                "{{\"draw_ns\":{:?},\"flush_ns\":{:?},\"frame_ns\":{:?}}}",
                draw_times, flush_times, frame_times
            )
            .expect("write samples");
            file.flush().expect("flush samples");
        }
        format!(
            "{{\"draw\":{},\"flush\":{},\"frame\":{}}}",
            distribution(&mut draw_times),
            distribution(&mut flush_times),
            distribution(&mut frame_times)
        )
    } else {
        "null".into()
    };
    let stress_json = scene
        .stress
        .as_ref()
        .map_or_else(|| "null".to_owned(), |s| s.metadata(&opt.stress));
    println!("{{\"stress\":{},\"schema_version\":1,\"case\":\"{}\",\"motion\":\"{}\",\"text\":\"{}\",\"measure\":\"{}\",\"qos\":\"{}\",\"lines\":{},\"warmup_frames\":{},\"frames\":{},\"glyphs_per_frame\":{},\"setup_ns\":{},\"first_frame_ns\":{},\"elapsed_ns\":{},\"stages\":{},\"allocations\":{},\"signposts\":{}}}",
        stress_json, opt.case, opt.motion, opt.text, opt.measure, opt.qos, opt.lines, opt.warmup, frames,
        glyphs_per_frame, setup_ns, first_frame_ns, elapsed_ns, stages, allocation_result,
        cfg!(all(target_os = "macos", feature = "signposts")));
}
