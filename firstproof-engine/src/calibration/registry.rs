// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! In-memory Camera Calibration Database mapping camera make and model
//! to authoritative physical sensor properties, eliminating fragile string heuristics.

#![allow(clippy::approx_constant)]

use super::models::CameraCalibration;
use crate::raw::CfaPattern;

// Standard Built-in Color Matrices
pub const CANON_COLOR_MATRIX_TUNGSTEN: [[f32; 3]; 3] = [
    [0.7428, -0.1897, -0.0491],
    [-0.3505, 1.0963, 0.2929],
    [-0.0337, 0.1242, 0.6413],
];

pub const CANON_COLOR_MATRIX_DAYLIGHT: [[f32; 3]; 3] = [
    [0.6719, -0.0994, -0.0925],
    [-0.4408, 1.2426, 0.2211],
    [-0.0887, 0.2129, 0.6051],
];

pub const CANON_FORWARD_MATRIX_TUNGSTEN: [[f32; 3]; 3] = [
    [ 0.7550,  0.0645,  0.1448],
    [ 0.2138,  0.8936, -0.1075],
    [-0.0005, -0.4306,  1.2562],
];

pub const CANON_FORWARD_MATRIX_DAYLIGHT: [[f32; 3]; 3] = [
    [ 0.7286,  0.1385,  0.0972],
    [ 0.2600,  0.9468, -0.2068],
    [ 0.0093, -0.2268,  1.0426],
];


pub const SONY_COLOR_MATRIX_TUNGSTEN: [[f32; 3]; 3] = [
    [0.7483, -0.0851, -0.0534],
    [-0.4109, 1.2392, 0.1971],
    [-0.0701, 0.1884, 0.7071],
];

pub const SONY_COLOR_MATRIX_DAYLIGHT: [[f32; 3]; 3] = [
    [0.6722, 0.0821, 0.1793],
    [0.2514, 0.6542, 0.0944],
    [-0.0084, -0.0782, 0.9118],
];

pub const SONY_FORWARD_MATRIX_TUNGSTEN: [[f32; 3]; 3] = [
    [0.8124, 0.0645, 0.0873],
    [0.2512, 0.8914, -0.1426],
    [-0.0321, -0.2215, 1.0785],
];

pub const SONY_FORWARD_MATRIX_DAYLIGHT: [[f32; 3]; 3] = [
    [0.7142, 0.1245, 0.1255],
    [0.2104, 0.9612, -0.1716],
    [-0.0152, -0.1421, 0.9822],
];

pub const NIKON_COLOR_MATRIX_TUNGSTEN: [[f32; 3]; 3] = [
    [0.7712, -0.0984, -0.0512],
    [-0.4011, 1.2281, 0.1974],
    [-0.0742, 0.2014, 0.7182],
];

pub const NIKON_COLOR_MATRIX_DAYLIGHT: [[f32; 3]; 3] = [
    [0.6881, 0.0714, 0.1892],
    [0.2612, 0.6481, 0.0907],
    [-0.0091, -0.0811, 0.9152],
];

// Family Fallback Calibrations
pub static CANON_FAMILY_FALLBACK: CameraCalibration = CameraCalibration {
    make: "Canon",
    model: "EOS Family",
    clean_name: "Canon EOS Standard",
    baseline_exposure: 0.00,
    linear_response_limit: 0.96,
    cfa_pattern: CfaPattern::Rggb,
    black_levels: [2048.0, 2048.0, 2048.0, 2048.0],
    white_levels: [15677.0, 15677.0, 15677.0, 15677.0],
    color_matrix_tungsten: Some(CANON_COLOR_MATRIX_TUNGSTEN),
    color_matrix_daylight: Some(CANON_COLOR_MATRIX_DAYLIGHT),
    forward_matrix_tungsten: Some(CANON_FORWARD_MATRIX_TUNGSTEN),
    forward_matrix_daylight: Some(CANON_FORWARD_MATRIX_DAYLIGHT),
};

pub static SONY_FAMILY_FALLBACK: CameraCalibration = CameraCalibration {
    make: "Sony",
    model: "Alpha Family",
    clean_name: "Sony Alpha Standard",
    baseline_exposure: 0.50,
    linear_response_limit: 0.95,
    cfa_pattern: CfaPattern::Rggb,
    black_levels: [512.0, 512.0, 512.0, 512.0],
    white_levels: [16383.0, 16383.0, 16383.0, 16383.0],
    color_matrix_tungsten: Some(SONY_COLOR_MATRIX_TUNGSTEN),
    color_matrix_daylight: Some(SONY_COLOR_MATRIX_DAYLIGHT),
    forward_matrix_tungsten: Some(SONY_FORWARD_MATRIX_TUNGSTEN),
    forward_matrix_daylight: Some(SONY_FORWARD_MATRIX_DAYLIGHT),
};

