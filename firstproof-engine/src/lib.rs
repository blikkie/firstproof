#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!("firstproof_engine");

pub mod auto;
pub mod calibration;
pub mod color;
pub mod container;
pub mod curve;
pub mod modes;
pub mod preview;
pub mod raw;
pub mod schema;
pub mod settings;
pub mod xmp;

pub use calibration::{CameraCalibration, CameraCalibrationRegistry};
pub use container::RawContainerManifest;


pub use schema::{
    all_adjustments, get_adjustment, kelvin_to_mired_norm, mired_norm_to_kelvin,
    AdjustmentDescriptor, ScaleMode, WorkspaceCategory,
};

pub use auto::{
    calculate_white_balance_from_sample, compute_auto_adjustments_from_f16_buffer,
    compute_auto_adjustments_from_luminances, compute_auto_adjustments_from_u16_slice,
    compute_auto_white_balance_from_u16_slice, AutoAdjustments, AutoWhiteBalanceResult,
};
pub use curve::{
    evaluate_all_curves_lut, evaluate_curve_lut, AllCurvesLutRecord, CurvePointRecord,
    ToneCurvesRecord,
};
pub use modes::{
    ColorProfileMode, DemosaicQuality, EmulsionArchetype, EmulsionFormat, EmulsionGrainMode,
    EnlargerFilterMode, ExportColorSpace, ExportRenderingIntent, PrintMediaMode, RemjetBackingMode,
    RenderTier,
};
pub use preview::{
    decode_exif_orientation, extract_metadata_from_bytes, extract_preview_slice,
    get_jpeg_dimensions, is_displayable_rgb_jpeg, ExifOrientationRecord, ImageMetadataRecord,
    PreviewResultRecord, RawLinearResultRecord,
};
pub use raw::{
    blend_sensor_highlights, f16_to_f32, f32_to_f16, read_dng_baseline_exposure, resolve_baseline_exposure, CfaPattern, RawSensorFrame, CFA_COLOR_BLUE, CFA_COLOR_GREEN, CFA_COLOR_RED,
};
pub use settings::{EditSettings, WhiteBalanceResult};
pub use xmp::{
    format_gps_xmp, generate_xmp_sidecar, parse_curve_points, parse_gps_xmp, parse_xmp_sidecar,
    serialize_curve_points, XmpData,
};
pub use color::{
    apply_lateral_chemical_diffusion, develop_frame_pipeline, develop_pixel_pipeline,
    develop_raw_sensor_frame_archival, evaluate_display_luma, is_neutral_edits,
    linear_rgb_to_srgb8, linear_to_srgb_gamma, srgb_gamma_to_linear, Color3, ColorSpacePrimaries,
    Matrix3x3,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_types_curve_monotonicity() {
        let pts = vec![
            CurvePointRecord { x: 0.0, y: 0.0 },
            CurvePointRecord { x: 0.25, y: 0.20 },
            CurvePointRecord { x: 0.75, y: 0.85 },
            CurvePointRecord { x: 1.0, y: 1.0 },
        ];
        let lut = evaluate_curve_lut(&pts);
        assert_eq!(lut.len(), 256);
        assert!((lut[0] - 0.0).abs() < 1e-4);
        assert!((lut[255] - 1.0).abs() < 1e-4);
        for i in 1..256 {
            assert!(lut[i] >= lut[i - 1] - 1e-5);
        }
    }

    #[test]
    fn test_types_xmp_roundtrip() {
        let mut xmp = XmpData::default();
        xmp.rating = Some(4);
        xmp.status = "Pick".to_string();
        xmp.title = Some("Lake Louise".to_string());
        xmp.edits.exposure = 0.5;
        xmp.edits.contrast = 20.0;
        xmp.edits.temperature = 5600.0;
        xmp.edits.emulsion_archetype = EmulsionArchetype::RochesterPortrait400;

        let xml = generate_xmp_sidecar(&xmp);
        let parsed = parse_xmp_sidecar(&xml).expect("Failed to parse XMP");

        assert_eq!(parsed.rating, Some(4));
        assert_eq!(parsed.status, "Pick");
        assert_eq!(parsed.title, Some("Lake Louise".to_string()));
        assert!((parsed.edits.exposure - 0.5).abs() < 1e-3);
        assert_eq!(
            parsed.edits.emulsion_archetype,
            EmulsionArchetype::RochesterPortrait400
        );
    }

    #[test]
    fn test_dng_subifd_raw_dimension_extraction() {
        // Construct minimal TIFF container with IFD0 (256x171) and SubIFD1 (5184x3456)
        let mut bytes = vec![0u8; 512];
        // Header: Little-Endian 'II', Magic 42, IFD0 offset 8
        bytes[0..4].copy_from_slice(b"II\x2A\x00");
        bytes[4..8].copy_from_slice(&8u32.to_le_bytes());

        // IFD0 at offset 8: 3 entries (0x0100=256, 0x0101=171, 0x014A=pointer to SubIFD at 100)
        let entry_count = 3u16;
        bytes[8..10].copy_from_slice(&entry_count.to_le_bytes());

        // Entry 0: 0x0100 (ImageWidth) = 256
        bytes[10..12].copy_from_slice(&0x0100u16.to_le_bytes()); // Tag
        bytes[12..14].copy_from_slice(&3u16.to_le_bytes());     // SHORT type
        bytes[14..18].copy_from_slice(&1u32.to_le_bytes());     // Count
        bytes[18..22].copy_from_slice(&256u32.to_le_bytes());   // Value

        // Entry 1: 0x0101 (ImageLength) = 171
        bytes[22..24].copy_from_slice(&0x0101u16.to_le_bytes());
        bytes[24..26].copy_from_slice(&3u16.to_le_bytes());
        bytes[26..30].copy_from_slice(&1u32.to_le_bytes());
        bytes[30..34].copy_from_slice(&171u32.to_le_bytes());

        // Entry 2: 0x014A (SubIFDs) -> points to offset 100
        bytes[34..36].copy_from_slice(&0x014Au16.to_le_bytes());
        bytes[36..38].copy_from_slice(&4u16.to_le_bytes());     // LONG type
        bytes[38..42].copy_from_slice(&1u32.to_le_bytes());     // Count = 1
        bytes[42..46].copy_from_slice(&100u32.to_le_bytes());   // Value = Offset 100

        // SubIFD at offset 100: 2 entries (0x0100=5184, 0x0101=3456)
        bytes[100..102].copy_from_slice(&2u16.to_le_bytes());

        bytes[102..104].copy_from_slice(&0x0100u16.to_le_bytes());
        bytes[104..106].copy_from_slice(&4u16.to_le_bytes());
        bytes[106..110].copy_from_slice(&1u32.to_le_bytes());
        bytes[110..114].copy_from_slice(&5184u32.to_le_bytes());

        bytes[114..116].copy_from_slice(&0x0101u16.to_le_bytes());
        bytes[116..118].copy_from_slice(&4u16.to_le_bytes());
        bytes[118..122].copy_from_slice(&1u32.to_le_bytes());
        bytes[122..126].copy_from_slice(&3456u32.to_le_bytes());

        let meta = extract_metadata_from_bytes(&bytes);
        assert_eq!(meta.raw_width, Some(5184));
        assert_eq!(meta.raw_height, Some(3456));
    }

    #[test]
    fn test_decode_exif_orientation_all_cases() {
        assert_eq!(
            decode_exif_orientation(1),
            ExifOrientationRecord { rotation: 0, flip_horizontal: false, flip_vertical: false }
        );
        assert_eq!(
            decode_exif_orientation(2),
            ExifOrientationRecord { rotation: 0, flip_horizontal: true, flip_vertical: false }
        );
        assert_eq!(
            decode_exif_orientation(3),
            ExifOrientationRecord { rotation: 180, flip_horizontal: false, flip_vertical: false }
        );
        assert_eq!(
            decode_exif_orientation(4),
            ExifOrientationRecord { rotation: 0, flip_horizontal: false, flip_vertical: true }
        );
        assert_eq!(
            decode_exif_orientation(5),
            ExifOrientationRecord { rotation: 270, flip_horizontal: true, flip_vertical: false }
        );
        assert_eq!(
            decode_exif_orientation(6),
            ExifOrientationRecord { rotation: 90, flip_horizontal: false, flip_vertical: false }
        );
        assert_eq!(
            decode_exif_orientation(7),
            ExifOrientationRecord { rotation: 90, flip_horizontal: true, flip_vertical: false }
        );
        assert_eq!(
            decode_exif_orientation(8),
            ExifOrientationRecord { rotation: 270, flip_horizontal: false, flip_vertical: false }
        );
        // Out of bounds / fallback
        assert_eq!(
            decode_exif_orientation(0),
            ExifOrientationRecord { rotation: 0, flip_horizontal: false, flip_vertical: false }
        );
        assert_eq!(
            decode_exif_orientation(9),
            ExifOrientationRecord { rotation: 0, flip_horizontal: false, flip_vertical: false }
        );
    }

    #[test]
    fn test_default_temperature_is_5500() {
        let edits = EditSettings::default();
        assert_eq!(edits.temperature, 5500.0);
    }

    #[test]
    fn test_extract_metadata_orientation_tag() {
        // Construct minimal TIFF container with IFD0 containing Orientation = 6 (Rotate 90 CW)
        let mut bytes = vec![0u8; 256];
        bytes[0..4].copy_from_slice(b"II\x2A\x00");
        bytes[4..8].copy_from_slice(&8u32.to_le_bytes());

        // IFD0 at offset 8: 1 entry (0x0112 = Orientation, SHORT, count 1, value 6)
        let entry_count = 1u16;
        bytes[8..10].copy_from_slice(&entry_count.to_le_bytes());
        bytes[10..12].copy_from_slice(&0x0112u16.to_le_bytes()); // Tag
        bytes[12..14].copy_from_slice(&3u16.to_le_bytes());      // SHORT type
        bytes[14..18].copy_from_slice(&1u32.to_le_bytes());      // Count
        bytes[18..22].copy_from_slice(&6u32.to_le_bytes());      // Value = 6

        let meta = extract_metadata_from_bytes(&bytes);
        assert_eq!(meta.orientation, Some(6));

        let orient = decode_exif_orientation(meta.orientation.unwrap());
        assert_eq!(orient.rotation, 90);
        assert!(!orient.flip_horizontal);
        assert!(!orient.flip_vertical);
    }

    #[test]
    fn test_as_shot_temperature_and_dual_illuminant_matrix() {
        use crate::raw::{RawSensorFrame, CfaPattern};

        // Canon EOS 60D color matrices
        let cm_tungsten: [[f32; 3]; 3] = [
            [0.6722, -0.0635, -0.0963],
            [-0.4285, 1.2448, 0.2117],
            [-0.0908, 0.2162, 0.5668],
        ];
        let cm_daylight: [[f32; 3]; 3] = [
            [0.6947, -0.1168, -0.0779],
            [-0.4127, 1.2209, 0.2138],
            [-0.0710, 0.1799, 0.6303],
        ];

        // Frame with Tungsten as-shot WB (wb_r = 1.2031, wb_b = 3.1699)
        let frame_tungsten = RawSensorFrame {
            full_width: 10,
            full_height: 10,
            active_x: 0,
            active_y: 0,
            active_width: 10,
            active_height: 10,
            cfa_pattern: CfaPattern::Rggb,
            black_levels: [2046.0; 4],
            white_levels: [15000.0; 4],
            wb_coeffs: [1.2031, 1.0, 3.1699, 1.0],
            color_matrix: Some(cm_daylight),
            color_matrix_tungsten: Some(cm_tungsten),
            forward_matrix_tungsten: None,
            forward_matrix_daylight: None,
            camera_model: "Canon EOS 60D".to_string(),
            baseline_exposure: 0.0,
            noise_profile: None,
            sensor_data: vec![2046; 100],
        };

        let temp_tungsten = frame_tungsten.as_shot_temperature();
        assert!(
            (2400.0..=3200.0).contains(&temp_tungsten),
            "Expected tungsten Kelvin around 2856K, got {}",
            temp_tungsten
        );

        // Frame with Daylight as-shot WB (wb_r = 2.0, wb_b = 1.5)
        let frame_daylight = RawSensorFrame {
            full_width: 10,
            full_height: 10,
            active_x: 0,
            active_y: 0,
            active_width: 10,
            active_height: 10,
            cfa_pattern: CfaPattern::Rggb,
            black_levels: [2046.0; 4],
            white_levels: [15000.0; 4],
            wb_coeffs: [2.0, 1.0, 1.5, 1.0],
            color_matrix: Some(cm_daylight),
            color_matrix_tungsten: Some(cm_tungsten),
            forward_matrix_tungsten: None,
            forward_matrix_daylight: None,
            camera_model: "Canon EOS 60D".to_string(),
            baseline_exposure: 0.0,
            noise_profile: None,
            sensor_data: vec![2046; 100],
        };


        let temp_daylight = frame_daylight.as_shot_temperature();
        assert!(
            (5000.0..=7500.0).contains(&temp_daylight),
            "Expected daylight Kelvin around 5500-6500K, got {}",
            temp_daylight
        );

        // Verify neutral axis mapping in sensor_to_rec2020: [1, 1, 1] maps to ~[1, 1, 1]
        let m = frame_tungsten.sensor_to_rec2020_at(2856.0);
        let neutral_rec2020 = [
            m[0][0] + m[0][1] + m[0][2],
            m[1][0] + m[1][1] + m[1][2],
            m[2][0] + m[2][1] + m[2][2],
        ];
        assert!((neutral_rec2020[0] - 1.0).abs() < 0.05);
        assert!((neutral_rec2020[1] - 1.0).abs() < 0.05);
        assert!((neutral_rec2020[2] - 1.0).abs() < 0.05);
    }

    #[test]
    fn test_dng_baseline_exposure_extraction() {
        use crate::raw::read_dng_baseline_exposure;
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixture_path = manifest_dir.join("../firstproof-core/tests/fixtures/_MG_2651.dng");
        let test_path = std::env::var("FIRSTPROOF_TEST_RAW_DIR")
            .map(|d| std::path::Path::new(&d).join("_MG_2332.dng"))
            .unwrap_or(fixture_path);
        if test_path.exists() {
            let bytes = std::fs::read(&test_path).expect("Read DNG file");
            let exp = read_dng_baseline_exposure(&bytes).expect("BaselineExposure tag 50730 not found");
            assert!(exp.is_finite());

            let manifest = crate::container::RawContainerManifest::parse(&bytes).expect("Parse DNG manifest");
            assert!(manifest.is_dng);
            assert!(manifest.baseline_exposure.is_some());
        }
    }

    #[test]
    fn test_suppress_sensor_read_noise_outliers() {
        use crate::raw::{suppress_sensor_read_noise_outliers, CfaPattern};

        let w = 12;
        let h = 12;
        // Flat shadow patch with intensity 0.005
        let mut cfa = vec![0.005f32; w * h];

        // Inject isolated readout spike at (4, 4) [Red site on RGGB]
        cfa[4 * w + 4] = 0.050; // 10x noise spike

        // Inject isolated readout spike at (5, 5) [Blue site on RGGB]
        cfa[5 * w + 5] = 0.050; // 10x noise spike

        // Inject isolated readout spike at (4, 5) [Green site on RGGB]
        cfa[4 * w + 5] = 0.050; // 10x noise spike

        // Inject a real optical step edge along column 8 and 9 (all channels high)
        for y in 0..h {
            cfa[y * w + 8] = 0.050;
            cfa[y * w + 9] = 0.050;
        }

        // Noise profile: S = 0.0005, O = 1.5e-6 -> sigma(0.005) ~ 0.00199 -> 3.5*sigma ~ 0.007
        let noise_profile = Some([0.0005, 1.5e-6]);
        suppress_sensor_read_noise_outliers(&mut cfa, w, h, CfaPattern::Rggb, noise_profile);

        // 1. Isolated spikes must be clamped to neighborhood max + 3.5*sigma (< 0.015)
        assert!(cfa[4 * w + 4] < 0.015, "Isolated Red spike was not suppressed: got {}", cfa[4 * w + 4]);
        assert!(cfa[5 * w + 5] < 0.015, "Isolated Blue spike was not suppressed: got {}", cfa[5 * w + 5]);
        assert!(cfa[4 * w + 5] < 0.015, "Isolated Green spike was not suppressed: got {}", cfa[4 * w + 5]);

        // 2. Optical step edge at column 8 and 9 must be 100% preserved (not clamped)
        for y in 2..(h - 2) {
            assert!(
                (cfa[y * w + 8] - 0.050).abs() < 1e-4,
                "Optical edge at column 8 was softened at y={}: got {}",
                y, cfa[y * w + 8]
            );
            assert!(
                (cfa[y * w + 9] - 0.050).abs() < 1e-4,
                "Optical edge at column 9 was softened at y={}: got {}",
                y, cfa[y * w + 9]
            );
        }
    }
}

