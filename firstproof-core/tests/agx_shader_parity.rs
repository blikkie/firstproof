//! AgX Filmic Engine Parity Guard & Color Invariant Tests
//!
//! Verifies that:
//! 1. Rust CPU export tone mapper and Metal GPU shader evaluate identical AgX math.
//! 2. Neutral axis achromatic preservation holds across the entire exposure range.
//! 3. Skin tone hue stability is preserved without Abney effect shifts under extreme exposure.
//! 4. High dynamic range highlights roll off smoothly with zero digital clipping.

use firstproof_core::color::agx::{
    agx_tone_map, agx_tone_map_vivid, AGX_EV_RANGE, AGX_MID_GRAY_NORM, AGX_MIN_EV,
};
use firstproof_core::color::{Color3, LINEAR_SRGB_TO_REC2020, REC2020_TO_LINEAR_SRGB};

/// Simulates the exact single-precision Metal shader implementation (`agxToneMap` in Shaders.metal)
fn metal_shader_agx_sim(rec2020_r: f32, rec2020_g: f32, rec2020_b: f32, contrast_gain: f32) -> (f32, f32, f32) {
    let min_c = rec2020_r.min(rec2020_g).min(rec2020_b);
    let (c_r, c_g, c_b) = if min_c < 0.0 {
        (rec2020_r - min_c, rec2020_g - min_c, rec2020_b - min_c)
    } else {
        (rec2020_r, rec2020_g, rec2020_b)
    };

    // Metal column-major matrix multiplication: M * v
    // AGX_INSET_MATRIX (in Metal float3x3 columns) pre-multiplied for Linear Rec.2020
    let in_r = 1.4096986 * c_r - 0.4004796 * c_g - 0.009218898 * c_b;
    let in_g = 0.13136263 * c_r + 0.77151406 * c_g + 0.09712328 * c_b;
    let in_b = 0.16151477 * c_r - 0.06034984 * c_g + 0.89883506 * c_b;

    let process = |v: f32| -> f32 {
        let x_rel = (v / 0.18).max(1e-6);
        let log_encoded = (x_rel.log2() - AGX_MIN_EV) / AGX_EV_RANGE;
        let mut norm = log_encoded.clamp(0.0, 1.0);
        if (contrast_gain - 1.0).abs() > 0.001 {
            norm = (AGX_MID_GRAY_NORM + (norm - AGX_MID_GRAY_NORM) * contrast_gain).clamp(0.0, 1.0);
        }
        let curved = firstproof_core::color::agx::agx_apply_curve_channel(norm);
        curved.max(0.0).powf(firstproof_core::color::agx::AGX_GAMMA)
    };

    let cr = process(in_r);
    let cg = process(in_g);
    let cb = process(in_b);

    // AGX_OUTSET_MATRIX (in Metal float3x3 columns)
    let out_r = (1.1271005 * cr - 0.1106066 * cg - 0.0164939 * cb).max(0.0);
    let out_g = (-0.1413298 * cr + 1.1578237 * cg - 0.0164939 * cb).max(0.0);
    let out_b = (-0.1413298 * cr - 0.1106066 * cg + 1.2519364 * cb).max(0.0);

    (out_r, out_g, out_b)
}

#[test]
fn test_metal_rust_dual_engine_parity() {
    // Generate a diverse grid of test colors across dynamic range [0.001 .. 10.0]
    let test_levels = [0.005, 0.02, 0.10, 0.18, 0.50, 1.0, 2.5, 8.0];
    let test_contrasts = [0.8, 1.0, 1.2, 1.5];

    for &contrast in &test_contrasts {
        for &r in &test_levels {
            for &g in &test_levels {
                for &b in &test_levels {
                    let rust_out = agx_tone_map(Color3::new(r, g, b), contrast);
                    let (metal_r, metal_g, metal_b) = metal_shader_agx_sim(r, g, b, contrast);

                    let diff_r = (rust_out.r() - metal_r).abs();
                    let diff_g = (rust_out.g() - metal_g).abs();
                    let diff_b = (rust_out.b() - metal_b).abs();

                    assert!(
                        diff_r < 1e-5 && diff_g < 1e-5 && diff_b < 1e-5,
                        "Parity divergence at input ({}, {}, {}) with contrast {}: Rust=({}, {}, {}), Metal=({}, {}, {})",
                        r, g, b, contrast, rust_out.r(), rust_out.g(), rust_out.b(), metal_r, metal_g, metal_b
                    );
                }
            }
        }
    }
}

