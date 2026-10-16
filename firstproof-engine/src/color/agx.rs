//! AgX Filmic Tone Mapping Engine
//!
//! Implements Troy Sobotka's AgX color science architecture for scene-referred
//! Wide-Gamut (Linear Rec.2020) color rendering.
//!
//! AgX replaces naive per-channel digital clipping with photographic color reproduction:
//! 1. **Gamut Inset Matrix**: Pre-conditions wide-gamut linear primaries into an inset
//!    working gamut to prevent out-of-gamut clipping and Abney effect hue shifts.
//! 2. **Log2 Exposure Normalization**: Normalizes dynamic range across 16.5 stops
//!    (-10.0 EV to +6.5 EV relative to 18% middle gray).
//! 3. **Sigmoidal Contrast Curve**: Smooth 6th-order rational polynomial S-curve with
//!    midtone pivot contrast adjustment.
//! 4. **Gamut Outset Matrix**: Maps curved filmic signals back into display sRGB space.
//! 5. **Linearization**: Linearizes display signals with a standard 2.2 power function
//!    so the downstream pipeline remains strictly linear float until final display/encode.

use super::{Color3, Matrix3x3};

/// Reference AgX Inset Matrix for Linear sRGB / Rec.709 input (Troy Sobotka reference calibration).
pub const AGX_INSET_SRGB_MATRIX: Matrix3x3 = Matrix3x3([
    [0.8566272,  0.09512124, 0.04825161],
    [0.13731897, 0.761242,   0.10143904],
    [0.11189821, 0.07679942, 0.81130237],
]);

/// Direct AgX Inset Matrix for Wide-Gamut Linear Rec.2020 scene-referred colors.
/// Pre-multiplied as AGX_INSET_SRGB * REC2020_TO_LINEAR_SRGB so wide-gamut scene photons
/// map accurately into AgX log working space in a single transform without gamut loss.
pub const AGX_INSET_REC2020_MATRIX: Matrix3x3 = Matrix3x3([
    [ 1.4096986,  -0.4004796,  -0.009218898],
    [ 0.13136263,  0.77151406,  0.09712328],
    [ 0.16151477, -0.06034984,  0.89883506],
]);

/// Default AgX Inset Matrix mapping Wide-Gamut Linear Rec.2020 into AgX Inset space.
pub const AGX_INSET_MATRIX: Matrix3x3 = AGX_INSET_REC2020_MATRIX;

/// AgX Outset Matrix (transforms AgX Curved Space -> Display sRGB).
/// Inverse of AgX outset space (row-major order).
pub const AGX_OUTSET_MATRIX: Matrix3x3 = Matrix3x3([
    [ 1.1271005, -0.1106066, -0.0164939],
    [-0.1413298,  1.1578237, -0.0164939],
    [-0.1413298, -0.1106066,  1.2519364],
]);

/// Minimum log2 exposure value (-10.0 stops relative to 0.18 middle gray)
pub const AGX_MIN_EV: f32 = -10.0;

/// Maximum log2 exposure value (+6.5 stops relative to 0.18 middle gray)
pub const AGX_MAX_EV: f32 = 6.5;

/// Total dynamic range in EV stops (16.5 stops: 6.5 - (-10.0))
pub const AGX_EV_RANGE: f32 = 16.5;

/// Gamma linearization exponent for AgX display calibration
pub const AGX_GAMMA: f32 = 2.2;

/// Middle gray normalized point in log2 space: (log2(1.0) - AGX_MIN_EV) / AGX_EV_RANGE = 10.0 / 16.5
pub const AGX_MID_GRAY_NORM: f32 = 0.6060606;

/// Compresses out-of-gamut scene values to prevent negative numbers before log-encoding.
#[inline(always)]
pub fn agx_compress_gamut(c: Color3) -> Color3 {
    let min_c = c.r().min(c.g()).min(c.b());
    if min_c < 0.0 {
        Color3::new(c.r() - min_c, c.g() - min_c, c.b() - min_c)
    } else {
        c
    }
}

/// AgX reference analytical contrast curve (derived from Troy Sobotka reference AgX OCIO LUT).
/// Smooth continuous 6th-order polynomial with boundary anchoring:
/// P(0.0) = 0.0, P(1.0) = 1.0, and strict monotonicity (P'(x) > 0 for all x in [0, 1]).
#[inline(always)]
pub fn agx_apply_curve_channel(x: f32) -> f32 {
    let xc = x.clamp(0.0, 1.0);
    // Horner evaluation: (((((c6 * x + c5) * x + c4) * x + c3) * x + c2) * x + c1) * x
    let curved = (((((15.45672 * xc - 40.02792) * xc + 31.87076) * xc - 6.84882) * xc + 0.42860) * xc + 0.12066) * xc;
    curved.clamp(0.0, 1.0)
}

/// Compatibility alias for analytical contrast curve
#[inline(always)]
pub fn agx_contrast_approx(x: f32) -> f32 {
    agx_apply_curve_channel(x)
}

