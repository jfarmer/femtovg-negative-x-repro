//! Puts the same rectangle of two renders of a scene into one picture, with a caption over each.
//!
//! usage: compose side <font.ttf> <first.png> <caption> <second.png> <caption> <x> <y> <w> <h> <out.png>
//!        compose stack <font.ttf> <first.png> <caption> <second.png> <caption> <x> <y> <w> <h> <out.png>
//!        compose zoom <font.ttf> <first.png> <caption> <second.png> <caption> <x> <y> <w> <h> <out.png>
//!        compose pairs <font.ttf> <first.png> <caption> <second.png> <caption> <x> <y> <w> <h> <step> <count> <out.png>
//!
//! `side` puts the first rectangle to the left of the second, pixel for pixel.
//! `stack` puts the first rectangle above the second, pixel for pixel.
//! `zoom` draws every pixel of the rectangle as a 4 x 4 square and puts the first above the second.
//! `pairs` takes `count` rectangles, each `step` pixels right of the one before, draws every pixel
//! of them as a 3 x 3 square, and puts each rectangle of the first render next to the same one of
//! the second, one pair to a row. The gap between the two of a pair is narrower than the gap
//! between rows, so that each pair reads as one group.
//!
//! The pixels of the renders are copied on the CPU and never resampled or blended. Only the
//! caption bars are drawn with femtovg. run.sh builds this program against the fixed revision,
//! and the captions are at a positive x.

mod offscreen;

use femtovg::{Color, Paint, TextContext};
use image::{Rgba, RgbaImage};

const ZOOM: u32 = 4;
const CAPTION_HEIGHT: u32 = 24;
/// Width of the bar between the two halves of `side`.
const GAP: u32 = 16;
/// How large `pairs` draws each pixel. At 4, three pairs of the half-pixel grids do not fit on a
/// laptop screen.
const PAIRS_ZOOM: u32 = 3;
/// The gap between the two rectangles of a pair, and between two pairs, in `pairs`.
const PAIR_GAP: u32 = 8;
const ROW_GAP: u32 = 40;
const BAR: Rgba<u8> = Rgba([40, 44, 52, 255]);

/// A bar `width` pixels wide with `text` in white.
fn caption(gpu: &offscreen::Gpu, font_data: &[u8], width: u32, text: &str) -> RgbaImage {
    let text_context = TextContext::default();
    let font = text_context.add_font_mem(font_data).expect("add_font_mem");
    let mut canvas = gpu.canvas(text_context, width, CAPTION_HEIGHT);
    canvas.clear_rect(0, 0, width, CAPTION_HEIGHT, Color::rgb(BAR[0], BAR[1], BAR[2]));
    let paint = Paint::color(Color::white()).with_font(&[font]).with_font_size(13.0);
    canvas.fill_text(8.0, 17.0, text, &paint).expect("fill_text");
    gpu.read_frame(&mut canvas, width, CAPTION_HEIGHT)
}

