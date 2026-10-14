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
    assert_eq!(cr2_frame.baseline_sharpness, 1.25);
    assert_eq!(dng_frame.baseline_sharpness, 1.25);

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

    // 2. Synthetic DNG container with BaselineExposure (tag 50730) and BaselineSharpness (tag 50732)
    let mut fake_dng = vec![
        b'I', b'I', 42, 0, 8, 0, 0, 0, // Header
        3, 0, // 3 IFD entries
        // Tag 50706 (DNGVersion): BYTE count 4
        0x12, 0xC6, 1, 0, 4, 0, 0, 0, 1, 4, 0, 0,
        // Tag 50730 (BaselineExposure): SRATIONAL count 1, offset 50
        0x2A, 0xC6, 10, 0, 1, 0, 0, 0, 50, 0, 0, 0,
        // Tag 50732 (BaselineSharpness): RATIONAL count 1, offset 58
        0x2C, 0xC6, 5, 0, 1, 0, 0, 0, 58, 0, 0, 0,
        0, 0, 0, 0, // Next IFD offset
    ];
    // Rational at offset 50: num=35, den=100 (+0.35 EV)
    fake_dng.extend_from_slice(&35i32.to_le_bytes());
    fake_dng.extend_from_slice(&100i32.to_le_bytes());
    // Rational at offset 58: num=125, den=100 (1.25x sharpness)
    fake_dng.extend_from_slice(&125u32.to_le_bytes());
    fake_dng.extend_from_slice(&100u32.to_le_bytes());

    let manifest = RawContainerManifest::parse(&fake_dng).expect("Parse synthetic DNG");
    assert!(manifest.is_dng);
    assert!((manifest.baseline_exposure.unwrap() - 0.35).abs() < 1e-4);
    assert!((manifest.baseline_sharpness.unwrap() - 1.25).abs() < 1e-4);
}

#[test]
fn test_leica_q2_matrix_parity() {
    use firstproof_core::raw_decoder::{RawDecoder, RawlerDecoder, RawSensorFrameExt};
    let path = Path::new("../target/raw_samples/leica_q2.dng");
    if !path.exists() {
        return;
    }
    let decoder = RawlerDecoder;
    let frame = RawDecoder::decode(&decoder, path).expect("Decode Leica Q2 frame");
    let temp = frame.as_shot_temperature();
    let m_binned = frame.compute_sensor_to_rec2020();
    let m_ext = frame.sensor_to_rec2020_matrix_at(temp);
    assert_eq!(m_binned, m_ext.0, "Binned viewport matrix and demosaic export matrix must be bit-identical");
}

#[test]
fn test_fuji_xt3_decode() {
    use firstproof_core::raw_decoder::{RawDecoder, RawlerDecoder};
    let path = Path::new("../target/raw_samples/fuji_xt3.raf");
    if !path.exists() {
        println!("Fuji file not found at {:?}", path);
        return;
    }
    let decoder = RawlerDecoder;
    let frame = RawDecoder::decode(&decoder, path).expect("Decode Fuji XT3 frame");
    println!("CFA pattern: {:?}", frame.cfa_pattern);
    println!("Dimensions: full {}x{}, active {}x{}, offset {},{}",
        frame.full_width, frame.full_height, frame.active_width, frame.active_height, frame.active_x, frame.active_y);
    println!("Black levels: {:?}", frame.black_levels);
    println!("White levels: {:?}", frame.white_levels);
    println!("WB: {:?}", frame.as_shot_wb());
    println!("Color matrix: {:?}", frame.color_matrix);
    println!("Camera model: {}", frame.camera_model);

    // Verify 1: decode_raw_preview_binned_buffer produces a valid, balanced preview
    let (bw, bh, binned_bytes) = firstproof_core::export::decode_raw_preview_binned_buffer(path)
        .expect("binned preview decode");
    assert!(bw > 0 && bh > 0, "Binned preview must have non-zero dimensions");
    println!("Binned preview: {}x{}, bytes: {}", bw, bh, binned_bytes.len());

    use firstproof_core::export::f16_to_f32;
    // Check pixel at (100, 100) in binned preview
    let b_idx = (100 * bw as usize + 100) * 8;
    let b_r = f16_to_f32(u16::from_le_bytes([binned_bytes[b_idx], binned_bytes[b_idx + 1]]));
    let b_g = f16_to_f32(u16::from_le_bytes([binned_bytes[b_idx + 2], binned_bytes[b_idx + 3]]));
    let b_b = f16_to_f32(u16::from_le_bytes([binned_bytes[b_idx + 4], binned_bytes[b_idx + 5]]));
    println!("Binned preview pixel (100, 100): R={:.4}, G={:.4}, B={:.4}", b_r, b_g, b_b);
    assert!(b_g > 0.0, "Green channel must be positive");
    assert!((b_r / b_g - 1.0).abs() < 0.5, "Red/Green ratio must be balanced, not pink/magenta blowout: R/G={}", b_r / b_g);
    assert!((b_b / b_g - 1.0).abs() < 0.5, "Blue/Green ratio must be balanced, not pink/magenta blowout: B/G={}", b_b / b_g);

    // Verify 2: Full demosaic via decode_image produces matching balanced colors
    let demosaiced = firstproof_core::export::decode_image(path).expect("full demosaic");
    let d_idx = (300 * demosaiced.width + 300) * 3;
    let d_r = demosaiced.data[d_idx];
    let d_g = demosaiced.data[d_idx + 1];
    let d_b = demosaiced.data[d_idx + 2];
    println!("Full demosaic pixel (300, 300): R={:.4}, G={:.4}, B={:.4}", d_r, d_g, d_b);
    assert!(d_g > 0.0, "Green channel must be positive");
    assert!((d_r / d_g - 1.0).abs() < 0.5, "Red/Green ratio in full demosaic must be balanced: R/G={}", d_r / d_g);
    assert!((d_b / d_g - 1.0).abs() < 0.5, "Blue/Green ratio in full demosaic must be balanced: B/G={}", d_b / d_g);
}

