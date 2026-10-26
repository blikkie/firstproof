//! Sensitometric EV Zone Histogram & Sensor Clipping HUD
//!
//! Evaluates linear scene radiance in Linear Rec.2020 prior to display tone-mapping,
//! measuring true optical dynamic range and sensor exposure relative to 18% middle grey.
//!
//! Log2 EV Stop Scale:
//! - Physical scale spanning -8.0 EV (Zone -III / deep shadow floor) to +4.0 EV (Zone IX / specular highlight)
//! - 18% middle grey (0.18 linear scene radiance) is 0.0 EV (Zone V)
//! - Evaluated in 120 discrete bins (10 bins per EV stop over 12 stops)
//! - Ansel Adams Zone markers: Zone 0 (-5 EV) to Zone X (+5 EV / clip)
//!
//! Clipping Telemetry:
//! - Shadow clip: Pixels crushed at noise floor (Y <= 0.18 * 2^-8.0 = 0.0007)
//! - Sensor clip: Photosite saturation where raw R, G, or B >= 1.0
//! - Display roll-off: Preserved highlights entering AgX filmic compression shoulder (Y >= 0.18 * 2^2.0 = 0.72, Zone VII+)

use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvHistogramRecord {
    pub bins_luminance: Vec<f32>,
    pub bins_r: Vec<f32>,
    pub bins_g: Vec<f32>,
    pub bins_b: Vec<f32>,
    pub shadow_clip_percent: f32,
    pub sensor_clip_percent: f32,
    pub display_rolloff_percent: f32,
    pub ev_min: f32,
    pub ev_max: f32,
    pub num_bins: u32,
}

pub const EV_MIN: f32 = -8.0;
pub const EV_MAX: f32 = 4.0;
pub const NUM_BINS: usize = 120;
pub const MIDDLE_GREY_LUM: f32 = 0.18;

// ITU-R BT.2020 photopic luminance coefficients
pub const REC2020_LUM_R: f32 = 0.2627;
pub const REC2020_LUM_G: f32 = 0.6780;
pub const REC2020_LUM_B: f32 = 0.0593;

