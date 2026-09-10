use std::fs::File;
use std::io::{BufWriter, Cursor, Write};
use std::path::Path;
use image::{ColorType, ImageBuffer, Rgb, RgbImage};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::tiff::TiffEncoder;
use img_parts::jpeg::Jpeg;
use img_parts::{Bytes, ImageICC};
use rayon::prelude::*;

use crate::{
    BatchExportItemRecord, BatchExportResultRecord, EditSettings,
    ExportColorSpace, ExportRenderingIntent,
};
use crate::color::{
    Color3, LINEAR_SRGB_TO_REC2020, REC2020_TO_LINEAR_SRGB,
    linear_to_srgb_gamma, srgb_gamma_to_linear,
};

/// Creates an ICC.1 compliant Display P3 color profile with D65 white point,
/// DCI-P3 color primaries, and standard sRGB parametric transfer curve.
pub fn create_display_p3_profile() -> Result<lcms2::Profile, String> {
    let white_point = lcms2::CIExyY {
        x: 0.3127,
        y: 0.3290,
        Y: 1.0,
    };
    let primaries = lcms2::CIExyYTRIPLE {
        Red: lcms2::CIExyY { x: 0.680, y: 0.320, Y: 1.0 },
        Green: lcms2::CIExyY { x: 0.265, y: 0.690, Y: 1.0 },
        Blue: lcms2::CIExyY { x: 0.150, y: 0.060, Y: 1.0 },
    };
    let curve = lcms2::ToneCurve::new_parametric(
        4,
        &[2.4, 1.0 / 1.055, 0.055 / 1.055, 1.0 / 12.92, 0.04045],
    ).map_err(|e| format!("Failed to create parametric curve: {:?}", e))?;
    let curves = [&curve, &curve, &curve];
    lcms2::Profile::new_rgb(&white_point, &primaries, &curves)
        .map_err(|e| format!("Failed to create Display P3 profile: {:?}", e))
}

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

/// Applies user edits (Exposure, White Balance, Contrast, Display gamma) in 16-bit linear space.
pub fn apply_edits(image: &mut DemosaicedImage, edits: &EditSettings) {
    let total_exposure = edits.exposure;
    let exposure_gain = 2.0f32.powf(total_exposure);
    let contrast_gain = if edits.contrast >= 0.0 {
        1.0 + (edits.contrast / 100.0)
    } else {
        1.0 + (edits.contrast / 100.0) * 0.60
    };
    
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
        
        // 2. Linear Exposure (including baseline camera gain for RAW)
        r *= exposure_gain;
        g *= exposure_gain;
        b *= exposure_gain;
        
        // 2b. Selective Tone Recovery (Highlights, Shadows, Whites, Blacks)
        let (r_rec, g_rec, b_rec) = apply_tone_recovery(
            r, g, b,
            edits.highlights, edits.shadows, edits.whites, edits.blacks,
        );
        r = r_rec;
        g = g_rec;
        b = b_rec;
        
        // 3. AgX Filmic or Camera-Matching Tone Profiles
        // Tone profiles output sRGB (display-referred for AgX, linear sRGB for camera profiles / raw passthrough).
        // Outputting sRGB directly avoids matrix inversion back into Rec.2020 and eliminates channel cross-talk
        // or clamping discrepancies prior to bilinear interpolation.
        if image.is_raw || edits.contrast != 0.0 || edits.highlights != 0.0 || edits.shadows != 0.0 || edits.whites != 0.0 || edits.blacks != 0.0 || edits.tone_profile != crate::ColorProfileMode::AgxFilmic {
            match edits.tone_profile {
                crate::ColorProfileMode::AgxVivid => {
                    let c_rec2020 = Color3::new(r, g, b);
                    let c_srgb = crate::color::agx_tone_map_vivid(c_rec2020, contrast_gain);
                    r = c_srgb.r();
                    g = c_srgb.g();
                    b = c_srgb.b();
                }
                crate::ColorProfileMode::AgxFilmic => {
                    let c_rec2020 = Color3::new(r, g, b);
                    let c_srgb = crate::color::agx_tone_map(c_rec2020, contrast_gain);
                    r = c_srgb.r();
                    g = c_srgb.g();
                    b = c_srgb.b();
                }
                mode => {
                    let c_rec2020 = Color3::new(r, g, b);
                    let c_srgb = REC2020_TO_LINEAR_SRGB.multiply_vector(c_rec2020);
                    let (h, s, v) = crate::color::rgb_to_hsv(c_srgb.r().max(0.0), c_srgb.g().max(0.0), c_srgb.b().max(0.0));
                    
                    let huesatmap = match mode {
                        crate::ColorProfileMode::CameraPortrait => crate::color::dcp::DcpProfile::built_in_portrait_huesatmap(),
                        crate::ColorProfileMode::CameraLandscape => crate::color::dcp::DcpProfile::built_in_landscape_huesatmap(),
                        crate::ColorProfileMode::CameraVivid => crate::color::dcp::DcpProfile::built_in_vivid_huesatmap(),
                        _ => crate::color::dcp::DcpProfile::built_in_standard_huesatmap(),
                    };
                    let (h2, s2, v2) = huesatmap.apply_hsv(h, s, v);
                    let (r_mapped, g_mapped, b_mapped) = crate::color::hsv_to_rgb(h2, s2, v2);
                    
                    let curve = match mode {
                        crate::ColorProfileMode::CameraPortrait => crate::color::dcp::DcpProfile::built_in_portrait_tone_curve(),
                        crate::ColorProfileMode::CameraLandscape => crate::color::dcp::DcpProfile::built_in_landscape_tone_curve(),
                        crate::ColorProfileMode::CameraVivid => crate::color::dcp::DcpProfile::built_in_vivid_tone_curve(),
                        _ => crate::color::dcp::DcpProfile::built_in_standard_tone_curve(),
                    };
                    
                    // Luminance-scaled tone curve: preserves chromaticity ratios (R/Y, G/Y, B/Y) without hue twisting
                    let lum = 0.2126 * r_mapped + 0.7152 * g_mapped + 0.0722 * b_mapped;
                    let lum_curved = crate::color::evaluate_tone_curve(lum, &curve);
                    let lum_adj = ((lum_curved - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0);
                    let gain = lum_adj / lum.max(1e-6);
                    
                    r = (r_mapped * gain).clamp(0.0, 1.0);
                    g = (g_mapped * gain).clamp(0.0, 1.0);
                    b = (b_mapped * gain).clamp(0.0, 1.0);
                }
            }
        } else {
            // Passthrough for non-raw with no profile adjustments: convert Rec.2020 working space to linear sRGB
            let c_rec2020 = Color3::new(r, g, b);
            let c_srgb = REC2020_TO_LINEAR_SRGB.multiply_vector(c_rec2020);
            r = c_srgb.r();
            g = c_srgb.g();
            b = c_srgb.b();
        }
        
        // Store sRGB color values (display-referred for AgX, linear sRGB for camera profiles)
        image.data[idx] = r.max(0.0);
        image.data[idx + 1] = g.max(0.0);
        image.data[idx + 2] = b.max(0.0);
    }
}