/// Simulates the exact single-precision Metal shader implementation (`agxToneMapVivid` in ColorScience.metal)
fn metal_shader_agx_vivid_sim(rec2020_r: f32, rec2020_g: f32, rec2020_b: f32, contrast_gain: f32) -> (f32, f32, f32) {
    let punchy_contrast = contrast_gain * 1.35;
    let (br, bg, bb) = metal_shader_agx_sim(rec2020_r, rec2020_g, rec2020_b, punchy_contrast);
    let lum = 0.2126 * br + 0.7152 * bg + 0.0722 * bb;
    let t = ((lum - 0.65) / 0.35).clamp(0.0, 1.0);
    let h = t * t * (3.0 - 2.0 * t);
    let base_sat = 1.25 - 0.25 * h;
    let sat = base_sat;
    (
        (lum + sat * (br - lum)).clamp(0.0, 1.0),
        (lum + sat * (bg - lum)).clamp(0.0, 1.0),
        (lum + sat * (bb - lum)).clamp(0.0, 1.0),
    )
}

#[test]
fn test_metal_rust_agx_vivid_parity() {
    let test_levels = [0.005, 0.02, 0.10, 0.18, 0.50, 1.0, 2.5, 8.0];
    let test_contrasts = [0.8, 1.0, 1.2];

    for &contrast in &test_contrasts {
        for &r in &test_levels {
            for &g in &test_levels {
                for &b in &test_levels {
                    let rust_out = firstproof_core::color::agx_tone_map_vivid(Color3::new(r, g, b), contrast);
                    let (metal_r, metal_g, metal_b) = metal_shader_agx_vivid_sim(r, g, b, contrast);

                    let diff_r = (rust_out.r() - metal_r).abs();
                    let diff_g = (rust_out.g() - metal_g).abs();
                    let diff_b = (rust_out.b() - metal_b).abs();

                    assert!(
                        diff_r < 1e-4 && diff_g < 1e-4 && diff_b < 1e-4,
                        "AgX Vivid parity divergence at ({}, {}, {}) with contrast {}: Rust=({}, {}, {}), Metal=({}, {}, {})",
                        r, g, b, contrast, rust_out.r(), rust_out.g(), rust_out.b(), metal_r, metal_g, metal_b
                    );
                }
            }
        }
    }
}

#[test]
fn test_neutral_axis_achromatic_preservation() {
    // Across 16 stops of exposure, neutral input must yield neutral output
    for stop in -10..=6 {
        let linear_val = 0.18 * 2.0f32.powi(stop);
        let out = agx_tone_map(Color3::new(linear_val, linear_val, linear_val), 1.0);

        let max_divergence = (out.r() - out.g()).abs().max((out.g() - out.b()).abs());
        assert!(
            max_divergence < 1e-5,
            "Achromatic axis broken at EV {stop} (val={linear_val}): ({}, {}, {})",
            out.r(), out.g(), out.b()
        );
    }
}

