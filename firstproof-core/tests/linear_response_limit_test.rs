// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Unit and integration tests for Story 3.15:
//! Sensor Non-Linearity & Anti-Blooming Knee Engine (LinearResponseLimit).

use firstproof_engine::calibration::CameraCalibrationRegistry;
use firstproof_engine::container::{
    tiff, RawContainerManifest, DNG_TAG_BASELINE_EXPOSURE, DNG_TAG_LINEAR_RESPONSE_LIMIT,
    TAG_UNIQUE_CAMERA_MODEL,
};
use firstproof_engine::raw::blend_sensor_highlights;

#[test]
fn test_linear_response_limit_monotonicity_across_21_step_wedge() {
    // 21-step photometric grey wedge from -10 EV to +6.5 EV relative to 18% middle grey
    let ev_steps: [f32; 21] = [
        -10.0, -9.0, -8.0, -7.0, -6.0, -5.0, -4.0, -3.0, -2.0, -1.0, 0.0, 0.5, 1.0, 1.5, 2.0,
        2.2, 2.3, 2.4, 2.45, 2.47, 2.5,
    ];

    let limits = [0.95f32, 0.96, 0.98];
    let wb_r = 1.8f32;
    let wb_b = 1.4f32;

    for &limit in &limits {
        let mut prev_out_r = -1.0f32;
        let mut prev_out_g = -1.0f32;
        let mut prev_out_b = -1.0f32;

        for &ev in &ev_steps {
            // Normalized signal: 18% middle grey at 0 EV maps to 0.18
            let raw_signal = (0.18f32 * 2.0f32.powf(ev)).clamp(0.0, 1.0);
            let (out_r, out_g, out_b) =
                blend_sensor_highlights(raw_signal, raw_signal, raw_signal, wb_r, wb_b, limit);

            assert!(
                out_r >= prev_out_r - 1e-6,
                "Monotonicity violation on Red channel at EV {}: prev={}, curr={}, limit={}",
                ev,
                prev_out_r,
                out_r,
                limit
            );
            assert!(
                out_g >= prev_out_g - 1e-6,
                "Monotonicity violation on Green channel at EV {}: prev={}, curr={}, limit={}",
                ev,
                prev_out_g,
                out_g,
                limit
            );
            assert!(
                out_b >= prev_out_b - 1e-6,
                "Monotonicity violation on Blue channel at EV {}: prev={}, curr={}, limit={}",
                ev,
                prev_out_b,
                out_b,
                limit
            );

            prev_out_r = out_r;
            prev_out_g = out_g;
            prev_out_b = out_b;
        }
    }

    // Dense grid test across [0.0, 1.0] with 1,000 steps
    for &limit in &limits {
        let steps = 1000;
        let mut prev_g = 0.0f32;
        for i in 0..=steps {
            let s = i as f32 / steps as f32;
            let (_, g, _) = blend_sensor_highlights(s, s, s, wb_r, wb_b, limit);
            assert!(
                g >= prev_g - 1e-6,
                "Dense monotonicity failed for limit {} at s={}: prev={}, curr={}",
                limit,
                s,
                prev_g,
                g
            );
            prev_g = g;
        }
    }
}

#[test]
fn test_continuous_first_derivative_c1_at_knee() {
    let limit = 0.96f32;
    let wb_r = 1.9f32;
    let wb_b = 1.3f32;

    let delta = 1e-5f32;

    // Left derivative at boundary limit (linear regime):
    let (r_limit, g_limit, b_limit) = blend_sensor_highlights(limit, limit, limit, wb_r, wb_b, limit);
    let (r_left, g_left, b_left) =
        blend_sensor_highlights(limit - delta, limit - delta, limit - delta, wb_r, wb_b, limit);
    let dr_left = (r_limit - r_left) / delta;
    let dg_left = (g_limit - g_left) / delta;
    let db_left = (b_limit - b_left) / delta;

    // Right derivative at boundary limit (Hermite rolloff regime):
    let (r_right, g_right, b_right) =
        blend_sensor_highlights(limit + delta, limit + delta, limit + delta, wb_r, wb_b, limit);
    let dr_right = (r_right - r_limit) / delta;
    let dg_right = (g_right - g_limit) / delta;
    let db_right = (b_right - b_limit) / delta;

    // The first derivative must match across the limit boundary (C^1 continuity, no slope kink):
    assert!(
        (dr_left - dr_right).abs() < 5e-2,
        "Red channel slope discontinuity at knee: left={}, right={}, diff={}",
        dr_left,
        dr_right,
        (dr_left - dr_right).abs()
    );
    assert!(
        (dg_left - dg_right).abs() < 5e-2,
        "Green channel slope discontinuity at knee: left={}, right={}, diff={}",
        dg_left,
        dg_right,
        (dg_left - dg_right).abs()
    );
    assert!(
        (db_left - db_right).abs() < 5e-2,
        "Blue channel slope discontinuity at knee: left={}, right={}, diff={}",
        db_left,
        db_right,
        (db_left - db_right).abs()
    );
}

