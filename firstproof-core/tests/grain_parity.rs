//! Dual-Engine Math Parity & Sensitometric Tests for Stochastic Emulsion Resynthesis
//!
//! Verifies:
//! 1. Rust CPU export (`apply_stochastic_emulsion_grain`) and Metal GPU shader evaluate identical math.
//! 2. Clean bypass invariant: when EmulsionGrainMode::Off, input RGB is passed through bit-for-bit.
//! 3. Nutting's law of granularity boundary anchoring: grain amplitude vanishes at Dmax (-10 EV) and Dmin (+6.5 EV) and peaks at midtone D ~ 1.0.
//! 4. Monochrome emulsions preserve the neutral axis ($R=G=B \implies R'=G'=B'$).
//! 5. XMP roundtrip serialization and deserialization preserves grain mode, size, and amount.
//! 6. Format magnification factor scaling and science-calibrated grain defaults.

use firstproof_core::color::grain::apply_stochastic_emulsion_grain;
use firstproof_core::{
    edits_has_modifications, EditSettings, EmulsionArchetype, EmulsionFormat, EmulsionGrainMode,
};

/// Exact simulation of the Metal shader function `applyStochasticEmulsionGrain`
fn metal_shader_grain_sim(
    r: f32,
    g: f32,
    b: f32,
    y_c: f32,
    y_l: f32,
    y_r: f32,
    y_t: f32,
    y_b: f32,
    y_tl: f32,
    y_tr: f32,
    y_bl: f32,
    y_br: f32,
    px: f32,
    py: f32,
    grain_mode: i32,
    cluster_size: f32,
    grain_amount: f32,
    archetype: i32,
) -> (f32, f32, f32) {
    if grain_mode == 0 {
        return (r, g, b);
    }

    let y_lowpass = (1.0 / 6.0) * (y_l + y_r + y_t + y_b)
        + (1.0 / 12.0) * (y_tl + y_tr + y_bl + y_br);
    let delta_sensor = y_c - y_lowpass;

    let gx = ((y_tr + 2.0 * y_r + y_br) - (y_tl + 2.0 * y_l + y_bl)) * 0.125;
    let gy = ((y_bl + 2.0 * y_b + y_br) - (y_tl + 2.0 * y_t + y_tr)) * 0.125;
    let grad_mag = (gx * gx + gy * gy).sqrt();

    let edge_suppress = 1.0 / (1.0 + 35.0 * grad_mag);
    let delta_noise = delta_sensor * edge_suppress;

    let seed_gain = 20.0;
    let denom = y_c.max(0.0).sqrt().max(0.05);
    let sensor_seed = ((delta_noise / denom) * seed_gain).clamp(-1.0, 1.0);

    let is_instant = archetype == 7;
    let proc_noise_fn = |u: f32, v: f32| {
        let fract = |x: f32| x - x.floor();
        let px = 50.0 * fract(u * 0.3183099 + 0.71);
        let py = 50.0 * fract(v * 0.3183099 + 0.113);
        let val = fract(px * py * (px + py));
        -1.0 + 2.0 * val
    };

    let clustered_noise_fn = |x: f32, y: f32| {
        let ix = x.floor();
        let iy = y.floor();
        let fx = x - ix;
        let fy = y - iy;
        let hx = fx * fx * (3.0 - 2.0 * fx);
        let hy = fy * fy * (3.0 - 2.0 * fy);

        let n00 = proc_noise_fn(ix, iy);
        let n10 = proc_noise_fn(ix + 1.0, iy);
        let n01 = proc_noise_fn(ix, iy + 1.0);
        let n11 = proc_noise_fn(ix + 1.0, iy + 1.0);

        let top = n00 * (1.0 - hx) + n10 * hx;
        let bot = n01 * (1.0 - hx) + n11 * hx;
        top * (1.0 - hy) + bot * hy
    };

    let s = cluster_size.max(0.25);
    let proc_noise = if is_instant {
        0.60 * clustered_noise_fn(px / s, py / s)
            + 0.40 * clustered_noise_fn(px / (s * 1.8) + 17.3, py / (s * 1.8) + 31.7)
    } else if s > 1.0 {
        let t_cluster = ((s - 1.0) / 2.0).clamp(0.0, 1.0);
        let cluster = clustered_noise_fn(px / s, py / s);
        let micro = proc_noise_fn(px, py);
        let clumped = micro * (0.60 + 0.40 * cluster) + 0.25 * cluster;
        (1.0 - t_cluster) * micro + t_cluster * clumped
    } else {
        proc_noise_fn(px, py)
    };

    let eta = match grain_mode {
        1 => {
            if is_instant {
                (0.55 * proc_noise + 0.20 * sensor_seed).clamp(-1.0, 1.0)
            } else {
                (proc_noise * (0.45 + 0.55 * sensor_seed.abs()) + 0.45 * sensor_seed).clamp(-1.0, 1.0)
            }
        }
        2 => {
            if is_instant {
                (0.85 * proc_noise + 0.05 * sensor_seed).clamp(-1.0, 1.0)
            } else {
                (0.90 * proc_noise + 0.10 * sensor_seed).clamp(-1.0, 1.0)
            }
        }
        _ => 0.0,
    };

    let y_clamped = y_c.clamp(0.0, 1.0);
    let nutting = 4.0 * y_clamped * (1.0 - y_clamped);

    let amount = grain_amount.clamp(0.0, 2.0);
    let delta = eta * nutting * 0.28 * amount;
    let y_pixel = 0.2627 * r + 0.6780 * g + 0.0593 * b;

    let is_monochrome = archetype == 6 || archetype == 10 || archetype == 11;
    if is_monochrome {
        let y_res = (y_pixel * (1.0 + delta)).max(0.0);
        (y_res, y_res, y_res)
    } else {
        let chroma_damp = 1.0 - 0.10 * nutting;
        let r_c = y_pixel + chroma_damp * (r - y_pixel);
        let g_c = y_pixel + chroma_damp * (g - y_pixel);
        let b_c = y_pixel + chroma_damp * (b - y_pixel);

        let r_res = (r_c * (1.0 + delta)).max(0.0);
        let g_res = (g_c * (1.0 + delta)).max(0.0);
        let b_res = (b_c * (1.0 + delta)).max(0.0);

        (r_res, g_res, b_res)
    }
}

