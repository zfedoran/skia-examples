use skia_safe::{
    Color, Data, EncodedImageFormat, Font, Paint, Surface, FontMgr
};
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    // Load the font data from the file.
    let font_path = "Roboto-LightItalic.ttf";
    let font_file = fs::read(font_path)?;
    let data = Data::new_copy(&font_file);

    // Use the system font manager to load the custom typeface.
    // This is similar to the docs example, but here we only need one font.
    let font_mgr = FontMgr::new();
    let typeface = font_mgr
        .new_from_data(&data, None)
        .ok_or("Failed to load the font from file")?;

    // Create and configure a simple Font.
    let font_size = 32.0;
    let mut font_obj = Font::default();
    font_obj.set_size(font_size);
    font_obj.set_typeface(typeface);
    // Enable sub-pixel anti-aliasing.
    font_obj.set_edging(skia_safe::font::Edging::SubpixelAntiAlias);

    // Set up a Paint for drawing the text.
    let mut paint = Paint::default();
    paint.set_color(Color::BLACK);
    paint.set_anti_alias(true);

    // Create a raster surface to draw on (e.g., 300x100 pixels).
    let width = 300;
    let height = 100;
    let mut surface = Surface::new_raster_n32_premul((width, height))
        .ok_or("Could not create a surface")?;
    let canvas = surface.canvas();

    // Clear the canvas with a white background.
    canvas.clear(Color::WHITE);

    // Draw the text "hello, world" at coordinates (50, 50).
    canvas.draw_str("hello, world", (50, 50), &font_obj, &paint);

    // Snapshot the surface as an image and encode it as PNG.
    let image = surface.image_snapshot();
    let png_data = image
        .encode_to_data(EncodedImageFormat::PNG)
        .ok_or("Failed to encode image")?;

    // Write the PNG data to a file.
    fs::write("output.png", png_data.as_bytes())?;

    println!("Image written to output.png");

    Ok(())
}