pub static NIKON_FAMILY_FALLBACK: CameraCalibration = CameraCalibration {
    make: "Nikon",
    model: "Z/D Family",
    clean_name: "Nikon Standard",
    baseline_exposure: 0.50,
    linear_response_limit: 0.98,
    cfa_pattern: CfaPattern::Rggb,
    black_levels: [600.0, 600.0, 600.0, 600.0],
    white_levels: [16383.0, 16383.0, 16383.0, 16383.0],
    color_matrix_tungsten: Some(NIKON_COLOR_MATRIX_TUNGSTEN),
    color_matrix_daylight: Some(NIKON_COLOR_MATRIX_DAYLIGHT),
    forward_matrix_tungsten: None,
    forward_matrix_daylight: None,
};

pub static FUJIFILM_FAMILY_FALLBACK: CameraCalibration = CameraCalibration {
    make: "Fujifilm",
    model: "X Family",
    clean_name: "Fujifilm X Standard",
    baseline_exposure: 0.72,
    linear_response_limit: 0.96,
    cfa_pattern: CfaPattern::Rggb,
    black_levels: [1024.0, 1024.0, 1024.0, 1024.0],
    white_levels: [16383.0, 16383.0, 16383.0, 16383.0],
    color_matrix_tungsten: None,
    color_matrix_daylight: None,
    forward_matrix_tungsten: None,
    forward_matrix_daylight: None,
};

pub static PANASONIC_FAMILY_FALLBACK: CameraCalibration = CameraCalibration {
    make: "Panasonic",
    model: "Lumix Family",
    clean_name: "Panasonic Lumix Standard",
    baseline_exposure: 0.45,
    linear_response_limit: 0.96,
    cfa_pattern: CfaPattern::Rggb,
    black_levels: [143.0, 143.0, 143.0, 143.0],
    white_levels: [4095.0, 4095.0, 4095.0, 4095.0],
    color_matrix_tungsten: None,
    color_matrix_daylight: None,
    forward_matrix_tungsten: None,
    forward_matrix_daylight: None,
};

pub static LEICA_FAMILY_FALLBACK: CameraCalibration = CameraCalibration {
    make: "Leica",
    model: "M/Q/SL Family",
    clean_name: "Leica Standard",
    baseline_exposure: 0.00,
    linear_response_limit: 0.96,
    cfa_pattern: CfaPattern::Rggb,
    black_levels: [0.0, 0.0, 0.0, 0.0],
    white_levels: [16383.0, 16383.0, 16383.0, 16383.0],
    color_matrix_tungsten: None,
    color_matrix_daylight: None,
    forward_matrix_tungsten: None,
    forward_matrix_daylight: None,
};

pub static GENERIC_FALLBACK: CameraCalibration = CameraCalibration {
    make: "Generic",
    model: "Standard",
    clean_name: "Generic Rec.2020 Standard",
    baseline_exposure: 0.00,
    linear_response_limit: 0.96,
    cfa_pattern: CfaPattern::Rggb,
    black_levels: [0.0, 0.0, 0.0, 0.0],
    white_levels: [16383.0, 16383.0, 16383.0, 16383.0],
    color_matrix_tungsten: None,
    color_matrix_daylight: None,
    forward_matrix_tungsten: None,
    forward_matrix_daylight: None,
};