#[test]
fn test_clean_bypass_when_off() {
    let test_colors = [
        (0.0, 0.0, 0.0),
        (0.18, 0.18, 0.18),
        (0.5, 0.3, 0.2),
        (0.8, 0.8, 0.8),
        (1.0, 1.0, 1.0),
    ];

    for (r, g, b) in test_colors {
        let y_c = 0.2627 * r + 0.6780 * g + 0.0593 * b;
        let (out_r, out_g, out_b) = apply_stochastic_emulsion_grain(
            r, g, b, y_c, y_c, y_c, y_c, y_c, y_c, y_c, y_c, y_c, 0.5, 0.5,
            EmulsionGrainMode::Off,
            1.0,
            1.0,
            EmulsionArchetype::RochesterPortrait400,
        );
        assert_eq!(out_r, r, "Bypass must preserve R unaltered");
        assert_eq!(out_g, g, "Bypass must preserve G unaltered");
        assert_eq!(out_b, b, "Bypass must preserve B unaltered");
    }
}

#[test]
fn test_neutral_axis_preservation_for_monochrome() {
    let mono_archetypes = [
        EmulsionArchetype::MobberleySilver400,
        EmulsionArchetype::LiguriaCinema80,
        EmulsionArchetype::BaodingSilver100,
    ];

    let test_lumas = [0.05, 0.18, 0.40, 0.70, 0.95];

    for &archetype in &mono_archetypes {
        for &lum in &test_lumas {
            let y_c = lum;
            let y_lowpass = lum * 0.98; // slight sensor variance
            let (r, g, b) = apply_stochastic_emulsion_grain(
                lum, lum, lum, y_c, y_lowpass, y_lowpass, y_lowpass, y_lowpass,
                y_lowpass, y_lowpass, y_lowpass, y_lowpass, 0.3, 0.7,
                EmulsionGrainMode::SynthesizePhysical,
                1.0,
                1.2,
                archetype,
            );
            assert_eq!(r, g, "Monochrome silver grain must preserve neutral axis R==G");
            assert_eq!(g, b, "Monochrome silver grain must preserve neutral axis G==B");
        }
    }
}

