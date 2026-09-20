#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!("firstproof_types");

pub mod curve;
pub mod modes;
pub mod preview;
pub mod raw;
pub mod settings;
pub mod xmp;


pub use curve::{
    evaluate_all_curves_lut, evaluate_curve_lut, AllCurvesLutRecord, CurvePointRecord,
    ToneCurvesRecord,
};
pub use modes::{
    ColorProfileMode, EmulsionArchetype, EmulsionFormat, EnlargerFilterMode, ExportColorSpace,
    ExportRenderingIntent, PrintMediaMode,
};
pub use preview::{
    extract_metadata_from_bytes, extract_preview_slice, get_jpeg_dimensions,
    is_displayable_rgb_jpeg, ImageMetadataRecord, PreviewResultRecord, RawLinearResultRecord,
};
pub use raw::{
    f16_to_f32, f32_to_f16, CfaPattern, RawSensorFrame, CFA_COLOR_BLUE, CFA_COLOR_GREEN, CFA_COLOR_RED,
};
pub use settings::{EditSettings, WhiteBalanceResult};
pub use xmp::{
    format_gps_xmp, generate_xmp_sidecar, parse_curve_points, parse_gps_xmp, parse_xmp_sidecar,
    serialize_curve_points, XmpData,
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
}
