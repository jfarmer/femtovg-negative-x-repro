//! A glyph that a color bitmap font has no bitmap for, as three font libraries and two builds of
//! femtovg see it, drawn with femtovg on `renderer::WGPURenderer` and saved as PNGs.
//!
//! usage: cbdt-<build> row <out.png> <out.ttf>
//!        cbdt-default figure <font.ttf> <swash row.png> <out.png>
//!
//! The font is built here, and `row` also writes it to `<out.ttf>`. It has three glyphs. Glyph 0 is
//! empty. Glyph 1 has a triangle in `glyf` and no bitmap. Glyph 2 has a red PNG in `CBDT` and no
//! outline. `CBLC` covers glyphs 1 and 2 with one index subtable of format 1. Format 1 lists one
//! offset per glyph, and one more after the last, and a glyph's bitmap is the bytes from its offset
//! to the next one. Glyph 1's two offsets are equal, which is how the OpenType specification marks
//! a glyph in the range that has no bitmap. So the correct drawing of glyph 1 is its outline, a
//! black triangle, and of glyph 2 its bitmap, a red square.
//!
//! swash 0.2.10 (`src/strike.rs:601-625`) and ttf-parser 0.25.1 (`src/tables/cblc.rs:227-243`)
//! read only the glyph's own offset. For glyph 1 they find the record that starts there, which is
//! glyph 2's. skrifa 0.44.0 reads both offsets and finds no bitmap.
//!
//! `row` draws glyphs 1 and 2 with `fill_glyph_run` and prints what each library returns for them
//! and the colour drawn in the middle of glyph 1. The swash-only build draws only this, because it
//! has no `fill_text` for labels. `figure`, in the default build, draws the labelled picture: what
//! each library gives for the two glyphs (the bitmap it returns, or the outline when it returns
//! none), then what femtovg draws with its default features and, from `<swash row.png>`, with
//! `swash` alone.

mod offscreen;

#[cfg(feature = "figure")]
use femtovg::Path;
use femtovg::{Color, Paint, PositionedGlyph};
use skrifa::MetadataProvider;

/// Pixels per em of the font's only strike. Its PNG is this many pixels square.
const PPEM: u8 = 24;
/// The font size of the glyphs. It is below 92 px, so femtovg draws them from its atlas.
const FONT_SIZE: f32 = 64.0;
const UNITS_PER_EM: u16 = 1024;
/// Glyph 1's triangle in font units, on the baseline.
const TRIANGLE: [(i16, i16); 3] = [(100, 0), (900, 0), (500, 800)];
/// A row: the two glyphs, each in a column of its own, on one baseline.
const COLUMN_WIDTH: f32 = 110.0;
const ROW_WIDTH: u32 = 240;
const ROW_HEIGHT: u32 = 90;
const BASELINE: f32 = 76.0;
const GLYPH_X: [f32; 2] = [20.0, 20.0 + COLUMN_WIDTH];
/// The figure: a header, then one row per library and per build, each with its label on the left.
#[cfg(feature = "figure")]
const LABEL_WIDTH: u32 = 290;
#[cfg(feature = "figure")]
const HEADER_HEIGHT: u32 = 74;
#[cfg(feature = "figure")]
const FIGURE_WIDTH: u32 = LABEL_WIDTH + ROW_WIDTH;
#[cfg(feature = "figure")]
const FIGURE_HEIGHT: u32 = HEADER_HEIGHT + 5 * ROW_HEIGHT;

fn push_u8(data: &mut Vec<u8>, value: u8) {
    data.push(value);
}

fn push_u16(data: &mut Vec<u8>, value: u16) {
    data.extend_from_slice(&value.to_be_bytes());
}

fn push_i16(data: &mut Vec<u8>, value: i16) {
    data.extend_from_slice(&value.to_be_bytes());
}

fn push_u32(data: &mut Vec<u8>, value: u32) {
    data.extend_from_slice(&value.to_be_bytes());
}

