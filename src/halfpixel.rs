//! Text under a canvas translation of half a pixel right and down, for the half-pixel bug.
//!
//! usage: halfpixel-<revision> <font.ttf> <out.png>
//!
//! The top of the frame has each of 62 characters at 9, 12 and 16 px, drawn 12 times one below
//! the other. Below that are three grids of one character each (d, g and o at 12 px), 14 copies
//! across and 6 down. Every copy of a glyph should be drawn the same way. Prints each character of
//! the top part whose copies are drawn in more than one way, and how many ways each grid's copies
//! are drawn in.
//!
//! The grids show the bug only below the top part. Drawn alone, nearly every copy is the same.

mod offscreen;

use femtovg::{Color, Paint, TextContext};
use std::collections::HashSet;

const CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const SIZES: [f32; 3] = [9.0, 12.0, 16.0];
/// The number of copies of each glyph in the top part, one below the other.
const COPIES: u32 = 12;
/// The side of the square that each copy in the top part is drawn in.
const CELL: u32 = 20;
/// The number of squares in a row of the top part.
const COLUMNS: u32 = 48;

const GRID_CHARS: [char; 3] = ['d', 'g', 'o'];
const GRID_SIZE: f32 = 12.0;
/// Where the grids start, below the top part.
const GRID_TOP: u32 = 980;
const GRID_COLUMNS: u32 = 14;
const GRID_ROWS: u32 = 6;
/// The distance between two copies in a grid, across and down.
const GRID_DX: u32 = 9;
const GRID_DY: u32 = 16;
/// The width of a grid with its margins, and the gap to the next grid.
const GRID_WIDTH: u32 = GRID_COLUMNS * GRID_DX + 8;
const GRID_GAP: u32 = 2 * GRID_DX;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, font_path, out_path] = args.as_slice() else {
        panic!("usage: halfpixel <font.ttf> <out.png>");
    };
    let chars: Vec<char> = CHARS.chars().collect();
    let glyphs = (chars.len() * SIZES.len()) as u32;
    let width = CELL * COLUMNS;
    let height = GRID_TOP + GRID_ROWS * GRID_DY + GRID_SIZE as u32 + 8;
    let gpu = offscreen::Gpu::new();
    let text_context = TextContext::default();
    let font = text_context.add_font_file(font_path).expect("cannot read the font");
    let mut canvas = gpu.canvas(text_context, width, height);
    canvas.clear_rect(0, 0, width, height, Color::white());
    canvas.translate(0.5, 0.5);

    // The square of copy `copy` of glyph `i` in the top part: the glyphs fill the rows, each with
    // its copies in the squares below it.
    let square = |i: u32, copy: u32| ((i % COLUMNS) * CELL, ((i / COLUMNS) * COPIES + copy) * CELL);
    // The first copy of every glyph is drawn first, so the atlas cells are in the order of the
    // glyphs and the other copies reuse them.
    for copy in 0..COPIES {
        for (s, size) in SIZES.iter().enumerate() {
            let paint = Paint::color(Color::black()).with_font(&[font]).with_font_size(*size);
            for (c, ch) in chars.iter().enumerate() {
                let (x, y) = square((s * chars.len() + c) as u32, copy);
                canvas.fill_text(x as f32 + 4.0, y as f32 + 15.0, ch.to_string(), &paint).expect("fill_text");
            }
        }
    }

    // The top left corner of the square of copy (column, row) of grid `k`.
    let grid_square = |k: u32, column: u32, row: u32| {
        (k * (GRID_WIDTH + GRID_GAP) + 4 + column * GRID_DX, GRID_TOP + 4 + row * GRID_DY)
    };
    let paint = Paint::color(Color::black()).with_font(&[font]).with_font_size(GRID_SIZE);
    for row in 0..GRID_ROWS {
        for (k, ch) in GRID_CHARS.iter().enumerate() {
            for column in 0..GRID_COLUMNS {
                let (x, y) = grid_square(k as u32, column, row);
                canvas.fill_text(x as f32, y as f32 + GRID_SIZE, ch.to_string(), &paint).expect("fill_text");
            }
        }
    }

    let frame = gpu.read_frame(&mut canvas, width, height);
    frame.save(out_path).expect("cannot write the frame");

    // The pixels of a rectangle of the frame, one value per pixel.
    let pixels = |x0: u32, y0: u32, w: u32, h: u32| -> Vec<u8> {
        (y0..y0 + h).flat_map(|y| (x0..x0 + w).map(move |x| (x, y))).map(|(x, y)| frame.get_pixel(x, y).0[0]).collect()
    };
    let mut inconsistent = 0;
    for i in 0..glyphs {
        let ways = (0..COPIES)
            .map(|copy| {
                let (x, y) = square(i, copy);
                pixels(x, y, CELL, CELL)
            })
            .collect::<HashSet<_>>()
            .len();
        if ways > 1 {
            inconsistent += 1;
            let (s, c) = (i as usize / chars.len(), i as usize % chars.len());
            println!("{} px {}: {ways} ways", SIZES[s], chars[c]);
        }
    }
    println!("glyphs drawn in more than one way: {inconsistent} of {glyphs}");
    // The copies on the edges of a grid are left out, because the copies around them differ.
    for (k, ch) in GRID_CHARS.iter().enumerate() {
        let ways = (1..GRID_ROWS - 1)
            .flat_map(|row| (1..GRID_COLUMNS - 1).map(move |column| (column, row)))
            .map(|(column, row)| {
                let (x, y) = grid_square(k as u32, column, row);
                pixels(x, y, GRID_DX, GRID_DY)
            })
            .collect::<HashSet<_>>()
            .len();
        println!("grid of {ch}: the inner copies are drawn in {ways} ways");
    }
}
