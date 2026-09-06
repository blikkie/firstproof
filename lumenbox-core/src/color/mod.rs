//! Scene-referred Color Science Core for LumenBox
//!
//! Provides mathematically rigorous definitions for:
//! - 3D color vectors and 3x3 colorimetric transformation matrices
//! - CIE XYZ (D50 and D65 reference white points)
//! - Linear Rec.2020 (Wide-gamut scene-referred working space)
//! - Bradford Chromatic Adaptation ($D50 \leftrightarrow D65$)
//! - Standard Display Transfer Functions (sRGB / Display P3)

pub mod ciede2000;

/// 3-component floating point color vector (e.g. RGB or XYZ)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color3(pub [f32; 3]);

impl Color3 {
    pub const BLACK: Self = Color3([0.0, 0.0, 0.0]);
    pub const WHITE: Self = Color3([1.0, 1.0, 1.0]);

    #[inline]
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Color3([r, g, b])
    }

    #[inline]
    pub fn r(&self) -> f32 { self.0[0] }
    #[inline]
    pub fn g(&self) -> f32 { self.0[1] }
    #[inline]
    pub fn b(&self) -> f32 { self.0[2] }

    #[inline]
    pub fn clamp(&self, min: f32, max: f32) -> Self {
        Color3([
            self.0[0].clamp(min, max),
            self.0[1].clamp(min, max),
            self.0[2].clamp(min, max),
        ])
    }

    #[inline]
    pub fn scale(&self, s: f32) -> Self {
        Color3([self.0[0] * s, self.0[1] * s, self.0[2] * s])
    }
}

/// 3x3 matrix in row-major order:
/// [ m00, m01, m02 ]
/// [ m10, m11, m12 ]
/// [ m20, m21, m22 ]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3x3(pub [[f32; 3]; 3]);

impl Matrix3x3 {
    pub const IDENTITY: Self = Matrix3x3([
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ]);

    #[inline]
    pub fn multiply_vector(&self, c: Color3) -> Color3 {
        let x = self.0[0][0] * c.0[0] + self.0[0][1] * c.0[1] + self.0[0][2] * c.0[2];
        let y = self.0[1][0] * c.0[0] + self.0[1][1] * c.0[1] + self.0[1][2] * c.0[2];
        let z = self.0[2][0] * c.0[0] + self.0[2][1] * c.0[1] + self.0[2][2] * c.0[2];
        Color3([x, y, z])
    }

    pub fn multiply_matrix(&self, other: &Matrix3x3) -> Matrix3x3 {
        let mut res = [[0.0f32; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                res[i][j] = self.0[i][0] * other.0[0][j]
                          + self.0[i][1] * other.0[1][j]
                          + self.0[i][2] * other.0[2][j];
            }
        }
        Matrix3x3(res)
    }

    /// Computes analytical inverse of 3x3 matrix
    pub fn inverse(&self) -> Option<Matrix3x3> {
        let m = &self.0;
        let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
                - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
                + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);

        if det.abs() < 1e-12 {
            return None;
        }
        let inv_det = 1.0 / det;

        let res = [
            [
                (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv_det,
                (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv_det,
                (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv_det,
            ],
            [
                (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv_det,
                (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv_det,
                (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inv_det,
            ],
            [
                (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv_det,
                (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inv_det,
                (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv_det,
            ],
        ];
        Some(Matrix3x3(res))
    }
}

// ---------------------------------------------------------------------------
// Standard Colorimetric Conversion Matrices
// ---------------------------------------------------------------------------

/// Bradford Chromatic Adaptation Matrix: CIE XYZ (D50) -> CIE XYZ (D65)
pub const BRADFORD_D50_TO_D65: Matrix3x3 = Matrix3x3([
    [ 0.9555766, -0.0230393,  0.0631636],
    [-0.0282895,  1.0099416,  0.0210077],
    [ 0.0122982, -0.0204830,  1.3299098],
]);

/// Bradford Chromatic Adaptation Matrix: CIE XYZ (D65) -> CIE XYZ (D50)
pub const BRADFORD_D65_TO_D50: Matrix3x3 = Matrix3x3([
    [ 1.0478112,  0.0228866, -0.0501270],
    [ 0.0295424,  0.9904844, -0.0170491],
    [-0.0092345,  0.0150436,  0.7521316],
]);

/// CIE XYZ (D65) -> Linear Rec.2020 Matrix (ITU-R BT.2020)
pub const XYZ_D65_TO_REC2020: Matrix3x3 = Matrix3x3([
    [ 1.7166511, -0.3556708, -0.2533663],
    [-0.6666844,  1.6164812,  0.0157685],
    [ 0.0176399, -0.0427706,  0.9421031],
]);

/// Linear Rec.2020 -> CIE XYZ (D65) Matrix (ITU-R BT.2020)
pub const REC2020_TO_XYZ_D65: Matrix3x3 = Matrix3x3([
    [0.6369580, 0.1446169, 0.1688810],
    [0.2627002, 0.6779981, 0.0593017],
    [0.0000000, 0.0280727, 1.0609851],
]);

/// Linear Rec.2020 -> Linear sRGB Matrix
pub const REC2020_TO_LINEAR_SRGB: Matrix3x3 = Matrix3x3([
    [ 1.6604910, -0.5876411, -0.0728499],
    [-0.1245505,  1.1328999, -0.0083494],
    [-0.0181508, -0.1005789,  1.1187297],
]);

/// Linear sRGB -> Linear Rec.2020 Matrix
pub const LINEAR_SRGB_TO_REC2020: Matrix3x3 = Matrix3x3([
    [0.6274041, 0.3292821, 0.0433138],
    [0.0690972, 0.9195411, 0.0113617],
    [0.0163914, 0.0880133, 0.8955953],
]);

/// Linear Rec.2020 -> Display P3 (D65) Matrix
pub const REC2020_TO_DISPLAY_P3: Matrix3x3 = Matrix3x3([
    [ 1.3416485, -0.2981577, -0.0434908],
    [-0.0596327,  1.0697926, -0.0101599],
    [-0.0125866, -0.0527376,  1.0653242],
]);

/// Standard sRGB electro-optical transfer function (linear -> gamma sRGB)
#[inline]
pub fn linear_to_srgb_gamma(x: f32) -> f32 {
    let c = x.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Standard sRGB optical-electro transfer function (gamma sRGB -> linear)
#[inline]
pub fn srgb_gamma_to_linear(x: f32) -> f32 {
    let c = x.clamp(0.0, 1.0);
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Helper to convert linear color to standard 8-bit sRGB display bytes
#[inline]
pub fn linear_rgb_to_srgb8(rgb: Color3) -> [u8; 3] {
    [
        (linear_to_srgb_gamma(rgb.r()) * 255.0).round() as u8,
        (linear_to_srgb_gamma(rgb.g()) * 255.0).round() as u8,
        (linear_to_srgb_gamma(rgb.b()) * 255.0).round() as u8,
    ]
}

/// Standard Canon EOS D65 Camera Color Matrix (Sensor RGB -> CIE XYZ D50)
/// Extracted from standard Adobe Camera Raw calibration for Canon EOS sensor line.
pub const CANON_EOS_SENSOR_TO_XYZ_D50: Matrix3x3 = Matrix3x3([
    [ 0.7034,  0.0613,  0.1995],
    [ 0.2721,  0.6409,  0.0870],
    [-0.0102, -0.0841,  0.9193],
]);
