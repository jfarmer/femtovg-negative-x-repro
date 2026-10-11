//! Two scenes for the shaping-key bugs, drawn with femtovg on `renderer::WGPURenderer` and saved
//! as a PNG.
//!
//! usage: shaping-<revision> <RobotoFlex-VariableFont.ttf> size|wrap <out.png>
//!
//! femtovg keeps shaped text in two caches whose key is a `ShapingId` (src/text/textlayout.rs in
//! femtovg). On a revision where `ShapingId::new` keys the font size as `(font_size * 10.0).trunc()`
//! and `max_width` as `max_width.trunc()`, text at two font sizes in one tenth of a pixel shares
//! one entry, and so does text broken at two widths in one whole pixel. The entry holds what the
//! first call computed. Both caches are LRU caches of 1000 entries, so the entry is gone after
//! 1000 other strings have been shaped, and the next call computes the result again.
//!
//! Both scenes do what an application does: lay a box out from one call, paint its text from a
//! later call. Between the two calls they shape `OTHER_STRINGS` other strings, so on a revision
//! with the bug the layout call gets the entry an earlier call left and the paint call does not.
//!
//! `size` draws one sentence at 10 px and then at 10.09 px. The 10.09 px line is measured for the
//! width of its cell, and painted after the other strings. On a revision with the bug the cell is
//! as wide as the 10 px line and the text runs through its right border.
//!
//! `wrap` breaks one paragraph into two boxes, the second 0.2 px narrower than the first and
//! both within one whole pixel of the paragraph's first `WORDS_ON_LINE_1` words. Each box has a
//! button directly below it. The second box is sized from its line count and painted after the
//! other strings. On a revision with the bug the layout gets the first box's line count, the
//! paint breaks one line more, and the last line is drawn over the button.
//!
//! The picture is as wide as its text needs. The frame is drawn once at device pixel ratio 1
//! with no canvas transform, so the font size and the width reach the cache key unchanged. Every
//! glyph is at a positive x. Each scene prints a table on stdout, computed from what
//! `measure_text`, `break_text_vec` and `fill_text` returned.

mod offscreen;

use femtovg::{renderer::WGPURenderer, Canvas, Color, FontId, Paint, Path, TextContext};
use std::fmt::Write as _;
use std::ops::Range;

/// More than the 1000 entries of each shaping cache, so that every entry made before these
/// strings were shaped is gone afterwards.
const OTHER_STRINGS: usize = 1100;

/// The space between the card and the picture's edge, and between the card and its text.
const MARGIN: f32 = 20.0;
/// The x that every line starts at.
const LEFT: f32 = 2.0 * MARGIN;
const HEADING_BASELINE: f32 = 52.0;

/// On a revision with the bug a line painted at size s is wider than its cell by its width at
/// 10 px times (s - 10) / 10, so the sentence is long.
const SENTENCE: &str = "Still legible, a little tighter: the title fits the line, the little filler fills it, \
and the last letter still sits flush at the little tick, as it did at ten pixels.";
const FIRST_SIZE: f32 = 10.0;
/// In the same tenth of a pixel as `FIRST_SIZE`, and as far from it as that allows.
const SECOND_SIZE: f32 = 10.09;
const ROW_HEIGHT: f32 = 48.0;

const PARAGRAPH: &str = "Text that is broken into lines is measured word by word, and a word goes to the \
next line when the line would be wider than the box. A box that is a fraction of a pixel wider \
than the first line should keep the same words on it, and a box that is narrower should not.";
const PARAGRAPH_SIZE: f32 = 13.0;
const LINE_HEIGHT: f32 = 17.0;
/// The first box is 0.1 px wider than the first `WORDS_ON_LINE_1` words of the paragraph, and
/// the second 0.1 px narrower.
const WORDS_ON_LINE_1: usize = 7;
const BOX_TOP: f32 = 76.0;
/// The border of a box or a cell is drawn this far outside the text on every side.
const PADDING: f32 = 2.0;
/// The space between the borders of the two boxes.
const BOX_GAP: f32 = 32.0;
const BUTTON_HEIGHT: f32 = 26.0;

struct Scene {
    canvas: Canvas<WGPURenderer>,
    width: u32,
    height: u32,
    font: FontId,
    /// The table that the README quotes.
    table: String,
}

