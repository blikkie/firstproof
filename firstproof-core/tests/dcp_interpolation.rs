use firstproof_core::color::dcp::{
    DcpProfile, DcpError, illuminant_to_temperature,
    ILLUMINANT_STANDARD_A, ILLUMINANT_D65,
    TAG_COLOR_MATRIX_1, TAG_COLOR_MATRIX_2,
    TAG_CALIBRATION_ILLUMINANT_1, TAG_CALIBRATION_ILLUMINANT_2,
    TAG_FORWARD_MATRIX_1, TAG_FORWARD_MATRIX_2,
    TAG_PROFILE_NAME,
};
use firstproof_core::color::{Color3, Matrix3x3};

/// Helper to serialize a 3x3 matrix into TIFF SRATIONAL bytes (9 pairs of i32 num/den).
fn matrix_to_srational_bytes(m: &Matrix3x3, scale: i32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(9 * 8);
    for row in 0..3 {
        for col in 0..3 {
            let val = m.0[row][col];
            let num = (val * scale as f32).round() as i32;
            let den = scale;
            bytes.extend_from_slice(&num.to_le_bytes());
            bytes.extend_from_slice(&den.to_le_bytes());
        }
    }
    bytes
}

/// Constructs a valid minimal little-endian TIFF DCP file buffer in memory.
fn build_synthetic_dcp(
    name: &str,
    ill1: u16,
    ill2: Option<u16>,
    cm1: &Matrix3x3,
    cm2: Option<&Matrix3x3>,
    fm1: Option<&Matrix3x3>,
    fm2: Option<&Matrix3x3>,
) -> Vec<u8> {
    let mut buffer = Vec::new();

    // 1. TIFF Header (8 bytes)
    buffer.extend_from_slice(b"II"); // Little endian
    buffer.extend_from_slice(&42u16.to_le_bytes()); // Magic 42
    let ifd_offset: u32 = 8;
    buffer.extend_from_slice(&ifd_offset.to_le_bytes());

    // Prepare tags
    struct TagEntry {
        tag: u16,
        field_type: u16,
        count: u32,
        val_or_data: Result<u32, Vec<u8>>,
    }

    let mut entries = Vec::new();

    // Profile Name (ASCII, type 2)
    let mut name_bytes = name.as_bytes().to_vec();
    name_bytes.push(0); // null terminate
    entries.push(TagEntry {
        tag: TAG_PROFILE_NAME,
        field_type: 2,
        count: name_bytes.len() as u32,
        val_or_data: Err(name_bytes),
    });

    // Illuminant 1 (SHORT, type 3, count 1)
    entries.push(TagEntry {
        tag: TAG_CALIBRATION_ILLUMINANT_1,
        field_type: 3,
        count: 1,
        val_or_data: Ok(ill1 as u32),
    });

    // Illuminant 2
    if let Some(i2) = ill2 {
        entries.push(TagEntry {
            tag: TAG_CALIBRATION_ILLUMINANT_2,
            field_type: 3,
            count: 1,
            val_or_data: Ok(i2 as u32),
        });
    }

    // Color Matrix 1 (SRATIONAL, type 10, count 9)
    entries.push(TagEntry {
        tag: TAG_COLOR_MATRIX_1,
        field_type: 10,
        count: 9,
        val_or_data: Err(matrix_to_srational_bytes(cm1, 10000)),
    });

    // Color Matrix 2
    if let Some(m2) = cm2 {
        entries.push(TagEntry {
            tag: TAG_COLOR_MATRIX_2,
            field_type: 10,
            count: 9,
            val_or_data: Err(matrix_to_srational_bytes(m2, 10000)),
        });
    }

    // Forward Matrix 1
    if let Some(m1) = fm1 {
        entries.push(TagEntry {
            tag: TAG_FORWARD_MATRIX_1,
            field_type: 10,
            count: 9,
            val_or_data: Err(matrix_to_srational_bytes(m1, 10000)),
        });
    }

    // Forward Matrix 2
    if let Some(m2) = fm2 {
        entries.push(TagEntry {
            tag: TAG_FORWARD_MATRIX_2,
            field_type: 10,
            count: 9,
            val_or_data: Err(matrix_to_srational_bytes(m2, 10000)),
        });
    }

    // Sort entries by tag ID (TIFF requirement)
    entries.sort_by_key(|e| e.tag);

    let num_entries = entries.len() as u16;
    buffer.extend_from_slice(&num_entries.to_le_bytes());

    // IFD takes: 2 + num_entries * 12 + 4 bytes
    let ifd_size = 2 + (entries.len() * 12) + 4;
    let mut payload_offset = ifd_offset as usize + ifd_size;

    let mut payloads = Vec::new();

    for entry in &entries {
        buffer.extend_from_slice(&entry.tag.to_le_bytes());
        buffer.extend_from_slice(&entry.field_type.to_le_bytes());
        buffer.extend_from_slice(&entry.count.to_le_bytes());
        match &entry.val_or_data {
            Ok(val) => {
                buffer.extend_from_slice(&val.to_le_bytes());
            }
            Err(data) => {
                buffer.extend_from_slice(&(payload_offset as u32).to_le_bytes());
                payload_offset += data.len();
                payloads.push(data.clone());
            }
        }
    }

    // Next IFD offset = 0
    buffer.extend_from_slice(&0u32.to_le_bytes());

    // Append payloads
    for p in payloads {
        buffer.extend_from_slice(&p);
    }

    buffer
}