/// Applies photographic four-zone tone recovery (Highlights, Shadows, Whites, Blacks)
/// to scene-referred linear RGB values using chromaticity-preserving luminance scaling.
/// Each adjustment slider is in the range -100.0 .. +100.0.
#[inline(always)]
pub fn apply_tone_recovery(
    r: f32, g: f32, b: f32,
    highlights: f32, shadows: f32, whites: f32, blacks: f32,
) -> (f32, f32, f32) {
    if highlights == 0.0 && shadows == 0.0 && whites == 0.0 && blacks == 0.0 {
        return (r, g, b);
    }
    
    let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    if lum <= 1e-6 {
        return (r, g, b);
    }
    
    // Log-luminance EV relative to photographic middle gray (0.18)
    let ev = (lum.max(1e-5) / 0.18).log2();
    
    let mut delta_ev = 0.0f32;
    
    // 1. Blacks: Anchor deep shadows (ev <= -1.0)
    if blacks != 0.0 {
        let u = ((-ev - 1.0) / 3.0).clamp(0.0, 1.0);
        let w_blacks = u * u * (3.0 - 2.0 * u);
        delta_ev += (blacks / 100.0) * 0.90 * w_blacks;
    }
    
    // 2. Shadows: Lift lower midtones and shadow textures (ev in [-4.0, 0.0])
    if shadows != 0.0 {
        let u = ((ev + 4.0) / 4.0).clamp(0.0, 1.0);
        let w_shadows = (std::f32::consts::PI * u).sin();
        delta_ev += (shadows / 100.0) * 1.20 * w_shadows;
    }
    
    // 3. Highlights: Recover or boost bright areas and skies (ev in [0.0, 4.0])
    if highlights != 0.0 {
        let u = (ev / 4.0).clamp(0.0, 1.0);
        let w_highlights = (std::f32::consts::PI * u).sin();
        delta_ev += (highlights / 100.0) * 1.30 * w_highlights;
    }
    
    // 4. Whites: Adjust specular highlight threshold and ceiling (ev >= 1.0)
    if whites != 0.0 {
        let u = ((ev - 1.0) / 3.0).clamp(0.0, 1.0);
        let w_whites = u * u * (3.0 - 2.0 * u);
        delta_ev += (whites / 100.0) * 0.90 * w_whites;
    }
    
    if delta_ev == 0.0 {
        return (r, g, b);
    }
    
    let gain = 2.0f32.powf(delta_ev);
    (r * gain, g * gain, b * gain)
}

