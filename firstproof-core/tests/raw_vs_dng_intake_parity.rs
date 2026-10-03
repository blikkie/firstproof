// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Dual-Format Intake Regression Suite: CR2 vs DNG Intake Parity
//!
//! Enforces mathematical and photometric parity between native camera RAW containers
//! (e.g. Canon CR2) and converted Adobe DNG files for the exact same capture.
//! Verifies baseline exposure compensation, active crop dimensions, CFA pattern,
//! white balance coefficients, and sensor-to-Rec.2020 matrix transforms.

use std::path::Path;
use firstproof_core::raw_decoder::{RawDecoder, RawlerDecoder};
use firstproof_core::export::decode::f16_to_f32;

#[test]
#[ignore = "Slow real RAW disk decode"]
fn test_cr2_vs_dng_real_file_parity() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cr2_path = manifest_dir.join("tests/fixtures/_MG_2651.CR2");
    let dng_path = manifest_dir.join("tests/fixtures/_MG_2651.dng");

    if !cr2_path.exists() || !dng_path.exists() {
        eprintln!("Paired RAW/DNG fixtures not found at {:?}, skipping real file test", cr2_path);
        return;
    }

    let decoder = RawlerDecoder;
    let cr2_frame = RawDecoder::decode(&decoder, &cr2_path).expect("Decode CR2 frame");
    let dng_frame = RawDecoder::decode(&decoder, &dng_path).expect("Decode DNG frame");

    // 1. Model Identification & Baseline Exposure Parity
    assert_eq!(cr2_frame.camera_model, "Canon EOS 60D");
    assert_eq!(dng_frame.camera_model, "Canon EOS 60D");
    // For native CR2, baseline_exposure defaults to the calibration profile (0.0 EV).
    // Converted DNG embeds Adobe DNG Tag 50730 BaselineExposure (-0.34 EV for Canon ISO 125 intermediate pull).
    assert_eq!(cr2_frame.baseline_exposure, 0.0);
    assert!(
        (dng_frame.baseline_exposure - (-0.34)).abs() < 1e-3,
        "DNG baseline exposure mismatch: expected -0.34 EV, got {} EV",
        dng_frame.baseline_exposure
    );

    // 2. Active Sensor Area & CFA Geometry Parity
    assert_eq!(cr2_frame.active_width, dng_frame.active_width);
    assert_eq!(cr2_frame.active_height, dng_frame.active_height);
    assert_eq!(cr2_frame.cfa_pattern, dng_frame.cfa_pattern);

    // 3. As-Shot White Balance Parity
    let (cr2_wb_r, cr2_wb_b) = cr2_frame.as_shot_wb();
    let (dng_wb_r, dng_wb_b) = dng_frame.as_shot_wb();
    assert!(
        (cr2_wb_r - dng_wb_r).abs() < 1e-3,
        "WB Red ratio mismatch: CR2 {} vs DNG {}",
        cr2_wb_r,
        dng_wb_r
    );
    assert!(
        (cr2_wb_b - dng_wb_b).abs() < 1e-3,
        "WB Blue ratio mismatch: CR2 {} vs DNG {}",
        cr2_wb_b,
        dng_wb_b
    );

    // 4. Dual-Illuminant Color Matrix Parity
    let cr2_temp = cr2_frame.as_shot_temperature();
    let dng_temp = dng_frame.as_shot_temperature();
    assert!(
        (cr2_temp - dng_temp).abs() < 100.0,
        "Correlated color temperature mismatch: CR2 {} K vs DNG {} K",
        cr2_temp,
        dng_temp
    );

    let cr2_m = cr2_frame.sensor_to_rec2020_at(cr2_temp);
    let dng_m = dng_frame.sensor_to_rec2020_at(dng_temp);
    for row in 0..3 {
        for col in 0..3 {
            assert!(
                (cr2_m[row][col] - dng_m[row][col]).abs() < 0.05,
                "Sensor-to-Rec2020 matrix mismatch at [{}][{}]: CR2 {} vs DNG {}",
                row, col, cr2_m[row][col], dng_m[row][col]
            );
        }
    }

    // 5. 2x2 Bayer Box Binned Photometric Output Continuity
    let (cr2_pixels, cr2_bw, cr2_bh) = cr2_frame.to_binned_linear_f16();
    let (dng_pixels, dng_bw, dng_bh) = dng_frame.to_binned_linear_f16();

    assert_eq!(cr2_bw, dng_bw);
    assert_eq!(cr2_bh, dng_bh);
    assert!(!cr2_pixels.is_empty());
    assert!(!dng_pixels.is_empty());

    // Sample center 200x200 patch to compare linear Rec.2020 luminance
    let cx = (cr2_bw / 2) as usize;
    let cy = (cr2_bh / 2) as usize;
    let mut cr2_lum_sum = 0.0f64;
    let mut dng_lum_sum = 0.0f64;
    let sample_radius = 100;

    for y in (cy - sample_radius)..(cy + sample_radius) {
        for x in (cx - sample_radius)..(cx + sample_radius) {
            let idx = (y * cr2_bw as usize + x) * 4;

            let cr2_r = f16_to_f32(cr2_pixels[idx]);
            let cr2_g = f16_to_f32(cr2_pixels[idx + 1]);
            let cr2_b = f16_to_f32(cr2_pixels[idx + 2]);
            // Rec.2020 photopic luminance: Y = 0.2627 R + 0.6780 G + 0.0593 B
            let cr2_y = 0.2627 * cr2_r + 0.6780 * cr2_g + 0.0593 * cr2_b;
            cr2_lum_sum += cr2_y as f64;

            let dng_r = f16_to_f32(dng_pixels[idx]);
            let dng_g = f16_to_f32(dng_pixels[idx + 1]);
            let dng_b = f16_to_f32(dng_pixels[idx + 2]);
            let dng_y = 0.2627 * dng_r + 0.6780 * dng_g + 0.0593 * dng_b;
            dng_lum_sum += dng_y as f64;
        }
    }

    let n_samples = (sample_radius * 2 * sample_radius * 2) as f64;
    let cr2_mean_y = cr2_lum_sum / n_samples;
    let dng_mean_y = dng_lum_sum / n_samples;

    // Both should be valid midtone exposure light
    assert!(cr2_mean_y > 0.01 && cr2_mean_y < 1.0, "CR2 mean Y out of range: {}", cr2_mean_y);
    assert!(dng_mean_y > 0.01 && dng_mean_y < 1.0, "DNG mean Y out of range: {}", dng_mean_y);

    // Delta EV between CR2 and DNG accounting for DNG baseline exposure gain (-0.34 EV)
    let baseline_delta_ev = (cr2_frame.baseline_exposure - dng_frame.baseline_exposure) as f64;
    let photometric_delta_ev = ((cr2_mean_y / dng_mean_y).log2() - baseline_delta_ev).abs();
    assert!(
        photometric_delta_ev < 0.10,
        "Photometric exposure divergence between CR2 and DNG exceeds 0.10 EV: delta_ev = {:.4} (CR2={:.4}, DNG={:.4})",
        photometric_delta_ev,
        cr2_mean_y,
        dng_mean_y
    );
}

