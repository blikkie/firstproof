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

/// Minimum log2 exposure value (-15.2 stops relative to 0.18 middle gray)
pub const AGX_MIN_EV: f32 = -15.2;

/// Maximum log2 exposure value (+5.0 stops relative to 0.18 middle gray)
pub const AGX_MAX_EV: f32 = 5.0;

/// Total dynamic range in EV stops (20.2 stops: 5.0 - (-15.2))
pub const AGX_EV_RANGE: f32 = 20.2;

/// Gamma linearization exponent for AgX display calibration
pub const AGX_GAMMA: f32 = 2.4;

/// Middle gray normalized point in log2 space: (log2(1.0) - AGX_MIN_EV) / AGX_EV_RANGE = 15.2 / 20.2
pub const AGX_MID_GRAY_NORM: f32 = 0.75247525;

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

/// Reference AgX rational sigmoid transfer function: x / (1.0 + x^power)^(1/power)
#[inline(always)]
fn agx_sigmoid(x: f32, power: f32) -> f32 {
    x / (1.0 + x.powf(power)).powf(1.0 / power)
}

/// Scaled, sloped, and shifted rational sigmoid
#[inline(always)]
fn agx_scaled_sigmoid(x: f32, scale: f32, slope: f32, power: f32, tx: f32, ty: f32) -> f32 {
    scale * agx_sigmoid(slope * (x - tx) / scale, power) + ty
}

/// AgX reference analytical contrast curve (Troy Sobotka reference calibration).
/// Smooth continuous rational sigmoid with deep unlifted blacks and filmic highlight rolloff.
#[inline(always)]
pub fn agx_apply_curve_channel(x: f32) -> f32 {
    if x < 0.6060606 {
        agx_scaled_sigmoid(x, -1.0359, 2.3843, 1.5, 0.6060606, 0.43446)
    } else {
        agx_scaled_sigmoid(x, 1.3475, 2.3843, 1.5, 0.6060606, 0.43446)
    }
    .clamp(0.0, 1.0)
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
/// Combines AgX's wide-dynamic-range highlight roll-off with a balanced 1.15x
/// midtone contrast curve, +18% chroma retention for landscapes/skies, and delicate skin-tone preservation.
pub fn agx_tone_map_vivid(c: Color3, contrast_gain: f32) -> Color3 {
    let vivid_contrast = contrast_gain * 1.15;
    let base = agx_tone_map(c, vivid_contrast);
    
    // Memory color tuning in HSV space
    let (mut h, mut s, v) = super::rgb_to_hsv(base.r(), base.g(), base.b());
    
    // Skin tone protection (15..45 deg, centered around 25 deg)
    let skin_weight = (1.0 - ((h - 25.0).abs() / 20.0)).clamp(0.0, 1.0);
    // Sky blue covers 200..270 deg, centered around 235 deg
    let sky_weight = (1.0 - ((h - 235.0).abs() / 35.0)).clamp(0.0, 1.0);
    
    // Azure sky shift
    h -= 18.0 * sky_weight;
    // Gentle peach alignment for skin (-3 deg away from yellow towards healthy rose)
    h -= 3.0 * skin_weight;
    if h < 0.0 {
        h += 360.0;
    }
    if h >= 360.0 {
        h -= 360.0;
    }
    
    // Vibrant midtone chroma with sky saturation normalized and delicate skin tones protected
    s = (s * (1.18 - 0.35 * skin_weight - 0.15 * sky_weight)).clamp(0.0, 1.0);
    
    let (r, g, b) = super::hsv_to_rgb(h, s, v);
    Color3::new(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
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
        // 0.18 linear mid-gray lands at normalized 15.2 / 20.2 = 0.75248, curve 0.75588,
        // linearized with gamma 2.4 to ~0.51085 linear sRGB (display sRGB ~188/255).
        let mid_gray = Color3::new(0.18, 0.18, 0.18);
        let out = agx_tone_map(mid_gray, 1.0);
        assert!((out.r() - 0.51085).abs() < 0.005, "Mid gray unexpected value: {}", out.r());
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
        assert!(h_sky <= 225.0 && h_sky >= 200.0, "AgX Vivid sky blue hue did not shift to azure: {}", h_sky);
    }
}
