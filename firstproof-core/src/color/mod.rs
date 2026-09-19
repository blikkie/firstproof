//! Scene-referred Color Science Core for First Proof
//!
//! Provides mathematically rigorous definitions for:
//! - 3D color vectors and 3x3 colorimetric transformation matrices
//! - CIE XYZ (D50 and D65 reference white points)
//! - Linear Rec.2020 (Wide-gamut scene-referred working space)
//! - Bradford Chromatic Adaptation ($D50 \leftrightarrow D65$)
//! - Standard Display Transfer Functions (sRGB / Display P3)

pub mod agx;
pub mod ciede2000;
pub mod dcp;
pub mod emulsion;
pub mod print_media;

pub use agx::{agx_tone_map, agx_tone_map_vivid, AGX_INSET_MATRIX, AGX_OUTSET_MATRIX};
pub use dcp::{DcpHueSatMap, DcpProfile, evaluate_tone_curve, hsv_to_rgb, rgb_to_hsv};
pub use emulsion::{apply_recording_emulsion, EmulsionArchetype, EmulsionFormat};
pub use print_media::{apply_print_media, EnlargerFilterMode, PrintMediaMode};

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
    pub fn to_vec3(&self) -> glam::Vec3 {
        glam::Vec3::new(self.0[0], self.0[1], self.0[2])
    }

    #[inline]
    pub fn from_vec3(v: glam::Vec3) -> Self {
        Color3([v.x, v.y, v.z])
    }

    #[inline]
    pub fn r(&self) -> f32 { self.0[0] }
    #[inline]
    pub fn g(&self) -> f32 { self.0[1] }
    #[inline]
    pub fn b(&self) -> f32 { self.0[2] }

    #[inline]
    pub fn clamp(&self, min: f32, max: f32) -> Self {
        Self::from_vec3(self.to_vec3().clamp(glam::Vec3::splat(min), glam::Vec3::splat(max)))
    }

    #[inline]
    pub fn scale(&self, s: f32) -> Self {
        Self::from_vec3(self.to_vec3() * s)
    }
}

impl From<glam::Vec3> for Color3 {
    #[inline]
    fn from(v: glam::Vec3) -> Self {
        Color3::from_vec3(v)
    }
}

