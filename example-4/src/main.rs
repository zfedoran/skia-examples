use harfbuzz_rs::{
    Face, Font as HbFont, UnicodeBuffer, GlyphInfo, GlyphPosition,
    shape
};
use skia_safe::{
    Color, Data, EncodedImageFormat, Font, FontMgr, Paint, Surface, TextBlobBuilder,
};
use unicode_segmentation::UnicodeSegmentation;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    // -------------------------------------------------
    // 1. Load and create HarfBuzz + Skia fonts
    // -------------------------------------------------

    let primary_font_path = "Roboto-LightItalic.ttf";      // or any Latin-capable font
    let fallback_font_path = "NotoColorEmoji-Regular.ttf"; // or any emoji-capable font

    let primary_data = fs::read(primary_font_path)?;
    let fallback_data = fs::read(fallback_font_path)?;

    // Create HarfBuzz face/font for primary
    let hb_face_primary = Face::from_bytes(&primary_data, 0);
    let mut hb_font_primary = HbFont::new(hb_face_primary);
    // We'll shape at 20px => 20 * 64 = 1280 in 26.6 fixed point
    let px_size = 20.0;
    let hb_scale = (px_size * 64.) as i32;
    hb_font_primary.set_scale(hb_scale, hb_scale);
    // Optional: set_ppem if you want hinting
    // hb_font_primary.set_ppem(px_size as u32, px_size as u32);

    // Create HarfBuzz face/font for fallback
    let hb_face_fallback = Face::from_bytes(&fallback_data, 0);
    let mut hb_font_fallback = HbFont::new(hb_face_fallback);
    hb_font_fallback.set_scale(hb_scale, hb_scale);
    // hb_font_fallback.set_ppem(px_size as u32, px_size as u32);

    // Create parallel Skia Font objects (so we can actually draw):
    let skia_data_primary = Data::new_copy(&primary_data);
    let skia_data_fallback = Data::new_copy(&fallback_data);

    let font_mgr = FontMgr::new();
    let typeface_primary = font_mgr
        .new_from_data(&skia_data_primary, None)
        .ok_or("Failed to load primary typeface")?;
    let typeface_fallback = font_mgr
        .new_from_data(&skia_data_fallback, None)
        .ok_or("Failed to load fallback typeface")?;

    let mut skia_font_primary = Font::default();
    skia_font_primary.set_size(px_size);
    skia_font_primary.set_typeface(typeface_primary);

    let mut skia_font_fallback = Font::default();
    skia_font_fallback.set_size(px_size);
    skia_font_fallback.set_typeface(typeface_fallback);

    // -------------------------------------------------
    // 2. Example string with emojis
    // -------------------------------------------------
    let text = "Hello, world 🌎";

    // -------------------------------------------------
    // 3. Segment text by grapheme clusters
    //    (so we don't split a multi-codepoint emoji)
    // -------------------------------------------------
    let graphemes = text.graphemes(true);

    // We'll accumulate everything into a single set of glyphs+positions
    // We'll also track "which HarfBuzz font" was used for each cluster
    // so we know which Skia font to use for drawing that cluster's glyphs.
    // We'll store these as "runs" for building a Skia TextBlob.
    // In a more sophisticated approach, you'd do fewer runs by merging adjacent
    // clusters that used the same font, but let's keep it simple.
    let mut shaped_runs = Vec::new();

    for cluster in graphemes {
        // shape with primary
        let (infos, positions) = shape_cluster(&hb_font_primary, cluster);
        // Check if we got only missing glyphs (codepoint=0). If so, fallback.
        let has_valid_glyph = infos.iter().any(|info| info.codepoint != 0);
        if has_valid_glyph {
            shaped_runs.push((infos, positions, FontChoice::Primary));
        } else {
            // shape with fallback
            let (infos_fb, positions_fb) = shape_cluster(&hb_font_fallback, cluster);
            shaped_runs.push((infos_fb, positions_fb, FontChoice::Fallback));
        }
    }

    // -------------------------------------------------
    // 4. Build a single Skia TextBlob from these runs
    // -------------------------------------------------
    // We'll do this by:
    //   - flattening the shaped data into (glyphId, xOffset, yOffset) arrays
    //   - but we have to break them up into separate runs if the font changes

    // For naive simplicity, let's do "one run per cluster".
    // (A more advanced approach would gather consecutive clusters that share the same font.)
    let mut blob_builder = TextBlobBuilder::new();

    // We'll place runs one after another horizontally. We'll track a global "x" offset.
    let mut x_cursor = 0.0;

    // We'll store "clusters" in runs. Each run is just one cluster in this simplified approach.
    for (infos, positions, which_font) in shaped_runs {
        // Pick the matching Skia font
        let skfont = match which_font {
            FontChoice::Primary => &skia_font_primary,
            FontChoice::Fallback => &skia_font_fallback,
        };

        let count = infos.len();
        if count == 0 {
            continue;
        }

        // Start a run
        let (glyphs, point_positions) = blob_builder.alloc_run_pos(skfont, count, None);

        // We'll keep track of local x as we place glyphs from this cluster
        let mut local_x = 0.0;
        for i in 0..count {
            let info = &infos[i];
            let pos = &positions[i];

            glyphs[i] = info.codepoint as u16;

            // HarfBuzz returns positions in 26.6 fixed -> /64.0
            let x_offset = pos.x_offset as f32 / 64.0;
            let y_offset = pos.y_offset as f32 / 64.0;
            let x_advance = pos.x_advance as f32 / 64.0;
            // typically y_advance is zero in horizontal text, but let's read it anyway:
            // let y_advance = pos.y_advance as f32 / 64.0;

            point_positions[i] = skia_safe::Point::new(x_cursor + local_x + x_offset, 50.0 + y_offset);

            // Move local_x by the horizontal advance
            local_x += x_advance;

            // If you had vertical text, you'd also add y_advance, etc.
        }
        // After finishing the run (cluster), we shift x_cursor by the total "local_x"
        x_cursor += local_x;
    }

    let text_blob = blob_builder.make().ok_or("Failed to build text blob")?;

    // -------------------------------------------------
    // 5. Draw to a Skia surface
    // -------------------------------------------------
    let width = 300;
    let height = 120;
    let mut surface = Surface::new_raster_n32_premul((width, height))
        .ok_or("Could not create a surface")?;
    let canvas = surface.canvas();
    canvas.clear(Color::WHITE);

    // Just draw the entire text_blob
    let paint = Paint::default();
    canvas.draw_text_blob(&text_blob, (50, 25), &paint);

    // Save result
    let image = surface.image_snapshot();
    let png_data = image
        .encode_to_data(EncodedImageFormat::PNG)
        .ok_or("Failed to encode image")?;
    fs::write("fallback_hb.png", png_data.as_bytes())?;
    println!("Wrote fallback_hb.png");
    Ok(())
}

/// A tiny helper to shape a single cluster (grapheme) with a given HarfBuzz font.
fn shape_cluster(hb_font: &harfbuzz_rs::Font, text: &str) -> (Vec<GlyphInfo>, Vec<GlyphPosition>) {
    // Create a buffer, add our cluster text, shape it horizontally (LTR) just for the example.
    // If your text might be RTL, you can set_direction(Direction::Rtl).
    //
    let shaped_buf = shape(
        hb_font,
        UnicodeBuffer::new()
            .add_str(text),
            // You could set script, language, direction as needed:
            //   .set_script(Tag::new('Z','y','y','y'))  // "Zyyy" = Common script, for example
            //   .set_language(Language::from_str("en").unwrap())
            //   .set_direction(Direction::Ltr)
        &[],  // optional features
    );

    let infos = shaped_buf.get_glyph_infos().to_vec();
    let positions = shaped_buf.get_glyph_positions().to_vec();

    (infos, positions)
}

#[derive(Debug)]
enum FontChoice {
    Primary,
    Fallback,
}