/// The number of lines `break_text_vec` gives the paragraph at `max_width` through a
/// `TextContext` with nothing in its caches.
fn lines_in_new_context(font_data: &[u8], max_width: f32) -> usize {
    let context = TextContext::default();
    let font = context.add_font_mem(font_data).expect("add_font_mem");
    let paint = Paint::color(Color::black())
        .with_font(&[font])
        .with_font_size(PARAGRAPH_SIZE);
    context
        .break_text_vec(max_width, PARAGRAPH, &paint)
        .expect("break_text_vec")
        .len()
}

impl Scene {
    fn text_paint(&self, size: f32, color: Color) -> Paint {
        Paint::color(color).with_font(&[self.font]).with_font_size(size)
    }

    fn card(&mut self) {
        let (width, height) = (self.width, self.height);
        self.canvas.clear_rect(0, 0, width, height, Color::rgb(236, 236, 240));
        let (w, h) = (width as f32 - 2.0 * MARGIN, height as f32 - 2.0 * MARGIN);
        let mut card = Path::new();
        card.rounded_rect(MARGIN, MARGIN, w, h, 8.0);
        self.canvas.fill_path(&card, &Paint::color(Color::white()));
        let mut border = Path::new();
        border.rounded_rect(MARGIN + 0.5, MARGIN + 0.5, w - 1.0, h - 1.0, 8.0);
        self.canvas
            .stroke_path(&border, &Paint::color(Color::rgb(200, 200, 204)).with_line_width(1.0));
    }

    /// A 1 px red border `PADDING` outside the rectangle of text at (x, y) with the given size.
    fn border(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let mut border = Path::new();
        border.rect(x - PADDING, y - PADDING, width + 2.0 * PADDING, height + 2.0 * PADDING);
        self.canvas
            .stroke_path(&border, &Paint::color(Color::rgb(226, 80, 64)).with_line_width(1.0));
    }

    /// A button as wide as the border of a box, directly below it.
    fn button(&mut self, x: f32, top: f32, width: f32) {
        let mut button = Path::new();
        button.rounded_rect(x - PADDING, top, width + 2.0 * PADDING, BUTTON_HEIGHT, 4.0);
        self.canvas.fill_path(&button, &Paint::color(Color::rgb(58, 110, 214)));
        let paint = self.text_paint(PARAGRAPH_SIZE, Color::white());
        self.canvas
            .fill_text(x + 8.0, top + 17.0, "Continue", &paint)
            .expect("fill_text");
    }

    /// Shapes `OTHER_STRINGS` strings that no other call in the scene uses, so that every entry
    /// made before this is gone from both caches afterwards.
    fn shape_other_strings(&mut self) {
        let paint = self.text_paint(PARAGRAPH_SIZE, Color::black());
        for index in 0..OTHER_STRINGS {
            self.canvas
                .measure_text(0.0, 0.0, format!("w{index}"), &paint)
                .expect("measure_text");
        }
    }

    fn draw_size(&mut self) {
        self.card();
        let ink = Color::rgb(20, 20, 24);
        let grey = Color::rgb(96, 96, 104);
        let heading = self.text_paint(13.0, ink);
        let label = self.text_paint(10.0, grey);
        let sample = self.text_paint(FIRST_SIZE, ink);
        let second = sample.clone().with_font_size(SECOND_SIZE);

        self.canvas
            .fill_text(
                LEFT,
                HEADING_BASELINE,
                &format!("Roboto Flex, one sentence at {FIRST_SIZE} px, then at {SECOND_SIZE} px in a cell as wide as it measures"),
                &heading,
            )
            .expect("fill_text");
        writeln!(self.table, "{:<46}  {:>8}", "call", "width").unwrap();

        // The 10 px line, laid out and painted at once.
        let baseline = HEADING_BASELINE + 32.0;
        self.canvas
            .fill_text(LEFT, baseline, &format!("{FIRST_SIZE} px"), &label)
            .expect("fill_text");
        let metrics = self
            .canvas
            .fill_text(LEFT, baseline + 20.0, SENTENCE, &sample)
            .expect("fill_text");
        writeln!(
            self.table,
            "{:<46}  {:>8.3}",
            format!("fill_text at {FIRST_SIZE} px"),
            metrics.width()
        )
        .unwrap();

        // Layout: the cell of the 10.09 px line is as wide as the line measures now.
        let baseline = baseline + ROW_HEIGHT;
        self.canvas
            .fill_text(
                LEFT,
                baseline,
                &format!("{SECOND_SIZE} px, measured for its cell, painted {OTHER_STRINGS} strings later"),
                &label,
            )
            .expect("fill_text");
        let measured = self
            .canvas
            .measure_text(LEFT, baseline + 20.0, SENTENCE, &second)
            .expect("measure_text");
        writeln!(
            self.table,
            "{:<46}  {:>8.3}",
            format!("measure_text at {SECOND_SIZE} px, for the cell"),
            measured.width()
        )
        .unwrap();
        self.border(LEFT, baseline + 20.0 - 10.0, measured.width(), 14.0);

        // Paint, after the caches have turned over.
        self.shape_other_strings();
        let painted = self
            .canvas
            .fill_text(LEFT, baseline + 20.0, SENTENCE, &second)
            .expect("fill_text");
        writeln!(
            self.table,
            "{:<46}  {:>8.3}",
            format!("fill_text at {SECOND_SIZE} px, {OTHER_STRINGS} strings later"),
            painted.width()
        )
        .unwrap();
        writeln!(
            self.table,
            "the painted line is {:.3} px wider than its cell",
            painted.width() - measured.width()
        )
        .unwrap();
    }