#[test]
fn test_synthetic_dcp_parsing() {
    let cm1 = Matrix3x3([
        [0.80, -0.10, -0.05],
        [-0.40, 1.20, 0.20],
        [-0.08, 0.22, 0.73],
    ]);
    let cm2 = Matrix3x3([
        [0.70, 0.06, 0.20],
        [0.27, 0.64, 0.09],
        [-0.01, -0.08, 0.92],
    ]);
    let fm1 = Matrix3x3([
        [0.65, 0.15, 0.15],
        [0.26, 0.72, 0.02],
        [0.02, -0.11, 0.92],
    ]);
    let fm2 = Matrix3x3([
        [0.61, 0.23, 0.12],
        [0.28, 0.69, 0.03],
        [0.02, -0.06, 0.87],
    ]);

    let dcp_bytes = build_synthetic_dcp(
        "First Proof Test Profile",
        ILLUMINANT_STANDARD_A,
        Some(ILLUMINANT_D65),
        &cm1,
        Some(&cm2),
        Some(&fm1),
        Some(&fm2),
    );

    let profile = DcpProfile::from_bytes(&dcp_bytes).expect("Failed to parse synthetic DCP");

    assert_eq!(profile.profile_name, "First Proof Test Profile");
    assert_eq!(profile.illuminant1, ILLUMINANT_STANDARD_A);
    assert_eq!(profile.illuminant2, Some(ILLUMINANT_D65));

    // Assert ColorMatrix1 parsed within rational precision
    for r in 0..3 {
        for c in 0..3 {
            assert!(
                (profile.color_matrix1.0[r][c] - cm1.0[r][c]).abs() < 1e-3,
                "ColorMatrix1 mismatch at ({},{})", r, c
            );
        }
    }

    // Assert ForwardMatrix1
    let parsed_fm1 = profile.forward_matrix1.expect("Missing ForwardMatrix1");
    for r in 0..3 {
        for c in 0..3 {
            assert!(
                (parsed_fm1.0[r][c] - fm1.0[r][c]).abs() < 1e-3,
                "ForwardMatrix1 mismatch at ({},{})", r, c
            );
        }
    }
}

