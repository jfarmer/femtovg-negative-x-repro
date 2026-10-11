//! Two scenes for the shaping-key bugs, drawn with femtovg on `renderer::WGPURenderer` and saved
//! as a PNG.
//!
//! usage: shaping-<revision> <RobotoFlex-VariableFont.ttf> size|wrap <out.png>
//!
//! femtovg keeps shaped text in two caches whose key is a `ShapingId` (src/text/textlayout.rs in
//! femtovg). On a revision where `ShapingId::new` keys the font size as `(font_size * 10.0).trunc()`
//! and `max_width` as `max_width.trunc()`, text at two font sizes in one tenth of a pixel shares
//! one entry, and so does text broken at two widths in one whole pixel. The entry holds what the
//! first call computed.
//!
//! `size` draws one sentence four times, left-aligned at the same x, at font sizes 10, 10.03,
//! 10.06 and 10.09 px, the 10 px line first. `shape_word` scales every advance by the exact font
//! size, so each later line should be a little longer than the one above it. A red tick marks where
//! each line should end, from the same measurement through a new `TextContext`. On a revision with
//! the bug the later lines get the advances of the 10 px line and end short of their ticks.
//!
//! `wrap` breaks one paragraph into two boxes whose widths differ by 0.2 px and lie in one whole
//! pixel: the first box is 0.1 px narrower than the paragraph's first `WORDS_ON_LINE_1` words, the
//! second 0.1 px wider. `break_text_vec` should keep those words on the first line of the second
//! box and break before the last of them in the first. On a revision with the bug the second box
//! gets the first box's lines.
//!
//! The frame is drawn once at device pixel ratio 1 with no canvas transform, so the font size and
//! the width reach the cache key unchanged. Every glyph is at a positive x. Each scene prints a
//! table on stdout, computed from what `fill_text`, `measure_text` and `break_text_vec` returned
//! and from the same calls through a new `TextContext` for every row, whose cache is empty.

mod offscreen;

use femtovg::{renderer::WGPURenderer, Canvas, Color, FontId, Paint, Path, TextContext};
use std::fmt::Write as _;

const WIDTH: u32 = 720;
const SIZE_HEIGHT: u32 = 268;
const WRAP_HEIGHT: u32 = 260;

/// The x that every line of the `size` scene starts at.
const LEFT: f32 = 40.0;
/// On a revision with the bug a line drawn at size s falls short by its width at 10 px times
/// (s - 10) / 10, so the sentence is as long as the card allows at 10 px.
const SENTENCE: &str = "Still legible, a little tighter: the title fits the line, the little filler fills it, \
and the last letter still sits flush at the little tick, as it did at ten pixels.";
/// The font sizes of the four lines in the order they are drawn, all in one tenth of a pixel.
const SIZES: [f32; 4] = [10.0, 10.03, 10.06, 10.09];

const PARAGRAPH: &str = "Text that is broken into lines is measured word by word, and a word goes to the \
next line when the line would be wider than the box. A box that is a fraction of a pixel wider \
than the first line should keep the same words on it, and a box that is a fraction narrower \
should move the last of them down.";
const PARAGRAPH_SIZE: f32 = 13.0;
const LINE_HEIGHT: f32 = 17.0;
/// The first box is 0.1 px narrower than the first `WORDS_ON_LINE_1` words of the paragraph, and
/// the second 0.1 px wider.
const WORDS_ON_LINE_1: usize = 8;
const BOX_LEFT: [f32; 2] = [40.0, 388.0];
const BOX_TOP: f32 = 76.0;
const BOX_LINES: usize = 8;
/// The border of a box is drawn this far outside the text on every side.
const BOX_PADDING: f32 = 4.0;

struct Scene {
    canvas: Canvas<WGPURenderer>,
    font: FontId,
    font_data: Vec<u8>,
    /// The table that the README quotes.
    table: String,
}

impl Scene {
    fn text_paint(&self, size: f32, color: Color) -> Paint {
        Paint::color(color).with_font(&[self.font]).with_font_size(size)
    }

    /// A `TextContext` with nothing in its caches, for the reference rows of the tables. Each
    /// row gets its own, so that no reference measurement finds an earlier one in the cache.
    fn new_context(&self) -> (TextContext, Paint) {
        let context = TextContext::default();
        let font = context.add_font_mem(&self.font_data).expect("add_font_mem");
        (context, Paint::color(Color::black()).with_font(&[font]))
    }