    /// Draws `lines` of the paragraph at `left`, the first on `baseline`.
    fn paint_lines(&mut self, left: f32, mut baseline: f32, lines: &[Range<usize>], body: &Paint) {
        for range in lines {
            self.canvas
                .fill_text(left, baseline, &PARAGRAPH[range.clone()], body)
                .expect("fill_text");
            baseline += LINE_HEIGHT;
        }
    }

    fn draw_wrap(&mut self) {
        self.card();
        let ink = Color::rgb(20, 20, 24);
        let grey = Color::rgb(96, 96, 104);
        let heading = self.text_paint(13.0, ink);
        let label = self.text_paint(10.0, grey);
        let body = self.text_paint(PARAGRAPH_SIZE, ink);

        self.canvas
            .fill_text(
                LEFT,
                HEADING_BASELINE,
                &format!("Box 2 is 0.2 px narrower, painted {OTHER_STRINGS} strings later"),
                &heading,
            )
            .expect("fill_text");

        let (first_words, first_width) = first_words_width(&self.canvas, &body);
        writeln!(
            self.table,
            "the first {WORDS_ON_LINE_1} words, \"{first_words}\", are {first_width:.3} px wide"
        )
        .unwrap();
        let wide = first_width + 0.1;
        let narrow = first_width - 0.1;
        let box_2_left = LEFT + wide + 2.0 * PADDING + BOX_GAP;

        // Box 1, laid out and painted at once.
        self.canvas
            .fill_text(LEFT, BOX_TOP - 10.0, &format!("box 1, {wide:.3} px"), &label)
            .expect("fill_text");
        let lines = self
            .canvas
            .break_text_vec(wide, PARAGRAPH, &body)
            .expect("break_text_vec");
        writeln!(self.table, "box 1 at {wide:.3} px: {} lines", lines.len()).unwrap();
        let box_height = lines.len() as f32 * LINE_HEIGHT;
        self.border(LEFT, BOX_TOP, wide, box_height);
        self.button(LEFT, BOX_TOP + box_height + 2.0 * PADDING, wide);
        self.paint_lines(LEFT, BOX_TOP + PARAGRAPH_SIZE, &lines, &body);

        // Box 2: laid out now, with its button.
        self.canvas
            .fill_text(box_2_left, BOX_TOP - 10.0, &format!("box 2, {narrow:.3} px"), &label)
            .expect("fill_text");
        let lines = self
            .canvas
            .break_text_vec(narrow, PARAGRAPH, &body)
            .expect("break_text_vec");
        writeln!(
            self.table,
            "box 2 at {narrow:.3} px, for the layout: {} lines",
            lines.len()
        )
        .unwrap();
        let box_height = lines.len() as f32 * LINE_HEIGHT;
        self.border(box_2_left, BOX_TOP, narrow, box_height);
        self.button(box_2_left, BOX_TOP + box_height + 2.0 * PADDING, narrow);

        // Paint, after the caches have turned over.
        self.shape_other_strings();
        let lines = self
            .canvas
            .break_text_vec(narrow, PARAGRAPH, &body)
            .expect("break_text_vec");
        writeln!(
            self.table,
            "box 2 at {narrow:.3} px, for the paint, {OTHER_STRINGS} strings later: {} lines",
            lines.len()
        )
        .unwrap();
        self.paint_lines(box_2_left, BOX_TOP + PARAGRAPH_SIZE, &lines, &body);
    }
}

