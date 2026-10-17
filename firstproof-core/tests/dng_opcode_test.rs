// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Comprehensive Unit and Integration Tests for Story 3.14:
//! DNG Opcode Processing Engine: Lens Flat-Field GainMap & Factory Bad-Pixel Repair.

use firstproof_engine::container::opcodes::*;
use firstproof_engine::container::{
    RawContainerManifest, DNG_TAG_OPCODE_LIST_1, TAG_UNIQUE_CAMERA_MODEL,
};

#[test]
fn test_opcode_list_bytecode_parser_and_spool_roundtrip() {
    let mut list = DngOpcodeList::new();

    // 1. FixBadPixelsConstant
    list.opcodes.push(DngOpcode::FixBadPixelsConstant {
        min_version: DNG_VERSION_1_3_0_0,
        flags: FLAG_NONE,
        constant: 0,
        bayer_phase: 0, // GRBG
    });

    // 2. FixBadPixelsList
    list.opcodes.push(DngOpcode::FixBadPixelsList {
        min_version: DNG_VERSION_1_3_0_0,
        flags: FLAG_NONE,
        bayer_phase: 1, // RGGB
        bad_points: vec![
            DngBadPoint { row: 12, col: 34 },
            DngBadPoint { row: 56, col: 78 },
        ],
        bad_rects: vec![DngBadRect {
            top: 0,
            left: 42,
            bottom: 100,
            right: 43, // Single dead column at col 42
        }],
    });

    // 3. GainMap
    let area = DngAreaSpec {
        top: 0,
        left: 0,
        bottom: 200,
        right: 300,
        plane: 0,
        planes: 1,
        row_pitch: 1,
        col_pitch: 1,
    };
    let map = DngGainMap {
        points_v: 2,
        points_h: 2,
        spacing_v: 1.0,
        spacing_h: 1.0,
        origin_v: 0.0,
        origin_h: 0.0,
        planes: 1,
        map_gain: vec![1.25, 1.50, 1.10, 1.35],
    };
    list.opcodes.push(DngOpcode::GainMap {
        min_version: DNG_VERSION_1_3_0_0,
        flags: FLAG_NONE,
        area,
        map,
    });

    // 4. WarpRectilinear
    list.opcodes.push(DngOpcode::WarpRectilinear {
        min_version: DNG_VERSION_1_3_0_0,
        flags: FLAG_OPTIONAL,
        planes: 1,
        coefficients: [0.001, -0.0002, 0.00003, 0.0],
        center: [0.5, 0.5],
    });

    // Serialize to standard big-endian DNG bytecode
    let bytecode = list.serialize();

    // Parse back
    let parsed = DngOpcodeList::parse(&bytecode).expect("Failed to parse valid opcode list bytecode");

    assert_eq!(parsed.len(), 4);
    assert_eq!(parsed.opcodes[0], list.opcodes[0]);
    assert_eq!(parsed.opcodes[1], list.opcodes[1]);
    assert_eq!(parsed.opcodes[2], list.opcodes[2]);
    assert_eq!(parsed.opcodes[3], list.opcodes[3]);
}

#[test]
fn test_bad_pixel_repair_constant_photosites_preserves_healthy_pixels() {
    let width = 8usize;
    let height = 8usize;
    // RGGB Bayer pattern:
    // row 0: R G R G R G R G
    // row 1: G B G B G B G B
    // row 2: R G R G R G R G
    // row 3: G B G B G B G B
    // ...
    let mut raw = vec![2000u16; width * height];

    // Inject 3 distinct dead photosites (value = 0)
    // 1. Green photosite at (2, 3)
    let idx_g = 2 * width + 3;
    raw[idx_g] = 0;

    // 2. Red photosite at (2, 2)
    let idx_r = 2 * width + 2;
    raw[idx_r] = 0;

    // 3. Blue photosite at (3, 3)
    let idx_b = 3 * width + 3;
    raw[idx_b] = 0;

    let mut list = DngOpcodeList::new();
    list.opcodes.push(DngOpcode::FixBadPixelsConstant {
        min_version: DNG_VERSION_1_3_0_0,
        flags: FLAG_NONE,
        constant: 0,
        bayer_phase: 1, // RGGB
    });

    apply_opcode_list_1_to_raw(&mut raw, width, height, [0.0; 4], [65535.0; 4], &list);

    // Defective pixels must be in-painted to the surrounding neighbor value (2000)
    assert_eq!(raw[idx_g], 2000, "Green dead pixel at (2, 3) must be repaired to 2000");
    assert_eq!(raw[idx_r], 2000, "Red dead pixel at (2, 2) must be repaired to 2000");
    assert_eq!(raw[idx_b], 2000, "Blue dead pixel at (3, 3) must be repaired to 2000");

    // All other healthy pixels must remain strictly 2000
    for (i, &val) in raw.iter().enumerate() {
        assert_eq!(val, 2000, "Healthy pixel at index {i} must remain unaltered");
    }
}

