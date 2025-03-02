use freetype as ft;
use rustybuzz::{Face, UnicodeBuffer, shape, Direction};
use skia_safe::{Color, EncodedImageFormat, Paint, Path, Surface};
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    // Load font data and create a FreeType face.
    let font_path = "NotoSans-VariableFont.ttf";
    let font_data = fs::read(font_path)?;

    let library = ft::Library::init()?;
    let ft_face = library.new_face(font_path, 0)?;
    
    // Set the desired pixel size for FreeType.
    let desired_font_size = 40.0;
    ft_face.set_pixel_sizes(0, desired_font_size as u32)?;
    
    // Create a rustybuzz face from the font data.
    let face = Face::from_slice(&font_data, 0).unwrap();
    
    // Get the font’s units per em (upem) and compute a scaling factor.
    let upem = face.units_per_em() as f32;
    let scale = desired_font_size / upem;
    
    // Build the UnicodeBuffer.
    let text = "ड्ड";
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(Direction::LeftToRight);
    
    // Shape the text.
    // Note: The arguments are (face, features, buffer). We use an empty features slice.
    let glyph_buffer = shape(&face, &[], buffer);
    let glyph_infos = glyph_buffer.glyph_infos();
    let glyph_positions = glyph_buffer.glyph_positions();
    
    // Create a drawing surface.
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
    let mut x_accum = 0.0;
    
    // Process each glyph from the shaping result.
    for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
        let glyph_id = info.glyph_id;
        // The shaping positions are in font units; scale them to pixels.
        let x_offset = pos.x_offset as f32 * scale;
        let y_offset = pos.y_offset as f32 * scale;
        let x_advance = pos.x_advance as f32 * scale;
        
        let glyph_origin_x = origin_x + x_accum + x_offset;
        let glyph_origin_y = origin_y + y_offset;
        
        // Load the glyph into FreeType (the glyph index should match).
        ft_face.load_glyph(glyph_id, ft::face::LoadFlag::NO_BITMAP)?;
        let glyph_slot = ft_face.glyph();
        
        // If the glyph has an outline, convert it into a Skia Path.
        if let Some(outline) = glyph_slot.outline() {
            let mut path = Path::new();
            for contour in outline.contours_iter() {
                let start_pt = contour.start();
                let start_x = start_pt.x as f32 / 64.0;
                let start_y = -start_pt.y as f32 / 64.0;
                path.move_to((start_x, start_y));
                
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
            // Offset the path to the correct glyph position.
            path.offset((glyph_origin_x, glyph_origin_y));
            canvas.draw_path(&path, &paint);
        }
        
        // Advance the current horizontal position.
        x_accum += x_advance;
    }
    
    let image = surface.image_snapshot();
    let png_data = image.encode_to_data(EncodedImageFormat::PNG)
        .ok_or("Failed to encode image")?;
    fs::write("output_ltr.png", png_data.as_bytes())?;
    println!("Image saved as output_ltr.png");
    
    Ok(())
}