/// Assembles a TrueType font from its tables: a header, one record per table, then the tables,
/// each padded to four bytes. Nothing here reads the checksums, so they are zero.
fn font_from_tables(mut tables: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    tables.sort_by_key(|(tag, _)| *tag);
    let count = tables.len() as u16;
    let entry_selector = count.ilog2() as u16;
    let search_range: u16 = 16 << entry_selector;
    let mut font = vec![0, 1, 0, 0];
    for field in [count, search_range, entry_selector, count * 16 - search_range] {
        push_u16(&mut font, field);
    }
    let mut body = Vec::new();
    for (tag, table) in &tables {
        font.extend_from_slice(tag);
        push_u32(&mut font, 0); // checksum
        push_u32(&mut font, (12 + 16 * tables.len() + body.len()) as u32);
        push_u32(&mut font, table.len() as u32);
        body.extend_from_slice(table);
        body.resize(body.len().next_multiple_of(4), 0);
    }
    font.extend_from_slice(&body);
    font
}

fn png() -> Vec<u8> {
    let mut png = std::io::Cursor::new(Vec::new());
    image::RgbaImage::from_pixel(PPEM.into(), PPEM.into(), image::Rgba([255, 0, 64, 255]))
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    png.into_inner()
}

fn font() -> Vec<u8> {
    let mut head = Vec::new();
    push_u16(&mut head, 1); // majorVersion
    push_u16(&mut head, 0); // minorVersion
    push_u32(&mut head, 0); // fontRevision
    push_u32(&mut head, 0); // checksumAdjustment
    push_u32(&mut head, 0x5F0F_3CF5); // magicNumber
    push_u16(&mut head, 0); // flags
    push_u16(&mut head, UNITS_PER_EM);
    for _ in 0..4 {
        push_u32(&mut head, 0); // created and modified
    }
    for value in [0, -200, 1024, 800] {
        push_i16(&mut head, value); // xMin, yMin, xMax, yMax
    }
    push_u16(&mut head, 0); // macStyle
    push_u16(&mut head, 8); // lowestRecPPEM
    push_i16(&mut head, 2); // fontDirectionHint
    push_i16(&mut head, 0); // indexToLocFormat: `loca` offsets are 16 bits, in units of two bytes
    push_i16(&mut head, 0); // glyphDataFormat

    let mut hhea = Vec::new();
    push_u16(&mut hhea, 1); // majorVersion
    push_u16(&mut hhea, 0); // minorVersion
    for value in [800, -200, 0] {
        push_i16(&mut hhea, value); // ascender, descender, lineGap
    }
    push_u16(&mut hhea, 1024); // advanceWidthMax
    for value in [0, 0, 1024, 1, 0, 0, 0, 0, 0, 0, 0] {
        // minLeftSideBearing, minRightSideBearing, xMaxExtent, caretSlopeRise, caretSlopeRun,
        // caretOffset, four reserved fields, metricDataFormat
        push_i16(&mut hhea, value);
    }
    push_u16(&mut hhea, 0); // numberOfHMetrics: `hmtx` has left side bearings only

    let mut maxp = Vec::new();
    push_u32(&mut maxp, 0x0000_5000); // version 0.5
    push_u16(&mut maxp, 3); // numGlyphs

    // swash reads `glyf` outlines only when the font has an `hmtx` table.
    let mut hmtx = Vec::new();
    for _ in 0..3 {
        push_i16(&mut hmtx, 0); // leftSideBearings
    }

    // Glyph 1 is one contour through the three points of the triangle, all on the curve. Each
    // flag byte is 1 (on the curve), so each coordinate is a 16-bit delta from the one before.
    let mut glyf = Vec::new();
    for value in [1, 100, 0, 900, 800] {
        push_i16(&mut glyf, value); // numberOfContours, xMin, yMin, xMax, yMax
    }
    push_u16(&mut glyf, 2); // endPtsOfContours[0]
    push_u16(&mut glyf, 0); // instructionLength
    glyf.extend_from_slice(&[1; 3]); // flags
    let mut previous = (0, 0);
    for (x, _) in TRIANGLE {
        push_i16(&mut glyf, x - previous.0);
        previous.0 = x;
    }
    for (_, y) in TRIANGLE {
        push_i16(&mut glyf, y - previous.1);
        previous.1 = y;
    }
    glyf.resize(glyf.len().next_multiple_of(2), 0);
    // Glyphs 0 and 2 have no outline, so their start and end are the same.
    let mut loca = Vec::new();
    for offset in [0, 0, glyf.len() / 2, glyf.len() / 2] {
        push_u16(&mut loca, offset as u16);
    }

    // CBDT: the version, then glyph 2's bitmap in image format 17, which is small metrics, the
    // length of the PNG and the PNG.
    let png = png();
    let mut glyph_2 = Vec::new();
    for value in [PPEM, PPEM, 0, PPEM, PPEM] {
        push_u8(&mut glyph_2, value); // height, width, bearingX, bearingY, advance
    }
    push_u32(&mut glyph_2, png.len() as u32);
    glyph_2.extend_from_slice(&png);
    let mut cbdt = Vec::new();
    push_u16(&mut cbdt, 3); // majorVersion
    push_u16(&mut cbdt, 0); // minorVersion
    cbdt.extend_from_slice(&glyph_2);

    // CBLC: the version and one strike, whose 48-byte record is followed by the strike's index
    // subtable list (one record) and its one index subtable.
    let mut cblc = Vec::new();
    push_u16(&mut cblc, 3); // majorVersion
    push_u16(&mut cblc, 0); // minorVersion
    push_u32(&mut cblc, 1); // numSizes
    let list_offset = 8 + 48;
    push_u32(&mut cblc, list_offset); // indexSubTableArrayOffset
    push_u32(&mut cblc, 8 + 8 + 4 * 3); // indexTablesSize: the list, a header and three offsets
    push_u32(&mut cblc, 1); // numberofIndexSubTables
    push_u32(&mut cblc, 0); // colorRef
    for line_metrics in [[PPEM as i8, 0, PPEM as i8, 1, 0, 0, 0, 0, PPEM as i8, 0, 0, 0], [0; 12]] {
        // hori and vert: ascender, descender, widthMax, caretSlopeNumerator,
        // caretSlopeDenominator, caretOffset, minOriginSB, minAdvanceSB, maxBeforeBL,
        // minAfterBL, pad1, pad2
        cblc.extend(line_metrics.map(|value| value as u8));
    }
    push_u16(&mut cblc, 1); // startGlyphIndex
    push_u16(&mut cblc, 2); // endGlyphIndex
    for value in [PPEM, PPEM, 32, 1] {
        push_u8(&mut cblc, value); // ppemX, ppemY, bitDepth, flags (horizontal metrics)
    }
    assert_eq!(cblc.len(), list_offset as usize);
    push_u16(&mut cblc, 1); // firstGlyphIndex
    push_u16(&mut cblc, 2); // lastGlyphIndex
    push_u32(&mut cblc, 8); // additionalOffsetToIndexSubtable: the subtable follows this record
    push_u16(&mut cblc, 1); // indexFormat
    push_u16(&mut cblc, 17); // imageFormat
    push_u32(&mut cblc, 4); // imageDataOffset: glyph 2's bitmap starts after CBDT's version
    // sbitOffsets: glyph 1 starts and ends at 0, so it has no bitmap, and glyph 2 takes the rest.
    for offset in [0, 0, glyph_2.len() as u32] {
        push_u32(&mut cblc, offset);
    }

    font_from_tables(vec![
        (*b"head", head),
        (*b"hhea", hhea),
        (*b"maxp", maxp),
        (*b"hmtx", hmtx),
        (*b"loca", loca),
        (*b"glyf", glyf),
        (*b"CBDT", cbdt),
        (*b"CBLC", cblc),
    ])
}

