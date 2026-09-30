use crate::color::{
    Color3, REC2020_TO_LINEAR_SRGB,
};
use crate::EditSettings;
use super::decode::DemosaicedImage;

#[inline(always)]
fn is_curve_active(pts: &[crate::CurvePointRecord]) -> bool {
    pts.len() > 2 || (pts.len() == 2 && ((pts[0].x - pts[0].y).abs() > 1e-4 || (pts[1].x - pts[1].y).abs() > 1e-4))
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
    
    let tint_factor = edits.tint * 0.002;
    let g_scale = (1.0 - tint_factor).max(0.01);
    r_scale = (r_scale + tint_factor * 0.5).max(0.01);
    b_scale = (b_scale + tint_factor * 0.5).max(0.01);
    
    let curve_lut = crate::curve::evaluate_curves_lut_ref(&edits.curves);
    let has_master_curve = is_curve_active(&edits.curves.master);
    let has_red_curve = is_curve_active(&edits.curves.red);
    let has_green_curve = is_curve_active(&edits.curves.green);
    let has_blue_curve = is_curve_active(&edits.curves.blue);
    let has_any_curve = has_master_curve || has_red_curve || has_green_curve || has_blue_curve;

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
            plane.push(0.2627 * r + 0.6780 * g + 0.0593 * b);
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
        
        // 1. White Balance in Linear Rec.2020
        r *= r_scale;
        g *= g_scale;
        b *= b_scale;
        
        // 2. Linear Exposure (including baseline camera gain for RAW) in Linear Rec.2020
        r *= exposure_gain;
        g *= exposure_gain;
        b *= exposure_gain;
        
        // 2b. Advanced Bilateral Base/Detail Tone Recovery (Highlights, Shadows, Whites, Blacks) in Linear Rec.2020
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
        
        // 2c. Upstream Recording Emulsion (DIR Chemical Couplers & Sensitometric Curves) in Linear Rec.2020
        if edits.emulsion_archetype != crate::EmulsionArchetype::None {
            let (r_em, g_em, b_em) = crate::color::apply_recording_emulsion(
                r, g, b,
                edits.emulsion_archetype,
                edits.emulsion_exposure,
            );
            r = r_em;
            g = g_em;
            b = b_em;
        }

        // 2d. Wide-Gamut Tone Curves (Master + RGB) in Linear Rec.2020
        if has_any_curve {
            if has_master_curve {
                r = sample_curve_lut(&curve_lut.master, r);
                g = sample_curve_lut(&curve_lut.master, g);
                b = sample_curve_lut(&curve_lut.master, b);
            }
            if has_red_curve {
                r = sample_curve_lut(&curve_lut.red, r);
            }
            if has_green_curve {
                g = sample_curve_lut(&curve_lut.green, g);
            }
            if has_blue_curve {
                b = sample_curve_lut(&curve_lut.blue, b);
            }
        }

        // 2e. Color Adjustments: Vibrance, Saturation, Deep Blue, Foliage in Linear Rec.2020
        let (r_adj, g_adj, b_adj) = apply_color_adjustments(
            r, g, b,
            edits.vibrance, edits.saturation, edits.deep_blue, edits.foliage,
        );
        r = r_adj;
        g = g_adj;
        b = b_adj;

        // 2f. Split Toning & Color Grading in Linear Rec.2020
        let (r_st, g_st, b_st) = apply_split_toning(
            r, g, b,
            edits.split_toning_shadow_hue, edits.split_toning_shadow_sat,
            edits.split_toning_highlight_hue, edits.split_toning_highlight_sat,
            edits.split_toning_balance,
        );
        r = r_st;
        g = g_st;
        b = b_st;
        
        // 3. TERMINAL STAGE: AgX Filmic or Camera-Matching Tone Profiles -> Linear sRGB Output
        // Tone profiles output linear sRGB at the terminal stage, ensuring all tone, curve,
        // and color adjustments execute with physical wide-gamut fidelity in Linear Rec.2020.
        if image.is_raw || edits.contrast != 0.0 || edits.highlights != 0.0 || edits.shadows != 0.0 || edits.whites != 0.0 || edits.blacks != 0.0 || edits.emulsion_archetype != crate::EmulsionArchetype::None || edits.tone_profile != crate::ColorProfileMode::AgxFilmic {
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
                    // Terminal Stage 10: Convert Rec.2020 working space to Linear sRGB
                    let c_rec2020 = Color3::new(r, g, b);
                    let c_srgb = REC2020_TO_LINEAR_SRGB.multiply_vector(c_rec2020);
                    let r_init = c_srgb.r().max(0.0);
                    let g_init = c_srgb.g().max(0.0);
                    let b_init = c_srgb.b().max(0.0);

                    // Clean camera profiles operate directly in linear sRGB without synthetic hue-twisting.
                    let (r_mapped, g_mapped, b_mapped) = (r_init, g_init, b_init);
                    
                    let curve = match mode {
                        crate::ColorProfileMode::CameraPortrait => crate::color::dcp::DcpProfile::built_in_portrait_tone_curve(),
                        crate::ColorProfileMode::CameraLandscape => crate::color::dcp::DcpProfile::built_in_landscape_tone_curve(),
                        crate::ColorProfileMode::CameraVivid => crate::color::dcp::DcpProfile::built_in_vivid_tone_curve(),
                        _ => crate::color::dcp::DcpProfile::built_in_standard_tone_curve(),
                    };
                    
                    // Continuous sensitometric per-channel tone curve (Hurter-Driffield film D-log H model):
                    // Evaluates each channel through the smooth C^1 camera spline.
                    // Naturally self-limiting to [0.0, 1.0] and desaturates highlights to white along the shoulder without arbitrary thresholds.
                    let apply_curve = |v: f32| -> f32 {
                        let curved = crate::color::evaluate_tone_curve(v, &curve);
                        if (contrast_gain - 1.0).abs() > 0.001 {
                            ((curved - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0)
                        } else {
                            curved.clamp(0.0, 1.0)
                        }
                    };

                    r = apply_curve(r_mapped);
                    g = apply_curve(g_mapped);
                    b = apply_curve(b_mapped);
                }
            }
        } else {
            // Passthrough for non-raw with no profile adjustments: convert Rec.2020 working space to linear sRGB
            let c_rec2020 = Color3::new(r, g, b);
            let c_srgb = REC2020_TO_LINEAR_SRGB.multiply_vector(c_rec2020);
            r = c_srgb.r().max(0.0);
            g = c_srgb.g().max(0.0);
            b = c_srgb.b().max(0.0);
        }
        
        // Store linear sRGB color values
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
    
    // 2. Shadows: Lift lower midtones and shadow textures (ev <= 0.0)
    if shadows != 0.0 {
        let u = (-ev_base / 3.0).clamp(0.0, 1.0);
        let w_shadows = u * u * (3.0 - 2.0 * u);
        delta_ev += (shadows / 100.0) * 1.20 * w_shadows;
    }
    
    // 3. Highlights: Recover or boost bright areas and skies (ev >= 0.0)
    if highlights != 0.0 {
        let u = (ev_base / 3.0).clamp(0.0, 1.0);
        let w_highlights = u * u * (3.0 - 2.0 * u);
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
    let lum = 0.2627 * r + 0.6780 * g + 0.0593 * b;
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

    let lum = 0.2627 * r + 0.6780 * g + 0.0593 * b;

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

        // Detect human skin tones in Linear Rec.2020 using continuous 2D Gaussian melanin locus:
        // Center: (r_n, g_n) = (0.44, 0.32) across Fitzpatrick I-VI / Monk Skin Tone 01-10.
        let sum = r + g + b;
        let skin_metric = if sum > 1e-5 {
            let r_n = r / sum;
            let g_n = g / sum;
            let d_r = (r_n - 0.44) / 0.12;
            let d_g = (g_n - 0.32) / 0.06;
            let d2 = d_r * d_r + d_g * d_g;
            let gauss = (-0.5 * d2).exp();

            // Smooth Hermite shadow fade between sum = 0.005 and sum = 0.030
            // so deep shadow noise does not trigger erratic chromaticity protection,
            // while ramping to 1.0 without hard thresholds.
            let u_lum = ((sum - 0.005) / 0.025).clamp(0.0, 1.0);
            let w_lum = u_lum * u_lum * (3.0 - 2.0 * u_lum);

            // Smooth warm orientation weighting: in human skin, r > g and g > b.
            // Continuous C1 Hermite transitions replace binary knife-edges:
            let u_rg = ((r_n - g_n + 0.02) / 0.06).clamp(0.0, 1.0);
            let w_rg = u_rg * u_rg * (3.0 - 2.0 * u_rg);

            let b_n = 1.0 - r_n - g_n;
            let u_gb = ((g_n - b_n + 0.02) / 0.06).clamp(0.0, 1.0);
            let w_gb = u_gb * u_gb * (3.0 - 2.0 * u_gb);

            gauss * w_lum * w_rg * w_gb
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
    // Symmetric subtractive density manifold across positive and negative adjustments.
    if deep_blue != 0.0 {
        let max_warm = r.max(g);
        let sky_signal = (b - max_warm).max(0.0);
        if sky_signal > 0.0 {
            let x = deep_blue / 100.0;
            let factor = x.signum() * x.abs().powf(1.6);
            r -= factor * sky_signal * 0.28;
            g -= factor * sky_signal * 0.18;
            b -= factor * sky_signal * 0.15;
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

    let lum = 0.2627 * r + 0.6780 * g + 0.0593 * b;
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
        let t_lum = 0.2627 * tr + 0.6780 * tg + 0.0593 * tb;
        delta_r += w_s * (tr - t_lum);
        delta_g += w_s * (tg - t_lum);
        delta_b += w_s * (tb - t_lum);
    }

    // Highlight Toning
    if highlight_sat > 0.0 && y_clamp > t {
        let u_h = (y_clamp - t) / (1.0 - t);
        let w_h = u_h * u_h * (3.0 - 2.0 * u_h) * (highlight_sat / 100.0);
        let (tr, tg, tb) = crate::color::hsv_to_rgb(highlight_hue, 1.0, 1.0);
        let t_lum = 0.2627 * tr + 0.6780 * tg + 0.0593 * tb;
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

    let sharp_gain = (amount / 100.0) * 1.5;
    let sharp_term = sharp_gain * w_edge * delta_y;
    let nr_term = (noise_reduction / 100.0) * (1.0 - w_edge) * delta_y;
    let delta_adj = sharp_term - nr_term;

    (
        (r + delta_adj).clamp(0.0, 1.0),
        (g + delta_adj).clamp(0.0, 1.0),
        (b + delta_adj).clamp(0.0, 1.0),
    )
}


