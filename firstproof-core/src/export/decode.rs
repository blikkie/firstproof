use std::io::Cursor;
use std::path::Path;
use crate::color::{
    Color3, LINEAR_SRGB_TO_REC2020, srgb_gamma_to_linear,
};

pub struct DemosaicedImage {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>, // Width * Height * 3 (Linear RGB in range 0.0 .. 1.0)
    pub is_raw: bool,
}

impl DemosaicedImage {
    pub fn new(width: usize, height: usize, is_raw: bool) -> Self {
        Self {
            width,
            height,
            data: vec![0.0; width * height * 3],
            is_raw,
        }
    }
}

/// Decodes a RAW CFA Bayer sensor image or master standard image into linear 3-channel float RGB.
pub fn decode_image<P: AsRef<Path>>(path: P) -> Result<DemosaicedImage, String> {
    let path = path.as_ref();
    
    // 1. Try decoding with library-agnostic RawDecoder (backed by RawlerDecoder)
    let decoder = crate::raw_decoder::RawlerDecoder;
    if let Ok(frame) = crate::raw_decoder::RawDecoder::decode(&decoder, path) {
        let active_w = frame.active_width;
        let active_h = frame.active_height;
        let mut demosaiced = DemosaicedImage::new(active_w, active_h, true);
        let (wb_r, wb_b) = frame.as_shot_wb();
        
        // Derive Camera -> Linear Rec.2020 matrix dynamically interpolated
        // along the Planckian blackbody locus for the photo's as-shot / default daylight temperature (5500K).
        let sensor_to_rec2020 = frame.sensor_to_rec2020_at(5500.0);

        let bayer_cfa = match frame.cfa_pattern {
            crate::raw_decoder::CfaPattern::Rggb => Some(bayer::CFA::RGGB),
            crate::raw_decoder::CfaPattern::Bggr => Some(bayer::CFA::BGGR),
            crate::raw_decoder::CfaPattern::Gbrg => Some(bayer::CFA::GBRG),
            crate::raw_decoder::CfaPattern::Grbg => Some(bayer::CFA::GRBG),
            _ => None,
        };

        let mut demosaiced_with_bayer = false;
        if let Some(cfa) = bayer_cfa {
            let full_w = frame.full_width;
            let full_h = frame.full_height;
            let total_pixels = full_w * full_h;
            if total_pixels > 0 && frame.sensor_data.len() >= total_pixels {
                // Buffer for full frame 16-bit RGB: 3 channels * 2 bytes = 6 bytes per pixel
                let mut rgb_raw_bytes = vec![0u8; total_pixels * 6];
                let mut raster = bayer::RasterMut::new(full_w, full_h, bayer::RasterDepth::Depth16, &mut rgb_raw_bytes);
                
                let raw_u8_slice: &[u8] = unsafe {
                    std::slice::from_raw_parts(
                        frame.sensor_data.as_ptr() as *const u8,
                        frame.sensor_data.len() * 2,
                    )
                };
                let depth = if cfg!(target_endian = "big") {
                    bayer::BayerDepth::Depth16BE
                } else {
                    bayer::BayerDepth::Depth16LE
                };

                if bayer::run_demosaic(
                    &mut Cursor::new(raw_u8_slice),
                    depth,
                    cfa,
                    bayer::Demosaic::Cubic,
                    &mut raster,
                ).is_ok() {
                    let black = frame.black_levels[0].max(0.0);
                    let white = frame.white_levels[0].max(black + 1.0);
                    let scale = 1.0 / (white - black);

                    for y in 0..active_h {
                        let full_y = (frame.active_y + y).min(full_h - 1);
                        for x in 0..active_w {
                            let full_x = (frame.active_x + x).min(full_w - 1);
                            let pixel_byte_idx = (full_y * full_w + full_x) * 6;
                            let r_raw = u16::from_ne_bytes([rgb_raw_bytes[pixel_byte_idx], rgb_raw_bytes[pixel_byte_idx + 1]]) as f32;
                            let g_raw = u16::from_ne_bytes([rgb_raw_bytes[pixel_byte_idx + 2], rgb_raw_bytes[pixel_byte_idx + 3]]) as f32;
                            let b_raw = u16::from_ne_bytes([rgb_raw_bytes[pixel_byte_idx + 4], rgb_raw_bytes[pixel_byte_idx + 5]]) as f32;

                            let r_norm = ((r_raw - black) * scale).clamp(0.0, 1.0);
                            let g_norm = ((g_raw - black) * scale).clamp(0.0, 1.0);
                            let b_norm = ((b_raw - black) * scale).clamp(0.0, 1.0);

                            let c_sensor = Color3::new(
                                (r_norm * wb_r).clamp(0.0, 1.0),
                                g_norm.clamp(0.0, 1.0),
                                (b_norm * wb_b).clamp(0.0, 1.0),
                            );
                            let c_rec2020 = sensor_to_rec2020.multiply_vector(c_sensor);

                            let out_idx = (y * active_w + x) * 3;
                            demosaiced.data[out_idx] = c_rec2020.r().max(0.0);
                            demosaiced.data[out_idx + 1] = c_rec2020.g().max(0.0);
                            demosaiced.data[out_idx + 2] = c_rec2020.b().max(0.0);
                        }
                    }
                    demosaiced_with_bayer = true;
                }
            }
        }

        if !demosaiced_with_bayer {
            for y in 0..active_h {
                let full_y = (frame.active_y + y).min(frame.full_height - 2).max(1);
                
                for x in 0..active_w {
                    let full_x = (frame.active_x + x).min(frame.full_width - 2).max(1);
                    
                    let color = frame.color_at(full_y, full_x);
                    
                    let r: f32;
                    let g: f32;
                    let b: f32;
                    
                    if color == crate::raw_decoder::CFA_COLOR_RED {
                        // Red photosite
                        r = frame.sample_normalized(full_x, full_y);
                        g = 0.25 * (frame.sample_normalized(full_x - 1, full_y) + frame.sample_normalized(full_x + 1, full_y) + frame.sample_normalized(full_x, full_y - 1) + frame.sample_normalized(full_x, full_y + 1));
                        b = 0.25 * (frame.sample_normalized(full_x - 1, full_y - 1) + frame.sample_normalized(full_x + 1, full_y - 1) + frame.sample_normalized(full_x - 1, full_y + 1) + frame.sample_normalized(full_x + 1, full_y + 1));
                    } else if color == crate::raw_decoder::CFA_COLOR_BLUE {
                        // Blue photosite
                        b = frame.sample_normalized(full_x, full_y);
                        g = 0.25 * (frame.sample_normalized(full_x - 1, full_y) + frame.sample_normalized(full_x + 1, full_y) + frame.sample_normalized(full_x, full_y - 1) + frame.sample_normalized(full_x, full_y + 1));
                        r = 0.25 * (frame.sample_normalized(full_x - 1, full_y - 1) + frame.sample_normalized(full_x + 1, full_y - 1) + frame.sample_normalized(full_x - 1, full_y + 1) + frame.sample_normalized(full_x + 1, full_y + 1));
                    } else {
                        // Green photosite
                        g = frame.sample_normalized(full_x, full_y);
                        // Check if Red is horizontal or vertical neighbor
                        if frame.color_at(full_y, full_x - 1) == crate::raw_decoder::CFA_COLOR_RED || frame.color_at(full_y, full_x + 1) == crate::raw_decoder::CFA_COLOR_RED {
                            r = 0.5 * (frame.sample_normalized(full_x - 1, full_y) + frame.sample_normalized(full_x + 1, full_y));
                            b = 0.5 * (frame.sample_normalized(full_x, full_y - 1) + frame.sample_normalized(full_x, full_y + 1));
                        } else {
                            r = 0.5 * (frame.sample_normalized(full_x, full_y - 1) + frame.sample_normalized(full_x, full_y + 1));
                            b = 0.5 * (frame.sample_normalized(full_x - 1, full_y) + frame.sample_normalized(full_x + 1, full_y));
                        }
                    }
                    
                    // Sensor linear RGB with white balance
                    let c_sensor = Color3::new((r * wb_r).clamp(0.0, 1.0), g.clamp(0.0, 1.0), (b * wb_b).clamp(0.0, 1.0));
                    
                    // Map to Wide-Gamut Linear Rec.2020 working space
                    let c_rec2020 = sensor_to_rec2020.multiply_vector(c_sensor);
                    
                    let out_idx = (y * active_w + x) * 3;
                    demosaiced.data[out_idx] = c_rec2020.r().max(0.0);
                    demosaiced.data[out_idx + 1] = c_rec2020.g().max(0.0);
                    demosaiced.data[out_idx + 2] = c_rec2020.b().max(0.0);
                }
            }
        }
        
        return Ok(demosaiced);
    }
    
    // 2. Standard master image decode (JPEG, PNG, TIFF master files)
    let dyn_img = image::open(path).map_err(|e| format!("Failed to open image file: {}", e))?;
    let rgb = dyn_img.to_rgb8();
    let width = rgb.width() as usize;
    let height = rgb.height() as usize;
    
    let mut demosaiced = DemosaicedImage::new(width, height, false);
    for (i, pixel) in rgb.pixels().enumerate() {
        let out_idx = i * 3;
        let r_lin = srgb_gamma_to_linear((pixel[0] as f32) / 255.0);
        let g_lin = srgb_gamma_to_linear((pixel[1] as f32) / 255.0);
        let b_lin = srgb_gamma_to_linear((pixel[2] as f32) / 255.0);
        let c_srgb = Color3::new(r_lin, g_lin, b_lin);
        let c_rec2020 = LINEAR_SRGB_TO_REC2020.multiply_vector(c_srgb);
        demosaiced.data[out_idx] = c_rec2020.r().max(0.0);
        demosaiced.data[out_idx + 1] = c_rec2020.g().max(0.0);
        demosaiced.data[out_idx + 2] = c_rec2020.b().max(0.0);
    }
    
    Ok(demosaiced)
}