/// What a library returns for a glyph's color bitmap: the pixels, or why there are none.
struct Lookup {
    /// `figure` draws it.
    #[cfg_attr(not(feature = "figure"), allow(dead_code))]
    bitmap: Option<image::RgbaImage>,
    /// One line for the summary.
    text: String,
}

fn image_line(image: &image::RgbaImage) -> String {
    let (width, height) = image.dimensions();
    let middle = image.get_pixel(width / 2, height / 2).0;
    format!("a {width} x {height} image, rgba{middle:?} in the middle")
}

fn lookup(bitmap: Option<image::RgbaImage>, what: &str) -> Lookup {
    let text = bitmap.as_ref().map_or("none".to_string(), image_line);
    Lookup {
        bitmap,
        text: format!("{what}: {text}"),
    }
}

/// What swash 0.2.10 gives for `glyph` when asked for a color bitmap at the strike's own size.
fn swash_lookup(font: &[u8], glyph: u16) -> Lookup {
    let font = swash::FontRef::from_index(font, 0).unwrap();
    let mut context = swash::scale::ScaleContext::new();
    let mut scaler = context.builder(font).build();
    let bitmap = scaler
        .scale_color_bitmap(glyph, swash::scale::StrikeWith::LargestSize)
        .map(|image| image::RgbaImage::from_raw(image.placement.width, image.placement.height, image.data).unwrap());
    lookup(bitmap, "swash 0.2.10 scale_color_bitmap")
}

