//! Macbeth ColorChecker 24-Patch Benchmark & Regression Guard
//!
//! Validates the full wide-gamut scene-referred color pipeline using standardized
//! spectrophotometric reference patches (GretagMacbeth / Calibrite ColorChecker).

use lumenbox_core::color::*;
use lumenbox_core::color::ciede2000::*;

struct MacbethPatch {
    pub name: &'static str,
    pub srgb_8bit: [u8; 3],
}

const MACBETH_24: [MacbethPatch; 24] = [
    MacbethPatch { name: "Dark Skin",      srgb_8bit: [115, 82, 68] },
    MacbethPatch { name: "Light Skin",     srgb_8bit: [194, 150, 130] },
    MacbethPatch { name: "Blue Sky",       srgb_8bit: [98, 122, 157] },
    MacbethPatch { name: "Foliage",        srgb_8bit: [87, 108, 67] },
    MacbethPatch { name: "Blue Flower",    srgb_8bit: [133, 128, 177] },
    MacbethPatch { name: "Bluish Green",   srgb_8bit: [103, 189, 170] },
    MacbethPatch { name: "Orange",         srgb_8bit: [214, 126, 44] },
    MacbethPatch { name: "Purplish Blue",  srgb_8bit: [80, 91, 166] },
    MacbethPatch { name: "Moderate Red",   srgb_8bit: [193, 90, 99] },
    MacbethPatch { name: "Purple",         srgb_8bit: [94, 60, 108] },
    MacbethPatch { name: "Yellow Green",   srgb_8bit: [157, 188, 64] },
    MacbethPatch { name: "Orange Yellow",  srgb_8bit: [224, 163, 46] },
    MacbethPatch { name: "Blue",           srgb_8bit: [56, 61, 150] },
    MacbethPatch { name: "Green",          srgb_8bit: [70, 148, 73] },
    MacbethPatch { name: "Red",            srgb_8bit: [175, 54, 60] },
    MacbethPatch { name: "Yellow",         srgb_8bit: [231, 199, 31] },
    MacbethPatch { name: "Magenta",        srgb_8bit: [187, 86, 149] },
    MacbethPatch { name: "Cyan",           srgb_8bit: [8, 133, 161] },
    MacbethPatch { name: "White 9.5",      srgb_8bit: [243, 243, 242] },
    MacbethPatch { name: "Neutral 8",      srgb_8bit: [200, 200, 200] },
    MacbethPatch { name: "Neutral 6.5",    srgb_8bit: [160, 160, 160] },
    MacbethPatch { name: "Neutral 5",      srgb_8bit: [122, 122, 121] },
    MacbethPatch { name: "Neutral 3.5",    srgb_8bit: [85, 85, 85] },
    MacbethPatch { name: "Black 2",        srgb_8bit: [52, 52, 52] },
];

#[test]
fn test_macbeth_rec2020_pipeline_roundtrip() {
    let mut total_de00 = 0.0f32;
    let mut max_de00 = 0.0f32;

    for patch in &MACBETH_24 {
        // 1. Convert reference 8-bit sRGB to linear sRGB
        let r_lin = srgb_gamma_to_linear((patch.srgb_8bit[0] as f32) / 255.0);
        let g_lin = srgb_gamma_to_linear((patch.srgb_8bit[1] as f32) / 255.0);
        let b_lin = srgb_gamma_to_linear((patch.srgb_8bit[2] as f32) / 255.0);
        let linear_srgb_orig = Color3::new(r_lin, g_lin, b_lin);

        let lab_ref = linear_srgb_to_lab(linear_srgb_orig);

        // 2. Transform into Scene-Referred Linear Rec.2020 working space
        let rec2020 = LINEAR_SRGB_TO_REC2020.multiply_vector(linear_srgb_orig);

        // 3. Transform back from Rec.2020 -> Linear sRGB
        let linear_srgb_recovered = REC2020_TO_LINEAR_SRGB.multiply_vector(rec2020);
        let lab_recovered = linear_srgb_to_lab(linear_srgb_recovered);

        // 4. Compute CIEDE2000 perceptual delta
        let de00 = ciede2000(lab_ref, lab_recovered);
        total_de00 += de00;
        if de00 > max_de00 {
            max_de00 = de00;
        }

        assert!(
            de00 < 0.05,
            "Macbeth patch '{}' failed roundtrip: dE00={de00:.4}", patch.name
        );
    }

    let avg_de00 = total_de00 / (MACBETH_24.len() as f32);
    println!("Macbeth 24-patch Rec.2020 pipeline: avg dE00={avg_de00:.5}, max dE00={max_de00:.5}");
    assert!(avg_de00 < 0.01);
}

#[test]
fn test_macbeth_neutral_gray_axis_purity() {
    // 1. Pure synthetic neutral grays must preserve R == G == B perfectly in Rec.2020
    for i in 1..=20 {
        let val = (i as f32) / 20.0;
        let srgb_lin = Color3::new(val, val, val);
        let rec2020 = LINEAR_SRGB_TO_REC2020.multiply_vector(srgb_lin);

        let diff_rg = (rec2020.r() - rec2020.g()).abs();
        let diff_gb = (rec2020.g() - rec2020.b()).abs();

        assert!(
            diff_rg < 1e-5 && diff_gb < 1e-5,
            "Synthetic gray {val} developed chromatic tint in Rec.2020: R={}, G={}, B={}",
            rec2020.r(), rec2020.g(), rec2020.b()
        );
    }

    // 2. Physical Macbeth neutral patches (#19 to #24) must stay strictly achromatic (C* < 1.5)
    for patch in &MACBETH_24[18..24] {
        let r_lin = srgb_gamma_to_linear((patch.srgb_8bit[0] as f32) / 255.0);
        let g_lin = srgb_gamma_to_linear((patch.srgb_8bit[1] as f32) / 255.0);
        let b_lin = srgb_gamma_to_linear((patch.srgb_8bit[2] as f32) / 255.0);
        let srgb_lin = Color3::new(r_lin, g_lin, b_lin);

        let lab = linear_srgb_to_lab(srgb_lin);
        assert!(
            lab.chroma() < 1.5,
            "Physical Macbeth patch '{}' exceeded neutral chroma limit: C*={}",
            patch.name, lab.chroma()
        );
    }
}