#[test]
fn test_planckian_interpolation_endpoints_and_midpoint() {
    let profile = DcpProfile::built_in_canon_eos();
    let fm1 = profile.forward_matrix1.unwrap();
    let fm2 = profile.forward_matrix2.unwrap();

    let t1 = illuminant_to_temperature(profile.illuminant1); // 2856.0
    let t2 = illuminant_to_temperature(profile.illuminant2.unwrap()); // 6504.0

    // 1. Below T1: should equal FM1 exactly (clamped weight 0.0)
    let interp_low = profile.interpolate_forward_matrix(2000.0).unwrap();
    for r in 0..3 {
        for c in 0..3 {
            assert!(
                (interp_low.0[r][c] - fm1.0[r][c]).abs() < 1e-6,
                "Lower clamp mismatch at ({},{})", r, c
            );
        }
    }

    // 2. Above T2: should equal FM2 exactly (clamped weight 1.0)
    let interp_high = profile.interpolate_forward_matrix(8000.0).unwrap();
    for r in 0..3 {
        for c in 0..3 {
            assert!(
                (interp_high.0[r][c] - fm2.0[r][c]).abs() < 1e-6,
                "Upper clamp mismatch at ({},{})", r, c
            );
        }
    }

    // 3. Exact T1 and T2
    let interp_t1 = profile.interpolate_forward_matrix(t1).unwrap();
    let interp_t2 = profile.interpolate_forward_matrix(t2).unwrap();
    for r in 0..3 {
        for c in 0..3 {
            assert!((interp_t1.0[r][c] - fm1.0[r][c]).abs() < 1e-6);
            assert!((interp_t2.0[r][c] - fm2.0[r][c]).abs() < 1e-6);
        }
    }

    // 4. Midpoint at 4000K: reciprocal temperature Planckian weight
    let t_mid = 4000.0;
    let m1 = 1.0 / t1;
    let m2 = 1.0 / t2;
    let m_mid = 1.0 / t_mid;
    let expected_w = (m_mid - m1) / (m2 - m1);

    let interp_mid = profile.interpolate_forward_matrix(t_mid).unwrap();
    for r in 0..3 {
        for c in 0..3 {
            let expected_val = (1.0 - expected_w) * fm1.0[r][c] + expected_w * fm2.0[r][c];
            assert!(
                (interp_mid.0[r][c] - expected_val).abs() < 1e-5,
                "Midpoint interpolation deviation at ({},{}): got {}, expected {}",
                r, c, interp_mid.0[r][c], expected_val
            );
        }
    }
}

#[test]
fn test_sensor_to_rec2020_neutral_preservation() {
    let profiles = [
        DcpProfile::built_in_canon_eos(),
        DcpProfile::built_in_sony_alpha(),
        DcpProfile::built_in_nikon_z(),
    ];

    let test_temperatures = [2856.0, 4500.0, 5500.0, 6504.0];

    for profile in &profiles {
        for &temp in &test_temperatures {
            let mat = profile.interpolate_sensor_to_rec2020(temp);
            let neutral_in = Color3::new(1.0, 1.0, 1.0);
            let neutral_out = mat.multiply_vector(neutral_in);

            // In white-balanced sensor space, (1, 1, 1) represents neutral gray.
            // Rec.2020 output must preserve equal energy across R, G, B channels (no chromatic tint).
            let diff_rg = (neutral_out.r() - neutral_out.g()).abs();
            let diff_gb = (neutral_out.g() - neutral_out.b()).abs();

            assert!(
                diff_rg < 0.05,
                "Profile '{}' at {}K introduced R-G tint: R={}, G={}",
                profile.profile_name, temp, neutral_out.r(), neutral_out.g()
            );
            assert!(
                diff_gb < 0.05,
                "Profile '{}' at {}K introduced G-B tint: G={}, B={}",
                profile.profile_name, temp, neutral_out.g(), neutral_out.b()
            );
        }
    }
}

#[test]
fn test_built_in_camera_lookups() {
    let canon = DcpProfile::built_in_for_camera("Canon EOS R5");
    assert!(canon.profile_name.contains("Canon"));

    let sony = DcpProfile::built_in_for_camera("SONY ILCE-7M4");
    assert!(sony.profile_name.contains("Sony"));

    let nikon = DcpProfile::built_in_for_camera("NIKON Z 6_2");
    assert!(nikon.profile_name.contains("Nikon"));

    let fallback = DcpProfile::built_in_for_camera("Unknown Smartphone Camera");
    assert!(fallback.profile_name.contains("Canon")); // Default baseline
}

#[test]
fn test_malformed_dcp_error_handling() {
    // Empty data
    assert_eq!(DcpProfile::from_bytes(&[]), Err(DcpError::UnexpectedEof));

    // Invalid header magic
    let bad_magic = vec![b'I', b'I', 0, 0, 8, 0, 0, 0];
    assert_eq!(DcpProfile::from_bytes(&bad_magic), Err(DcpError::InvalidHeader));

    // Unsupported endianness
    let bad_endian = vec![b'X', b'X', 42, 0, 8, 0, 0, 0];
    assert_eq!(DcpProfile::from_bytes(&bad_endian), Err(DcpError::UnsupportedByteOrder));
}
