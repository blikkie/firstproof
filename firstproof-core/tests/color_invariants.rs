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
    // Macbeth Light Skin baseline (linear sRGB coordinates)
    let skin_base = Color3::new(0.450, 0.312, 0.241);
    let base_lab = linear_srgb_to_lab(skin_base);
    let base_hue = base_lab.hue_angle_deg();

    // Sweep exposure from -3 EV (1/8x) to +1.5 EV (2.8x)
    for ev in &[-3.0, -2.0, -1.0, 0.0, 0.5, 1.0, 1.5] {
        let factor = 2.0f32.powf(*ev);
        let scaled_skin = skin_base.scale(factor);
        
        let lab = linear_srgb_to_lab(scaled_skin);
        let hue = lab.hue_angle_deg();
        
        let hue_diff = (hue - base_hue).abs();
        assert!(
            hue_diff < 2.0,
            "Skin tone hue drifted by {hue_diff:.2} deg at EV={ev} (base={base_hue:.2}, curr={hue:.2})"
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
