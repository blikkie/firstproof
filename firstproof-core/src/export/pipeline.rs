use crate::color::{
    Color3, REC2020_TO_LINEAR_SRGB, agx_tone_map_vivid, apply_print_media,
    apply_recording_emulsion,
};
pub use crate::color::apply_split_toning;
use crate::EditSettings;
use crate::RenderTier;
use super::decode::DemosaicedImage;

#[inline(always)]
fn is_curve_active(pts: &[crate::CurvePointRecord]) -> bool {
    pts.len() > 2 || (pts.len() == 2 && ((pts[0].x - pts[0].y).abs() > 1e-4 || (pts[1].x - pts[1].y).abs() > 1e-4))
}

/// Applies user edits in 16-bit linear space using default archival export tier.
pub fn apply_edits(image: &mut DemosaicedImage, edits: &EditSettings) {
    apply_edits_with_tier(image, edits, RenderTier::ArchivalExport);
}

/// Applies scene-referred upstream optical adjustments in Wide-Gamut Linear Rec.2020 space:
/// 1. White Balance & Exposure
/// 2. Lens Vignetting Radial Illumination Gain
/// 3. Bilateral Tone Recovery (Highlights, Shadows, Whites, Blacks)
/// 4. Recording Emulsion Sensitometry & Point-Spread Halation
/// 5. Wide-Gamut Tone Curves (Master + RGB)
/// 6. Color Adjustments (Vibrance, Saturation, Deep Blue, Foliage)
/// 7. Split Toning & Color Grading
pub fn apply_scene_referred_edits_with_tier(
    image: &mut DemosaicedImage,
    edits: &EditSettings,
    lens_coeffs: Option<&crate::LensCorrectionCoefficientsRecord>,
    _tier: RenderTier,
) {
    let total_exposure = edits.exposure;
    let exposure_gain = 2.0f32.powf(total_exposure);

    let base_temp = if image.is_raw {
        image.as_shot_temperature.unwrap_or(5500.0)
    } else {
        5500.0
    };
    let base_tint = if image.is_raw {
        image.as_shot_tint.unwrap_or(0.0)
    } else {
        0.0
    };

    let (r_scale, g_scale, b_scale) = if (edits.temperature - base_temp).abs() >= 1.0 || (edits.tint - base_tint).abs() >= 0.1 {
        let temp_norm = (edits.temperature / base_temp).clamp(0.25, 4.0);
        let mut r = temp_norm.powf(0.75);
        let mut b = (1.0 / temp_norm).powf(0.75);

        let tint_factor = (edits.tint - base_tint) * 0.002;
        let g = (1.0 - tint_factor).max(0.01);
        r = (r + tint_factor * 0.5).max(0.01);
        b = (b + tint_factor * 0.5).max(0.01);
        (r, g, b)
    } else {
        (1.0, 1.0, 1.0)
    };

    let (has_vignetting, vig_k1, vig_k2, vig_k3, vig_k4) = if let Some(c) = lens_coeffs {
        if edits.vignetting_correction_enabled && c.has_vignetting {
            let k1 = c.vignetting.first().copied().unwrap_or(0.0);
            let k2 = c.vignetting.get(1).copied().unwrap_or(0.0);
            let k3 = c.vignetting.get(2).copied().unwrap_or(0.0);
            let k4 = c.vignetting.get(3).copied().unwrap_or(0.0);
            (true, k1, k2, k3, k4)
        } else {
            (false, 0.0, 0.0, 0.0, 0.0)
        }
    } else {
        (false, 0.0, 0.0, 0.0, 0.0)
    };

    let curve_lut = crate::curve::evaluate_curves_lut_ref(&edits.curves);
    let has_master_curve = is_curve_active(&edits.curves.master);
    let has_red_curve = is_curve_active(&edits.curves.red);
    let has_green_curve = is_curve_active(&edits.curves.green);
    let has_blue_curve = is_curve_active(&edits.curves.blue);
    let has_any_curve = has_master_curve || has_red_curve || has_green_curve || has_blue_curve;

    let w = image.width;
    let h = image.height;
    let total_pixels = w * h;
    if total_pixels == 0 {
        return;
    }

    let aspect_ratio = (w as f32) / (h as f32).max(1.0);
    let norm_denom = aspect_ratio * aspect_ratio + 1.0;

    // Pass 0: Apply White Balance, Exposure, and Optical Vignetting in Linear Rec.2020
    for i in 0..total_pixels {
        let idx = i * 3;
        let mut r = image.data[idx] * r_scale * exposure_gain;
        let mut g = image.data[idx + 1] * g_scale * exposure_gain;
        let mut b = image.data[idx + 2] * b_scale * exposure_gain;

        if has_vignetting {
            let x = i % w;
            let y = i / w;
            let u = (x as f32 + 0.5) / (w as f32).max(1.0);
            let v = (y as f32 + 0.5) / (h as f32).max(1.0);
            let xc = (u - 0.5) * 2.0 * aspect_ratio;
            let yc = (v - 0.5) * 2.0;
            let r2 = (xc * xc + yc * yc) / norm_denom;
            let r4 = r2 * r2;
            let r6 = r4 * r2;
            let r8 = r4 * r4;
            let poly = 1.0 + vig_k1 * r2 + vig_k2 * r4 + vig_k3 * r6 + vig_k4 * r8;
            let vig_gain = if poly < 1.0 { 1.0 / poly.max(0.05) } else { poly };
            r *= vig_gain;
            g *= vig_gain;
            b *= vig_gain;
        }

        image.data[idx] = r;
        image.data[idx + 1] = g;
        image.data[idx + 2] = b;
    }

    // Pass 1: Bilateral Tone Recovery (Highlights, Shadows, Whites, Blacks)
    let needs_tone_recovery = edits.highlights.abs() > 0.001
        || edits.shadows.abs() > 0.001
        || edits.whites.abs() > 0.001
        || edits.blacks.abs() > 0.001;

    if needs_tone_recovery {
        let max_dim = w.max(h) as f32;
        let r_inner = ((max_dim * 0.003).round() as usize).max(3);
        let r_outer = ((max_dim * 0.024).round() as usize).max(24);
        let r_inner_diag = ((r_inner as f32 * 0.7071).round() as usize).max(2);
        let r_outer_diag = ((r_outer as f32 * 0.7071).round() as usize).max(17);

        let mut luma_plane = Vec::with_capacity(total_pixels);
        for i in 0..total_pixels {
            let idx = i * 3;
            let r = image.data[idx];
            let g = image.data[idx + 1];
            let b = image.data[idx + 2];
            luma_plane.push(0.2627 * r + 0.6780 * g + 0.0593 * b);
        }

        for i in 0..total_pixels {
            let idx = i * 3;
            let r = image.data[idx];
            let g = image.data[idx + 1];
            let b = image.data[idx + 2];
            let x = i % w;
            let y = i / w;

            let x_l1 = x.saturating_sub(r_inner);
            let x_r1 = (x + r_inner).min(w - 1);
            let y_t1 = y.saturating_sub(r_inner);
            let y_b1 = (y + r_inner).min(h - 1);

            let x_tl1 = x.saturating_sub(r_inner_diag);
            let x_tr1 = (x + r_inner_diag).min(w - 1);
            let y_tl1 = y.saturating_sub(r_inner_diag);
            let y_bl1 = (y + r_inner_diag).min(h - 1);

            let x_l2 = x.saturating_sub(r_outer);
            let x_r2 = (x + r_outer).min(w - 1);
            let y_t2 = y.saturating_sub(r_outer);
            let y_b2 = (y + r_outer).min(h - 1);

            let x_tl2 = x.saturating_sub(r_outer_diag);
            let x_tr2 = (x + r_outer_diag).min(w - 1);
            let y_tl2 = y.saturating_sub(r_outer_diag);
            let y_bl2 = (y + r_outer_diag).min(h - 1);

            let y_c = luma_plane[i];
            let neighbors = [
                luma_plane[y * w + x_l1],
                luma_plane[y * w + x_r1],
                luma_plane[y_t1 * w + x],
                luma_plane[y_b1 * w + x],
                luma_plane[y_tl1 * w + x_tl1],
                luma_plane[y_tl1 * w + x_tr1],
                luma_plane[y_bl1 * w + x_tl1],
                luma_plane[y_bl1 * w + x_tr1],
                luma_plane[y * w + x_l2],
                luma_plane[y * w + x_r2],
                luma_plane[y_t2 * w + x],
                luma_plane[y_b2 * w + x],
                luma_plane[y_tl2 * w + x_tl2],
                luma_plane[y_tl2 * w + x_tr2],
                luma_plane[y_bl2 * w + x_tl2],
                luma_plane[y_bl2 * w + x_tr2],
            ];

            let (r_rec, g_rec, b_rec) = apply_bilateral_tone_recovery_rgb(
                r, g, b,
                y_c,
                &neighbors,
                edits.highlights,
                edits.shadows,
                edits.whites,
                edits.blacks,
            );
            image.data[idx] = r_rec;
            image.data[idx + 1] = g_rec;
            image.data[idx + 2] = b_rec;
        }
    }

    // Pass 2: Recording Emulsion Sensitometry & Point-Spread Halation
    if edits.emulsion_archetype != crate::EmulsionArchetype::None {
        for i in 0..total_pixels {
            let idx = i * 3;
            let (re, ge, be) = apply_recording_emulsion(
                image.data[idx],
                image.data[idx + 1],
                image.data[idx + 2],
                edits.emulsion_archetype,
                edits.emulsion_exposure,
                edits.enlarger_filter,
            );
            image.data[idx] = re;
            image.data[idx + 1] = ge;
            image.data[idx + 2] = be;
        }

        if edits.emulsion_archetype == crate::EmulsionArchetype::RochesterCinema500
            && edits.remjet_backing == crate::RemjetBackingMode::Stripped
        {
            let max_dim = w.max(h) as f32;
            let r_hal = ((max_dim * 0.008).round() as usize).max(2);
            let r_hal_diag = ((r_hal as f32 * 0.7071).round() as usize).max(1);
            let hal_scale = edits.emulsion_format.magnification_factor().sqrt();
            let pre_hal = image.data.clone();

            for i in 0..total_pixels {
                let x = i % w;
                let y = i / w;

                let x_l = x.saturating_sub(r_hal);
                let x_r = (x + r_hal).min(w - 1);
                let y_t = y.saturating_sub(r_hal);
                let y_b = (y + r_hal).min(h - 1);

                let x_dl = x.saturating_sub(r_hal_diag);
                let x_dr = (x + r_hal_diag).min(w - 1);
                let y_dt = y.saturating_sub(r_hal_diag);
                let y_db = (y + r_hal_diag).min(h - 1);

                let sample_excess = |px: usize, py: usize| -> f32 {
                    let s_idx = (py * w + px) * 3;
                    let sr = pre_hal[s_idx];
                    let sg = pre_hal[s_idx + 1];
                    let sb = pre_hal[s_idx + 2];
                    let sy = 0.2627 * sr + 0.6780 * sg + 0.0593 * sb;
                    (sy - 0.55).max(0.0)
                };

                let e_ortho = sample_excess(x_l, y)
                    + sample_excess(x_r, y)
                    + sample_excess(x, y_t)
                    + sample_excess(x, y_b);
                let e_diag = sample_excess(x_dl, y_dt)
                    + sample_excess(x_dr, y_dt)
                    + sample_excess(x_dl, y_db)
                    + sample_excess(x_dr, y_db);
                let bloom = (0.15 * e_ortho + 0.10 * e_diag) * hal_scale;

                if bloom > 0.0 {
                    let idx = i * 3;
                    image.data[idx] += bloom * 0.85;
                    image.data[idx + 1] += bloom * 0.18;
                    image.data[idx + 2] += bloom * 0.02;
                }
            }
        }
    }

    // Pass 3: Unified per-pixel Wide-Gamut Tone Curves, Color Adjustments, and Split Toning in Linear Rec.2020
    for i in 0..total_pixels {
        let idx = i * 3;
        let mut r = image.data[idx];
        let mut g = image.data[idx + 1];
        let mut b = image.data[idx + 2];

        // 3a. Wide-Gamut Tone Curves (Master + Per-Channel RGB)
        if has_any_curve {
            let eval_lut = |val: f32, lut: &[f32]| -> f32 {
                let display_x = val.max(0.0).powf(1.0 / 2.2);
                if display_x <= 1.0 {
                    let pos = display_x * 255.0;
                    let idx0 = (pos.floor() as usize).min(255);
                    let idx1 = (idx0 + 1).min(255);
                    let frac = pos - (idx0 as f32);
                    let curved_display = lut[idx0] * (1.0 - frac) + lut[idx1] * frac;
                    curved_display.max(0.0).powf(2.2)
                } else {
                    let top_gain = lut[255].max(0.0).powf(2.2);
                    val * top_gain
                }
            };

            if has_master_curve {
                let lum_in = 0.2627 * r + 0.6780 * g + 0.0593 * b;
                if lum_in > 1e-6 {
                    let lum_out = eval_lut(lum_in, &curve_lut.master);
                    let m_gain = lum_out / lum_in;
                    r *= m_gain;
                    g *= m_gain;
                    b *= m_gain;
                } else {
                    let lift = eval_lut(0.0, &curve_lut.master);
                    r += lift;
                    g += lift;
                    b += lift;
                }
            }

            if has_red_curve {
                r = eval_lut(r, &curve_lut.red);
            }
            if has_green_curve {
                g = eval_lut(g, &curve_lut.green);
            }
            if has_blue_curve {
                b = eval_lut(b, &curve_lut.blue);
            }
        }

        // 3b. Color Adjustments (Vibrance, Saturation, Deep Blue, Foliage) in Linear Rec.2020
        let (r_col, g_col, b_col) = apply_color_adjustments(
            r, g, b,
            edits.vibrance,
            edits.saturation,
            edits.deep_blue,
            edits.foliage,
        );

        // 3c. Split Toning & Color Grading in Linear Rec.2020
        let (r_st, g_st, b_st) = apply_split_toning(
            r_col, g_col, b_col,
            edits.split_toning_shadow_hue,
            edits.split_toning_shadow_sat,
            edits.split_toning_highlight_hue,
            edits.split_toning_highlight_sat,
            edits.split_toning_balance,
        );

        image.data[idx] = r_st.max(0.0);
        image.data[idx + 1] = g_st.max(0.0);
        image.data[idx + 2] = b_st.max(0.0);
    }
}

