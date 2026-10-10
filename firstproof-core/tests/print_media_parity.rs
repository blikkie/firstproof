//! Dual-Engine Math Parity & Sensitometric Tests for Curated Fine-Art Print Media
//!
//! Verifies:
//! 1. Rust CPU export (`apply_print_media`) and Metal GPU shader (`applyPrintMedia`) evaluate identical math.
//! 2. Clean bypass invariant: when PrintMediaMode::None, input RGB is passed through bit-for-bit.
//! 3. Monochrome media properly collapses RGB to toned monochromes via enlarger filtration.
//! 4. XMP roundtrip serialization and deserialization preserves all print media settings.

use firstproof_core::color::print_media::{
    apply_print_media, EnlargerFilterMode, PrintMediaMode,
};
use firstproof_core::{edits_has_modifications, EditSettings};

/// Exact simulation of the Metal shader function `applyPrintMedia` from ColorScience.metal
fn metal_shader_print_media_sim(
    r: f32,
    g: f32,
    b: f32,
    media_mode: i32,
    filter_mode: i32,
    paper_grade: f32,
    process_depth: f32,
) -> (f32, f32, f32) {
    if media_mode == 0 {
        return (r, g, b);
    }

    let depth = (process_depth / 100.0).clamp(0.0, 1.0);

    let metal_paper_grade = |val: f32, grade: f32| -> f32 {
        if grade.abs() < 0.001 {
            return val;
        }
        let alpha = (grade / 2.0) * if grade >= 0.0 { 0.60 } else { 0.40 };
        (val + alpha * val * (1.0 - val) * (2.0 * val - 1.0)).clamp(0.0, 1.0)
    };

    let metal_base_warmth = |rgb: (f32, f32, f32), warmth: f32| -> (f32, f32, f32) {
        if warmth.abs() < 0.001 {
            return rgb;
        }
        let lum = 0.2627 * rgb.0 + 0.6780 * rgb.1 + 0.0593 * rgb.2;
        let highlight_weight = lum.powf(0.6).clamp(0.0, 1.0);
        let factor = (warmth / 100.0) * highlight_weight;

        let mult = if factor > 0.0 {
            (
                1.0,
                1.0 - factor * 0.013849,
                1.0 - factor * 0.051111,
            )
        } else {
            let abs_f = -factor;
            (
                1.0 - abs_f * 0.033699,
                1.0 - abs_f * 0.031749,
                1.0,
            )
        };
        ((rgb.0 * mult.0).clamp(0.0, 1.0), (rgb.1 * mult.1).clamp(0.0, 1.0), (rgb.2 * mult.2).clamp(0.0, 1.0))
    };

    let filter_weights = match filter_mode {
        1 => (0.35f32, 0.60f32, 0.05f32), // YellowK2
        2 => (0.55f32, 0.42f32, 0.03f32), // Orange16
        3 => (0.85f32, 0.15f32, 0.00f32), // Red25A
        4 => (0.15f32, 0.80f32, 0.05f32), // GreenX1
        5 => (0.00f32, 0.60f32, 0.40f32), // Orthochromatic
        _ => (0.2627f32, 0.6780f32, 0.0593f32), // Neutral
    };

    match media_mode {
        1 => { // ChromogenicTypeC
            let r_g = metal_paper_grade(r, paper_grade);
            let g_g = metal_paper_grade(g, paper_grade);
            let b_g = metal_paper_grade(b, paper_grade);

            let r_p = if r_g > 0.70 {
                let u = ((r_g - 0.70) / 0.30).clamp(0.0, 1.0);
                r_g - 0.045 * depth * u * u * (3.0 - 2.0 * u)
            } else { r_g };
            let g_p = if g_g > 0.70 {
                let u = ((g_g - 0.70) / 0.30).clamp(0.0, 1.0);
                g_g - 0.045 * depth * u * u * (3.0 - 2.0 * u)
            } else { g_g };
            let b_p = if b_g > 0.70 {
                let u = ((b_g - 0.70) / 0.30).clamp(0.0, 1.0);
                b_g - 0.045 * depth * u * u * (3.0 - 2.0 * u)
            } else { b_g };

            let lum = 0.2627 * r_p + 0.6780 * g_p + 0.0593 * b_p;
            let sat_scale = 0.70 + 0.35 * depth;
            let mut r_out = (lum + (r_p - lum) * sat_scale).clamp(0.0, 1.0);
            let mut g_out = (lum + (g_p - lum) * sat_scale).clamp(0.0, 1.0);
            let mut b_out = (lum + (b_p - lum) * sat_scale).clamp(0.0, 1.0);

            let dye_density = 0.04 * (1.0 - lum).max(0.0) * depth;
            r_out = (r_out * (1.0 - dye_density * 0.4)).clamp(0.0, 1.0);
            g_out = (g_out * (1.0 - dye_density * 0.6)).clamp(0.0, 1.0);
            b_out = (b_out * (1.0 - dye_density * 0.9)).clamp(0.0, 1.0);

            (r_out, g_out, b_out)
        }
        2 => { // SilverDyeBleach
            let r_g = metal_paper_grade(r, paper_grade);
            let g_g = metal_paper_grade(g, paper_grade);
            let b_g = metal_paper_grade(b, paper_grade);

            let shadow_pow = 1.0 + 0.12 * depth;
            let r_p = r_g.powf(shadow_pow);
            let g_p = g_g.powf(shadow_pow);
            let b_p = b_g.powf(shadow_pow);

            let lum = 0.2627 * r_p + 0.6780 * g_p + 0.0593 * b_p;
            let azo_sat = 0.80 + 0.30 * depth;

            let r_out = (lum + (r_p - lum) * azo_sat).clamp(0.0, 1.0);
            let g_out = (lum + (g_p - lum) * azo_sat).clamp(0.0, 1.0);
            let b_out = (lum + (b_p - lum) * azo_sat).clamp(0.0, 1.0);

            (r_out, g_out, b_out)
        }
        3 => { // BarytaSilverGelatin
            let raw_lum = (r * filter_weights.0 + g * filter_weights.1 + b * filter_weights.2).clamp(0.0, 1.0);
            let silver = metal_paper_grade(raw_lum, paper_grade);

            let dmax_gamma = 1.0 + 0.08 * depth;
            let silver_toned = silver.powf(dmax_gamma);

            (silver_toned, silver_toned, silver_toned)
        }
        4 => { // WarmtoneChlorobromide
            let raw_lum = (r * filter_weights.0 + g * filter_weights.1 + b * filter_weights.2).clamp(0.0, 1.0);
            let silver = metal_paper_grade(raw_lum, paper_grade);

            let shadow_pow = 1.0 + 0.10 * depth;
            let silver_deep = silver.powf(shadow_pow);

            let mid_mask = 4.0 * silver * (1.0 - silver);
            let shadow_mask = 4.0 * silver * (1.0 - silver) * (1.0 - silver);

            let r_tone = silver_deep + depth * (mid_mask * 0.025 + shadow_mask * 0.020);
            let g_tone = silver_deep + depth * (mid_mask * 0.010 - shadow_mask * 0.012);
            let b_tone = silver_deep + depth * (-mid_mask * 0.015 + shadow_mask * 0.025);

            let toned = (r_tone.clamp(0.0, 1.0), g_tone.clamp(0.0, 1.0), b_tone.clamp(0.0, 1.0));
            metal_base_warmth(toned, 5.0)
        }
        5 => { // PlatinumPalladium
            let raw_lum = (r * filter_weights.0 + g * filter_weights.1 + b * filter_weights.2).clamp(0.0, 1.0);
            let silver = metal_paper_grade(raw_lum, paper_grade);

            let pt_scale = silver + depth * 0.10 * silver * (1.0 - silver) * (1.0 - silver);
            let mid_warmth = 4.0 * silver * (1.0 - silver);
            let r_tone = pt_scale + depth * mid_warmth * 0.040;
            let g_tone = pt_scale + depth * mid_warmth * 0.024;
            let b_tone = pt_scale - depth * mid_warmth * 0.016;

            let toned = (r_tone.clamp(0.0, 1.0), g_tone.clamp(0.0, 1.0), b_tone.clamp(0.0, 1.0));
            metal_base_warmth(toned, 10.0 * depth)
        }
        6 => { // Cyanotype
            let raw_lum = (r * filter_weights.0 + g * filter_weights.1 + b * filter_weights.2).clamp(0.0, 1.0);
            let silver = metal_paper_grade(raw_lum, paper_grade);

            let density = 1.0 - silver;
            let r_p = silver * (1.0 - 0.85 * depth * density);
            let g_p = silver * (1.0 - 0.45 * depth * density);
            let b_p = silver * (1.0 - 0.05 * depth * density);

            let toned = (r_p.clamp(0.0, 1.0), g_p.clamp(0.0, 1.0), b_p.clamp(0.0, 1.0));
            metal_base_warmth(toned, 8.0 * depth)
        }
        7 => { // BamakoSilverContact
            let raw_lum = (r * filter_weights.0 + g * filter_weights.1 + b * filter_weights.2).clamp(0.0, 1.0);
            let graded = metal_paper_grade(raw_lum, paper_grade);
            let c = graded.clamp(0.0, 1.0);
            let s_lum = (c + 0.22 * c * (1.0 - c) * (2.0 * c - 1.0) + 0.14 * c * (1.0 - c)).clamp(0.0, 1.0);
            let ferro = s_lum.powf(1.0 + 0.15 * depth);
            let silver = (ferro * depth + graded * (1.0 - depth)).clamp(0.0, 1.0);
            (silver, silver, silver)
        }
        _ => (r, g, b),
    }
}

