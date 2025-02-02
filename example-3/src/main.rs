use harfbuzz_rs::{Face, Font as HbFont, UnicodeBuffer, shape, Direction, Language, Tag};
use skia_safe::{
    Color, Data, EncodedImageFormat, Font, FontMgr, Paint, Point, Surface, TextBlobBuilder,
};
use std::error::Error;
use std::fs;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Load the font file into memory.
    let font_path = "Rubik-VariableFont_wght.ttf";
    let font_data = fs::read(font_path)?;
    let skia_data = Data::new_copy(&font_data);

    // 2. Set up a Skia Font (20px).
    let font_mgr = FontMgr::new();
    let typeface = font_mgr
        .new_from_data(&skia_data, None)
        .ok_or("Failed to load typeface")?;
    let font_size = 20.0;
    let mut skia_font = Font::default();
    skia_font.set_size(font_size);
    skia_font.set_typeface(typeface);
    skia_font.set_edging(skia_safe::font::Edging::SubpixelAntiAlias);

    // 3. Create a HarfBuzz Font at the same size (20px).
    let hb_face = Face::from_bytes(&font_data, 0);
    let mut hb_font = HbFont::new(hb_face);

    // HarfBuzz uses 26.6 fixed-point units, so multiply by 64.
    let hb_scale = (font_size * 64.0) as i32;
    hb_font.set_scale(hb_scale, hb_scale);

    // (Optional) You can also set "pixels per EM" (PPEM) for hinting:
    // hb_font.set_ppem(font_size as u32, font_size as u32);

    // 4. Prepare the Arabic text (logical order, no manual Bidi reorder).
    let text = "يحتوي على شريط التمرير على الجانب الأيمن";

    // 5. Shape the text with HarfBuzz in RTL mode.
    let hb_buffer = UnicodeBuffer::new()
        .add_str(text)
        .set_direction(Direction::Rtl)
        .set_language(Language::from_str("ar").unwrap())
        .set_script(Tag::new('a', 'r', 'a', 'b'));

    let shaped_result = shape(&hb_font, hb_buffer, &[]);
    let glyph_infos = shaped_result.get_glyph_infos();
    let glyph_positions = shaped_result.get_glyph_positions();

    // 6. Build a Skia TextBlob from HarfBuzz glyphs & positions.
    let count = glyph_infos.len();
    let mut builder = TextBlobBuilder::new();
    // alloc_run_pos: pass None for the optional bounding box.
    let (glyphs, positions) = builder.alloc_run_pos(&skia_font, count, None);

    // We track our running "x" in pixel coordinates, matching HarfBuzz scale.
    let mut x_accum = 0.0;
    for i in 0..count {
        glyphs[i] = glyph_infos[i].codepoint as u16;

        // HarfBuzz returns x_offset, x_advance, etc. in 26.6 fixed point => divide by 64.0.
        let x_offset = glyph_positions[i].x_offset as f32 / 64.0;
        let y_offset = glyph_positions[i].y_offset as f32 / 64.0;
        let x_advance = glyph_positions[i].x_advance as f32 / 64.0;

        positions[i] = Point::new(x_accum + x_offset, y_offset);
        x_accum += x_advance;
    }

    let text_blob = builder.make().ok_or("Failed to build text blob")?;

    // 7. Draw the TextBlob onto a Skia surface.
    let width = 500;
    let height = 100;
    let mut surface = Surface::new_raster_n32_premul((width, height))
        .ok_or("Could not create surface")?;
    let canvas = surface.canvas();
    canvas.clear(Color::WHITE);

    // Example: place the text blob near x=400, y=50.
    // (If you want right alignment to a specific edge, subtract x_accum, etc.)
    let origin_x = 50.0;
    let origin_y = 50.0;
    canvas.draw_text_blob(&text_blob, (origin_x, origin_y), &Paint::default());

    // 8. Save the result as a PNG.
    let image = surface.image_snapshot();
    let png_data = image
        .encode_to_data(EncodedImageFormat::PNG)
        .ok_or("Failed to encode image")?;
    fs::write("output_rtl.png", png_data.as_bytes())?;

    println!("Image saved as output_rtl.png");
    Ok(())
}