/// The first `WORDS_ON_LINE_1` words of the paragraph without the space after them, and their
/// width. shape_run adds the words of a line in this order, so this is the width it compares
/// with max_width when it reaches the last of them.
fn first_words_width(canvas: &Canvas<WGPURenderer>, body: &Paint) -> (String, f32) {
    let words: Vec<&str> = PARAGRAPH.split(' ').take(WORDS_ON_LINE_1).collect();
    let words = words.join(" ");
    let width = canvas
        .measure_text(0.0, 0.0, &words, body)
        .expect("measure_text")
        .width();
    (words, width)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, font_path, scene, out_png] = args.as_slice() else {
        eprintln!("usage: {} <RobotoFlex-VariableFont.ttf> size|wrap <out.png>", args[0]);
        std::process::exit(2);
    };
    let font_data = std::fs::read(font_path).unwrap_or_else(|error| panic!("cannot read {font_path}: {error}"));

    let gpu = offscreen::Gpu::new();
    eprintln!("{}: drawing on {}", args[0], gpu.adapter_name);
    let text_context = TextContext::default();
    let font = text_context.add_font_mem(&font_data).expect("add_font_mem");
    let ink = Paint::color(Color::black()).with_font(&[font]);

    // The size of the picture comes from the text. These measurements leave the same entries in
    // the caches as the first calls of the scene do, because they are the same calls.
    let (width, height, draw): (f32, f32, fn(&mut Scene)) = match scene.as_str() {
        "size" => {
            let first = text_context
                .measure_text(0.0, 0.0, SENTENCE, &ink.with_font_size(FIRST_SIZE))
                .expect("measure_text")
                .width();
            // Wide enough for the 10.09 px line, which is wider by SECOND_SIZE / FIRST_SIZE.
            let width = LEFT + first * SECOND_SIZE / FIRST_SIZE + LEFT;
            (
                width,
                HEADING_BASELINE + 32.0 + 2.0 * ROW_HEIGHT + MARGIN,
                Scene::draw_size,
            )
        }
        "wrap" => {
            let body = ink.with_font_size(PARAGRAPH_SIZE);
            let words: Vec<&str> = PARAGRAPH.split(' ').take(WORDS_ON_LINE_1).collect();
            let first_width = text_context
                .measure_text(0.0, 0.0, words.join(" "), &body)
                .expect("measure_text")
                .width();
            // Both box widths have to lie in one whole pixel, or they do not share a cache entry
            // on a revision with the bug.
            assert!(
                first_width.fract() > 0.1 && first_width.fract() < 0.9,
                "the first {WORDS_ON_LINE_1} words are {first_width} px wide, within 0.1 px of a whole number; \
                 change WORDS_ON_LINE_1 or the paragraph"
            );
            let (wide, narrow) = (first_width + 0.1, first_width - 0.1);
            // The narrower box needs one line more than the wider one, or the layout and the
            // paint agree on a revision with the bug too.
            let (wide_lines, narrow_lines) = (
                lines_in_new_context(&font_data, wide),
                lines_in_new_context(&font_data, narrow),
            );
            assert!(
                narrow_lines == wide_lines + 1,
                "the paragraph needs {wide_lines} lines at {wide} px and {narrow_lines} at {narrow} px; change the paragraph"
            );
            let width = LEFT + wide + 2.0 * PADDING + BOX_GAP + narrow + 2.0 * PADDING + LEFT;
            let height = BOX_TOP + narrow_lines as f32 * LINE_HEIGHT + 2.0 * PADDING + BUTTON_HEIGHT + MARGIN + MARGIN;
            (width, height, Scene::draw_wrap)
        }
        other => panic!("unknown scene {other}"),
    };
    let (width, height) = (width.ceil() as u32, height.ceil() as u32);
    eprintln!("{}: {width} x {height}", args[0]);

    let mut scene_state = Scene {
        canvas: gpu.canvas(text_context, width, height),
        width,
        height,
        font,
        table: String::new(),
    };
    draw(&mut scene_state);

    gpu.read_frame(&mut scene_state.canvas, width, height)
        .save(out_png)
        .unwrap_or_else(|error| panic!("cannot write {out_png}: {error}"));
    print!("{}", scene_state.table);
}
