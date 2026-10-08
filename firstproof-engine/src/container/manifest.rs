// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Strongly-typed manifest extracted from RAW/DNG containers and TIFF IFD hierarchies.

use super::tags::*;
use super::tiff::TiffContainer;

/// Strongly-typed manifest encapsulating all camera calibration, exposure,
/// sensor geometry, and noise metadata extracted from a RAW or DNG container.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RawContainerManifest {
    pub is_dng: bool,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub unique_camera_model: Option<String>,

    // Exposure & Sensitometric Limits
    pub baseline_exposure: Option<f32>,
    pub baseline_noise: Option<f32>,
    pub baseline_sharpness: Option<f32>,
    pub linear_response_limit: Option<f32>,

    // Dual-Illuminant Color Calibration
    pub color_matrix_1: Option<[[f32; 3]; 3]>,
    pub color_matrix_2: Option<[[f32; 3]; 3]>,
    pub forward_matrix_1: Option<[[f32; 3]; 3]>,
    pub forward_matrix_2: Option<[[f32; 3]; 3]>,
    pub calibration_illuminant_1: Option<u16>,
    pub calibration_illuminant_2: Option<u16>,

    // White Balance & Analog Gains
    pub analog_balance: Option<[f32; 3]>,
    pub as_shot_neutral: Option<Vec<f32>>,
    pub as_shot_white_xy: Option<[f32; 2]>,

    // Sensor Geometry & Active Areas
    pub active_area: Option<[u32; 4]>,
    pub default_crop_origin: Option<[u32; 2]>,
    pub default_crop_size: Option<[u32; 2]>,

    // Per-Channel Black & White Levels
    pub black_levels: Option<[f32; 4]>,
    pub white_levels: Option<[f32; 4]>,

    // Sensor Read/Shot Noise Profile
    pub noise_profile: Option<Vec<f64>>,

    // Embedded Preview Slice (offset, length)
    pub preview_slice: Option<(usize, usize)>,

    // Embedded Camera Profile Tone Curve (Tag 50940: pairs of (x, y))
    pub profile_tone_curve: Option<Vec<(f32, f32)>>,

    // DNG Opcode Lists (Stage 1, Stage 2, Stage 3)
    pub opcode_list_1: Option<Vec<u8>>,
    pub opcode_list_2: Option<Vec<u8>>,
    pub opcode_list_3: Option<Vec<u8>>,
}