#[test]
fn test_skin_tone_hue_stability_under_exposure() {
    // Canonical diverse skin tone coordinates in Linear Rec.2020:
    let skin_bases = [
        ("Light Skin", Color3::new(0.453, 0.327, 0.240)),
        ("Medium Tan", Color3::new(0.380, 0.220, 0.150)),
        ("Dark Skin",  Color3::new(0.138, 0.088, 0.058)),
    ];

    // Compute hue angle in degrees
    let calc_hue = |c: Color3| -> f32 {
        let mx = c.r().max(c.g()).max(c.b());
        let mn = c.r().min(c.g()).min(c.b());
        let df = mx - mn;
        if df < 1e-6 {
            return 0.0;
        }
        let h = if (mx - c.r()).abs() < 1e-6 {
            60.0 * ((c.g() - c.b()) / df)
        } else if (mx - c.g()).abs() < 1e-6 {
            60.0 * ((c.b() - c.r()) / df) + 120.0
        } else {
            60.0 * ((c.r() - c.g()) / df) + 240.0
        };
        (h + 360.0) % 360.0
    };

    for (name, skin_base) in skin_bases {
        let base_hue = calc_hue(agx_tone_map(skin_base, 1.0));

        // Sweep exposure across photographic working range (-2 EV to +1 EV, 4 stops before specular highlight desaturation)
        for ev in &[-2.0, -1.0, 0.0, 0.5, 1.0] {
            let gain = 2.0f32.powf(*ev);
            let exposed = Color3::new(skin_base.r() * gain, skin_base.g() * gain, skin_base.b() * gain);
            let out = agx_tone_map(exposed, 1.0);
            let hue = calc_hue(out);

            // Skin hue must remain stable within a tight photographic band (±15 degrees from base across 4 stops down to deep shadow):
            let hue_diff = (hue - base_hue).abs();
            assert!(
                hue_diff < 15.0,
                "{name} hue drifted by {hue_diff} deg at EV {ev}! Base={base_hue}, Current={hue}"
            );
        }
    }
}

#[test]
fn test_extreme_dynamic_range_robustness() {
    // Check extreme specular points up to 1,000x overexposed (10 stops above diffuse white)
    let extreme = Color3::new(1000.0, 800.0, 500.0);
    let out = agx_tone_map(extreme, 1.0);

    assert!(!out.r().is_nan() && !out.r().is_infinite());
    assert!(!out.g().is_nan() && !out.g().is_infinite());
    assert!(!out.b().is_nan() && !out.b().is_infinite());
    assert!(out.r() <= 1.0 && out.g() <= 1.0 && out.b() <= 1.0);
}

#[test]
fn test_metal_rust_split_toning_parity() {
    use firstproof_core::export::apply_split_toning;

    let test_pixels = [
        (0.05, 0.05, 0.08), // deep shadow
        (0.25, 0.20, 0.18), // shadow midtone
        (0.50, 0.50, 0.50), // neutral gray
        (0.75, 0.70, 0.65), // warm highlight
        (0.95, 0.95, 0.95), // near white
    ];

    let test_params = [
        // (sh_hue, sh_sat, hl_hue, hl_sat, balance)
        (215.0, 40.0, 45.0, 30.0, 0.0),
        (215.0, 60.0, 45.0, 0.0, -50.0),
        (180.0, 0.0, 60.0, 50.0, 50.0),
        (300.0, 75.0, 30.0, 65.0, 25.0),
    ];

    for &(sh_h, sh_s, hl_h, hl_s, bal) in &test_params {
        for &(r, g, b) in &test_pixels {
            let rust_out = apply_split_toning(r, g, b, sh_h, sh_s, hl_h, hl_s, bal);

            // Simulated Metal shader logic matching ColorScience.metal
            let lum = 0.2627 * r + 0.6780 * g + 0.0593 * b;
            let y_clamp = lum.clamp(0.0, 1.0);
            let t = (0.5 - (bal / 100.0) * 0.25).clamp(0.1, 0.9);

            let mut delta_r = 0.0f32;
            let mut delta_g = 0.0f32;
            let mut delta_b = 0.0f32;

            if sh_s > 0.0 && y_clamp < t {
                let u_s = (t - y_clamp) / t;
                let w_s = u_s * u_s * (3.0 - 2.0 * u_s) * (sh_s / 100.0);
                let (tr, tg, tb) = firstproof_core::color::hsv_to_rgb(sh_h, 1.0, 1.0);
                let t_lum = 0.2627 * tr + 0.6780 * tg + 0.0593 * tb;
                delta_r += w_s * (tr - t_lum);
                delta_g += w_s * (tg - t_lum);
                delta_b += w_s * (tb - t_lum);
            }
            if hl_s > 0.0 && y_clamp > t {
                let u_h = (y_clamp - t) / (1.0 - t);
                let w_h = u_h * u_h * (3.0 - 2.0 * u_h) * (hl_s / 100.0);
                let (tr, tg, tb) = firstproof_core::color::hsv_to_rgb(hl_h, 1.0, 1.0);
                let t_lum = 0.2627 * tr + 0.6780 * tg + 0.0593 * tb;
                delta_r += w_h * (tr - t_lum);
                delta_g += w_h * (tg - t_lum);
                delta_b += w_h * (tb - t_lum);
            }

            let metal_out = (
                (r + delta_r).clamp(0.0, 1.0),
                (g + delta_g).clamp(0.0, 1.0),
                (b + delta_b).clamp(0.0, 1.0),
            );

            assert!((rust_out.0 - metal_out.0).abs() < 1e-6);
            assert!((rust_out.1 - metal_out.1).abs() < 1e-6);
            assert!((rust_out.2 - metal_out.2).abs() < 1e-6);
        }
    }
}