#[test]
fn test_bad_pixel_repair_defective_list_and_columns() {
    let width = 10usize;
    let height = 10usize;
    let mut raw = vec![3000u16; width * height];

    // Simulate a dead factory sensor column at column 4
    for r in 0..height {
        raw[r * width + 4] = 0;
    }

    // Also a discrete hot pixel at (7, 8)
    raw[7 * width + 8] = 65535;

    let mut list = DngOpcodeList::new();
    list.opcodes.push(DngOpcode::FixBadPixelsList {
        min_version: DNG_VERSION_1_3_0_0,
        flags: FLAG_NONE,
        bayer_phase: 1,
        bad_points: vec![DngBadPoint { row: 7, col: 8 }],
        bad_rects: vec![DngBadRect {
            top: 0,
            left: 4,
            bottom: height as i32,
            right: 5,
        }],
    });

    apply_opcode_list_1_to_raw(&mut raw, width, height, [0.0; 4], [65535.0; 4], &list);

    // Discrete hot pixel must be repaired
    assert_eq!(raw[7 * width + 8], 3000, "Discrete bad point must be repaired to 3000");

    // Every pixel in the dead column must be restored from adjacent same-channel columns (col 2 and col 6)
    for r in 0..height {
        assert_eq!(raw[r * width + 4], 3000, "Dead column pixel at ({r}, 4) must be repaired to 3000");
    }
}

#[test]
fn test_gain_map_radial_vignetting_preserves_black_level() {
    let width = 50usize;
    let height = 50usize;

    // Black level = 512, White level = 16383 (typical 14-bit RAW)
    let black = 512.0f32;
    let white = 16383.0f32;

    // Construct a frame:
    // Pixel 1: Pure black frame photosite = 512
    // Pixel 2: Optical corner photosite with signal 1000 above black = 1512
    let mut raw = vec![1512u16; width * height];
    // Put black pixel in corner (0, 0)
    raw[0] = 512;

    // GainMap with 2.0x gain at corner (0, 0) and 1.0x at center (25, 25)
    let area = DngAreaSpec {
        top: 0,
        left: 0,
        bottom: height as i32,
        right: width as i32,
        plane: 0,
        planes: 1,
        row_pitch: 1,
        col_pitch: 1,
    };
    let map = DngGainMap {
        points_v: 2,
        points_h: 2,
        spacing_v: 1.0,
        spacing_h: 1.0,
        origin_v: 0.0,
        origin_h: 0.0,
        planes: 1,
        map_gain: vec![
            2.0, 1.0,
            1.0, 1.0,
        ],
    };

    let mut list = DngOpcodeList::new();
    list.opcodes.push(DngOpcode::GainMap {
        min_version: DNG_VERSION_1_3_0_0,
        flags: FLAG_NONE,
        area,
        map,
    });

    apply_opcode_list_1_to_raw(&mut raw, width, height, [black; 4], [white; 4], &list);

    // 1. Sensitometric Invariant: True black level must NOT be scaled!
    // (512 - 512) * 2.0 + 512 = 512.
    assert_eq!(
        raw[0], 512,
        "Black level photosite must remain at 512 despite 2.0x gain map multiplier"
    );

    // 2. Corner with signal above black:
    // (1512 - 512) * ~2.0 + 512 = 1000 * 2.0 + 512 = 2512.
    let corner_signal = raw[1]; // adjacent to corner (0,0)
    assert!(
        corner_signal > 2400 && corner_signal <= 2512,
        "Signal above black level must be boosted by corner vignetting gain: got {corner_signal}"
    );
}