#[test]
fn test_synthetic_dual_format_container_parity() {
    use firstproof_engine::calibration::CameraCalibrationRegistry;
    use firstproof_engine::container::RawContainerManifest;

    // 1. Unregistered camera falls back to standard family default
    let cal_canon = CameraCalibrationRegistry::lookup_or_fallback("Canon", "EOS 90D");
    assert_eq!(cal_canon.clean_name, "Canon EOS Standard");
    assert_eq!(cal_canon.baseline_exposure, 0.0);
    assert!(cal_canon.color_matrix_daylight.is_some());

    let cal_sony = CameraCalibrationRegistry::lookup_or_fallback("Sony", "ILCE-7RM5");
    assert_eq!(cal_sony.clean_name, "Sony Alpha Standard");
    assert_eq!(cal_sony.baseline_exposure, 0.50);

    // 2. Synthetic DNG container with BaselineExposure tag 50730 overrides fallback
    let mut fake_dng = vec![
        b'I', b'I', 42, 0, 8, 0, 0, 0, // Header
        2, 0, // 2 IFD entries
        // Tag 50706 (DNGVersion): BYTE count 4
        0x12, 0xC6, 1, 0, 4, 0, 0, 0, 1, 4, 0, 0,
        // Tag 50730 (BaselineExposure): SRATIONAL count 1, offset 38
        0x2A, 0xC6, 10, 0, 1, 0, 0, 0, 38, 0, 0, 0,
        0, 0, 0, 0, // Next IFD offset
    ];
    // Rational at offset 38: num=35, den=100 (+0.35 EV)
    fake_dng.extend_from_slice(&35i32.to_le_bytes());
    fake_dng.extend_from_slice(&100i32.to_le_bytes());

    let manifest = RawContainerManifest::parse(&fake_dng).expect("Parse synthetic DNG");
    assert!(manifest.is_dng);
    assert!((manifest.baseline_exposure.unwrap() - 0.35).abs() < 1e-4);
}
