//! Automatic Photographic Exposure & Contrast Solver
//!
//! Analyzes scene-referred linear Rec.2020 image luminance distributions to compute
//! conservative, aesthetically pleasing exposure compensation (EV) and contrast adjustments.

use crate::{calculate_white_balance_from_sample, WhiteBalanceResult};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(uniffi::Record)]
pub struct AutoAdjustments {
    pub exposure: f32, // -1.0 .. +1.75 EV
    pub contrast: f32, // -15.0 .. +20.0
}

impl From<firstproof_engine::auto::AutoAdjustments> for AutoAdjustments {
    fn from(a: firstproof_engine::auto::AutoAdjustments) -> Self {
        Self {
            exposure: a.exposure,
            contrast: a.contrast,
        }
    }
}

/// Computes conservative auto adjustments from an array of linear Rec.2020 luminance samples.
pub fn compute_auto_adjustments_from_luminances(lums: Vec<f32>) -> AutoAdjustments {
    firstproof_engine::auto::compute_auto_adjustments_from_luminances(lums).into()
}

/// Computes auto tone adjustments (exposure, contrast) from a 16-bit half-float RGBA buffer in Linear Rec.2020 space.
pub fn compute_auto_adjustments_from_f16_buffer(
    width: u32,
    height: u32,
    data: &[u8],
) -> AutoAdjustments {
    firstproof_engine::auto::compute_auto_adjustments_from_f16_buffer(width, height, data).into()
}

/// Computes standalone Gray-World white balance from a 16-bit half-float RGBA buffer in Linear Rec.2020 space.
pub fn compute_auto_white_balance_from_f16_buffer(
    width: u32,
    height: u32,
    data: &[u8],
) -> WhiteBalanceResult {
    let res = firstproof_engine::auto::compute_auto_white_balance_from_f16_buffer(width, height, data);
    WhiteBalanceResult {
        temperature: res.temperature,
        tint: res.tint,
    }
}

/// Computes auto adjustments directly from an image file path (RAW or standard image).
pub fn compute_auto_adjustments_from_path<P: AsRef<Path>>(path: P) -> Option<AutoAdjustments> {
    let demosaiced = crate::export::decode_image(path).ok()?;
    let pixel_count = demosaiced.width * demosaiced.height;
    if pixel_count == 0 || demosaiced.data.len() < pixel_count * 3 {
        return None;
    }

    let step = (pixel_count / 10_000).max(1);
    let mut lums = Vec::with_capacity(pixel_count / step + 1);

    for i in (0..pixel_count).step_by(step) {
        let offset = i * 3;
        if offset + 3 > demosaiced.data.len() {
            break;
        }
        let r = demosaiced.data[offset];
        let g = demosaiced.data[offset + 1];
        let b = demosaiced.data[offset + 2];

        let y = 0.2627 * r + 0.6780 * g + 0.0593 * b;
        lums.push(y.max(0.0));
    }

    Some(compute_auto_adjustments_from_luminances(lums))
}

