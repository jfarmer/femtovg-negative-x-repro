//! Adversarial workloads: persistent geometry and seeded schedules built before timing.
use femtovg::{renderer::Void, Canvas, Color, FontId, Paint, Path, PositionedGlyph};

pub struct Options {
    pub draws: usize,
    pub working_set: usize,
    pub ratio: usize,
    pub seed: u32,
    pub order: String,
    pub shape: String,
    pub paint: String,
    pub aa: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            draws: 512,
            working_set: 16,
            ratio: 50,
            seed: 389,
            order: "grouped".into(),
            shape: "triangle".into(),
            paint: "gradient".into(),
            aa: true,
        }
    }
}
impl Options {
    pub fn set(&mut self, key: &str, value: &str) {
        match key {
            "--draws" => self.draws = value.parse().expect("draws"),
            "--working-set" => self.working_set = value.parse().expect("working set"),
            "--glyph-percent" => self.ratio = value.parse().expect("glyph percent"),
            "--seed" => self.seed = value.parse().expect("seed"),
            "--order" => self.order = value.into(),
            "--shape" => self.shape = value.into(),
            "--paint" => self.paint = value.into(),
            "--aa" => self.aa = value.parse().expect("aa: true or false"),
            _ => panic!("unknown option {key}"),
        }
    }
    pub fn validate(&self) {
        assert!((1..=8192).contains(&self.draws));
        assert!((1..=4096).contains(&self.working_set));
        assert!(self.ratio <= 100 && self.seed != 0);
        assert!(["grouped", "shuffled"].contains(&self.order.as_str()));
        assert!(["triangle", "rectangle", "line"].contains(&self.shape.as_str()));
        assert!(["solid", "gradient"].contains(&self.paint.as_str()));
    }
}
pub fn is_case(case: &str) -> bool {
    case.starts_with("tiny-path-") || case == "cache-glyph-fill" || case.starts_with("mixed-")
}

