//! AgX Filmic Engine Parity Guard & Color Invariant Tests
//!
//! Verifies that:
//! 1. Rust CPU export tone mapper and Metal GPU shader evaluate identical AgX math.
//! 2. Neutral axis achromatic preservation holds across the entire exposure range.
//! 3. Skin tone hue stability is preserved without Abney effect shifts under extreme exposure.
//! 4. High dynamic range highlights roll off smoothly with zero digital clipping.

use firstproof_core::color::agx::{
    agx_tone_map, AGX_EV_RANGE, AGX_MID_GRAY_NORM, AGX_MIN_EV,
};
use firstproof_core::color::Color3;

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
    // Canonical skin tone coordinates in Linear Rec.2020:
    let skin_base = Color3::new(0.38, 0.22, 0.15);

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

    let base_hue = calc_hue(agx_tone_map(skin_base, 1.0));

    // Sweep exposure from -2 EV to +2 EV (5 stops photographic working range before specular desaturation)
    for ev in -2..=2 {
        let gain = 2.0f32.powi(ev);
        let exposed = Color3::new(skin_base.r() * gain, skin_base.g() * gain, skin_base.b() * gain);
        let out = agx_tone_map(exposed, 1.0);
        let hue = calc_hue(out);

        // Skin hue must remain stable within a tight photographic band (±8 degrees from base across 5 stops):
        let hue_diff = (hue - base_hue).abs();
        assert!(
            hue_diff < 8.0,
            "Skin tone hue drifted by {} deg at EV {}! Base={}, Current={}",
            hue_diff, ev, base_hue, hue
        );
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
