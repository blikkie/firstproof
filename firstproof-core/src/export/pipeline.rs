use crate::color::{
    Color3, REC2020_TO_LINEAR_SRGB,
};
use crate::EditSettings;
use super::decode::DemosaicedImage;

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
    
    let tint_factor = edits.tint * 0.002;
    let g_scale = (1.0 - tint_factor).max(0.01);
    r_scale = (r_scale + tint_factor * 0.5).max(0.01);
    b_scale = (b_scale + tint_factor * 0.5).max(0.01);
    
    let total_pixels = image.width * image.height;
    let has_tone_recovery = edits.highlights != 0.0 || edits.shadows != 0.0 || edits.whites != 0.0 || edits.blacks != 0.0;
    
    // If bilateral tone recovery is active, extract the pre-adjusted linear luminance plane
    // to separate base illumination from high-frequency detail without edge halos.
    let luma_plane = if has_tone_recovery {
        let mut plane = Vec::with_capacity(total_pixels);
        for i in 0..total_pixels {
            let idx = i * 3;
            let r = image.data[idx] * r_scale * exposure_gain;
            let g = image.data[idx + 1] * g_scale * exposure_gain;
            let b = image.data[idx + 2] * b_scale * exposure_gain;
            plane.push(0.2126 * r + 0.7152 * g + 0.0722 * b);
        }
        Some(plane)
    } else {
        None
    };
    
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
        
        // 2b. Advanced Bilateral Base/Detail Tone Recovery (Highlights, Shadows, Whites, Blacks)
        if let Some(ref luma) = luma_plane {
            let x = i % image.width;
            let y = i / image.width;
            let w = image.width;
            let h = image.height;
            
            let x_l = x.saturating_sub(2);
            let x_r = (x + 2).min(w - 1);
            let y_t = y.saturating_sub(2);
            let y_b = (y + 2).min(h - 1);
            
            let y_c = luma[i];
            let y_neighbors = [
                luma[y * w + x_l],
                luma[y * w + x_r],
                luma[y_t * w + x],
                luma[y_b * w + x],
                luma[y_t * w + x_l],
                luma[y_t * w + x_r],
                luma[y_b * w + x_l],
                luma[y_b * w + x_r],
            ];
            
            let (r_rec, g_rec, b_rec) = apply_bilateral_tone_recovery_rgb(
                r, g, b,
                y_c,
                &y_neighbors,
                edits.highlights, edits.shadows, edits.whites, edits.blacks,
            );
            r = r_rec;
            g = g_rec;
            b = b_rec;
        }
        
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