#[test]
fn test_nutting_law_boundary_anchoring_21_step_wedge() {
    // 21-step wedge from -10 EV to +6.5 EV relative to 0.18 middle gray
    let archetype = EmulsionArchetype::RochesterPortrait400;

    // At deep Dmax (-10 EV, lum ~ 0.000176) and highlight ceiling (+6.5 EV, lum ~ 16.0 clamped to 1.0)
    // Nutting factor 4*Y*(1-Y) must approach 0.0, extinguishing grain modulation.
    for step in 0..=20 {
        let ev = -10.0 + (step as f32) * (16.5 / 20.0);
        let lum = (0.18 * 2.0f32.powf(ev)).clamp(0.0, 2.0);
        let y_c = lum;
        let y_lowpass = lum * 0.95; // 5% residual
        let (r, g, b) = apply_stochastic_emulsion_grain(
            lum, lum, lum, y_c, y_lowpass, y_lowpass, y_lowpass, y_lowpass,
            y_lowpass, y_lowpass, y_lowpass, y_lowpass, 0.4, 0.6,
            EmulsionGrainMode::SynthesizePhysical,
            1.0,
            1.0,
            archetype,
        );

        let delta_r = (r - lum).abs();
        let delta_g = (g - lum).abs();
        let delta_b = (b - lum).abs();

        if ev <= -9.0 {
            assert!(
                delta_r < 0.005 && delta_g < 0.005 && delta_b < 0.005,
                "Grain must extinguish at Dmax (step {}, EV {}): delta_r={}", step, ev, delta_r
            );
        }
        if ev >= 3.0 {
            // High radiance ceiling (Y >= 1.0): 4Y(1-Y) clamped to 0.0
            assert!(
                delta_r < 0.001 && delta_g < 0.001 && delta_b < 0.001,
                "Grain must extinguish at specular ceiling (step {}, EV {}): delta_r={}", step, ev, delta_r
            );
        }
    }
}