/// Computes standalone Gray-World white balance directly from an image file path.
pub fn compute_auto_white_balance_from_path<P: AsRef<Path>>(path: P) -> Option<WhiteBalanceResult> {
    let demosaiced = crate::export::decode_image(path).ok()?;
    let pixel_count = demosaiced.width * demosaiced.height;
    if pixel_count == 0 || demosaiced.data.len() < pixel_count * 3 {
        return None;
    }

    let step = (pixel_count / 10_000).max(1);
    let mut r_sum = 0.0f64;
    let mut g_sum = 0.0f64;
    let mut b_sum = 0.0f64;
    let mut wb_count = 0usize;

    for i in (0..pixel_count).step_by(step) {
        let offset = i * 3;
        if offset + 3 > demosaiced.data.len() {
            break;
        }
        let r = demosaiced.data[offset];
        let g = demosaiced.data[offset + 1];
        let b = demosaiced.data[offset + 2];

        let y = 0.2627 * r + 0.6780 * g + 0.0593 * b;

        if (0.02..=0.85).contains(&y) && r > 0.0 && g > 0.0 && b > 0.0 {
            r_sum += r as f64;
            g_sum += g as f64;
            b_sum += b as f64;
            wb_count += 1;
        }
    }

    if wb_count > 0 {
        Some(calculate_white_balance_from_sample(
            (r_sum / wb_count as f64) as f32,
            (g_sum / wb_count as f64) as f32,
            (b_sum / wb_count as f64) as f32,
        ))
    } else {
        Some(WhiteBalanceResult {
            temperature: 5500.0,
            tint: 0.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_dark_image() {
        // Simulated underexposed image: median = 0.05, p98 = 0.30
        let mut lums = Vec::new();
        for i in 0..1000 {
            let val = (i as f32 / 1000.0) * 0.10;
            lums.push(val);
        }
        let adj = compute_auto_adjustments_from_luminances(lums);
        assert!(adj.exposure > 0.0, "Dark image should receive positive exposure boost, got {}", adj.exposure);
        assert!(adj.exposure <= 1.75, "Exposure should remain conservatively capped");
    }

    #[test]
    fn test_auto_highlight_limiting() {
        // Image with dark midtones but specular highlights near 0.95
        let mut lums = vec![0.03; 950];
        lums.extend(vec![0.95; 50]);
        let adj = compute_auto_adjustments_from_luminances(lums);
        // Headroom for 0.95 is log2(1.0 / 0.95) ~ 0.07 EV
        assert!(adj.exposure <= 0.15, "Highlights must limit exposure gain, got {}", adj.exposure);
    }

    #[test]
    fn test_auto_normal_image() {
        // Well-exposed image centered around 0.16
        let mut lums = Vec::new();
        for i in 0..1000 {
            let val = 0.02 + (i as f32 / 1000.0) * 0.30;
            lums.push(val);
        }
        let adj = compute_auto_adjustments_from_luminances(lums);
        assert!(adj.exposure.abs() <= 0.30, "Normal image should receive minimal change, got {}", adj.exposure);
    }

    #[test]
    fn test_auto_flat_contrast() {
        // Flat image with narrow dynamic range (all pixels between 0.10 and 0.20)
        let lums = vec![0.15; 1000];
        let adj = compute_auto_adjustments_from_luminances(lums);
        assert!(adj.contrast >= 0.0, "Low contrast image should receive positive contrast, got {}", adj.contrast);
    }

    #[test]
    fn test_auto_white_balance_from_f16_buffer() {
        use crate::export::f32_to_f16;

        // Create a 10x10 test image with a warm cast (R=0.6, G=0.4, B=0.2)
        let mut warm_data = Vec::new();
        let r16 = f32_to_f16(0.60);
        let g16 = f32_to_f16(0.40);
        let b16 = f32_to_f16(0.20);
        let a16 = f32_to_f16(1.00);

        for _ in 0..100 {
            warm_data.extend_from_slice(&r16.to_le_bytes());
            warm_data.extend_from_slice(&g16.to_le_bytes());
            warm_data.extend_from_slice(&b16.to_le_bytes());
            warm_data.extend_from_slice(&a16.to_le_bytes());
        }

        let wb = compute_auto_white_balance_from_f16_buffer(10, 10, &warm_data);
        assert!(wb.temperature < 5500.0, "Warm image should cool down temperature, got {}", wb.temperature);

        let adj = compute_auto_adjustments_from_f16_buffer(10, 10, &warm_data);
        assert!(adj.exposure.is_finite());

        // Test with green cast (R=0.3, G=0.7, B=0.3)
        let mut green_data = Vec::new();
        let r16_g = f32_to_f16(0.30);
        let g16_g = f32_to_f16(0.70);
        let b16_g = f32_to_f16(0.30);
        for _ in 0..100 {
            green_data.extend_from_slice(&r16_g.to_le_bytes());
            green_data.extend_from_slice(&g16_g.to_le_bytes());
            green_data.extend_from_slice(&b16_g.to_le_bytes());
            green_data.extend_from_slice(&a16.to_le_bytes());
        }
        let wb_green = compute_auto_white_balance_from_f16_buffer(10, 10, &green_data);
        assert!(wb_green.tint > 0.0, "Green cast should require positive magenta tint compensation, got {}", wb_green.tint);
    }
}