/// What ttf-parser 0.25.1 gives for `glyph`, decoded.
fn ttf_parser_lookup(font: &[u8], glyph: u16) -> Lookup {
    let face = ttf_parser::Face::parse(font, 0).unwrap();
    let bitmap = face
        .glyph_raster_image(ttf_parser::GlyphId(glyph), u16::MAX)
        .map(|found| image::load_from_memory(found.data).unwrap().to_rgba8());
    lookup(bitmap, "ttf-parser 0.25.1 glyph_raster_image")
}

/// What skrifa 0.44.0 gives for `glyph` at the strike's size, decoded.
fn skrifa_lookup(font: &[u8], glyph: u16) -> Lookup {
    let font = skrifa::FontRef::new(font).unwrap();
    let size = skrifa::instance::Size::new(PPEM.into());
    let bitmap = font
        .bitmap_strikes()
        .glyph_for_size(size, skrifa::GlyphId::new(glyph.into()))
        .map(|found| match found.data {
            skrifa::bitmap::BitmapData::Png(png) => image::load_from_memory(png).unwrap().to_rgba8(),
            _ => panic!("glyph {glyph}'s bitmap is not a PNG"),
        });
    lookup(bitmap, "skrifa 0.44.0 glyph_for_size")
}

/// Glyph 1's outline at FONT_SIZE, with its origin at (`x`, `baseline`). Font units point up and
/// canvas pixels point down.
#[cfg(feature = "figure")]
fn triangle(x: f32, baseline: f32) -> Path {
    let scale = FONT_SIZE / f32::from(UNITS_PER_EM);
    let point = |(px, py): (i16, i16)| (x + f32::from(px) * scale, baseline - f32::from(py) * scale);
    let mut path = Path::new();
    let (first_x, first_y) = point(TRIANGLE[0]);
    path.move_to(first_x, first_y);
    for corner in &TRIANGLE[1..] {
        let (corner_x, corner_y) = point(*corner);
        path.line_to(corner_x, corner_y);
    }
    path.close();
    path
}

/// The middle of glyph 1's triangle, in a row whose glyph 1 has its origin at (`x`, `baseline`).
fn middle_of_glyph_1(x: f32, baseline: f32) -> (u32, u32) {
    let scale = FONT_SIZE / f32::from(UNITS_PER_EM);
    let (sum_x, sum_y) = TRIANGLE.iter().fold((0.0, 0.0), |(sum_x, sum_y), (px, py)| {
        (sum_x + f32::from(*px), sum_y + f32::from(*py))
    });
    ((x + sum_x / 3.0 * scale) as u32, (baseline - sum_y / 3.0 * scale) as u32)
}

/// Draws glyphs 1 and 2 with `fill_glyph_run`, in a row whose top left is (`left`, `top`).
fn draw_glyphs<T: femtovg::Renderer>(canvas: &mut femtovg::Canvas<T>, font: femtovg::FontId, left: f32, top: f32) {
    let paint = Paint::color(Color::black()).with_font_size(FONT_SIZE);
    let glyphs = [1, 2].map(|glyph_id| PositionedGlyph {
        x: left + GLYPH_X[usize::from(glyph_id) - 1],
        y: top + BASELINE,
        glyph_id,
    });
    canvas.fill_glyph_run(font, &[], glyphs, &paint).expect("fill_glyph_run failed");
}

