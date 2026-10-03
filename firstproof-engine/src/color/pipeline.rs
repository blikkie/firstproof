//! Canonical Reference Development Pipeline
//!
//! Ground truth implementation of First Proof's complete image processing pipeline,
//! enforcing bit-level multi-engine mathematical parity across:
//! - Rust CPU export pipeline (`firstproof-core`)
//! - WebGL 2 GPU shader pipeline (`web/src/shaders/`)
//! - macOS Metal GPU shader pipeline (`Shaders.metal`)
//!
//! Guarantees:
//! 1. Hardware-bilinear curve LUT sampling (matching GPU texture units).
//! 2. Strict IEEE-754 32-bit floating point arithmetic.
//! 3. Pure Linear Rec.2020 working space before terminal display conversion.

use crate::color::{
    agx_tone_map, agx_tone_map_vivid, apply_print_media, apply_recording_emulsion,
    built_in_landscape_tone_curve, built_in_portrait_tone_curve, built_in_standard_tone_curve,
    built_in_vivid_tone_curve, evaluate_tone_curve, hsv_to_rgb, linear_rgb_to_srgb8,
    srgb_gamma_to_linear, Color3, LINEAR_SRGB_TO_REC2020, REC2020_TO_LINEAR_SRGB,
};
use crate::curve::AllCurvesLutRecord;
use crate::modes::{ColorProfileMode, EmulsionArchetype, EmulsionGrainMode, PrintMediaMode};
use crate::raw::f16_to_f32;
use crate::settings::EditSettings;

/// Determines if edits represent the untouched default state for neutral passthrough.
pub fn is_neutral_edits(edits: &EditSettings) -> bool {
    edits.exposure == 0.0
        && edits.contrast == 0.0
        && edits.highlights == 0.0
        && edits.shadows == 0.0
        && edits.whites == 0.0
        && edits.blacks == 0.0
        && edits.vibrance == 0.0
        && edits.saturation == 0.0
        && edits.deep_blue == 0.0
        && edits.foliage == 0.0
        && (edits.temperature - 5500.0).abs() < 1.0
        && edits.tint == 0.0
        && edits.emulsion_archetype == EmulsionArchetype::None
        && edits.emulsion_grain_mode == EmulsionGrainMode::Off
        && edits.print_media == PrintMediaMode::None
        && edits.split_toning_shadow_sat == 0.0
        && edits.split_toning_highlight_sat == 0.0
        && edits.sharpening_amount == 0.0
        && edits.noise_reduction == 0.0
}

/// Samples a 256-entry 1D curve LUT with linear interpolation matching GPU texture hardware samplers.
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

