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

/// Sensitometrically blends raw white-balanced color towards neutral white as photosites approach saturation.
/// Eliminates asymmetric highlight blowout (e.g. tungsten blue channel amplification)
/// along a smooth C^1 Hermite curve while preserving full highlight luminance headroom.
#[inline]
pub fn blend_sensor_highlights(r: f32, g: f32, b: f32, wb_r: f32, wb_b: f32) -> (f32, f32, f32) {
    let s_max = r.max(g).max(b);
    let p_r = r * wb_r;
    let p_g = g;
    let p_b = b * wb_b;
    if s_max > 0.88 {
        let t = ((s_max - 0.88) / 0.12).clamp(0.0, 1.0);
        let alpha = t * t * (3.0 - 2.0 * t);
        let l_est = p_r.max(p_g).max(p_b);
        (
            (1.0 - alpha) * p_r + alpha * l_est,
            (1.0 - alpha) * p_g + alpha * l_est,
            (1.0 - alpha) * p_b + alpha * l_est,
        )
    } else {
        (p_r, p_g, p_b)
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
    pub baseline_exposure: f32,
    pub sensor_data: Vec<u16>,
}

/// Extracts Tag 50728 (BaselineExposure) from a DNG / TIFF byte buffer (IFD0).
/// Returns linear exposure offset in EV (e.g. +0.831169).
pub fn read_dng_baseline_exposure(bytes: &[u8]) -> Option<f32> {
    if bytes.len() < 8 {
        return None;
    }
    let is_le = match &bytes[0..2] {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let read_u16 = |buf: &[u8]| -> u16 {
        if is_le { u16::from_le_bytes([buf[0], buf[1]]) }
        else { u16::from_be_bytes([buf[0], buf[1]]) }
    };
    let read_u32 = |buf: &[u8]| -> u32 {
        if is_le { u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) }
        else { u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) }
    };
    let read_i32 = |buf: &[u8]| -> i32 {
        if is_le { i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) }
        else { i32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) }
    };

    let magic = read_u16(&bytes[2..4]);
    if magic != 42 && magic != 0x55 {
        return None;
    }
    let ifd_offset = read_u32(&bytes[4..8]) as usize;
    if ifd_offset + 2 > bytes.len() {
        return None;
    }
    let num_entries = read_u16(&bytes[ifd_offset..ifd_offset + 2]) as usize;
    let mut curr = ifd_offset + 2;

    for _ in 0..num_entries {
        if curr + 12 > bytes.len() {
            break;
        }
        let tag = read_u16(&bytes[curr..curr + 2]);
        let typ = read_u16(&bytes[curr + 2..curr + 4]);
        if tag == 50728 {
            let offset = read_u32(&bytes[curr + 8..curr + 12]) as usize;
            if offset + 8 <= bytes.len() {
                if typ == 10 { // SRATIONAL (2 x i32)
                    let num = read_i32(&bytes[offset..offset + 4]);
                    let den = read_i32(&bytes[offset + 4..offset + 8]);
                    if den != 0 {
                        return Some(num as f32 / den as f32);
                    }
                } else if typ == 5 { // RATIONAL (2 x u32)
                    let num = read_u32(&bytes[offset..offset + 4]);
                    let den = read_u32(&bytes[offset + 4..offset + 8]);
                    if den != 0 {
                        return Some(num as f32 / den as f32);
                    }
                }
            }
        }
        curr += 12;
    }
    None
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

    /// Estimates the correlated color temperature (in Kelvin) from camera as-shot white balance
    /// and dual-illuminant sensor calibration matrices.
    pub fn as_shot_temperature(&self) -> f32 {
        let (wb_r, wb_b) = self.as_shot_wb();
        if let (Some(m_tungsten), Some(m_daylight)) = (self.color_matrix_tungsten, self.color_matrix) {
            let m1 = glam::Mat3::from_cols_array(&[
                m_tungsten[0][0], m_tungsten[1][0], m_tungsten[2][0],
                m_tungsten[0][1], m_tungsten[1][1], m_tungsten[2][1],
                m_tungsten[0][2], m_tungsten[1][2], m_tungsten[2][2],
            ]);
            let m2 = glam::Mat3::from_cols_array(&[
                m_daylight[0][0], m_daylight[1][0], m_daylight[2][0],
                m_daylight[0][1], m_daylight[1][1], m_daylight[2][1],
                m_daylight[0][2], m_daylight[1][2], m_daylight[2][2],
            ]);

            // CIE XYZ coordinates for Standard Illuminant A (2856K) and D65 (6504K)
            let xyz_a = glam::Vec3::new(1.09867, 1.0, 0.35592);
            let xyz_d65 = glam::Vec3::new(0.95045, 1.0, 1.08906);

            let n_a = m1 * xyz_a;
            let n_d65 = m2 * xyz_d65;

            let r1 = (n_a.x / n_a.z.max(1e-6)).max(1e-6);
            let r2 = (n_d65.x / n_d65.z.max(1e-6)).max(1e-6);

            let r_shot = (wb_b / wb_r.max(1e-6)).max(1e-6);

            // Along reciprocal temperature locus (m = 1/T)
            let m_inv1 = 1.0 / 2856.0;
            let m_inv2 = 1.0 / 6504.0;

            let w = if (r2 - r1).abs() > 1e-5 {
                ((r_shot - r1) / (r2 - r1)).clamp(0.0, 1.0)
            } else {
                0.5
            };

            let m_inv = m_inv1 + w * (m_inv2 - m_inv1);
            (1.0 / m_inv).clamp(2000.0, 25000.0)
        } else {
            // Standard empirical Kelvin estimate based on neutral R/B multiplier ratio
            let r_shot = wb_b / wb_r.max(1e-6);
            (5500.0 * (r_shot / 0.75).powf(0.85)).clamp(2000.0, 25000.0)
        }
    }

    /// Computes the normalized camera Sensor to Linear Rec.2020 3x3 matrix interpolated
    /// for the specified Kelvin temperature along the Planckian reciprocal temperature locus.
    /// Preserves neutral axis [1, 1, 1] per Adobe DNG / First Proof color science.
    pub fn sensor_to_rec2020_at(&self, temp_kelvin: f32) -> [[f32; 3]; 3] {
        let (cm1, cm2) = if let (Some(m_tungsten), Some(m_daylight)) = (self.color_matrix_tungsten, self.color_matrix) {
            (m_tungsten, m_daylight)
        } else if let Some(m_daylight) = self.color_matrix {
            (m_daylight, m_daylight)
        } else {
            // Built-in calibrated fallback ColorMatrix (XYZ D50 -> Sensor RGB)
            let m = self.camera_model.to_lowercase();
            if m.contains("canon") || m.contains("eos") {
                (
                    [
                        [ 0.7428, -0.1897, -0.0491],
                        [-0.3505,  1.0963,  0.2929],
                        [-0.0337,  0.1242,  0.6413],
                    ],
                    [
                        [ 0.6719, -0.0994, -0.0925],
                        [-0.4408,  1.2426,  0.2211],
                        [-0.0887,  0.2129,  0.6051],
                    ],
                )
            } else if m.contains("sony") || m.contains("ilce") || m.contains("alpha") {
                (
                    [
                        [ 0.7483, -0.0851, -0.0534],
                        [-0.4109,  1.2392,  0.1971],
                        [-0.0701,  0.1884,  0.7071],
                    ],
                    [
                        [ 0.6722,  0.0821,  0.1793],
                        [ 0.2514,  0.6542,  0.0944],
                        [-0.0084, -0.0782,  0.9118],
                    ],
                )
            } else if m.contains("nikon") {
                (
                    [
                        [ 0.7712, -0.0984, -0.0512],
                        [-0.4011,  1.2281,  0.1974],
                        [-0.0742,  0.2014,  0.7182],
                    ],
                    [
                        [ 0.6881,  0.0714,  0.1892],
                        [ 0.2612,  0.6481,  0.0907],
                        [-0.0091, -0.0811,  0.9152],
                    ],
                )
            } else {
                (
                    [
                        [ 0.7428, -0.1897, -0.0491],
                        [-0.3505,  1.0963,  0.2929],
                        [-0.0337,  0.1242,  0.6413],
                    ],
                    [
                        [ 0.6719, -0.0994, -0.0925],
                        [-0.4408,  1.2426,  0.2211],
                        [-0.0887,  0.2129,  0.6051],
                    ],
                )
            }
        };

        let t1 = 2856.0f32;
        let t2 = 6504.0f32;
        let m_inv1 = 1.0 / t1;
        let m_inv2 = 1.0 / t2;
        let m_inv = 1.0 / temp_kelvin.clamp(2000.0, 50000.0);
        let w = ((m_inv - m_inv1) / (m_inv2 - m_inv1)).clamp(0.0, 1.0);

        let mut cm = [[0.0f32; 3]; 3];
        for r in 0..3 {
            for c in 0..3 {
                cm[r][c] = (1.0 - w) * cm1[r][c] + w * cm2[r][c];
            }
        }

        let xyz2cam = glam::Mat3::from_cols_array(&[
            cm[0][0], cm[1][0], cm[2][0],
            cm[0][1], cm[1][1], cm[2][1],
            cm[0][2], cm[1][2], cm[2][2],
        ]);

        let rec2020_to_xyz = glam::Mat3::from_cols_array(&[
            0.6369580, 0.2627002, 0.0000000,
            0.1446169, 0.6779981, 0.0280727,
            0.1688810, 0.0593017, 1.0609851,
        ]);

        let rec2cam_raw = xyz2cam * rec2020_to_xyz;

        let r0 = rec2cam_raw.row(0);
        let r1 = rec2cam_raw.row(1);
        let r2 = rec2cam_raw.row(2);

        let n0 = (r0.x + r0.y + r0.z).max(1e-6);
        let n1 = (r1.x + r1.y + r1.z).max(1e-6);
        let n2 = (r2.x + r2.y + r2.z).max(1e-6);

        let rec2cam = glam::Mat3::from_cols_array(&[
            r0.x / n0, r1.x / n1, r2.x / n2,
            r0.y / n0, r1.y / n1, r2.y / n2,
            r0.z / n0, r1.z / n1, r2.z / n2,
        ]);

        let sensor_to_rec = rec2cam.inverse();

        [
            [sensor_to_rec.row(0).x, sensor_to_rec.row(0).y, sensor_to_rec.row(0).z],
            [sensor_to_rec.row(1).x, sensor_to_rec.row(1).y, sensor_to_rec.row(1).z],
            [sensor_to_rec.row(2).x, sensor_to_rec.row(2).y, sensor_to_rec.row(2).z],
        ]
    }

    /// Computes the normalized camera Sensor to Linear Rec.2020 3x3 matrix for the photo's as-shot Kelvin.
    /// Preserves neutral axis [1, 1, 1] per Adobe DNG / First Proof color science.
    pub fn compute_sensor_to_rec2020(&self) -> [[f32; 3]; 3] {
        self.sensor_to_rec2020_at(self.as_shot_temperature())
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
        let baseline_gain = 2.0f32.powf(self.baseline_exposure);
        let m00 = sensor_to_rec2020[0][0] * baseline_gain;
        let m01 = sensor_to_rec2020[0][1] * baseline_gain;
        let m02 = sensor_to_rec2020[0][2] * baseline_gain;
        let m10 = sensor_to_rec2020[1][0] * baseline_gain;
        let m11 = sensor_to_rec2020[1][1] * baseline_gain;
        let m12 = sensor_to_rec2020[1][2] * baseline_gain;
        let m20 = sensor_to_rec2020[2][0] * baseline_gain;
        let m21 = sensor_to_rec2020[2][1] * baseline_gain;
        let m22 = sensor_to_rec2020[2][2] * baseline_gain;

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

                            let (r_wb, g_wb, b_wb) = blend_sensor_highlights(r_norm, g_norm, b_norm, wb_r, wb_b);

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

                            let (r_wb, g_wb, b_wb) = blend_sensor_highlights(r_norm, g_norm, b_norm, wb_r, wb_b);

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

                            let (r_wb, g_wb, b_wb) = blend_sensor_highlights(r_norm, g_norm, b_norm, wb_r, wb_b);

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

                            let (r_wb, g_wb, b_wb) = blend_sensor_highlights(r_norm, g_norm, b_norm, wb_r, wb_b);

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

    /// Renders a full-resolution edge-directed demosaiced 16-bit float (f16) RGBA buffer in Linear Rec.2020 space.
    /// Provides 1:1 sensor-pixel resolution for full-res export pipelines across native macOS and WebAssembly.
    pub fn to_full_demosaiced_linear_f16(&self) -> (Vec<u16>, u32, u32) {
        let active_w = self.active_width;
        let active_h = self.active_height;
        if active_w == 0 || active_h == 0 {
            return (Vec::new(), 0, 0);
        }

        let full_w = self.full_width;
        let full_h = self.full_height;
        let sensor = &self.sensor_data;
        if sensor.is_empty() || full_w == 0 || full_h == 0 {
            return (Vec::new(), 0, 0);
        }

        let (wb_r, wb_b) = self.as_shot_wb();
        let sensor_to_rec2020 = self.compute_sensor_to_rec2020();
        let baseline_gain = 2.0f32.powf(self.baseline_exposure);
        let m00 = sensor_to_rec2020[0][0] * baseline_gain;
        let m01 = sensor_to_rec2020[0][1] * baseline_gain;
        let m02 = sensor_to_rec2020[0][2] * baseline_gain;
        let m10 = sensor_to_rec2020[1][0] * baseline_gain;
        let m11 = sensor_to_rec2020[1][1] * baseline_gain;
        let m12 = sensor_to_rec2020[1][2] * baseline_gain;
        let m20 = sensor_to_rec2020[2][0] * baseline_gain;
        let m21 = sensor_to_rec2020[2][1] * baseline_gain;
        let m22 = sensor_to_rec2020[2][2] * baseline_gain;

        let black = self.black_levels[0].max(0.0);
        let white = self.white_levels[0].max(black + 1.0);
        let scale = 1.0 / (white - black);
        let a16 = f32_to_f16(1.0);

        // Site type lookup table per pattern:
        // 0: R site (Red measured)
        // 1: B site (Blue measured)
        // 2: G site on R row (R horizontal, B vertical)
        // 3: G site on B row (B horizontal, R vertical)
        let site_map: [[u8; 2]; 2] = match self.cfa_pattern {
            CfaPattern::Rggb => [[0, 2], [3, 1]],
            CfaPattern::Bggr => [[1, 3], [2, 0]],
            CfaPattern::Gbrg => [[3, 1], [0, 2]],
            CfaPattern::Grbg => [[2, 0], [1, 3]],
            _ => [[0, 2], [3, 1]],
        };

        let sample = |px: isize, py: isize| -> f32 {
            let cx = px.clamp(0, (full_w - 1) as isize) as usize;
            let cy = py.clamp(0, (full_h - 1) as isize) as usize;
            let val = sensor.get(cy * full_w + cx).copied().unwrap_or(0) as f32;
            ((val - black) * scale).clamp(0.0, 1.0)
        };

        // Malvar-He-Cutler (2004) 5x5 gradient-corrected linear demosaicing filters:
        // GR_GB: Green at Red or Blue site (using discrete Laplacian of measured channel)
        let g_at_rb = |px: isize, py: isize| -> f32 {
            (
                4.0 * sample(px, py)
                + 2.0 * (sample(px - 1, py) + sample(px + 1, py) + sample(px, py - 1) + sample(px, py + 1))
                - (sample(px - 2, py) + sample(px + 2, py) + sample(px, py - 2) + sample(px, py + 2))
            ) * 0.125
        };

        // Horizontal: R (or B) at Green site with horizontal R (or B) neighbors
        let c_at_g_horiz = |px: isize, py: isize| -> f32 {
            (
                5.0 * sample(px, py)
                + 4.0 * (sample(px - 1, py) + sample(px + 1, py))
                - (sample(px - 2, py) + sample(px + 2, py))
                - (sample(px - 1, py - 1) + sample(px + 1, py - 1) + sample(px - 1, py + 1) + sample(px + 1, py + 1))
                + 0.5 * (sample(px, py - 2) + sample(px, py + 2))
            ) * 0.125
        };

        // Vertical: R (or B) at Green site with vertical R (or B) neighbors (transpose of horizontal)
        let c_at_g_vert = |px: isize, py: isize| -> f32 {
            (
                5.0 * sample(px, py)
                + 4.0 * (sample(px, py - 1) + sample(px, py + 1))
                - (sample(px - 2, py) + sample(px + 2, py))
                - (sample(px - 1, py - 1) + sample(px + 1, py - 1) + sample(px - 1, py + 1) + sample(px + 1, py + 1))
                + 0.5 * (sample(px - 2, py) + sample(px + 2, py))
            ) * 0.125
        };

        // Opposite: R at Blue site, or B at Red site
        let c_at_opp = |px: isize, py: isize| -> f32 {
            (
                6.0 * sample(px, py)
                + 2.0 * (sample(px - 1, py - 1) + sample(px + 1, py - 1) + sample(px - 1, py + 1) + sample(px + 1, py + 1))
                - 1.5 * (sample(px - 2, py) + sample(px + 2, py) + sample(px, py - 2) + sample(px, py + 2))
            ) * 0.125
        };

        let mut u16_pixels = vec![0u16; active_w * active_h * 4];

        for y in 0..active_h {
            let sy = self.active_y + y;
            let dst_row_off = y * active_w * 4;

            for x in 0..active_w {
                let sx = self.active_x + x;

                let site = site_map[sy % 2][sx % 2];
                let isx = sx as isize;
                let isy = sy as isize;
                let (r, g, b) = match site {
                    0 => {
                        // R site: Red is measured
                        let r = sample(isx, isy);
                        let g = g_at_rb(isx, isy);
                        let b = c_at_opp(isx, isy);
                        (r.max(0.0), g.max(0.0), b.max(0.0))
                    }
                    1 => {
                        // B site: Blue is measured
                        let b = sample(isx, isy);
                        let g = g_at_rb(isx, isy);
                        let r = c_at_opp(isx, isy);
                        (r.max(0.0), g.max(0.0), b.max(0.0))
                    }
                    2 => {
                        // G site on R row: R is horizontal, B is vertical
                        let g = sample(isx, isy);
                        let r = c_at_g_horiz(isx, isy);
                        let b = c_at_g_vert(isx, isy);
                        (r.max(0.0), g.max(0.0), b.max(0.0))
                    }
                    _ => {
                        // G site on B row: B is horizontal, R is vertical
                        let g = sample(isx, isy);
                        let b = c_at_g_horiz(isx, isy);
                        let r = c_at_g_vert(isx, isy);
                        (r.max(0.0), g.max(0.0), b.max(0.0))
                    }
                };

                // As-shot white balance with sensitometric highlight roll-off
                let (r_wb, g_wb, b_wb) = blend_sensor_highlights(r, g, b, wb_r, wb_b);

                // Sensor to Linear Rec.2020 matrix transform
                let r_lin = (m00 * r_wb + m01 * g_wb + m02 * b_wb).max(0.0);
                let g_lin = (m10 * r_wb + m11 * g_wb + m12 * b_wb).max(0.0);
                let b_lin = (m20 * r_wb + m21 * g_wb + m22 * b_wb).max(0.0);

                let dst_idx = dst_row_off + x * 4;
                u16_pixels[dst_idx] = f32_to_f16(r_lin);
                u16_pixels[dst_idx + 1] = f32_to_f16(g_lin);
                u16_pixels[dst_idx + 2] = f32_to_f16(b_lin);
                u16_pixels[dst_idx + 3] = a16;
            }
        }

        (u16_pixels, active_w as u32, active_h as u32)
    }
}
