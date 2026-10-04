// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Camera calibration records and sensor definitions.

use crate::raw::CfaPattern;

/// Authoritative physical sensor calibration parameters for a specific camera model.
#[derive(Clone, Debug, PartialEq)]
pub struct CameraCalibration {
    pub make: &'static str,
    pub model: &'static str,
    pub clean_name: &'static str,
    pub baseline_exposure: f32,
    pub baseline_sharpness: f32,
    pub linear_response_limit: f32,
    pub cfa_pattern: CfaPattern,
    pub black_levels: [f32; 4],
    pub white_levels: [f32; 4],
    pub color_matrix_tungsten: Option<[[f32; 3]; 3]>,
    pub color_matrix_daylight: Option<[[f32; 3]; 3]>,
    pub forward_matrix_tungsten: Option<[[f32; 3]; 3]>,
    pub forward_matrix_daylight: Option<[[f32; 3]; 3]>,
}

impl Default for CameraCalibration {
    fn default() -> Self {
        Self {
            make: "Generic",
            model: "Standard",
            clean_name: "Generic Rec.2020 Standard",
            baseline_exposure: 0.0,
            baseline_sharpness: 1.0,
            linear_response_limit: 0.96,
            cfa_pattern: CfaPattern::Rggb,
            black_levels: [0.0, 0.0, 0.0, 0.0],
            white_levels: [16383.0, 16383.0, 16383.0, 16383.0],
            color_matrix_tungsten: None,
            color_matrix_daylight: None,
            forward_matrix_tungsten: None,
            forward_matrix_daylight: None,
        }
    }
}