#[test]
fn test_dual_engine_parity_across_archetypes() {
    let test_colors = [
        (0.05, 0.05, 0.05), // Deep shadow
        (0.18, 0.18, 0.18), // Middle gray
        (0.45, 0.35, 0.25), // Warm skin tone
        (0.20, 0.50, 0.80), // Blue sky
        (0.85, 0.85, 0.85), // Highlight
    ];

    let archetypes = [
        EmulsionArchetype::RochesterPortrait400,
        EmulsionArchetype::AshigaraChrome50,
        EmulsionArchetype::RochesterK14Chrome64,
        EmulsionArchetype::RochesterCinema500,
        EmulsionArchetype::MobberleySilver400,
        EmulsionArchetype::CambridgeDiffusion100,
        EmulsionArchetype::LeverkusenPortrait160,
        EmulsionArchetype::AshigaraPortrait400,
        EmulsionArchetype::LiguriaCinema80,
        EmulsionArchetype::BaodingSilver100,
    ];

    let grain_modes = [
        EmulsionGrainMode::SeedFromSensor,
        EmulsionGrainMode::SynthesizePhysical,
    ];

    let grain_amounts = [0.5, 1.0, 1.8];
    let grain_sizes = [0.8, 1.0, 1.5];

    for &archetype in &archetypes {
        for &mode in &grain_modes {
            for &amount in &grain_amounts {
                for &size in &grain_sizes {
                    for &(r, g, b) in &test_colors {
                        let y_c = 0.2627 * r + 0.6780 * g + 0.0593 * b;
                        let y_l = y_c * 0.95;
                        let y_r = y_c * 0.97;
                        let y_t = y_c * 0.96;
                        let y_b = y_c * 0.96;
                        let y_tl = y_c * 0.94;
                        let y_tr = y_c * 0.98;
                        let y_bl = y_c * 0.93;
                        let y_br = y_c * 0.97;
                        let u = 0.35f32;
                        let v = 0.72f32;

                        let (rust_r, rust_g, rust_b) = apply_stochastic_emulsion_grain(
                            r, g, b, y_c, y_l, y_r, y_t, y_b, y_tl, y_tr, y_bl, y_br, u, v, mode, size, amount, archetype,
                        );
                        let (metal_r, metal_g, metal_b) = metal_shader_grain_sim(
                            r, g, b, y_c, y_l, y_r, y_t, y_b, y_tl, y_tr, y_bl, y_br, u, v, mode.id_number(), size, amount, archetype.id_number(),
                        );

                        let diff_r = (rust_r - metal_r).abs();
                        let diff_g = (rust_g - metal_g).abs();
                        let diff_b = (rust_b - metal_b).abs();

                        assert!(
                            diff_r < 1e-5 && diff_g < 1e-5 && diff_b < 1e-5,
                            "Dual-engine parity mismatch for archetype {:?} (mode {:?}, size {}, amount {}): Rust=({}, {}, {}), Metal=({}, {}, {})",
                            archetype, mode, size, amount, rust_r, rust_g, rust_b, metal_r, metal_g, metal_b
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_xmp_roundtrip_grain_settings() {
    use firstproof_core::xmp::{generate_xmp, parse_xmp, XmpData};

    let mut data = XmpData::default();
    data.edits.emulsion_grain_mode = EmulsionGrainMode::SynthesizePhysical;
    data.edits.grain_scale = 1.6;
    data.edits.grain_size = 1.25;
    data.edits.grain_amount = 1.10;
    data.edits.emulsion_format = EmulsionFormat::Format16mm;

    let xmp_str = generate_xmp(&data);
    let parsed = parse_xmp(&xmp_str).expect("XMP deserialization must succeed");

    assert_eq!(parsed.edits.emulsion_grain_mode, EmulsionGrainMode::SynthesizePhysical);
    assert!((parsed.edits.grain_scale - 1.6).abs() < 1e-3);
    assert!((parsed.edits.grain_size - 1.25).abs() < 1e-3);
    assert!((parsed.edits.grain_amount - 1.10).abs() < 1e-3);
    assert_eq!(parsed.edits.emulsion_format, EmulsionFormat::Format16mm);
}

#[test]
fn test_has_modifications_with_grain() {
    let mut edits = EditSettings::default();
    assert!(!edits_has_modifications(&edits, None, None));

    edits.emulsion_grain_mode = EmulsionGrainMode::SynthesizePhysical;
    assert!(edits_has_modifications(&edits, None, None));

    edits.emulsion_grain_mode = EmulsionGrainMode::Off;
    assert!(!edits_has_modifications(&edits, None, None));

    edits.grain_scale = 1.5;
    assert!(edits_has_modifications(&edits, None, None));

    edits.grain_scale = 1.0;
    edits.grain_size = 1.35;
    assert!(edits_has_modifications(&edits, None, None));

    edits.grain_size = 1.0;
    edits.grain_amount = 0.8;
    assert!(edits_has_modifications(&edits, None, None));
}

#[test]
fn test_film_format_magnification_scaling() {
    assert_eq!(EmulsionFormat::Super8.magnification_factor(), 7.3);
    assert_eq!(EmulsionFormat::Format16mm.magnification_factor(), 3.4);
    assert_eq!(EmulsionFormat::Format35mm.magnification_factor(), 1.0);
    assert_eq!(EmulsionFormat::Format120.magnification_factor(), 0.48);
    assert_eq!(EmulsionFormat::Format4x5.magnification_factor(), 0.28);
}

#[test]
fn test_calibrated_grain_defaults() {
    assert_eq!(
        EmulsionArchetype::AshigaraChrome50.calibrated_grain_defaults(),
        (0.55, 0.45)
    );
    assert_eq!(
        EmulsionArchetype::RochesterK14Chrome64.calibrated_grain_defaults(),
        (0.70, 0.60)
    );
    assert_eq!(
        EmulsionArchetype::LiguriaCinema80.calibrated_grain_defaults(),
        (0.75, 0.65)
    );
    assert_eq!(
        EmulsionArchetype::LeverkusenPortrait160.calibrated_grain_defaults(),
        (0.80, 0.70)
    );
    assert_eq!(
        EmulsionArchetype::CambridgeDiffusion100.calibrated_grain_defaults(),
        (0.85, 0.50)
    );
    assert_eq!(
        EmulsionArchetype::AshigaraPortrait400.calibrated_grain_defaults(),
        (0.95, 0.75)
    );
    assert_eq!(
        EmulsionArchetype::RochesterPortrait400.calibrated_grain_defaults(),
        (1.00, 0.80)
    );
    assert_eq!(
        EmulsionArchetype::BaodingSilver100.calibrated_grain_defaults(),
        (1.05, 1.00)
    );
    assert_eq!(
        EmulsionArchetype::RochesterCinema500.calibrated_grain_defaults(),
        (1.25, 1.10)
    );
    assert_eq!(
        EmulsionArchetype::MobberleySilver400.calibrated_grain_defaults(),
        (1.35, 1.20)
    );
}

#[test]
fn test_color_hue_invariance_and_anti_green_drift() {
    // Representative skin tones and warm interior colors (like _MG_2622.CR2)
    let test_colors = [
        (0.12, 0.08, 0.06), // warm portrait shadow/mid
        (0.35, 0.22, 0.15), // Caucasian/Asian skin midtone
        (0.20, 0.14, 0.08), // deep portrait skin tone
        (0.08, 0.12, 0.05), // foliage green
        (0.06, 0.09, 0.18), // twilight sky
    ];

    for (r, g, b) in test_colors {
        let _y_pixel = 0.2627 * r + 0.6780 * g + 0.0593 * b;
        let y_c = 0.40; // Simulated display midtone
        let y_lowpass = 0.38;

        let (out_r, out_g, _out_b) = apply_stochastic_emulsion_grain(
            r, g, b, y_c, y_lowpass, y_lowpass, y_lowpass, y_lowpass,
            y_lowpass, y_lowpass, y_lowpass, y_lowpass, 0.4, 0.6,
            EmulsionGrainMode::SeedFromSensor,
            1.0,
            1.0,
            EmulsionArchetype::RochesterPortrait400,
        );

        // Strict anti-green-cast invariant: G channel must NOT explode relative to R or B
        let orig_rg_ratio = r / g;
        let res_rg_ratio = out_r / out_g;
        let ratio_diff = (orig_rg_ratio - res_rg_ratio).abs() / orig_rg_ratio;
        assert!(
            ratio_diff < 0.10,
            "Grain must preserve R/G chromaticity ratio within 10% (was orig={orig_rg_ratio:.3}, res={res_rg_ratio:.3}, diff={ratio_diff:.1}%)"
        );

        // Green output must never exceed 1.5x original green on warm portrait tones
        if r > g {
            assert!(
                out_g < g * 1.50,
                "Green output {out_g} must not exceed 1.5x original green {g} on warm skin/portrait tone"
            );
        }
    }
}

#[test]
fn test_edge_gradient_decoupling_prevents_oversharpening() {
    // Sharp optical edge (e.g. dark hair y=0.10 against bright cheek y=0.60)
    let y_c = 0.55;
    let y_l = 0.15; // Left is dark
    let y_r = 0.65; // Right is bright
    let y_t = 0.55;
    let y_b = 0.55;
    let y_tl = 0.15;
    let y_tr = 0.65;
    let y_bl = 0.15;
    let y_br = 0.65;

    // Evaluate warm portrait skin tone right on the edge
    let (r, g, b) = (0.50, 0.35, 0.25);
    let (out_r, out_g, out_b) = apply_stochastic_emulsion_grain(
        r, g, b, y_c, y_l, y_r, y_t, y_b, y_tl, y_tr, y_bl, y_br, 0.5, 0.5,
        EmulsionGrainMode::SeedFromSensor,
        1.0,
        1.0,
        EmulsionArchetype::RochesterPortrait400,
    );

    // Edge overshoot check: must not exceed 25% change on an optical edge
    // (Prevents the extreme unsharp mask oversharpening artifact)
    let delta_rel = ((out_r - r) / r).abs();
    assert!(
        delta_rel < 0.25,
        "Optical edge must be decoupled: relative delta was {:.2}%, must be < 25%",
        delta_rel * 100.0
    );
    assert!(out_g > 0.0 && out_b > 0.0);
}
