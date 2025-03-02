use freetype as ft;
use harfbuzz_rs::{Face, Font as HbFont, UnicodeBuffer, shape, Direction, Language, Tag};
use skia_safe::{Color, EncodedImageFormat, Paint, Path, Surface};
use std::error::Error;
use std::fs;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let font_path = "Rubik-VariableFont_wght.ttf";
    let font_data = fs::read(font_path)?;
    
    let library = ft::Library::init()?;
    let ft_face = library.new_face(font_path, 0)?;
    
    // Set the desired font size (in pixels).
    let desired_font_size = 40.0;
    ft_face.set_pixel_sizes(0, desired_font_size as u32)?;
    
    let hb_face = Face::from_bytes(&font_data, 0);
    let mut hb_font = HbFont::new(hb_face);
    
    // HarfBuzz uses 26.6 fixed‑point values, so multiply the size by 64.
    let hb_scale = (desired_font_size * 64.0) as i32;
    hb_font.set_scale(hb_scale, hb_scale);
    
    let text = "مرحبا بالعالم";
    let hb_buffer = UnicodeBuffer::new()
        .add_str(text)
        .set_direction(Direction::Rtl)
        .set_language(Language::from_str("ar").unwrap())
        .set_script(Tag::new('a', 'r', 'a', 'b'));
    
    let shaped_result = shape(&hb_font, hb_buffer, &[]);
    let glyph_infos = shaped_result.get_glyph_infos();
    let glyph_positions = shaped_result.get_glyph_positions();
    
    let width = 500;
    let height = 200;
    let mut surface = Surface::new_raster_n32_premul((width, height))
        .ok_or("Could not create surface")?;
    let canvas = surface.canvas();
    canvas.clear(Color::WHITE);
    
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    
    let origin_x = 50.0;
    let origin_y = 100.0;
    
    // Running horizontal offset (in pixels) for glyph placement.
    let mut x_accum = 0.0;
    
    // Process each glyph from the HarfBuzz shaping result.
    for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
        let glyph_id = info.codepoint;
        // HarfBuzz positions are in 26.6 fixed point.
        let x_offset = pos.x_offset as f32 / 64.0;
        let y_offset = pos.y_offset as f32 / 64.0;
        let x_advance = pos.x_advance as f32 / 64.0;
        
        // Compute the glyph’s drawing origin.
        let glyph_origin_x = origin_x + x_accum + x_offset;
        let glyph_origin_y = origin_y + y_offset;
        
        // Load the glyph into the FreeType face.
        // (The glyph index from HarfBuzz should match FreeType’s index.)
        ft_face.load_glyph(glyph_id, ft::face::LoadFlag::NO_BITMAP)?;
        let glyph_slot = ft_face.glyph();
        
        // If the glyph has an outline, convert it into a Skia Path.
        if let Some(outline) = glyph_slot.outline() {
            let mut path = Path::new();
            // Iterate over each contour in the outline.
            for contour in outline.contours_iter() {
                // Get the starting point of the contour.
                let start_pt = contour.start();
                // Convert from 26.6 fixed point to float (divide by 64)
                // and flip the y-axis (FreeType’s y goes up; Skia’s goes down).
                let start_x = start_pt.x as f32 / 64.0;
                let start_y = -start_pt.y as f32 / 64.0;
                path.move_to((start_x, start_y));
                
                // Process each curve segment in the contour.
                for curve in contour {
                    match curve {
                        ft::outline::Curve::Line(pt) => {
                            let x = pt.x as f32 / 64.0;
                            let y = -pt.y as f32 / 64.0;
                            path.line_to((x, y));
                        }
                        ft::outline::Curve::Bezier2(pt1, pt2) => {
                            let x1 = pt1.x as f32 / 64.0;
                            let y1 = -pt1.y as f32 / 64.0;
                            let x2 = pt2.x as f32 / 64.0;
                            let y2 = -pt2.y as f32 / 64.0;
                            path.quad_to((x1, y1), (x2, y2));
                        }
                        ft::outline::Curve::Bezier3(pt1, pt2, pt3) => {
                            let x1 = pt1.x as f32 / 64.0;
                            let y1 = -pt1.y as f32 / 64.0;
                            let x2 = pt2.x as f32 / 64.0;
                            let y2 = -pt2.y as f32 / 64.0;
                            let x3 = pt3.x as f32 / 64.0;
                            let y3 = -pt3.y as f32 / 64.0;
                            path.cubic_to((x1, y1), (x2, y2), (x3, y3));
                        }
                    }
                }
                path.close();
            }
            // Offset the path so that it is drawn at the correct glyph position.
            path.offset((glyph_origin_x, glyph_origin_y));
            canvas.draw_path(&path, &paint);
        }
        
        // Advance the horizontal position by the glyph’s advance width.
        x_accum += x_advance;
    }
    
    let image = surface.image_snapshot();
    let png_data = image
        .encode_to_data(EncodedImageFormat::PNG)
        .ok_or("Failed to encode image")?;
    fs::write("output_rtl.png", png_data.as_bytes())?;
    println!("Image saved as output_rtl.png");
    
    Ok(())
}