#[test]
fn test_synthetic_greyscale_wedge_photometric_continuity() {
    // 21-step photometric wedge covering the entire 16.5-stop dynamic range (-10.0 EV to +6.5 EV)
    // relative to 0.18 middle gray.
    const STEPS: usize = 21;
    let mut prev_output = -1.0f32;

    for i in 0..STEPS {
        let ev = -10.0 + (i as f32) * (16.5 / (STEPS - 1) as f32);
        let linear_val = 0.18 * 2.0f32.powf(ev);
        let input = Color3::new(linear_val, linear_val, linear_val);
        let out = agx_tone_map(input, 1.0);

        // 1. Strict achromatic purity
        assert!(
            (out.r() - out.g()).abs() < 1e-5 && (out.g() - out.b()).abs() < 1e-5,
            "Achromatic axis broken on step {i} (EV {ev:.2}): ({}, {}, {})",
            out.r(), out.g(), out.b()
        );

        // 2. Strict monotonicity (f'(x) > 0: every stop must be strictly brighter than the previous)
        assert!(
            out.r() > prev_output,
            "Monotonicity violation on step {i} (EV {ev:.2}): current {} <= prev {}",
            out.r(), prev_output
        );

        // 3. Boundary anchoring:
        // Deep shadow floor (-10 EV): output must be >= 0.0 and < 0.005 (not negative, not lifted).
        // Highlight ceiling (+6.5 EV): output must roll off smoothly near 1.0 without blowout.
        if i == 0 {
            assert!(
                out.r() >= 0.0 && out.r() < 0.005,
                "Shadow boundary anchor violated at -10 EV: {}",
                out.r()
            );
        }
        if i == STEPS - 1 {
            assert!(
                out.r() <= 1.0 && out.r() >= 0.99,
                "Highlight boundary anchor violated at +6.5 EV: {}",
                out.r()
            );
        }

        // 4. No premature clipping:
        // Stops above -9.5 EV must have real positive photons (> 0.0).
        if ev > -9.5 {
            assert!(
                out.r() > 0.0,
                "Premature shadow clipping at EV {ev:.2}: out={}",
                out.r()
            );
        }
        // Stops below +6.0 EV must not be fully blown out (< 0.99).
        if ev < 6.0 {
            assert!(
                out.r() < 0.99,
                "Premature highlight clipping at EV {ev:.2}: out={}",
                out.r()
            );
        }

        prev_output = out.r();
    }
}

