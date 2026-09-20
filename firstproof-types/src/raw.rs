/// Canonical Bayer CFA color indices
pub const CFA_COLOR_RED: usize = 0;
pub const CFA_COLOR_GREEN: usize = 1;
pub const CFA_COLOR_BLUE: usize = 2;

/// Standard Color Filter Array patterns
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CfaPattern {
    Rggb,
    Bggr,
    Gbrg,
    Grbg,
    XTrans,
    LinearRgb,
}

#[inline]
pub fn f32_to_f16(val: f32) -> u16 {
    let bits = val.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exp = (((bits >> 23) & 0xff) as i32) - 127 + 15;
    let mant = bits & 0x007f_ffff;

    if exp <= 0 {
        sign
    } else if exp >= 31 {
        sign | 0x7c00
    } else {
        sign | ((exp as u16) << 10) | ((mant >> 13) as u16)
    }
}

/// Convert IEEE 754 16-bit half-precision float (f16) bit representation to f32.
pub fn f16_to_f32(val: u16) -> f32 {
    let sign = ((val & 0x8000) as u32) << 16;
    let exp = ((val & 0x7c00) >> 10) as u32;
    let mant = (val & 0x03ff) as u32;

    if exp == 0 {
        if mant == 0 {
            f32::from_bits(sign)
        } else {
            let mut m = mant << 1;
            let mut e = 0;
            while (m & 0x0400) == 0 {
                m <<= 1;
                e += 1;
            }
            let exp_f32 = (127 - 15 - e) << 23;
            let mant_f32 = (m & 0x03ff) << 13;
            f32::from_bits(sign | exp_f32 | mant_f32)
        }
    } else if exp == 31 {
        let exp_f32 = 0xff << 23;
        let mant_f32 = mant << 13;
        f32::from_bits(sign | exp_f32 | mant_f32)
    } else {
        let exp_f32 = (exp + 127 - 15) << 23;
        let mant_f32 = mant << 13;
        f32::from_bits(sign | exp_f32 | mant_f32)
    }
}

/// A library-agnostic raw sensor frame representation.
/// Completely decouples downstream demosaicing, tone mapping, and export pipelines
/// from specific RAW decoding crates (rawler, libraw, dcraw, etc.).
#[derive(Clone, Debug)]
pub struct RawSensorFrame {
    pub full_width: usize,
    pub full_height: usize,
    pub active_x: usize,
    pub active_y: usize,
    pub active_width: usize,
    pub active_height: usize,
    pub cfa_pattern: CfaPattern,
    pub black_levels: [f32; 4],
    pub white_levels: [f32; 4],
    pub wb_coeffs: [f32; 4],
    pub color_matrix: Option<[[f32; 3]; 3]>,
    pub color_matrix_tungsten: Option<[[f32; 3]; 3]>,
    pub camera_model: String,
    pub sensor_data: Vec<u16>,
}

impl RawSensorFrame {
    /// Returns the color index at the given (row, col) sensor coordinates.
    /// 0 = Red, 1 = Green, 2 = Blue.
    #[inline]
    pub fn color_at(&self, row: usize, col: usize) -> usize {
        match self.cfa_pattern {
            CfaPattern::Rggb => match (row % 2, col % 2) {
                (0, 0) => CFA_COLOR_RED,
                (0, 1) => CFA_COLOR_GREEN,
                (1, 0) => CFA_COLOR_GREEN,
                (1, 1) => CFA_COLOR_BLUE,
                _ => CFA_COLOR_GREEN,
            },
            CfaPattern::Bggr => match (row % 2, col % 2) {
                (0, 0) => CFA_COLOR_BLUE,
                (0, 1) => CFA_COLOR_GREEN,
                (1, 0) => CFA_COLOR_GREEN,
                (1, 1) => CFA_COLOR_RED,
                _ => CFA_COLOR_GREEN,
            },
            CfaPattern::Gbrg => match (row % 2, col % 2) {
                (0, 0) => CFA_COLOR_GREEN,
                (0, 1) => CFA_COLOR_BLUE,
                (1, 0) => CFA_COLOR_RED,
                (1, 1) => CFA_COLOR_GREEN,
                _ => CFA_COLOR_GREEN,
            },
            CfaPattern::Grbg => match (row % 2, col % 2) {
                (0, 0) => CFA_COLOR_GREEN,
                (0, 1) => CFA_COLOR_RED,
                (1, 0) => CFA_COLOR_BLUE,
                (1, 1) => CFA_COLOR_GREEN,
                _ => CFA_COLOR_GREEN,
            },
            _ => CFA_COLOR_GREEN,
        }
    }

