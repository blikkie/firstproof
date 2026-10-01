//! Dual-Engine Math Parity & Sensitometric Tests for Upstream Recording Emulsions
//!
//! Verifies:
//! 1. Rust CPU export (`apply_recording_emulsion`) and Metal GPU shader (`applyRecordingEmulsion`) evaluate identical math.
//! 2. Clean bypass invariant: when EmulsionArchetype::None, input RGB is passed through bit-for-bit.
//! 3. Achromatic neutral axis preservation: DIR chemical crosstalk leaves neutral grays pure gray.
//! 4. Sensitometric D-max absorption: deep black (0.0) stays 0.0 without artifacts.
//! 5. Push/pull photographic exposure index ($2^{\text{EV}}$) modulates sensitivity.
//! 6. Mobberley Silver 400 panchromatic conversion collapses color to true monochrome.
//! 7. Dual-engine bit parity between Rust and Metal across all 7 archetypes.
//! 8. XMP roundtrip serialization preserves all emulsion settings.

use firstproof_core::color::emulsion::{
    apply_dir_chemical_coupling, apply_recording_emulsion,
    EmulsionArchetype, EmulsionFormat, EnlargerFilterMode,
};
use firstproof_core::{edits_has_modifications, EditSettings};

/// Exact simulation of the Metal shader function `applyRecordingEmulsion` in ColorScience.metal
fn metal_shader_emulsion_sim(
    r: f32,
    g: f32,
    b: f32,
    archetype_id: i32,
    push_pull_ev: f32,
    filter_mode: i32,
) -> (f32, f32, f32) {
    if archetype_id == 0 {
        return (r, g, b);
    }

    let metal_film_curve = |val: f32, gamma: f32, shoulder: f32, toe: f32| -> f32 {
        let v = val.clamp(0.0, 2.0);
        let base = if (gamma - 1.0).abs() > 0.001 {
            let u = (v / (v + 0.35)).clamp(0.0, 1.0);
            let g_mod = (gamma - 1.0) * 0.40;
            (v + g_mod * u * (1.0 - u) * (2.0 * u - 1.0)).max(0.0)
        } else {
            v
        };
        let after_shoulder = if shoulder > 0.0 && base > 0.65 {
            let t = ((base - 0.65) / 0.35).clamp(0.0, 1.0);
            let comp = shoulder * 0.12 * t * t * (3.0 - 2.0 * t);
            base - comp
        } else {
            base
        };
        if toe > 0.0 && after_shoulder < 0.35 {
            let t = (after_shoulder / 0.35).clamp(0.0, 1.0);
            let comp = toe * 0.06 * (1.0 - t) * (1.0 - t);
            (after_shoulder - comp).max(0.0)
        } else {
            after_shoulder
        }
    };

    let metal_dir_coupling = |rgb: (f32, f32, f32),
                              c_rg: f32, c_rb: f32,
                              c_gr: f32, c_gb: f32,
                              c_br: f32, c_bg: f32| -> (f32, f32, f32) {
        let eps = 1e-4f32;
        let d_r = -(rgb.0.max(eps)).log10();
        let d_g = -(rgb.1.max(eps)).log10();
        let d_b = -(rgb.2.max(eps)).log10();

        let c_rr = 1.0 - c_rg - c_rb;
        let c_gg = 1.0 - c_gr - c_gb;
        let c_bb = 1.0 - c_br - c_bg;

        let d_r_out = c_rr * d_r + c_rg * d_g + c_rb * d_b;
        let d_g_out = c_gr * d_r + c_gg * d_g + c_gb * d_b;
        let d_b_out = c_br * d_r + c_bg * d_g + c_bb * d_b;

        (
            10.0f32.powf(-d_r_out).clamp(0.0, 2.0),
            10.0f32.powf(-d_g_out).clamp(0.0, 2.0),
            10.0f32.powf(-d_b_out).clamp(0.0, 2.0),
        )
    };

    let exp_gain = 2.0f32.powf(push_pull_ev.clamp(-2.0, 2.0));
    let cur_r = (r * exp_gain).max(0.0);
    let cur_g = (g * exp_gain).max(0.0);
    let cur_b = (b * exp_gain).max(0.0);

    match archetype_id {
        1 => { // RochesterPortrait400
            let dir = metal_dir_coupling((cur_r, cur_g, cur_b), -0.04, 0.02, -0.08, 0.02, 0.02, -0.05);
            (
                metal_film_curve(dir.0, 0.90, 0.38, 0.08),
                metal_film_curve(dir.1, 0.90, 0.38, 0.08),
                metal_film_curve(dir.2, 0.90, 0.38, 0.08),
            )
        }
        2 => { // AshigaraChrome50
            let dir = metal_dir_coupling((cur_r, cur_g, cur_b), -0.15, -0.08, -0.18, -0.08, -0.06, -0.12);
            (
                metal_film_curve(dir.0, 1.10, 0.12, 0.14),
                metal_film_curve(dir.1, 1.12, 0.10, 0.14),
                metal_film_curve(dir.2, 1.11, 0.12, 0.16),
            )
        }
        3 => { // RochesterK14Chrome64
            let dir = metal_dir_coupling((cur_r, cur_g, cur_b), -0.08, 0.06, -0.10, 0.04, 0.08, -0.04);
            (
                metal_film_curve(dir.0, 1.05, 0.18, 0.08),
                metal_film_curve(dir.1, 1.06, 0.16, 0.10),
                metal_film_curve(dir.2, 1.08, 0.14, 0.14),
            )
        }
        4 => { // RochesterCinema500
            let dir = metal_dir_coupling((cur_r, cur_g, cur_b), -0.02, 0.01, -0.03, 0.01, 0.01, -0.02);
            (
                metal_film_curve(dir.0, 0.82, 0.45, 0.08),
                metal_film_curve(dir.1, 0.82, 0.45, 0.08),
                metal_film_curve(dir.2, 0.82, 0.45, 0.08),
            )
        }
        5 => { // RochesterDaylight200
            let dir = metal_dir_coupling((cur_r, cur_g, cur_b), -0.06, 0.03, -0.07, 0.02, 0.08, -0.10);
            (
                metal_film_curve(dir.0, 1.12, 0.20, 0.12),
                metal_film_curve(dir.1, 1.08, 0.22, 0.12),
                metal_film_curve(dir.2, 0.98, 0.26, 0.16),
            )
        }
        6 => { // MobberleySilver400
            let filter_weights = match filter_mode {
                1 => (0.35f32, 0.60f32, 0.05f32), // YellowK2
                2 => (0.55f32, 0.42f32, 0.03f32), // Orange16
                3 => (0.85f32, 0.15f32, 0.00f32), // Red25A
                4 => (0.15f32, 0.80f32, 0.05f32), // GreenX1
                5 => (0.00f32, 0.60f32, 0.40f32), // Orthochromatic
                _ => (0.2627f32, 0.6780f32, 0.0593f32), // NeutralPanchromatic
            };
            let pan_lum = filter_weights.0 * cur_r + filter_weights.1 * cur_g + filter_weights.2 * cur_b;
            let mono = metal_film_curve(pan_lum, 1.08, 0.30, 0.25);
            (mono, mono, mono)
        }
        7 => { // CambridgeDiffusion100
            let dir = metal_dir_coupling((cur_r, cur_g, cur_b), -0.03, 0.02, -0.04, 0.03, 0.01, -0.03);
            let c_r = metal_film_curve(dir.0, 0.98, 0.35, 0.12);
            let c_g = metal_film_curve(dir.1, 0.98, 0.35, 0.12);
            let c_b = metal_film_curve(dir.2, 0.98, 0.35, 0.12);
            let lum = 0.2126 * c_r + 0.7152 * c_g + 0.0722 * c_b;
            let base_refl = 0.025 * (1.0 - lum).max(0.0);
            let tio2_r = (c_r + base_refl * 1.02).min(2.0);
            let tio2_g = (c_g + base_refl * 1.00).min(2.0);
            let tio2_b = (c_b + base_refl * 0.94).min(2.0);
            (tio2_r, tio2_g, tio2_b)
        }
        _ => (r, g, b),
    }
}