/// Applies user edits in 16-bit linear space with selectable RenderTier,
/// finishing with terminal AgX Filmic tone mapping and fine-art print media in Linear sRGB.
pub fn apply_edits_with_tier(
    image: &mut DemosaicedImage,
    edits: &EditSettings,
    tier: RenderTier,
) {
    apply_scene_referred_edits_with_tier(image, edits, None, tier);

    let w = image.width;
    let h = image.height;
    let total_pixels = w * h;
    if total_pixels == 0 {
        return;
    }

    let has_spatial_detail = edits.defringe_amount > 0.0
        || edits.sharpening_amount > 0.0
        || edits.noise_reduction > 0.0
        || edits.emulsion_grain_mode != crate::EmulsionGrainMode::Off;

    if has_spatial_detail {
        let mut luma_plane = Vec::with_capacity(total_pixels);
        for i in 0..total_pixels {
            let idx = i * 3;
            let r = image.data[idx];
            let g = image.data[idx + 1];
            let b = image.data[idx + 2];
            luma_plane.push(0.2627 * r + 0.6780 * g + 0.0593 * b);
        }

        let r_step = if edits.sharpening_amount > 0.0 || edits.noise_reduction > 0.0 {
            ((edits.sharpening_radius * image.baseline_sharpness).round() as usize).max(1)
        } else {
            1
        };

        let grain_mag = edits.emulsion_format.magnification_factor();
        let s_format = grain_mag.sqrt();
        let cluster_size = (edits.grain_size * s_format).clamp(0.5, 3.5);
        let effective_grain_amount = edits.grain_amount * grain_mag.powf(0.25);

        for i in 0..total_pixels {
            let idx = i * 3;
            let mut r = image.data[idx];
            let mut g = image.data[idx + 1];
            let mut b = image.data[idx + 2];
            let x = i % w;
            let y = i / w;
            let y_c = luma_plane[i];

            if edits.defringe_amount > 0.0 {
                let xl1 = x.saturating_sub(1);
                let xr1 = (x + 1).min(w - 1);
                let yt1 = y.saturating_sub(1);
                let yb1 = (y + 1).min(h - 1);
                let y_max_neighbor = luma_plane[y * w + xl1]
                    .max(luma_plane[y * w + xr1])
                    .max(luma_plane[yt1 * w + x])
                    .max(luma_plane[yb1 * w + x])
                    .max(luma_plane[yt1 * w + xl1])
                    .max(luma_plane[yt1 * w + xr1])
                    .max(luma_plane[yb1 * w + xl1])
                    .max(luma_plane[yb1 * w + xr1]);
                let (rd, gd, bd) = firstproof_engine::color::apply_defringe_rgb(
                    r, g, b, y_c, y_max_neighbor, edits.defringe_amount,
                );
                r = rd;
                g = gd;
                b = bd;
            }

            if edits.sharpening_amount > 0.0
                || edits.noise_reduction > 0.0
                || edits.emulsion_grain_mode != crate::EmulsionGrainMode::Off
            {
                let xl = x.saturating_sub(r_step);
                let xr = (x + r_step).min(w - 1);
                let yt = y.saturating_sub(r_step);
                let yb = (y + r_step).min(h - 1);

                let y_l = luma_plane[y * w + xl];
                let y_r = luma_plane[y * w + xr];
                let y_t = luma_plane[yt * w + x];
                let y_b = luma_plane[yb * w + x];
                let y_tl = luma_plane[yt * w + xl];
                let y_tr = luma_plane[yt * w + xr];
                let y_bl = luma_plane[yb * w + xl];
                let y_br = luma_plane[yb * w + xr];

                if edits.sharpening_amount > 0.0 || edits.noise_reduction > 0.0 {
                    let (rs, gs, bs) = apply_detail_and_sharpening_rgb(
                        r, g, b,
                        y_c, y_l, y_r, y_t, y_b, y_tl, y_tr, y_bl, y_br,
                        edits.sharpening_amount,
                        edits.sharpening_masking,
                        edits.noise_reduction,
                    );
                    r = rs;
                    g = gs;
                    b = bs;
                }

                if edits.emulsion_grain_mode != crate::EmulsionGrainMode::Off {
                    let (rg, gg, bg) = firstproof_engine::color::grain::apply_stochastic_emulsion_grain(
                        r, g, b,
                        y_c, y_l, y_r, y_t, y_b, y_tl, y_tr, y_bl, y_br,
                        x as f32, y as f32,
                        edits.emulsion_grain_mode,
                        cluster_size,
                        effective_grain_amount,
                        edits.emulsion_archetype,
                    );
                    r = rg;
                    g = gg;
                    b = bg;
                }
            }

            image.data[idx] = r;
            image.data[idx + 1] = g;
            image.data[idx + 2] = b;
        }
    }

    let contrast_gain = if edits.contrast >= 0.0 {
        1.0 + (edits.contrast / 100.0)
    } else {
        1.0 + (edits.contrast / 100.0) * 0.60
    };

    for i in 0..total_pixels {
        let idx = i * 3;
        let (r_out, g_out, b_out) = apply_terminal_tone_map_and_print_media(
            image.data[idx],
            image.data[idx + 1],
            image.data[idx + 2],
            image.is_raw,
            contrast_gain,
            edits,
        );
        image.data[idx] = r_out;
        image.data[idx + 1] = g_out;
        image.data[idx + 2] = b_out;
    }
}

