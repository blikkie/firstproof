use std::path::Path;
use firstproof_core::proof::{discover_icc_profiles, generate_soft_proof_lut, ProofIntent};

#[test]
fn test_discover_icc_profiles_includes_standard_profiles() {
    let profiles = discover_icc_profiles();
    assert!(!profiles.is_empty(), "Profile discovery should return at least built-in profiles");

    let srgb = profiles.iter().find(|p| p.id == "builtin:srgb");
    assert!(srgb.is_some(), "sRGB built-in profile must be discovered");
    let p3 = profiles.iter().find(|p| p.id == "builtin:display_p3");
    assert!(p3.is_some(), "Display P3 built-in profile must be discovered");

    #[cfg(target_os = "macos")]
    {
        // On macOS, system profiles should also be indexed
        let system_cmyk = profiles.iter().find(|p| p.color_space == "CMYK" || p.name.contains("CMYK"));
        assert!(system_cmyk.is_some(), "System CMYK profile should be discovered on macOS");
    }
}

#[test]
fn test_generate_soft_proof_lut_srgb_target() {
    // Soft proofing against sRGB from Display P3 source
    let lut = generate_soft_proof_lut("builtin:srgb", ProofIntent::RelativeColorimetric, true, true, 17)
        .expect("Generating soft proof LUT for sRGB should succeed");

    assert_eq!(lut.size, 17);
    assert_eq!(lut.data_rgba32f.len(), 17 * 17 * 17 * 16);
    assert!(lut.title.contains("sRGB"));
}

#[test]
fn test_generate_soft_proof_lut_cmyk_target() {
    let cmyk_path = "/System/Library/ColorSync/Profiles/Generic CMYK Profile.icc";
    if !Path::new(cmyk_path).exists() {
        return;
    }

    let lut = generate_soft_proof_lut(cmyk_path, ProofIntent::RelativeColorimetric, true, true, 17)
        .expect("Generating soft proof LUT for CMYK should succeed");

    assert_eq!(lut.size, 17);
    assert_eq!(lut.data_rgba32f.len(), 17 * 17 * 17 * 16);

    // Verify out-of-gamut detection flags some points
    let points = 17 * 17 * 17;
    let mut out_of_gamut = 0;
    for i in 0..points {
        let alpha_bytes = &lut.data_rgba32f[i * 16 + 12..i * 16 + 16];
        let alpha = f32::from_le_bytes(alpha_bytes.try_into().unwrap());
        if alpha < 0.5 {
            out_of_gamut += 1;
        }
    }

    assert!(out_of_gamut > 0, "CMYK profile must flag out-of-gamut colors in wide-gamut source");
}

#[test]
fn test_generate_soft_proof_lut_paper_white_simulation() {
    let cmyk_path = "/System/Library/ColorSync/Profiles/Generic CMYK Profile.icc";
    if !Path::new(cmyk_path).exists() {
        return;
    }

    // Absolute colorimetric simulates paper white substrate
    let lut = generate_soft_proof_lut(cmyk_path, ProofIntent::AbsoluteColorimetric, false, false, 17)
        .expect("Generating absolute colorimetric proof LUT should succeed");

    assert_eq!(lut.size, 17);
    assert_eq!(lut.data_rgba32f.len(), 17 * 17 * 17 * 16);
}

#[test]
fn test_neutral_gray_not_alarmed_and_identity_maintained() {
    // Linear sRGB to Linear sRGB proofing must be an identity transform without false gamut alarms
    let lut = generate_soft_proof_lut("builtin:srgb", ProofIntent::RelativeColorimetric, false, false, 17)
        .expect("Generating soft proof LUT for sRGB should succeed");

    assert_eq!(lut.size, 17);

    // 50% neutral gray point: r = 8, g = 8, b = 8
    let mid_idx = (8 * 17 * 17) + (8 * 17) + 8;
    let r_bytes = &lut.data_rgba32f[mid_idx * 16..mid_idx * 16 + 4];
    let g_bytes = &lut.data_rgba32f[mid_idx * 16 + 4..mid_idx * 16 + 8];
    let b_bytes = &lut.data_rgba32f[mid_idx * 16 + 8..mid_idx * 16 + 12];
    let a_bytes = &lut.data_rgba32f[mid_idx * 16 + 12..mid_idx * 16 + 16];

    let r = f32::from_le_bytes(r_bytes.try_into().unwrap());
    let g = f32::from_le_bytes(g_bytes.try_into().unwrap());
    let b = f32::from_le_bytes(b_bytes.try_into().unwrap());
    let a = f32::from_le_bytes(a_bytes.try_into().unwrap());

    assert_eq!(a, 1.0, "50% neutral gray must be strictly in-gamut (alpha = 1.0)");
    assert!((r - 0.5).abs() < 0.02, "Red channel at 50% gray must remain ~0.5, got {}", r);
    assert!((g - 0.5).abs() < 0.02, "Green channel at 50% gray must remain ~0.5, got {}", g);
    assert!((b - 0.5).abs() < 0.02, "Blue channel at 50% gray must remain ~0.5, got {}", b);
}