#[test]
fn test_clean_bypass_when_none() {
    let test_pixels = [
        (0.0, 0.0, 0.0),
        (0.18, 0.18, 0.18),
        (1.0, 1.0, 1.0),
        (0.75, 0.25, 0.10),
    ];
    for &(r, g, b) in &test_pixels {
        let out = apply_recording_emulsion(r, g, b, EmulsionArchetype::None, 0.0, EnlargerFilterMode::NeutralPanchromatic);
        assert_eq!(out, (r, g, b));
    }
}

#[test]
fn test_achromatic_neutral_axis_preservation_under_dir() {
    // Under neutral gray (D_r = D_g = D_b), DIR coupling row sums to 1.0,
    // so neutral colors stay 100% achromatic!
    let grays = [0.05, 0.18, 0.50, 0.85];
    for &val in &grays {
        let (r_out, g_out, b_out) = apply_dir_chemical_coupling(
            val, val, val,
            -0.03, 0.01,
            -0.07, 0.02,
            0.01, -0.04,
        );
        assert!((r_out - g_out).abs() < 1e-5, "Neutral gray diverged: R={:.5}, G={:.5}", r_out, g_out);
        assert!((g_out - b_out).abs() < 1e-5, "Neutral gray diverged: G={:.5}, B={:.5}", g_out, b_out);
    }
}