fn row(out_png: &str, out_ttf: &str) {
    let font = font();
    std::fs::write(out_ttf, &font).unwrap_or_else(|error| panic!("cannot write {out_ttf}: {error}"));
    for glyph in [1, 2] {
        for found in [swash_lookup(&font, glyph), ttf_parser_lookup(&font, glyph), skrifa_lookup(&font, glyph)] {
            println!("glyph {glyph}, {}", found.text);
        }
    }

    let gpu = offscreen::Gpu::new();
    eprintln!("drawing on {}", gpu.adapter_name);
    let text_context = femtovg::TextContext::default();
    let font_id = text_context.add_font_mem(&font).expect("femtovg cannot read the font");
    let mut canvas = gpu.canvas(text_context, ROW_WIDTH, ROW_HEIGHT);
    canvas.clear_rect(0, 0, ROW_WIDTH, ROW_HEIGHT, Color::white());
    draw_glyphs(&mut canvas, font_id, 0.0, 0.0);
    let frame = gpu.read_frame(&mut canvas, ROW_WIDTH, ROW_HEIGHT);
    let (x, y) = middle_of_glyph_1(GLYPH_X[0], BASELINE);
    println!("glyph 1, fill_glyph_run: rgba{:?} in the middle of the triangle", frame.get_pixel(x, y).0);
    frame
        .save(out_png)
        .unwrap_or_else(|error| panic!("cannot write {out_png}: {error}"));
}