impl RawContainerManifest {
    /// Parses a `RawContainerManifest` from raw bytes (first 64KB+ or full file).
    pub fn parse(buf: &[u8]) -> Option<Self> {
        let tiff = TiffContainer::parse(buf)?;
        let is_le = tiff.is_little_endian;

        let is_dng = tiff.find_entry(TAG_DNG_VERSION).is_some()
            || tiff.find_entry(TAG_UNIQUE_CAMERA_MODEL).is_some();

        let camera_make = tiff
            .find_entry(TAG_MAKE)
            .and_then(|e| e.as_string(buf));

        let camera_model = tiff
            .find_entry(TAG_MODEL)
            .and_then(|e| e.as_string(buf));

        let unique_camera_model = tiff
            .find_entry(TAG_UNIQUE_CAMERA_MODEL)
            .and_then(|e| e.as_string(buf));

        // BaselineExposure: Check Tag 50730 (Standard DNG) or Tag 50728 (Alt/legacy when count == 1)
        let baseline_exposure = tiff
            .find_entry(DNG_TAG_BASELINE_EXPOSURE)
            .and_then(|e| e.as_f32(buf, is_le))
            .or_else(|| {
                tiff.find_entry(DNG_TAG_BASELINE_EXPOSURE_ALT).and_then(|e| {
                    if e.count == 1 {
                        e.as_f32(buf, is_le)
                    } else {
                        None
                    }
                })
            });

        let baseline_noise = tiff
            .find_entry(DNG_TAG_BASELINE_NOISE)
            .and_then(|e| e.as_f32(buf, is_le));

        let baseline_sharpness = tiff
            .find_entry(DNG_TAG_BASELINE_SHARPNESS)
            .and_then(|e| e.as_f32(buf, is_le));

        let linear_response_limit = tiff
            .find_entry(DNG_TAG_LINEAR_RESPONSE_LIMIT)
            .and_then(|e| e.as_f32(buf, is_le));

        // Color and Forward Matrices
        let color_matrix_1 = tiff
            .find_entry(TAG_COLOR_MATRIX_1)
            .and_then(|e| e.as_matrix3x3(buf, is_le));

        let color_matrix_2 = tiff
            .find_entry(TAG_COLOR_MATRIX_2)
            .and_then(|e| e.as_matrix3x3(buf, is_le));

        let forward_matrix_1 = tiff
            .find_entry(DNG_TAG_FORWARD_MATRIX_1)
            .and_then(|e| e.as_matrix3x3(buf, is_le));

        let forward_matrix_2 = tiff
            .find_entry(DNG_TAG_FORWARD_MATRIX_2)
            .and_then(|e| e.as_matrix3x3(buf, is_le));

        let calibration_illuminant_1 = tiff
            .find_entry(DNG_TAG_CALIBRATION_ILLUMINANT_1)
            .and_then(|e| e.as_u32(buf, is_le))
            .map(|v| v as u16);

        let calibration_illuminant_2 = tiff
            .find_entry(DNG_TAG_CALIBRATION_ILLUMINANT_2)
            .and_then(|e| e.as_u32(buf, is_le))
            .map(|v| v as u16);

        // AnalogBalance
        let analog_balance = tiff
            .find_entry(TAG_ANALOG_BALANCE)
            .and_then(|e| e.as_f32_vec(buf, is_le))
            .and_then(|v| {
                if v.len() >= 3 {
                    Some([v[0], v[1], v[2]])
                } else {
                    None
                }
            });

        // AsShotNeutral (Tag 50728 with count >= 3)
        let as_shot_neutral = tiff
            .find_entry(DNG_TAG_AS_SHOT_NEUTRAL)
            .and_then(|e| {
                if e.count >= 3 {
                    e.as_f32_vec(buf, is_le)
                } else {
                    None
                }
            });

        let as_shot_white_xy = tiff
            .find_entry(DNG_TAG_AS_SHOT_WHITE_XY)
            .and_then(|e| e.as_f32_vec(buf, is_le))
            .and_then(|v| {
                if v.len() >= 2 {
                    Some([v[0], v[1]])
                } else {
                    None
                }
            });

        // ActiveArea (Tag 50829: [top, left, bottom, right])
        let active_area = tiff
            .find_entry(DNG_TAG_ACTIVE_AREA)
            .and_then(|e| e.as_u32_vec(buf, is_le))
            .and_then(|v| {
                if v.len() >= 4 {
                    Some([v[0], v[1], v[2], v[3]])
                } else {
                    None
                }
            });

        let default_crop_origin = tiff
            .find_entry(TAG_DEFAULT_CROP_ORIGIN)
            .and_then(|e| e.as_u32_vec(buf, is_le))
            .and_then(|v| {
                if v.len() >= 2 {
                    Some([v[0], v[1]])
                } else {
                    None
                }
            });

        let default_crop_size = tiff
            .find_entry(TAG_DEFAULT_CROP_SIZE)
            .and_then(|e| e.as_u32_vec(buf, is_le))
            .and_then(|v| {
                if v.len() >= 2 {
                    Some([v[0], v[1]])
                } else {
                    None
                }
            });

        // BlackLevels (Tag 50714)
        let black_levels = tiff
            .find_entry(TAG_BLACK_LEVEL)
            .and_then(|e| e.as_f32_vec(buf, is_le))
            .and_then(|v| {
                if v.len() >= 4 {
                    Some([v[0], v[1], v[2], v[3]])
                } else if !v.is_empty() {
                    Some([v[0], v[0], v[0], v[0]])
                } else {
                    None
                }
            });

        // WhiteLevels (Tag 50717)
        let white_levels = tiff
            .find_entry(TAG_WHITE_LEVEL)
            .and_then(|e| e.as_f32_vec(buf, is_le))
            .and_then(|v| {
                if v.len() >= 4 {
                    Some([v[0], v[1], v[2], v[3]])
                } else if !v.is_empty() {
                    Some([v[0], v[0], v[0], v[0]])
                } else {
                    None
                }
            });

        // NoiseProfile (Tag 51041)
        let noise_profile = tiff
            .find_entry(DNG_TAG_NOISE_PROFILE)
            .and_then(|e| {
                if let Some(slice) = e.data_slice(buf) {
                    let count = e.count as usize;
                    let mut out = Vec::with_capacity(count);
                    for i in 0..count {
                        let off = i * 8;
                        if off + 8 <= slice.len() {
                            let mut b = [0u8; 8];
                            b.copy_from_slice(&slice[off..off+8]);
                            out.push(if is_le { f64::from_le_bytes(b) } else { f64::from_be_bytes(b) });
                        }
                    }
                    Some(out)
                } else {
                    None
                }
            });

        // Embedded Preview Slice (JPEG in SubIFDs or IFD0)
        let preview_slice = tiff
            .find_entry(TAG_JPEG_INTERCHANGE_FORMAT)
            .and_then(|e_off| {
                let off = e_off.as_u32(buf, is_le)? as usize;
                let len = tiff
                    .find_entry(TAG_JPEG_INTERCHANGE_FORMAT_LENGTH)
                    .and_then(|e_len| e_len.as_u32(buf, is_le))
                    .unwrap_or(0) as usize;
                if off > 0 && off < buf.len() {
                    let max_len = if len > 0 { len } else { buf.len() - off };
                    Some((off, max_len))
                } else {
                    None
                }
            });

        // Embedded Camera Profile Tone Curve (Tag 50940)
        let profile_tone_curve = tiff
            .find_entry(DNG_TAG_PROFILE_TONE_CURVE)
            .and_then(|e| e.as_f32_vec(buf, is_le))
            .and_then(|v| {
                if v.len() >= 4 && v.len() % 2 == 0 {
                    let (chunks, _) = v.as_chunks::<2>();
                    Some(chunks.iter().map(|c| (c[0], c[1])).collect())
                } else {
                    None
                }
            });

        // DNG Opcode Lists (Tags 51008, 51009, 51022)
        let opcode_list_1 = tiff
            .find_entry(DNG_TAG_OPCODE_LIST_1)
            .and_then(|e| e.raw_bytes(buf, is_le));

        let opcode_list_2 = tiff
            .find_entry(DNG_TAG_OPCODE_LIST_2)
            .and_then(|e| e.raw_bytes(buf, is_le));

        let opcode_list_3 = tiff
            .find_entry(DNG_TAG_OPCODE_LIST_3)
            .and_then(|e| e.raw_bytes(buf, is_le));

        Some(Self {
            is_dng,
            camera_make,
            camera_model,
            unique_camera_model,
            baseline_exposure,
            baseline_noise,
            baseline_sharpness,
            linear_response_limit,
            color_matrix_1,
            color_matrix_2,
            forward_matrix_1,
            forward_matrix_2,
            calibration_illuminant_1,
            calibration_illuminant_2,
            analog_balance,
            as_shot_neutral,
            as_shot_white_xy,
            active_area,
            default_crop_origin,
            default_crop_size,
            black_levels,
            white_levels,
            noise_profile,
            preview_slice,
            profile_tone_curve,
            opcode_list_1,
            opcode_list_2,
            opcode_list_3,
        })
    }
}