    fn card(&mut self, height: u32) {
        self.canvas.clear_rect(0, 0, WIDTH, height, Color::rgb(236, 236, 240));
        let (x, y, w, h) = (16.0, 20.0, WIDTH as f32 - 32.0, height as f32 - 40.0);
        let mut card = Path::new();
        card.rounded_rect(x, y, w, h, 8.0);
        self.canvas.fill_path(&card, &Paint::color(Color::white()));
        let mut border = Path::new();
        border.rounded_rect(x + 0.5, y + 0.5, w - 1.0, h - 1.0, 8.0);
        self.canvas
            .stroke_path(&border, &Paint::color(Color::rgb(200, 200, 204)).with_line_width(1.0));
    }

    fn draw_size(&mut self) {
        self.card(SIZE_HEIGHT);
        let ink = Color::rgb(20, 20, 24);
        let grey = Color::rgb(96, 96, 104);
        let heading = self.text_paint(13.0, ink);
        let label = self.text_paint(10.0, grey);
        let sample = self.text_paint(SIZES[0], ink);

        self.canvas
            .fill_text(
                LEFT,
                52.0,
                "Roboto Flex, four font sizes in one tenth of a pixel, each line as long as its tick",
                &heading,
            )
            .expect("fill_text");

        writeln!(
            self.table,
            "font size  glyphs  TextMetrics::width()  new TextContext  ends at x  tick at x  ends left of its tick by"
        )
        .unwrap();
        let mut baseline = 84.0;
        for size in SIZES {
            let text = format!("font size {size} px");
            self.canvas.fill_text(LEFT, baseline, &text, &label).expect("fill_text");
            let paint = sample.clone().with_font_size(size);
            let metrics = self
                .canvas
                .fill_text(LEFT, baseline + 20.0, SENTENCE, &paint)
                .expect("fill_text");
            // The same measurement through a context that has shaped nothing, which is the width
            // the line should have.
            let (context, reference) = self.new_context();
            let fresh = context
                .measure_text(0.0, 0.0, SENTENCE, &reference.with_font_size(size))
                .expect("measure_text")
                .width();
            // A tick where the line should end, from above the x-height to below the baseline.
            let tick = LEFT + fresh;
            assert!(
                tick <= WIDTH as f32 - 40.0,
                "the sentence is too long for the card at {size} px: {fresh} px"
            );
            let mut mark = Path::new();
            mark.rect(tick, baseline + 10.0, 1.0, 14.0);
            self.canvas.fill_path(&mark, &Paint::color(Color::rgb(226, 80, 64)));

            // Where the line ends: the x of the last glyph without its offset_x, plus its advance.
            // That is where layout leaves its cursor after the last glyph.
            let last = metrics.glyphs.last().expect("the sentence has glyphs");
            let end = last.x - last.offset_x + last.advance_x;
            // Rounded to the 0.001 px that is printed. Adding 0.0 turns -0.0 into 0.0.
            let shortfall = ((tick - end) * 1000.0).round() / 1000.0 + 0.0;
            writeln!(
                self.table,
                "{size:>6.2} px  {:>6}  {:>20.3}  {fresh:>15.3}  {end:>9.3}  {tick:>9.3}  {shortfall:>21.3} px",
                metrics.glyphs.len(),
                metrics.width(),
            )
            .unwrap();
            baseline += 44.0;
        }
    }

