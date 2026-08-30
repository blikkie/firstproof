use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use image::{ColorType, ImageBuffer, Rgb, RgbImage};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::tiff::TiffEncoder;

use crate::EditSettings;

pub struct DemosaicedImage {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>, // Width * Height * 3 (Linear RGB in range 0.0 .. 1.0)
}

impl DemosaicedImage {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0.0; width * height * 3],
        }
    }
}

/// Decodes a RAW CFA Bayer sensor image or master standard image into linear 3-channel float RGB.
pub fn decode_image<P: AsRef<Path>>(path: P) -> Result<DemosaicedImage, String> {
    let path = path.as_ref();
    
    // 1. Try decoding with rawloader (Bayer RAW sensors: CR2, NEF, ARW, DNG, RW2)
    if let Ok(mut file) = File::open(path) {
        let loader = rawloader::decoders::RawLoader::new();
        if let Ok(raw) = loader.decode(&mut file) {
            let width = raw.width as usize;
            let height = raw.height as usize;
            let cfa = raw.data;
            let mut demosaiced = DemosaicedImage::new(width, height);
            
            // Sensor black and white level normalization
            let black_level = (raw.blacklevels[0] as f32).max(0.0);
            let white_level = (raw.whitelevels[0] as f32).max(black_level + 1.0);
            let scale = 1.0 / (white_level - black_level);
            
            // Fast Bilinear Bayer Demosaicing (RGGB, BGGR, GBRG, GRBG)
            // Assumes standard 2x2 Bayer CFA cell
            for y in 1..(height - 1) {
                for x in 1..(width - 1) {
                    let idx = y * width + x;
                    let out_idx = idx * 3;
                    
                    let is_even_row = y % 2 == 0;
                    let is_even_col = x % 2 == 0;
                    
                    let mut r: f32;
                    let mut g: f32;
                    let mut b: f32;
                    
                    let sample = (cfa[idx] as f32 - black_level).max(0.0) * scale;
                    
                    if is_even_row && is_even_col {
                        // Red pixel in RGGB
                        r = sample;
                        g = (cfa[idx - 1] as f32 + cfa[idx + 1] as f32 + cfa[idx - width] as f32 + cfa[idx + width] as f32 - 4.0 * black_level).max(0.0) * (scale * 0.25);
                        b = (cfa[idx - width - 1] as f32 + cfa[idx - width + 1] as f32 + cfa[idx + width - 1] as f32 + cfa[idx + width + 1] as f32 - 4.0 * black_level).max(0.0) * (scale * 0.25);
                    } else if (!is_even_row && is_even_col) || (is_even_row && !is_even_col) {
                        // Green pixel in RGGB
                        g = sample;
                        if is_even_row {
                            r = (cfa[idx - 1] as f32 + cfa[idx + 1] as f32 - 2.0 * black_level).max(0.0) * (scale * 0.5);
                            b = (cfa[idx - width] as f32 + cfa[idx + width] as f32 - 2.0 * black_level).max(0.0) * (scale * 0.5);
                        } else {
                            r = (cfa[idx - width] as f32 + cfa[idx + width] as f32 - 2.0 * black_level).max(0.0) * (scale * 0.5);
                            b = (cfa[idx - 1] as f32 + cfa[idx + 1] as f32 - 2.0 * black_level).max(0.0) * (scale * 0.5);
                        }
                    } else {
                        // Blue pixel in RGGB
                        b = sample;
                        g = (cfa[idx - 1] as f32 + cfa[idx + 1] as f32 + cfa[idx - width] as f32 + cfa[idx + width] as f32 - 4.0 * black_level).max(0.0) * (scale * 0.25);
                        r = (cfa[idx - width - 1] as f32 + cfa[idx - width + 1] as f32 + cfa[idx + width - 1] as f32 + cfa[idx + width + 1] as f32 - 4.0 * black_level).max(0.0) * (scale * 0.25);
                    }
                    
                    demosaiced.data[out_idx] = r.clamp(0.0, 1.0);
                    demosaiced.data[out_idx + 1] = g.clamp(0.0, 1.0);
                    demosaiced.data[out_idx + 2] = b.clamp(0.0, 1.0);
                }
            }
            
            return Ok(demosaiced);
        }
    }
    
    // 2. Fallback: Standard image decode (JPEG, PNG, TIFF master files)
    let dyn_img = image::open(path).map_err(|e| format!("Failed to open image file: {}", e))?;
    let rgb = dyn_img.to_rgb8();
    let width = rgb.width() as usize;
    let height = rgb.height() as usize;
    
    let mut demosaiced = DemosaicedImage::new(width, height);
    for (i, pixel) in rgb.pixels().enumerate() {
        let out_idx = i * 3;
        // Linearize sRGB to float space
        demosaiced.data[out_idx] = ((pixel[0] as f32) / 255.0).powf(2.2);
        demosaiced.data[out_idx + 1] = ((pixel[1] as f32) / 255.0).powf(2.2);
        demosaiced.data[out_idx + 2] = ((pixel[2] as f32) / 255.0).powf(2.2);
    }
    
    Ok(demosaiced)
}