/// `compose pairs`: see the module comment.
fn pairs(args: &[String]) {
    let [_, _, font_path, first_path, first_caption, second_path, second_caption, x, y, w, h, step, count, out_path] = args
    else {
        eprintln!(
            "usage: {} pairs <font.ttf> <first.png> <caption> <second.png> <caption> <x> <y> <w> <h> <step> <count> <out.png>",
            args[0]
        );
        std::process::exit(2);
    };
    let [x, y, w, h, step, count] =
        [x, y, w, h, step, count].map(|value| value.parse::<u32>().expect("the numbers are whole numbers"));
    let font_data = std::fs::read(font_path).unwrap_or_else(|error| panic!("cannot read {font_path}: {error}"));
    let open = |path: &String| {
        let picture = image::open(path)
            .unwrap_or_else(|error| panic!("cannot read {path}: {error}"))
            .to_rgba8();
        assert!(
            x + (count - 1) * step + w <= picture.width() && y + h <= picture.height(),
            "the rectangles do not fit in {path}"
        );
        picture
    };
    let pictures = [open(first_path), open(second_path)];
    let gpu = offscreen::Gpu::new();

    let (block_w, block_h) = (w * PAIRS_ZOOM, h * PAIRS_ZOOM);
    let out_w = 2 * block_w + PAIR_GAP;
    let out_h = CAPTION_HEIGHT + ROW_GAP / 2 + count * block_h + (count - 1) * ROW_GAP + ROW_GAP / 2;
    let mut out = RgbaImage::from_pixel(out_w, out_h, Rgba([255, 255, 255, 255]));
    for (column, name) in [first_caption, second_caption].into_iter().enumerate() {
        let text = format!("{name}    each pixel drawn {PAIRS_ZOOM} x {PAIRS_ZOOM}");
        let left = column as u32 * (block_w + PAIR_GAP);
        image::imageops::replace(&mut out, &caption(&gpu, &font_data, block_w, &text), left.into(), 0);
    }
    for i in 0..count {
        let top = CAPTION_HEIGHT + ROW_GAP / 2 + i * (block_h + ROW_GAP);
        for (column, picture) in pictures.iter().enumerate() {
            let left = column as u32 * (block_w + PAIR_GAP);
            let x0 = x + i * step;
            // Every pixel of the rectangle, repeated PAIRS_ZOOM times in each direction.
            let block = RgbaImage::from_fn(block_w, block_h, |c, r| {
                *picture.get_pixel(x0 + c / PAIRS_ZOOM, y + r / PAIRS_ZOOM)
            });
            image::imageops::replace(&mut out, &block, left.into(), top.into());
        }
    }
    out.save(out_path)
        .unwrap_or_else(|error| panic!("cannot write {out_path}: {error}"));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|mode| mode == "pairs") {
        pairs(&args);
        return;
    }
    let [_, mode, font_path, first_path, first_caption, second_path, second_caption, x, y, w, h, out_path] =
        args.as_slice()
    else {
        eprintln!(
            "usage: {} side|stack|zoom <font.ttf> <first.png> <caption> <second.png> <caption> <x> <y> <w> <h> <out.png>",
            args[0]
        );
        std::process::exit(2);
    };
    let [x, y, w, h] = [x, y, w, h].map(|value| value.parse::<u32>().expect("x, y, w and h are whole numbers"));
    let (factor, stacked) = match mode.as_str() {
        "side" => (1, false),
        "stack" => (1, true),
        "zoom" => (ZOOM, true),
        other => panic!("unknown mode {other}"),
    };
    let font_data = std::fs::read(font_path).unwrap_or_else(|error| panic!("cannot read {font_path}: {error}"));
    let open = |path: &String| {
        let picture = image::open(path)
            .unwrap_or_else(|error| panic!("cannot read {path}: {error}"))
            .to_rgba8();
        assert!(
            x + w <= picture.width() && y + h <= picture.height(),
            "the rectangle does not fit in {path}"
        );
        picture
    };
    let gpu = offscreen::Gpu::new();

    let (block_w, block_h) = (w * factor, CAPTION_HEIGHT + h * factor);
    let (out_w, out_h, second_left, second_top) = if stacked {
        (block_w, 2 * block_h, 0, block_h)
    } else {
        (2 * block_w + GAP, block_h, block_w + GAP, 0)
    };
    let mut out = RgbaImage::from_pixel(out_w, out_h, BAR);
    for (path, name, left, top) in [
        (first_path, first_caption, 0, 0),
        (second_path, second_caption, second_left, second_top),
    ] {
        let picture = open(path);
        let text = if factor > 1 {
            format!(
                "{name}    x {x}..{}, y {y}..{}, each pixel drawn {factor} x {factor}",
                x + w,
                y + h
            )
        } else {
            name.clone()
        };
        image::imageops::replace(
            &mut out,
            &caption(&gpu, &font_data, block_w, &text),
            left.into(),
            top.into(),
        );
        // Every pixel of the rectangle, repeated `factor` times in each direction.
        let block = RgbaImage::from_fn(block_w, h * factor, |column, row| {
            *picture.get_pixel(x + column / factor, y + row / factor)
        });
        image::imageops::replace(&mut out, &block, left.into(), (top + CAPTION_HEIGHT).into());
    }
    out.save(out_path)
        .unwrap_or_else(|error| panic!("cannot write {out_path}: {error}"));
}