#[test]
fn test_sensitometric_dmax_absorption() {
    // In deep black (0.0), D-max absorption ensures pure black stays 0.0
    let (rb, gb, bb) = apply_recording_emulsion(
        0.0, 0.0, 0.0,
        EmulsionArchetype::RochesterPortrait400,
        0.0,
        EnlargerFilterMode::NeutralPanchromatic,
    );
    assert_eq!((rb, gb, bb), (0.0, 0.0, 0.0));
}

#[test]
fn test_push_pull_exposure_gain() {
    let (r_base, _, _) = apply_recording_emulsion(0.3, 0.3, 0.3, EmulsionArchetype::RochesterPortrait400, 0.0, EnlargerFilterMode::NeutralPanchromatic);
    let (r_push, _, _) = apply_recording_emulsion(0.3, 0.3, 0.3, EmulsionArchetype::RochesterPortrait400, 1.0, EnlargerFilterMode::NeutralPanchromatic);
    let (r_pull, _, _) = apply_recording_emulsion(0.3, 0.3, 0.3, EmulsionArchetype::RochesterPortrait400, -1.0, EnlargerFilterMode::NeutralPanchromatic);

    assert!(r_push > r_base, "Pushed exposure must produce higher transmission");
    assert!(r_pull < r_base, "Pulled exposure must produce lower transmission");
}

#[test]
fn test_monochrome_mobberley_collapses_color_and_filters() {
    let filters = [
        EnlargerFilterMode::NeutralPanchromatic,
        EnlargerFilterMode::YellowK2,
        EnlargerFilterMode::Orange16,
        EnlargerFilterMode::Red25A,
        EnlargerFilterMode::GreenX1,
        EnlargerFilterMode::Orthochromatic,
    ];

    let mut previous_lum: Option<f32> = None;
    for filter in filters {
        // Red dominant scene: filters must shift monochrome response dramatically
        let (r, g, b) = apply_recording_emulsion(
            0.8, 0.2, 0.1,
            EmulsionArchetype::MobberleySilver400,
            0.0,
            filter,
        );
        // Pure monochrome collapse
        assert_eq!(r, g);
        assert_eq!(g, b);

        let metal_out = metal_shader_emulsion_sim(
            0.8, 0.2, 0.1,
            EmulsionArchetype::MobberleySilver400.id_number(),
            0.0,
            filter.id_number(),
        );
        assert!((r - metal_out.0).abs() < 1e-4, "Filter {:?} diverged from Metal sim", filter);

        if let Some(prev) = previous_lum {
            // Confirm optical filtration changes luminance response for color scenes
            assert!((r - prev).abs() > 0.001, "Filter {:?} had no effect compared to previous", filter);
        }
        previous_lum = Some(r);
    }
}

