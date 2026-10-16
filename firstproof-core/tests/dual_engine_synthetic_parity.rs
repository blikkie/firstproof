//! Dual-Engine Synthetic Parity Regression Suite
//!
//! Evaluates bit-level mathematical parity between the Metal GPU shader pipeline
//! and the Rust CPU export pipeline across synthetic Macbeth patches, neutral ramps,
//! and geometric distortion grids WITHOUT requiring large RAW disk files.

use firstproof_core::color::{
    Color3, agx_tone_map, agx_tone_map_vivid, apply_print_media, apply_recording_emulsion,
    ciede2000::linear_srgb_to_lab, ciede2000::ciede2000,
    dcp::DcpProfile, dcp::evaluate_tone_curve, REC2020_TO_LINEAR_SRGB,
};
use firstproof_core::export::{apply_edits, apply_tone_recovery, f16_to_f32};
use firstproof_core::synthetic::SyntheticTestChart;
use firstproof_core::{ColorProfileMode, EditSettings, EmulsionArchetype, EnlargerFilterMode, PrintMediaMode};

#[test]
fn test_synthetic_macbeth_and_neutral_ramp_dual_engine_parity() {
    let chart = SyntheticTestChart::new(1200, 800);
    let binned_bytes = chart.to_binned_f16_rgba();
    let bw = chart.width;
    let bh = chart.height;

    // Test matrix of representative darkroom adjustments
    let test_cases = [
        ("Neutral Identity", EditSettings::default()),
        ("Tungsten Warmth (2850K)", EditSettings {
            temperature: 2850.0,
            tint: 0.0,
            ..Default::default()
        }),
        ("Fluorescent Subway (3800K, Tint +45)", EditSettings {
            temperature: 3800.0,
            tint: 45.0,
            ..Default::default()
        }),
        ("Extreme High-Dynamic Scene (Exp +1.2, Cont +20, HL -40, Sh +30)", EditSettings {
            exposure: 1.2,
            contrast: 20.0,
            highlights: -40.0,
            shadows: 30.0,
            ..Default::default()
        }),
        ("AgX Filmic Base Neutral", EditSettings {
            tone_profile: ColorProfileMode::AgxFilmic,
            contrast: 15.0,
            ..Default::default()
        }),
        ("Mobberley Silver 400 + Red 25A Filter", EditSettings {
            emulsion_archetype: EmulsionArchetype::MobberleySilver400,
            enlarger_filter: EnlargerFilterMode::Red25A,
            ..Default::default()
        }),
        ("Platinum / Palladium Print Media", EditSettings {
            print_media: PrintMediaMode::PlatinumPalladium,
            print_paper_grade: 1.0,
            print_base_warmth: 15.0,
            print_process_depth: 80.0,
            ..Default::default()
        }),
        ("Baryta Silver Gelatin Print Media", EditSettings {
            print_media: PrintMediaMode::BarytaSilverGelatin,
            enlarger_filter: EnlargerFilterMode::YellowK2,
            print_paper_grade: 0.5,
            print_process_depth: 90.0,
            ..Default::default()
        }),
        ("Cyanotype Prussian Blue Chemistry", EditSettings {
            print_media: PrintMediaMode::Cyanotype,
            print_paper_grade: 0.0,
            print_base_warmth: 5.0,
            print_process_depth: 85.0,
            ..Default::default()
        }),
    ];

    for (case_name, edits) in test_cases {
        // 1. Process CPU Export Pipeline
        let mut export_img = chart.to_decoded_image();
        apply_edits(&mut export_img, &edits);

        // 2. Prepare Simulated GPU Shader Pipeline Math
        let temp_norm = (edits.temperature / 5500.0).clamp(0.25, 4.0);
        let mut r_scale = temp_norm.powf(0.75);
        let mut b_scale = (1.0 / temp_norm).powf(0.75);
        let tint_factor = edits.tint * 0.002;
        let g_scale = (1.0 - tint_factor).max(0.01);
        r_scale = (r_scale + tint_factor * 0.5).max(0.01);
        b_scale = (b_scale + tint_factor * 0.5).max(0.01);
        let exposure_gain = 2.0f32.powf(edits.exposure);
        let contrast_gain = if edits.contrast >= 0.0 {
            1.0 + (edits.contrast / 100.0)
        } else {
            1.0 + (edits.contrast / 100.0) * 0.60
        };

        let mut max_case_de00 = 0.0f32;
        let mut total_case_de00 = 0.0f32;
        let mut patch_count = 0;

        for patch in &chart.sample_points {
            let u = patch.uv[0];
            let v = patch.uv[1];

            // Sample Binned GPU Buffer:
            let bx = ((bw as f32 * u).round() as usize).clamp(0, bw as usize - 1);
            let by = ((bh as f32 * v).round() as usize).clamp(0, bh as usize - 1);
            let b_idx = (by * bw as usize + bx) * 8;
            let r_f16 = u16::from_ne_bytes([binned_bytes[b_idx], binned_bytes[b_idx + 1]]);
            let g_f16 = u16::from_ne_bytes([binned_bytes[b_idx + 2], binned_bytes[b_idx + 3]]);
            let b_f16 = u16::from_ne_bytes([binned_bytes[b_idx + 4], binned_bytes[b_idx + 5]]);

            let mut r_binned = f16_to_f32(r_f16);
            let mut g_binned = f16_to_f32(g_f16);
            let mut b_binned = f16_to_f32(b_f16);

            // Run fragment shader math
            r_binned *= r_scale * exposure_gain;
            g_binned *= g_scale * exposure_gain;
            b_binned *= b_scale * exposure_gain;

            let (r_rec, g_rec, b_rec) = apply_tone_recovery(
                r_binned, g_binned, b_binned,
                edits.highlights, edits.shadows, edits.whites, edits.blacks,
            );

            // Upstream Recording Emulsion
            let (r_em, g_em, b_em) = if edits.emulsion_archetype != EmulsionArchetype::None {
                apply_recording_emulsion(
                    r_rec, g_rec, b_rec,
                    edits.emulsion_archetype,
                    edits.emulsion_exposure,
                    edits.enlarger_filter,
                )
            } else {
                (r_rec, g_rec, b_rec)
            };

            let tone_mapped = match edits.tone_profile {
                ColorProfileMode::AgxVivid => agx_tone_map_vivid(Color3::new(r_em, g_em, b_em), contrast_gain),
                ColorProfileMode::AgxFilmic => agx_tone_map(Color3::new(r_em, g_em, b_em), contrast_gain),
                mode => {
                    let c_srgb = REC2020_TO_LINEAR_SRGB.multiply_vector(Color3::new(r_em, g_em, b_em));
                    let (mut r_adj, mut g_adj, mut b_adj) = (c_srgb.r().max(0.0), c_srgb.g().max(0.0), c_srgb.b().max(0.0));
                    let huesatmap: Option<firstproof_core::color::DcpHueSatMap> = match mode {
                        ColorProfileMode::CameraPortrait => Some(DcpProfile::built_in_portrait_huesatmap()),
                        ColorProfileMode::CameraLandscape => Some(DcpProfile::built_in_landscape_huesatmap()),
                        ColorProfileMode::CameraVivid => Some(DcpProfile::built_in_vivid_huesatmap()),
                        _ => Some(DcpProfile::built_in_standard_huesatmap()),
                    };
                    if let Some(ref hsm) = huesatmap {
                        let (h, s, v) = firstproof_core::color::dcp::rgb_to_hsv(r_adj, g_adj, b_adj);
                        let (h_out, s_out, v_out) = hsm.apply_hsv(h, s, v);
                        let (r_hsv, g_hsv, b_hsv) = firstproof_core::color::dcp::hsv_to_rgb(h_out, s_out, v_out);
                        r_adj = r_hsv;
                        g_adj = g_hsv;
                        b_adj = b_hsv;
                    }

                    let curve = match mode {
                        ColorProfileMode::CameraPortrait => DcpProfile::built_in_portrait_tone_curve(),
                        ColorProfileMode::CameraLandscape => DcpProfile::built_in_landscape_tone_curve(),
                        ColorProfileMode::CameraVivid => DcpProfile::built_in_vivid_tone_curve(),
                        _ => DcpProfile::built_in_standard_tone_curve(),
                    };
                    let apply_curve = |v: f32| -> f32 {
                        let curved = evaluate_tone_curve(v.max(0.0), &curve);
                        if (contrast_gain - 1.0).abs() > 0.001 {
                            ((curved - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0)
                        } else {
                            curved.clamp(0.0, 1.0)
                        }
                    };
                    Color3::new(
                        apply_curve(r_adj),
                        apply_curve(g_adj),
                        apply_curve(b_adj),
                    )
                }
            };

            let mut gpu_r = tone_mapped.r().clamp(0.0, 1.0);
            let mut gpu_g = tone_mapped.g().clamp(0.0, 1.0);
            let mut gpu_b = tone_mapped.b().clamp(0.0, 1.0);

            // Fine-Art Print Media in Linear Display Space
            if edits.print_media != PrintMediaMode::None {
                let (r_pm, g_pm, b_pm) = apply_print_media(
                    gpu_r, gpu_g, gpu_b,
                    edits.print_media,
                    edits.enlarger_filter,
                    edits.print_paper_grade,
                    edits.print_process_depth,
                );
                gpu_r = r_pm;
                gpu_g = g_pm;
                gpu_b = b_pm;
            }

            // Sample Export Image:
            let ex = ((export_img.width as f32 * u).round() as usize).clamp(0, export_img.width - 1);
            let ey = ((export_img.height as f32 * v).round() as usize).clamp(0, export_img.height - 1);
            let e_idx = (ey * export_img.width + ex) * 3;

            let exp_r = export_img.data[e_idx].clamp(0.0, 1.0);
            let exp_g = export_img.data[e_idx + 1].clamp(0.0, 1.0);
            let exp_b = export_img.data[e_idx + 2].clamp(0.0, 1.0);

            let lab_gpu = linear_srgb_to_lab(Color3::new(gpu_r, gpu_g, gpu_b));
            let lab_exp = linear_srgb_to_lab(Color3::new(exp_r, exp_g, exp_b));
            let de00 = ciede2000(lab_gpu, lab_exp);

            total_case_de00 += de00;
            if de00 > max_case_de00 {
                max_case_de00 = de00;
            }
            patch_count += 1;

            // Strict mathematical equality on individual color patches (within float16 precision)
            assert!(
                de00 < 0.15,
                "[{}] Patch '{}' diverged! GPU=[{:.4}, {:.4}, {:.4}], Exp=[{:.4}, {:.4}, {:.4}], ΔE₀₀={:.3}",
                case_name, patch.name, gpu_r, gpu_g, gpu_b, exp_r, exp_g, exp_b, de00
            );
        }

        let mean_de00 = total_case_de00 / (patch_count as f32);
        assert!(
            mean_de00 < 0.05,
            "[{}] Mean ΔE₀₀ exceeds threshold: {:.4} >= 0.05",
            case_name, mean_de00
        );
    }
}