#[test]
fn test_tungsten_highlight_blending_desaturates_to_neutral_white() {
    // Realistic tungsten lighting: blue channel requires high analog/digital gain (~2.4x)
    let wb_r = 1.3f32;
    let wb_b = 2.4f32;
    let limit = 0.96f32;

    // Below knee (e.g. 0.80): color is preserved according to white balance
    let (r_mid, g_mid, b_mid) = blend_sensor_highlights(0.80, 0.80, 0.80, wb_r, wb_b, limit);
    assert!((r_mid - 0.80 * wb_r).abs() < 1e-5);
    assert!((g_mid - 0.80).abs() < 1e-5);
    assert!((b_mid - 0.80 * wb_b).abs() < 1e-5);

    // At full sensor saturation (1.0):
    // Without highlight blending, Blue would be 2.4, Green 1.0, Red 1.3 (harsh blue/cyan cast).
    // With anti-blooming knee, all three channels should converge towards neutral white luminance.
    let (r_sat, g_sat, b_sat) = blend_sensor_highlights(1.0, 1.0, 1.0, wb_r, wb_b, limit);

    // Check that all 3 channels converge to identical neutral luminance at 1.0
    assert!(
        (r_sat - g_sat).abs() < 1e-5,
        "Saturated highlight must be neutral white: R={}, G={}",
        r_sat,
        g_sat
    );
    assert!(
        (g_sat - b_sat).abs() < 1e-5,
        "Saturated highlight must be neutral white: G={}, B={}",
        g_sat,
        b_sat
    );

    // And luminance headroom is preserved (greater than green channel 1.0)
    assert!(
        r_sat >= 1.0,
        "Highlight luminance must preserve full headroom: got {}",
        r_sat
    );
}

#[test]
fn test_asymmetric_colored_highlight_chromatic_preservation() {
    // Atmospheric blue sky capture: Green is saturated at 1.0, Blue is near clipping at 0.95,
    // but Red is low at 0.40 due to natural Rayleigh scattering.
    let wb_r = 1.9434f32;
    let wb_b = 1.5928f32;
    let limit = 0.96f32;

    let (r_out, _g_out, b_out) = blend_sensor_highlights(0.40, 1.0, 0.95, wb_r, wb_b, limit);

    // Unclipped red channel must remain at its honest measured sensor value
    assert!(
        (r_out - 0.40 * wb_r).abs() < 1e-4,
        "Unclipped red photosite signal must not be inflated to grey: got {}",
        r_out
    );

    // Blue channel must maintain its full optical dominance
    assert!(
        b_out > 1.45,
        "Blue highlight signal must preserve headroom: got {}",
        b_out
    );

    // Strong chromatic saturation must be preserved, not wiped to monochrome
    let chroma = (b_out - r_out) / b_out;
    assert!(
        chroma > 0.45,
        "Chromatic contrast must be preserved: got chroma {}",
        chroma
    );
}