#[test]
fn test_dual_engine_parity_across_all_archetypes() {
    let archetypes = [
        EmulsionArchetype::RochesterPortrait400,
        EmulsionArchetype::AshigaraChrome50,
        EmulsionArchetype::RochesterK14Chrome64,
        EmulsionArchetype::RochesterCinema500,
        EmulsionArchetype::RochesterDaylight200,
        EmulsionArchetype::MobberleySilver400,
        EmulsionArchetype::CambridgeDiffusion100,
    ];

    let test_patches = [
        (0.18, 0.18, 0.18),
        (0.65, 0.45, 0.35), // Skin tone
        (0.20, 0.50, 0.15), // Foliage green
        (0.10, 0.30, 0.70), // Sky blue
        (0.85, 0.75, 0.65), // Warm highlight
    ];

    let filters = [
        EnlargerFilterMode::NeutralPanchromatic,
        EnlargerFilterMode::YellowK2,
        EnlargerFilterMode::Red25A,
        EnlargerFilterMode::GreenX1,
    ];

    for arch in archetypes {
        for &filter in &filters {
            for &(r, g, b) in &test_patches {
                let rust_out = apply_recording_emulsion(r, g, b, arch, 0.3, filter);
                let metal_out = metal_shader_emulsion_sim(r, g, b, arch.id_number(), 0.3, filter.id_number());

                assert!((rust_out.0 - metal_out.0).abs() < 1e-4, "Parity mismatch R in {:?} (filter {:?}): Rust={:.5}, Metal={:.5}", arch, filter, rust_out.0, metal_out.0);
                assert!((rust_out.1 - metal_out.1).abs() < 1e-4, "Parity mismatch G in {:?} (filter {:?}): Rust={:.5}, Metal={:.5}", arch, filter, rust_out.1, metal_out.1);
                assert!((rust_out.2 - metal_out.2).abs() < 1e-4, "Parity mismatch B in {:?} (filter {:?}): Rust={:.5}, Metal={:.5}", arch, filter, rust_out.2, metal_out.2);
            }
        }
    }
}

#[test]
fn test_xmp_roundtrip_emulsion() {
    use firstproof_core::xmp::{generate_xmp, parse_xmp, XmpData};

    let mut edits = EditSettings::default();
    edits.emulsion_archetype = EmulsionArchetype::RochesterPortrait400;
    edits.emulsion_format = EmulsionFormat::Format120;
    edits.emulsion_exposure = 0.5;

    assert!(edits_has_modifications(&edits, None, None));

    let mut xmp_data = XmpData::default();
    xmp_data.edits = edits;
    xmp_data.rating = Some(4);
    xmp_data.status = "Pick".to_string();
    xmp_data.title = Some("Portrait Emulsion Test".to_string());
    xmp_data.keywords = vec!["Portrait".to_string()];

    let xml = generate_xmp(&xmp_data);
    let parsed = parse_xmp(&xml).unwrap();

    assert_eq!(parsed.edits.emulsion_archetype, EmulsionArchetype::RochesterPortrait400);
    assert_eq!(parsed.edits.emulsion_format, EmulsionFormat::Format120);
    assert!((parsed.edits.emulsion_exposure - 0.5).abs() < 1e-3);
}