    fn draw_wrap(&mut self) {
        self.card(WRAP_HEIGHT);
        let ink = Color::rgb(20, 20, 24);
        let grey = Color::rgb(96, 96, 104);
        let heading = self.text_paint(13.0, ink);
        let label = self.text_paint(10.0, grey);
        let body = self.text_paint(PARAGRAPH_SIZE, ink);

        self.canvas
            .fill_text(
                40.0,
                52.0,
                &format!("Roboto Flex {PARAGRAPH_SIZE} px, one paragraph in two boxes whose widths differ by 0.2 px"),
                &heading,
            )
            .expect("fill_text");

        // The width of the first WORDS_ON_LINE_1 words without the space after them. shape_run
        // adds the words of a line in this order, so this is the width it compares with max_width
        // when it reaches the last of them.
        let first_words: Vec<&str> = PARAGRAPH.split(' ').take(WORDS_ON_LINE_1).collect();
        let first_words = first_words.join(" ");
        let first_width = self
            .canvas
            .measure_text(0.0, 0.0, &first_words, &body)
            .expect("measure_text")
            .width();
        // Both box widths have to lie in one whole pixel, or they do not share a cache entry on a
        // revision with the bug.
        assert!(
            first_width.fract() > 0.1 && first_width.fract() < 0.9,
            "the first {WORDS_ON_LINE_1} words are {first_width} px wide, within 0.1 px of a whole number; \
             change WORDS_ON_LINE_1 or the paragraph"
        );
        writeln!(
            self.table,
            "the first {WORDS_ON_LINE_1} words, \"{first_words}\", are {first_width:.3} px wide"
        )
        .unwrap();

        for (index, left) in BOX_LEFT.into_iter().enumerate() {
            let max_width = if index == 0 {
                first_width - 0.1
            } else {
                first_width + 0.1
            };
            let text = format!(
                "box {}, max_width {max_width:.3} px, broken {}",
                index + 1,
                if index == 0 { "first" } else { "second" }
            );
            self.canvas
                .fill_text(left, BOX_TOP - 10.0, &text, &label)
                .expect("fill_text");
            let mut border = Path::new();
            border.rect(
                left - BOX_PADDING,
                BOX_TOP - BOX_PADDING,
                max_width + 2.0 * BOX_PADDING,
                BOX_LINES as f32 * LINE_HEIGHT + 2.0 * BOX_PADDING,
            );
            self.canvas
                .stroke_path(&border, &Paint::color(Color::rgb(226, 80, 64)).with_line_width(1.0));

            let lines = self
                .canvas
                .break_text_vec(max_width, PARAGRAPH, &body)
                .expect("break_text_vec");
            assert!(lines.len() <= BOX_LINES, "the paragraph needs {} lines", lines.len());
            writeln!(self.table, "box {} at max_width {max_width:.3} px:", index + 1).unwrap();
            let mut baseline = BOX_TOP + PARAGRAPH_SIZE;
            for range in &lines {
                let line = &PARAGRAPH[range.clone()];
                let metrics = self.canvas.fill_text(left, baseline, line, &body).expect("fill_text");
                writeln!(self.table, "  {:>8.3} px  \"{line}\"", metrics.width()).unwrap();
                baseline += LINE_HEIGHT;
            }

            // Not part of the drawing: the same break through a context that has broken
            // nothing, which is how the box should be broken.
            let (context, reference) = self.new_context();
            let fresh = context
                .break_text_vec(max_width, PARAGRAPH, &reference.with_font_size(PARAGRAPH_SIZE))
                .expect("break_text_vec");
            if fresh == lines {
                writeln!(self.table, "  a new TextContext breaks it the same way").unwrap();
            } else {
                writeln!(self.table, "  a new TextContext breaks it differently:").unwrap();
                for range in &fresh {
                    writeln!(self.table, "    \"{}\"", &PARAGRAPH[range.clone()]).unwrap();
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, font_path, scene, out_png] = args.as_slice() else {
        eprintln!("usage: {} <RobotoFlex-VariableFont.ttf> size|wrap <out.png>", args[0]);
        std::process::exit(2);
    };
    let font_data = std::fs::read(font_path).unwrap_or_else(|error| panic!("cannot read {font_path}: {error}"));
    let (height, draw): (u32, fn(&mut Scene)) = match scene.as_str() {
        "size" => (SIZE_HEIGHT, Scene::draw_size),
        "wrap" => (WRAP_HEIGHT, Scene::draw_wrap),
        other => panic!("unknown scene {other}"),
    };

    let gpu = offscreen::Gpu::new();
    eprintln!("{}: drawing on {}", args[0], gpu.adapter_name);
    let text_context = TextContext::default();
    let font = text_context.add_font_mem(&font_data).expect("add_font_mem");
    let mut scene_state = Scene {
        canvas: gpu.canvas(text_context, WIDTH, height),
        font,
        font_data,
        table: String::new(),
    };
    draw(&mut scene_state);

    gpu.read_frame(&mut scene_state.canvas, WIDTH, height)
        .save(out_png)
        .unwrap_or_else(|error| panic!("cannot write {out_png}: {error}"));
    print!("{}", scene_state.table);
}
