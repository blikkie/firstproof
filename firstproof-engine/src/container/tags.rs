// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Standard TIFF, EXIF, and Adobe DNG Specification Tag IDs.

// Standard TIFF IFD Tags
pub const TAG_IMAGE_WIDTH: u16 = 256;              // 0x0100
pub const TAG_IMAGE_LENGTH: u16 = 257;             // 0x0101
pub const TAG_BITS_PER_SAMPLE: u16 = 258;          // 0x0102
pub const TAG_COMPRESSION: u16 = 259;              // 0x0103
pub const TAG_PHOTOMETRIC_INTERPRETATION: u16 = 262;// 0x0106
pub const TAG_MAKE: u16 = 271;                     // 0x010F
pub const TAG_MODEL: u16 = 272;                    // 0x0110
pub const TAG_STRIP_OFFSETS: u16 = 273;            // 0x0111
pub const TAG_ORIENTATION: u16 = 274;              // 0x0112
pub const TAG_SAMPLES_PER_PIXEL: u16 = 277;        // 0x0115
pub const TAG_ROWS_PER_STRIP: u16 = 278;           // 0x0116
pub const TAG_STRIP_BYTE_COUNTS: u16 = 279;        // 0x0117
pub const TAG_SUB_IFDS: u16 = 330;                 // 0x014A
pub const TAG_JPEG_INTERCHANGE_FORMAT: u16 = 513;  // 0x0201
pub const TAG_JPEG_INTERCHANGE_FORMAT_LENGTH: u16 = 514; // 0x0202
pub const TAG_EXIF_IFD_POINTER: u16 = 34665;       // 0x8769

// Adobe DNG Specification Tags
pub const TAG_DNG_VERSION: u16 = 50706;            // 0xC612
pub const TAG_DNG_BACKWARD_VERSION: u16 = 50707;   // 0xC613
pub const TAG_UNIQUE_CAMERA_MODEL: u16 = 50708;    // 0xC614
pub const TAG_CFAPLANE_COLOR: u16 = 50710;         // 0xC616
pub const TAG_CFALAYOUT: u16 = 50711;              // 0xC617
pub const TAG_LINEARIZATION_TABLE: u16 = 50712;    // 0xC618
pub const TAG_BLACK_LEVEL_REPEAT_DIM: u16 = 50713; // 0xC619
pub const TAG_BLACK_LEVEL: u16 = 50714;            // 0xC61A
pub const TAG_BLACK_LEVEL_DELTA_H: u16 = 50715;    // 0xC61B
pub const TAG_BLACK_LEVEL_DELTA_V: u16 = 50716;    // 0xC61C
pub const TAG_WHITE_LEVEL: u16 = 50717;            // 0xC61D
pub const TAG_DEFAULT_SCALE: u16 = 50718;          // 0xC61E
pub const TAG_DEFAULT_CROP_ORIGIN: u16 = 50719;    // 0xC61F
pub const TAG_DEFAULT_CROP_SIZE: u16 = 50720;      // 0xC620
pub const TAG_COLOR_MATRIX_1: u16 = 50721;         // 0xC621
pub const TAG_COLOR_MATRIX_2: u16 = 50722;         // 0xC622
pub const TAG_CAMERA_CALIBRATION_1: u16 = 50723;   // 0xC623
pub const TAG_CAMERA_CALIBRATION_2: u16 = 50724;   // 0xC624
pub const TAG_ANALOG_BALANCE: u16 = 50727;         // 0xC627
pub const TAG_AS_SHOT_NEUTRAL: u16 = 50728;        // 0xC628 (often same ID as BaselineExposure depending on IFD context; in IFD0 / SubIFD)
pub const TAG_AS_SHOT_WHITE_XY: u16 = 50729;       // 0xC629
pub const TAG_BASELINE_EXPOSURE: u16 = 50730;      // Note: In DNG spec, Tag 50730 is BaselineExposure or BaselineNoise depending on edition.
// Official Adobe DNG 1.6.0.0 Specification:
// Tag 50728: AsShotNeutral (Count 3, RATIONAL or SRATIONAL)
// Tag 50729: AsShotWhiteXY (Count 2, RATIONAL)
// Tag 50730: BaselineExposure (Count 1, SRATIONAL) -- Note: in Canon/Lightroom DNGs, 50728 vs 50730 is handled with type checking!
// Tag 50731: BaselineNoise (Count 1, RATIONAL)
// Tag 50732: BaselineSharpness (Count 1, RATIONAL)
// Tag 50734: LinearResponseLimit (Count 1, RATIONAL)
// Tag 50735: CameraSerialNumber (ASCII)
// Tag 50736: LensInfo (Count 4, RATIONAL)
// Tag 50738: ShadowScale (Count 1, RATIONAL)
// Tag 50778: CalibrationIlluminant1 (Count 1, SHORT)
// Tag 50779: CalibrationIlluminant2 (Count 1, SHORT)
// Tag 50780: BestQualityScale (Count 1, RATIONAL)
// Tag 50829: ActiveArea (Count 4, SHORT or LONG)
// Tag 50964: ForwardMatrix1 (Count 9, SRATIONAL)
// Tag 50965: ForwardMatrix2 (Count 9, SRATIONAL)
// Tag 51008: OpcodeList1
// Tag 51009: OpcodeList2
// Tag 51022: OpcodeList3
// Tag 51041: NoiseProfile (Count 2 or 4, DOUBLE)

pub const DNG_TAG_AS_SHOT_NEUTRAL: u16 = 50728;
pub const DNG_TAG_BASELINE_EXPOSURE_ALT: u16 = 50728; // Some converters write BaselineExposure as 50728 (SRATIONAL count 1)
pub const DNG_TAG_AS_SHOT_WHITE_XY: u16 = 50729;
pub const DNG_TAG_BASELINE_EXPOSURE: u16 = 50730;     // Standard DNG 1.1+ BaselineExposure (SRATIONAL count 1)
pub const DNG_TAG_BASELINE_NOISE: u16 = 50731;
pub const DNG_TAG_BASELINE_SHARPNESS: u16 = 50732;
pub const DNG_TAG_LINEAR_RESPONSE_LIMIT: u16 = 50734;
pub const DNG_TAG_CALIBRATION_ILLUMINANT_1: u16 = 50778;
pub const DNG_TAG_CALIBRATION_ILLUMINANT_2: u16 = 50779;
pub const DNG_TAG_ACTIVE_AREA: u16 = 50829;
pub const DNG_TAG_FORWARD_MATRIX_1: u16 = 50964;
pub const DNG_TAG_FORWARD_MATRIX_2: u16 = 50965;
pub const DNG_TAG_PROFILE_TONE_CURVE: u16 = 50940;
pub const DNG_TAG_OPCODE_LIST_1: u16 = 51008;
pub const DNG_TAG_OPCODE_LIST_2: u16 = 51009;
pub const DNG_TAG_OPCODE_LIST_3: u16 = 51022;
pub const DNG_TAG_NOISE_PROFILE: u16 = 51041;
