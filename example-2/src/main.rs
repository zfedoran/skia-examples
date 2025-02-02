use skia_safe::{
    Color, Data, EncodedImageFormat, Font, FontMgr, GlyphId, Paint, Surface
};
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    // ---------------------------
    // 1. Load the fonts
    // ---------------------------

    // Primary font (e.g., Roboto for Latin text).
    let primary_font_path = "Roboto-LightItalic.ttf";
    let primary_font_data = Data::new_copy(&fs::read(primary_font_path)?);
    let font_mgr = FontMgr::new();
    let primary_typeface = font_mgr
        .new_from_data(&primary_font_data, None)
        .ok_or("Failed to load the primary font")?;
    let mut primary_font = Font::default();
    let font_size = 20.0;
    primary_font.set_size(font_size);
    primary_font.set_typeface(primary_typeface);
    primary_font.set_edging(skia_safe::font::Edging::SubpixelAntiAlias);

    // Fallback font (e.g., an emoji-capable font such as NotoColorEmoji).
    let fallback_font_path = "NotoColorEmoji-Regular.ttf";
    let fallback_font_data = Data::new_copy(&fs::read(fallback_font_path)?);
    let fallback_typeface = font_mgr
        .new_from_data(&fallback_font_data, None)
        .ok_or("Failed to load the fallback font")?;
    let mut fallback_font = Font::default();
    fallback_font.set_size(font_size);
    fallback_font.set_typeface(fallback_typeface);
    fallback_font.set_edging(skia_safe::font::Edging::SubpixelAntiAlias);

    // Set up a Paint for drawing.
    let mut paint = Paint::default();
    paint.set_color(Color::BLACK);
    paint.set_anti_alias(true);

    // ---------------------------
    // 2. Set up drawing surface
    // ---------------------------

    let width = 500;
    let height = 100;
    let mut surface = Surface::new_raster_n32_premul((width, height))
        .ok_or("Could not create a surface")?;
    let canvas = surface.canvas();
    canvas.clear(Color::WHITE);

    // ---------------------------
    // 3. Prepare the text with fallback
    // ---------------------------

    // Our mixed text string.
    let text = "hello, world 🌎";

    // We will split the text into runs: each run is a String along with a flag
    // that indicates whether the primary font can render those characters.
    let mut runs: Vec<(String, bool)> = Vec::new();
    let mut current_run = String::new();
    // For the first character, decide which font to use.
    let mut use_primary = false;

    // Process each character.
    for c in text.chars() {
        let can_primary_render = has_glyph(&primary_font, c);
        if current_run.is_empty() {
            // Start a new run.
            use_primary = can_primary_render;
            current_run.push(c);
        } else if can_primary_render == use_primary {
            // Same font works for this character; add to the current run.
            current_run.push(c);
        } else {
            // The required font has switched. Push the current run and start a new one.
            runs.push((current_run.clone(), use_primary));
            current_run.clear();
            current_run.push(c);
            use_primary = can_primary_render;
        }
    }
    if !current_run.is_empty() {
        runs.push((current_run, use_primary));
    }

    // ---------------------------
    // 4. Draw the text runs
    // ---------------------------

    // Starting coordinates.
    let mut x = 50.0;
    let y = 50.0;

    // For each run, select the appropriate font and draw the run,
    // then update x for the next run based on measured width.
    for (run, use_primary_font) in runs {
        let font = if use_primary_font {
            &primary_font
        } else {
            &fallback_font
        };

        // Draw the text run.
        canvas.draw_str(&run, (x, y), font, &paint);

        // Measure the width of the run to update the x coordinate.
        let (run_width, _) = font.measure_str(&run, Some(&paint));
        x += run_width;
    }

    // ---------------------------
    // 5. Save the result
    // ---------------------------

    let image = surface.image_snapshot();
    let png_data = image
        .encode_to_data(EncodedImageFormat::PNG)
        .ok_or("Failed to encode image")?;
    fs::write("output_fallback.png", png_data.as_bytes())?;
    println!("Image written to output_fallback.png");

    Ok(())
}

// Helper function to check if a font has a glyph for a given character.
fn has_glyph(font: &Font, c: char) -> bool {
    let s = c.to_string();
    let num_chars = s.chars().count();
    let mut glyphs = vec![0 as GlyphId; num_chars];
    let count = font.text_to_glyphs(&s, glyphs.as_mut_slice());
    count > 0 && glyphs[0] != 0
}