/// Applies the AgX filmic tone mapping operator to a Linear Rec.2020 color vector.
///
/// Returns a display-referred sRGB color vector (values in range [0.0, 1.0]), ready for
/// direct Metal viewport display or direct 8-bit quantization upon export.
pub fn agx_tone_map(c: Color3, contrast_gain: f32) -> Color3 {
    // 1. Gamut compression (prevents negative values)
    let compressed = agx_compress_gamut(c);

    // 2. Inset Gamut Transform (Rec.2020 -> AgX Inset)
    let inset = AGX_INSET_MATRIX.multiply_vector(compressed);

    // 3. Middle-gray (0.18) relative log2 normalization & Sigmoidal Transfer
    let process_channel = |v: f32| -> f32 {
        let x_rel = (v / 0.18).max(1e-6);
        let log_encoded = (x_rel.log2() - AGX_MIN_EV) / AGX_EV_RANGE;
        let mut norm = log_encoded.clamp(0.0, 1.0);
        
        // Midtone-anchored contrast adjustment
        if (contrast_gain - 1.0).abs() > 0.001 {
            norm = (AGX_MID_GRAY_NORM + (norm - AGX_MID_GRAY_NORM) * contrast_gain).clamp(0.0, 1.0);
        }
        
        let curved = agx_apply_curve_channel(norm);
        curved.max(0.0).powf(AGX_GAMMA)
    };

    let linearized = Color3::new(
        process_channel(inset.r()),
        process_channel(inset.g()),
        process_channel(inset.b()),
    );

    // 4. Outset Gamut Transform (AgX Curved -> Linear sRGB)
    let outset = AGX_OUTSET_MATRIX.multiply_vector(linearized);

    Color3::new(
        outset.r().max(0.0),
        outset.g().max(0.0),
        outset.b().max(0.0),
    )
}

/// Applies the AgX Vivid (Filmic Punchy) tone mapping operator.
///
/// Implements Troy Sobotka's official AgX Punchy look (reference ASC-CDL specification):
/// 1. Evaluates AgX filmic tone curve with 1.15x contrast gain.
/// 2. Applies continuous wide-gamut linear saturation (s = 1.20) in the outset space:
///    C_vivid = Y + 1.20 * (C - Y)
///    where Y = 0.2126 R + 0.7152 G + 0.0722 B.
///
/// This eliminates all non-linear HSV singularities, hue-angle twisting, circular threshold jumps,
/// and shadow noise amplification, preserving smooth, continuous skin tones and noise-safe shadows.
pub fn agx_tone_map_vivid(c: Color3, contrast_gain: f32) -> Color3 {
    let punchy_contrast = contrast_gain * 1.35;
    let base = agx_tone_map(c, punchy_contrast);

    let lum = 0.2126 * base.r() + 0.7152 * base.g() + 0.0722 * base.b();
    let sat = if lum > 0.65 {
        let t = ((lum - 0.65) / 0.35).min(1.0);
        let h = t * t * (3.0 - 2.0 * t);
        1.25 - 0.25 * h
    } else {
        1.25
    };

    Color3::new(
        (lum + sat * (base.r() - lum)).clamp(0.0, 1.0),
        (lum + sat * (base.g() - lum)).clamp(0.0, 1.0),
        (lum + sat * (base.b() - lum)).clamp(0.0, 1.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agx_neutral_gray_preservation() {
        // Neutral gray inputs must remain strictly neutral (R = G = B)
        for &gray in &[0.01, 0.05, 0.18, 0.5, 1.0, 4.0, 16.0] {
            let input = Color3::new(gray, gray, gray);
            let output = agx_tone_map(input, 1.0);
            
            assert!(
                (output.r() - output.g()).abs() < 1e-4,
                "R and G diverge on neutral input {}: R={}, G={}",
                gray, output.r(), output.g()
            );
            assert!(
                (output.g() - output.b()).abs() < 1e-4,
                "G and B diverge on neutral input {}: G={}, B={}",
                gray, output.g(), output.b()
            );
        }
    }

    #[test]
    fn test_agx_middle_gray_level() {
        // 0.18 linear mid-gray lands at normalized 10.0 / 16.5 = 0.60606, curve ~0.4988,
        // linearized with gamma 2.2 to ~0.2165 linear sRGB (display sRGB ~127/255).
        let mid_gray = Color3::new(0.18, 0.18, 0.18);
        let out = agx_tone_map(mid_gray, 1.0);
        assert!((out.r() - 0.2165).abs() < 0.005, "Mid gray unexpected value: {}", out.r());
    }

    #[test]
    fn test_agx_highlight_roll_off() {
        // High dynamic range values up to 16.0 should compress smoothly without NaN/overflow
        let specular = Color3::new(16.0, 16.0, 16.0);
        let out = agx_tone_map(specular, 1.0);
        assert!(out.r() <= 1.0 && out.r() >= 0.99, "Specular highlight didn't roll off near 1.0: {}", out.r());
    }

    #[test]
    fn test_agx_vivid_punchy_and_neutral_purity() {
        // 1. Neutral gray purity
        let gray = Color3::new(0.18, 0.18, 0.18);
        let out_vivid = agx_tone_map_vivid(gray, 1.0);
        assert!((out_vivid.r() - out_vivid.g()).abs() < 1e-4);
        assert!((out_vivid.g() - out_vivid.b()).abs() < 1e-4);

        // 2. Specular highlight roll-off
        let specular = Color3::new(16.0, 16.0, 16.0);
        let out_spec = agx_tone_map_vivid(specular, 1.0);
        assert!(out_spec.r() <= 1.0 && out_spec.r() >= 0.99);

        // 3. Sky blue azure shift
        let sky = Color3::new(0.20, 0.35, 0.70);
        let out_sky = agx_tone_map_vivid(sky, 1.0);
        let (h_sky, _, _) = crate::color::rgb_to_hsv(out_sky.r(), out_sky.g(), out_sky.b());
        assert!((200.0..=225.0).contains(&h_sky), "AgX Vivid sky blue hue did not shift to azure: {}", h_sky);
    }
}