    /// Normalized sensor photosite sample clamped to 0.0 .. 1.0 linear light
    #[inline]
    pub fn sample_normalized(&self, x: usize, y: usize) -> f32 {
        let idx = y * self.full_width + x;
        if idx >= self.sensor_data.len() {
            return 0.0;
        }
        let raw_val = self.sensor_data[idx] as f32;
        let black = self.black_levels[0].max(0.0);
        let white = self.white_levels[0].max(black + 1.0);
        let scale = 1.0 / (white - black);
        ((raw_val - black) * scale).clamp(0.0, 1.0)
    }

    /// Camera As-Shot White Balance Red & Blue multipliers normalized to Green
    #[inline]
    pub fn as_shot_wb(&self) -> (f32, f32) {
        let wb_r = if self.wb_coeffs[0] > 0.0 && self.wb_coeffs[1] > 0.0 {
            self.wb_coeffs[0] / self.wb_coeffs[1]
        } else {
            2.0
        };
        let wb_b = if self.wb_coeffs[2] > 0.0 && self.wb_coeffs[1] > 0.0 {
            self.wb_coeffs[2] / self.wb_coeffs[1]
        } else {
            1.5
        };
        (wb_r, wb_b)
    }

    /// Computes the normalized camera Sensor to Linear Rec.2020 3x3 matrix from ColorMatrix.
    /// Preserves neutral axis [1, 1, 1] per Adobe DNG / First Proof color science.
    pub fn compute_sensor_to_rec2020(&self) -> [[f32; 3]; 3] {
        let cm = match self.color_matrix {
            Some(m) => glam::Mat3::from_cols_array(&[
                m[0][0], m[1][0], m[2][0],
                m[0][1], m[1][1], m[2][1],
                m[0][2], m[1][2], m[2][2],
            ]),
            None => return [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        };

        // REC2020_TO_XYZ (D65) column-major matrix
        let rec2xyz = glam::Mat3::from_cols_array(&[
            0.636958, 0.262700, 0.000000,
            0.144617, 0.677998, 0.028073,
            0.168881, 0.059302, 1.060985,
        ]);

        let rec2cam_raw = cm * rec2xyz;
        let r0 = rec2cam_raw.row(0);
        let r1 = rec2cam_raw.row(1);
        let r2 = rec2cam_raw.row(2);

        let norm_r = (r0.x + r0.y + r0.z).max(1e-6);
        let norm_g = (r1.x + r1.y + r1.z).max(1e-6);
        let norm_b = (r2.x + r2.y + r2.z).max(1e-6);

        let rec2cam = glam::Mat3::from_cols_array(&[
            r0.x / norm_r, r1.x / norm_g, r2.x / norm_b,
            r0.y / norm_r, r1.y / norm_g, r2.y / norm_b,
            r0.z / norm_r, r1.z / norm_g, r2.z / norm_b,
        ]);

        let sensor_to_rec2020 = if rec2cam.determinant().abs() > 1e-12 {
            rec2cam.inverse()
        } else {
            glam::Mat3::IDENTITY
        };

        let c0 = sensor_to_rec2020.x_axis;
        let c1 = sensor_to_rec2020.y_axis;
        let c2 = sensor_to_rec2020.z_axis;
        [
            [c0.x, c1.x, c2.x],
            [c0.y, c1.y, c2.y],
            [c0.z, c1.z, c2.z],
        ]
    }

    /// Renders a fast 2x2 box-binned 16-bit float (f16) RGBA texture buffer in Linear Rec.2020 space.
    /// Maintains 1:1 math and color science parity with Metal canvas viewport rendering.
    pub fn to_binned_linear_f16(&self) -> (Vec<u16>, u32, u32) {
        let active_w = self.active_width;
        let active_h = self.active_height;
        let bin_w = (active_w / 2) as u32;
        let bin_h = (active_h / 2) as u32;
        if bin_w == 0 || bin_h == 0 {
            return (Vec::new(), 0, 0);
        }

        let (wb_r, wb_b) = self.as_shot_wb();
        let sensor_to_rec2020 = self.compute_sensor_to_rec2020();
        let m00 = sensor_to_rec2020[0][0];
        let m01 = sensor_to_rec2020[0][1];
        let m02 = sensor_to_rec2020[0][2];
        let m10 = sensor_to_rec2020[1][0];
        let m11 = sensor_to_rec2020[1][1];
        let m12 = sensor_to_rec2020[1][2];
        let m20 = sensor_to_rec2020[2][0];
        let m21 = sensor_to_rec2020[2][1];
        let m22 = sensor_to_rec2020[2][2];

        let black = self.black_levels[0].max(0.0);
        let white = self.white_levels[0].max(black + 1.0);
        let scale = 1.0 / (white - black);
        let a16 = f32_to_f16(1.0);

        let mut u16_pixels = vec![0u16; (bin_w * bin_h * 4) as usize];
        let sensor = &self.sensor_data;
        let full_w = self.full_width;
        let row_len = (bin_w as usize) * 2;

        for by in 0..(bin_h as usize) {
            let sy = self.active_y + by * 2;
            let dst_row_off = by * (bin_w as usize) * 4;

            let row0_start = sy * full_w + self.active_x;
            let row1_start = (sy + 1) * full_w + self.active_x;

            if row0_start + row_len <= sensor.len() && row1_start + row_len <= sensor.len() {
                let row0 = &sensor[row0_start..row0_start + row_len];
                let row1 = &sensor[row1_start..row1_start + row_len];

                match self.cfa_pattern {
                    CfaPattern::Rggb => {
                        for bx in 0..(bin_w as usize) {
                            let p00 = row0[bx * 2] as f32;
                            let p01 = row0[bx * 2 + 1] as f32;
                            let p10 = row1[bx * 2] as f32;
                            let p11 = row1[bx * 2 + 1] as f32;

                            let r_norm = ((p00 - black) * scale).clamp(0.0, 1.0);
                            let g_norm = (((p01 + p10) * 0.5 - black) * scale).clamp(0.0, 1.0);
                            let b_norm = ((p11 - black) * scale).clamp(0.0, 1.0);

                            let r_wb = (r_norm * wb_r).clamp(0.0, 1.0);
                            let g_wb = g_norm;
                            let b_wb = (b_norm * wb_b).clamp(0.0, 1.0);

                            let r_lin = (m00 * r_wb + m01 * g_wb + m02 * b_wb).max(0.0);
                            let g_lin = (m10 * r_wb + m11 * g_wb + m12 * b_wb).max(0.0);
                            let b_lin = (m20 * r_wb + m21 * g_wb + m22 * b_wb).max(0.0);

                            let dst_idx = dst_row_off + bx * 4;
                            u16_pixels[dst_idx] = f32_to_f16(r_lin);
                            u16_pixels[dst_idx + 1] = f32_to_f16(g_lin);
                            u16_pixels[dst_idx + 2] = f32_to_f16(b_lin);
                            u16_pixels[dst_idx + 3] = a16;
                        }
                    }
                    CfaPattern::Bggr => {
                        for bx in 0..(bin_w as usize) {
                            let p00 = row0[bx * 2] as f32;
                            let p01 = row0[bx * 2 + 1] as f32;
                            let p10 = row1[bx * 2] as f32;
                            let p11 = row1[bx * 2 + 1] as f32;

                            let b_norm = ((p00 - black) * scale).clamp(0.0, 1.0);
                            let g_norm = (((p01 + p10) * 0.5 - black) * scale).clamp(0.0, 1.0);
                            let r_norm = ((p11 - black) * scale).clamp(0.0, 1.0);

                            let r_wb = (r_norm * wb_r).clamp(0.0, 1.0);
                            let g_wb = g_norm;
                            let b_wb = (b_norm * wb_b).clamp(0.0, 1.0);

                            let r_lin = (m00 * r_wb + m01 * g_wb + m02 * b_wb).max(0.0);
                            let g_lin = (m10 * r_wb + m11 * g_wb + m12 * b_wb).max(0.0);
                            let b_lin = (m20 * r_wb + m21 * g_wb + m22 * b_wb).max(0.0);

                            let dst_idx = dst_row_off + bx * 4;
                            u16_pixels[dst_idx] = f32_to_f16(r_lin);
                            u16_pixels[dst_idx + 1] = f32_to_f16(g_lin);
                            u16_pixels[dst_idx + 2] = f32_to_f16(b_lin);
                            u16_pixels[dst_idx + 3] = a16;
                        }
                    }
                    CfaPattern::Gbrg => {
                        for bx in 0..(bin_w as usize) {
                            let p00 = row0[bx * 2] as f32;
                            let p01 = row0[bx * 2 + 1] as f32;
                            let p10 = row1[bx * 2] as f32;
                            let p11 = row1[bx * 2 + 1] as f32;

                            let g_norm = (((p00 + p11) * 0.5 - black) * scale).clamp(0.0, 1.0);
                            let b_norm = ((p01 - black) * scale).clamp(0.0, 1.0);
                            let r_norm = ((p10 - black) * scale).clamp(0.0, 1.0);

                            let r_wb = (r_norm * wb_r).clamp(0.0, 1.0);
                            let g_wb = g_norm;
                            let b_wb = (b_norm * wb_b).clamp(0.0, 1.0);

                            let r_lin = (m00 * r_wb + m01 * g_wb + m02 * b_wb).max(0.0);
                            let g_lin = (m10 * r_wb + m11 * g_wb + m12 * b_wb).max(0.0);
                            let b_lin = (m20 * r_wb + m21 * g_wb + m22 * b_wb).max(0.0);

                            let dst_idx = dst_row_off + bx * 4;
                            u16_pixels[dst_idx] = f32_to_f16(r_lin);
                            u16_pixels[dst_idx + 1] = f32_to_f16(g_lin);
                            u16_pixels[dst_idx + 2] = f32_to_f16(b_lin);
                            u16_pixels[dst_idx + 3] = a16;
                        }
                    }
                    CfaPattern::Grbg => {
                        for bx in 0..(bin_w as usize) {
                            let p00 = row0[bx * 2] as f32;
                            let p01 = row0[bx * 2 + 1] as f32;
                            let p10 = row1[bx * 2] as f32;
                            let p11 = row1[bx * 2 + 1] as f32;

                            let g_norm = (((p00 + p11) * 0.5 - black) * scale).clamp(0.0, 1.0);
                            let r_norm = ((p01 - black) * scale).clamp(0.0, 1.0);
                            let b_norm = ((p10 - black) * scale).clamp(0.0, 1.0);

                            let r_wb = (r_norm * wb_r).clamp(0.0, 1.0);
                            let g_wb = g_norm;
                            let b_wb = (b_norm * wb_b).clamp(0.0, 1.0);

                            let r_lin = (m00 * r_wb + m01 * g_wb + m02 * b_wb).max(0.0);
                            let g_lin = (m10 * r_wb + m11 * g_wb + m12 * b_wb).max(0.0);
                            let b_lin = (m20 * r_wb + m21 * g_wb + m22 * b_wb).max(0.0);

                            let dst_idx = dst_row_off + bx * 4;
                            u16_pixels[dst_idx] = f32_to_f16(r_lin);
                            u16_pixels[dst_idx + 1] = f32_to_f16(g_lin);
                            u16_pixels[dst_idx + 2] = f32_to_f16(b_lin);
                            u16_pixels[dst_idx + 3] = a16;
                        }
                    }
                    _ => {}
                }
            }
        }

        (u16_pixels, bin_w, bin_h)
    }
}