/// Applies advanced bilateral base/detail tone recovery (Highlights, Shadows, Whites, Blacks)
/// to scene-referred linear RGB values using edge-preserving spatial illumination separation.
/// y_neighbors contains 8 neighborhood luminances: 4 orthogonal (L, R, T, B) followed by 4 diagonal (TL, TR, BL, BR).
#[inline(always)]
pub fn apply_bilateral_tone_recovery_rgb(
    r: f32, g: f32, b: f32,
    y_c: f32,
    y_neighbors: &[f32; 8],
    highlights: f32, shadows: f32, whites: f32, blacks: f32,
) -> (f32, f32, f32) {
    if highlights == 0.0 && shadows == 0.0 && whites == 0.0 && blacks == 0.0 {
        return (r, g, b);
    }
    
    if y_c <= 1e-6 {
        return (r, g, b);
    }
    
    // Log-luminance EV relative to photographic middle gray (0.18)
    let ev_c = (y_c.max(1e-5) / 0.18).log2();
    
    // Bilateral filter base illumination estimation:
    // Spatial weights: Center = 1.0, Orthogonal (dist 1) = exp(-0.5) ~ 0.60653066, Diagonal (dist sqrt(2)) = exp(-1.0) ~ 0.36787944
    // Range weights: Gaussian in EV stops with sigma_r = 0.85 EV (2 * sigma_r^2 = 1.445)
    const W_S_ORTHO: f32 = 0.60653066;
    const W_S_DIAG: f32 = 0.36787944;
    const INV_TWO_SIGMA_R_SQ: f32 = 1.0 / 1.445;
    
    let mut total_weight = 1.0f32;
    let mut weighted_ev_sum = ev_c;
    
    // 0..3: Orthogonal neighbors (left, right, top, bottom)
    for i in 0..4 {
        let ev_i = (y_neighbors[i].max(1e-5) / 0.18).log2();
        let d_ev = ev_i - ev_c;
        let w_r = (-(d_ev * d_ev) * INV_TWO_SIGMA_R_SQ).exp();
        let w = W_S_ORTHO * w_r;
        total_weight += w;
        weighted_ev_sum += w * ev_i;
    }
    
    // 4..7: Diagonal neighbors (top-left, top-right, bottom-left, bottom-right)
    for i in 4..8 {
        let ev_i = (y_neighbors[i].max(1e-5) / 0.18).log2();
        let d_ev = ev_i - ev_c;
        let w_r = (-(d_ev * d_ev) * INV_TWO_SIGMA_R_SQ).exp();
        let w = W_S_DIAG * w_r;
        total_weight += w;
        weighted_ev_sum += w * ev_i;
    }
    
    let ev_base = weighted_ev_sum / total_weight;
    let mut delta_ev = 0.0f32;
    
    // 1. Blacks: Anchor deep shadows (ev <= -1.0)
    if blacks != 0.0 {
        let u = ((-ev_base - 1.0) / 3.0).clamp(0.0, 1.0);
        let w_blacks = u * u * (3.0 - 2.0 * u);
        delta_ev += (blacks / 100.0) * 0.90 * w_blacks;
    }
    
    // 2. Shadows: Lift lower midtones and shadow textures (ev in [-4.0, 0.0])
    if shadows != 0.0 {
        let u = ((ev_base + 4.0) / 4.0).clamp(0.0, 1.0);
        let w_shadows = (std::f32::consts::PI * u).sin();
        delta_ev += (shadows / 100.0) * 1.20 * w_shadows;
    }
    
    // 3. Highlights: Recover or boost bright areas and skies (ev in [0.0, 4.0])
    if highlights != 0.0 {
        let u = (ev_base / 4.0).clamp(0.0, 1.0);
        let w_highlights = (std::f32::consts::PI * u).sin();
        delta_ev += (highlights / 100.0) * 1.30 * w_highlights;
    }
    
    // 4. Whites: Adjust specular highlight threshold and ceiling (ev >= 1.0)
    if whites != 0.0 {
        let u = ((ev_base - 1.0) / 3.0).clamp(0.0, 1.0);
        let w_whites = u * u * (3.0 - 2.0 * u);
        delta_ev += (whites / 100.0) * 0.90 * w_whites;
    }
    
    if delta_ev == 0.0 {
        return (r, g, b);
    }
    
    let gain = 2.0f32.powf(delta_ev);
    (r * gain, g * gain, b * gain)
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
    let neighbors = [lum; 8];
    apply_bilateral_tone_recovery_rgb(r, g, b, lum, &neighbors, highlights, shadows, whites, blacks)
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

        // Detect human skin tones in Linear Rec.2020 using normalized (r_n, g_n) chromaticity locus:
        // Universal across the entire human melanin spectrum (Fitzpatrick I-VI / MST 01-10) from fair to deep dark.
        let sum = r + g + b;
        let is_warm = (r > g) && (g > b);
        let skin_metric = if is_warm && sum > 0.015 {
            let r_n = r / sum;
            let g_n = g / sum;
            let d_r = (r_n - 0.44).abs();
            let d_g = (g_n - 0.32).abs();
            (1.0 - (d_r * 4.5 + d_g * 9.0)).clamp(0.0, 1.0)
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
    if deep_blue != 0.0 {
        let max_warm = r.max(g);
        let sky_signal = (b - max_warm).max(0.0);
        if sky_signal > 0.0 {
            let x = deep_blue / 100.0;
            let factor = x.signum() * x.abs().powf(1.6);
            if factor > 0.0 {
                r -= factor * sky_signal * 0.30;
                g -= factor * sky_signal * 0.15;
                b -= factor * sky_signal * 0.20;
            } else {
                r -= factor * sky_signal * 0.25;
                g -= factor * sky_signal * 0.20;
                b -= factor * sky_signal * 0.10;
            }
        }
    }

    // 4. Foliage (Lush Greens) (-100..+100)
    if foliage != 0.0 {
        let max_rb = r.max(b);
        let green_signal = (g - max_rb).max(0.0);
        if green_signal > 0.0 {
            let x = foliage / 100.0;
            let factor = x.signum() * x.abs().powf(1.6);
            r -= factor * green_signal * 0.30;
            b -= factor * green_signal * 0.15;
            g += factor * green_signal * 0.10;
        }
    }

    (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}

/// Applies Split Toning & Color Grading with optical balance anchoring.
/// Uses pure chrominance vectors (zero photometric luminance shift) and smooth
/// cubic Hermite transitions anchored to the perceptual luminance midpoint.
#[inline(always)]
pub fn apply_split_toning(
    r: f32, g: f32, b: f32,
    shadow_hue: f32,
    shadow_sat: f32,
    highlight_hue: f32,
    highlight_sat: f32,
    balance: f32,
) -> (f32, f32, f32) {
    if shadow_sat <= 0.0 && highlight_sat <= 0.0 {
        return (r, g, b);
    }

    let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let y_clamp = lum.clamp(0.0, 1.0);

    // Midpoint threshold shifted by balance (-100..+100)
    let t = (0.5 - (balance / 100.0) * 0.25).clamp(0.1, 0.9);

    let mut delta_r = 0.0f32;
    let mut delta_g = 0.0f32;
    let mut delta_b = 0.0f32;

    // Shadow Toning
    if shadow_sat > 0.0 && y_clamp < t {
        let u_s = (t - y_clamp) / t;
        let w_s = u_s * u_s * (3.0 - 2.0 * u_s) * (shadow_sat / 100.0);
        let (tr, tg, tb) = crate::color::hsv_to_rgb(shadow_hue, 1.0, 1.0);
        let t_lum = 0.2126 * tr + 0.7152 * tg + 0.0722 * tb;
        delta_r += w_s * (tr - t_lum);
        delta_g += w_s * (tg - t_lum);
        delta_b += w_s * (tb - t_lum);
    }

    // Highlight Toning
    if highlight_sat > 0.0 && y_clamp > t {
        let u_h = (y_clamp - t) / (1.0 - t);
        let w_h = u_h * u_h * (3.0 - 2.0 * u_h) * (highlight_sat / 100.0);
        let (tr, tg, tb) = crate::color::hsv_to_rgb(highlight_hue, 1.0, 1.0);
        let t_lum = 0.2126 * tr + 0.7152 * tg + 0.0722 * tb;
        delta_r += w_h * (tr - t_lum);
        delta_g += w_h * (tg - t_lum);
        delta_b += w_h * (tb - t_lum);
    }

    (
        (r + delta_r).clamp(0.0, 1.0),
        (g + delta_g).clamp(0.0, 1.0),
        (b + delta_b).clamp(0.0, 1.0),
    )
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

    let w_edge = if masking <= 0.0 {
        1.0
    } else {
        let m = (masking / 100.0).clamp(0.0, 1.0);
        let threshold = m.powf(1.5) * 0.08;
        let t = (grad_mag / threshold.max(1e-4)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };

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
        let theta = c_t * std::f32::consts::FRAC_PI_2;
        let w_mono = theta.cos();
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