/// Computes the Sensitometric EV Zone Histogram from an RGBA 16-bit half-float buffer.
pub fn compute_ev_histogram_from_f16_buffer(
    width: u32,
    height: u32,
    data: &[u8],
    exposure: f32,
    contrast: f32,
    highlights: f32,
    shadows: f32,
    whites: f32,
    blacks: f32,
    crop_left: f32,
    crop_top: f32,
    crop_right: f32,
    crop_bottom: f32,
    lens_correction_enabled: bool,
    tca_correction_enabled: bool,
) -> Option<EvHistogramRecord> {
    let pixel_count = (width * height) as usize;
    if pixel_count == 0 || data.len() < pixel_count * 8 {
        return None;
    }

    let (c_left, c_top, c_right, c_bottom) = if lens_correction_enabled {
        // Geometric distortion correction applies auto-crop scaling to eliminate peripheral warped corners
        let inset_x = (crop_right - crop_left).max(0.0) * 0.02;
        let inset_y = (crop_bottom - crop_top).max(0.0) * 0.02;
        (
            (crop_left + inset_x).clamp(0.0, 1.0),
            (crop_top + inset_y).clamp(0.0, 1.0),
            (crop_right - inset_x).clamp(0.0, 1.0),
            (crop_bottom - inset_y).clamp(0.0, 1.0),
        )
    } else {
        (
            crop_left.clamp(0.0, 1.0),
            crop_top.clamp(0.0, 1.0),
            crop_right.clamp(0.0, 1.0),
            crop_bottom.clamp(0.0, 1.0),
        )
    };

    let (x_start, x_end) = if c_right > c_left {
        (
            ((c_left * width as f32).floor() as usize).min(width as usize),
            ((c_right * width as f32).ceil() as usize).min(width as usize),
        )
    } else {
        (0, width as usize)
    };

    let (y_start, y_end) = if c_bottom > c_top {
        (
            ((c_top * height as f32).floor() as usize).min(height as usize),
            ((c_bottom * height as f32).ceil() as usize).min(height as usize),
        )
    } else {
        (0, height as usize)
    };

    let crop_w = (x_end.saturating_sub(x_start)).max(1);
    let crop_h = (y_end.saturating_sub(y_start)).max(1);
    let crop_pixel_count = crop_w * crop_h;

    // Subsample ~30,000 pixels across the cropped frame for sub-millisecond decoupled execution
    let target_samples = 30_000usize;
    let step = (crop_pixel_count / target_samples).max(1);

    let mut counts_lum = vec![0u32; NUM_BINS];
    let mut counts_r = vec![0u32; NUM_BINS];
    let mut counts_g = vec![0u32; NUM_BINS];
    let mut counts_b = vec![0u32; NUM_BINS];

    let mut shadow_clipped = 0usize;
    let mut sensor_clipped = 0usize;
    let mut rolloff_count = 0usize;
    let mut total_samples = 0usize;

    let exposure_gain = 2.0f32.powf(exposure);
    let has_tone_recovery = highlights != 0.0 || shadows != 0.0 || whites != 0.0 || blacks != 0.0;
    let contrast_gain = if contrast >= 0.0 {
        1.0 + (contrast / 100.0)
    } else {
        1.0 + (contrast / 100.0) * 0.60
    };

    for idx in (0..crop_pixel_count).step_by(step) {
        let local_x = idx % crop_w;
        let local_y = idx / crop_w;
        let x = x_start + local_x;
        let y = y_start + local_y;
        if x >= width as usize || y >= height as usize {
            continue;
        }

        let pixel_idx = y * (width as usize) + x;
        let offset = pixel_idx * 8;
        if offset + 6 > data.len() {
            break;
        }

        let r_u16 = u16::from_le_bytes([data[offset], data[offset + 1]]);
        let g_u16 = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
        let b_u16 = u16::from_le_bytes([data[offset + 4], data[offset + 5]]);

        let mut raw_r = crate::raw::f16_to_f32(r_u16);
        let raw_g = crate::raw::f16_to_f32(g_u16);
        let mut raw_b = crate::raw::f16_to_f32(b_u16);

        let is_sensor_saturated = raw_r >= 1.0 || raw_g >= 1.0 || raw_b >= 1.0;

        // Transverse chromatic aberration (TCA) alignment: reconciles R and B lateral phase misalignments
        if tca_correction_enabled {
            let r_diff = raw_r - raw_g;
            let b_diff = raw_b - raw_g;
            if r_diff.abs() > 0.4 {
                raw_r = raw_g + r_diff * 0.5;
            }
            if b_diff.abs() > 0.4 {
                raw_b = raw_g + b_diff * 0.5;
            }
        }

        let mut r = (raw_r * exposure_gain).max(0.0);
        let mut g = (raw_g * exposure_gain).max(0.0);
        let mut b = (raw_b * exposure_gain).max(0.0);

        // Apply four-zone tone recovery (highlights, shadows, whites, blacks)
        if has_tone_recovery {
            let (r_rec, g_rec, b_rec) = crate::color::apply_tone_recovery(
                r, g, b, highlights, shadows, whites, blacks,
            );
            r = r_rec;
            g = g_rec;
            b = b_rec;
        }

        // Apply contrast pivoting around Zone V (0.18 middle grey)
        if contrast != 0.0 {
            let apply_contrast = |v: f32| -> f32 {
                if v <= 1e-6 {
                    0.0
                } else {
                    let ev = (v / MIDDLE_GREY_LUM).log2();
                    (MIDDLE_GREY_LUM * 2.0f32.powf(ev * contrast_gain)).max(0.0)
                }
            };
            r = apply_contrast(r);
            g = apply_contrast(g);
            b = apply_contrast(b);
        }

        let lum = REC2020_LUM_R * r + REC2020_LUM_G * g + REC2020_LUM_B * b;

        // Highlight clipping check: true silicon sensor saturation OR post-develop highlight clipping exceeding Zone X (+4.0 EV ceiling)
        let is_develop_highlight_clipped = r >= (MIDDLE_GREY_LUM * 2.0f32.powf(EV_MAX))
            || g >= (MIDDLE_GREY_LUM * 2.0f32.powf(EV_MAX))
            || b >= (MIDDLE_GREY_LUM * 2.0f32.powf(EV_MAX));

        if is_sensor_saturated || is_develop_highlight_clipped {
            sensor_clipped += 1;
        }

        // Shadow floor clip check: scene radiance at or below -8.0 EV
        if lum <= (MIDDLE_GREY_LUM * 2.0f32.powf(EV_MIN)) {
            shadow_clipped += 1;
        }

        // AgX filmic shoulder compression check: Zone VII+ (+2.0 EV above middle grey)
        if lum >= (MIDDLE_GREY_LUM * 2.0f32.powf(2.0)) {
            rolloff_count += 1;
        }

        let val_to_bin = |v: f32| -> usize {
            if v <= 1e-6 {
                0
            } else {
                let ev = (v / MIDDLE_GREY_LUM).log2();
                if ev <= EV_MIN {
                    0
                } else if ev >= EV_MAX {
                    NUM_BINS - 1
                } else {
                    let frac = (ev - EV_MIN) / (EV_MAX - EV_MIN);
                    ((frac * NUM_BINS as f32).floor() as usize).min(NUM_BINS - 1)
                }
            }
        };

        counts_lum[val_to_bin(lum)] += 1;
        counts_r[val_to_bin(r)] += 1;
        counts_g[val_to_bin(g)] += 1;
        counts_b[val_to_bin(b)] += 1;

        total_samples += 1;
    }

    if total_samples == 0 {
        return None;
    }

    // Normalization with square-root damping for photographic dynamic range readability
    let normalize_counts = |counts: &[u32]| -> Vec<f32> {
        let max_val = counts.iter().copied().map(|c| (c as f32).sqrt()).fold(0.0f32, f32::max);
        if max_val <= 0.0 {
            vec![0.0; NUM_BINS]
        } else {
            counts.iter().map(|&c| ((c as f32).sqrt() / max_val).clamp(0.0, 1.0)).collect()
        }
    };

    Some(EvHistogramRecord {
        bins_luminance: normalize_counts(&counts_lum),
        bins_r: normalize_counts(&counts_r),
        bins_g: normalize_counts(&counts_g),
        bins_b: normalize_counts(&counts_b),
        shadow_clip_percent: (shadow_clipped as f32 / total_samples as f32) * 100.0,
        sensor_clip_percent: (sensor_clipped as f32 / total_samples as f32) * 100.0,
        display_rolloff_percent: (rolloff_count as f32 / total_samples as f32) * 100.0,
        ev_min: EV_MIN,
        ev_max: EV_MAX,
        num_bins: NUM_BINS as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::f32_to_f16;

    fn make_test_buffer(r: f32, g: f32, b: f32, count: usize) -> Vec<u8> {
        let mut buf = Vec::with_capacity(count * 8);
        let r_u16 = f32_to_f16(r);
        let g_u16 = f32_to_f16(g);
        let b_u16 = f32_to_f16(b);
        let a_u16 = f32_to_f16(1.0);

        for _ in 0..count {
            buf.extend_from_slice(&r_u16.to_le_bytes());
            buf.extend_from_slice(&g_u16.to_le_bytes());
            buf.extend_from_slice(&b_u16.to_le_bytes());
            buf.extend_from_slice(&a_u16.to_le_bytes());
        }
        buf
    }

    #[test]
    fn test_zone_v_middle_grey_centering() {
        // 18% middle grey linear radiance should land squarely in Zone V (0.0 EV)
        // Bin formula: frac = (0.0 - (-8.0)) / 12.0 = 8.0 / 12.0 = 2/3.
        // 2/3 * 120 = 80.
        let buf = make_test_buffer(0.18, 0.18, 0.18, 1000);
        let hist = compute_ev_histogram_from_f16_buffer(100, 10, &buf, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, false, true)
            .expect("histogram generated");

        assert_eq!(hist.num_bins, 120);
        assert_eq!(hist.shadow_clip_percent, 0.0);
        assert_eq!(hist.sensor_clip_percent, 0.0);
        assert_eq!(hist.display_rolloff_percent, 0.0);

        // Peak bin must be adjacent to Zone V nominal center (bin 79 or 80 within f16 rounding of 0.18)
        assert!(hist.bins_luminance[79] == 1.0 || hist.bins_luminance[80] == 1.0);
        assert!(hist.bins_r[79] == 1.0 || hist.bins_r[80] == 1.0);
        assert!(hist.bins_g[79] == 1.0 || hist.bins_g[80] == 1.0);
        assert!(hist.bins_b[79] == 1.0 || hist.bins_b[80] == 1.0);
    }

    #[test]
    fn test_exposure_offset_predictable_shift() {
        let buf = make_test_buffer(0.18, 0.18, 0.18, 1000);

        // +1.0 EV exposure gain shifts exactly 10 bins (1 full stop = 10 bins)
        let hist_plus_1 = compute_ev_histogram_from_f16_buffer(100, 10, &buf, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, false, true)
            .expect("histogram generated");
        assert!(hist_plus_1.bins_luminance[89] == 1.0 || hist_plus_1.bins_luminance[90] == 1.0);

        // -2.0 EV exposure cut shifts exactly 20 bins (2 full stops = 20 bins)
        let hist_minus_2 = compute_ev_histogram_from_f16_buffer(100, 10, &buf, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, false, true)
            .expect("histogram generated");
        assert!(hist_minus_2.bins_luminance[59] == 1.0 || hist_minus_2.bins_luminance[60] == 1.0);
    }

    #[test]
    fn test_shadow_and_sensor_clipping_telemetry() {
        // 1. Crushed black pixels
        let black_buf = make_test_buffer(0.0, 0.0, 0.0, 500);
        let hist_black = compute_ev_histogram_from_f16_buffer(50, 10, &black_buf, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, false, true).unwrap();
        assert_eq!(hist_black.shadow_clip_percent, 100.0);
        assert_eq!(hist_black.sensor_clip_percent, 0.0);
        assert_eq!(hist_black.bins_luminance[0], 1.0);

        // 2. Hardware photosite sensor clip (raw >= 1.0)
        let clip_buf = make_test_buffer(1.2, 0.5, 0.5, 500);
        let hist_clip = compute_ev_histogram_from_f16_buffer(50, 10, &clip_buf, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, false, true).unwrap();
        assert_eq!(hist_clip.sensor_clip_percent, 100.0);
        assert_eq!(hist_clip.shadow_clip_percent, 0.0);

        // 3. AgX filmic shoulder roll-off (> +2.0 EV, e.g. 0.85 linear radiance without sensor clip)
        let rolloff_buf = make_test_buffer(0.85, 0.85, 0.85, 500);
        let hist_rolloff = compute_ev_histogram_from_f16_buffer(50, 10, &rolloff_buf, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, false, true).unwrap();
        assert_eq!(hist_rolloff.sensor_clip_percent, 0.0);
        assert_eq!(hist_rolloff.display_rolloff_percent, 100.0);
    }

    #[test]
    fn test_tone_recovery_histogram_reactivity() {
        // 1. Highlight recovery: pulling highlights (-100) on Zone VII/VIII patch (0.72) compresses peak leftward
        let high_buf = make_test_buffer(0.72, 0.72, 0.72, 1000);
        let hist_base = compute_ev_histogram_from_f16_buffer(100, 10, &high_buf, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, false, true).unwrap();
        let hist_hl_pull = compute_ev_histogram_from_f16_buffer(100, 10, &high_buf, 0.0, 0.0, -100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, false, true).unwrap();

        let peak_base = hist_base.bins_luminance.iter().position(|&v| v == 1.0).unwrap();
        let peak_pulled = hist_hl_pull.bins_luminance.iter().position(|&v| v == 1.0).unwrap();
        assert!(peak_pulled < peak_base, "Pulling highlights (-100) must shift peak leftward");

        // 2. Shadow lift: lifting shadows (+100) on Zone II shadow patch (0.022) lifts peak rightward
        let shadow_buf = make_test_buffer(0.022, 0.022, 0.022, 1000);
        let hist_s_base = compute_ev_histogram_from_f16_buffer(100, 10, &shadow_buf, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, false, true).unwrap();
        let hist_s_lift = compute_ev_histogram_from_f16_buffer(100, 10, &shadow_buf, 0.0, 0.0, 0.0, 100.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, false, true).unwrap();

        let peak_s_base = hist_s_base.bins_luminance.iter().position(|&v| v == 1.0).unwrap();
        let peak_s_lift = hist_s_lift.bins_luminance.iter().position(|&v| v == 1.0).unwrap();
        assert!(peak_s_lift > peak_s_base, "Lifting shadows (+100) must shift peak rightward");
    }

    #[test]
    fn test_crop_histogram_sampling_isolation() {
        // Build a split buffer: left half (x < 50) is pure black (0.0), right half (x >= 50) is saturated white (1.5)
        let width = 100usize;
        let height = 20usize;
        let mut split_buf = Vec::with_capacity(width * height * 8);

        let black_r = f32_to_f16(0.0).to_le_bytes();
        let white_r = f32_to_f16(1.5).to_le_bytes();
        let one_a = f32_to_f16(1.0).to_le_bytes();

        for _y in 0..height {
            for x in 0..width {
                if x < 50 {
                    split_buf.extend_from_slice(&black_r);
                    split_buf.extend_from_slice(&black_r);
                    split_buf.extend_from_slice(&black_r);
                    split_buf.extend_from_slice(&one_a);
                } else {
                    split_buf.extend_from_slice(&white_r);
                    split_buf.extend_from_slice(&white_r);
                    split_buf.extend_from_slice(&white_r);
                    split_buf.extend_from_slice(&one_a);
                }
            }
        }

        // 1. Full uncropped frame: both shadow and sensor clipping should be ~50%
        let full = compute_ev_histogram_from_f16_buffer(
            width as u32, height as u32, &split_buf,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0,
            false, true,
        ).unwrap();
        assert!((full.shadow_clip_percent - 50.0).abs() < 2.0);
        assert!((full.sensor_clip_percent - 50.0).abs() < 2.0);

        // 2. Cropped to left half (0.0 .. 0.5): strictly black, 100% shadow clip, 0% sensor clip
        let crop_left = compute_ev_histogram_from_f16_buffer(
            width as u32, height as u32, &split_buf,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 1.0,
            false, true,
        ).unwrap();
        assert_eq!(crop_left.shadow_clip_percent, 100.0);
        assert_eq!(crop_left.sensor_clip_percent, 0.0);

        // 3. Cropped to right half (0.5 .. 1.0): strictly white, 0% shadow clip, 100% sensor clip
        let crop_right = compute_ev_histogram_from_f16_buffer(
            width as u32, height as u32, &split_buf,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 1.0, 1.0,
            false, true,
        ).unwrap();
        assert_eq!(crop_right.shadow_clip_percent, 0.0);
        assert_eq!(crop_right.sensor_clip_percent, 100.0);
    }

    #[test]
    fn test_lens_distortion_auto_crop_histogram() {
        // Build a border buffer: extreme outer 1 pixel border is blown white (1.5), interior is black (0.0)
        let width = 100usize;
        let height = 100usize;
        let mut border_buf = Vec::with_capacity(width * height * 8);

        let black_r = f32_to_f16(0.0).to_le_bytes();
        let white_r = f32_to_f16(1.5).to_le_bytes();
        let one_a = f32_to_f16(1.0).to_le_bytes();

        for y in 0..height {
            for x in 0..width {
                let is_outer_border = x == 0 || x == width - 1 || y == 0 || y == height - 1;
                let val = if is_outer_border { &white_r } else { &black_r };
                border_buf.extend_from_slice(val);
                border_buf.extend_from_slice(val);
                border_buf.extend_from_slice(val);
                border_buf.extend_from_slice(&one_a);
            }
        }

        // Without lens correction: border is sampled, non-zero sensor clip
        let uncorrected = compute_ev_histogram_from_f16_buffer(
            width as u32, height as u32, &border_buf,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0,
            false, true,
        ).unwrap();
        assert!(uncorrected.sensor_clip_percent > 0.0);

        // With lens correction enabled: auto-crop insets the boundary past the 1px border, 0% sensor clip
        let corrected = compute_ev_histogram_from_f16_buffer(
            width as u32, height as u32, &border_buf,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0,
            true, true,
        ).unwrap();
        assert_eq!(corrected.sensor_clip_percent, 0.0);
    }
}
