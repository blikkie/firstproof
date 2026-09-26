use std::path::Path;
use rayon::prelude::*;
use crate::color::{
    Color3, LINEAR_SRGB_TO_REC2020, srgb_gamma_to_linear,
};
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

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
        // along the Planckian blackbody locus for the photo's as-shot temperature.
        let as_shot_temp = frame.as_shot_temperature();
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

        let mut demosaiced_with_ppg = false;
        if let Some(patname) = ppg_cfa {
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

                let pixels = rawler::pixarray::PixF32::new_with(norm_pixels, full_w, full_h);
                let cfa = rawler::CFA::new(patname);
                let colors = rawler::cfa::PlaneColor::new(patname);
                let roi = rawler::imgop::Rect::new(
                    rawler::imgop::Point::new(frame.active_x, frame.active_y),
                    rawler::imgop::Dim2::new(active_w, active_h),
                );

                use rawler::imgop::sensor::Demosaic;
                let ppg = rawler::imgop::sensor::bayer::ppg::PPGDemosaic::new();
                let demosaiced_rgb = ppg.demosaic(&pixels, &cfa, &colors, roi);

                if demosaiced_rgb.width == active_w && demosaiced_rgb.height == active_h && demosaiced_rgb.data.len() == active_w * active_h {
                    demosaiced.data.par_chunks_exact_mut(active_w * 3).enumerate().for_each(|(y, row_slice)| {
                        let row_start = y * active_w;
                        for x in 0..active_w {
                            let [r_norm, g_norm, b_norm] = demosaiced_rgb.data[row_start + x];
                            let (r_wb, g_wb, b_wb) = firstproof_types::blend_sensor_highlights(r_norm, g_norm, b_norm, wb_r, wb_b);
                            let c_sensor = Color3::new(r_wb, g_wb, b_wb);
                            let baseline_gain = 2.0f32.powf(frame.baseline_exposure);
                            let c_rec2020 = sensor_to_rec2020.multiply_vector(c_sensor).scale(baseline_gain);

                            let out_idx = x * 3;
                            row_slice[out_idx] = c_rec2020.r().max(0.0);
                            row_slice[out_idx + 1] = c_rec2020.g().max(0.0);
                            row_slice[out_idx + 2] = c_rec2020.b().max(0.0);
                        }
                    });
                    demosaiced_with_ppg = true;
                }
            }
        }

        if !demosaiced_with_ppg {
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
                    let (r_wb, g_wb, b_wb) = firstproof_types::blend_sensor_highlights(r, g, b, wb_r, wb_b);
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
        let active_w = frame.active_width & !1;
        let active_h = frame.active_height & !1;
        if active_w >= 2 && active_h >= 2 && !frame.sensor_data.is_empty() {
            let binned_w = active_w / 2;
            let binned_h = active_h / 2;

            let black = frame.black_levels[0].max(0.0);
            let white = frame.white_levels[0].max(black + 1.0);
            let scale = 1.0 / (white - black);
            let (wb_r, wb_b) = frame.as_shot_wb();

            let as_shot_temp = frame.as_shot_temperature();
            let baseline_gain = 2.0f32.powf(frame.baseline_exposure);
            let m_base = frame.sensor_to_rec2020_at(as_shot_temp);
            let m = [
                [m_base[0][0] * baseline_gain, m_base[0][1] * baseline_gain, m_base[0][2] * baseline_gain],
                [m_base[1][0] * baseline_gain, m_base[1][1] * baseline_gain, m_base[1][2] * baseline_gain],
                [m_base[2][0] * baseline_gain, m_base[2][1] * baseline_gain, m_base[2][2] * baseline_gain],
            ];

            let mut out_rgba_f16 = vec![0u8; binned_w * binned_h * 8];
            let full_w = frame.full_width;
            let full_h = frame.full_height;
            let sensor_data = &frame.sensor_data;

            out_rgba_f16.par_chunks_exact_mut(binned_w * 8).enumerate().for_each(|(by, row_bytes)| {
                let row0 = (frame.active_y + by * 2).min(full_h.saturating_sub(2));
                let row1 = (row0 + 1).min(full_h.saturating_sub(1));
                let row0_start = row0 * full_w + frame.active_x;
                let row1_start = row1 * full_w + frame.active_x;
                let row_len = binned_w * 2;

                let mut bx = 0;

                if row0_start + row_len <= sensor_data.len() && row1_start + row_len <= sensor_data.len() {
                    let r0 = &sensor_data[row0_start..row0_start + row_len];
                    let r1 = &sensor_data[row1_start..row1_start + row_len];

                    #[cfg(target_arch = "aarch64")]
                    unsafe {
                        let zero = vdupq_n_f32(0.0);
                        let one = vdupq_n_f32(1.0);
                        let black_v = vdupq_n_f32(black);
                        let scale_v = vdupq_n_f32(scale);
                        let wb_r_v = vdupq_n_f32(wb_r);
                        let wb_b_v = vdupq_n_f32(wb_b);

                        while bx + 4 <= binned_w {
                            let mut r_arr = [0f32; 4];
                            let mut g_arr = [0f32; 4];
                            let mut b_arr = [0f32; 4];

                            match frame.cfa_pattern {
                                crate::raw_decoder::CfaPattern::Rggb => {
                                    for i in 0..4 {
                                        let off = (bx + i) * 2;
                                        r_arr[i] = r0[off] as f32;
                                        g_arr[i] = (r0[off + 1] as f32 + r1[off] as f32) * 0.5;
                                        b_arr[i] = r1[off + 1] as f32;
                                    }
                                }
                                crate::raw_decoder::CfaPattern::Bggr => {
                                    for i in 0..4 {
                                        let off = (bx + i) * 2;
                                        b_arr[i] = r0[off] as f32;
                                        g_arr[i] = (r0[off + 1] as f32 + r1[off] as f32) * 0.5;
                                        r_arr[i] = r1[off + 1] as f32;
                                    }
                                }
                                crate::raw_decoder::CfaPattern::Gbrg => {
                                    for i in 0..4 {
                                        let off = (bx + i) * 2;
                                        g_arr[i] = (r0[off] as f32 + r1[off + 1] as f32) * 0.5;
                                        b_arr[i] = r0[off + 1] as f32;
                                        r_arr[i] = r1[off] as f32;
                                    }
                                }
                                crate::raw_decoder::CfaPattern::Grbg => {
                                    for i in 0..4 {
                                        let off = (bx + i) * 2;
                                        g_arr[i] = (r0[off] as f32 + r1[off + 1] as f32) * 0.5;
                                        r_arr[i] = r0[off + 1] as f32;
                                        b_arr[i] = r1[off] as f32;
                                    }
                                }
                                _ => {
                                    for i in 0..4 {
                                        let off = (bx + i) * 2;
                                        r_arr[i] = r0[off] as f32;
                                        g_arr[i] = (r0[off + 1] as f32 + r1[off] as f32) * 0.5;
                                        b_arr[i] = r1[off + 1] as f32;
                                    }
                                }
                            }

                            // Load 4 binned pixels into NEON registers
                            let r_v = vld1q_f32(r_arr.as_ptr());
                            let g_v = vld1q_f32(g_arr.as_ptr());
                            let b_v = vld1q_f32(b_arr.as_ptr());

                            // Normalize: clamp((raw - black) * scale, 0.0, 1.0)
                            let r_norm = vminq_f32(vmaxq_f32(vmulq_f32(vsubq_f32(r_v, black_v), scale_v), zero), one);
                            let g_norm = vminq_f32(vmaxq_f32(vmulq_f32(vsubq_f32(g_v, black_v), scale_v), zero), one);
                            let b_norm = vminq_f32(vmaxq_f32(vmulq_f32(vsubq_f32(b_v, black_v), scale_v), zero), one);

                            // White balance multipliers
                            let r_wb = vmulq_f32(r_norm, wb_r_v);
                            let g_wb = g_norm;
                            let b_wb = vmulq_f32(b_norm, wb_b_v);

                            // NEON 128-bit SIMD matrix transformation (fmla.4s)
                            let r_rec = vmaxq_f32(
                                vmlaq_n_f32(vmlaq_n_f32(vmulq_n_f32(r_wb, m[0][0]), g_wb, m[0][1]), b_wb, m[0][2]),
                                zero,
                            );
                            let g_rec = vmaxq_f32(
                                vmlaq_n_f32(vmlaq_n_f32(vmulq_n_f32(r_wb, m[1][0]), g_wb, m[1][1]), b_wb, m[1][2]),
                                zero,
                            );
                            let b_rec = vmaxq_f32(
                                vmlaq_n_f32(vmlaq_n_f32(vmulq_n_f32(r_wb, m[2][0]), g_wb, m[2][1]), b_wb, m[2][2]),
                                zero,
                            );

                            // Convert 4 floats to half-float (f16) in single instruction
                            let r_f16 = vcvt_f16_f32(r_rec);
                            let g_f16 = vcvt_f16_f32(g_rec);
                            let b_f16 = vcvt_f16_f32(b_rec);
                            let a_f16 = vcvt_f16_f32(one);

                            // Interleave into RGBA RGBA RGBA RGBA in single instruction
                            let rgba = uint16x4x4_t(
                                vreinterpret_u16_f16(r_f16),
                                vreinterpret_u16_f16(g_f16),
                                vreinterpret_u16_f16(b_f16),
                                vreinterpret_u16_f16(a_f16),
                            );
                            let out_ptr = row_bytes.as_mut_ptr().add(bx * 8) as *mut u16;
                            vst4_u16(out_ptr, rgba);

                            bx += 4;
                        }
                    }

                    // Scalar remainder or portable fallback (zero branches inside loop)
                    match frame.cfa_pattern {
                        crate::raw_decoder::CfaPattern::Rggb => {
                            while bx < binned_w {
                                let off = bx * 2;
                                let r_raw = r0[off] as f32;
                                let g_raw = (r0[off + 1] as f32 + r1[off] as f32) * 0.5;
                                let b_raw = r1[off + 1] as f32;

                                let r_norm = ((r_raw - black) * scale).clamp(0.0, 1.0);
                                let g_norm = ((g_raw - black) * scale).clamp(0.0, 1.0);
                                let b_norm = ((b_raw - black) * scale).clamp(0.0, 1.0);

                                let r_wb = r_norm * wb_r;
                                let g_wb = g_norm;
                                let b_wb = b_norm * wb_b;

                                let r_rec = (m[0][0] * r_wb + m[0][1] * g_wb + m[0][2] * b_wb).max(0.0);
                                let g_rec = (m[1][0] * r_wb + m[1][1] * g_wb + m[1][2] * b_wb).max(0.0);
                                let b_rec = (m[2][0] * r_wb + m[2][1] * g_wb + m[2][2] * b_wb).max(0.0);

                                let r16 = f32_to_f16(r_rec);
                                let g16 = f32_to_f16(g_rec);
                                let b16 = f32_to_f16(b_rec);
                                let a16 = 0x3c00u16;

                                let out_idx = bx * 8;
                                row_bytes[out_idx..out_idx + 2].copy_from_slice(&r16.to_le_bytes());
                                row_bytes[out_idx + 2..out_idx + 4].copy_from_slice(&g16.to_le_bytes());
                                row_bytes[out_idx + 4..out_idx + 6].copy_from_slice(&b16.to_le_bytes());
                                row_bytes[out_idx + 6..out_idx + 8].copy_from_slice(&a16.to_le_bytes());

                                bx += 1;
                            }
                        }
                        crate::raw_decoder::CfaPattern::Bggr => {
                            while bx < binned_w {
                                let off = bx * 2;
                                let b_raw = r0[off] as f32;
                                let g_raw = (r0[off + 1] as f32 + r1[off] as f32) * 0.5;
                                let r_raw = r1[off + 1] as f32;

                                let r_norm = ((r_raw - black) * scale).clamp(0.0, 1.0);
                                let g_norm = ((g_raw - black) * scale).clamp(0.0, 1.0);
                                let b_norm = ((b_raw - black) * scale).clamp(0.0, 1.0);

                                let r_wb = r_norm * wb_r;
                                let g_wb = g_norm;
                                let b_wb = b_norm * wb_b;

                                let r_rec = (m[0][0] * r_wb + m[0][1] * g_wb + m[0][2] * b_wb).max(0.0);
                                let g_rec = (m[1][0] * r_wb + m[1][1] * g_wb + m[1][2] * b_wb).max(0.0);
                                let b_rec = (m[2][0] * r_wb + m[2][1] * g_wb + m[2][2] * b_wb).max(0.0);

                                let r16 = f32_to_f16(r_rec);
                                let g16 = f32_to_f16(g_rec);
                                let b16 = f32_to_f16(b_rec);
                                let a16 = 0x3c00u16;

                                let out_idx = bx * 8;
                                row_bytes[out_idx..out_idx + 2].copy_from_slice(&r16.to_le_bytes());
                                row_bytes[out_idx + 2..out_idx + 4].copy_from_slice(&g16.to_le_bytes());
                                row_bytes[out_idx + 4..out_idx + 6].copy_from_slice(&b16.to_le_bytes());
                                row_bytes[out_idx + 6..out_idx + 8].copy_from_slice(&a16.to_le_bytes());

                                bx += 1;
                            }
                        }
                        crate::raw_decoder::CfaPattern::Gbrg => {
                            while bx < binned_w {
                                let off = bx * 2;
                                let g_raw = (r0[off] as f32 + r1[off + 1] as f32) * 0.5;
                                let b_raw = r0[off + 1] as f32;
                                let r_raw = r1[off] as f32;

                                let r_norm = ((r_raw - black) * scale).clamp(0.0, 1.0);
                                let g_norm = ((g_raw - black) * scale).clamp(0.0, 1.0);
                                let b_norm = ((b_raw - black) * scale).clamp(0.0, 1.0);

                                let r_wb = r_norm * wb_r;
                                let g_wb = g_norm;
                                let b_wb = b_norm * wb_b;

                                let r_rec = (m[0][0] * r_wb + m[0][1] * g_wb + m[0][2] * b_wb).max(0.0);
                                let g_rec = (m[1][0] * r_wb + m[1][1] * g_wb + m[1][2] * b_wb).max(0.0);
                                let b_rec = (m[2][0] * r_wb + m[2][1] * g_wb + m[2][2] * b_wb).max(0.0);

                                let r16 = f32_to_f16(r_rec);
                                let g16 = f32_to_f16(g_rec);
                                let b16 = f32_to_f16(b_rec);
                                let a16 = 0x3c00u16;

                                let out_idx = bx * 8;
                                row_bytes[out_idx..out_idx + 2].copy_from_slice(&r16.to_le_bytes());
                                row_bytes[out_idx + 2..out_idx + 4].copy_from_slice(&g16.to_le_bytes());
                                row_bytes[out_idx + 4..out_idx + 6].copy_from_slice(&b16.to_le_bytes());
                                row_bytes[out_idx + 6..out_idx + 8].copy_from_slice(&a16.to_le_bytes());

                                bx += 1;
                            }
                        }
                        crate::raw_decoder::CfaPattern::Grbg => {
                            while bx < binned_w {
                                let off = bx * 2;
                                let g_raw = (r0[off] as f32 + r1[off + 1] as f32) * 0.5;
                                let r_raw = r0[off + 1] as f32;
                                let b_raw = r1[off] as f32;

                                let r_norm = ((r_raw - black) * scale).clamp(0.0, 1.0);
                                let g_norm = ((g_raw - black) * scale).clamp(0.0, 1.0);
                                let b_norm = ((b_raw - black) * scale).clamp(0.0, 1.0);

                                let r_wb = r_norm * wb_r;
                                let g_wb = g_norm;
                                let b_wb = b_norm * wb_b;

                                let r_rec = (m[0][0] * r_wb + m[0][1] * g_wb + m[0][2] * b_wb).max(0.0);
                                let g_rec = (m[1][0] * r_wb + m[1][1] * g_wb + m[1][2] * b_wb).max(0.0);
                                let b_rec = (m[2][0] * r_wb + m[2][1] * g_wb + m[2][2] * b_wb).max(0.0);

                                let r16 = f32_to_f16(r_rec);
                                let g16 = f32_to_f16(g_rec);
                                let b16 = f32_to_f16(b_rec);
                                let a16 = 0x3c00u16;

                                let out_idx = bx * 8;
                                row_bytes[out_idx..out_idx + 2].copy_from_slice(&r16.to_le_bytes());
                                row_bytes[out_idx + 2..out_idx + 4].copy_from_slice(&g16.to_le_bytes());
                                row_bytes[out_idx + 4..out_idx + 6].copy_from_slice(&b16.to_le_bytes());
                                row_bytes[out_idx + 6..out_idx + 8].copy_from_slice(&a16.to_le_bytes());

                                bx += 1;
                            }
                        }
                        _ => {
                            while bx < binned_w {
                                let off = bx * 2;
                                let r_raw = r0[off] as f32;
                                let g_raw = (r0[off + 1] as f32 + r1[off] as f32) * 0.5;
                                let b_raw = r1[off + 1] as f32;

                                let r_norm = ((r_raw - black) * scale).clamp(0.0, 1.0);
                                let g_norm = ((g_raw - black) * scale).clamp(0.0, 1.0);
                                let b_norm = ((b_raw - black) * scale).clamp(0.0, 1.0);

                                let r_wb = (r_norm * wb_r).clamp(0.0, 1.0);
                                let g_wb = g_norm.clamp(0.0, 1.0);
                                let b_wb = (b_norm * wb_b).clamp(0.0, 1.0);

                                let r_rec = (m[0][0] * r_wb + m[0][1] * g_wb + m[0][2] * b_wb).max(0.0);
                                let g_rec = (m[1][0] * r_wb + m[1][1] * g_wb + m[1][2] * b_wb).max(0.0);
                                let b_rec = (m[2][0] * r_wb + m[2][1] * g_wb + m[2][2] * b_wb).max(0.0);

                                let r16 = f32_to_f16(r_rec);
                                let g16 = f32_to_f16(g_rec);
                                let b16 = f32_to_f16(b_rec);
                                let a16 = 0x3c00u16;

                                let out_idx = bx * 8;
                                row_bytes[out_idx..out_idx + 2].copy_from_slice(&r16.to_le_bytes());
                                row_bytes[out_idx + 2..out_idx + 4].copy_from_slice(&g16.to_le_bytes());
                                row_bytes[out_idx + 4..out_idx + 6].copy_from_slice(&b16.to_le_bytes());
                                row_bytes[out_idx + 6..out_idx + 8].copy_from_slice(&a16.to_le_bytes());

                                bx += 1;
                            }
                        }
                    }
                } else {
                    let row0_idx = row0 * full_w;
                    let row1_idx = row1 * full_w;

                    while bx < binned_w {
                        let col0 = (frame.active_x + bx * 2).min(full_w.saturating_sub(2));
                        let col1 = (col0 + 1).min(full_w.saturating_sub(1));

                        let s00 = sensor_data[row0_idx + col0] as f32;
                        let s01 = sensor_data[row0_idx + col1] as f32;
                        let s10 = sensor_data[row1_idx + col0] as f32;
                        let s11 = sensor_data[row1_idx + col1] as f32;

                        let (r_raw, g_raw, b_raw) = match frame.cfa_pattern {
                            crate::raw_decoder::CfaPattern::Rggb => (s00, (s01 + s10) * 0.5, s11),
                            crate::raw_decoder::CfaPattern::Bggr => (s11, (s01 + s10) * 0.5, s00),
                            crate::raw_decoder::CfaPattern::Gbrg => (s10, (s00 + s11) * 0.5, s01),
                            crate::raw_decoder::CfaPattern::Grbg => (s01, (s00 + s11) * 0.5, s10),
                            _ => (s00, (s01 + s10) * 0.5, s11),
                        };

                        let r_norm = ((r_raw - black) * scale).clamp(0.0, 1.0);
                        let g_norm = ((g_raw - black) * scale).clamp(0.0, 1.0);
                        let b_norm = ((b_raw - black) * scale).clamp(0.0, 1.0);

                        let r_wb = (r_norm * wb_r).clamp(0.0, 1.0);
                        let g_wb = g_norm.clamp(0.0, 1.0);
                        let b_wb = (b_norm * wb_b).clamp(0.0, 1.0);

                        let r_rec = (m[0][0] * r_wb + m[0][1] * g_wb + m[0][2] * b_wb).max(0.0);
                        let g_rec = (m[1][0] * r_wb + m[1][1] * g_wb + m[1][2] * b_wb).max(0.0);
                        let b_rec = (m[2][0] * r_wb + m[2][1] * g_wb + m[2][2] * b_wb).max(0.0);

                        let r16 = f32_to_f16(r_rec);
                        let g16 = f32_to_f16(g_rec);
                        let b16 = f32_to_f16(b_rec);
                        let a16 = 0x3c00u16;

                        let out_idx = bx * 8;
                        row_bytes[out_idx..out_idx + 2].copy_from_slice(&r16.to_le_bytes());
                        row_bytes[out_idx + 2..out_idx + 4].copy_from_slice(&g16.to_le_bytes());
                        row_bytes[out_idx + 4..out_idx + 6].copy_from_slice(&b16.to_le_bytes());
                        row_bytes[out_idx + 6..out_idx + 8].copy_from_slice(&a16.to_le_bytes());

                        bx += 1;
                    }
                }
            });

            return Ok((binned_w as u32, binned_h as u32, out_rgba_f16));
        }
    }

    // Fallback for non-RAW or non-Bayer formats
    decode_raw_preview_buffer(path, Some(2048))
}


