use crate::color::{
    Color3, REC2020_TO_LINEAR_SRGB,
};
use crate::EditSettings;
use crate::RenderTier;
use super::decode::DemosaicedImage;
use rayon::prelude::*;

#[inline(always)]
fn is_curve_active(pts: &[crate::CurvePointRecord]) -> bool {
    pts.len() > 2 || (pts.len() == 2 && ((pts[0].x - pts[0].y).abs() > 1e-4 || (pts[1].x - pts[1].y).abs() > 1e-4))
}

/// Applies user edits in 16-bit linear space using default archival export tier.
pub fn apply_edits(image: &mut DemosaicedImage, edits: &EditSettings) {
    apply_edits_with_tier(image, edits, RenderTier::ArchivalExport, false);
}

/// Applies user edits in 16-bit linear space with selectable RenderTier and optional chemical edge diffusion.
pub fn apply_edits_with_tier(
    image: &mut DemosaicedImage,
    edits: &EditSettings,
    _tier: RenderTier,
    chemical_edge_diffusion: bool,
) {
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

    let has_halation = edits.emulsion_archetype == crate::EmulsionArchetype::BaodingSilver100
        || (edits.emulsion_archetype == crate::EmulsionArchetype::RochesterCinema500
            && edits.remjet_backing == crate::RemjetBackingMode::Stripped);
    let needs_spatial_chem_diffusion = chemical_edge_diffusion && edits.emulsion_archetype != crate::EmulsionArchetype::None;
    let needs_spatial_pass = needs_spatial_chem_diffusion || has_halation;

    let max_dim = image.width.max(image.height) as f32;
    let r_inner = 1usize;
    let r_outer = ((max_dim * 0.016).round() as usize).max(16);

    if needs_spatial_pass {
        // Pass 1: Upstream optical exposure, WB, bilateral tone recovery, and recording emulsion
        for i in 0..total_pixels {
            let idx = i * 3;
            let mut r = image.data[idx] * r_scale * exposure_gain;
            let mut g = image.data[idx + 1] * g_scale * exposure_gain;
            let mut b = image.data[idx + 2] * b_scale * exposure_gain;

            if let Some(ref luma) = luma_plane {
                let x = i % image.width;
                let y = i / image.width;
                let w = image.width;
                let h = image.height;
                
                let x_l = x.saturating_sub(r_inner);
                let x_r = (x + r_inner).min(w - 1);
                let y_t = y.saturating_sub(r_inner);
                let y_b = (y + r_inner).min(h - 1);

                let ox_l = x.saturating_sub(r_outer);
                let ox_r = (x + r_outer).min(w - 1);
                let oy_t = y.saturating_sub(r_outer);
                let oy_b = (y + r_outer).min(h - 1);
                
                let y_c = luma[i];
                let y_neighbors = [
                    // Inner ring: 4 orthogonal + 4 diagonal (r_inner)
                    luma[y * w + x_l],
                    luma[y * w + x_r],
                    luma[y_t * w + x],
                    luma[y_b * w + x],
                    luma[y_t * w + x_l],
                    luma[y_t * w + x_r],
                    luma[y_b * w + x_l],
                    luma[y_b * w + x_r],
                    // Outer ring: 4 orthogonal + 4 diagonal (r_outer)
                    luma[y * w + ox_l],
                    luma[y * w + ox_r],
                    luma[oy_t * w + x],
                    luma[oy_b * w + x],
                    luma[oy_t * w + ox_l],
                    luma[oy_t * w + ox_r],
                    luma[oy_b * w + ox_l],
                    luma[oy_b * w + ox_r],
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

            let (r_em, g_em, b_em) = crate::color::apply_recording_emulsion(
                r, g, b,
                edits.emulsion_archetype,
                edits.emulsion_exposure,
                edits.enlarger_filter,
            );

            image.data[idx] = r_em;
            image.data[idx + 1] = g_em;
            image.data[idx + 2] = b_em;
        }

        // Pass 2: 2D Spatial Lateral Chemical Diffusion (Mackie Lines / Developer Exhaustion)
        if needs_spatial_chem_diffusion {
            apply_lateral_chemical_diffusion(&mut image.data, image.width, image.height, 0.15);
        }

        // Pass 2b: 2D Spatial Optical Halation Point-Spread Scatter
        if has_halation {
            crate::color::apply_point_spread_halation(
                &mut image.data,
                image.width,
                image.height,
                edits.emulsion_archetype,
                edits.remjet_backing,
            );
        }

        // Pass 3: Downstream Curves, Color Grading, Tone Mapping, and Print Media
        for i in 0..total_pixels {
            let idx = i * 3;
            let mut r = image.data[idx];
            let mut g = image.data[idx + 1];
            let mut b = image.data[idx + 2];

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
                        let c_rec2020 = Color3::new(r, g, b);
                        let c_srgb = REC2020_TO_LINEAR_SRGB.multiply_vector(c_rec2020);
                        let r_init = c_srgb.r().max(0.0);
                        let g_init = c_srgb.g().max(0.0);
                        let b_init = c_srgb.b().max(0.0);

                        let curve = match mode {
                            crate::ColorProfileMode::CameraPortrait => crate::color::dcp::DcpProfile::built_in_portrait_tone_curve(),
                            crate::ColorProfileMode::CameraLandscape => crate::color::dcp::DcpProfile::built_in_landscape_tone_curve(),
                            crate::ColorProfileMode::CameraVivid => crate::color::dcp::DcpProfile::built_in_vivid_tone_curve(),
                            _ => crate::color::dcp::DcpProfile::built_in_standard_tone_curve(),
                        };
                        
                        let apply_curve = |v: f32| -> f32 {
                            let curved = crate::color::evaluate_tone_curve(v, &curve);
                            if (contrast_gain - 1.0).abs() > 0.001 {
                                ((curved - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0)
                            } else {
                                curved.clamp(0.0, 1.0)
                            }
                        };

                        r = apply_curve(r_init);
                        g = apply_curve(g_init);
                        b = apply_curve(b_init);
                    }
                }
            } else {
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

            image.data[idx] = r.max(0.0);
            image.data[idx + 1] = g.max(0.0);
            image.data[idx + 2] = b.max(0.0);
        }
    } else {
        // Single unified pass when chemical edge diffusion is off
        for i in 0..total_pixels {
            let idx = i * 3;
            let mut r = image.data[idx] * r_scale * exposure_gain;
            let mut g = image.data[idx + 1] * g_scale * exposure_gain;
            let mut b = image.data[idx + 2] * b_scale * exposure_gain;
            
            if let Some(ref luma) = luma_plane {
                let x = i % image.width;
                let y = i / image.width;
                let w = image.width;
                let h = image.height;
                
                let x_l = x.saturating_sub(r_inner);
                let x_r = (x + r_inner).min(w - 1);
                let y_t = y.saturating_sub(r_inner);
                let y_b = (y + r_inner).min(h - 1);

                let ox_l = x.saturating_sub(r_outer);
                let ox_r = (x + r_outer).min(w - 1);
                let oy_t = y.saturating_sub(r_outer);
                let oy_b = (y + r_outer).min(h - 1);
                
                let y_c = luma[i];
                let y_neighbors = [
                    // Inner ring: 4 orthogonal + 4 diagonal (r_inner)
                    luma[y * w + x_l],
                    luma[y * w + x_r],
                    luma[y_t * w + x],
                    luma[y_b * w + x],
                    luma[y_t * w + x_l],
                    luma[y_t * w + x_r],
                    luma[y_b * w + x_l],
                    luma[y_b * w + x_r],
                    // Outer ring: 4 orthogonal + 4 diagonal (r_outer)
                    luma[y * w + ox_l],
                    luma[y * w + ox_r],
                    luma[oy_t * w + x],
                    luma[oy_b * w + x],
                    luma[oy_t * w + ox_l],
                    luma[oy_t * w + ox_r],
                    luma[oy_b * w + ox_l],
                    luma[oy_b * w + ox_r],
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
                        let c_rec2020 = Color3::new(r, g, b);
                        let c_srgb = REC2020_TO_LINEAR_SRGB.multiply_vector(c_rec2020);
                        let r_init = c_srgb.r().max(0.0);
                        let g_init = c_srgb.g().max(0.0);
                        let b_init = c_srgb.b().max(0.0);

                        let curve = match mode {
                            crate::ColorProfileMode::CameraPortrait => crate::color::dcp::DcpProfile::built_in_portrait_tone_curve(),
                            crate::ColorProfileMode::CameraLandscape => crate::color::dcp::DcpProfile::built_in_landscape_tone_curve(),
                            crate::ColorProfileMode::CameraVivid => crate::color::dcp::DcpProfile::built_in_vivid_tone_curve(),
                            _ => crate::color::dcp::DcpProfile::built_in_standard_tone_curve(),
                        };
                        
                        let apply_curve = |v: f32| -> f32 {
                            let curved = crate::color::evaluate_tone_curve(v, &curve);
                            if (contrast_gain - 1.0).abs() > 0.001 {
                                ((curved - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0)
                            } else {
                                curved.clamp(0.0, 1.0)
                            }
                        };

                        r = apply_curve(r_init);
                        g = apply_curve(g_init);
                        b = apply_curve(b_init);
                    }
                }
            } else {
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

            image.data[idx] = r.max(0.0);
            image.data[idx + 1] = g.max(0.0);
            image.data[idx + 2] = b.max(0.0);
        }
    }
}

/// Photochemical Lateral Chemical Diffusion (Mackie Lines / Developer Exhaustion)
///
/// Models 2D spatial diffusion of development inhibitor by-products (bromide ions)
/// across contrasting density borders in Linear Rec.2020 space.
pub fn apply_lateral_chemical_diffusion(
    data: &mut [f32],
    width: usize,
    height: usize,
    strength: f32,
) {
    if strength <= 0.0 || width < 5 || height < 5 || data.len() < width * height * 3 {
        return;
    }

    let total_pixels = width * height;
    let mut luma = Vec::with_capacity(total_pixels);
    for i in 0..total_pixels {
        let idx = i * 3;
        let y = 0.2627 * data[idx] + 0.6780 * data[idx + 1] + 0.0593 * data[idx + 2];
        luma.push(y);
    }

    let mut h_blur = vec![0.0f32; total_pixels];
    let w = width;
    let h = height;

    // Horizontal pass: 5-tap binomial [1, 4, 6, 4, 1] / 16
    h_blur.par_chunks_exact_mut(w).enumerate().for_each(|(y, row)| {
        let row_start = y * w;
        for x in 0..w {
            let x0 = x.saturating_sub(2);
            let x1 = x.saturating_sub(1);
            let x2 = x;
            let x3 = (x + 1).min(w - 1);
            let x4 = (x + 2).min(w - 1);

            let v = (luma[row_start + x0]
                + 4.0 * luma[row_start + x1]
                + 6.0 * luma[row_start + x2]
                + 4.0 * luma[row_start + x3]
                + luma[row_start + x4])
                / 16.0;
            row[x] = v;
        }
    });

    let mut diff_luma = vec![0.0f32; total_pixels];
    diff_luma.par_chunks_exact_mut(w).enumerate().for_each(|(y, row)| {
        let y0 = y.saturating_sub(2);
        let y1 = y.saturating_sub(1);
        let y2 = y;
        let y3 = (y + 1).min(h - 1);
        let y4 = (y + 2).min(h - 1);

        for x in 0..w {
            let v = (h_blur[y0 * w + x]
                + 4.0 * h_blur[y1 * w + x]
                + 6.0 * h_blur[y2 * w + x]
                + 4.0 * h_blur[y3 * w + x]
                + h_blur[y4 * w + x])
                / 16.0;
            row[x] = v;
        }
    });

    let kappa = strength.clamp(0.0, 0.35);
    data.par_chunks_exact_mut(w * 3).enumerate().for_each(|(y, row_slice)| {
        let row_start = y * w;
        for x in 0..w {
            let p_idx = row_start + x;
            let y_c = luma[p_idx];
            let y_diff = diff_luma[p_idx];

            let contrast_diff = y_c - y_diff;
            let denom = y_diff + 0.18;
            let factor = (1.0 + kappa * (contrast_diff / denom)).clamp(0.70, 1.35);

            let out_idx = x * 3;
            row_slice[out_idx] = (row_slice[out_idx] * factor).max(0.0);
            row_slice[out_idx + 1] = (row_slice[out_idx + 1] * factor).max(0.0);
            row_slice[out_idx + 2] = (row_slice[out_idx + 2] * factor).max(0.0);
        }
    });
}

pub use crate::color::{
    apply_bilateral_tone_recovery_rgb, apply_color_adjustments, apply_detail_and_sharpening_rgb,
    apply_split_toning, apply_tone_recovery, sample_curve_lut,
};