impl From<Color3> for glam::Vec3 {
    #[inline]
    fn from(c: Color3) -> Self {
        c.to_vec3()
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
    pub fn to_mat3(&self) -> glam::Mat3 {
        // glam::Mat3 stores columns: col0, col1, col2
        glam::Mat3::from_cols(
            glam::Vec3::new(self.0[0][0], self.0[1][0], self.0[2][0]),
            glam::Vec3::new(self.0[0][1], self.0[1][1], self.0[2][1]),
            glam::Vec3::new(self.0[0][2], self.0[1][2], self.0[2][2]),
        )
    }

    #[inline]
    pub fn from_mat3(m: glam::Mat3) -> Self {
        let c0 = m.x_axis;
        let c1 = m.y_axis;
        let c2 = m.z_axis;
        Matrix3x3([
            [c0.x, c1.x, c2.x],
            [c0.y, c1.y, c2.y],
            [c0.z, c1.z, c2.z],
        ])
    }

    #[inline]
    pub fn multiply_vector(&self, c: Color3) -> Color3 {
        let v = self.to_mat3() * c.to_vec3();
        Color3::from_vec3(v)
    }

    #[inline]
    pub fn multiply_matrix(&self, other: &Matrix3x3) -> Matrix3x3 {
        let res = self.to_mat3() * other.to_mat3();
        Matrix3x3::from_mat3(res)
    }

    #[inline]
    pub fn inverse(&self) -> Option<Matrix3x3> {
        let m = self.to_mat3();
        if m.determinant().abs() < 1e-12 {
            None
        } else {
            Some(Matrix3x3::from_mat3(m.inverse()))
        }
    }

    #[inline]
    pub fn transpose(&self) -> Matrix3x3 {
        Matrix3x3::from_mat3(self.to_mat3().transpose())
    }

    #[inline]
    pub fn determinant(&self) -> f32 {
        self.to_mat3().determinant()
    }
}

impl From<glam::Mat3> for Matrix3x3 {
    #[inline]
    fn from(m: glam::Mat3) -> Self {
        Matrix3x3::from_mat3(m)
    }
}

impl From<Matrix3x3> for glam::Mat3 {
    #[inline]
    fn from(m: Matrix3x3) -> Self {
        m.to_mat3()
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

/// Returns dynamic CAT16 chromatic adaptation matrix mapping CIE XYZ (D50) -> CIE XYZ (D65)
pub fn d50_to_d65_cat16_matrix() -> Matrix3x3 {
    cat16_adaptation_matrix((0.3457, 0.3585), (0.3127, 0.3290))
}

/// Returns dynamic CAT16 chromatic adaptation matrix mapping CIE XYZ (D65) -> CIE XYZ (D50)
pub fn d65_to_d50_cat16_matrix() -> Matrix3x3 {
    cat16_adaptation_matrix((0.3127, 0.3290), (0.3457, 0.3585))
}

/// Returns dynamic conversion matrix mapping Linear Rec.2020 -> Linear sRGB
pub fn rec2020_to_linear_srgb_matrix() -> Matrix3x3 {
    ColorSpacePrimaries::REC2020.conversion_matrix_to(&ColorSpacePrimaries::SRGB)
}

/// Returns dynamic conversion matrix mapping Linear sRGB -> Linear Rec.2020
pub fn linear_srgb_to_rec2020_matrix() -> Matrix3x3 {
    ColorSpacePrimaries::SRGB.conversion_matrix_to(&ColorSpacePrimaries::REC2020)
}

/// Returns dynamic conversion matrix mapping Linear Rec.2020 -> Display P3 (D65)
pub fn rec2020_to_display_p3_matrix() -> Matrix3x3 {
    ColorSpacePrimaries::REC2020.conversion_matrix_to(&ColorSpacePrimaries::DISPLAY_P3)
}

// ---------------------------------------------------------------------------
// Dynamic Color Science & Primary Algebra
// ---------------------------------------------------------------------------

/// Primary CIE 1931 (x, y) chromaticity coordinates and reference white point
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorSpacePrimaries {
    pub red: (f32, f32),
    pub green: (f32, f32),
    pub blue: (f32, f32),
    pub white: (f32, f32),
}

impl ColorSpacePrimaries {
    pub const SRGB: Self = Self {
        red: (0.6400, 0.3300),
        green: (0.3000, 0.6000),
        blue: (0.1500, 0.0600),
        white: (0.3127, 0.3290), // D65
    };

    pub const REC2020: Self = Self {
        red: (0.7080, 0.2920),
        green: (0.1700, 0.7970),
        blue: (0.1310, 0.0460),
        white: (0.3127, 0.3290), // D65
    };

    pub const DISPLAY_P3: Self = Self {
        red: (0.6800, 0.3200),
        green: (0.2650, 0.6900),
        blue: (0.1500, 0.0600),
        white: (0.3127, 0.3290), // D65
    };

    pub const ADOBE_RGB: Self = Self {
        red: (0.6400, 0.3300),
        green: (0.2100, 0.7100),
        blue: (0.1500, 0.0600),
        white: (0.3127, 0.3290), // D65
    };

    /// Computes the exact 3x3 transformation matrix mapping RGB -> CIE XYZ (D65)
    pub fn rgb_to_xyz_matrix(&self) -> Matrix3x3 {
        let (xr, yr) = self.red;
        let (xg, yg) = self.green;
        let (xb, yb) = self.blue;
        let (xw, yw) = self.white;

        let xr_x = xr / yr;
        let xr_y = 1.0f32;
        let xr_z = (1.0 - xr - yr) / yr;

        let xg_x = xg / yg;
        let xg_y = 1.0f32;
        let xg_z = (1.0 - xg - yg) / yg;

        let xb_x = xb / yb;
        let xb_y = 1.0f32;
        let xb_z = (1.0 - xb - yb) / yb;

        let xw_x = xw / yw;
        let xw_y = 1.0f32;
        let xw_z = (1.0 - xw - yw) / yw;

        let m_prim = Matrix3x3([
            [xr_x, xg_x, xb_x],
            [xr_y, xg_y, xb_y],
            [xr_z, xg_z, xb_z],
        ]);

        let m_inv = m_prim.inverse().unwrap_or(Matrix3x3::IDENTITY);
        let s = m_inv.multiply_vector(Color3::new(xw_x, xw_y, xw_z));

        Matrix3x3([
            [xr_x * s.r(), xg_x * s.g(), xb_x * s.b()],
            [xr_y * s.r(), xg_y * s.g(), xb_y * s.b()],
            [xr_z * s.r(), xg_z * s.g(), xb_z * s.b()],
        ])
    }

    /// Computes the exact 3x3 transformation matrix mapping CIE XYZ (D65) -> RGB
    pub fn xyz_to_rgb_matrix(&self) -> Matrix3x3 {
        self.rgb_to_xyz_matrix().inverse().unwrap_or(Matrix3x3::IDENTITY)
    }

    /// Computes direct conversion matrix from this color space to target color space
    pub fn conversion_matrix_to(&self, target: &Self) -> Matrix3x3 {
        let xyz_to_target = target.xyz_to_rgb_matrix();
        let self_to_xyz = self.rgb_to_xyz_matrix();
        xyz_to_target.multiply_matrix(&self_to_xyz)
    }
}

/// CAT16 Chromatic Adaptation Transform matrix generator for arbitrary white points
pub fn cat16_adaptation_matrix(source_xy: (f32, f32), target_xy: (f32, f32)) -> Matrix3x3 {
    let cat16 = Matrix3x3([
        [ 0.401288,  0.650173, -0.051461],
        [-0.250268,  1.204414,  0.045854],
        [-0.002079,  0.048952,  0.953127],
    ]);
    let cat16_inv = cat16.inverse().unwrap_or(Matrix3x3::IDENTITY);

    let (x1, y1) = source_xy;
    let (x2, y2) = target_xy;

    let xyz1 = Color3::new(x1 / y1, 1.0, (1.0 - x1 - y1) / y1);
    let xyz2 = Color3::new(x2 / y2, 1.0, (1.0 - x2 - y2) / y2);

    let rgb1 = cat16.multiply_vector(xyz1);
    let rgb2 = cat16.multiply_vector(xyz2);

    let diag = Matrix3x3([
        [rgb2.r() / rgb1.r().max(1e-6), 0.0, 0.0],
        [0.0, rgb2.g() / rgb1.g().max(1e-6), 0.0],
        [0.0, 0.0, rgb2.b() / rgb1.b().max(1e-6)],
    ]);

    let step = diag.multiply_matrix(&cat16);
    cat16_inv.multiply_matrix(&step)
}

/// Computes CIE 1931 (x, y) chromaticity for a given correlated color temperature (Kelvin) and green-magenta tint.
pub fn kelvin_and_tint_to_xy(temp_kelvin: f32, tint: f32) -> (f32, f32) {
    let t = temp_kelvin.clamp(2000.0, 50000.0);
    let x = if t <= 7000.0 {
        -4.6070e9 / (t * t * t) + 2.9678e6 / (t * t) + 0.09911e3 / t + 0.244063
    } else {
        -2.0064e9 / (t * t * t) + 1.9018e6 / (t * t) + 0.24748e3 / t + 0.237040
    };
    let y = -3.000 * x * x + 2.870 * x - 0.275;

    // Apply Tint perpendicular to Planckian locus in CIE 1976 u'v' space
    let denom = (-2.0 * x + 12.0 * y + 3.0).max(1e-6);
    let u_prime = 4.0 * x / denom;
    let v_prime = 9.0 * y / denom;

    // Perpendicular green-magenta shift vector in u'v'
    let tint_shift = tint * 0.0002;
    let u_corr = u_prime - tint_shift * 0.3;
    let v_corr = v_prime + tint_shift;

    // Convert u'v' back to xy
    let x_corr = (9.0 * u_corr) / (6.0 * u_corr - 16.0 * v_corr + 12.0).max(1e-6);
    let y_corr = (4.0 * v_corr) / (6.0 * u_corr - 16.0 * v_corr + 12.0).max(1e-6);

    (x_corr, y_corr)
}

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

/// Canonical sRGB ICC profile ("sRGB IEC61966-2.1") for embedding into exported JPEGs.
/// Enables macOS ColorSync and display servers to accurately map sRGB pixel values
/// into wide-gamut Display P3 panels without green/cyan chromaticity drift.
pub const SRGB_ICC_PROFILE: &[u8] = include_bytes!("srgb.icc");
