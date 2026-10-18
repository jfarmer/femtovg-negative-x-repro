//! A glyph that a color bitmap font has no bitmap for, drawn with femtovg on
//! `renderer::WGPURenderer` and saved as a PNG.
//!
//! usage: cbdt-<build> <out.png> <out.ttf>
//!
//! The font is built here and also written to `<out.ttf>`. It has three glyphs. Glyph 0 is empty.
//! Glyph 1 has a triangle in `glyf` and no bitmap. Glyph 2 has a red PNG in `CBDT` and no outline.
//! `CBLC` covers glyphs 1 and 2 with one index subtable of format 1. Format 1 lists one offset per
//! glyph, and one more after the last, and a glyph's bitmap is the bytes from its offset to the
//! next one. Glyph 1's two offsets are equal, which is how the OpenType specification marks a
//! glyph in the range that has no bitmap.
//!
//! swash 0.2.10 (`src/strike.rs:601-625`) and ttf-parser 0.25.1 (`src/tables/cblc.rs:227-243`)
//! read only the glyph's own offset. For glyph 1 they find the record that starts there, which is
//! glyph 2's, so femtovg draws a red square where the triangle belongs. skrifa 0.44.0 reads both
//! offsets and finds no bitmap.
//!
//! The scene draws glyphs 1 and 2 with `fill_glyph_run` and then glyph 1's outline with
//! `fill_path`, to the right. The first and the third should look the same. The program prints
//! what each library returns for glyphs 1 and 2, and the colour femtovg drew in the middle of the
//! first and the third.

mod offscreen;

use femtovg::{Color, Paint, Path, PositionedGlyph};
use skrifa::MetadataProvider;

/// Pixels per em of the font's only strike. Its PNG is this many pixels square.
const PPEM: u8 = 24;
/// The font size of the scene. It is below 92 px, so femtovg draws the glyphs from its atlas.
const FONT_SIZE: f32 = 64.0;
const UNITS_PER_EM: u16 = 1024;
/// Glyph 1's triangle in font units, on the baseline.
const TRIANGLE: [(i16, i16); 3] = [(100, 0), (900, 0), (500, 800)];
/// The canvas, the baseline and the x of each of the three drawings.
const WIDTH: u32 = 360;
const HEIGHT: u32 = 120;
const BASELINE: f32 = 90.0;
const GLYPH_1_X: f32 = 30.0;
const GLYPH_2_X: f32 = 140.0;
const PATH_X: f32 = 250.0;

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

/// What swash 0.2.10 gives for `glyph` when asked for a color bitmap at the strike's own size.
fn swash_lookup(font: &[u8], glyph: u16) -> String {
    let font = swash::FontRef::from_index(font, 0).unwrap();
    let mut context = swash::scale::ScaleContext::new();
    let mut scaler = context.builder(font).build();
    match scaler.scale_color_bitmap(glyph, swash::scale::StrikeWith::LargestSize) {
        Some(image) => {
            let (width, height) = (image.placement.width as usize, image.placement.height as usize);
            let centre = 4 * (height / 2 * width + width / 2);
            let rgba = &image.data[centre..centre + 4];
            format!("a {width} x {height} image, rgba{rgba:?} in the middle")
        }
        None => "none".to_string(),
    }
}

/// What ttf-parser 0.25.1 gives for `glyph`, and whether its data is glyph 2's PNG.
fn ttf_parser_lookup(font: &[u8], glyph: u16) -> String {
    let face = ttf_parser::Face::parse(font, 0).unwrap();
    let image = |glyph| face.glyph_raster_image(ttf_parser::GlyphId(glyph), u16::MAX);
    match image(glyph) {
        Some(found) => {
            let same = image(2).is_some_and(|glyph_2| glyph_2.data == found.data);
            let whose = if same { "glyph 2's PNG" } else { "another PNG" };
            format!("a {} x {} {:?} image, the bytes of {whose}", found.width, found.height, found.format)
        }
        None => "none".to_string(),
    }
}

/// What skrifa 0.44.0 gives for `glyph` at the strike's size.
fn skrifa_lookup(font: &[u8], glyph: u32) -> String {
    let font = skrifa::FontRef::new(font).unwrap();
    let size = skrifa::instance::Size::new(PPEM.into());
    match font.bitmap_strikes().glyph_for_size(size, skrifa::GlyphId::new(glyph)) {
        Some(found) => format!("a {} x {} image", found.width, found.height),
        None => "none".to_string(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, out_png, out_ttf] = args.as_slice() else {
        eprintln!("usage: {} <out.png> <out.ttf>", args[0]);
        std::process::exit(2);
    };
    let font = font();
    std::fs::write(out_ttf, &font).unwrap_or_else(|error| panic!("cannot write {out_ttf}: {error}"));

    for glyph in [1, 2] {
        println!("glyph {glyph}, swash 0.2.10 scale_color_bitmap: {}", swash_lookup(&font, glyph));
        println!("glyph {glyph}, ttf-parser 0.25.1 glyph_raster_image: {}", ttf_parser_lookup(&font, glyph));
        println!("glyph {glyph}, skrifa 0.44.0 glyph_for_size: {}", skrifa_lookup(&font, glyph.into()));
    }

    let gpu = offscreen::Gpu::new();
    eprintln!("{}: drawing on {}", args[0], gpu.adapter_name);
    let text_context = femtovg::TextContext::default();
    let font_id = text_context.add_font_mem(&font).expect("femtovg cannot read the font");
    let mut canvas = gpu.canvas(text_context, WIDTH, HEIGHT);
    canvas.clear_rect(0, 0, WIDTH, HEIGHT, Color::white());

    let paint = Paint::color(Color::black()).with_font_size(FONT_SIZE);
    let glyphs = [(1, GLYPH_1_X), (2, GLYPH_2_X)].map(|(glyph_id, x)| PositionedGlyph {
        x,
        y: BASELINE,
        glyph_id,
    });
    canvas.fill_glyph_run(font_id, &[], glyphs, &paint).expect("fill_glyph_run failed");

    // Glyph 1's outline at FONT_SIZE. Font units point up and canvas pixels point down.
    let scale = FONT_SIZE / f32::from(UNITS_PER_EM);
    let point = |(x, y): (i16, i16)| (PATH_X + f32::from(x) * scale, BASELINE - f32::from(y) * scale);
    let mut path = Path::new();
    let (x, y) = point(TRIANGLE[0]);
    path.move_to(x, y);
    for corner in &TRIANGLE[1..] {
        let (x, y) = point(*corner);
        path.line_to(x, y);
    }
    path.close();
    canvas.fill_path(&path, &paint);

    let frame = gpu.read_frame(&mut canvas, WIDTH, HEIGHT);
    // The middle of the triangle, where its three corners average out.
    let middle = |left: f32| {
        let (x, y) = TRIANGLE.iter().fold((0.0, 0.0), |(x, y), corner| {
            let (px, py) = point(*corner);
            (x + px - PATH_X + left, y + py)
        });
        (x / 3.0, y / 3.0)
    };
    for (what, left) in [("glyph 1, fill_glyph_run", GLYPH_1_X), ("glyph 1's outline, fill_path", PATH_X)] {
        let (x, y) = middle(left);
        let pixel = frame.get_pixel(x as u32, y as u32).0;
        println!("{what}: rgba{pixel:?} at ({}, {})", x as u32, y as u32);
    }
    frame
        .save(out_png)
        .unwrap_or_else(|error| panic!("cannot write {out_png}: {error}"));
}