/// Terminal tone mapping (Canonical AgX Filmic for RAW, Linear sRGB contrast for non-RAW)
/// followed by Fine-Art Print Media simulation in Linear sRGB.
#[inline]
pub fn apply_terminal_tone_map_and_print_media(
    r: f32,
    g: f32,
    b: f32,
    is_raw: bool,
    contrast_gain: f32,
    edits: &EditSettings,
) -> (f32, f32, f32) {
    let (mut r_out, mut g_out, mut b_out) = if is_raw {
        let c_rec2020 = Color3::new(r, g, b);
        let mapped = agx_tone_map_vivid(c_rec2020, contrast_gain);
        (mapped.r().clamp(0.0, 1.0), mapped.g().clamp(0.0, 1.0), mapped.b().clamp(0.0, 1.0))
    } else {
        let c_rec2020 = Color3::new(r, g, b);
        let c_srgb = REC2020_TO_LINEAR_SRGB.multiply_vector(c_rec2020);
        let apply_c = |v: f32| -> f32 {
            let v_c = v.max(0.0);
            if (contrast_gain - 1.0).abs() > 0.001 {
                ((v_c - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0)
            } else {
                v_c.clamp(0.0, 1.0)
            }
        };
        (apply_c(c_srgb.r()), apply_c(c_srgb.g()), apply_c(c_srgb.b()))
    };

    if edits.print_media != crate::PrintMediaMode::None {
        let (rp, gp, bp) = apply_print_media(
            r_out, g_out, b_out,
            edits.print_media,
            edits.enlarger_filter,
            edits.print_paper_grade,
            edits.print_process_depth,
        );
        r_out = rp;
        g_out = gp;
        b_out = bp;
    }

    (r_out.max(0.0), g_out.max(0.0), b_out.max(0.0))
}

pub use firstproof_engine::color::{
    apply_bilateral_tone_recovery_rgb, apply_color_adjustments, apply_detail_and_sharpening_rgb,
    apply_tone_recovery,
};
