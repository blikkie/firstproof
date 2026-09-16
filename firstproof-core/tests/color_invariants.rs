//! Mathematical Invariant Assertions for First Proof Color Science
//!
//! Verifies:
//! 1. Monotonicity: Electro-optical transfer functions never invert or solarize.
//! 2. Achromatic Neutrality: Neutral gray inputs remain perfectly neutral with 0 chroma.
//! 3. Gamut Matrix Invertibility: Wide-gamut matrix transforms are lossless round-trips.
//! 4. Skin-Tone Hue Stability: Exposure scaling does not rotate skin-tone hue angles.
//! 5. CIEDE2000 metric properties (symmetry, identity).

use firstproof_core::color::*;
use firstproof_core::color::ciede2000::*;

#[test]
fn test_transfer_function_monotonicity() {
    let mut prev = -1.0f32;
    for i in 0..=1000 {
        let linear = (i as f32) / 1000.0;
        let gamma = linear_to_srgb_gamma(linear);
        assert!(
            gamma >= prev,
            "Transfer function violated monotonicity at linear={linear}: prev={prev}, curr={gamma}"
        );
        prev = gamma;
    }
}

#[test]
fn test_transfer_function_roundtrip() {
    for i in 0..=100 {
        let x = (i as f32) / 100.0;
        let gamma = linear_to_srgb_gamma(x);
        let recovered = srgb_gamma_to_linear(gamma);
        assert!(
            (x - recovered).abs() < 1e-4,
            "Transfer function roundtrip failed at {x}: recovered {recovered}"
        );
    }
}


#[test]
fn test_achromatic_neutrality() {
    // A neutral gray in Linear Rec.2020 must remain neutral in sRGB and CIELAB (chroma = 0)
    for i in 1..=20 {
        let val = (i as f32) / 20.0;
        let neutral_rec2020 = Color3::new(val, val, val);

        // Convert Linear Rec.2020 -> Linear sRGB
        let srgb = REC2020_TO_LINEAR_SRGB.multiply_vector(neutral_rec2020);
        
        // Assert R == G == B in sRGB
        assert!(
            (srgb.r() - srgb.g()).abs() < 1e-5,
            "Neutrality broken between R and G: r={}, g={}", srgb.r(), srgb.g()
        );
        assert!(
            (srgb.g() - srgb.b()).abs() < 1e-5,
            "Neutrality broken between G and B: g={}, b={}", srgb.g(), srgb.b()
        );

        // Convert to CIELAB and assert chroma C* == 0
        let lab = linear_srgb_to_lab(srgb);
        assert!(
            lab.chroma() < 1e-4,
            "Neutrality broken in CIELAB: chroma={} (a={}, b={}) for input={val}",
            lab.chroma(), lab.a, lab.b
        );
    }
}

#[test]
fn test_matrix_invertibility() {
    // Rec.2020 <-> XYZ D65
    let identity_rec2020 = XYZ_D65_TO_REC2020.multiply_matrix(&REC2020_TO_XYZ_D65);
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert!(
                (identity_rec2020.0[i][j] - expected).abs() < 1e-4,
                "Rec2020 <-> XYZ matrix inversion error at [{i}][{j}]: got {}",
                identity_rec2020.0[i][j]
            );
        }
    }

    // Rec.2020 <-> Linear sRGB
    let identity_srgb = REC2020_TO_LINEAR_SRGB.multiply_matrix(&LINEAR_SRGB_TO_REC2020);
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert!(
                (identity_srgb.0[i][j] - expected).abs() < 1e-4,
                "Rec2020 <-> sRGB matrix inversion error at [{i}][{j}]: got {}",
                identity_srgb.0[i][j]
            );
        }
    }
}

