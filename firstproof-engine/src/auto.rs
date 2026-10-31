//! Automatic Photographic Exposure, Contrast & White Balance Solvers
//!
//! Analyzes scene-referred linear Rec.2020 image luminance distributions to compute
//! conservative, aesthetically pleasing exposure compensation (EV) and contrast adjustments.
//!
//! Shared across native macOS desktop (firstproof-core) and web darkroom (firstproof-wasm).

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AutoAdjustments {
    pub exposure: f32, // -1.0 .. +1.75 EV
    pub contrast: f32, // -15.0 .. +20.0
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AutoWhiteBalanceResult {
    pub temperature: f32, // 2000.0 .. 20000.0 K
    pub tint: f32,        // -200.0 .. +200.0
}

pub use crate::raw::f16_to_f32;

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
    let p70 = p(70.0); // Upper midtones / key illuminated subject
    let p90 = p(90.0);
    let p95 = p(95.0); // Diffuse highlight anchor (Zone VII/VIII)
    let p98 = p(98.0); // Specular highlight shoulder

    // Dynamic range spread between 10th and 90th percentile
    let spread = (p90.max(0.01) / p10.max(0.001)).log2();

    // 1. Exposure Calculation (Zone System Evaluative Key Anchoring):
    // Compute log-geometric mean luminance L_g across all samples
    let log_sum: f32 = lums.iter().map(|&l| l.max(1e-5).ln()).sum();
    let log_mean_lum = (log_sum / (n as f32)).exp();

    // In high-contrast scenes (spread > 4.5 EV), dark backgrounds (foliage, dark rooms, night)
    // would artificially pull the geometric mean down. Weight the illuminated key midtone (P70)
    // to prevent blowing out illuminated subjects against dark surroundings.
    let key_midtone = if spread > 4.5 {
        0.35 * p50 + 0.65 * p70
    } else {
        0.50 * log_mean_lum + 0.50 * p50
    };

    let target_mid = 0.16f32;
    let midtone_ev = (target_mid / key_midtone.max(0.01)).log2();

    // Highlight ceiling safety check (Zone VII/VIII diffuse highlight anchor):
    // Diffuse white highlights (P95) should never be pushed past ~0.55 linear scene radiance,
    // and near-specular highlights (P98) should remain capped at 0.85.
    let diffuse_headroom_ev = (0.55 / p95.max(0.05)).log2();
    let specular_headroom_ev = (0.85 / p98.max(0.05)).log2();
    let max_headroom_ev = diffuse_headroom_ev.min(specular_headroom_ev);

    let target_ev = midtone_ev.min(max_headroom_ev);

    // Apply conservative photographic damping (40% factor) and clamp to [-0.85, +1.00]
    // Ensures Auto Tone acts as a subtle refinement without blowing out open-domain filmic tone mappers.
    let damped_ev = target_ev * 0.40;
    let clamped_ev = damped_ev.clamp(-0.85, 1.00);
    // Round to 0.05 EV steps (e.g. 0.15 EV)
    let final_exposure = (clamped_ev * 20.0).round() / 20.0;

    // 2. Contrast Calculation:
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

/// Computes auto tone adjustments (exposure, contrast) from a 16-bit half-float RGBA buffer in Linear Rec.2020 space.
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

        let y = 0.2627 * r + 0.6780 * g + 0.0593 * b;
        lums.push(y.max(0.0));
    }

    compute_auto_adjustments_from_luminances(lums)
}

/// Computes auto tone adjustments from a slice of u16 half-floats (RGBA layout: 4 u16s per pixel).
pub fn compute_auto_adjustments_from_u16_slice(
    width: u32,
    height: u32,
    data: &[u16],
) -> AutoAdjustments {
    let pixel_count = (width * height) as usize;
    if pixel_count == 0 || data.len() < pixel_count * 4 {
        return AutoAdjustments {
            exposure: 0.0,
            contrast: 0.0,
        };
    }

    let step = (pixel_count / 10_000).max(1);
    let mut lums = Vec::with_capacity(pixel_count / step + 1);

    for i in (0..pixel_count).step_by(step) {
        let offset = i * 4;
        if offset + 3 > data.len() {
            break;
        }
        let r = f16_to_f32(data[offset]);
        let g = f16_to_f32(data[offset + 1]);
        let b = f16_to_f32(data[offset + 2]);

        let y = 0.2627 * r + 0.6780 * g + 0.0593 * b;
        lums.push(y.max(0.0));
    }

    compute_auto_adjustments_from_luminances(lums)
}