#[test]
fn test_clean_bypass_when_none() {
    let test_colors = [
        (0.0, 0.0, 0.0),
        (0.18, 0.18, 0.18),
        (0.8, 0.4, 0.2),
        (0.1, 0.5, 0.9),
        (1.0, 1.0, 1.0),
    ];

    for (r, g, b) in test_colors {
        let (out_r, out_g, out_b) = apply_print_media(
            r, g, b,
            PrintMediaMode::None,
            EnlargerFilterMode::NeutralPanchromatic,
            1.5,
            80.0,
        );
        assert_eq!(out_r, r, "Bypass must pass R unaltered");
        assert_eq!(out_g, g, "Bypass must pass G unaltered");
        assert_eq!(out_b, b, "Bypass must pass B unaltered");
    }
}

#[test]
fn test_dual_engine_parity_across_all_archetypes() {
    let test_colors = [
        (0.02, 0.02, 0.02), // Deep shadow
        (0.18, 0.18, 0.18), // Middle gray
        (0.50, 0.50, 0.50), // Midtone
        (0.85, 0.85, 0.85), // Highlight
        (0.75, 0.55, 0.45), // Caucasian skin tone
        (0.20, 0.45, 0.85), // Blue sky
        (0.25, 0.70, 0.25), // Foliage
    ];

    let media_modes = [
        PrintMediaMode::ChromogenicTypeC,
        PrintMediaMode::SilverDyeBleach,
        PrintMediaMode::BarytaSilverGelatin,
        PrintMediaMode::WarmtoneChlorobromide,
        PrintMediaMode::PlatinumPalladium,
        PrintMediaMode::Cyanotype,
        PrintMediaMode::BamakoSilverContact,
    ];

    let filter_modes = [
        EnlargerFilterMode::NeutralPanchromatic,
        EnlargerFilterMode::YellowK2,
        EnlargerFilterMode::Orange16,
        EnlargerFilterMode::Red25A,
        EnlargerFilterMode::GreenX1,
        EnlargerFilterMode::Orthochromatic,
    ];

    let paper_grades = [-1.5, 0.0, 1.5];
    let process_depths = [50.0, 100.0];

    for &media in &media_modes {
        for &filter in &filter_modes {
            for &grade in &paper_grades {
                for &depth in &process_depths {
                    for &(r, g, b) in &test_colors {
                        let (rust_r, rust_g, rust_b) = apply_print_media(
                            r, g, b, media, filter, grade, depth,
                        );
                        let (metal_r, metal_g, metal_b) = metal_shader_print_media_sim(
                            r, g, b, media.id_number(), filter.id_number(), grade, depth,
                        );

                        let diff_r = (rust_r - metal_r).abs();
                        let diff_g = (rust_g - metal_g).abs();
                        let diff_b = (rust_b - metal_b).abs();

                        assert!(
                            diff_r < 1e-5 && diff_g < 1e-5 && diff_b < 1e-5,
                            "Parity mismatch for {:?} (filter {:?}, grade {}, depth {}): Rust=({}, {}, {}), Metal=({}, {}, {})",
                            media, filter, grade, depth,
                            rust_r, rust_g, rust_b,
                            metal_r, metal_g, metal_b
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_cyanotype_prussian_blue_chemistry() {
    // Historic Cyanotype produces intense Prussian blue shadows where Red is heavily absorbed
    let (r, g, b) = apply_print_media(
        0.05, 0.05, 0.05, // Deep shadow
        PrintMediaMode::Cyanotype,
        EnlargerFilterMode::NeutralPanchromatic,
        0.0,
        100.0,
    );
    assert!(b > r, "Cyanotype must have dominant blue over red in shadow density: R={}, B={}", r, b);
    assert!(b > g, "Cyanotype must have dominant blue over green in shadow density: G={}, B={}", g, b);
}

#[test]
fn test_bw_print_media_boundary_anchoring_and_toning() {
    let bw_media = [
        PrintMediaMode::PlatinumPalladium,
        PrintMediaMode::WarmtoneChlorobromide,
        PrintMediaMode::Cyanotype,
        PrintMediaMode::BarytaSilverGelatin,
        PrintMediaMode::BamakoSilverContact,
    ];

    // 1. Strict Boundary Anchoring Invariant: f(0.0) == 0.0
    // Zero additive floor lifts; terminal Dmax must reach absolute black for printability.
    for &media in &bw_media {
        let (r, g, b) = apply_print_media(
            0.0, 0.0, 0.0,
            media,
            EnlargerFilterMode::NeutralPanchromatic,
            0.0,
            100.0,
        );
        assert_eq!(
            (r, g, b), (0.0, 0.0, 0.0),
            "{:?} must strictly anchor to (0.0, 0.0, 0.0) at maximum density: got ({}, {}, {})",
            media, r, g, b
        );
    }

    // 2. Platinum / Palladium: Luminous open shadow scale + warm palladium glow
    let (pt_r, pt_g, pt_b) = apply_print_media(
        0.05, 0.05, 0.05,
        PrintMediaMode::PlatinumPalladium,
        EnlargerFilterMode::NeutralPanchromatic,
        0.0,
        100.0,
    );
    assert!(
        pt_r > 0.05,
        "Platinum/Palladium must lift shadow scale for luminous open shadows: R={}", pt_r
    );
    assert!(
        pt_r > pt_g && pt_g > pt_b,
        "Platinum/Palladium must exhibit warm amber glow in shadows: R={}, G={}, B={}",
        pt_r, pt_g, pt_b
    );

    // 3. Warmtone Chlorobromide: Selenium plum undertone in deep shadows
    let (ch_r, ch_g, ch_b) = apply_print_media(
        0.10, 0.10, 0.10,
        PrintMediaMode::WarmtoneChlorobromide,
        EnlargerFilterMode::NeutralPanchromatic,
        0.0,
        100.0,
    );
    assert!(
        ch_r > ch_g && ch_b > ch_g,
        "Warmtone Chlorobromide must exhibit selenium plum undertone (R, B > G) in shadows: R={}, G={}, B={}",
        ch_r, ch_g, ch_b
    );
}

#[test]
fn test_xmp_roundtrip_print_media() {
    use firstproof_core::xmp::{generate_xmp, parse_xmp, XmpData};

    let mut data = XmpData::default();
    data.edits.print_media = PrintMediaMode::WarmtoneChlorobromide;
    data.edits.enlarger_filter = EnlargerFilterMode::Red25A;
    data.edits.print_paper_grade = 1.2;
    data.edits.print_base_warmth = 25.0;
    data.edits.print_process_depth = 85.0;

    let xmp_str = generate_xmp(&data);

    let parsed = parse_xmp(&xmp_str)
        .expect("XMP deserialization must succeed");

    assert_eq!(parsed.edits.print_media, PrintMediaMode::WarmtoneChlorobromide);
    assert_eq!(parsed.edits.enlarger_filter, EnlargerFilterMode::Red25A);
    assert!((parsed.edits.print_paper_grade - 1.2).abs() < 1e-3);
    assert!((parsed.edits.print_base_warmth - 25.0).abs() < 1e-3);
    assert!((parsed.edits.print_process_depth - 85.0).abs() < 1e-3);
}

#[test]
fn test_has_modifications_with_print_media() {
    let mut edits = EditSettings::default();
    assert!(!edits_has_modifications(&edits, None, None));

    edits.print_media = PrintMediaMode::ChromogenicTypeC;
    assert!(edits_has_modifications(&edits, None, None));

    edits.print_media = PrintMediaMode::None;
    assert!(!edits_has_modifications(&edits, None, None));

    edits.print_paper_grade = 0.5;
    assert!(edits_has_modifications(&edits, None, None));
}

#[test]
fn test_bamako_silver_contact_deep_dmax_and_ferrotype_depth() {
    // Bamako contact prints exhibit deep specular blacks and Zone III-V expansion
    let (r, g, b) = apply_print_media(
        0.18, 0.18, 0.18, // Mid gray
        PrintMediaMode::BamakoSilverContact,
        EnlargerFilterMode::NeutralPanchromatic,
        0.0,
        100.0,
    );
    // Neutral axis must be preserved
    assert_eq!(r, g);
    assert_eq!(g, b);
    assert!(r > 0.0 && r < 1.0);

    // Deep shadow should have high Dmax
    let (r_sh, g_sh, b_sh) = apply_print_media(
        0.02, 0.02, 0.02,
        PrintMediaMode::BamakoSilverContact,
        EnlargerFilterMode::NeutralPanchromatic,
        0.0,
        100.0,
    );
    assert_eq!(r_sh, g_sh);
    assert_eq!(g_sh, b_sh);
    assert!(r_sh < 0.02, "Ferrotype process must deepen maximum shadow density");
}

#[test]
fn test_iso_6846_paper_grade_strict_monotonicity_and_anchoring() {
    use firstproof_core::color::print_media::apply_paper_grade;

    // Test across grade range from -2.0 (Extra Soft / Grade 0) to +2.0 (Hard / Grade 4)
    let grades = [-2.0, -1.5, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0];

    for &grade in &grades {
        // 1. Boundary and midtone anchoring invariants
        assert_eq!(apply_paper_grade(0.0, grade), 0.0);
        assert!((apply_paper_grade(0.5, grade) - 0.5).abs() < 1e-6);
        assert_eq!(apply_paper_grade(1.0, grade), 1.0);

        // 2. Strict monotonicity check across 1000 steps
        let steps = 1000;
        let mut prev = 0.0;
        for i in 1..=steps {
            let x = i as f32 / steps as f32;
            let y = apply_paper_grade(x, grade);
            assert!(
                y > prev,
                "Strict monotonicity violated at x = {} for grade {}: y ({}) <= prev ({})",
                x, grade, y, prev
            );
            prev = y;
        }
    }
}

#[test]
fn test_iso_6846_contrast_ratios() {
    use firstproof_core::color::print_media::apply_paper_grade;

    // Evaluate contrast between 25% shadow and 75% highlight
    let contrast_box_speed = apply_paper_grade(0.75, 0.0) - apply_paper_grade(0.25, 0.0);
    assert_eq!(contrast_box_speed, 0.50);

    // Grade 4 (+2.0): alpha = +0.60 => contrast slope delta = 0.50 + 0.1875(0.60) = 0.6125 (ratio 1.225)
    let contrast_grade_4 = apply_paper_grade(0.75, 2.0) - apply_paper_grade(0.25, 2.0);
    let ratio_4 = contrast_grade_4 / contrast_box_speed;
    assert!(
        (ratio_4 - 1.225).abs() < 0.001,
        "Grade 4 contrast ratio must equal 1.225, got {}",
        ratio_4
    );

    // Grade 0 (-2.0): alpha = -0.40 => contrast slope delta = 0.50 - 0.1875(0.40) = 0.425 (ratio 0.850)
    let contrast_grade_0 = apply_paper_grade(0.75, -2.0) - apply_paper_grade(0.25, -2.0);
    let ratio_0 = contrast_grade_0 / contrast_box_speed;
    assert!(
        (ratio_0 - 0.850).abs() < 0.001,
        "Grade 0 contrast ratio must equal 0.850, got {}",
        ratio_0
    );
}

#[test]
fn test_sensitometric_substrate_reflectance_white_points() {
    use firstproof_core::color::print_media::apply_base_warmth;

    // 1. True blacks remain anchored at 0.0 under any warmth
    let (r0_warm, g0_warm, b0_warm) = apply_base_warmth(0.0, 0.0, 0.0, 100.0);
    assert_eq!((r0_warm, g0_warm, b0_warm), (0.0, 0.0, 0.0));

    let (r0_cool, g0_cool, b0_cool) = apply_base_warmth(0.0, 0.0, 0.0, -100.0);
    assert_eq!((r0_cool, g0_cool, b0_cool), (0.0, 0.0, 0.0));

    // 2. Pure specular white (1.0, 1.0, 1.0) under warmth +100 (Hahnemühle Photo Rag 308)
    // Normalized W_base = [1.0000, 0.986151, 0.948889]
    let (r_h, g_h, b_h) = apply_base_warmth(1.0, 1.0, 1.0, 100.0);
    assert_eq!(r_h, 1.0);
    assert!((g_h - 0.986151).abs() < 1e-4, "Green delta mismatch: {}", g_h);
    assert!((b_h - 0.948889).abs() < 1e-4, "Blue delta mismatch: {}", b_h);

    // 3. Pure specular white (1.0, 1.0, 1.0) under warmth -100 (Bright White Fine Art OBA)
    // Normalized W_base = [0.966301, 0.968251, 1.0000]
    let (r_c, g_c, b_c) = apply_base_warmth(1.0, 1.0, 1.0, -100.0);
    assert!((r_c - 0.966301).abs() < 1e-4, "Red delta mismatch: {}", r_c);
    assert!((g_c - 0.968251).abs() < 1e-4, "Green delta mismatch: {}", g_c);
    assert_eq!(b_c, 1.0);
}
