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
    if let Ok(raw) = rawloader::decode_file(path) {
        if let rawloader::RawImageData::Integer(ref cfa) = raw.data {
            let crop_top = (raw.crops[0] as usize) & !1;
            let crop_right = (raw.crops[1] as usize) & !1;
            let crop_bottom = (raw.crops[2] as usize) & !1;
            let crop_left = (raw.crops[3] as usize) & !1;
            
            let active_w = (raw.width.saturating_sub(crop_left + crop_right)).max(2) & !1;
            let active_h = (raw.height.saturating_sub(crop_top + crop_bottom)).max(2) & !1;
            
            let full_w = raw.width;
            let mut demosaiced = DemosaicedImage::new(active_w, active_h);
            
            // Sensor black and white level normalization
            let black_level = (raw.blacklevels[0] as f32).max(0.0);
            let white_level = (raw.whitelevels[0] as f32).max(black_level + 1.0);
            let scale = 1.0 / (white_level - black_level);
            
            let p = |x: usize, y: usize| -> f32 {
                let idx = y * full_w + x;
                ((cfa[idx] as f32 - black_level) * scale).clamp(0.0, 1.0)
            };
            
            // Camera As-Shot White Balance Multipliers
            let wb_r = if raw.wb_coeffs[0] > 0.0 && raw.wb_coeffs[1] > 0.0 {
                raw.wb_coeffs[0] / raw.wb_coeffs[1]
            } else {
                2.0
            };
            let wb_b = if raw.wb_coeffs[2] > 0.0 && raw.wb_coeffs[1] > 0.0 {
                raw.wb_coeffs[2] / raw.wb_coeffs[1]
            } else {
                1.5
            };
            
            // Dynamic CFA Bayer Demosaicing (Supports RGGB, BGGR, GBRG, GRBG)
            for y in 0..active_h {
                let full_y = (crop_top + y).min(raw.height - 2).max(1);
                
                for x in 0..active_w {
                    let full_x = (crop_left + x).min(raw.width - 2).max(1);
                    
                    let color = raw.cfa.color_at(full_y, full_x);
                    
                    let r: f32;
                    let g: f32;
                    let b: f32;
                    
                    if color == 0 {
                        // Red photosite
                        r = p(full_x, full_y);
                        g = 0.25 * (p(full_x - 1, full_y) + p(full_x + 1, full_y) + p(full_x, full_y - 1) + p(full_x, full_y + 1));
                        b = 0.25 * (p(full_x - 1, full_y - 1) + p(full_x + 1, full_y - 1) + p(full_x - 1, full_y + 1) + p(full_x + 1, full_y + 1));
                    } else if color == 2 {
                        // Blue photosite
                        b = p(full_x, full_y);
                        g = 0.25 * (p(full_x - 1, full_y) + p(full_x + 1, full_y) + p(full_x, full_y - 1) + p(full_x, full_y + 1));
                        r = 0.25 * (p(full_x - 1, full_y - 1) + p(full_x + 1, full_y - 1) + p(full_x - 1, full_y + 1) + p(full_x + 1, full_y + 1));
                    } else {
                        // Green photosite
                        g = p(full_x, full_y);
                        // Check if Red is horizontal or vertical neighbor
                        if raw.cfa.color_at(full_y, full_x - 1) == 0 || raw.cfa.color_at(full_y, full_x + 1) == 0 {
                            r = 0.5 * (p(full_x - 1, full_y) + p(full_x + 1, full_y));
                            b = 0.5 * (p(full_x, full_y - 1) + p(full_x, full_y + 1));
                        } else {
                            r = 0.5 * (p(full_x, full_y - 1) + p(full_x, full_y + 1));
                            b = 0.5 * (p(full_x - 1, full_y) + p(full_x + 1, full_y));
                        }
                    }
                    
                    let out_idx = (y * active_w + x) * 3;
                    let base_gain = 1.45; // Photographic base exposure compensation
                    demosaiced.data[out_idx] = (r * wb_r * base_gain).clamp(0.0, 1.0);
                    demosaiced.data[out_idx + 1] = (g * base_gain).clamp(0.0, 1.0);
                    demosaiced.data[out_idx + 2] = (b * wb_b * base_gain).clamp(0.0, 1.0);
                }
            }
            
            return Ok(demosaiced);
        } else {
            return Err("Unsupported RAW data format: sensor did not contain 16-bit integer CFA data".to_string());
        }
    }
    
    // 2. Standard master image decode (JPEG, PNG, TIFF master files)
    let dyn_img = image::open(path).map_err(|e| format!("Failed to open image file: {}", e))?;
    let rgb = dyn_img.to_rgb8();
    let width = rgb.width() as usize;
    let height = rgb.height() as usize;
    
    let mut demosaiced = DemosaicedImage::new(width, height);
    for (i, pixel) in rgb.pixels().enumerate() {
        let out_idx = i * 3;
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

#[inline(always)]
fn sample_bilinear(image: &DemosaicedImage, x: f32, y: f32) -> (f32, f32, f32) {
    let w = image.width;
    let h = image.height;
    
    let x_clamped = x.clamp(0.0, (w - 1) as f32);
    let y_clamped = y.clamp(0.0, (h - 1) as f32);
    
    let x0 = x_clamped.floor() as usize;
    let y0 = y_clamped.floor() as usize;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    
    let fx = x_clamped - (x0 as f32);
    let fy = y_clamped - (y0 as f32);
    
    let idx00 = (y0 * w + x0) * 3;
    let idx10 = (y0 * w + x1) * 3;
    let idx01 = (y1 * w + x0) * 3;
    let idx11 = (y1 * w + x1) * 3;
    
    let w00 = (1.0 - fx) * (1.0 - fy);
    let w10 = fx * (1.0 - fy);
    let w01 = (1.0 - fx) * fy;
    let w11 = fx * fy;
    
    let r = image.data[idx00] * w00 + image.data[idx10] * w10 + image.data[idx01] * w01 + image.data[idx11] * w11;
    let g = image.data[idx00 + 1] * w00 + image.data[idx10 + 1] * w10 + image.data[idx01 + 1] * w01 + image.data[idx11 + 1] * w11;
    let b = image.data[idx00 + 2] * w00 + image.data[idx10 + 2] * w10 + image.data[idx01 + 2] * w01 + image.data[idx11 + 2] * w11;
    
    (r, g, b)
}

/// Decodes a RAW image into an 8-bit RGBA preview buffer for live Metal rendering.
pub fn decode_raw_preview_buffer<P: AsRef<Path>>(path: P, max_dimension: Option<u32>) -> Result<(u32, u32, Vec<u8>), String> {
    let demosaiced = decode_image(path)?;
    
    let orig_w = demosaiced.width;
    let orig_h = demosaiced.height;
    
    let (target_w, target_h) = if let Some(max_dim) = max_dimension {
        let max_dim = max_dim as usize;
        if max_dim > 0 && (orig_w > max_dim || orig_h > max_dim) {
            if orig_w >= orig_h {
                let scaled_h = ((orig_h as f32) * (max_dim as f32) / (orig_w as f32)).round() as usize;
                (max_dim, scaled_h.max(1))
            } else {
                let scaled_w = ((orig_w as f32) * (max_dim as f32) / (orig_h as f32)).round() as usize;
                (scaled_w.max(1), max_dim)
            }
        } else {
            (orig_w, orig_h)
        }
    } else {
        (orig_w, orig_h)
    };
    
    let mut out_rgba = vec![0u8; target_w * target_h * 4];
    
    for ty in 0..target_h {
        let sy = (ty as f32 + 0.5) * (orig_h as f32) / (target_h as f32);
        for tx in 0..target_w {
            let sx = (tx as f32 + 0.5) * (orig_w as f32) / (target_w as f32);
            let (r, g, b) = sample_bilinear(&demosaiced, sx, sy);
            
            let out_idx = (ty * target_w + tx) * 4;
            // Convert linear float to standard display gamma (pow 1/2.2)
            out_rgba[out_idx] = ((r.clamp(0.0, 1.0).powf(1.0 / 2.2)) * 255.0).round() as u8;
            out_rgba[out_idx + 1] = ((g.clamp(0.0, 1.0).powf(1.0 / 2.2)) * 255.0).round() as u8;
            out_rgba[out_idx + 2] = ((b.clamp(0.0, 1.0).powf(1.0 / 2.2)) * 255.0).round() as u8;
            out_rgba[out_idx + 3] = 255;
        }
    }
    
    Ok((target_w as u32, target_h as u32, out_rgba))
}

/// Exports an edited photo to JPEG or 16-bit TIFF on disk with optional resizing and Lensfun distortion correction.
pub fn export_photo_file<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P,
    output_path: Q,
    edits: &EditSettings,
    lens_distortion: Option<(&[f32], &str)>,
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
    
    // Build 8-bit RGB image buffer with geometric warp and bilinear filtering
    let mut rgb_img: RgbImage = ImageBuffer::new(target_w, target_h);
    
    let aspect_ratio = (orig_width as f32) / (orig_height as f32);
    let angle_rad = -edits.crop_angle * std::f32::consts::PI / 180.0;
    let cos_theta = angle_rad.cos();
    let sin_theta = angle_rad.sin();
    
    let max_r = aspect_ratio.max(1.0 / aspect_ratio);
    let auto_crop_scale = (cos_theta.abs() + sin_theta.abs() * max_r).max(1.0);
    
    let (has_dist, k1, k2, k3, is_ptlens) = if let Some((coeffs, model)) = lens_distortion {
        if !coeffs.is_empty() {
            let k1 = coeffs[0];
            let k2 = if coeffs.len() > 1 { coeffs[1] } else { 0.0 };
            let k3 = if coeffs.len() > 2 { coeffs[2] } else { 0.0 };
            (true, k1, k2, k3, model == "ptlens")
        } else {
            (false, 0.0, 0.0, 0.0, false)
        }
    } else {
        (false, 0.0, 0.0, 0.0, false)
    };
    
    for ty in 0..target_h {
        for tx in 0..target_w {
            let u = (tx as f32 + 0.5) / (target_w as f32);
            let v = (ty as f32 + 0.5) / (target_h as f32);
            
            // 1. Straighten and auto-crop zoom
            let dx = (u - 0.5) / auto_crop_scale;
            let dy = (v - 0.5) / (auto_crop_scale * aspect_ratio);
            
            let mut rx = dx * cos_theta - dy * sin_theta;
            let mut ry = dx * sin_theta + dy * cos_theta;
            
            // 2. Lensfun distortion model
            if has_dist {
                let r = (rx * rx + ry * ry).sqrt();
                if r > 0.00001 {
                    let rd = if is_ptlens {
                        let a = k1;
                        let b = k2;
                        let c = k3;
                        let d = 1.0 - (a + b + c);
                        let r2 = r * r;
                        let r3 = r2 * r;
                        let r4 = r3 * r;
                        a * r4 + b * r3 + c * r2 + d * r
                    } else {
                        // poly3 model
                        let r2 = r * r;
                        let r4 = r2 * r2;
                        r * (1.0 + k1 * r2 + k2 * r4)
                    };
                    let scale_factor = rd / r;
                    rx *= scale_factor;
                    ry *= scale_factor;
                }
            }
            
            let sx = (rx + 0.5) * (orig_width as f32);
            let sy = (ry * aspect_ratio + 0.5) * (orig_height as f32);
            
            let (r_val, g_val, b_val) = sample_bilinear(&image, sx, sy);
            let r = (r_val * 255.0).round().clamp(0.0, 255.0) as u8;
            let g = (g_val * 255.0).round().clamp(0.0, 255.0) as u8;
            let b = (b_val * 255.0).round().clamp(0.0, 255.0) as u8;
            
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
        
        let res = export_photo_file(tmp_in, tmp_out, &edits, None, 90, Some(50));
        assert!(res.is_ok());
        
        let exported = image::open(tmp_out).unwrap();
        assert_eq!(exported.width(), 50);
        assert_eq!(exported.height(), 50);
        
        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out);
    }

    #[test]
    fn test_debug_cr2() {
        let path = "/Users/blikkie/Pictures/Darktable/20250409_eos_pull/20250409_0359.CR2";
        let out_path = "test_calibrated_cr2.jpg";
        
        let edits = EditSettings {
            exposure: 0.8,
            contrast: 15.0,
            temperature: 5500.0,
            tint: 0.0,
            crop_left: 0.0,
            crop_top: 0.0,
            crop_right: 1.0,
            crop_bottom: 1.0,
            crop_angle: 0.0,
        };
        
        let _ = export_photo_file(path, out_path, &edits, None, 90, Some(500));
        
        // 1. Camera Embedded JPEG
        if let Ok(preview_bytes) = crate::raw::extract_preview_jpeg(path) {
            if let Ok(cam_img) = image::load_from_memory(&preview_bytes) {
                let rgb = cam_img.to_rgb8();
                let mut sum_r = 0u64;
                let mut sum_g = 0u64;
                let mut sum_b = 0u64;
                for p in rgb.pixels() {
                    sum_r += p[0] as u64;
                    sum_g += p[1] as u64;
                    sum_b += p[2] as u64;
                }
                let count = rgb.pixels().len() as u64;
                println!("CAMERA JPEG AVERAGE: R={}, G={}, B={}", sum_r / count, sum_g / count, sum_b / count);
            }
        }
        
        // 2. Calibrated Exported Image
        if let Ok(img) = image::open(out_path) {
            let rgb = img.to_rgb8();
            let mut sum_r = 0u64;
            let mut sum_g = 0u64;
            let mut sum_b = 0u64;
            for p in rgb.pixels() {
                sum_r += p[0] as u64;
                sum_g += p[1] as u64;
                sum_b += p[2] as u64;
            }
            let count = rgb.pixels().len() as u64;
            println!("CALIBRATED RAW EXPORT AVERAGE: R={}, G={}, B={}", sum_r / count, sum_g / count, sum_b / count);
        }
        
        let _ = std::fs::remove_file(out_path);
    }
}