#[cfg(feature = "figure")]
fn figure(label_font: &str, swash_row: &str, out_png: &str) {
    use femtovg::{ImageFlags, ImageSource};

    let font = font();
    let gpu = offscreen::Gpu::new();
    eprintln!("drawing on {}", gpu.adapter_name);
    let text_context = femtovg::TextContext::default();
    let font_id = text_context.add_font_mem(&font).expect("femtovg cannot read the font");
    let label_font = text_context
        .add_font_file(label_font)
        .unwrap_or_else(|error| panic!("cannot read {label_font}: {error:?}"));
    let mut canvas = gpu.canvas(text_context, FIGURE_WIDTH, FIGURE_HEIGHT);
    canvas.clear_rect(0, 0, FIGURE_WIDTH, FIGURE_HEIGHT, Color::white());

    let text = |canvas: &mut femtovg::Canvas<_>, x: f32, y: f32, line: &str, size: f32, color: Color| {
        let paint = Paint::color(color).with_font(&[label_font]).with_font_size(size);
        canvas.fill_text(x, y, line, &paint).expect("fill_text failed");
    };
    let black = Color::rgb(20, 20, 20);
    let grey = Color::rgb(110, 110, 110);
    let left = LABEL_WIDTH as f32;
    text(&mut canvas, 16.0, 26.0, "Correct: glyph 1 is a black triangle,", 15.0, black);
    text(&mut canvas, 16.0, 46.0, "glyph 2 is a red square.", 15.0, black);
    for (column, (title, detail)) in [("glyph 1", "outline, no bitmap"), ("glyph 2", "bitmap, no outline")]
        .into_iter()
        .enumerate()
    {
        let x = left + GLYPH_X[column];
        text(&mut canvas, x, 26.0, title, 15.0, black);
        text(&mut canvas, x, 46.0, detail, 12.0, grey);
    }

    // A library's row draws the bitmap it returns for a glyph, scaled to FONT_SIZE as femtovg
    // places a bitmap glyph, or the glyph's outline when it returns none.
    let libraries: [(&str, &str, fn(&[u8], u16) -> Lookup); 3] = [
        ("skrifa 0.44.0", "no bitmap for glyph 1: correct", skrifa_lookup),
        ("swash 0.2.10", "glyph 2's bitmap for glyph 1: wrong", swash_lookup),
        ("ttf-parser 0.25.1", "glyph 2's PNG for glyph 1: wrong", ttf_parser_lookup),
    ];
    let mut summary = Vec::new();
    for (index, (name, verdict, look_up)) in libraries.into_iter().enumerate() {
        let top = (HEADER_HEIGHT + index as u32 * ROW_HEIGHT) as f32;
        text(&mut canvas, 16.0, top + 40.0, name, 15.0, black);
        text(&mut canvas, 16.0, top + 60.0, verdict, 12.0, grey);
        for glyph in [1, 2] {
            let x = left + GLYPH_X[usize::from(glyph) - 1];
            match look_up(&font, glyph).bitmap {
                Some(bitmap) => {
                    let pixels = image::DynamicImage::ImageRgba8(bitmap);
                    let image = canvas
                        .create_image(ImageSource::try_from(&pixels).unwrap(), ImageFlags::empty())
                        .unwrap();
                    let (bitmap_top, size) = (top + BASELINE - FONT_SIZE, FONT_SIZE);
                    let mut square = Path::new();
                    square.rect(x, bitmap_top, size, size);
                    canvas.fill_path(&square, &Paint::image(image, x, bitmap_top, size, size, 0.0, 1.0));
                }
                None => canvas.fill_path(&triangle(x, top + BASELINE), &Paint::color(Color::black())),
            }
        }
        summary.push((name.to_string(), top));
    }

    // femtovg with its default features draws here. The swash-only build drew its row in its own
    // run, and the row is copied in pixel for pixel.
    let default_top = (HEADER_HEIGHT + 3 * ROW_HEIGHT) as f32;
    text(&mut canvas, 16.0, default_top + 40.0, "femtovg master, default features", 15.0, black);
    text(&mut canvas, 16.0, default_top + 60.0, "uses ttf-parser: wrong", 12.0, grey);
    draw_glyphs(&mut canvas, font_id, left, default_top);
    summary.push(("femtovg, default features".to_string(), default_top));

    let swash_top = (HEADER_HEIGHT + 4 * ROW_HEIGHT) as f32;
    text(&mut canvas, 16.0, swash_top + 40.0, "femtovg master, swash only", 15.0, black);
    text(&mut canvas, 16.0, swash_top + 60.0, "uses swash: wrong", 12.0, grey);
    let row = image::open(swash_row).unwrap_or_else(|error| panic!("cannot read {swash_row}: {error}"));
    let row_image = canvas
        .create_image(ImageSource::try_from(&row).unwrap(), ImageFlags::NEAREST)
        .unwrap();
    let mut rect = Path::new();
    rect.rect(left, swash_top, ROW_WIDTH as f32, ROW_HEIGHT as f32);
    canvas.fill_path(
        &rect,
        &Paint::image(row_image, left, swash_top, ROW_WIDTH as f32, ROW_HEIGHT as f32, 0.0, 1.0),
    );
    summary.push(("femtovg, swash only".to_string(), swash_top));

    // A thin rule above every row.
    for index in 0..5 {
        let mut rule = Path::new();
        rule.rect(0.0, (HEADER_HEIGHT + index * ROW_HEIGHT) as f32, FIGURE_WIDTH as f32, 1.0);
        canvas.fill_path(&rule, &Paint::color(Color::rgb(210, 210, 210)));
    }

    let frame = gpu.read_frame(&mut canvas, FIGURE_WIDTH, FIGURE_HEIGHT);
    for (name, top) in summary {
        let (x, y) = middle_of_glyph_1(left + GLYPH_X[0], top + BASELINE);
        println!("{name}, glyph 1: rgba{:?} in the middle of the triangle", frame.get_pixel(x, y).0);
    }
    frame
        .save(out_png)
        .unwrap_or_else(|error| panic!("cannot write {out_png}: {error}"));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [_, "row", out_png, out_ttf] => row(out_png, out_ttf),
        #[cfg(feature = "figure")]
        [_, "figure", label_font, swash_row, out_png] => figure(label_font, swash_row, out_png),
        _ => {
            eprintln!("usage: {} row <out.png> <out.ttf>", args[0]);
            eprintln!("       {} figure <font.ttf> <swash row.png> <out.png>  (default build only)", args[0]);
            std::process::exit(2);
        }
    }
}