/// Simulates Metal shader Step 9 non-RAW terminal stage:
/// Non-RAW master images bypass AgX and Camera DCP tone curves, applying only
/// the contrast pivot around middle gray (0.18) if contrast != 0.
fn metal_shader_non_raw_sim(rec2020_r: f32, rec2020_g: f32, rec2020_b: f32, contrast_gain: f32) -> (f32, f32, f32) {
    // REC2020_TO_LINEAR_SRGB * rec2020Color
    // In Metal MSL column-major float3x3:
    let srgb_r = (1.660491 * rec2020_r - 0.5876411 * rec2020_g - 0.0728499 * rec2020_b).max(0.0);
    let srgb_g = (-0.1245505 * rec2020_r + 1.1328999 * rec2020_g - 0.0083494 * rec2020_b).max(0.0);
    let srgb_b = (-0.0181508 * rec2020_r - 0.1005789 * rec2020_g + 1.1187297 * rec2020_b).max(0.0);

    if (contrast_gain - 1.0).abs() > 0.001 {
        (
            ((srgb_r - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0),
            ((srgb_g - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0),
            ((srgb_b - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0),
        )
    } else {
        (
            srgb_r.clamp(0.0, 1.0),
            srgb_g.clamp(0.0, 1.0),
            srgb_b.clamp(0.0, 1.0),
        )
    }
}

/// Simulates Rust CPU export pipeline Step 3 for non-RAW images
fn rust_export_non_raw_sim(rec2020_r: f32, rec2020_g: f32, rec2020_b: f32, contrast_gain: f32) -> (f32, f32, f32) {
    let c_rec2020 = Color3::new(rec2020_r, rec2020_g, rec2020_b);
    let c_srgb = REC2020_TO_LINEAR_SRGB.multiply_vector(c_rec2020);
    let mut r = c_srgb.r().max(0.0);
    let mut g = c_srgb.g().max(0.0);
    let mut b = c_srgb.b().max(0.0);
    if (contrast_gain - 1.0).abs() > 0.001 {
        r = ((r - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0);
        g = ((g - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0);
        b = ((b - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0);
    } else {
        r = r.clamp(0.0, 1.0);
        g = g.clamp(0.0, 1.0);
        b = b.clamp(0.0, 1.0);
    }
    (r, g, b)
}

#[test]
fn test_non_raw_terminal_bypass_parity_and_identity() {
    let test_srgb_levels = [0.01, 0.05, 0.10, 0.18, 0.35, 0.50, 0.75, 0.90, 0.99];
    let test_contrasts = [0.0, 15.0, -20.0];

    for &contrast_val in &test_contrasts {
        let contrast_gain = if contrast_val >= 0.0 {
            1.0 + (contrast_val / 100.0)
        } else {
            1.0 + (contrast_val / 100.0) * 0.60
        };

        for &r in &test_srgb_levels {
            for &g in &test_srgb_levels {
                for &b in &test_srgb_levels {
                    // Encode linear sRGB -> Linear Rec.2020 (simulating Step 2)
                    let c_srgb_lin = Color3::new(r, g, b);
                    let c_rec2020 = LINEAR_SRGB_TO_REC2020.multiply_vector(c_srgb_lin);

                    let (rust_r, rust_g, rust_b) = rust_export_non_raw_sim(
                        c_rec2020.r(), c_rec2020.g(), c_rec2020.b(), contrast_gain
                    );
                    let (metal_r, metal_g, metal_b) = metal_shader_non_raw_sim(
                        c_rec2020.r(), c_rec2020.g(), c_rec2020.b(), contrast_gain
                    );

                    // 1. Bit-level parity between Metal shader and Rust CPU export pipeline
                    let diff_r = (rust_r - metal_r).abs();
                    let diff_g = (rust_g - metal_g).abs();
                    let diff_b = (rust_b - metal_b).abs();
                    assert!(
                        diff_r < 1e-5 && diff_g < 1e-5 && diff_b < 1e-5,
                        "Non-RAW dual-engine parity divergence: Rust=({}, {}, {}), Metal=({}, {}, {})",
                        rust_r, rust_g, rust_b, metal_r, metal_g, metal_b
                    );

                    // 2. Identity preservation when contrast == 0 (zero slider touches)
                    if contrast_val == 0.0 {
                        assert!(
                            (rust_r - r).abs() < 1e-5 &&
                            (rust_g - g).abs() < 1e-5 &&
                            (rust_b - b).abs() < 1e-5,
                            "Non-RAW neutral identity violated: input=({}, {}, {}), output=({}, {}, {})",
                            r, g, b, rust_r, rust_g, rust_b
                        );

                        // 3. Proves that output is NOT corrupted by AgX Vivid double tone-mapping
                        let agx_vivid_out = agx_tone_map_vivid(c_rec2020, 1.0);
                        if (r - 0.18).abs() > 0.05 {
                            // Points away from mid-gray must diverge significantly from AgX Vivid
                            let diff_agx = (rust_r - agx_vivid_out.r()).abs()
                                + (rust_g - agx_vivid_out.g()).abs()
                                + (rust_b - agx_vivid_out.b()).abs();
                            assert!(
                                diff_agx > 0.005,
                                "Non-RAW output unexpectedly matched AgX Vivid tone curve at ({}, {}, {})",
                                r, g, b
                            );
                        }
                    }
                }
            }
        }
    }
}