// Authoritative Exact Model Registry
pub static CAMERA_CALIBRATIONS: &[CameraCalibration] = &[
    // Canon EOS 60D (Photometrically verified against reference CR2/DNG)
    CameraCalibration {
        make: "Canon",
        model: "EOS 60D",
        clean_name: "Canon EOS 60D",
        baseline_exposure: 0.00,
        linear_response_limit: 0.96,
        cfa_pattern: CfaPattern::Gbrg, // 60D active crop Bayer layout
        black_levels: [2045.0, 2045.0, 2050.0, 2050.0],
        white_levels: [15677.0, 15677.0, 15677.0, 15677.0],
        color_matrix_tungsten: Some(CANON_COLOR_MATRIX_TUNGSTEN),
        color_matrix_daylight: Some(CANON_COLOR_MATRIX_DAYLIGHT),
        forward_matrix_tungsten: Some(CANON_FORWARD_MATRIX_TUNGSTEN),
        forward_matrix_daylight: Some(CANON_FORWARD_MATRIX_DAYLIGHT),
    },
    // Canon EOS 5D Mark II
    CameraCalibration {
        make: "Canon",
        model: "EOS 5D Mark II",
        clean_name: "Canon EOS 5D Mark II",
        baseline_exposure: 0.00,
        linear_response_limit: 0.96,
        cfa_pattern: CfaPattern::Rggb,
        black_levels: [1024.0, 1024.0, 1024.0, 1024.0],
        white_levels: [15677.0, 15677.0, 15677.0, 15677.0],
        color_matrix_tungsten: Some(CANON_COLOR_MATRIX_TUNGSTEN),
        color_matrix_daylight: Some(CANON_COLOR_MATRIX_DAYLIGHT),
        forward_matrix_tungsten: Some(CANON_FORWARD_MATRIX_TUNGSTEN),
        forward_matrix_daylight: Some(CANON_FORWARD_MATRIX_DAYLIGHT),
    },
    // Canon EOS 5D Mark III
    CameraCalibration {
        make: "Canon",
        model: "EOS 5D Mark III",
        clean_name: "Canon EOS 5D Mark III",
        baseline_exposure: 0.00,
        linear_response_limit: 0.96,
        cfa_pattern: CfaPattern::Rggb,
        black_levels: [2048.0, 2048.0, 2048.0, 2048.0],
        white_levels: [15677.0, 15677.0, 15677.0, 15677.0],
        color_matrix_tungsten: Some(CANON_COLOR_MATRIX_TUNGSTEN),
        color_matrix_daylight: Some(CANON_COLOR_MATRIX_DAYLIGHT),
        forward_matrix_tungsten: Some(CANON_FORWARD_MATRIX_TUNGSTEN),
        forward_matrix_daylight: Some(CANON_FORWARD_MATRIX_DAYLIGHT),
    },
    // Canon EOS 5D Mark IV
    CameraCalibration {
        make: "Canon",
        model: "EOS 5D Mark IV",
        clean_name: "Canon EOS 5D Mark IV",
        baseline_exposure: 0.00,
        linear_response_limit: 0.96,
        cfa_pattern: CfaPattern::Rggb,
        black_levels: [2048.0, 2048.0, 2048.0, 2048.0],
        white_levels: [16383.0, 16383.0, 16383.0, 16383.0],
        color_matrix_tungsten: Some(CANON_COLOR_MATRIX_TUNGSTEN),
        color_matrix_daylight: Some(CANON_COLOR_MATRIX_DAYLIGHT),
        forward_matrix_tungsten: Some(CANON_FORWARD_MATRIX_TUNGSTEN),
        forward_matrix_daylight: Some(CANON_FORWARD_MATRIX_DAYLIGHT),
    },
    // Canon EOS R5
    CameraCalibration {
        make: "Canon",
        model: "EOS R5",
        clean_name: "Canon EOS R5",
        baseline_exposure: 0.00,
        linear_response_limit: 0.96,
        cfa_pattern: CfaPattern::Rggb,
        black_levels: [2048.0, 2048.0, 2048.0, 2048.0],
        white_levels: [16383.0, 16383.0, 16383.0, 16383.0],
        color_matrix_tungsten: Some(CANON_COLOR_MATRIX_TUNGSTEN),
        color_matrix_daylight: Some(CANON_COLOR_MATRIX_DAYLIGHT),
        forward_matrix_tungsten: Some(CANON_FORWARD_MATRIX_TUNGSTEN),
        forward_matrix_daylight: Some(CANON_FORWARD_MATRIX_DAYLIGHT),
    },
    // Sony ILCE-7M3 (A7 III)
    CameraCalibration {
        make: "Sony",
        model: "ILCE-7M3",
        clean_name: "Sony Alpha 7 III",
        baseline_exposure: 0.50,
        linear_response_limit: 0.95,
        cfa_pattern: CfaPattern::Rggb,
        black_levels: [512.0, 512.0, 512.0, 512.0],
        white_levels: [16383.0, 16383.0, 16383.0, 16383.0],
        color_matrix_tungsten: Some(SONY_COLOR_MATRIX_TUNGSTEN),
        color_matrix_daylight: Some(SONY_COLOR_MATRIX_DAYLIGHT),
        forward_matrix_tungsten: Some(SONY_FORWARD_MATRIX_TUNGSTEN),
        forward_matrix_daylight: Some(SONY_FORWARD_MATRIX_DAYLIGHT),
    },
    // Sony ILCE-7M4 (A7 IV)
    CameraCalibration {
        make: "Sony",
        model: "ILCE-7M4",
        clean_name: "Sony Alpha 7 IV",
        baseline_exposure: 0.50,
        linear_response_limit: 0.95,
        cfa_pattern: CfaPattern::Rggb,
        black_levels: [512.0, 512.0, 512.0, 512.0],
        white_levels: [16383.0, 16383.0, 16383.0, 16383.0],
        color_matrix_tungsten: Some(SONY_COLOR_MATRIX_TUNGSTEN),
        color_matrix_daylight: Some(SONY_COLOR_MATRIX_DAYLIGHT),
        forward_matrix_tungsten: Some(SONY_FORWARD_MATRIX_TUNGSTEN),
        forward_matrix_daylight: Some(SONY_FORWARD_MATRIX_DAYLIGHT),
    },
    // Nikon D850
    CameraCalibration {
        make: "Nikon",
        model: "D850",
        clean_name: "Nikon D850",
        baseline_exposure: 0.50,
        linear_response_limit: 0.98,
        cfa_pattern: CfaPattern::Rggb,
        black_levels: [600.0, 600.0, 600.0, 600.0],
        white_levels: [16383.0, 16383.0, 16383.0, 16383.0],
        color_matrix_tungsten: Some(NIKON_COLOR_MATRIX_TUNGSTEN),
        color_matrix_daylight: Some(NIKON_COLOR_MATRIX_DAYLIGHT),
        forward_matrix_tungsten: None,
        forward_matrix_daylight: None,
    },
    // Nikon Z 6
    CameraCalibration {
        make: "Nikon",
        model: "Z 6",
        clean_name: "Nikon Z 6",
        baseline_exposure: 0.50,
        linear_response_limit: 0.98,
        cfa_pattern: CfaPattern::Rggb,
        black_levels: [600.0, 600.0, 600.0, 600.0],
        white_levels: [16383.0, 16383.0, 16383.0, 16383.0],
        color_matrix_tungsten: Some(NIKON_COLOR_MATRIX_TUNGSTEN),
        color_matrix_daylight: Some(NIKON_COLOR_MATRIX_DAYLIGHT),
        forward_matrix_tungsten: None,
        forward_matrix_daylight: None,
    },
];

