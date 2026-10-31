use std::path::Path;
use rayon::prelude::*;
use crate::color::{
    Color3, LINEAR_SRGB_TO_REC2020, srgb_gamma_to_linear,
};
use crate::DemosaicQuality;

pub struct DemosaicedImage {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>, // Width * Height * 3 (Linear RGB in range 0.0 .. 1.0)
    pub is_raw: bool,
    pub baseline_sharpness: f32,
    pub profile_tone_curve: Option<Vec<(f32, f32)>>,
    pub warp_rectilinear: Option<([f64; 4], [f64; 2])>,
    pub as_shot_temperature: Option<f32>,
    pub as_shot_tint: Option<f32>,
}

impl DemosaicedImage {
    pub fn new(width: usize, height: usize, is_raw: bool) -> Self {
        Self {
            width,
            height,
            data: vec![0.0; width * height * 3],
            is_raw,
            baseline_sharpness: 1.0,
            profile_tone_curve: None,
            warp_rectilinear: None,
            as_shot_temperature: None,
            as_shot_tint: None,
        }
    }
}

/// Decodes a RAW CFA Bayer sensor image or master standard image into linear 3-channel float RGB using default standard quality.
pub fn decode_image<P: AsRef<Path>>(path: P) -> Result<DemosaicedImage, String> {
    decode_image_with_quality(path, DemosaicQuality::Standard)
}