#[test]
fn test_gain_map_opcode_list_2_rgb_pipeline() {
    let width = 10usize;
    let height = 10usize;

    // Interleaved RGB float buffer (10 x 10 x 3)
    let mut rgb_data = vec![1.0f32; width * height * 3];

    // GainMap with red gain 1.2, green gain 1.0, blue gain 0.8
    let area = DngAreaSpec {
        top: 0,
        left: 0,
        bottom: height as i32,
        right: width as i32,
        plane: 0,
        planes: 3,
        row_pitch: 1,
        col_pitch: 1,
    };
    let map = DngGainMap {
        points_v: 1,
        points_h: 1,
        spacing_v: 1.0,
        spacing_h: 1.0,
        origin_v: 0.0,
        origin_h: 0.0,
        planes: 3,
        map_gain: vec![1.2, 1.0, 0.8],
    };

    let mut list = DngOpcodeList::new();
    list.opcodes.push(DngOpcode::GainMap {
        min_version: DNG_VERSION_1_3_0_0,
        flags: FLAG_NONE,
        area,
        map,
    });

    apply_opcode_list_2_to_rgb(&mut rgb_data, width, height, &list);

    for i in 0..(width * height) {
        let r = rgb_data[i * 3];
        let g = rgb_data[i * 3 + 1];
        let b = rgb_data[i * 3 + 2];

        assert!((r - 1.2).abs() < 1e-5, "Expected R=1.2, got {r}");
        assert!((g - 1.0).abs() < 1e-5, "Expected G=1.0, got {g}");
        assert!((b - 0.8).abs() < 1e-5, "Expected B=0.8, got {b}");
    }
}

#[test]
fn test_malformed_opcode_stream_safety() {
    // 1. Truncated count header
    assert!(matches!(
        DngOpcodeList::parse(&[0, 0]),
        Err(OpcodeError::UnexpectedEof)
    ));

    // 2. Count = 1, but no opcode header bytes
    assert!(matches!(
        DngOpcodeList::parse(&[0, 0, 0, 1]),
        Err(OpcodeError::UnexpectedEof)
    ));

    // 3. Opcode header claims 100 bytes of data, but only 4 bytes provided
    let mut truncated = Vec::new();
    truncated.extend_from_slice(&1u32.to_be_bytes()); // Count = 1
    truncated.extend_from_slice(&OPCODE_GAIN_MAP.to_be_bytes());
    truncated.extend_from_slice(&DNG_VERSION_1_3_0_0.to_be_bytes());
    truncated.extend_from_slice(&FLAG_NONE.to_be_bytes());
    truncated.extend_from_slice(&100u32.to_be_bytes()); // Byte count = 100
    truncated.extend_from_slice(&[0u8; 4]); // Only 4 bytes

    assert!(matches!(
        DngOpcodeList::parse(&truncated),
        Err(OpcodeError::InvalidDataLength { .. })
    ));

    // 4. Unknown opcode with FLAG_OPTIONAL should succeed and parse as Unknown
    let mut optional_unknown = Vec::new();
    optional_unknown.extend_from_slice(&1u32.to_be_bytes());
    optional_unknown.extend_from_slice(&9999u32.to_be_bytes()); // Unknown ID 9999
    optional_unknown.extend_from_slice(&DNG_VERSION_1_3_0_0.to_be_bytes());
    optional_unknown.extend_from_slice(&FLAG_OPTIONAL.to_be_bytes());
    optional_unknown.extend_from_slice(&4u32.to_be_bytes());
    optional_unknown.extend_from_slice(&[1, 2, 3, 4]);

    let parsed = DngOpcodeList::parse(&optional_unknown).expect("Optional unknown opcode must succeed");
    assert_eq!(parsed.len(), 1);
    assert!(matches!(parsed.opcodes[0], DngOpcode::Unknown { opcode_id: 9999, .. }));

    // 5. Unknown opcode without FLAG_OPTIONAL should return UnsupportedOpcode error
    let mut required_unknown = Vec::new();
    required_unknown.extend_from_slice(&1u32.to_be_bytes());
    required_unknown.extend_from_slice(&9999u32.to_be_bytes());
    required_unknown.extend_from_slice(&DNG_VERSION_1_3_0_0.to_be_bytes());
    required_unknown.extend_from_slice(&FLAG_NONE.to_be_bytes());
    required_unknown.extend_from_slice(&4u32.to_be_bytes());
    required_unknown.extend_from_slice(&[1, 2, 3, 4]);

    assert!(matches!(
        DngOpcodeList::parse(&required_unknown),
        Err(OpcodeError::UnsupportedOpcode { opcode_id: 9999 })
    ));
}

