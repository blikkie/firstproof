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
                edits.enlarger_filter,
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

        // 4. Curated Fine-Art Print Media Archetypes & Optical Enlarger Filtration in Linear Display Space
        if edits.print_media != crate::PrintMediaMode::None {
            let (r_p, g_p, b_p) = crate::color::apply_print_media(
                r, g, b,
                edits.print_media,
                edits.enlarger_filter,
                edits.print_paper_grade,
                edits.print_base_warmth,
                edits.print_process_depth,
            );
            r = r_p;
            g = g_p;
            b = b_p;
        }
        
        // Store linear sRGB color values
        image.data[idx] = r.max(0.0);
        image.data[idx + 1] = g.max(0.0);
        image.data[idx + 2] = b.max(0.0);
    }
}

pub use crate::color::{
    apply_bilateral_tone_recovery_rgb, apply_color_adjustments, apply_detail_and_sharpening_rgb,
    apply_split_toning, apply_tone_recovery, sample_curve_lut,
};