#[test]
fn test_cloud_highlight_non_magenta_invariance() {
    // Cloud highlight: Green photosites are saturated (1.0), Blue is saturated (1.0),
    // and Red is bright (0.75, unclipped).
    // White balance multipliers: daylight wb_r = 1.9434, wb_b = 1.5928.
    let wb_r = 1.9434f32;
    let wb_b = 1.5928f32;
    let limit = 0.96f32;

    for r_raw_step in 55..=100 {
        let r_raw = r_raw_step as f32 / 100.0;
        let (r_out, g_out, b_out) = blend_sensor_highlights(r_raw, 1.0, 1.0, wb_r, wb_b, limit);

        // In authentic clouds, Green must NEVER lag behind Red (which creates fluorescent magenta).
        assert!(
            g_out >= r_out - 1e-4,
            "Magenta cloud artifact detected at r_raw={}: r_out={}, g_out={}, b_out={}",
            r_raw,
            r_out,
            g_out,
            b_out
        );
        // And Blue should be balanced
        assert!(
            b_out >= r_out - 1e-4,
            "Cloud highlight balance error at r_raw={}: r_out={}, b_out={}",
            r_raw,
            r_out,
            b_out
        );
    }
}

#[test]
fn test_dng_tag_50734_extraction_and_camera_registry_fallback() {
    // 1. Synthetic DNG with Tag 50734 (LinearResponseLimit) = 975/1000 (0.975)
    let mut buf = vec![0u8; 256];
    buf[0..2].copy_from_slice(b"II");
    buf[2..4].copy_from_slice(&42u16.to_le_bytes());
    buf[4..8].copy_from_slice(&8u32.to_le_bytes());

    let num_entries = 3u16;
    buf[8..10].copy_from_slice(&num_entries.to_le_bytes());
    let mut curr = 10;

    // Entry 1: Tag 50708 (UniqueCameraModel)
    buf[curr..curr + 2].copy_from_slice(&TAG_UNIQUE_CAMERA_MODEL.to_le_bytes());
    buf[curr + 2..curr + 4].copy_from_slice(&tiff::TYPE_ASCII.to_le_bytes());
    buf[curr + 4..curr + 8].copy_from_slice(&10u32.to_le_bytes());
    buf[curr + 8..curr + 12].copy_from_slice(&80u32.to_le_bytes());
    buf[80..90].copy_from_slice(b"Nikon Z 6\0");
    curr += 12;

    // Entry 2: Tag 50730 (BaselineExposure)
    buf[curr..curr + 2].copy_from_slice(&DNG_TAG_BASELINE_EXPOSURE.to_le_bytes());
    buf[curr + 2..curr + 4].copy_from_slice(&tiff::TYPE_SRATIONAL.to_le_bytes());
    buf[curr + 4..curr + 8].copy_from_slice(&1u32.to_le_bytes());
    buf[curr + 8..curr + 12].copy_from_slice(&100u32.to_le_bytes());
    buf[100..104].copy_from_slice(&500i32.to_le_bytes());
    buf[104..108].copy_from_slice(&1000i32.to_le_bytes());
    curr += 12;

    // Entry 3: Tag 50734 (LinearResponseLimit) = 975/1000 = 0.975
    buf[curr..curr + 2].copy_from_slice(&DNG_TAG_LINEAR_RESPONSE_LIMIT.to_le_bytes());
    buf[curr + 2..curr + 4].copy_from_slice(&tiff::TYPE_RATIONAL.to_le_bytes());
    buf[curr + 4..curr + 8].copy_from_slice(&1u32.to_le_bytes());
    buf[curr + 8..curr + 12].copy_from_slice(&110u32.to_le_bytes());
    buf[110..114].copy_from_slice(&975u32.to_le_bytes());
    buf[114..118].copy_from_slice(&1000u32.to_le_bytes());

    let manifest = RawContainerManifest::parse(&buf).expect("Failed to parse synthetic DNG");
    assert_eq!(manifest.linear_response_limit, Some(0.975));

    // 2. Camera Registry Fallbacks:
    let sony_cal = CameraCalibrationRegistry::lookup_or_fallback("Sony", "ILCE-7M3");
    assert_eq!(sony_cal.linear_response_limit, 0.95);

    let canon_cal = CameraCalibrationRegistry::lookup_or_fallback("Canon", "EOS 60D");
    assert_eq!(canon_cal.linear_response_limit, 0.96);

    let nikon_cal = CameraCalibrationRegistry::lookup_or_fallback("Nikon", "D850");
    assert_eq!(nikon_cal.linear_response_limit, 0.98);

    let generic_cal = CameraCalibrationRegistry::lookup_or_fallback("Unknown", "Camera");
    assert_eq!(generic_cal.linear_response_limit, 0.96);
}
