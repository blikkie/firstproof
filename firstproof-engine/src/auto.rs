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

/// Convert IEEE 754 half-precision float (u16) to single-precision float (f32).
#[inline]
pub fn f16_to_f32(h: u16) -> f32 {
    let sign = ((h >> 15) & 1) as u32;
    let exp = ((h >> 10) & 0x1f) as u32;
    let mant = (h & 0x3ff) as u32;

    if exp == 0 {
        if mant == 0 {
            f32::from_bits(sign << 31)
        } else {
            let mut m = mant << 1;
            let mut e = 0;
            while (m & 0x400) == 0 {
                m <<= 1;
                e += 1;
            }
            let exp_f32 = 127 - 15 - e;
            let mant_f32 = (m & 0x3ff) << 13;
            f32::from_bits((sign << 31) | (exp_f32 << 23) | mant_f32)
        }
    } else if exp == 31 {
        if mant == 0 {
            f32::from_bits((sign << 31) | (0xff << 23))
        } else {
            f32::from_bits((sign << 31) | (0xff << 23) | (mant << 13))
        }
    } else {
        let exp_f32 = exp + (127 - 15);
        let mant_f32 = mant << 13;
        f32::from_bits((sign << 31) | (exp_f32 << 23) | mant_f32)
    }
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

        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
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

        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
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

        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        // Filter out extreme highlights (>0.85) and deep shadow noise (<0.02)
        if y >= 0.02 && y <= 0.85 && r > 0.0 && g > 0.0 && b > 0.0 {
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

/// Computes White Balance (Kelvin & Tint) from an averaged RGB reflectance sample in Linear Rec.2020.
pub fn calculate_white_balance_from_sample(r: f32, g: f32, b: f32) -> AutoWhiteBalanceResult {
    let r = r.max(1e-4);
    let g = g.max(1e-4);
    let b = b.max(1e-4);

    // Target is neutral D65 white point in Rec.2020 (R = G = B)
    // Multipliers needed to bring the sample to neutral:
    // WB_R = G / R, WB_B = G / B
    let ratio_r = g / r;
    let ratio_b = g / b;

    // Rec.2020 / D65 correlated color temperature empirical Planckian inversion:
    // When blue dominates (ratio_b < 1.0), scene is cool -> raise temperature
    // When red dominates (ratio_r < 1.0), scene is warm -> lower temperature
    let log_rb = (ratio_r / ratio_b).ln();
    let temp = 5500.0 * (-0.72 * log_rb).exp();
    let clamped_temp = temp.clamp(2000.0, 20000.0);

    // Tint represents Green-Magenta axis deviation from the Planckian blackbody locus
    // Expected G for blackbody at this balance is approximately sqrt(R * B)
    let expected_g = (r * b).sqrt();
    let tint_raw = (expected_g - g) / g.max(1e-3) * 100.0;
    let clamped_tint = tint_raw.clamp(-200.0, 200.0);

    AutoWhiteBalanceResult {
        temperature: (clamped_temp / 50.0).round() * 50.0,
        tint: clamped_tint.round(),
    }
}
