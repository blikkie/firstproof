//! Automatic Photographic Exposure & Contrast Solver
//!
//! Analyzes scene-referred linear Rec.2020 image luminance distributions to compute
//! conservative, aesthetically pleasing exposure compensation (EV) and contrast adjustments.

use crate::export::f16_to_f32;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(uniffi::Record)]
pub struct AutoAdjustments {
    pub exposure: f32, // -1.0 .. +1.75 EV
    pub contrast: f32, // -15.0 .. +20.0
}

/// Computes conservative auto adjustments from an array of linear Rec.2020 luminance samples.
pub fn compute_auto_adjustments_from_luminances(mut lums: Vec<f32>) -> AutoAdjustments {
    if lums.is_empty() {
        return AutoAdjustments {
            exposure: 0.0,
            contrast: 0.0,
        };
    }

    // Sort to compute statistical percentiles
    lums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = lums.len();

    let p = |pct: f32| -> f32 {
        let idx = ((pct / 100.0) * (n - 1) as f32).round() as usize;
        lums[idx.min(n - 1)]
    };

    let p10 = p(10.0);
    let p50 = p(50.0); // Median scene midtone
    let p90 = p(90.0);
    let p98 = p(98.0); // Highlight ceiling / shoulder

    // 1. Exposure Calculation (Zone System Log-Luminance Key-Value Anchoring):
    // Photographic middle gray targets ~0.16 linear scene reflectance.
    let target_mid = 0.16f32;

    // Compute log-geometric mean luminance L_g across all samples:
    let log_sum: f32 = lums.iter().map(|&l| l.max(1e-5).ln()).sum();
    let log_mean_lum = (log_sum / (n as f32)).exp();

    // Zone V (18% photographic middle gray) target EV compensation:
    let zone_v_ev = (target_mid / log_mean_lum.max(0.005)).log2();

    // Percentile midtone EV requirement:
    let midtone_ev = (target_mid / p50.max(0.01)).log2();

    // Combine Zone V log-geometric mean with 50th percentile median (50/50 weighted key anchor)
    let combined_mid_ev: f32 = 0.5 * zone_v_ev + 0.5 * midtone_ev;

    // Highlight ceiling safety check:
    // Ensure the 98th percentile does not get pushed past 1.0 (specular threshold)
    let max_headroom_ev = (1.0 / p98.max(0.05)).log2().max(0.0);

    // Take the minimum of midtone requirement and highlight headroom
    let target_ev = combined_mid_ev.min(max_headroom_ev);

    // Apply conservative damping (65% factor) and clamp to [-1.0, +1.75]
    let damped_ev = target_ev * 0.65;
    let clamped_ev = damped_ev.clamp(-1.0, 1.75);
    // Round to 0.05 EV steps (e.g. 0.35 EV)
    let final_exposure = (clamped_ev * 20.0).round() / 20.0;

    // 2. Contrast Calculation:
    // Evaluate dynamic range spread between 10th and 90th percentile
    let spread = (p90.max(0.01) / p10.max(0.001)).log2();

    // Standard photographic scene has around 5.0 stops between P10 and P90.
    // Flat scenes (< 4.5 stops) benefit from gentle contrast (+5 .. +15).
    // Harsh scenes (> 6.5 stops) benefit from slight contrast easing (-5 .. -10).
    let raw_contrast = if spread < 4.5 {
        (4.5 - spread) * 5.0
    } else if spread > 6.5 {
        (6.5 - spread) * 4.0
    } else {
        0.0
    };

    let clamped_contrast = raw_contrast.clamp(-15.0, 20.0).round();

    AutoAdjustments {
        exposure: final_exposure,
        contrast: clamped_contrast,
    }
}

/// Computes auto adjustments from a 16-bit half-float RGBA buffer in Linear Rec.2020 space.
pub fn compute_auto_adjustments_from_f16_buffer(
    width: u32,
    height: u32,
    data: &[u8],
) -> AutoAdjustments {
    let pixel_count = (width * height) as usize;
    if pixel_count == 0 || data.len() < pixel_count * 8 {
        return AutoAdjustments {
            exposure: 0.0,
            contrast: 0.0,
        };
    }

    // Stride to sample ~10,000 pixels across the image for sub-millisecond execution
    let step = (pixel_count / 10_000).max(1);
    let mut lums = Vec::with_capacity(pixel_count / step + 1);

    for i in (0..pixel_count).step_by(step) {
        let offset = i * 8;
        if offset + 6 > data.len() {
            break;
        }
        let r_u16 = u16::from_le_bytes([data[offset], data[offset + 1]]);
        let g_u16 = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
        let b_u16 = u16::from_le_bytes([data[offset + 4], data[offset + 5]]);

        let r = f16_to_f32(r_u16);
        let g = f16_to_f32(g_u16);
        let b = f16_to_f32(b_u16);

        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        lums.push(y.max(0.0));
    }

    compute_auto_adjustments_from_luminances(lums)
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

        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        lums.push(y.max(0.0));
    }

    Some(compute_auto_adjustments_from_luminances(lums))
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
}