#[test]
fn test_dng_manifest_opcode_integration() {
    // Construct a synthetic Little-Endian TIFF container with IFD0 containing:
    // - Tag 50708 (UniqueCameraModel): ASCII "FirstProof Lab Camera"
    // - Tag 51008 (OpcodeList1): UNDEFINED bytes of serialized DngOpcodeList
    let mut op_list = DngOpcodeList::new();
    op_list.opcodes.push(DngOpcode::FixBadPixelsConstant {
        min_version: DNG_VERSION_1_3_0_0,
        flags: FLAG_NONE,
        constant: 0,
        bayer_phase: 1,
    });
    let op_bytes = op_list.serialize();

    let mut buf = vec![0u8; 512];
    // TIFF Header: II (LE), Magic 42, IFD0 Offset = 8
    buf[0..2].copy_from_slice(b"II");
    buf[2..4].copy_from_slice(&42u16.to_le_bytes());
    buf[4..8].copy_from_slice(&8u32.to_le_bytes());

    // IFD0 at offset 8: 2 entries
    buf[8..10].copy_from_slice(&2u16.to_le_bytes());

    // Entry 1: Tag 50708 (UniqueCameraModel)
    let model_str = b"FirstProof Lab Camera\0";
    let model_offset = 200u32;
    buf[model_offset as usize..model_offset as usize + model_str.len()].copy_from_slice(model_str);

    let mut curr = 10;
    buf[curr..curr + 2].copy_from_slice(&TAG_UNIQUE_CAMERA_MODEL.to_le_bytes());
    buf[curr + 2..curr + 4].copy_from_slice(&2u16.to_le_bytes()); // TYPE_ASCII
    buf[curr + 4..curr + 8].copy_from_slice(&(model_str.len() as u32).to_le_bytes());
    buf[curr + 8..curr + 12].copy_from_slice(&model_offset.to_le_bytes());
    curr += 12;

    // Entry 2: Tag 51008 (OpcodeList1)
    let op_offset = 300u32;
    buf[op_offset as usize..op_offset as usize + op_bytes.len()].copy_from_slice(&op_bytes);

    buf[curr..curr + 2].copy_from_slice(&DNG_TAG_OPCODE_LIST_1.to_le_bytes());
    buf[curr + 2..curr + 4].copy_from_slice(&7u16.to_le_bytes()); // TYPE_UNDEFINED
    buf[curr + 4..curr + 8].copy_from_slice(&(op_bytes.len() as u32).to_le_bytes());
    buf[curr + 8..curr + 12].copy_from_slice(&op_offset.to_le_bytes());

    // Parse manifest
    let manifest = RawContainerManifest::parse(&buf).expect("Failed to parse DNG manifest");
    assert!(manifest.is_dng);
    assert_eq!(manifest.unique_camera_model.as_deref(), Some("FirstProof Lab Camera"));

    let extracted_ops = manifest.opcode_list_1.expect("OpcodeList1 must be extracted");
    assert_eq!(extracted_ops, op_bytes);

    let parsed_ops = DngOpcodeList::parse(&extracted_ops).expect("Extracted bytes must parse as DngOpcodeList");
    assert_eq!(parsed_ops, op_list);
}