#[derive(Clone, Copy)]
enum Draw {
    Path(usize),
    Glyph(usize),
}
pub struct Workload {
    paths: Vec<Path>,
    paints: Vec<Paint>,
    glyphs: Vec<PositionedGlyph>,
    normalized_coords: Vec<Vec<i16>>,
    schedule: Vec<Draw>,
    hash: u32,
    glyph_count: usize,
}
fn next(seed: &mut u32) -> u32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    *seed
}
impl Workload {
    pub fn new(
        canvas: &mut Canvas<Void>,
        font: FontId,
        case: &str,
        opt: &Options,
        font_data: &[u8],
    ) -> Self {
        let glyph_count = if case == "cache-glyph-fill" {
            opt.draws
        } else if case.starts_with("mixed-") {
            opt.draws * opt.ratio / 100
        } else {
            0
        };
        let path_count = opt.draws - glyph_count;
        assert!(case != "cache-glyph-fill" || opt.draws >= opt.working_set);
        let mut paths = Vec::new();
        for index in 0..path_count {
            let x = 10.0 + (index % 64) as f32 * 64.0;
            let y = 10.0 + (index / 64) as f32 * 64.0;
            let mut path = Path::new();
            match opt.shape.as_str() {
                "rectangle" => path.rect(x, y, 16.0, 16.0),
                "line" => {
                    path.move_to(x, y);
                    path.line_to(x + 16.0, y + 8.0);
                }
                _ => {
                    path.move_to(x, y);
                    path.line_to(x + 16.0, y);
                    path.line_to(x + 8.0, y + 16.0);
                    path.close();
                }
            }
            paths.push(path);
        }
        let make_paint = |size| {
            let paint = if opt.paint == "solid" {
                Paint::color(Color::rgb(80, 120, 180))
            } else {
                Paint::linear_gradient(0.0, 0.0, 8192.0, 0.0, Color::black(), Color::white())
            };
            paint
                .with_font(&[font])
                .with_font_size(size)
                .with_line_width(1.0)
                .with_anti_alias(opt.aa)
        };
        // Each cache entry has one fixed position. Reordering alone therefore does not
        // invalidate its flattened outline; moving deliberately invalidates both variants.
        let mut paints = vec![make_paint(100.0)];
        let mut glyphs = Vec::new();
        let mut normalized_coords = Vec::new();
        if case == "cache-glyph-fill" {
            let chars: Vec<char> = std::iter::once('I')
                .chain((33u8..=126).map(char::from).filter(|&c| c != 'I'))
                .collect();
            let (axis_count, weight_axis) = variation_axis(font_data, b"wght");
            for index in 0..opt.working_set {
                let mut coords = vec![0i16; axis_count];
                coords[weight_axis] = ((index / chars.len()) * 256) as i16;
                normalized_coords.push(coords);
                let size = 100.0;
                let paint = make_paint(size);
                let x = 10.0 + (index % 64) as f32 * 220.0;
                let y = 180.0 + (index / 64) as f32 * 220.0;
                let metrics = canvas
                    .measure_text(x, y, chars[index % chars.len()].to_string(), &paint)
                    .expect("shape glyph");
                assert_eq!(metrics.glyphs.len(), 1);
                let g = &metrics.glyphs[0];
                glyphs.push(PositionedGlyph {
                    x: g.x,
                    y: g.y,
                    glyph_id: g.glyph_id,
                });
                paints.push(paint);
            }
        } else if glyph_count > 0 {
            normalized_coords.push(Vec::new());
            // One representation and one location minimize tessellation/rebuild work,
            // making the added Some/None branch visible in the mixed test.
            let metrics = canvas
                .measure_text(5000.0, 180.0, "I", &paints[0])
                .expect("shape I");
            let g = &metrics.glyphs[0];
            glyphs.push(PositionedGlyph {
                x: g.x,
                y: g.y,
                glyph_id: g.glyph_id,
            });
        }
        let mut schedule: Vec<Draw> = (0..path_count)
            .map(Draw::Path)
            .chain((0..glyph_count).map(|i| {
                Draw::Glyph(if case == "cache-glyph-fill" {
                    i % opt.working_set
                } else {
                    0
                })
            }))
            .collect();
        if opt.order == "shuffled" {
            let mut seed = opt.seed;
            for i in (1..schedule.len()).rev() {
                let j = next(&mut seed) as usize % (i + 1);
                schedule.swap(i, j);
            }
        }
        let mut hash = 2166136261u32;
        for draw in &schedule {
            let value = match draw {
                Draw::Path(i) => *i as u32,
                Draw::Glyph(i) => (*i as u32) | 0x80000000,
            };
            hash = (hash ^ value).wrapping_mul(16777619);
        }
        for glyph in &glyphs {
            for value in [glyph.x.to_bits(), glyph.y.to_bits(), glyph.glyph_id as u32] {
                hash = (hash ^ value).wrapping_mul(16777619);
            }
        }
        for coords in &normalized_coords {
            for &value in coords {
                hash = (hash ^ value as u16 as u32).wrapping_mul(16777619);
            }
        }
        Self {
            paths,
            paints,
            glyphs,
            normalized_coords,
            schedule,
            hash,
            glyph_count,
        }
    }
    pub fn glyphs_per_frame(&self) -> usize {
        self.glyph_count
    }
    pub fn metadata(&self, opt: &Options) -> String {
        format!("{{\"draws\":{},\"working_set\":{},\"glyph_percent\":{},\"seed\":{},\"order\":\"{}\",\"shape\":\"{}\",\"paint\":\"{}\",\"aa\":{},\"schedule_hash\":{},\"unique_paths\":{},\"unique_glyph_representations\":{},\"motion_period_frames\":16}}",
            opt.draws, opt.working_set, opt.ratio, opt.seed, opt.order, opt.shape, opt.paint, opt.aa,
            self.hash, self.paths.len(), self.glyphs.len())
    }
    pub fn draw(
        &self,
        canvas: &mut Canvas<Void>,
        font: FontId,
        case: &str,
        motion: &str,
        frame: usize,
    ) {
        let offset = if motion == "moving" {
            (frame % 16) as f32 * 0.25
        } else {
            0.0
        };
        for draw in &self.schedule {
            match draw {
                Draw::Path(index) => {
                    if case.ends_with("stroke") {
                        canvas.stroke_path(&self.paths[*index], &self.paints[0]);
                    } else {
                        canvas.fill_path(&self.paths[*index], &self.paints[0]);
                    }
                }
                Draw::Glyph(index) => {
                    let g = &self.glyphs[*index];
                    let glyph = PositionedGlyph {
                        x: g.x + offset,
                        y: g.y,
                        glyph_id: g.glyph_id,
                    };
                    let paint = &self.paints[if case == "cache-glyph-fill" {
                        index + 1
                    } else {
                        0
                    }];
                    if case.ends_with("stroke") {
                        canvas
                            .stroke_glyph_run(font, &self.normalized_coords[*index], [glyph], paint)
                            .expect("stroke glyph");
                    } else {
                        canvas
                            .fill_glyph_run(font, &self.normalized_coords[*index], [glyph], paint)
                            .expect("fill glyph");
                    }
                }
            }
        }
    }
}

// Read fvar's axis order rather than assuming a platform or a particular font's order.
// Public glyph-run APIs accept normalized coordinates in that order.
fn variation_axis(data: &[u8], tag: &[u8; 4]) -> (usize, usize) {
    let u16_at = |i| u16::from_be_bytes(data[i..i + 2].try_into().expect("font u16")) as usize;
    let u32_at = |i| u32::from_be_bytes(data[i..i + 4].try_into().expect("font u32")) as usize;
    for table in 0..u16_at(4) {
        let record = 12 + table * 16;
        if &data[record..record + 4] == b"fvar" {
            let offset = u32_at(record + 8);
            let axes = offset + u16_at(offset + 4);
            let count = u16_at(offset + 8);
            let size = u16_at(offset + 10);
            for index in 0..count {
                let axis = axes + index * size;
                if &data[axis..axis + 4] == tag {
                    return (count, index);
                }
            }
        }
    }
    panic!("cache stress requires a variable font with a wght axis");
}