#[test]
fn test_sample_metadata_extraction() {
    let files = [
        "canon_5div.cr2",
        "canon_r5.cr3",
        "fuji_xt3.raf",
        "leica_q2.dng",
        "nikon_z6.nef",
        "olympus_em1.orf",
        "panasonic_s5.rw2",
        "sony_a7iii.arw",
    ];

    for f in &files {
        let path = format!("../target/raw_samples/{}", f);
        if !Path::new(&path).exists() {
            continue;
        }
        let meta = firstproof_core::raw::extract_metadata(&path).expect("extract metadata");
        println!("=== File: {} ===", f);
        println!("  Make/Model: {:?} / {:?}", meta.camera_make, meta.camera_model);
        println!("  Lens: {:?}", meta.lens_model);
        println!("  ISO: {:?}", meta.iso);
        println!("  Aperture: {:?}", meta.aperture);
        println!("  Shutter: {:?}", meta.shutter_speed);
        println!("  Focal length: {:?}", meta.focal_length);

        assert!(meta.camera_make.is_some(), "Camera make must be present for {}", f);
        assert!(meta.camera_model.is_some(), "Camera model must be present for {}", f);
        assert!(meta.iso.is_some(), "ISO must be present for {}", f);
        assert!(meta.aperture.is_some(), "Aperture must be present for {}", f);
        assert!(meta.shutter_speed.is_some(), "Shutter speed must be present for {}", f);
        assert!(meta.focal_length.is_some(), "Focal length must be present for {}", f);
    }
}

#[test]
fn test_olympus_color_chart_engine_parity_and_stability() {
    let path = Path::new("../target/raw_samples/olympus_em1.orf");
    if !path.exists() {
        return;
    }

    use firstproof_core::export::{decode_image, decode_raw_preview_binned_buffer, f16_to_f32};

    // 1. Decode binned (fast viewport) and full archival demosaic
    let (bw, bh, binned_bytes) = decode_raw_preview_binned_buffer(path).expect("binned preview decode");
    let full_img = decode_image(path).expect("full demosaic decode");

    assert!(bw > 0 && bh > 0);
    assert!(full_img.width > 0 && full_img.height > 0);

    // 2. Spatial Grid Parity Audit across 25 probe locations (5x5 grid across active area)
    let mut total_delta_r = 0.0f32;
    let mut total_delta_g = 0.0f32;
    let mut total_delta_b = 0.0f32;
    let mut sample_count = 0;

    for gy in 1..6 {
        for gx in 1..6 {
            let rx = (gx as f32) / 6.0;
            let ry = (gy as f32) / 6.0;

            let bx = ((bw as f32) * rx) as usize;
            let by = ((bh as f32) * ry) as usize;
            let b_idx = (by * (bw as usize) + bx) * 8;
            let b_r = f16_to_f32(u16::from_le_bytes([binned_bytes[b_idx], binned_bytes[b_idx + 1]]));
            let b_g = f16_to_f32(u16::from_le_bytes([binned_bytes[b_idx + 2], binned_bytes[b_idx + 3]]));
            let b_b = f16_to_f32(u16::from_le_bytes([binned_bytes[b_idx + 4], binned_bytes[b_idx + 5]]));

            let fx = ((full_img.width as f32) * rx) as usize;
            let fy = ((full_img.height as f32) * ry) as usize;
            let f_idx = (fy * full_img.width + fx) * 3;
            let f_r = full_img.data[f_idx];
            let f_g = full_img.data[f_idx + 1];
            let f_b = full_img.data[f_idx + 2];

            // Invariants: No NaN/Inf, strictly non-negative
            assert!(b_r.is_finite() && b_g.is_finite() && b_b.is_finite(), "Binned values must be finite");
            assert!(f_r.is_finite() && f_g.is_finite() && f_b.is_finite(), "Full demosaic values must be finite");
            assert!(b_r >= 0.0 && b_g >= 0.0 && b_b >= 0.0, "Binned values must be non-negative");
            assert!(f_r >= 0.0 && f_g >= 0.0 && f_b >= 0.0, "Full demosaic values must be non-negative");

            total_delta_r += (b_r - f_r).abs();
            total_delta_g += (b_g - f_g).abs();
            total_delta_b += (b_b - f_b).abs();
            sample_count += 1;
        }
    }

    let avg_dr = total_delta_r / sample_count as f32;
    let avg_dg = total_delta_g / sample_count as f32;
    let avg_db = total_delta_b / sample_count as f32;

    println!("Olympus Color Chart Field Parity (25 points): avg delta R={:.5}, G={:.5}, B={:.5}", avg_dr, avg_dg, avg_db);

    // Dual-domain parity invariant: Mean delta across field must remain under 0.02
    assert!(avg_dr < 0.02, "Red channel parity must be within 0.02, got {}", avg_dr);
    assert!(avg_dg < 0.02, "Green channel parity must be within 0.02, got {}", avg_dg);
    assert!(avg_db < 0.02, "Blue channel parity must be within 0.02, got {}", avg_db);
}