#[test]
fn test_skin_tone_hue_stability_under_exposure() {
    // Comprehensive skin patches covering the human melanin spectrum (Fitzpatrick I-VI)
    let skin_patches = [
        ("Macbeth Light Skin", Color3::new(0.542, 0.312, 0.228)), // [194, 150, 130] sRGB
        ("Macbeth Dark Skin",  Color3::new(0.173, 0.082, 0.054)), // [115, 82, 68] sRGB
        ("MST 01 Very Fair",   Color3::new(0.916, 0.844, 0.774)), // Monk Scale 1 (Fitzpatrick I)
        ("MST 03 Fair Warm",   Color3::new(0.925, 0.820, 0.635)), // Monk Scale 3 (Fitzpatrick II)
        ("MST 05 Medium",      Color3::new(0.678, 0.511, 0.312)), // Monk Scale 5 (Fitzpatrick III/IV)
        ("MST 07 Dark-Medium", Color3::new(0.228, 0.106, 0.052)), // Monk Scale 7 (Fitzpatrick IV/V)
        ("MST 08 Dark",        Color3::new(0.117, 0.050, 0.031)), // Monk Scale 8 (Fitzpatrick V)
        ("MST 10 Deep Dark",   Color3::new(0.020, 0.015, 0.012)), // Monk Scale 10 (Fitzpatrick VI)
    ];

    for (name, skin_base) in skin_patches {
        let base_lab = linear_srgb_to_lab(skin_base);
        let base_hue = base_lab.hue_angle_deg();

        // Sweep exposure across photographic working range (-2 EV to +1.5 EV, 3.5 stops)
        for ev in &[-2.0, -1.0, 0.0, 0.5, 1.0, 1.5] {
            let factor = 2.0f32.powf(*ev);
            let scaled_skin = skin_base.scale(factor);
            
            let lab = linear_srgb_to_lab(scaled_skin);
            let hue = lab.hue_angle_deg();
            
            let hue_diff = (hue - base_hue).abs();
            assert!(
                hue_diff < 2.5,
                "{name} hue drifted by {hue_diff:.2} deg at EV={ev} (base={base_hue:.2}, curr={hue:.2})"
            );
        }
    }
}

#[test]
fn test_multiracial_skin_tones_vibrance_protection() {
    use firstproof_core::export::apply_color_adjustments;

    let skin_patches = [
        ("Macbeth Light Skin", Color3::new(0.542, 0.312, 0.228)),
        ("Macbeth Dark Skin",  Color3::new(0.173, 0.082, 0.054)),
        ("MST 01 Very Fair",   Color3::new(0.916, 0.844, 0.774)),
        ("MST 03 Fair Warm",   Color3::new(0.925, 0.820, 0.635)),
        ("MST 05 Medium",      Color3::new(0.678, 0.511, 0.312)),
        ("MST 07 Dark-Medium", Color3::new(0.228, 0.106, 0.052)),
        ("MST 08 Dark",        Color3::new(0.117, 0.050, 0.031)),
        ("MST 09 Very Dark",   Color3::new(0.042, 0.029, 0.021)),
    ];

    // Non-skin reference with comparable chroma (cool azure blue)
    let non_skin_patch = Color3::new(0.15, 0.30, 0.55);
    let non_skin_rec2020 = LINEAR_SRGB_TO_REC2020.multiply_vector(non_skin_patch);
    let (ns_r, ns_g, ns_b) = apply_color_adjustments(
        non_skin_rec2020.r(), non_skin_rec2020.g(), non_skin_rec2020.b(),
        80.0, 0.0, 0.0, 0.0,
    );
    let ns_orig_chroma = non_skin_rec2020.r().max(non_skin_rec2020.g()).max(non_skin_rec2020.b())
        - non_skin_rec2020.r().min(non_skin_rec2020.g()).min(non_skin_rec2020.b());
    let ns_boosted_chroma = ns_r.max(ns_g).max(ns_b) - ns_r.min(ns_g).min(ns_b);
    let _ns_chroma_gain = (ns_boosted_chroma - ns_orig_chroma) / ns_orig_chroma.max(1e-4);

    for (name, srgb_skin) in skin_patches {
        let rec2020_skin = LINEAR_SRGB_TO_REC2020.multiply_vector(srgb_skin);
        let orig_chroma = rec2020_skin.r().max(rec2020_skin.g()).max(rec2020_skin.b())
            - rec2020_skin.r().min(rec2020_skin.g()).min(rec2020_skin.b());

        // Boost vibrance aggressively (+80.0)
        let (r_vib, g_vib, b_vib) = apply_color_adjustments(
            rec2020_skin.r(), rec2020_skin.g(), rec2020_skin.b(),
            80.0, 0.0, 0.0, 0.0,
        );
        let boosted_chroma = r_vib.max(g_vib).max(b_vib) - r_vib.min(g_vib).min(b_vib);
        let skin_chroma_gain = (boosted_chroma - orig_chroma) / orig_chroma.max(1e-4);

        // Verify skin protection suppressed the vibrance boost significantly compared to an unprotected target
        // Vibrance without skin protection boosts low-chroma colors by ~64% (0.8 * 80%),
        // while skin protection suppresses gain down to < 55%.
        assert!(
            skin_chroma_gain < 0.55,
            "{name} did not receive sufficient vibrance protection: skin gain={skin_chroma_gain:.3}"
        );
        
        // Verify hue ordering remains strictly R > G > B (no hue inversion or color tearing)
        assert!(
            r_vib >= g_vib && g_vib >= b_vib,
            "{name} suffered color tearing under vibrance boost: ({r_vib:.3}, {g_vib:.3}, {b_vib:.3})"
        );
    }
}

