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
    base_warmth: f32,
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
        let alpha = 0.40 * (grade / 2.0);
        (val + alpha * val * (1.0 - val) * (2.0 * val - 1.0)).clamp(0.0, 1.0)
    };

    let metal_base_warmth = |rgb: (f32, f32, f32), warmth: f32| -> (f32, f32, f32) {
        if warmth.abs() < 0.001 {
            return rgb;
        }
        let lum = 0.2126 * rgb.0 + 0.7152 * rgb.1 + 0.0722 * rgb.2;
        let highlight_weight = lum.powf(0.6).clamp(0.0, 1.0);
        let factor = (warmth / 100.0) * highlight_weight;

        let mult = if factor > 0.0 {
            (
                1.0 + factor * 0.060,
                1.0 + factor * 0.025,
                1.0 - factor * 0.045,
            )
        } else {
            let abs_f = -factor;
            (
                1.0 - abs_f * 0.025,
                1.0 - abs_f * 0.008,
                1.0 + abs_f * 0.045,
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

            let lum = 0.2126 * r_p + 0.7152 * g_p + 0.0722 * b_p;
            let sat_scale = 0.70 + 0.35 * depth;
            let mut r_out = (lum + (r_p - lum) * sat_scale).clamp(0.0, 1.0);
            let mut g_out = (lum + (g_p - lum) * sat_scale).clamp(0.0, 1.0);
            let mut b_out = (lum + (b_p - lum) * sat_scale).clamp(0.0, 1.0);

            let dye_density = 0.04 * (1.0 - lum).max(0.0) * depth;
            r_out = (r_out * (1.0 - dye_density * 0.4)).clamp(0.0, 1.0);
            g_out = (g_out * (1.0 - dye_density * 0.6)).clamp(0.0, 1.0);
            b_out = (b_out * (1.0 - dye_density * 0.9)).clamp(0.0, 1.0);

            metal_base_warmth((r_out, g_out, b_out), base_warmth)
        }
        2 => { // SilverDyeBleach
            let r_g = metal_paper_grade(r, paper_grade);
            let g_g = metal_paper_grade(g, paper_grade);
            let b_g = metal_paper_grade(b, paper_grade);

            let shadow_pow = 1.0 + 0.12 * depth;
            let r_p = r_g.powf(shadow_pow);
            let g_p = g_g.powf(shadow_pow);
            let b_p = b_g.powf(shadow_pow);

            let lum = 0.2126 * r_p + 0.7152 * g_p + 0.0722 * b_p;
            let azo_sat = 0.80 + 0.30 * depth;

            let r_out = (lum + (r_p - lum) * azo_sat).clamp(0.0, 1.0);
            let g_out = (lum + (g_p - lum) * azo_sat).clamp(0.0, 1.0);
            let b_out = (lum + (b_p - lum) * azo_sat).clamp(0.0, 1.0);

            metal_base_warmth((r_out, g_out, b_out), base_warmth)
        }
        3 => { // FineArtCottonRag
            let r_g = metal_paper_grade(r, paper_grade);
            let g_g = metal_paper_grade(g, paper_grade);
            let b_g = metal_paper_grade(b, paper_grade);

            let matte_floor = 0.055 - 0.015 * depth;
            let r_p = r_g + matte_floor * (1.0 - r_g) * (1.0 - r_g);
            let g_p = g_g + matte_floor * (1.0 - g_g) * (1.0 - g_g);
            let b_p = b_g + matte_floor * (1.0 - b_g) * (1.0 - b_g);

            let lum = 0.2126 * r_p + 0.7152 * g_p + 0.0722 * b_p;
            let pigment_sat = 0.75 + 0.25 * depth;

            let r_out = (lum + (r_p - lum) * pigment_sat).clamp(0.0, 1.0);
            let g_out = (lum + (g_p - lum) * pigment_sat).clamp(0.0, 1.0);
            let b_out = (lum + (b_p - lum) * pigment_sat).clamp(0.0, 1.0);

            metal_base_warmth((r_out, g_out, b_out), base_warmth + 15.0)
        }
        4 => { // BarytaSilverGelatin
            let raw_lum = (r * filter_weights.0 + g * filter_weights.1 + b * filter_weights.2).clamp(0.0, 1.0);
            let silver = metal_paper_grade(raw_lum, paper_grade);

            let dmax_gamma = 1.0 + 0.08 * depth;
            let silver_toned = silver.powf(dmax_gamma);

            metal_base_warmth((silver_toned, silver_toned, silver_toned), base_warmth)
        }
        5 => { // WarmtoneChlorobromide
            let raw_lum = (r * filter_weights.0 + g * filter_weights.1 + b * filter_weights.2).clamp(0.0, 1.0);
            let silver = metal_paper_grade(raw_lum, paper_grade);

            let shadow_pow = 1.0 + 0.10 * depth;
            let silver_deep = silver.powf(shadow_pow);

            let mid_mask = 4.0 * silver * (1.0 - silver);
            let shadow_density = (1.0 - silver).max(0.0);

            let r_tone = silver_deep + depth * (mid_mask * 0.030 + shadow_density * 0.020);
            let g_tone = silver_deep + depth * (mid_mask * 0.012 - shadow_density * 0.015);
            let b_tone = silver_deep + depth * (-mid_mask * 0.018 + shadow_density * 0.030);

            let toned = (r_tone.clamp(0.0, 1.0), g_tone.clamp(0.0, 1.0), b_tone.clamp(0.0, 1.0));
            metal_base_warmth(toned, base_warmth)
        }
        6 => { // PlatinumPalladium
            let raw_lum = (r * filter_weights.0 + g * filter_weights.1 + b * filter_weights.2).clamp(0.0, 1.0);
            let silver = metal_paper_grade(raw_lum, paper_grade);

            let matte_floor = 0.035 * depth;
            let silver_base = matte_floor + (1.0 - matte_floor) * silver;

            let warm_lift = (1.0 - silver_base.powf(0.6)).max(0.0) * depth;
            let r_tone = silver_base + warm_lift * 0.045;
            let g_tone = silver_base + warm_lift * 0.028;
            let b_tone = silver_base - warm_lift * 0.020;

            let toned = (r_tone.clamp(0.0, 1.0), g_tone.clamp(0.0, 1.0), b_tone.clamp(0.0, 1.0));
            metal_base_warmth(toned, base_warmth + 10.0 * depth)
        }
        7 => { // Cyanotype
            let raw_lum = (r * filter_weights.0 + g * filter_weights.1 + b * filter_weights.2).clamp(0.0, 1.0);
            let silver = metal_paper_grade(raw_lum, paper_grade);

            let density = 1.0 - silver;

            let r_p = silver * (1.0 - 0.75 * depth * density);
            let g_p = silver * (1.0 - 0.35 * depth * density) + 0.02 * depth * density;
            let b_p = silver * (1.0 - 0.05 * depth * density) + 0.07 * depth * density;

            let toned = (r_p.clamp(0.0, 1.0), g_p.clamp(0.0, 1.0), b_p.clamp(0.0, 1.0));
            metal_base_warmth(toned, base_warmth + 8.0 * depth)
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
            50.0,
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
        PrintMediaMode::FineArtCottonRag,
        PrintMediaMode::BarytaSilverGelatin,
        PrintMediaMode::WarmtoneChlorobromide,
        PrintMediaMode::PlatinumPalladium,
        PrintMediaMode::Cyanotype,
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
    let base_warmths = [-40.0, 0.0, 40.0];
    let process_depths = [50.0, 100.0];

    for &media in &media_modes {
        for &filter in &filter_modes {
            for &grade in &paper_grades {
                for &warmth in &base_warmths {
                    for &depth in &process_depths {
                        for &(r, g, b) in &test_colors {
                            let (rust_r, rust_g, rust_b) = apply_print_media(
                                r, g, b, media, filter, grade, warmth, depth,
                            );
                            let (metal_r, metal_g, metal_b) = metal_shader_print_media_sim(
                                r, g, b, media.id_number(), filter.id_number(), grade, warmth, depth,
                            );

                            let diff_r = (rust_r - metal_r).abs();
                            let diff_g = (rust_g - metal_g).abs();
                            let diff_b = (rust_b - metal_b).abs();

                            assert!(
                                diff_r < 1e-5 && diff_g < 1e-5 && diff_b < 1e-5,
                                "Parity mismatch for {:?} (filter {:?}, grade {}, warmth {}, depth {}): Rust=({}, {}, {}), Metal=({}, {}, {})",
                                media, filter, grade, warmth, depth,
                                rust_r, rust_g, rust_b,
                                metal_r, metal_g, metal_b
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_cotton_rag_matte_shadow_floor() {
    // Archival Cotton Rag has a matte shadow floor of Dmax ~ 1.65 (floor >= 0.045)
    let (r, g, b) = apply_print_media(
        0.0, 0.0, 0.0,
        PrintMediaMode::FineArtCottonRag,
        EnlargerFilterMode::NeutralPanchromatic,
        0.0,
        0.0, // Baseline warmth
        100.0,
    );
    assert!(r >= 0.040, "Cotton rag black floor R must be lifted to matte floor: {}", r);
    assert!(g >= 0.040, "Cotton rag black floor G must be lifted to matte floor: {}", g);
    assert!(b >= 0.035, "Cotton rag black floor B must be lifted to matte floor: {}", b);
}

#[test]
fn test_cyanotype_prussian_blue_chemistry() {
    // Historic Cyanotype produces intense Prussian blue shadows where Red is heavily absorbed
    let (r, g, b) = apply_print_media(
        0.05, 0.05, 0.05, // Deep shadow
        PrintMediaMode::Cyanotype,
        EnlargerFilterMode::NeutralPanchromatic,
        0.0,
        0.0,
        100.0,
    );
    assert!(b > r, "Cyanotype must have dominant blue over red in shadow density: R={}, B={}", r, b);
    assert!(b > g, "Cyanotype must have dominant blue over green in shadow density: G={}, B={}", g, b);
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