/// Applies advanced bilateral base/detail tone recovery (Highlights, Shadows, Whites, Blacks)
/// to scene-referred linear RGB values using edge-preserving spatial illumination separation.
/// y_neighbors contains 8 neighborhood luminances: 4 orthogonal (L, R, T, B) followed by 4 diagonal (TL, TR, BL, BR).
#[inline(always)]
pub fn apply_bilateral_tone_recovery_rgb(
    r: f32,
    g: f32,
    b: f32,
    y_c: f32,
    y_neighbors: &[f32; 8],
    highlights: f32,
    shadows: f32,
    whites: f32,
    blacks: f32,
) -> (f32, f32, f32) {
    if highlights == 0.0 && shadows == 0.0 && whites == 0.0 && blacks == 0.0 {
        return (r, g, b);
    }

    if y_c <= 1e-6 {
        return (r, g, b);
    }

    // Log-luminance EV relative to photographic middle gray (0.18)
    let ev_c = (y_c.max(1e-5) / 0.18).log2();

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
#[inline(always)]
pub fn apply_tone_recovery(
    r: f32,
    g: f32,
    b: f32,
    highlights: f32,
    shadows: f32,
    whites: f32,
    blacks: f32,
) -> (f32, f32, f32) {
    if highlights == 0.0 && shadows == 0.0 && whites == 0.0 && blacks == 0.0 {
        return (r, g, b);
    }
    let lum = 0.2627 * r + 0.6780 * g + 0.0593 * b;
    let neighbors = [lum; 8];
    apply_bilateral_tone_recovery_rgb(
        r, g, b, lum, &neighbors, highlights, shadows, whites, blacks,
    )
}

/// Applies photographic color adjustments (Vibrance, Saturation, Deep Blue, Foliage)
/// in Linear Rec.2020 working space.
#[inline(always)]
pub fn apply_color_adjustments(
    mut r: f32,
    mut g: f32,
    mut b: f32,
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
    if saturation != 0.0 {
        let sat_gain = (1.0 + saturation / 100.0).max(0.0);
        r = lum + (r - lum) * sat_gain;
        g = lum + (g - lum) * sat_gain;
        b = lum + (b - lum) * sat_gain;
    }

    // 2. Smart Vibrance (-100..+100)
    if vibrance != 0.0 {
        let max_c = r.max(g).max(b);
        let min_c = r.min(g).min(b);
        let chroma = max_c - min_c;

        let sum = r + g + b;
        let skin_metric = if sum > 1e-5 {
            let r_n = r / sum;
            let g_n = g / sum;
            let d_r = (r_n - 0.44) / 0.12;
            let d_g = (g_n - 0.32) / 0.06;
            let d2 = d_r * d_r + d_g * d_g;
            let gauss = (-0.5 * d2).exp();

            let u_lum = ((sum - 0.005) / 0.025).clamp(0.0, 1.0);
            let w_lum = u_lum * u_lum * (3.0 - 2.0 * u_lum);

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

/// Applies Split Toning & Color Grading with optical balance anchoring in Linear Rec.2020.
#[inline(always)]
pub fn apply_split_toning(
    r: f32,
    g: f32,
    b: f32,
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

    let t = (0.5 - (balance / 100.0) * 0.25).clamp(0.1, 0.9);

    let mut delta_r = 0.0f32;
    let mut delta_g = 0.0f32;
    let mut delta_b = 0.0f32;

    // Shadow Toning
    if shadow_sat > 0.0 && y_clamp < t {
        let u_s = (t - y_clamp) / t;
        let w_s = u_s * u_s * (3.0 - 2.0 * u_s) * (shadow_sat / 100.0);
        let (tr, tg, tb) = hsv_to_rgb(shadow_hue, 1.0, 1.0);
        let t_lum = 0.2627 * tr + 0.6780 * tg + 0.0593 * tb;
        delta_r += w_s * (tr - t_lum);
        delta_g += w_s * (tg - t_lum);
        delta_b += w_s * (tb - t_lum);
    }

    // Highlight Toning
    if highlight_sat > 0.0 && y_clamp > t {
        let u_h = (y_clamp - t) / (1.0 - t);
        let w_h = u_h * u_h * (3.0 - 2.0 * u_h) * (highlight_sat / 100.0);
        let (tr, tg, tb) = hsv_to_rgb(highlight_hue, 1.0, 1.0);
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

/// Applies edge-masked high-frequency luminance unsharp masking and flat-area noise reduction.
#[inline(always)]
pub fn apply_detail_and_sharpening_rgb(
    r: f32,
    g: f32,
    b: f32,
    y_c: f32,
    y_l: f32,
    y_r: f32,
    y_t: f32,
    y_b: f32,
    amount: f32,
    masking: f32,
    noise_reduction: f32,
) -> (f32, f32, f32) {
    if amount <= 0.0 && noise_reduction <= 0.0 {
        return (r, g, b);
    }

    let y_neighbors = 0.25 * (y_l + y_r + y_t + y_b);
    let delta_y = y_c - y_neighbors;

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

/// Evaluates pre-sharpening display luminance for unsharp mask neighborhood probing.
pub fn evaluate_display_luma(
    r_in: f32,
    g_in: f32,
    b_in: f32,
    is_linear_rec2020: bool,
    edits: &EditSettings,
) -> f32 {
    let mut rec2020 = if is_linear_rec2020 {
        Color3::new(r_in, g_in, b_in)
    } else {
        let srgb_lin = Color3::new(
            srgb_gamma_to_linear(r_in),
            srgb_gamma_to_linear(g_in),
            srgb_gamma_to_linear(b_in),
        );
        LINEAR_SRGB_TO_REC2020.multiply_vector(srgb_lin)
    };

    // White balance
    if (edits.temperature - 5500.0).abs() >= 1.0 || edits.tint != 0.0 {
        let temp_norm = (edits.temperature / 5500.0).clamp(0.25, 4.0);
        let mut r_scale = temp_norm.powf(0.75);
        let mut b_scale = (1.0 / temp_norm).powf(0.75);
        let tint_factor = edits.tint * 0.002;
        let g_scale = (1.0 - tint_factor).max(0.01);
        r_scale = (r_scale + tint_factor * 0.5).max(0.01);
        b_scale = (b_scale + tint_factor * 0.5).max(0.01);
        rec2020 = Color3::new(
            rec2020.r() * r_scale,
            rec2020.g() * g_scale,
            rec2020.b() * b_scale,
        );
    }

    // Exposure
    if edits.exposure != 0.0 {
        let gain = 2.0f32.powf(edits.exposure);
        rec2020 = Color3::new(rec2020.r() * gain, rec2020.g() * gain, rec2020.b() * gain);
    }

    // Tone recovery
    let (tr, tg, tb) = apply_tone_recovery(
        rec2020.r(),
        rec2020.g(),
        rec2020.b(),
        edits.highlights,
        edits.shadows,
        edits.whites,
        edits.blacks,
    );
    rec2020 = Color3::new(tr, tg, tb);

    // Recording emulsion
    if edits.emulsion_archetype != EmulsionArchetype::None {
        let (er, eg, eb) = apply_recording_emulsion(
            rec2020.r(),
            rec2020.g(),
            rec2020.b(),
            edits.emulsion_archetype,
            edits.emulsion_exposure,
            edits.enlarger_filter,
        );
        rec2020 = Color3::new(er, eg, eb);
    }

    let contrast_gain = if edits.contrast >= 0.0 {
        1.0 + edits.contrast / 100.0
    } else {
        1.0 + (edits.contrast / 100.0) * 0.60
    };

    match edits.tone_profile {
        ColorProfileMode::AgxVivid => {
            let srgb = agx_tone_map_vivid(rec2020, contrast_gain);
            0.2627 * srgb.r().clamp(0.0, 1.0)
                + 0.6780 * srgb.g().clamp(0.0, 1.0)
                + 0.0593 * srgb.b().clamp(0.0, 1.0)
        }
        ColorProfileMode::AgxFilmic => {
            let srgb = agx_tone_map(rec2020, contrast_gain);
            0.2627 * srgb.r().clamp(0.0, 1.0)
                + 0.6780 * srgb.g().clamp(0.0, 1.0)
                + 0.0593 * srgb.b().clamp(0.0, 1.0)
        }
        mode => {
            let c_srgb = REC2020_TO_LINEAR_SRGB.multiply_vector(rec2020);
            let lum = 0.2627 * c_srgb.r() + 0.6780 * c_srgb.g() + 0.0593 * c_srgb.b();
            let curve = match mode {
                ColorProfileMode::CameraPortrait => built_in_portrait_tone_curve(),
                ColorProfileMode::CameraLandscape => built_in_landscape_tone_curve(),
                ColorProfileMode::CameraVivid => built_in_vivid_tone_curve(),
                _ => built_in_standard_tone_curve(),
            };
            let lum_curved = evaluate_tone_curve(lum, &curve);
            if (contrast_gain - 1.0).abs() > 0.001 {
                ((lum_curved - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0)
            } else {
                lum_curved.clamp(0.0, 1.0)
            }
        }
    }
}

/// Executes full reference CPU export pipeline on a single pixel.
pub fn develop_pixel_pipeline(
    r_in: f32,
    g_in: f32,
    b_in: f32,
    is_linear_rec2020: bool,
    edits: &EditSettings,
    curve_lut: Option<&AllCurvesLutRecord>,
    neighbors: Option<[f32; 5]>, // [y_c, y_l, y_r, y_t, y_b]
) -> [u8; 3] {
    // 1. Scene-Referred Working Space
    let mut rec2020 = if is_linear_rec2020 {
        Color3::new(r_in, g_in, b_in)
    } else {
        let srgb_lin = Color3::new(
            srgb_gamma_to_linear(r_in),
            srgb_gamma_to_linear(g_in),
            srgb_gamma_to_linear(b_in),
        );
        LINEAR_SRGB_TO_REC2020.multiply_vector(srgb_lin)
    };

    // 2. White Balance
    if (edits.temperature - 5500.0).abs() >= 1.0 || edits.tint != 0.0 {
        let temp_norm = (edits.temperature / 5500.0).clamp(0.25, 4.0);
        let mut r_scale = temp_norm.powf(0.75);
        let mut b_scale = (1.0 / temp_norm).powf(0.75);
        let tint_factor = edits.tint * 0.002;
        let g_scale = (1.0 - tint_factor).max(0.01);
        r_scale = (r_scale + tint_factor * 0.5).max(0.01);
        b_scale = (b_scale + tint_factor * 0.5).max(0.01);
        rec2020 = Color3::new(
            rec2020.r() * r_scale,
            rec2020.g() * g_scale,
            rec2020.b() * b_scale,
        );
    }

    // 3. Exposure Multiplication
    if edits.exposure != 0.0 {
        let gain = 2.0f32.powf(edits.exposure);
        rec2020 = Color3::new(rec2020.r() * gain, rec2020.g() * gain, rec2020.b() * gain);
    }

    // 3b. Tone Recovery
    let (tr, tg, tb) = apply_tone_recovery(
        rec2020.r(),
        rec2020.g(),
        rec2020.b(),
        edits.highlights,
        edits.shadows,
        edits.whites,
        edits.blacks,
    );
    rec2020 = Color3::new(tr, tg, tb);

    // 4. Photochemical Recording Emulsion
    if edits.emulsion_archetype != EmulsionArchetype::None {
        let (er, eg, eb) = apply_recording_emulsion(
            rec2020.r(),
            rec2020.g(),
            rec2020.b(),
            edits.emulsion_archetype,
            edits.emulsion_exposure,
            edits.enlarger_filter,
        );
        rec2020 = Color3::new(er, eg, eb);
    }

    // 5. Tone Curves (Master + R + G + B) in Linear Rec.2020
    if let Some(lut) = curve_lut {
        if !lut.master.is_empty() {
            rec2020 = Color3::new(
                sample_curve_lut(&lut.master, rec2020.r()),
                sample_curve_lut(&lut.master, rec2020.g()),
                sample_curve_lut(&lut.master, rec2020.b()),
            );
        }
        if !lut.red.is_empty() {
            rec2020 = Color3::new(
                sample_curve_lut(&lut.red, rec2020.r()),
                rec2020.g(),
                rec2020.b(),
            );
        }
        if !lut.green.is_empty() {
            rec2020 = Color3::new(
                rec2020.r(),
                sample_curve_lut(&lut.green, rec2020.g()),
                rec2020.b(),
            );
        }
        if !lut.blue.is_empty() {
            rec2020 = Color3::new(
                rec2020.r(),
                rec2020.g(),
                sample_curve_lut(&lut.blue, rec2020.b()),
            );
        }
    }

    // 6. Color & Vibrance in Linear Rec.2020
    let (cr, cg, cb) = apply_color_adjustments(
        rec2020.r(),
        rec2020.g(),
        rec2020.b(),
        edits.vibrance,
        edits.saturation,
        edits.deep_blue,
        edits.foliage,
    );
    rec2020 = Color3::new(cr, cg, cb);

    // 7. Split Toning in Linear Rec.2020
    let (sr, sg, sb) = apply_split_toning(
        rec2020.r(),
        rec2020.g(),
        rec2020.b(),
        edits.split_toning_shadow_hue,
        edits.split_toning_shadow_sat,
        edits.split_toning_highlight_hue,
        edits.split_toning_highlight_sat,
        edits.split_toning_balance,
    );
    rec2020 = Color3::new(sr, sg, sb);

    // 8. Detail, Sharpening & Stochastic Emulsion Resynthesis (Optical Domain)
    if let Some([y_c, y_l, y_r, y_t, y_b]) = neighbors {
        if edits.sharpening_amount > 0.0 || edits.noise_reduction > 0.0 {
            let (dr, dg, db) = apply_detail_and_sharpening_rgb(
                rec2020.r(),
                rec2020.g(),
                rec2020.b(),
                y_c,
                y_l,
                y_r,
                y_t,
                y_b,
                edits.sharpening_amount,
                edits.sharpening_masking,
                edits.noise_reduction,
            );
            rec2020 = Color3::new(dr, dg, db);
        }

        if edits.emulsion_grain_mode != EmulsionGrainMode::Off {
            let (gr, gg, gb) = super::grain::apply_stochastic_emulsion_grain(
                rec2020.r(),
                rec2020.g(),
                rec2020.b(),
                y_c,
                y_l,
                y_r,
                y_t,
                y_b,
                y_c,
                y_l,
                edits.emulsion_grain_mode,
                edits.grain_scale,
                edits.emulsion_archetype,
            );
            rec2020 = Color3::new(gr, gg, gb);
        }
    }

    // 9. TERMINAL STAGE: AgX Filmic Tone Mapping Engine OR Camera Matching Profiles -> Linear sRGB
    let contrast_gain = if edits.contrast >= 0.0 {
        1.0 + edits.contrast / 100.0
    } else {
        1.0 + (edits.contrast / 100.0) * 0.60
    };

    let mut srgb_linear_out: Color3 = match edits.tone_profile {
        ColorProfileMode::AgxVivid => agx_tone_map_vivid(rec2020, contrast_gain),
        ColorProfileMode::AgxFilmic => agx_tone_map(rec2020, contrast_gain),
        mode => {
            let c_srgb = REC2020_TO_LINEAR_SRGB.multiply_vector(rec2020);
            let curve = match mode {
                ColorProfileMode::CameraPortrait => built_in_portrait_tone_curve(),
                ColorProfileMode::CameraLandscape => built_in_landscape_tone_curve(),
                ColorProfileMode::CameraVivid => built_in_vivid_tone_curve(),
                _ => built_in_standard_tone_curve(),
            };
            let apply_curve = |v: f32| -> f32 {
                let c = evaluate_tone_curve(v.max(0.0), &curve);
                if (contrast_gain - 1.0).abs() > 0.001 {
                    ((c - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0)
                } else {
                    c.clamp(0.0, 1.0)
                }
            };
            Color3::new(
                apply_curve(c_srgb.r()),
                apply_curve(c_srgb.g()),
                apply_curve(c_srgb.b()),
            )
        }
    };

    // 10. Curated Fine-Art Print Media Archetypes & Optical Enlarger Filtration in Linear Display Space
    if edits.print_media != PrintMediaMode::None {
        let (pr, pg, pb) = apply_print_media(
            srgb_linear_out.r(),
            srgb_linear_out.g(),
            srgb_linear_out.b(),
            edits.print_media,
            edits.enlarger_filter,
            edits.print_paper_grade,
            edits.print_base_warmth,
            edits.print_process_depth,
        );
        srgb_linear_out = Color3::new(pr, pg, pb);
    }

    // 11. Final Display Output: Display Gamma (Terminal Step)
    linear_rgb_to_srgb8(srgb_linear_out)
}

/// Develops an entire frame buffer into 8-bit RGBA display pixels.
/// Accepts either 16-bit half-float RGBA (for linear Rec.2020) or 8-bit RGBA (for sRGB).
pub fn develop_frame_pipeline(
    input_data: &[u8],
    is_linear_rec2020: bool,
    width: u32,
    height: u32,
    edits: &EditSettings,
    curve_lut: Option<&AllCurvesLutRecord>,
) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let total_pixels = w * h;
    let mut out_rgba = vec![0u8; total_pixels * 4];

    let needs_sharpening = edits.sharpening_amount > 0.0
        || edits.noise_reduction > 0.0
        || edits.emulsion_grain_mode != EmulsionGrainMode::Off;

    // Helper to extract a pixel in float [0..] from input buffer
    let get_input_pixel = |x: usize, y: usize| -> (f32, f32, f32) {
        if is_linear_rec2020 {
            let offset = (y * w + x) * 8; // 4 channels * 2 bytes
            if offset + 6 <= input_data.len() {
                let r16 = u16::from_le_bytes([input_data[offset], input_data[offset + 1]]);
                let g16 = u16::from_le_bytes([input_data[offset + 2], input_data[offset + 3]]);
                let b16 = u16::from_le_bytes([input_data[offset + 4], input_data[offset + 5]]);
                (f16_to_f32(r16), f16_to_f32(g16), f16_to_f32(b16))
            } else {
                (0.0, 0.0, 0.0)
            }
        } else {
            let offset = (y * w + x) * 4;
            if offset + 3 <= input_data.len() {
                (
                    input_data[offset] as f32 / 255.0,
                    input_data[offset + 1] as f32 / 255.0,
                    input_data[offset + 2] as f32 / 255.0,
                )
            } else {
                (0.0, 0.0, 0.0)
            }
        }
    };

    if !needs_sharpening {
        for y in 0..h {
            for x in 0..w {
                let (r, g, b) = get_input_pixel(x, y);
                let rgb = develop_pixel_pipeline(
                    r,
                    g,
                    b,
                    is_linear_rec2020,
                    edits,
                    curve_lut,
                    None,
                );
                let dst_idx = (y * w + x) * 4;
                out_rgba[dst_idx] = rgb[0];
                out_rgba[dst_idx + 1] = rgb[1];
                out_rgba[dst_idx + 2] = rgb[2];
                out_rgba[dst_idx + 3] = 255;
            }
        }
        return out_rgba;
    }

    // When sharpening is active, precompute display luma plane for neighborhood sampling
    let mut luma_plane = Vec::with_capacity(total_pixels);
    for y in 0..h {
        for x in 0..w {
            let (r, g, b) = get_input_pixel(x, y);
            luma_plane.push(evaluate_display_luma(r, g, b, is_linear_rec2020, edits));
        }
    }

    let sample_lum_bilinear = |px: f32, py: f32| -> f32 {
        let cx = px.clamp(0.0, (w - 1) as f32);
        let cy = py.clamp(0.0, (h - 1) as f32);
        let x0 = cx.floor() as usize;
        let x1 = (x0 + 1).min(w - 1);
        let y0 = cy.floor() as usize;
        let y1 = (y0 + 1).min(h - 1);
        let fx = cx - x0 as f32;
        let fy = cy - y0 as f32;

        let l00 = luma_plane[y0 * w + x0];
        let l10 = luma_plane[y0 * w + x1];
        let l01 = luma_plane[y1 * w + x0];
        let l11 = luma_plane[y1 * w + x1];

        let top = l00 * (1.0 - fx) + l10 * fx;
        let bottom = l01 * (1.0 - fx) + l11 * fx;
        top * (1.0 - fy) + bottom * fy
    };

    let r_step = edits.sharpening_radius.max(0.5);

    for y in 0..h {
        for x in 0..w {
            let (r, g, b) = get_input_pixel(x, y);
            let y_c = luma_plane[y * w + x];
            let y_l = sample_lum_bilinear(x as f32 - r_step, y as f32);
            let y_r = sample_lum_bilinear(x as f32 + r_step, y as f32);
            let y_t = sample_lum_bilinear(x as f32, y as f32 - r_step);
            let y_b = sample_lum_bilinear(x as f32, y as f32 + r_step);

            let rgb = develop_pixel_pipeline(
                r,
                g,
                b,
                is_linear_rec2020,
                edits,
                curve_lut,
                Some([y_c, y_l, y_r, y_t, y_b]),
            );
            let dst_idx = (y * w + x) * 4;
            out_rgba[dst_idx] = rgb[0];
            out_rgba[dst_idx + 1] = rgb[1];
            out_rgba[dst_idx + 2] = rgb[2];
            out_rgba[dst_idx + 3] = 255;
        }
    }

    out_rgba
}