/// Computes standalone Gray-World white balance from a 16-bit half-float RGBA slice in Linear Rec.2020 space.
pub fn compute_auto_white_balance_from_u16_slice(
    width: u32,
    height: u32,
    data: &[u16],
) -> AutoWhiteBalanceResult {
    let pixel_count = (width * height) as usize;
    if pixel_count == 0 || data.len() < pixel_count * 4 {
        return AutoWhiteBalanceResult {
            temperature: 5500.0,
            tint: 0.0,
        };
    }

    let step = (pixel_count / 10_000).max(1);
    let mut r_sum = 0.0f64;
    let mut g_sum = 0.0f64;
    let mut b_sum = 0.0f64;
    let mut count = 0usize;

    for i in (0..pixel_count).step_by(step) {
        let offset = i * 4;
        if offset + 3 > data.len() {
            break;
        }
        let r = f16_to_f32(data[offset]);
        let g = f16_to_f32(data[offset + 1]);
        let b = f16_to_f32(data[offset + 2]);

        let y = 0.2627 * r + 0.6780 * g + 0.0593 * b;
        // Filter out extreme highlights (>0.85) and deep shadow noise (<0.02)
        if (0.02..=0.85).contains(&y) && r > 0.0 && g > 0.0 && b > 0.0 {
            r_sum += r as f64;
            g_sum += g as f64;
            b_sum += b as f64;
            count += 1;
        }
    }

    if count > 50 {
        let mean_r = (r_sum / count as f64) as f32;
        let mean_g = (g_sum / count as f64) as f32;
        let mean_b = (b_sum / count as f64) as f32;
        calculate_white_balance_from_sample(mean_r, mean_g, mean_b)
    } else {
        AutoWhiteBalanceResult {
            temperature: 5500.0,
            tint: 0.0,
        }
    }
}

/// Computes standalone Gray-World white balance from a 16-bit half-float RGBA byte buffer in Linear Rec.2020 space.
pub fn compute_auto_white_balance_from_f16_buffer(
    width: u32,
    height: u32,
    data: &[u8],
) -> AutoWhiteBalanceResult {
    let pixel_count = (width * height) as usize;
    if pixel_count == 0 || data.len() < pixel_count * 8 {
        return AutoWhiteBalanceResult {
            temperature: 5500.0,
            tint: 0.0,
        };
    }

    let step = (pixel_count / 10_000).max(1);
    let mut r_sum = 0.0f64;
    let mut g_sum = 0.0f64;
    let mut b_sum = 0.0f64;
    let mut wb_count = 0usize;

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

        let y = 0.2627 * r + 0.6780 * g + 0.0593 * b;

        if (0.02..=0.85).contains(&y) && r > 0.0 && g > 0.0 && b > 0.0 {
            r_sum += r as f64;
            g_sum += g as f64;
            b_sum += b as f64;
            wb_count += 1;
        }
    }

    if wb_count > 0 {
        calculate_white_balance_from_sample(
            (r_sum / wb_count as f64) as f32,
            (g_sum / wb_count as f64) as f32,
            (b_sum / wb_count as f64) as f32,
        )
    } else {
        AutoWhiteBalanceResult {
            temperature: 5500.0,
            tint: 0.0,
        }
    }
}

/// Computes White Balance (Kelvin & Tint) from an averaged RGB reflectance sample in Linear Rec.2020.
pub fn calculate_white_balance_from_sample(r: f32, g: f32, b: f32) -> AutoWhiteBalanceResult {
    if r <= 1e-5 || g <= 1e-5 || b <= 1e-5 {
        return AutoWhiteBalanceResult {
            temperature: 5500.0,
            tint: 0.0,
        };
    }

    let temp_ratio = (b / r).max(1e-4);
    let temp_norm = temp_ratio.powf(2.0 / 3.0);
    let target_temp = (5500.0 * temp_norm).clamp(2000.0, 20000.0);

    let r_scaled = r * temp_norm.powf(0.75);
    let b_scaled = b * (1.0 / temp_norm).powf(0.75);
    let rb_avg = (r_scaled + b_scaled) * 0.5;

    let tint_factor = 1.0 - (rb_avg / g.max(1e-4));
    let target_tint = (tint_factor / 0.002).clamp(-200.0, 200.0);

    AutoWhiteBalanceResult {
        temperature: (target_temp / 50.0).round() * 50.0,
        tint: target_tint.round(),
    }
}