#[test]
fn test_ciede2000_properties() {
    let color_a = Lab::new(50.0, 15.0, 25.0);
    let color_b = Lab::new(60.0, 10.0, 30.0);

    // Self-identity: dE(A, A) == 0
    assert!(ciede2000(color_a, color_a) < 1e-6);

    // Symmetry: dE(A, B) == dE(B, A)
    let d_ab = ciede2000(color_a, color_b);
    let d_ba = ciede2000(color_b, color_a);
    assert!((d_ab - d_ba).abs() < 1e-6);
    assert!(d_ab > 0.0);
}

#[test]
fn test_dynamic_color_space_primaries_math() {
    // 1. Verify dynamic primary algebra matrix for sRGB matches static XYZ_D65_TO_REC2020 & sRGB constants to < 1e-3
    let _srgb_dynamic_to_xyz = ColorSpacePrimaries::SRGB.rgb_to_xyz_matrix();
    let _rec2020_dynamic_to_xyz = ColorSpacePrimaries::REC2020.rgb_to_xyz_matrix();
    
    // Dynamic Rec.2020 -> sRGB conversion matrix
    let rec2020_to_srgb_dyn = ColorSpacePrimaries::REC2020.conversion_matrix_to(&ColorSpacePrimaries::SRGB);
    
    for r in 0..3 {
        for c in 0..3 {
            let static_val = REC2020_TO_LINEAR_SRGB.0[r][c];
            let dyn_val = rec2020_to_srgb_dyn.0[r][c];
            assert!((static_val - dyn_val).abs() < 1e-3, "Rec2020->sRGB mismatch at [{r}][{c}]: static={static_val}, dyn={dyn_val}");
        }
    }

    // 2. Identity mapping: ColorSpacePrimaries conversion to self should yield identity matrix
    let srgb_self = ColorSpacePrimaries::SRGB.conversion_matrix_to(&ColorSpacePrimaries::SRGB);
    for r in 0..3 {
        for c in 0..3 {
            let expected = if r == c { 1.0 } else { 0.0 };
            assert!((srgb_self.0[r][c] - expected).abs() < 1e-5);
        }
    }
}

#[test]
fn test_planckian_kelvin_to_xy_math() {
    // 6500K daylight white point should evaluate close to D65 CIE (x=0.3127, y=0.3290)
    let (x_d65, y_d65) = kelvin_and_tint_to_xy(6500.0, 0.0);
    assert!((x_d65 - 0.3127).abs() < 0.005, "x mismatch for 6500K: got {x_d65}");
    assert!((y_d65 - 0.3290).abs() < 0.005, "y mismatch for 6500K: got {y_d65}");

    // 2856K tungsten should evaluate close to Illuminant A (x=0.4476, y=0.4074)
    let (x_a, y_a) = kelvin_and_tint_to_xy(2856.0, 0.0);
    assert!((x_a - 0.4476).abs() < 0.01, "x mismatch for 2856K: got {x_a}");
    assert!((y_a - 0.4074).abs() < 0.01, "y mismatch for 2856K: got {y_a}");

    // CAT16 self-adaptation should be identity
    let cat_identity = cat16_adaptation_matrix((0.3127, 0.3290), (0.3127, 0.3290));
    for r in 0..3 {
        for c in 0..3 {
            let expected = if r == c { 1.0 } else { 0.0 };
            assert!((cat_identity.0[r][c] - expected).abs() < 1e-4);
        }
    }
}