#[inline(always)]
pub fn sample_bilinear(image: &DemosaicedImage, x: f32, y: f32) -> (f32, f32, f32) {
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

/// Converts an f32 float to IEEE 754 binary16 (half-float) bit representation.
#[inline]
pub fn f32_to_f16(val: f32) -> u16 {
    let bits = val.to_bits();
    let sign = (bits >> 16) & 0x8000;
    let exp = ((bits >> 23) & 0xff) as i32 - 127 + 15;
    let mant = bits & 0x007f_ffff;
    if exp <= 0 {
        0
    } else if exp >= 31 {
        (sign | 0x7c00) as u16
    } else {
        (sign | ((exp as u32) << 10) | (mant >> 13)) as u16
    }
}

/// Converts an IEEE 754 binary16 (half-float) bit representation to an f32 float.
#[inline]
pub fn f16_to_f32(val: u16) -> f32 {
    let sign = ((val & 0x8000) as u32) << 16;
    let exp = ((val >> 10) & 0x1f) as u32;
    let mant = (val & 0x03ff) as u32;
    if exp == 0 {
        if mant == 0 {
            f32::from_bits(sign)
        } else {
            let mut m = mant;
            let mut e = 113;
            while (m & 0x0400) == 0 {
                m <<= 1;
                e -= 1;
            }
            m &= 0x03ff;
            f32::from_bits(sign | (e << 23) | (m << 13))
        }
    } else if exp == 31 {
        f32::from_bits(sign | 0x7f80_0000 | (mant << 13))
    } else {
        let e = exp + 127 - 15;
        f32::from_bits(sign | (e << 23) | (mant << 13))
    }
}

/// Decodes a RAW image into a 16-bit half-float RGBA preview buffer in Wide-Gamut Linear Rec.2020 space.
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
    
    let mut out_rgba_f16 = vec![0u8; target_w * target_h * 8];
    let a16 = f32_to_f16(1.0);
    
    for ty in 0..target_h {
        let sy = (ty as f32 + 0.5) * (orig_h as f32) / (target_h as f32);
        for tx in 0..target_w {
            let sx = (tx as f32 + 0.5) * (orig_w as f32) / (target_w as f32);
            let (r, g, b) = sample_bilinear(&demosaiced, sx, sy);
            
            let r16 = f32_to_f16(r.max(0.0));
            let g16 = f32_to_f16(g.max(0.0));
            let b16 = f32_to_f16(b.max(0.0));
            
            let out_idx = (ty * target_w + tx) * 8;
            out_rgba_f16[out_idx..out_idx + 2].copy_from_slice(&r16.to_le_bytes());
            out_rgba_f16[out_idx + 2..out_idx + 4].copy_from_slice(&g16.to_le_bytes());
            out_rgba_f16[out_idx + 4..out_idx + 6].copy_from_slice(&b16.to_le_bytes());
            out_rgba_f16[out_idx + 6..out_idx + 8].copy_from_slice(&a16.to_le_bytes());
        }
    }
    
    Ok((target_w as u32, target_h as u32, out_rgba_f16))
}
