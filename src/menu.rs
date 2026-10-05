//! A restaurant menu card in which every line of text is centred, drawn with femtovg on
//! `renderer::WGPURenderer` and saved as a PNG.
//!
//! usage: menu-<revision> <NewYork.ttf> <NewYorkItalic.ttf> <out.png> <glyphs.tsv>
//!
//! `Card::centered_text` draws every line the same way: it translates the canvas to the centre
//! of the card and the baseline of the line and calls `fill_text(0.0, 0.0, ..)` with
//! `Align::Center`. femtovg lays the line out from -width / 2 to +width / 2, so the glyphs in the
//! left half of every line have a negative x.
//!
//! The frame is drawn once, top to bottom, at device pixel ratio 1. glyphs.tsv and the lines
//! printed on stdout come from replay.rs.

mod offscreen;
mod replay;

use femtovg::{renderer::WGPURenderer, Align, Canvas, Color, FontId, Paint, Path, TextContext};

const WIDTH: u32 = 392;
const HEIGHT: u32 = 912;
/// The x of the centre of the card.
const CX: f32 = 196.0;

/// The courses, separated by blank lines. The first line of a course is its heading; every
/// other line is `dish | description | price`. search/menu_climb.py chose which dishes are
/// listed, their order, the prices and the order of the ingredients in each description.
const MENU: &str = include_str!("menu.txt");

struct Style {
    paint: Paint,
    line_height: f32,
}

/// New York is a variable font. `opsz` selects its optical size, which defaults to the largest
/// (a display cut), and `wght` its weight.
fn style(font: FontId, size: f32, color: Color, line_height: f32, variations: &[(&[u8; 4], f32)]) -> Style {
    let mut paint = Paint::color(color)
        .with_font(&[font])
        .with_font_size(size)
        .with_text_align(Align::Center);
    for (axis, value) in variations {
        paint.set_font_variation(axis, *value);
    }
    Style { paint, line_height }
}

struct Card {
    canvas: Canvas<WGPURenderer>,
    /// The y of the last baseline drawn, or of the bottom of whatever else was drawn last.
    y: f32,
    replay: replay::Replay,
}

impl Card {
    /// Draws `text` centred on `cx`, with its baseline at `baseline`.
    fn centered_text(&mut self, cx: f32, baseline: f32, text: &str, paint: &Paint) {
        self.canvas.save();
        self.canvas.translate(cx, baseline);
        let metrics = self.canvas.fill_text(0.0, 0.0, text, paint).expect("fill_text");
        self.canvas.restore();
        // Not part of the drawing: keeps the glyph positions fill_text used, for glyphs.tsv.
        self.replay.record(text, paint.font_size(), &metrics);
    }

    /// Draws `text` one line height below the previous line.
    fn line(&mut self, text: &str, style: &Style) {
        self.y += style.line_height;
        self.centered_text(CX, self.y, text, &style.paint);
    }

    fn draw(&mut self, roman: FontId, italic: FontId) {
        self.canvas
            .clear_rect(0, 0, WIDTH, HEIGHT, Color::rgb(0xec, 0xec, 0xea));

        let (card_x, card_y, card_w, card_h) = (16.0, 20.0, 360.0, 872.0);
        let mut card = Path::new();
        card.rounded_rect(card_x, card_y, card_w, card_h, 6.0);
        self.canvas.fill_path(&card, &Paint::color(Color::white()));
        let mut border = Path::new();
        border.rounded_rect(card_x + 0.5, card_y + 0.5, card_w - 1.0, card_h - 1.0, 6.0);
        self.canvas.stroke_path(
            &border,
            &Paint::color(Color::rgb(0xc8, 0xc8, 0xc4)).with_line_width(1.0),
        );

        let ink = Color::rgb(0x1c, 0x1c, 0x1c);
        let title = style(roman, 22.0, ink, 32.0, &[(b"opsz", 22.0), (b"wght", 674.0)]);
        let caption = style(italic, 12.0, Color::rgb(0x6a, 0x6a, 0x6a), 20.0, &[(b"opsz", 12.0)]);
        let course = style(
            roman,
            11.0,
            Color::rgb(0x8a, 0x5a, 0x2a),
            20.0,
            &[(b"opsz", 12.0), (b"wght", 674.0)],
        );
        let dish = style(roman, 12.0, ink, 17.0, &[(b"opsz", 12.0)]);
        let note = style(roman, 12.0, Color::rgb(0x5c, 0x5c, 0x5c), 17.0, &[(b"opsz", 12.0)]);

        self.y = 34.0;
        self.line("Trattoria Esempio", &title);
        self.line("Dinner, Tuesday to Sunday", &caption);
        let mut rule = Path::new();
        rule.rect(CX - 60.0, self.y + 10.0, 120.0, 1.0);
        self.canvas
            .fill_path(&rule, &Paint::color(Color::rgb(0xd0, 0xd0, 0xcc)));
        self.y += 14.0;

        for block in MENU.trim_end().split("\n\n") {
            let mut lines = block.lines();
            self.y += 14.0;
            self.line(lines.next().expect("course heading"), &course);
            self.y += 2.0;
            for entry in lines {
                let [name, description, price] = entry.split(" | ").collect::<Vec<_>>()[..] else {
                    panic!("expected `dish | description | price`, found {entry:?}");
                };
                self.line(&format!("{name} \u{b7} {price}"), &dish);
                self.line(description, &note);
                self.y += 5.0;
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, roman_path, italic_path, out_png, out_tsv] = args.as_slice() else {
        eprintln!(
            "usage: {} <NewYork.ttf> <NewYorkItalic.ttf> <out.png> <glyphs.tsv>",
            args[0]
        );
        std::process::exit(2);
    };
    let read = |path: &String| std::fs::read(path).unwrap_or_else(|error| panic!("cannot read {path}: {error}"));

    let gpu = offscreen::Gpu::new();
    eprintln!("{}: drawing on {}", args[0], gpu.adapter_name);
    let text_context = TextContext::default();
    let roman = text_context.add_font_mem(&read(roman_path)).expect("add_font_mem");
    let italic = text_context.add_font_mem(&read(italic_path)).expect("add_font_mem");
    let mut card = Card {
        canvas: gpu.canvas(text_context, WIDTH, HEIGHT),
        y: 0.0,
        replay: replay::Replay::default(),
    };
    card.draw(roman, italic);

    gpu.read_frame(&mut card.canvas, WIDTH, HEIGHT)
        .save(out_png)
        .unwrap_or_else(|error| panic!("cannot write {out_png}: {error}"));
    std::fs::write(out_tsv, card.replay.table()).unwrap_or_else(|error| panic!("cannot write {out_tsv}: {error}"));
    print!("{}", card.replay.summary());
}