/// Decodes a RAW CFA Bayer sensor image or master standard image with selectable demosaicing quality (Standard vs Archival).
pub fn decode_image_with_quality<P: AsRef<Path>>(
    path: P,
    quality: DemosaicQuality,
) -> Result<DemosaicedImage, String> {
    let path = path.as_ref();
    
    // 1. Try decoding with library-agnostic RawDecoder (backed by RawlerDecoder)
    let decoder = crate::raw_decoder::RawlerDecoder;
    if let Ok(frame) = crate::raw_decoder::RawDecoder::decode(&decoder, path) {
        let active_w = frame.active_width;
        let active_h = frame.active_height;
        let mut demosaiced = DemosaicedImage::new(active_w, active_h, true);
        demosaiced.baseline_sharpness = frame.baseline_sharpness;
        demosaiced.profile_tone_curve = frame.profile_tone_curve.clone();
        demosaiced.warp_rectilinear = frame.warp_rectilinear;
        let (wb_r, wb_b) = frame.as_shot_wb();
        
        // Derive Camera -> Linear Rec.2020 matrix dynamically interpolated
        // along the Planckian blackbody locus for the photo's as-shot temperature.
        let as_shot_temp = frame.as_shot_temperature();
        demosaiced.as_shot_temperature = Some(as_shot_temp);
        demosaiced.as_shot_tint = Some(frame.as_shot_tint());
        let sensor_to_rec2020 = {
            use crate::raw_decoder::RawSensorFrameExt;
            frame.sensor_to_rec2020_matrix_at(as_shot_temp)
        };

        let ppg_cfa = match frame.cfa_pattern {
            crate::raw_decoder::CfaPattern::Rggb => Some("RGGB"),
            crate::raw_decoder::CfaPattern::Bggr => Some("BGGR"),
            crate::raw_decoder::CfaPattern::Gbrg => Some("GBRG"),
            crate::raw_decoder::CfaPattern::Grbg => Some("GRBG"),
            _ => None,
        };

        let mut demosaiced_with_soft_adaptive = false;
        if ppg_cfa.is_some() {
            let full_w = frame.full_width;
            let full_h = frame.full_height;
            let total_pixels = full_w * full_h;
            if total_pixels > 0 && frame.sensor_data.len() >= total_pixels {
                let mut norm_pixels: Vec<f32> = frame.sensor_data.iter().map(|&v| v as f32).collect();
                rawler::imgop::raw::correct_blacklevel_cfa(
                    &mut norm_pixels,
                    full_w,
                    full_h,
                    &frame.black_levels,
                    &frame.white_levels,
                );

                // Pre-demosaicing White Balance (standard in dcraw / LibRaw):
                // Applying white balance multipliers directly to CFA photosites before demosaicing
                // ensures cross-channel edge gradients (R vs G, B vs G) and Laplacian high-frequency
                // transfers in PPG operate in calibrated optical units, preventing 3.17x asymmetric
                // noise amplification and checkerboard phase distortion under tungsten illumination.
                norm_pixels.par_chunks_exact_mut(full_w).enumerate().for_each(|(y, row)| {
                    for (x, p) in row.iter_mut().enumerate() {
                        let color = frame.color_at(y, x);
                        if color == crate::raw_decoder::CFA_COLOR_RED {
                            *p *= wb_r;
                        } else if color == crate::raw_decoder::CFA_COLOR_BLUE {
                            *p *= wb_b;
                        }
                    }
                });

                let mut demosaiced_rgb = match quality {
                    DemosaicQuality::Archival => rcd_archival_demosaic(
                        &norm_pixels,
                        full_w,
                        full_h,
                        frame.active_x,
                        frame.active_y,
                        active_w,
                        active_h,
                        &frame,
                    ),
                    DemosaicQuality::Standard => soft_adaptive_demosaic(
                        &norm_pixels,
                        full_w,
                        full_h,
                        frame.active_x,
                        frame.active_y,
                        active_w,
                        active_h,
                        &frame,
                    ),
                };

                if demosaiced_rgb.len() == active_w * active_h {
                    let median_passes = match quality {
                        DemosaicQuality::Archival => 4,
                        DemosaicQuality::Standard => 2,
                    };
                    apply_chromatic_median_filter(&mut demosaiced_rgb, active_w, active_h, median_passes);

                    demosaiced.data.par_chunks_exact_mut(active_w * 3).enumerate().for_each(|(y, row_slice)| {
                        let row_start = y * active_w;
                        for x in 0..active_w {
                            let [r_wb, g_wb, b_wb] = demosaiced_rgb[row_start + x];
                            let r_raw = if wb_r > 0.001 { r_wb / wb_r } else { r_wb };
                            let g_raw = g_wb;
                            let b_raw = if wb_b > 0.001 { b_wb / wb_b } else { b_wb };
                            let (r_clean, g_clean, b_clean) =
                                firstproof_engine::blend_sensor_highlights(r_raw, g_raw, b_raw, wb_r, wb_b, frame.linear_response_limit);
                            let c_sensor = Color3::new(r_clean, g_clean, b_clean);
                            let baseline_gain = 2.0f32.powf(frame.baseline_exposure);
                            let c_rec2020 = sensor_to_rec2020.multiply_vector(c_sensor).scale(baseline_gain);

                            let out_idx = x * 3;
                            row_slice[out_idx] = c_rec2020.r().max(0.0);
                            row_slice[out_idx + 1] = c_rec2020.g().max(0.0);
                            row_slice[out_idx + 2] = c_rec2020.b().max(0.0);
                        }
                    });
                    demosaiced_with_soft_adaptive = true;
                }
            }
        } else if frame.cfa_pattern == crate::raw_decoder::CfaPattern::XTrans {
            let full_w = frame.full_width;
            let full_h = frame.full_height;
            let total_pixels = full_w * full_h;
            if total_pixels > 0 && frame.sensor_data.len() >= total_pixels {
                let mut norm_pixels: Vec<f32> = frame.sensor_data.iter().map(|&v| v as f32).collect();
                rawler::imgop::raw::correct_blacklevel_cfa(
                    &mut norm_pixels,
                    full_w,
                    full_h,
                    &frame.black_levels,
                    &frame.white_levels,
                );

                use rawler::imgop::sensor::Demosaic;
                let pixels = rawler::pixarray::PixF32::new_with(norm_pixels, full_w, full_h);
                let cfa = rawler::cfa::CFA::new("GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRG");
                let roi = rawler::imgop::Rect::new(
                    rawler::imgop::Point::new(frame.active_x, frame.active_y),
                    rawler::imgop::Dim2::new(active_w, active_h),
                );
                let colors = rawler::cfa::PlaneColor::new("RGB");
                let xtrans_demosaic = rawler::imgop::sensor::xtrans::bilinear::XTransBilinearDemosaic::new();
                let mut demosaiced_rgb = xtrans_demosaic.demosaic(&pixels, &cfa, &colors, roi);

                if demosaiced_rgb.data.len() == active_w * active_h {
                    // Apply chromatic difference median filtering to eliminate residual color fringing
                    let median_passes = match quality {
                        DemosaicQuality::Archival => 4,
                        DemosaicQuality::Standard => 2,
                    };
                    apply_chromatic_median_filter(&mut demosaiced_rgb.data, active_w, active_h, median_passes);

                    demosaiced.data.par_chunks_exact_mut(active_w * 3).enumerate().for_each(|(y, row_slice)| {
                        let row_start = y * active_w;
                        for x in 0..active_w {
                            let [r_raw, g_raw, b_raw] = demosaiced_rgb.data[row_start + x];
                            let (r_clean, g_clean, b_clean) =
                                firstproof_engine::blend_sensor_highlights(r_raw, g_raw, b_raw, wb_r, wb_b, frame.linear_response_limit);
                            let c_sensor = Color3::new(r_clean, g_clean, b_clean);
                            let baseline_gain = 2.0f32.powf(frame.baseline_exposure);
                            let c_rec2020 = sensor_to_rec2020.multiply_vector(c_sensor).scale(baseline_gain);

                            let out_idx = x * 3;
                            row_slice[out_idx] = c_rec2020.r().max(0.0);
                            row_slice[out_idx + 1] = c_rec2020.g().max(0.0);
                            row_slice[out_idx + 2] = c_rec2020.b().max(0.0);
                        }
                    });
                    demosaiced_with_soft_adaptive = true;
                }
            }
        }

        if !demosaiced_with_soft_adaptive {
            for y in 0..active_h {
                let full_y = (frame.active_y + y).min(frame.full_height - 2).max(1);
                for x in 0..active_w {
                    let full_x = (frame.active_x + x).min(frame.full_width - 2).max(1);
                    let color = frame.color_at(full_y, full_x);
                    let r: f32;
                    let g: f32;
                    let b: f32;
                    if color == crate::raw_decoder::CFA_COLOR_RED {
                        r = frame.sample_normalized(full_x, full_y);
                        g = 0.25 * (frame.sample_normalized(full_x - 1, full_y) + frame.sample_normalized(full_x + 1, full_y) + frame.sample_normalized(full_x, full_y - 1) + frame.sample_normalized(full_x, full_y + 1));
                        b = 0.25 * (frame.sample_normalized(full_x - 1, full_y - 1) + frame.sample_normalized(full_x + 1, full_y - 1) + frame.sample_normalized(full_x - 1, full_y + 1) + frame.sample_normalized(full_x + 1, full_y + 1));
                    } else if color == crate::raw_decoder::CFA_COLOR_BLUE {
                        b = frame.sample_normalized(full_x, full_y);
                        g = 0.25 * (frame.sample_normalized(full_x - 1, full_y) + frame.sample_normalized(full_x + 1, full_y) + frame.sample_normalized(full_x, full_y - 1) + frame.sample_normalized(full_x, full_y + 1));
                        r = 0.25 * (frame.sample_normalized(full_x - 1, full_y - 1) + frame.sample_normalized(full_x + 1, full_y - 1) + frame.sample_normalized(full_x - 1, full_y + 1) + frame.sample_normalized(full_x + 1, full_y + 1));
                    } else {
                        g = frame.sample_normalized(full_x, full_y);
                        if frame.color_at(full_y, full_x - 1) == crate::raw_decoder::CFA_COLOR_RED || frame.color_at(full_y, full_x + 1) == crate::raw_decoder::CFA_COLOR_RED {
                            r = 0.5 * (frame.sample_normalized(full_x - 1, full_y) + frame.sample_normalized(full_x + 1, full_y));
                            b = 0.5 * (frame.sample_normalized(full_x, full_y - 1) + frame.sample_normalized(full_x, full_y + 1));
                        } else {
                            r = 0.5 * (frame.sample_normalized(full_x, full_y - 1) + frame.sample_normalized(full_x, full_y + 1));
                            b = 0.5 * (frame.sample_normalized(full_x - 1, full_y) + frame.sample_normalized(full_x + 1, full_y));
                        }
                    }
                    let (r_wb, g_wb, b_wb) =
                        firstproof_engine::blend_sensor_highlights(r, g, b, wb_r, wb_b, frame.linear_response_limit);
                    let c_sensor = Color3::new(r_wb, g_wb, b_wb);
                    let baseline_gain = 2.0f32.powf(frame.baseline_exposure);
                    let c_rec2020 = sensor_to_rec2020.multiply_vector(c_sensor).scale(baseline_gain);
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

pub use firstproof_engine::raw::{f16_to_f32, f32_to_f16};
pub use firstproof_engine::color::demosaic::{
    apply_chromatic_median_filter, rcd_archival_demosaic, soft_adaptive_demosaic,
};

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
    
    out_rgba_f16.par_chunks_exact_mut(target_w * 8).enumerate().for_each(|(ty, row_slice)| {
        let sy = (ty as f32 + 0.5) * (orig_h as f32) / (target_h as f32);
        for tx in 0..target_w {
            let sx = (tx as f32 + 0.5) * (orig_w as f32) / (target_w as f32);
            let (r, g, b) = sample_bilinear(&demosaiced, sx, sy);
            
            let r16 = f32_to_f16(r.max(0.0));
            let g16 = f32_to_f16(g.max(0.0));
            let b16 = f32_to_f16(b.max(0.0));
            
            let out_idx = tx * 8;
            row_slice[out_idx..out_idx + 2].copy_from_slice(&r16.to_le_bytes());
            row_slice[out_idx + 2..out_idx + 4].copy_from_slice(&g16.to_le_bytes());
            row_slice[out_idx + 4..out_idx + 6].copy_from_slice(&b16.to_le_bytes());
            row_slice[out_idx + 6..out_idx + 8].copy_from_slice(&a16.to_le_bytes());
        }
    });
    
    Ok((target_w as u32, target_h as u32, out_rgba_f16))
}

/// Viewport-Aware 2×2 Bayer Box Binning RAW preview decode.
/// Decimates 2×2 Bayer quads directly to true RGB at half resolution without demosaic
/// interpolation overhead (75% fewer pixels). Eliminates intermediate 600MB+ buffer churn
/// and delivers a calibrated Linear Rec.2020 wide-gamut half-float preview in 10–15ms.
pub fn decode_raw_preview_binned_buffer<P: AsRef<Path>>(path: P) -> Result<(u32, u32, Vec<u8>), String> {
    let path = path.as_ref();
    let decoder = crate::raw_decoder::RawlerDecoder;
    if let Ok(frame) = crate::raw_decoder::RawDecoder::decode(&decoder, path) {
        let (u16_pixels, bin_w, bin_h) = frame.to_binned_linear_f16();
        if bin_w > 0 && bin_h > 0 && !u16_pixels.is_empty() {
            let mut out_rgba_f16 = vec![0u8; u16_pixels.len() * 2];
            for (i, &p) in u16_pixels.iter().enumerate() {
                let bytes = p.to_le_bytes();
                out_rgba_f16[i * 2] = bytes[0];
                out_rgba_f16[i * 2 + 1] = bytes[1];
            }
            return Ok((bin_w, bin_h, out_rgba_f16));
        }
    }

    // Fallback for non-RAW or non-Bayer formats
    decode_raw_preview_buffer(path, Some(2048))
}



