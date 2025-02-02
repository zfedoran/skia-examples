use harfbuzz_rs::{Face, Font as HbFont, UnicodeBuffer, shape, Direction, Language, Tag};
use skia_safe::{
    Color, Data, EncodedImageFormat, Font, FontMgr, Paint, Point, Surface, TextBlobBuilder,
};
use std::error::Error;
use std::fs;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Load the font.
    let font_path = "Rubik-VariableFont_wght.ttf";
    let font_file = fs::read(font_path)?;
    let data = Data::new_copy(&font_file);
    let font_mgr = FontMgr::new();
    let typeface = font_mgr
        .new_from_data(&data, None)
        .ok_or("Failed to load the primary font")?;

    // 2. Create a Skia Font.
    let font_size = 20.0;
    let mut primary_font = Font::default();
    primary_font.set_size(font_size);
    primary_font.set_typeface(typeface);
    primary_font.set_edging(skia_safe::font::Edging::SubpixelAntiAlias);

    // 3. The Arabic text in logical order (no manual reorder).
    let text = "يحتوي على شريط التمرير على الجانب الأيمن";

    // 4. Shape the text with HarfBuzz in RTL direction.
    let hb_face = Face::from_bytes(&font_file, 0);
    let hb_font = HbFont::new(hb_face);

    let hb_buffer = UnicodeBuffer::new()
        .add_str(text)
        .set_direction(Direction::Rtl) 
        .set_language(Language::from_str("ar").unwrap())
        .set_script(Tag::new('a', 'r', 'a', 'b'));

    let shaped = shape(&hb_font, hb_buffer, &[]);
    let glyph_infos = shaped.get_glyph_infos();
    let glyph_positions = shaped.get_glyph_positions();

    // 5. Build a TextBlob.
    let count = glyph_infos.len();
    let mut blob_builder = TextBlobBuilder::new();
    let (glyphs, points) = blob_builder.alloc_run_pos(&primary_font, count, None);

    // HarfBuzz positions are in 26.6 fixed point, so divide by 64.
    let mut x = 0.0;
    for i in 0..count {
        glyphs[i] = glyph_infos[i].codepoint as u16;
        let x_offset = glyph_positions[i].x_offset as f32 / 64.0;
        let y_offset = glyph_positions[i].y_offset as f32 / 64.0;
        let x_advance = glyph_positions[i].x_advance as f32 / 64.0;
        points[i] = Point::new(x + x_offset, y_offset);
        x += x_advance;
    }
    let text_blob = blob_builder.make().ok_or("Failed to build TextBlob")?;

    // 6. Draw onto a surface in a straightforward way (no flipping).
    let width = 500;
    let height = 100;
    let mut surface = Surface::new_raster_n32_premul((width, height))
        .ok_or("Could not create a surface")?;
    let canvas = surface.canvas();
    canvas.clear(Color::WHITE);

    // We'll just draw the blob at some position. 
    // The shaped glyphs themselves are already in right-to-left order.
    let origin_x = 50.0;
    let origin_y = 50.0;
    // For an RTL run, "starting point" is typically the right end, so:
    canvas.draw_text_blob(&text_blob, (origin_x, origin_y), &Paint::default());

    // 7. Save.
    let image = surface.image_snapshot();
    let png_data = image.encode_to_data(EncodedImageFormat::PNG).unwrap();
    fs::write("output_rtl.png", png_data.as_bytes())?;

    println!("Image written to output_rtl.png");
    Ok(())
}