#[test]
fn test_pixel_computational_dng_optical_fidelity() {
    let path_str = match std::env::var("FIRSTPROOF_PIXEL_SAMPLE") {
        Ok(p) => p,
        Err(_) => return,
    };
    let path = std::path::Path::new(&path_str);
    if !path.exists() {
        return;
    }
    let bytes = std::fs::read(path).unwrap();
    let manifest = RawContainerManifest::parse(&bytes).expect("Failed to parse Pixel DNG manifest");
    assert!(manifest.is_dng);
    let exp = manifest.baseline_exposure.expect("BaselineExposure must be extracted");
    assert!((exp - 1.770).abs() < 0.01, "Expected baseline exposure ~1.77 EV, got {exp}");

    let noise = manifest.noise_profile.as_ref().expect("NoiseProfile must be extracted");
    assert_eq!(noise.len(), 6, "Expected 6 noise profile parameters (3 pairs of O, S)");
    assert!(noise[0] > 0.0 && noise[1] > 0.0, "Noise parameters must be positive");

    let ptc = manifest.profile_tone_curve.as_ref().expect("ProfileToneCurve must be extracted");
    assert_eq!(ptc.len(), 257, "Google Pixel DNG must contain 257 tone curve points");
    assert_eq!(firstproof_engine::color::evaluate_tone_curve(0.0, ptc), 0.0);
    assert_eq!(firstproof_engine::color::evaluate_tone_curve(1.0, ptc), 1.0);

    // Verify monotonicity of the embedded tone curve across 1000 sample points
    let mut prev = -1.0f32;
    for i in 0..=1000 {
        let x = i as f32 / 1000.0;
        let y = firstproof_engine::color::evaluate_tone_curve(x, ptc);
        assert!(y >= prev, "Non-monotonic step at x={}: prev={}, curr={}", x, prev, y);
        prev = y;
    }

    let op2_bytes = manifest.opcode_list_2.as_ref().expect("OpcodeList2 must be extracted");
    let parsed_op2 = DngOpcodeList::parse(op2_bytes).expect("OpcodeList2 must parse");
    assert_eq!(parsed_op2.opcodes.len(), 4, "Expected 4 Bayer channel GainMaps in OpcodeList2");
    for op in &parsed_op2.opcodes {
        match op {
            DngOpcode::GainMap { area, map, .. } => {
                assert_eq!(area.row_pitch, 2, "Bayer GainMap must have row_pitch 2");
                assert_eq!(area.col_pitch, 2, "Bayer GainMap must have col_pitch 2");
                assert_eq!(map.points_h, 40, "Expected 40 horizontal grid points");
                assert_eq!(map.points_v, 30, "Expected 30 vertical grid points");
            }
            _ => panic!("Expected GainMap opcode in OpcodeList2"),
        }
    }

    let op3_bytes = manifest.opcode_list_3.as_ref().expect("OpcodeList3 must be extracted");
    let parsed_op3 = DngOpcodeList::parse(op3_bytes).expect("OpcodeList3 must parse");
    let (warp_k, warp_c) = parsed_op3.find_warp_rectilinear().expect("WarpRectilinear opcode must be present");
    assert_eq!(warp_k.len(), 4, "Expected 4 radial distortion coefficients");
    assert!((warp_k[0] - 0.997).abs() < 0.01, "Expected k0 ~0.997, got {}", warp_k[0]);
    assert_eq!(warp_c, [0.0, 0.0]);

    // Test end-to-end decoding with RawDecoder to ensure frame populates all fields
    use firstproof_core::raw_decoder::{RawDecoder, RawlerDecoder};
    let frame = RawDecoder::decode(&RawlerDecoder, path).expect("RawDecoder must successfully decode Pixel DNG");
    assert_eq!(frame.camera_model, "Google Pixel 11 Pro");
    assert!((frame.baseline_exposure - 1.770).abs() < 0.01);
    assert!(frame.noise_profile.is_some());
    assert_eq!(frame.profile_tone_curve.as_ref().map(|c| c.len()), Some(257));
    assert!(frame.warp_rectilinear.is_some());
    assert!(!frame.sensor_data.is_empty());
}

