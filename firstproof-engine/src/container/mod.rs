// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Unified RAW and DNG container intake module.
//!
//! Replaces fragmented, ad-hoc binary IFD scanners with a strongly-typed,
//! zero-allocation-friendly `RawContainerManifest` parser.

pub mod manifest;
pub mod opcodes;
pub mod tags;
pub mod tiff;

pub use manifest::RawContainerManifest;
pub use opcodes::{
    apply_opcode_list_1_to_raw, apply_opcode_list_2_to_rgb, is_bayer_green, DngAreaSpec,
    DngBadPoint, DngBadRect, DngGainMap, DngOpcode, DngOpcodeList, OpcodeError,
    OPCODE_FIX_BAD_PIXELS_CONSTANT, OPCODE_FIX_BAD_PIXELS_LIST, OPCODE_GAIN_MAP,
    OPCODE_WARP_RECTILINEAR,
};
pub use tags::*;
pub use tiff::{ParsedIfd, TiffContainer};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthetic_dng_manifest_parsing() {
        // Construct a synthetic 80-byte Little-Endian TIFF with IFD0 containing:
        // - Tag 50708 (UniqueCameraModel): ASCII "Canon EOS 60D"
        // - Tag 50730 (BaselineExposure): SRATIONAL 831/1000 (+0.831 EV)
        // - Tag 50734 (LinearResponseLimit): RATIONAL 96/100 (0.96)
        // - Tag 50714 (BlackLevel): SHORT count 4: [2045, 2045, 2050, 2050]
        let mut buf = vec![0u8; 256];

        // TIFF Header: II (LE), Magic 42, IFD0 Offset = 8
        buf[0..2].copy_from_slice(b"II");
        buf[2..4].copy_from_slice(&42u16.to_le_bytes());
        buf[4..8].copy_from_slice(&8u32.to_le_bytes());

        // IFD0 at offset 8:
        // Number of entries: 4
        let num_entries = 4u16;
        buf[8..10].copy_from_slice(&num_entries.to_le_bytes());

        let mut curr = 10;

        // Entry 1: Tag 50708 (UniqueCameraModel), ASCII, count = 14, offset = 100
        buf[curr..curr+2].copy_from_slice(&tags::TAG_UNIQUE_CAMERA_MODEL.to_le_bytes());
        buf[curr+2..curr+4].copy_from_slice(&tiff::TYPE_ASCII.to_le_bytes());
        buf[curr+4..curr+8].copy_from_slice(&14u32.to_le_bytes());
        buf[curr+8..curr+12].copy_from_slice(&100u32.to_le_bytes());
        buf[100..114].copy_from_slice(b"Canon EOS 60D\0");
        curr += 12;

        // Entry 2: Tag 50730 (BaselineExposure), SRATIONAL, count = 1, offset = 120
        buf[curr..curr+2].copy_from_slice(&tags::DNG_TAG_BASELINE_EXPOSURE.to_le_bytes());
        buf[curr+2..curr+4].copy_from_slice(&tiff::TYPE_SRATIONAL.to_le_bytes());
        buf[curr+4..curr+8].copy_from_slice(&1u32.to_le_bytes());
        buf[curr+8..curr+12].copy_from_slice(&120u32.to_le_bytes());
        buf[120..124].copy_from_slice(&831i32.to_le_bytes());
        buf[124..128].copy_from_slice(&1000i32.to_le_bytes());
        curr += 12;

        // Entry 3: Tag 50734 (LinearResponseLimit), RATIONAL, count = 1, offset = 130
        buf[curr..curr+2].copy_from_slice(&tags::DNG_TAG_LINEAR_RESPONSE_LIMIT.to_le_bytes());
        buf[curr+2..curr+4].copy_from_slice(&tiff::TYPE_RATIONAL.to_le_bytes());
        buf[curr+4..curr+8].copy_from_slice(&1u32.to_le_bytes());
        buf[curr+8..curr+12].copy_from_slice(&130u32.to_le_bytes());
        buf[130..134].copy_from_slice(&96u32.to_le_bytes());
        buf[134..138].copy_from_slice(&100u32.to_le_bytes());
        curr += 12;

        // Entry 4: Tag 50714 (BlackLevel), SHORT, count = 4, offset = 140
        buf[curr..curr+2].copy_from_slice(&tags::TAG_BLACK_LEVEL.to_le_bytes());
        buf[curr+2..curr+4].copy_from_slice(&tiff::TYPE_SHORT.to_le_bytes());
        buf[curr+4..curr+8].copy_from_slice(&4u32.to_le_bytes());
        buf[curr+8..curr+12].copy_from_slice(&140u32.to_le_bytes());
        buf[140..142].copy_from_slice(&2045u16.to_le_bytes());
        buf[142..144].copy_from_slice(&2045u16.to_le_bytes());
        buf[144..146].copy_from_slice(&2050u16.to_le_bytes());
        buf[146..148].copy_from_slice(&2050u16.to_le_bytes());
        curr += 12;

        // Next IFD offset = 0
        buf[curr..curr+4].copy_from_slice(&0u32.to_le_bytes());

        let manifest = RawContainerManifest::parse(&buf).expect("Failed to parse synthetic DNG");
        assert!(manifest.is_dng);
        assert_eq!(manifest.unique_camera_model.as_deref(), Some("Canon EOS 60D"));
        assert!((manifest.baseline_exposure.unwrap() - 0.831).abs() < 1e-4);
        assert!((manifest.linear_response_limit.unwrap() - 0.96).abs() < 1e-4);
        assert_eq!(manifest.black_levels, Some([2045.0, 2045.0, 2050.0, 2050.0]));
    }
}