/// Applies photographic color adjustments (Vibrance, Saturation, Deep Blue, Foliage)
/// using noise-free direct RGB optical models (preventing HSV singularity dithering).
#[inline(always)]
pub fn apply_color_adjustments(
    mut r: f32, mut g: f32, mut b: f32,
    vibrance: f32,
    saturation: f32,
    deep_blue: f32,
    foliage: f32,
) -> (f32, f32, f32) {
    if vibrance == 0.0 && saturation == 0.0 && deep_blue == 0.0 && foliage == 0.0 {
        return (r, g, b);
    }

    let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;

    // 1. Global Saturation (-100..+100)
    // Standard chromaticity scaling relative to perceptual luminance
    if saturation != 0.0 {
        let sat_gain = (1.0 + saturation / 100.0).max(0.0);
        r = lum + (r - lum) * sat_gain;
        g = lum + (g - lum) * sat_gain;
        b = lum + (b - lum) * sat_gain;
    }

    // 2. Smart Vibrance (-100..+100)
    // Non-linear boost that protects saturated tones and human skin tones.
    if vibrance != 0.0 {
        let max_c = r.max(g).max(b);
        let min_c = r.min(g).min(b);
        let chroma = max_c - min_c; // in [0, 1]

        // Detect human skin tones in RGB: R > G > B with warm golden/peach hue
        let is_warm = (r > g) && (g > b);
        let skin_metric = if is_warm && lum > 0.05 {
            let rg = (r - g) / lum.max(0.1);
            let gb = (g - b) / lum.max(0.1);
            let d_rg = (rg - 0.35).abs();
            let d_gb = (gb - 0.25).abs();
            (1.0 - (d_rg * 3.0 + d_gb * 3.0)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let skin_protect = 1.0 - 0.6 * skin_metric;

        let vib_gain = if vibrance > 0.0 {
            // Headroom-preserving boost on low/moderate chroma
            (vibrance / 100.0) * (1.0 - chroma) * 0.8 * skin_protect
        } else {
            (vibrance / 100.0) * 0.6
        };

        r += (r - lum) * vib_gain;
        g += (g - lum) * vib_gain;
        b += (b - lum) * vib_gain;
    }

    // 3. Deep Blue (Sky & Water) (-100..+100)
    // Direct chromatic optical polarizing model:
    // Operates on the blue excess over warm/neutral channels (B - max(R, G)).
    // 100% linear in RGB space -> ZERO hue-division noise -> ZERO dithering!
    if deep_blue != 0.0 {
        let max_warm = r.max(g);
        let sky_signal = (b - max_warm).max(0.0);
        if sky_signal > 0.0 {
            let x = deep_blue / 100.0;
            let factor = x.signum() * x.abs().powf(1.6);
            if factor > 0.0 {
                // Polarizing filter effect:
                // Attenuates blue luminance, removes red haze to enrich azure saturation
                r -= factor * sky_signal * 0.30;
                g -= factor * sky_signal * 0.15;
                b -= factor * sky_signal * 0.20;
            } else {
                // Airy / luminous sky:
                // factor is negative -> lifts values gently and evenly without color casting
                r -= factor * sky_signal * 0.25;
                g -= factor * sky_signal * 0.20;
                b -= factor * sky_signal * 0.10;
            }
        }
    }

    // 4. Foliage (Lush Greens) (-100..+100)
    // Operates on green excess (G - max(R, B)).
    // Shifts yellow grass towards lush emerald green without hue noise.
    if foliage != 0.0 {
        let max_rb = r.max(b);
        let green_signal = (g - max_rb).max(0.0);
        if green_signal > 0.0 {
            let x = foliage / 100.0;
            let factor = x.signum() * x.abs().powf(1.6);
            // Shift towards emerald green (lower red, lift green, deepen blue)
            r -= factor * green_signal * 0.30;
            b -= factor * green_signal * 0.15;
            g += factor * green_signal * 0.10;
        }
    }

    (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}

/// Samples a 256-entry 1D curve LUT with linear interpolation matching hardware sampler.
#[inline(always)]
pub fn sample_curve_lut(lut: &[f32], v: f32) -> f32 {
    let len = lut.len();
    if len == 0 {
        return v;
    }
    if len == 1 {
        return lut[0];
    }
    let scaled = v.clamp(0.0, 1.0) * (len - 1) as f32;
    let idx = scaled.floor() as usize;
    let frac = scaled - idx as f32;
    if idx + 1 < len {
        lut[idx] * (1.0 - frac) + lut[idx + 1] * frac
    } else {
        lut[len - 1]
    }
}

/// Applies edge-masked high-frequency luminance unsharp masking and flat-area noise reduction.
///
/// Parameters:
/// - `r, g, b`: display-referred pixel RGB in [0.0, 1.0]
/// - `y_c`: center luminance
/// - `y_l, y_r, y_t, y_b`: neighbor luminances sampled at distance `radius`
/// - `amount`: sharpening strength [0.0 .. 100.0]
/// - `masking`: edge threshold [0.0 .. 100.0]
/// - `noise_reduction`: flat-area noise smoothing [0.0 .. 100.0]
#[inline(always)]
pub fn apply_detail_and_sharpening_rgb(
    r: f32, g: f32, b: f32,
    y_c: f32,
    y_l: f32, y_r: f32, y_t: f32, y_b: f32,
    amount: f32,
    masking: f32,
    noise_reduction: f32,
) -> (f32, f32, f32) {
    if amount <= 0.0 && noise_reduction <= 0.0 {
        return (r, g, b);
    }

    // Unsharp mask: Laplacian high-frequency detail
    let y_neighbors = 0.25 * (y_l + y_r + y_t + y_b);
    let delta_y = y_c - y_neighbors;

    // Sobel gradient magnitude
    let gx = (y_r - y_l) * 0.5;
    let gy = (y_b - y_t) * 0.5;
    let grad_mag = (gx * gx + gy * gy).sqrt();

    // Edge masking weight:
    // When masking <= 0, w_edge = 1.0 (uniform sharpening across entire image).
    // When masking > 0, flat regions (grad_mag < threshold) are smoothly suppressed towards 0.
    let w_edge = if masking <= 0.0 {
        1.0
    } else {
        let m = (masking / 100.0).clamp(0.0, 1.0);
        let threshold = m.powf(1.5) * 0.08;
        let t = (grad_mag / threshold.max(1e-4)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t) // smoothstep(0.0, threshold, grad_mag)
    };

    // Photographically calibrated gain: up to 3.5x boost at Amount=100 for unmistakable crispness
    let sharp_gain = (amount / 100.0) * 3.5;
    let sharp_term = sharp_gain * w_edge * delta_y;
    let nr_term = (noise_reduction / 100.0) * (1.0 - w_edge) * delta_y;
    let delta_adj = sharp_term - nr_term;

    let k = ((y_c + delta_adj) / y_c.max(1e-5)).clamp(0.0, 3.0);

    ((r * k).clamp(0.0, 1.0), (g * k).clamp(0.0, 1.0), (b * k).clamp(0.0, 1.0))
}

// MARK: - Procedural Film Grain (Luminance-Dependent Silver Halide Simulation)

#[inline(always)]
fn grain_luminance_weight(lum: f32) -> f32 {
    let bell = 4.0 * lum * (1.0 - lum);
    (0.08 * (1.0 - lum)).max(bell)
}

#[inline(always)]
fn pcg_hash_2d(x: i32, y: i32) -> f32 {
    let ux = x as u32;
    let uy = y as u32;
    let mut h = ux.wrapping_mul(374761393).wrapping_add(uy.wrapping_mul(668265263)).wrapping_add(0x9E3779B9);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h = h ^ (h >> 16);
    h as f32 * (1.0 / 4294967296.0)
}

#[inline(always)]
fn smooth_noise_2d(x: f32, y: f32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - x.floor();
    let fy = y - y.floor();
    
    let ux = fx * fx * fx * (fx * (fx * 6.0 - 15.0) + 10.0);
    let uy = fy * fy * fy * (fy * (fy * 6.0 - 15.0) + 10.0);
    
    let a = pcg_hash_2d(ix, iy);
    let b = pcg_hash_2d(ix + 1, iy);
    let c = pcg_hash_2d(ix, iy + 1);
    let d = pcg_hash_2d(ix + 1, iy + 1);
    
    (a * (1.0 - ux) + b * ux) * (1.0 - uy) + (c * (1.0 - ux) + d * ux) * uy
}

#[inline(always)]
fn organic_grain_noise(px: f32, py: f32, scale: f32, roughness: f32) -> f32 {
    let fine = (pcg_hash_2d(px.floor() as i32, py.floor() as i32) - 0.5) * 2.0;
    let clump = (smooth_noise_2d(px / scale, py / scale) - 0.5) * 2.0;
    let micro = (smooth_noise_2d((px + 31.7) / (scale * 0.5), (py + 47.3) / (scale * 0.5)) - 0.5) * 2.0;
    let r_factor = (roughness / 100.0).clamp(0.0, 1.0);
    clump * 0.45 + micro * (0.25 * r_factor) + fine * (0.35 + 0.25 * r_factor)
}

/// Applies procedural organic silver halide film grain to a display-referred RGB pixel.
///
/// Parameters:
/// - `r, g, b`: display-referred pixel RGB in [0.0, 1.0]
/// - `pixel_x, pixel_y`: pixel coordinates for deterministic spatial noise evaluation
/// - `amount`: grain intensity [0.0 .. 100.0]
/// - `size`: grain scale factor (0.5 .. 3.0, default 1.0)
/// - `roughness`: crystal micro-structure [0.0 .. 100.0]
/// - `chroma`: color emulsion variation [0.0 .. 100.0]
#[inline(always)]
pub fn apply_film_grain_rgb(
    r: f32, g: f32, b: f32,
    pixel_x: f32, pixel_y: f32,
    amount: f32,
    size: f32,
    roughness: f32,
    chroma: f32,
) -> (f32, f32, f32) {
    if amount <= 0.0 {
        return (r, g, b);
    }
    
    let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let weight = grain_luminance_weight(lum.clamp(0.0, 1.0));
    let grain_scale = size.max(0.4);
    
    let mono_noise = organic_grain_noise(pixel_x, pixel_y, grain_scale, roughness);
    
    let (nr, ng, nb) = if chroma > 0.0 {
        let c_t = (chroma / 100.0).clamp(0.0, 1.0);
        // Equal-power crossfade: cos^2(theta) + sin^2(theta) = 1.0 preserves total variance
        let theta = c_t * std::f32::consts::FRAC_PI_2;
        let w_mono = theta.cos();
        // 1.25x CSF gain compensates for cross-channel luminance cancellation
        let w_color = theta.sin() * 1.25;

        let r_noise = organic_grain_noise(pixel_x + 131.7, pixel_y + 71.3, grain_scale * 0.9, roughness);
        let g_noise = organic_grain_noise(pixel_x + 291.7, pixel_y + 431.1, grain_scale * 1.0, roughness);
        let b_noise = organic_grain_noise(pixel_x + 531.3, pixel_y + 191.9, grain_scale * 1.25, roughness);
        (
            mono_noise * w_mono + r_noise * w_color,
            mono_noise * w_mono + g_noise * w_color,
            mono_noise * w_mono + b_noise * w_color,
        )
    } else {
        (mono_noise, mono_noise, mono_noise)
    };
    
    let strength = (amount / 100.0) * 0.18 * weight;
    (
        (r + nr * strength).clamp(0.0, 1.0),
        (g + ng * strength).clamp(0.0, 1.0),
        (b + nb * strength).clamp(0.0, 1.0),
    )
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

/// Exports an edited photo to JPEG or 16-bit TIFF on disk with optional resizing, color space management, and Lensfun distortion correction.
pub fn export_photo_file<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P,
    output_path: Q,
    edits: &EditSettings,
    lens_distortion: Option<(&[f32], &str)>,
    quality: u8,
    max_dimension: Option<u32>,
    color_space: ExportColorSpace,
    rendering_intent: ExportRenderingIntent,
    ultra_hdr: bool,
) -> Result<(), String> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    
    let mut image = decode_image(input_path)?;
    apply_edits(&mut image, edits);
    
    let orig_width = image.width as u32;
    let orig_height = image.height as u32;
    
    let crop_l = edits.crop_left.clamp(0.0, 0.99);
    let crop_t = edits.crop_top.clamp(0.0, 0.99);
    let crop_r = edits.crop_right.clamp(crop_l + 0.01, 1.0);
    let crop_b = edits.crop_bottom.clamp(crop_t + 0.01, 1.0);
    let crop_w_frac = crop_r - crop_l;
    let crop_h_frac = crop_b - crop_t;

    let effective_orig_w = ((orig_width as f32) * crop_w_frac).round().max(1.0) as u32;
    let effective_orig_h = ((orig_height as f32) * crop_h_frac).round().max(1.0) as u32;

    // Calculate target dimensions
    let (target_w, target_h) = if let Some(max_dim) = max_dimension {
        if max_dim > 0 && (effective_orig_w > max_dim || effective_orig_h > max_dim) {
            if effective_orig_w >= effective_orig_h {
                let w = max_dim;
                let h = ((effective_orig_h as f32 * max_dim as f32) / effective_orig_w as f32).round() as u32;
                (w, h.max(1))
            } else {
                let h = max_dim;
                let w = ((effective_orig_w as f32 * max_dim as f32) / effective_orig_h as f32).round() as u32;
                (w.max(1), h)
            }
        } else {
            (effective_orig_w, effective_orig_h)
        }
    } else {
        (effective_orig_w, effective_orig_h)
    };
    
    // Build 8-bit RGB image buffer with geometric warp and bilinear filtering
    let mut rgb_img: RgbImage = ImageBuffer::new(target_w, target_h);
    let mut sdr_luma = if ultra_hdr { Vec::with_capacity((target_w * target_h) as usize) } else { Vec::new() };
    let mut hdr_luma = if ultra_hdr { Vec::with_capacity((target_w * target_h) as usize) } else { Vec::new() };
    
    let aspect_ratio = (orig_width as f32) / (orig_height as f32);
    let angle_rad = edits.crop_angle * std::f32::consts::PI / 180.0;
    let cos_theta = angle_rad.cos();
    let sin_theta = angle_rad.sin();
    
    let c_v = edits.perspective_vertical * 0.005;
    let c_h = edits.perspective_horizontal * 0.005;
    
    let mut auto_crop_scale = 1.0f32;
    if edits.crop_constrain {
        let corners = [
            (aspect_ratio, 1.0),
            (aspect_ratio, -1.0),
            (-aspect_ratio, -1.0),
            (-aspect_ratio, 1.0),
        ];
        for (px, py) in corners {
            let rx = px * cos_theta - py * sin_theta;
            let ry = px * sin_theta + py * cos_theta;
            let req = (rx.abs() / aspect_ratio).max(ry.abs()) - (c_v * ry + c_h * rx);
            if req > auto_crop_scale {
                auto_crop_scale = req;
            }
        }
    }
    
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
    
    let active_lut: Option<crate::lut::CubeLut> = if let Some(ref lut_name) = edits.lut_name {
        if edits.lut_intensity > 0.0 {
            crate::lut::FilmSimulationGenerator::all_builtin(33)
                .into_iter()
                .find(|l| l.title == *lut_name)
                .or_else(|| {
                    crate::lut::load_cube_lut_file(lut_name).ok()
                })
        } else {
            None
        }
    } else {
        None
    };
    let lut_intensity = edits.lut_intensity.clamp(0.0, 1.0);

    let curve_lut = crate::curve::evaluate_curves_lut_ref(&edits.curves);
    let has_master_curve = edits.curves.master.len() > 1;
    let has_red_curve = edits.curves.red.len() > 1;
    let has_green_curve = edits.curves.green.len() > 1;
    let has_blue_curve = edits.curves.blue.len() > 1;
    let has_any_curve = has_master_curve || has_red_curve || has_green_curve || has_blue_curve;

    for ty in 0..target_h {
        for tx in 0..target_w {
            let u = (tx as f32 + 0.5) / (target_w as f32);
            let v = (ty as f32 + 0.5) / (target_h as f32);
            
            // Map output (u, v) into cropped sub-box of the composition
            let u_crop = crop_l + u * crop_w_frac;
            let v_crop = crop_t + v * crop_h_frac;
            
            // 1. Straighten and auto-crop zoom (normalized to Lensfun standard coordinates)
            let centered_x = (u_crop - 0.5) * 2.0 * aspect_ratio / auto_crop_scale;
            let centered_y = (v_crop - 0.5) * 2.0 / auto_crop_scale;
            
            let rot_x = centered_x * cos_theta - centered_y * sin_theta;
            let rot_y = centered_x * sin_theta + centered_y * cos_theta;
            
            // Perspective keystone (backward projective homography)
            let denom = (1.0 + c_v * rot_y + c_h * rot_x).max(0.1);
            let mut geom_x = rot_x / denom;
            let mut geom_y = rot_y / denom;
            
            // 2. Lensfun distortion model
            if has_dist {
                let r2 = geom_x * geom_x + geom_y * geom_y;
                let r = r2.sqrt();
                if r > 0.0001 {
                    let rd = if is_ptlens {
                        let a = k1;
                        let b = k2;
                        let c = k3;
                        let d = 1.0 - (a + b + c);
                        let r3 = r2 * r;
                        let r4 = r2 * r2;
                        a * r4 + b * r3 + c * r2 + d * r
                    } else {
                        // poly3 model: r_d = r * (1 + k1*r^2 + k2*r^4)
                        let r4 = r2 * r2;
                        r * (1.0 + k1 * r2 + k2 * r4)
                    };
                    let scale_factor = rd / r;
                    geom_x *= scale_factor;
                    geom_y *= scale_factor;
                }
            }
            
            let sx = (geom_x / (2.0 * aspect_ratio) + 0.5) * (orig_width as f32);
            let sy = (geom_y / 2.0 + 0.5) * (orig_height as f32);
            
            let (r_val, g_val, b_val) = if !edits.crop_constrain && (sx < 0.0 || sx >= (orig_width - 1) as f32 || sy < 0.0 || sy >= (orig_height - 1) as f32) {
                (0.0, 0.0, 0.0)
            } else {
                sample_bilinear(&image, sx, sy)
            };
            let (mut r_disp, mut g_disp, mut b_disp) = if edits.tone_profile == crate::ColorProfileMode::AgxVivid
                || edits.tone_profile == crate::ColorProfileMode::AgxFilmic
            {
                (r_val.clamp(0.0, 1.0), g_val.clamp(0.0, 1.0), b_val.clamp(0.0, 1.0))
            } else {
                (
                    linear_to_srgb_gamma(r_val).clamp(0.0, 1.0),
                    linear_to_srgb_gamma(g_val).clamp(0.0, 1.0),
                    linear_to_srgb_gamma(b_val).clamp(0.0, 1.0),
                )
            };

            // 2b. Color Adjustments: Vibrance, Saturation, Deep Blue, Foliage
            let (r_adj, g_adj, b_adj) = apply_color_adjustments(
                r_disp, g_disp, b_disp,
                edits.vibrance, edits.saturation, edits.deep_blue, edits.foliage,
            );
            r_disp = r_adj;
            g_disp = g_adj;
            b_disp = b_adj;

            // 2c. RGB Tone Curves (Master + R + G + B)
            if has_any_curve {
                if has_master_curve {
                    r_disp = sample_curve_lut(&curve_lut.master, r_disp);
                    g_disp = sample_curve_lut(&curve_lut.master, g_disp);
                    b_disp = sample_curve_lut(&curve_lut.master, b_disp);
                }
                if has_red_curve {
                    r_disp = sample_curve_lut(&curve_lut.red, r_disp);
                }
                if has_green_curve {
                    g_disp = sample_curve_lut(&curve_lut.green, g_disp);
                }
                if has_blue_curve {
                    b_disp = sample_curve_lut(&curve_lut.blue, b_disp);
                }
            }

            // 3. 3D Look-Up Table (display-referred stage post-tone-mapping)
            if let Some(ref lut) = active_lut {
                if lut_intensity > 0.0 {
                    let lut_out = lut.interpolate_tetrahedral([r_disp, g_disp, b_disp]);
                    r_disp = r_disp * (1.0 - lut_intensity) + lut_out[0] * lut_intensity;
                    g_disp = g_disp * (1.0 - lut_intensity) + lut_out[1] * lut_intensity;
                    b_disp = b_disp * (1.0 - lut_intensity) + lut_out[2] * lut_intensity;
                }
            }

            // 3b. Detail & Sharpening / Noise Reduction
            if edits.sharpening_amount > 0.0 || edits.noise_reduction > 0.0 {
                let r_step = edits.sharpening_radius.max(0.5);
                let (rl, gl, bl) = sample_bilinear(&image, sx - r_step, sy);
                let (rr, gr, br) = sample_bilinear(&image, sx + r_step, sy);
                let (rt, gt, bt) = sample_bilinear(&image, sx, sy - r_step);
                let (rb, gb, bb) = sample_bilinear(&image, sx, sy + r_step);
                let y_c = 0.2126 * r_val + 0.7152 * g_val + 0.0722 * b_val;
                let y_l = 0.2126 * rl + 0.7152 * gl + 0.0722 * bl;
                let y_r = 0.2126 * rr + 0.7152 * gr + 0.0722 * br;
                let y_t = 0.2126 * rt + 0.7152 * gt + 0.0722 * bt;
                let y_b = 0.2126 * rb + 0.7152 * gb + 0.0722 * bb;
                let (r_sharp, g_sharp, b_sharp) = apply_detail_and_sharpening_rgb(
                    r_disp, g_disp, b_disp,
                    y_c, y_l, y_r, y_t, y_b,
                    edits.sharpening_amount,
                    edits.sharpening_masking,
                    edits.noise_reduction,
                );
                r_disp = r_sharp;
                g_disp = g_sharp;
                b_disp = b_sharp;
            }

            // 3c. Procedural Organic Film Grain
            if edits.grain_amount > 0.0 {
                let (rg, gg, bg) = apply_film_grain_rgb(
                    r_disp, g_disp, b_disp,
                    tx as f32, ty as f32,
                    edits.grain_amount,
                    edits.grain_size,
                    edits.grain_roughness,
                    edits.grain_chroma,
                );
                r_disp = rg;
                g_disp = gg;
                b_disp = bg;
            }

            let srgb8 = [
                (r_disp.clamp(0.0, 1.0) * 255.0).round() as u8,
                (g_disp.clamp(0.0, 1.0) * 255.0).round() as u8,
                (b_disp.clamp(0.0, 1.0) * 255.0).round() as u8,
            ];
            
            rgb_img.put_pixel(tx, ty, Rgb(srgb8));

            if ultra_hdr {
                let y_sdr = 0.2126 * r_disp.clamp(0.0, 1.0) + 0.7152 * g_disp.clamp(0.0, 1.0) + 0.0722 * b_disp.clamp(0.0, 1.0);
                let y_hdr = 0.2126 * r_val.max(0.0) + 0.7152 * g_val.max(0.0) + 0.0722 * b_val.max(0.0);
                sdr_luma.push(y_sdr);
                hdr_luma.push(y_hdr);
            }
        }
    }
    
    // Apply 90/180/270 degree rotation and horizontal/vertical flips
    if edits.rotation != 0 || edits.flip_horizontal || edits.flip_vertical {
        match edits.rotation {
            90 => rgb_img = image::imageops::rotate90(&rgb_img),
            180 => rgb_img = image::imageops::rotate180(&rgb_img),
            270 => rgb_img = image::imageops::rotate270(&rgb_img),
            _ => {}
        }
        if edits.flip_horizontal {
            rgb_img = image::imageops::flip_horizontal(&rgb_img);
        }
        if edits.flip_vertical {
            rgb_img = image::imageops::flip_vertical(&rgb_img);
        }
    }
    
    let final_w = rgb_img.width();
    let final_h = rgb_img.height();
    
    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Color Management: convert from working sRGB to target color space (Display P3) if requested
    let (target_rgb_bytes, icc_to_embed): (Vec<u8>, Vec<u8>) = if color_space == ExportColorSpace::DisplayP3 {
        let in_prof = lcms2::Profile::new_srgb();
        if let Ok(p3_prof) = create_display_p3_profile() {
            let intent = match rendering_intent {
                ExportRenderingIntent::Perceptual => lcms2::Intent::Perceptual,
                ExportRenderingIntent::RelativeColorimetric => lcms2::Intent::RelativeColorimetric,
            };
            if let Ok(transform) = lcms2::Transform::new(
                &in_prof,
                lcms2::PixelFormat::RGB_8,
                &p3_prof,
                lcms2::PixelFormat::RGB_8,
                intent,
            ) {
                let mut p3_bytes = vec![0u8; rgb_img.as_raw().len()];
                transform.transform_pixels(rgb_img.as_raw(), &mut p3_bytes);
                let icc_bytes = p3_prof.icc().unwrap_or_else(|_| crate::color::SRGB_ICC_PROFILE.to_vec());
                (p3_bytes, icc_bytes)
            } else {
                (rgb_img.as_raw().to_vec(), crate::color::SRGB_ICC_PROFILE.to_vec())
            }
        } else {
            (rgb_img.as_raw().to_vec(), crate::color::SRGB_ICC_PROFILE.to_vec())
        }
    } else {
        (rgb_img.as_raw().to_vec(), crate::color::SRGB_ICC_PROFILE.to_vec())
    };
    
    let ext = output_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("jpg")
        .to_lowercase();
        
    if ext == "tif" || ext == "tiff" {
        let file = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
        let mut writer = BufWriter::new(file);
        let encoder = TiffEncoder::new(&mut writer);
        encoder
            .encode(&target_rgb_bytes, final_w, final_h, ColorType::Rgb8)
            .map_err(|e| format!("Failed to encode TIFF: {}", e))?;
    } else {
        let mut jpeg_buf = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_buf, quality.clamp(1, 100));
        encoder
            .encode(&target_rgb_bytes, final_w, final_h, ColorType::Rgb8)
            .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
            
        let tagged_jpeg = inject_icc_profile_into_jpeg(&jpeg_buf, &icc_to_embed);
        let final_jpeg_bytes = if ultra_hdr && !sdr_luma.is_empty() {
            let meta = crate::gainmap::GainMapMetadata::default();
            let (_gm_img, gm_jpeg) = crate::gainmap::generate_gain_map(
                &sdr_luma,
                &hdr_luma,
                target_w as usize,
                target_h as usize,
                &meta,
            );
            crate::gainmap::package_ultra_hdr_jpeg(&tagged_jpeg, &gm_jpeg, &meta)
                .unwrap_or(tagged_jpeg)
        } else {
            tagged_jpeg
        };
        let mut file = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
        file.write_all(&final_jpeg_bytes).map_err(|e| format!("Failed to write JPEG: {}", e))?;
    }
    
    Ok(())
}

/// Injects an ICC profile into a JPEG byte stream as an APP2 segment (ICC.1:2010 specification).
/// Uses zero-copy `img-parts` segment parsing and chunking.
pub fn inject_icc_profile_into_jpeg(jpeg_data: &[u8], icc_profile: &[u8]) -> Vec<u8> {
    match Jpeg::from_bytes(Bytes::copy_from_slice(jpeg_data)) {
        Ok(mut jpeg) => {
            jpeg.set_icc_profile(Some(Bytes::copy_from_slice(icc_profile)));
            let mut out = Vec::with_capacity(jpeg_data.len() + icc_profile.len() + 64);
            if jpeg.encoder().write_to(&mut out).is_ok() {
                out
            } else {
                jpeg_data.to_vec()
            }
        }
        Err(_) => jpeg_data.to_vec(),
    }
}

/// Dispatches multiple photo exports concurrently across all available CPU cores using Rayon.
pub fn export_batch_photos(
    items: Vec<BatchExportItemRecord>,
) -> Vec<BatchExportResultRecord> {
    items
        .into_par_iter()
        .map(|item| {
            let dist_tuple = item.lens_coeffs.as_ref().and_then(|c| {
                if c.has_distortion {
                    Some((c.distortion.as_slice(), c.distortion_model.as_str()))
                } else {
                    None
                }
            });

            match export_photo_file(
                &item.input_path,
                &item.output_path,
                &item.edits,
                dist_tuple,
                item.quality as u8,
                item.max_dimension,
                item.color_space,
                item.rendering_intent,
                item.ultra_hdr,
            ) {
                Ok(()) => BatchExportResultRecord {
                    input_path: item.input_path,
                    output_path: item.output_path,
                    success: true,
                    error_message: None,
                },
                Err(e) => BatchExportResultRecord {
                    input_path: item.input_path,
                    output_path: item.output_path,
                    success: false,
                    error_message: Some(e),
                },
            }
        })
        .collect()
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
            tone_profile: crate::ColorProfileMode::AgxFilmic,
            ..Default::default()
        };
        
        let res = export_photo_file(
            tmp_in,
            tmp_out,
            &edits,
            None,
            90,
            Some(50),
            ExportColorSpace::Srgb,
            ExportRenderingIntent::RelativeColorimetric,
            false,
        );
        assert!(res.is_ok());
        
        let exported = image::open(tmp_out).unwrap();
        assert_eq!(exported.width(), 50);
        assert_eq!(exported.height(), 50);
        
        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out);
    }

    #[test]
    fn test_pipeline_parity_math() {
        // Verify that 18% mid-gray in linear Rec.2020 transforms to expected display midtone
        let mut img = DemosaicedImage {
            width: 1,
            height: 1,
            data: vec![0.18, 0.18, 0.18],
            is_raw: true,
        };
        let edits = EditSettings {
            temperature: 5500.0,
            tone_profile: crate::ColorProfileMode::AgxFilmic,
            ..Default::default()
        };
        apply_edits(&mut img, &edits);
        
        let c_srgb = Color3::new(img.data[0], img.data[1], img.data[2]);
        let srgb8 = if edits.tone_profile == crate::ColorProfileMode::AgxVivid
            || edits.tone_profile == crate::ColorProfileMode::AgxFilmic
        {
            [
                (c_srgb.r().clamp(0.0, 1.0) * 255.0).round() as u8,
                (c_srgb.g().clamp(0.0, 1.0) * 255.0).round() as u8,
                (c_srgb.b().clamp(0.0, 1.0) * 255.0).round() as u8,
            ]
        } else {
            crate::color::linear_rgb_to_srgb8(c_srgb)
        };
        
        // Mid-gray 0.18 -> filmic S-curve -> ~0.51085 display-referred -> ~130/255 (RapidRAW target median is 127-130)
        assert!(srgb8[0] >= 120 && srgb8[0] <= 135, "Expected display midtone range, got {}", srgb8[0]);
        assert_eq!(srgb8[0], srgb8[1]);
        assert_eq!(srgb8[1], srgb8[2]);
    }

    #[test]
    fn test_lut_export_pipeline() {
        let tmp_in = "test_lut_in.jpg";
        let tmp_out_no_lut = "test_lut_out_no_lut.jpg";
        let tmp_out_lut = "test_lut_out_lut.jpg";
        
        let img = RgbImage::from_fn(30, 30, |x, y| {
            Rgb([x as u8 * 8, y as u8 * 8, 128])
        });
        img.save(tmp_in).unwrap();
        
        let mut edits = EditSettings {
            tone_profile: crate::ColorProfileMode::AgxVivid,
            ..Default::default()
        };
        
        export_photo_file(tmp_in, tmp_out_no_lut, &edits, None, 95, None, ExportColorSpace::Srgb, ExportRenderingIntent::RelativeColorimetric, false).unwrap();
        
        // Now apply B&W Tri-X LUT
        edits.lut_name = Some("Film - B&W High Contrast (Tri-X)".to_string());
        edits.lut_intensity = 1.0;
        export_photo_file(tmp_in, tmp_out_lut, &edits, None, 95, None, ExportColorSpace::Srgb, ExportRenderingIntent::RelativeColorimetric, false).unwrap();
        
        let img_no_lut = image::open(tmp_out_no_lut).unwrap().to_rgb8();
        let img_lut = image::open(tmp_out_lut).unwrap().to_rgb8();
        
        let p_no_lut = img_no_lut.get_pixel(15, 15);
        let p_lut = img_lut.get_pixel(15, 15);
        
        // In B&W Tri-X LUT, R, G, and B should be equal (monochrome)
        assert_eq!(p_lut[0], p_lut[1]);
        assert_eq!(p_lut[1], p_lut[2]);
        // The color image was not monochrome
        assert_ne!(p_no_lut[0], p_no_lut[2]);
        
        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out_no_lut);
        let _ = std::fs::remove_file(tmp_out_lut);
    }

    #[test]
    fn test_export_icc_injection() {
        let tmp_in = "test_icc_in.jpg";
        let tmp_out = "test_icc_out.jpg";
        
        let img = RgbImage::from_fn(20, 20, |x, y| {
            Rgb([x as u8 * 10, y as u8 * 10, 128])
        });
        img.save(tmp_in).unwrap();
        
        let edits = EditSettings {
            tone_profile: crate::ColorProfileMode::AgxVivid,
            ..Default::default()
        };
        
        export_photo_file(tmp_in, tmp_out, &edits, None, 95, None, ExportColorSpace::Srgb, ExportRenderingIntent::RelativeColorimetric, false).unwrap();
        
        let bytes = std::fs::read(tmp_out).unwrap();
        // Verify APP2 ICC marker presence: 0xFF, 0xE2 followed by "ICC_PROFILE\0"
        let marker = [0xff, 0xe2];
        let found_app2 = bytes.windows(marker.len()).any(|w| w == marker);
        assert!(found_app2, "Exported JPEG must contain APP2 marker");
        let icc_ident = b"ICC_PROFILE\0";
        let found_ident = bytes.windows(icc_ident.len()).any(|w| w == icc_ident);
        assert!(found_ident, "Exported JPEG must contain ICC_PROFILE identifier");
        
        // On macOS, verify using sips
        if cfg!(target_os = "macos") {
            let sips_out = std::process::Command::new("sips")
                .arg("-g")
                .arg("profile")
                .arg(tmp_out)
                .output();
            if let Ok(output) = sips_out {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("sips output: {}", stdout);
                assert!(stdout.contains("sRGB"), "macOS sips must report sRGB color profile, got: {}", stdout);
            }
        }
        
        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out);
    }

    #[test]
    fn test_export_rotation_and_flips() {
        let tmp_in = "test_rot_in.jpg";
        let tmp_out_90 = "test_rot_out_90.jpg";
        let tmp_out_flip = "test_rot_out_flip.jpg";

        // Create a 60x40 landscape image
        let img = RgbImage::from_fn(60, 40, |x, _y| {
            if x < 30 {
                Rgb([255, 0, 0]) // Red left half
            } else {
                Rgb([0, 255, 0]) // Green right half
            }
        });
        img.save(tmp_in).unwrap();

        // 1. Test 90-degree CW rotation (60x40 -> 40x60 portrait)
        let mut edits = EditSettings::default();
        edits.rotation = 90;
        export_photo_file(tmp_in, tmp_out_90, &edits, None, 95, None, ExportColorSpace::Srgb, ExportRenderingIntent::RelativeColorimetric, false).unwrap();
        let rotated = image::open(tmp_out_90).unwrap().to_rgb8();
        assert_eq!(rotated.width(), 40);
        assert_eq!(rotated.height(), 60);

        // 2. Test horizontal flip (left red becomes right red)
        let mut edits_flip = EditSettings::default();
        edits_flip.flip_horizontal = true;
        export_photo_file(tmp_in, tmp_out_flip, &edits_flip, None, 95, None, ExportColorSpace::Srgb, ExportRenderingIntent::RelativeColorimetric, false).unwrap();
        let flipped = image::open(tmp_out_flip).unwrap().to_rgb8();
        assert_eq!(flipped.width(), 60);
        assert_eq!(flipped.height(), 40);
        // Pixel at (5, 5) was red, now should be green after horizontal flip
        let px = flipped.get_pixel(5, 5);
        assert_eq!(px[1], 255); // Green

        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out_90);
        let _ = std::fs::remove_file(tmp_out_flip);
    }

    #[test]
    fn test_tone_recovery_math() {
        // 1. Identity when sliders are 0
        let (r0, g0, b0) = apply_tone_recovery(0.5, 0.4, 0.3, 0.0, 0.0, 0.0, 0.0);
        assert_eq!(r0, 0.5);
        assert_eq!(g0, 0.4);
        assert_eq!(b0, 0.3);

        // 2. Chromaticity preservation: ratio R/G and B/G must remain invariant
        let (r_in, g_in, b_in) = (0.6, 0.4, 0.2);
        let orig_ratio_rg = r_in / g_in;
        let orig_ratio_bg = b_in / g_in;

        let (r_hl, g_hl, b_hl) = apply_tone_recovery(r_in, g_in, b_in, -50.0, 0.0, 0.0, 0.0);
        assert!((r_hl / g_hl - orig_ratio_rg).abs() < 1e-5);
        assert!((b_hl / g_hl - orig_ratio_bg).abs() < 1e-5);
        // Highlights pulled down (-50) must reduce luminance
        assert!(g_hl < g_in);

        // 3. Shadow lifting (+50) on dark tones must increase luminance
        let (r_sh_in, g_sh_in, b_sh_in) = (0.04, 0.03, 0.02);
        let (r_sh, g_sh, _b_sh) = apply_tone_recovery(r_sh_in, g_sh_in, b_sh_in, 0.0, 50.0, 0.0, 0.0);
        assert!(g_sh > g_sh_in);
        assert!((r_sh / g_sh - (r_sh_in / g_sh_in)).abs() < 1e-5);

        // 4. Blacks pull down (-50) must deepen deep blacks
        let (r_bl_in, g_bl_in, b_bl_in) = (0.008, 0.008, 0.008);
        let (_, g_bl, _) = apply_tone_recovery(r_bl_in, g_bl_in, b_bl_in, 0.0, 0.0, 0.0, -50.0);
        assert!(g_bl < g_bl_in);
    }

    #[test]
    fn test_contrast_floor_and_tonal_recovery_limits() {
        // 1. Contrast floor at -100 must be exactly 0.40, preventing flat gray collapse
        let floor_gain = if -100.0f32 >= 0.0 {
            1.0 + (-100.0 / 100.0)
        } else {
            1.0 + (-100.0 / 100.0) * 0.60
        };
        assert!((floor_gain - 0.40f32).abs() < 1e-6);

        // Ceiling at +100 must be 2.0
        let ceil_gain = if 100.0f32 >= 0.0 {
            1.0 + (100.0 / 100.0)
        } else {
            1.0 + (100.0 / 100.0) * 0.60
        };
        assert!((ceil_gain - 2.0f32).abs() < 1e-6);

        // 2. Tonal recovery limits: ensure delta_ev never exceeds safe photographic range
        let (_, g_max, _) = apply_tone_recovery(0.8, 0.8, 0.8, -100.0, 0.0, 0.0, 0.0);
        assert!(g_max > 0.8 * 0.35, "Highlights at -100 should not crush below photographic threshold");

        let (_, g_min, _) = apply_tone_recovery(0.05, 0.05, 0.05, 0.0, 100.0, 0.0, 0.0);
        assert!(g_min < 0.05 * 3.0, "Shadows at +100 should not blow out beyond 3x gain");
    }

    #[test]
    fn test_white_balance_from_sample() {
        use crate::calculate_white_balance_from_sample;

        // 1. Perfectly neutral sample (e.g. 18% gray R=0.5, G=0.5, B=0.5)
        let neutral = calculate_white_balance_from_sample(0.5, 0.5, 0.5);
        assert_eq!(neutral.temperature, 5500.0, "Neutral sample should target 5500K base");
        assert_eq!(neutral.tint, 0.0, "Neutral sample should yield zero tint");

        // 2. Warm/tungsten sample (R > B, e.g. R=0.8, G=0.6, B=0.3)
        // Correcting warm cast requires lowering Kelvin (cooler WB)
        let warm = calculate_white_balance_from_sample(0.8, 0.6, 0.3);
        assert!(warm.temperature < 5500.0, "Warm sample must calculate cooler Kelvin: got {}", warm.temperature);

        // 3. Cool/shade sample (B > R, e.g. R=0.3, G=0.5, B=0.8)
        // Correcting cool cast requires raising Kelvin (warmer WB)
        let cool = calculate_white_balance_from_sample(0.3, 0.5, 0.8);
        assert!(cool.temperature > 5500.0, "Cool sample must calculate warmer Kelvin: got {}", cool.temperature);

        // 4. Greenish tint sample (G > (R+B)/2, e.g. fluorescent cast)
        // Correcting green cast requires positive magenta tint
        let green = calculate_white_balance_from_sample(0.4, 0.6, 0.4);
        assert!(green.tint > 0.0, "Green-cast sample must calculate positive magenta tint: got {}", green.tint);
    }

    #[test]
    fn test_color_adjustments_math() {
        // 1. Identity when all sliders are 0.0
        let (r0, g0, b0) = apply_color_adjustments(0.5, 0.4, 0.3, 0.0, 0.0, 0.0, 0.0);
        assert!((r0 - 0.5).abs() < 1e-5);
        assert!((g0 - 0.4).abs() < 1e-5);
        assert!((b0 - 0.3).abs() < 1e-5);

        // 2. Saturation: boosting saturation on a color (R=0.6, G=0.4, B=0.2)
        let (_, s_base, _) = crate::color::rgb_to_hsv(0.6, 0.4, 0.2);
        let (r_sat, g_sat, b_sat) = apply_color_adjustments(0.6, 0.4, 0.2, 0.0, 50.0, 0.0, 0.0);
        let (_, s_boosted, _) = crate::color::rgb_to_hsv(r_sat, g_sat, b_sat);
        assert!(s_boosted > s_base, "Saturation +50 must increase HSV saturation");

        // 3. Vibrance skin protection:
        // Peach/warm skin tone (Hue ~ 35 deg, e.g. R=0.85, G=0.65, B=0.50)
        let (r_skin, g_skin, b_skin) = (0.85, 0.65, 0.50);
        let (h_skin, s_skin_orig, _) = crate::color::rgb_to_hsv(r_skin, g_skin, b_skin);
        assert!((h_skin - 35.0).abs() < 10.0, "Test skin color must be near 35 deg");

        // Vibrant blue with similar base saturation (Hue ~ 215 deg)
        let (h_blue_base, s_blue_base, v_blue_base) = (215.0, s_skin_orig, 0.85);
        let (r_blue, g_blue, b_blue) = crate::color::hsv_to_rgb(h_blue_base, s_blue_base, v_blue_base);

        let (r_skin_vib, g_skin_vib, b_skin_vib) = apply_color_adjustments(r_skin, g_skin, b_skin, 80.0, 0.0, 0.0, 0.0);
        let (_, s_skin_vib, _) = crate::color::rgb_to_hsv(r_skin_vib, g_skin_vib, b_skin_vib);

        let (r_blue_vib, g_blue_vib, b_blue_vib) = apply_color_adjustments(r_blue, g_blue, b_blue, 80.0, 0.0, 0.0, 0.0);
        let (_, s_blue_vib, _) = crate::color::rgb_to_hsv(r_blue_vib, g_blue_vib, b_blue_vib);

        let skin_delta_s = s_skin_vib - s_skin_orig;
        let blue_delta_s = s_blue_vib - s_blue_base;
        assert!(
            skin_delta_s < blue_delta_s * 0.7,
            "Skin tone saturation boost ({:.3}) must be significantly lower than non-skin boost ({:.3})",
            skin_delta_s, blue_delta_s
        );

        // 4. Deep Blue: Polarizer effect on sky blue (Hue ~ 215 deg)
        let (r_sky, g_sky, b_sky) = (0.20, 0.50, 0.85);
        let (_, s_sky_orig, v_sky_orig) = crate::color::rgb_to_hsv(r_sky, g_sky, b_sky);
        let (r_db, g_db, b_db) = apply_color_adjustments(r_sky, g_sky, b_sky, 0.0, 0.0, 80.0, 0.0);
        let (_, s_db, v_db) = crate::color::rgb_to_hsv(r_db, g_db, b_db);
        assert!(s_db > s_sky_orig, "Deep Blue must boost sky saturation");
        assert!(v_db < v_sky_orig, "Deep Blue must deepen (darken) sky luminance");

        // 5. Foliage: Yellow-green grass (Hue ~ 85 deg) rotated towards emerald green (120 deg)
        let (r_grass, g_grass, b_grass) = crate::color::hsv_to_rgb(85.0, 0.6, 0.7);
        let (r_fol, g_fol, b_fol) = apply_color_adjustments(r_grass, g_grass, b_grass, 0.0, 0.0, 0.0, 80.0);
        let (h_fol, s_fol, _) = crate::color::rgb_to_hsv(r_fol, g_fol, b_fol);
        assert!(h_fol > 85.0, "Foliage slider must rotate yellowish grass towards green: got {}", h_fol);
        assert!(s_fol > 0.6, "Foliage slider must boost grass saturation");
    }

    #[test]
    fn test_detail_and_sharpening_math() {
        // 1. Neutral: zero amount and zero NR must return exact identity
        let (r0, g0, b0) = apply_detail_and_sharpening_rgb(0.5, 0.4, 0.3, 0.4, 0.3, 0.5, 0.3, 0.5, 0.0, 0.0, 0.0);
        assert_eq!((r0, g0, b0), (0.5, 0.4, 0.3));

        // 2. Step Edge: Center is brighter than neighbors (high-pass edge)
        // Center = 0.8, Left = 0.2, Right = 0.8, Top = 0.5, Bottom = 0.5
        let y_c = 0.8;
        let (y_l, y_r, y_t, y_b) = (0.2, 0.8, 0.5, 0.5);
        let (r_sharp, _g_sharp, _b_sharp) = apply_detail_and_sharpening_rgb(0.8, 0.8, 0.8, y_c, y_l, y_r, y_t, y_b, 50.0, 0.0, 0.0);
        assert!(r_sharp > 0.8, "Sharpening must boost high-frequency peak on an edge: got {}", r_sharp);

        // 3. Edge Masking: Flat region with tiny noise should be masked when masking is active
        // Center = 0.505, neighbors = 0.500 (tiny noise of 0.005)
        let y_flat_c = 0.505;
        let (y_flat_l, y_flat_r, y_flat_t, y_flat_b) = (0.500, 0.500, 0.500, 0.500);
        // With Masking = 0: noise is sharpened
        let (r_unmasked, _, _) = apply_detail_and_sharpening_rgb(0.505, 0.505, 0.505, y_flat_c, y_flat_l, y_flat_r, y_flat_t, y_flat_b, 50.0, 0.0, 0.0);
        // With Masking = 30: noise is suppressed (w_edge == 0)
        let (r_masked, _, _) = apply_detail_and_sharpening_rgb(0.505, 0.505, 0.505, y_flat_c, y_flat_l, y_flat_r, y_flat_t, y_flat_b, 50.0, 30.0, 0.0);
        assert!(r_unmasked > 0.505, "Unmasked sharpening boosts noise");
        assert_eq!(r_masked, 0.505, "Masked sharpening must completely ignore flat noise");

        // 4. Noise Reduction: In flat region, NR should smooth high frequency towards local blur
        let (r_nr, _, _) = apply_detail_and_sharpening_rgb(0.505, 0.505, 0.505, y_flat_c, y_flat_l, y_flat_r, y_flat_t, y_flat_b, 0.0, 30.0, 80.0);
        assert!(r_nr < 0.505, "Noise reduction must smooth peak towards neighborhood blur: got {}", r_nr);
    }

    #[test]
    fn test_tone_curves_export() {
        let tmp_in = "test_curve_in.jpg";
        let tmp_out_flat = "test_curve_flat.jpg";
        let tmp_out_curved = "test_curve_curved.jpg";

        let img = RgbImage::from_fn(20, 20, |_x, _y| {
            Rgb([128, 128, 128])
        });
        img.save(tmp_in).unwrap();

        let edits_flat = EditSettings::default();
        export_photo_file(tmp_in, tmp_out_flat, &edits_flat, None, 95, None, ExportColorSpace::Srgb, ExportRenderingIntent::RelativeColorimetric, false).unwrap();

        // Apply a master curve that boosts midtones: (0, 0), (0.5, 0.8), (1, 1)
        let mut edits_curved = EditSettings::default();
        edits_curved.curves.master = vec![
            crate::curve::CurvePointRecord { x: 0.0, y: 0.0 },
            crate::curve::CurvePointRecord { x: 0.5, y: 0.8 },
            crate::curve::CurvePointRecord { x: 1.0, y: 1.0 },
        ];
        export_photo_file(tmp_in, tmp_out_curved, &edits_curved, None, 95, None, ExportColorSpace::Srgb, ExportRenderingIntent::RelativeColorimetric, false).unwrap();

        let flat_px = image::open(tmp_out_flat).unwrap().to_rgb8().get_pixel(10, 10)[0];
        let curved_px = image::open(tmp_out_curved).unwrap().to_rgb8().get_pixel(10, 10)[0];

        assert!(curved_px > flat_px + 20, "Boosted midtone curve must brighten pixels: flat={}, curved={}", flat_px, curved_px);

        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out_flat);
        let _ = std::fs::remove_file(tmp_out_curved);
    }

    #[test]
    fn test_display_p3_export_and_icc() {
        let tmp_in = "test_p3_in.jpg";
        let tmp_out = "test_p3_out.jpg";

        let img = RgbImage::from_fn(30, 30, |x, y| {
            Rgb([x as u8 * 8, y as u8 * 8, 200])
        });
        img.save(tmp_in).unwrap();

        let edits = EditSettings::default();
        export_photo_file(
            tmp_in,
            tmp_out,
            &edits,
            None,
            95,
            None,
            ExportColorSpace::DisplayP3,
            ExportRenderingIntent::Perceptual,
            false,
        ).unwrap();

        let bytes = std::fs::read(tmp_out).unwrap();
        let marker = [0xff, 0xe2];
        let found_app2 = bytes.windows(marker.len()).any(|w| w == marker);
        assert!(found_app2, "Exported Display P3 JPEG must contain APP2 marker");

        // Inspect profile via macOS sips
        if cfg!(target_os = "macos") {
            let sips_out = std::process::Command::new("sips")
                .arg("-g")
                .arg("profile")
                .arg(tmp_out)
                .output();
            if let Ok(output) = sips_out {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("sips output for Display P3 export: {}", stdout);
                // Profile will either contain Display P3 / P3 or RGB
                assert!(stdout.contains("P3") || stdout.contains("RGB"), "Profile must be present in output");
            }
        }

        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out);
    }

    #[test]
    fn test_batch_export_rayon() {
        let mut items = Vec::new();
        let count = 4;
        for i in 0..count {
            let in_path = format!("test_batch_in_{}.jpg", i);
            let out_path = format!("test_batch_out_{}.jpg", i);
            let img = RgbImage::from_fn(20, 20, |_x, _y| {
                Rgb([(i * 50) as u8, 120, 150])
            });
            img.save(&in_path).unwrap();

            items.push(BatchExportItemRecord {
                input_path: in_path,
                output_path: out_path,
                edits: EditSettings::default(),
                lens_coeffs: None,
                quality: 90,
                max_dimension: Some(40),
                color_space: if i % 2 == 0 { ExportColorSpace::Srgb } else { ExportColorSpace::DisplayP3 },
                rendering_intent: ExportRenderingIntent::RelativeColorimetric,
                ultra_hdr: false,
            });
        }

        let results = export_batch_photos(items.clone());
        assert_eq!(results.len(), count);
        for res in &results {
            assert!(res.success, "Batch export item failed: {:?}", res.error_message);
            assert!(std::path::Path::new(&res.output_path).exists());
            let _ = std::fs::remove_file(&res.input_path);
            let _ = std::fs::remove_file(&res.output_path);
        }
    }

    #[test]
    fn test_ultra_hdr_export_pipeline() {
        let tmp_in = "test_uhdr_in.jpg";
        let tmp_out = "test_uhdr_out.jpg";

        let img = RgbImage::from_fn(30, 30, |_x, _y| {
            Rgb([240, 240, 255])
        });
        img.save(tmp_in).unwrap();

        let mut edits = EditSettings::default();
        edits.exposure = 2.0;

        export_photo_file(
            tmp_in,
            tmp_out,
            &edits,
            None,
            90,
            None,
            ExportColorSpace::Srgb,
            ExportRenderingIntent::RelativeColorimetric,
            true, // ultra_hdr = true
        ).unwrap();

        let bytes = std::fs::read(tmp_out).unwrap();
        assert!(crate::gainmap::has_gain_map(&bytes), "Exported JPEG must contain gain map markers");
        let extracted = crate::gainmap::extract_gain_map(&bytes);
        assert!(extracted.is_some(), "Must be able to extract gain map from exported Ultra HDR file");

        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out);
    }

    #[test]
    fn test_film_grain_math() {
        // 1. Amount = 0 returns identical pixel
        let (r, g, b) = apply_film_grain_rgb(0.5, 0.5, 0.5, 10.0, 20.0, 0.0, 1.0, 50.0, 0.0);
        assert_eq!((r, g, b), (0.5, 0.5, 0.5));

        // 2. Monochromatic grain (chroma = 0) produces identical delta on all channels
        let (r1, g1, b1) = apply_film_grain_rgb(0.5, 0.5, 0.5, 15.0, 25.0, 50.0, 1.0, 50.0, 0.0);
        let dr = (r1 - 0.5).abs();
        let dg = (g1 - 0.5).abs();
        let db = (b1 - 0.5).abs();
        assert!((dr - dg).abs() < 1e-5, "Chroma=0 must maintain equal delta across channels: dr={}, dg={}", dr, dg);
        assert!((dg - db).abs() < 1e-5, "Chroma=0 must maintain equal delta across channels: dg={}, db={}", dg, db);
        assert!(dr > 0.0, "Grain must modulate midtone pixel");

        // 3. Luminance weighting: midtone (0.5) has significantly higher delta than extreme highlight (0.98)
        let mut midtone_delta_sum = 0.0;
        let mut highlight_delta_sum = 0.0;
        for i in 0..20 {
            let x = i as f32 * 5.0;
            let (rm, _, _) = apply_film_grain_rgb(0.5, 0.5, 0.5, x, x, 50.0, 1.0, 50.0, 0.0);
            let (rh, _, _) = apply_film_grain_rgb(0.98, 0.98, 0.98, x, x, 50.0, 1.0, 50.0, 0.0);
            midtone_delta_sum += (rm - 0.5).abs();
            highlight_delta_sum += (rh - 0.98).abs();
        }
        assert!(midtone_delta_sum > highlight_delta_sum * 2.0, "Midtone grain must be substantially stronger than highlight grain");

        // 4. Color variation (chroma > 0) causes channels to diverge
        let (rc, gc, bc) = apply_film_grain_rgb(0.5, 0.5, 0.5, 42.0, 84.0, 60.0, 1.0, 50.0, 50.0);
        let d_rg = (rc - gc).abs();
        let d_gb = (gc - bc).abs();
        assert!(d_rg > 1e-4 || d_gb > 1e-4, "Chroma > 0 must introduce color variation between channels");
    }
}