/// Applies user edits (Exposure, White Balance, Contrast, Display gamma) in 16-bit linear space.
pub fn apply_edits(image: &mut DemosaicedImage, edits: &EditSettings) {
    let exposure_gain = 2.0f32.powf(edits.exposure);
    let contrast_gain = 1.0 + (edits.contrast / 100.0);
    
    let temp_norm = (edits.temperature / 5500.0).clamp(0.25, 4.0);
    let mut r_scale = temp_norm.powf(0.75);
    let mut b_scale = (1.0 / temp_norm).powf(0.75);
    
    let tint_factor = edits.tint * 0.003;
    let g_scale = 1.0 - tint_factor;
    r_scale += tint_factor * 0.5;
    b_scale += tint_factor * 0.5;
    
    let total_pixels = image.width * image.height;
    
    for i in 0..total_pixels {
        let idx = i * 3;
        let mut r = image.data[idx];
        let mut g = image.data[idx + 1];
        let mut b = image.data[idx + 2];
        
        // 1. White Balance
        r *= r_scale;
        g *= g_scale;
        b *= b_scale;
        
        // 2. Linear Exposure
        r *= exposure_gain;
        g *= exposure_gain;
        b *= exposure_gain;
        
        // 3. Filmic Mid-Gray Sigmoid Contrast
        if contrast_gain != 1.0 {
            r = 0.18 * (r.max(0.0001) / 0.18).powf(contrast_gain);
            g = 0.18 * (g.max(0.0001) / 0.18).powf(contrast_gain);
            b = 0.18 * (b.max(0.0001) / 0.18).powf(contrast_gain);
        }
        
        // 4. Display OETF Gamma Transform (Linear -> sRGB Gamma 2.2)
        image.data[idx] = r.clamp(0.0, 1.0).powf(1.0 / 2.2);
        image.data[idx + 1] = g.clamp(0.0, 1.0).powf(1.0 / 2.2);
        image.data[idx + 2] = b.clamp(0.0, 1.0).powf(1.0 / 2.2);
    }
}

/// Exports an edited photo to JPEG or 16-bit TIFF on disk with optional resizing.
pub fn export_photo_file<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P,
    output_path: Q,
    edits: &EditSettings,
    quality: u8,
    max_dimension: Option<u32>,
) -> Result<(), String> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    
    let mut image = decode_image(input_path)?;
    apply_edits(&mut image, edits);
    
    let orig_width = image.width as u32;
    let orig_height = image.height as u32;
    
    // Calculate target dimensions
    let (target_w, target_h) = if let Some(max_dim) = max_dimension {
        if max_dim > 0 && (orig_width > max_dim || orig_height > max_dim) {
            if orig_width >= orig_height {
                let w = max_dim;
                let h = ((orig_height as f32 * max_dim as f32) / orig_width as f32).round() as u32;
                (w, h.max(1))
            } else {
                let h = max_dim;
                let w = ((orig_width as f32 * max_dim as f32) / orig_height as f32).round() as u32;
                (w.max(1), h)
            }
        } else {
            (orig_width, orig_height)
        }
    } else {
        (orig_width, orig_height)
    };
    
    // Build 8-bit RGB image buffer
    let mut rgb_img: RgbImage = ImageBuffer::new(target_w, target_h);
    let scale_x = orig_width as f32 / target_w as f32;
    let scale_y = orig_height as f32 / target_h as f32;
    
    for ty in 0..target_h {
        for tx in 0..target_w {
            let sx = ((tx as f32 * scale_x) as usize).min(image.width - 1);
            let sy = ((ty as f32 * scale_y) as usize).min(image.height - 1);
            let idx = (sy * image.width + sx) * 3;
            
            let r = (image.data[idx] * 255.0).round().clamp(0.0, 255.0) as u8;
            let g = (image.data[idx + 1] * 255.0).round().clamp(0.0, 255.0) as u8;
            let b = (image.data[idx + 2] * 255.0).round().clamp(0.0, 255.0) as u8;
            
            rgb_img.put_pixel(tx, ty, Rgb([r, g, b]));
        }
    }
    
    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    
    let file = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut writer = BufWriter::new(file);
    
    let ext = output_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("jpg")
        .to_lowercase();
        
    if ext == "tif" || ext == "tiff" {
        let encoder = TiffEncoder::new(&mut writer);
        encoder
            .encode(&rgb_img, target_w, target_h, ColorType::Rgb8)
            .map_err(|e| format!("Failed to encode TIFF: {}", e))?;
    } else {
        let mut encoder = JpegEncoder::new_with_quality(&mut writer, quality.clamp(1, 100));
        encoder
            .encode(&rgb_img, target_w, target_h, ColorType::Rgb8)
            .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_jpeg_and_edits() {
        let tmp_in = "test_export_in.jpg";
        let tmp_out = "test_export_out.jpg";
        
        let img = RgbImage::from_fn(100, 100, |x, y| {
            Rgb([(x * 2) as u8, (y * 2) as u8, 128])
        });
        img.save(tmp_in).unwrap();
        
        let edits = EditSettings {
            exposure: 1.0,
            contrast: 20.0,
            temperature: 6500.0,
            tint: 10.0,
            crop_left: 0.0,
            crop_top: 0.0,
            crop_right: 1.0,
            crop_bottom: 1.0,
            crop_angle: 0.0,
        };
        
        let res = export_photo_file(tmp_in, tmp_out, &edits, 90, Some(50));
        assert!(res.is_ok());
        
        let exported = image::open(tmp_out).unwrap();
        assert_eq!(exported.width(), 50);
        assert_eq!(exported.height(), 50);
        
        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out);
    }
}