/// The central Camera Calibration Registry interface.
pub struct CameraCalibrationRegistry;

impl CameraCalibrationRegistry {
    /// Look up an exact camera calibration record by make and model.
    pub fn lookup(make: &str, model: &str) -> Option<&'static CameraCalibration> {
        let (norm_make, norm_model) = normalize_camera_identifiers(make, model);

        // 1. Exact match on normalized identifiers
        for cal in CAMERA_CALIBRATIONS {
            let (cal_make, cal_model) = normalize_camera_identifiers(cal.make, cal.model);
            if norm_make == cal_make && norm_model == cal_model {
                return Some(cal);
            }
        }

        // 2. Suffix/prefix model match (e.g. "Canon EOS 60D" vs "60D")
        for cal in CAMERA_CALIBRATIONS {
            let (cal_make, cal_model) = normalize_camera_identifiers(cal.make, cal.model);
            if norm_make == cal_make && (norm_model.contains(&cal_model) || cal_model.contains(&norm_model)) {
                return Some(cal);
            }
        }

        None
    }

    /// Look up camera calibration with principled family fallbacks.
    /// Never fails: returns family default or neutral generic default.
    pub fn lookup_or_fallback(make: &str, model: &str) -> &'static CameraCalibration {
        if let Some(cal) = Self::lookup(make, model) {
            return cal;
        }

        let make_lower = make.to_lowercase();
        let model_lower = model.to_lowercase();

        if make_lower.contains("canon") || model_lower.contains("canon") || model_lower.contains("eos") {
            &CANON_FAMILY_FALLBACK
        } else if make_lower.contains("sony") || model_lower.contains("sony") || model_lower.contains("ilce") || model_lower.contains("alpha") {
            &SONY_FAMILY_FALLBACK
        } else if make_lower.contains("nikon") || model_lower.contains("nikon") {
            &NIKON_FAMILY_FALLBACK
        } else if make_lower.contains("fuji") || model_lower.contains("fuji") {
            &FUJIFILM_FAMILY_FALLBACK
        } else if make_lower.contains("panasonic") || model_lower.contains("lumix") {
            &PANASONIC_FAMILY_FALLBACK
        } else if make_lower.contains("leica") || model_lower.contains("leica") {
            &LEICA_FAMILY_FALLBACK
        } else {
            &GENERIC_FALLBACK
        }
    }
}

/// Normalizes camera make and model strings for robust, resilient matching.
fn normalize_camera_identifiers(make: &str, model: &str) -> (String, String) {
    let mut m_make = make.trim().to_lowercase();
    let mut m_model = model.trim().to_lowercase();

    // Standardize common make aliases
    if m_make.contains("canon") {
        m_make = "canon".to_string();
    } else if m_make.contains("sony") {
        m_make = "sony".to_string();
    } else if m_make.contains("nikon") {
        m_make = "nikon".to_string();
    } else if m_make.contains("fuji") {
        m_make = "fujifilm".to_string();
    } else if m_make.contains("panasonic") || m_make.contains("lumix") {
        m_make = "panasonic".to_string();
    } else if m_make.contains("leica") {
        m_make = "leica".to_string();
    }

    // Strip make prefix from model if present (e.g. "Canon EOS 60D" -> "eos 60d")
    if m_model.starts_with(&m_make) {
        m_model = m_model[m_make.len()..].trim().to_string();
    }

    (m_make, m_model)
}
