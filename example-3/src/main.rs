use harfbuzz_rs::{Face, Font as HbFont, UnicodeBuffer, shape, Direction, Language, Tag};
use skia_safe::{
    Color, Data, EncodedImageFormat, Font, FontMgr, Paint, Point, Surface, TextBlobBuilder,
};
use std::error::Error;
use std::fs;
use unicode_bidi::BidiInfo;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    // --- 1. Load the font and create a Skia font ---
    let font_path = "Rubik-VariableFont_wght.ttf";
    let font_file = fs::read(font_path)?; // font_file is a Vec<u8>
    let data = Data::new_copy(&font_file);
    let font_mgr = FontMgr::new();
    let typeface = font_mgr
        .new_from_data(&data, None)
        .ok_or("Failed to load the primary font")?;
    let font_size = 20.0;
    let mut primary_font = Font::default();
    primary_font.set_size(font_size);
    primary_font.set_typeface(typeface);
    primary_font.set_edging(skia_safe::font::Edging::SubpixelAntiAlias);

    // --- 2. Prepare the RTL text with BiDi reordering ---
    let text = "يحتوي على شريط التمرير على الجانب الأيمن"; // Arabic text
    let bidi_info = BidiInfo::new(text, None);
    let para = &bidi_info.paragraphs[0];
    let display_text = bidi_info.reorder_line(para, para.range.clone());

    // --- 3. Shape the text with HarfBuzz ---
    let hb_face = Face::from_bytes(&font_file, 0);
    let hb_font = HbFont::new(hb_face);
    let hb_buffer = UnicodeBuffer::new()
        .add_str(&display_text)
        .set_direction(Direction::Rtl)
        .set_language(Language::from_str("ar").unwrap())
        .set_script(Tag::new('a', 'r', 'a', 'b')); // produces "arab"
    let shaped = shape(&hb_font, hb_buffer, &[]);
    let glyph_infos = shaped.get_glyph_infos();
    let glyph_positions = shaped.get_glyph_positions();

    // --- 4. Build a Skia TextBlob using the natural (LTR) positions ---
    let count = glyph_infos.len();
    let mut blob_builder = TextBlobBuilder::new();
    // alloc_run_pos: Pass None for bounds.
    let (glyphs, points) = blob_builder.alloc_run_pos(&primary_font, count, None);

    // Accumulate advances to get positions.
    // We compute positions as if the text were LTR.
    let mut x = 0.0;
    for i in 0..count {
        glyphs[i] = glyph_infos[i].codepoint as u16;
        // HarfBuzz returns values in 26.6 fixed‑point. Divide by 64 to convert to pixels.
        let x_offset = glyph_positions[i].x_offset as f32 / 64.0;
        let y_offset = glyph_positions[i].y_offset as f32 / 64.0;
        let x_advance = glyph_positions[i].x_advance as f32 / 64.0;
        // Here we ignore y_advance (usually zero for horizontal text)
        points[i] = Point::new(x + x_offset, y_offset);
        x += x_advance;
    }
    let total_width = x;
    let text_blob = blob_builder.make().ok_or("Failed to build TextBlob")?;

    // --- 5. Draw the TextBlob using a canvas transform ---
    let width = 500;
    let height = 100;
    let mut surface = Surface::new_raster_n32_premul((width, height))
        .ok_or("Could not create a surface")?;
    let canvas = surface.canvas();
    canvas.clear(Color::WHITE);

    // Choose the origin where you want the right edge of the text to appear.
    let origin_x = 50.0;
    let origin_y = 50.0;
    
    // Save canvas state.
    canvas.save();
    // Translate so that the right edge is at origin_x.
    // Since our text blob's coordinates start at 0 and extend to total_width,
    // translating by (origin_x + total_width) shifts the blob so that its right edge
    // is at (origin_x + total_width). Then scaling by -1 in x flips it.
    canvas.translate((origin_x + total_width, origin_y));
    // Mirror horizontally.
    canvas.scale((-1.0, 1.0));
    // Draw the blob at the transformed origin.
    canvas.draw_text_blob(&text_blob, (0.0, 0.0), &Paint::default());
    // Restore canvas state.
    canvas.restore();

    // --- 6. Save the result ---
    let image = surface.image_snapshot();
    let png_data = image
        .encode_to_data(EncodedImageFormat::PNG)
        .ok_or("Failed to encode image")?;

    fs::write("output_rtl.png", png_data.as_bytes())?;
    println!("Image written to output_rtl.png");

    Ok(())
}
