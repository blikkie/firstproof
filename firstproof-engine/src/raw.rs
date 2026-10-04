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
/// along a smooth C^1 Hermite curve anchored to the sensor's linear response limit while preserving full highlight luminance headroom.
#[inline]
pub fn blend_sensor_highlights(
    r: f32,
    g: f32,
    b: f32,
    wb_r: f32,
    wb_b: f32,
    linear_response_limit: f32,
) -> (f32, f32, f32) {
    let limit = linear_response_limit.clamp(0.90, 0.99);
    let s_max = r.max(g).max(b);
    let p_r = r * wb_r;
    let p_g = g;
    let p_b = b * wb_b;
    if s_max > limit {
        let t = ((s_max - limit) / (1.0 - limit)).clamp(0.0, 1.0);
        let alpha = t * t * (3.0 - 2.0 * t);
        let l_target = p_r.max(p_g).max(p_b);
        (
            (1.0 - alpha) * p_r + alpha * l_target,
            (1.0 - alpha) * p_g + alpha * l_target,
            (1.0 - alpha) * p_b + alpha * l_target,
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
    pub forward_matrix_tungsten: Option<[[f32; 3]; 3]>,
    pub forward_matrix_daylight: Option<[[f32; 3]; 3]>,
    pub camera_model: String,
    pub baseline_exposure: f32,
    pub linear_response_limit: f32,
    pub noise_profile: Option<[f64; 2]>,
    pub sensor_data: Vec<u16>,
}

/// Extracts Tag 50728 / 50730 (BaselineExposure) from a DNG / TIFF byte buffer (IFD0).
/// Returns linear exposure offset in EV (e.g. +0.831169).
pub fn read_dng_baseline_exposure(bytes: &[u8]) -> Option<f32> {
    crate::container::RawContainerManifest::parse(bytes).and_then(|m| m.baseline_exposure)
}

/// Resolves calibrated camera baseline exposure compensation (in EV).
/// First attempts to read embedded DNG BaselineExposure.
/// If not present (e.g. proprietary Canon CR2/CR3, Sony ARW, Nikon NEF),
/// falls back to the authoritative CameraCalibrationRegistry.
pub fn resolve_baseline_exposure(bytes: &[u8], camera_model: &str) -> f32 {
    if let Some(exp) = read_dng_baseline_exposure(bytes) {
        return exp;
    }
    let cal = crate::calibration::CameraCalibrationRegistry::lookup_or_fallback("", camera_model);
    cal.baseline_exposure
}


impl RawSensorFrame {
    /// Decodes a `RawSensorFrame` from a decoded `rawler::RawImage` structure and an optional header/full-file byte slice.
    pub fn decode_from_raw_image(raw: rawler::RawImage, header_or_full_slice: &[u8]) -> Option<Self> {
        let (crop_top, crop_left, active_w, active_h) = if let Some(crop) = raw.crop_area {
            let x = crop.p.x & !1;
            let y = crop.p.y & !1;
            let w = crop.d.w.max(2) & !1;
            let h = crop.d.h.max(2) & !1;
            (y, x, w, h)
        } else {
            (0, 0, raw.width & !1, raw.height & !1)
        };

        let c00 = raw.camera.cfa.color_at(0, 0);
        let c01 = raw.camera.cfa.color_at(0, 1);
        let c10 = raw.camera.cfa.color_at(1, 0);
        let c11 = raw.camera.cfa.color_at(1, 1);

        let cfa_pattern = match (c00, c01, c10, c11) {
            (0, 1, 1, 2) => CfaPattern::Rggb,
            (2, 1, 1, 0) => CfaPattern::Bggr,
            (1, 2, 0, 1) => CfaPattern::Gbrg,
            (1, 0, 2, 1) => CfaPattern::Grbg,
            _ => CfaPattern::Rggb,
        };

        let find_matrix = |pattern: &str| -> Option<[[f32; 3]; 3]> {
            raw.color_matrix
                .iter()
                .find(|(k, _)| format!("{:?}", k).to_lowercase() == pattern)
                .and_then(|(_, v)| {
                    if v.len() >= 9 {
                        Some([
                            [v[0], v[1], v[2]],
                            [v[3], v[4], v[5]],
                            [v[6], v[7], v[8]],
                        ])
                    } else {
                        None
                    }
                })
        };

        let manifest = crate::container::RawContainerManifest::parse(header_or_full_slice);
        let camera_make = raw.clean_make.clone();
        let camera_model = format!("{} {}", raw.clean_make, raw.clean_model)
            .trim()
            .to_string();

        let cal = crate::calibration::CameraCalibrationRegistry::lookup_or_fallback(&camera_make, &camera_model);

        let color_matrix_tungsten = manifest
            .as_ref()
            .and_then(|m| m.color_matrix_1)
            .or_else(|| find_matrix("a"))
            .or(cal.color_matrix_tungsten);

        let color_matrix = manifest
            .as_ref()
            .and_then(|m| m.color_matrix_2)
            .or_else(|| find_matrix("d65"))
            .or_else(|| find_matrix("d50"))
            .or_else(|| {
                raw.color_matrix.iter().next().and_then(|(_, v)| {
                    if v.len() >= 9 {
                        Some([
                            [v[0], v[1], v[2]],
                            [v[3], v[4], v[5]],
                            [v[6], v[7], v[8]],
                        ])
                    } else {
                        None
                    }
                })
            })
            .or(cal.color_matrix_daylight);

        let black_levels = manifest
            .as_ref()
            .and_then(|m| m.black_levels)
            .unwrap_or_else(|| {
                if raw.blacklevel.levels.is_empty() {
                    cal.black_levels
                } else if raw.blacklevel.levels.len() == 1 {
                    let r = &raw.blacklevel.levels[0];
                    let val = if r.d != 0 {
                        r.n as f32 / r.d as f32
                    } else {
                        r.n as f32
                    };
                    [val; 4]
                } else {
                    let mut b = cal.black_levels;
                    for i in 0..4.min(raw.blacklevel.levels.len()) {
                        let r = &raw.blacklevel.levels[i];
                        b[i] = if r.d != 0 {
                            r.n as f32 / r.d as f32
                        } else {
                            r.n as f32
                        };
                    }
                    b
                }
            });

        let white_levels = manifest
            .as_ref()
            .and_then(|m| m.white_levels)
            .unwrap_or_else(|| {
                if raw.whitelevel.0.is_empty() {
                    cal.white_levels
                } else if raw.whitelevel.0.len() == 1 {
                    [raw.whitelevel.0[0] as f32; 4]
                } else {
                    let mut w = cal.white_levels;
                    for i in 0..4.min(raw.whitelevel.0.len()) {
                        w[i] = raw.whitelevel.0[i] as f32;
                    }
                    w
                }
            });

        let wb = raw.wb_coeffs;
        let wb_coeffs = [wb[0], wb[1], wb[2], wb[3]];

        let baseline_exposure = manifest
            .as_ref()
            .and_then(|m| m.baseline_exposure)
            .unwrap_or(cal.baseline_exposure);

        let linear_response_limit = manifest
            .as_ref()
            .and_then(|m| m.linear_response_limit)
            .unwrap_or(cal.linear_response_limit);

        let forward_matrix_tungsten = manifest
            .as_ref()
            .and_then(|m| m.forward_matrix_1)
            .or(cal.forward_matrix_tungsten);

        let forward_matrix_daylight = manifest
            .as_ref()
            .and_then(|m| m.forward_matrix_2)
            .or(cal.forward_matrix_daylight);

        let noise_profile = manifest
            .as_ref()
            .and_then(|m| m.noise_profile.as_ref())
            .and_then(|np| {
                if np.len() >= 2 {
                    Some([np[0], np[1]])
                } else {
                    None
                }
            });

        let sensor_data = match raw.data {
            rawler::RawImageData::Integer(cfa) => cfa,
            _ => return None,
        };

        let mut frame = RawSensorFrame {
            full_width: raw.width,
            full_height: raw.height,
            active_x: crop_left,
            active_y: crop_top,
            active_width: active_w,
            active_height: active_h,
            cfa_pattern,
            black_levels,
            white_levels,
            wb_coeffs,
            color_matrix,
            color_matrix_tungsten,
            forward_matrix_tungsten,
            forward_matrix_daylight,
            camera_model,
            baseline_exposure,
            linear_response_limit,
            noise_profile,
            sensor_data,
        };

        if let Some(op_bytes) = manifest.as_ref().and_then(|m| m.opcode_list_1.as_ref()) {
            if let Ok(opcodes) = crate::container::DngOpcodeList::parse(op_bytes) {
                frame.apply_opcode_list_1(&opcodes);
            }
        }

        Some(frame)
    }

    /// Applies DNG OpcodeList1 (pre-demosaic / CFA raw corrections such as bad pixel repair and sensor GainMap).
    pub fn apply_opcode_list_1(&mut self, opcodes: &crate::container::DngOpcodeList) {
        crate::container::apply_opcode_list_1_to_raw(
            &mut self.sensor_data,
            self.full_width,
            self.full_height,
            self.black_levels,
            self.white_levels,
            opcodes,
        );
    }

    /// Decodes a `RawSensorFrame` directly from an in-memory RAW file byte slice using rawler.
    pub fn decode_from_bytes(slice: &[u8]) -> Option<Self> {
        let rawfile = rawler::rawsource::RawSource::new_from_slice(slice);
        let params = rawler::decoders::RawDecodeParams::default();
        let raw = rawler::decode(&rawfile, &params).ok()?;
        Self::decode_from_raw_image(raw, slice)
    }

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
        let cfa_idx = (y % 2) * 2 + (x % 2);
        let black = self.black_levels[cfa_idx].max(0.0);
        let white = self.white_levels[cfa_idx].max(black + 1.0);
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
        let cal = crate::calibration::CameraCalibrationRegistry::lookup_or_fallback("", &self.camera_model);
        let fwd_matrices = if let (Some(fm1), Some(fm2)) = (self.forward_matrix_tungsten, self.forward_matrix_daylight) {
            Some((fm1, fm2))
        } else if let (Some(fm1), Some(fm2)) = (cal.forward_matrix_tungsten, cal.forward_matrix_daylight) {
            Some((fm1, fm2))
        } else {
            None
        };



        if let Some((fm1, fm2)) = fwd_matrices {
            let t1 = 2856.0f32;
            let t2 = 6504.0f32;
            let m_inv1 = 1.0 / t1;
            let m_inv2 = 1.0 / t2;
            let m_inv = 1.0 / temp_kelvin.clamp(2000.0, 50000.0);
            let w = ((m_inv - m_inv1) / (m_inv2 - m_inv1)).clamp(0.0, 1.0);

            let mut fm = [[0.0f32; 3]; 3];
            for r in 0..3 {
                for c in 0..3 {
                    fm[r][c] = (1.0 - w) * fm1[r][c] + w * fm2[r][c];
                }
            }

            // Universal XYZ D50 -> CAT16 -> Linear Rec.2020 matrix:
            let d50_to_rec2020 = glam::Mat3::from_cols_array(&[
                1.700_593, -0.66840535, 0.017304685,
                -0.43057728, 1.6541731, -0.029557694,
                -0.25366065, -0.01167498, 1.2275668,
            ]);
            let fm_mat = glam::Mat3::from_cols_array(&[
                fm[0][0], fm[1][0], fm[2][0],
                fm[0][1], fm[1][1], fm[2][1],
                fm[0][2], fm[1][2], fm[2][2],
            ]);
            let sensor_to_rec = d50_to_rec2020 * fm_mat;
            return [
                [sensor_to_rec.row(0).x, sensor_to_rec.row(0).y, sensor_to_rec.row(0).z],
                [sensor_to_rec.row(1).x, sensor_to_rec.row(1).y, sensor_to_rec.row(1).z],
                [sensor_to_rec.row(2).x, sensor_to_rec.row(2).y, sensor_to_rec.row(2).z],
            ];
        }

        let (cm1, cm2) = if let (Some(m_tungsten), Some(m_daylight)) = (self.color_matrix_tungsten, self.color_matrix) {
            (m_tungsten, m_daylight)
        } else if let Some(m_daylight) = self.color_matrix {
            (m_daylight, m_daylight)
        } else {
            (
                cal.color_matrix_tungsten.unwrap_or(crate::calibration::registry::CANON_COLOR_MATRIX_TUNGSTEN),
                cal.color_matrix_daylight.unwrap_or(crate::calibration::registry::CANON_COLOR_MATRIX_DAYLIGHT),
            )
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
            0.636_958, 0.2627002, 0.0000000,
            0.1446169, 0.6779981, 0.0280727,
            0.168_881, 0.0593017, 1.0609851,
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

        let mut scales = [1.0f32; 4];
        let mut blacks = [0.0f32; 4];
        for i in 0..4 {
            let b = self.black_levels[i].max(0.0);
            let w = self.white_levels[i].max(b + 1.0);
            blacks[i] = b;
            scales[i] = 1.0 / (w - b);
        }

        let a16 = f32_to_f16(1.0);

        let mut binned_rgb = Vec::with_capacity((bin_w * bin_h) as usize);
        let sensor = &self.sensor_data;
        let full_w = self.full_width;
        let row_len = (bin_w as usize) * 2;

        for by in 0..(bin_h as usize) {
            let sy = self.active_y + by * 2;

            let row0_start = sy * full_w + self.active_x;
            let row1_start = (sy + 1) * full_w + self.active_x;

            if row0_start + row_len <= sensor.len() && row1_start + row_len <= sensor.len() {
                let row0 = &sensor[row0_start..row0_start + row_len];
                let row1 = &sensor[row1_start..row1_start + row_len];

                for bx in 0..(bin_w as usize) {
                    let p00 = row0[bx * 2] as f32;
                    let p01 = row0[bx * 2 + 1] as f32;
                    let p10 = row1[bx * 2] as f32;
                    let p11 = row1[bx * 2 + 1] as f32;

                    let norm_00 = ((p00 - blacks[0]) * scales[0]).clamp(0.0, 1.0);
                    let norm_01 = ((p01 - blacks[1]) * scales[1]).clamp(0.0, 1.0);
                    let norm_10 = ((p10 - blacks[2]) * scales[2]).clamp(0.0, 1.0);
                    let norm_11 = ((p11 - blacks[3]) * scales[3]).clamp(0.0, 1.0);

                    let (r_norm, g_norm, b_norm) = match self.cfa_pattern {
                        CfaPattern::Rggb => (norm_00, (norm_01 + norm_10) * 0.5, norm_11),
                        CfaPattern::Bggr => (norm_11, (norm_01 + norm_10) * 0.5, norm_00),
                        CfaPattern::Gbrg => (norm_10, (norm_00 + norm_11) * 0.5, norm_01),
                        CfaPattern::Grbg => (norm_01, (norm_00 + norm_11) * 0.5, norm_10),
                        _ => (norm_00, (norm_01 + norm_10) * 0.5, norm_11),
                    };

                    let (r_wb, g_wb, b_wb) =
                        blend_sensor_highlights(r_norm, g_norm, b_norm, wb_r, wb_b, self.linear_response_limit);
                    binned_rgb.push([r_wb, g_wb, b_wb]);
                }
            } else {
                for _ in 0..(bin_w as usize) {
                    binned_rgb.push([0.0, 0.0, 0.0]);
                }
            }
        }

        // Apply 2 passes of chromatic difference median filtering on the binned frame.
        // Matches the 2-pass chromatic median filtering in full demosaic export (Story 5.8).
        // Replaces isolated single-pixel chroma speckles with the local chromatic median
        // while leaving optical luminance G (high-frequency scene sharpness) 100% untouched.
        apply_chromatic_median_filter_rgb(&mut binned_rgb, bin_w as usize, bin_h as usize, 2);

        let mut u16_pixels = vec![0u16; (bin_w * bin_h * 4) as usize];
        for (i, &[r_wb, g_wb, b_wb]) in binned_rgb.iter().enumerate() {
            let r_lin = (m00 * r_wb + m01 * g_wb + m02 * b_wb).max(0.0);
            let g_lin = (m10 * r_wb + m11 * g_wb + m12 * b_wb).max(0.0);
            let b_lin = (m20 * r_wb + m21 * g_wb + m22 * b_wb).max(0.0);

            let dst_idx = i * 4;
            u16_pixels[dst_idx] = f32_to_f16(r_lin);
            u16_pixels[dst_idx + 1] = f32_to_f16(g_lin);
            u16_pixels[dst_idx + 2] = f32_to_f16(b_lin);
            u16_pixels[dst_idx + 3] = a16;
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
        let mut scales = [1.0f32; 4];
        let mut blacks = [0.0f32; 4];
        for i in 0..4 {
            let b = self.black_levels[i].max(0.0);
            let w = self.white_levels[i].max(b + 1.0);
            blacks[i] = b;
            scales[i] = 1.0 / (w - b);
        }

        let mut norm_pixels = Vec::with_capacity(full_w * full_h);
        for y in 0..full_h {
            let row_off = y * full_w;
            let y_mod = (y % 2) * 2;
            for x in 0..full_w {
                let cfa_idx = y_mod + (x % 2);
                let v = sensor[row_off + x] as f32;
                norm_pixels.push(((v - blacks[cfa_idx]) * scales[cfa_idx]).clamp(0.0, 1.0));
            }
        }

        // Physical Pre-WB Sensor Outlier Suppression:
        // Clamps isolated single-photosite readout noise spikes and hot-pixels on the raw CFA grid
        // BEFORE white balance multipliers and spatial interpolation kernels are evaluated.
        suppress_sensor_read_noise_outliers(
            &mut norm_pixels,
            full_w,
            full_h,
            self.cfa_pattern,
            self.noise_profile,
        );

        // Pre-demosaicing White Balance (standard in dcraw / LibRaw):
        // Applying white balance multipliers directly to CFA photosites before demosaicing
        // ensures cross-channel edge gradients (R vs G, B vs G) and Laplacian high-frequency
        // transfers operate in calibrated optical units, preventing asymmetric noise amplification.
        for y in 0..full_h {
            let row_offset = y * full_w;
            for x in 0..full_w {
                let c = self.color_at(y, x);
                if c == CFA_COLOR_RED {
                    norm_pixels[row_offset + x] *= wb_r;
                } else if c == CFA_COLOR_BLUE {
                    norm_pixels[row_offset + x] *= wb_b;
                }
            }
        }

        let mut demosaiced_rgb = soft_adaptive_demosaic_rgb(
            &norm_pixels,
            full_w,
            full_h,
            self.active_x,
            self.active_y,
            active_w,
            active_h,
            self,
        );

        apply_chromatic_median_filter_rgb(&mut demosaiced_rgb, active_w, active_h, 2);

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

        let a16 = f32_to_f16(1.0);
        let mut u16_pixels = vec![0u16; active_w * active_h * 4];

        for y in 0..active_h {
            let src_row = y * active_w;
            let dst_row = y * active_w * 4;
            for x in 0..active_w {
                let [r_wb, g_wb, b_wb] = demosaiced_rgb[src_row + x];
                let r_raw = if wb_r > 0.001 { r_wb / wb_r } else { r_wb };
                let g_raw = g_wb;
                let b_raw = if wb_b > 0.001 { b_wb / wb_b } else { b_wb };
                let (r_clean, g_clean, b_clean) =
                    blend_sensor_highlights(r_raw, g_raw, b_raw, wb_r, wb_b, self.linear_response_limit);
                let r_lin = (m00 * r_clean + m01 * g_clean + m02 * b_clean).max(0.0);
                let g_lin = (m10 * r_clean + m11 * g_clean + m12 * b_clean).max(0.0);
                let b_lin = (m20 * r_clean + m21 * g_clean + m22 * b_clean).max(0.0);

                let dst_idx = dst_row + x * 4;
                u16_pixels[dst_idx] = f32_to_f16(r_lin);
                u16_pixels[dst_idx + 1] = f32_to_f16(g_lin);
                u16_pixels[dst_idx + 2] = f32_to_f16(b_lin);
                u16_pixels[dst_idx + 3] = a16;
            }
        }

        (u16_pixels, active_w as u32, active_h as u32)
    }
}

/// Reconstructs the full color RGB buffer using soft-adaptive directional demosaicing.
///
/// Blends horizontal and vertical estimates smoothly via C^1 Hermite weights,
/// permanently eliminating "labyrinths", maze patterns, and checkerboard artifacts across smooth skin,
/// out-of-focus backgrounds, and low-contrast gradients while preserving 100% of optical edge acutance.
pub fn soft_adaptive_demosaic_rgb(
    norm_pixels: &[f32],
    full_w: usize,
    full_h: usize,
    active_x: usize,
    active_y: usize,
    active_w: usize,
    active_h: usize,
    frame: &RawSensorFrame,
) -> Vec<[f32; 3]> {
    let sample = |fy: usize, fx: usize| -> f32 {
        let y_clamped = fy.clamp(0, full_h - 1);
        let x_clamped = fx.clamp(0, full_w - 1);
        norm_pixels[y_clamped * full_w + x_clamped]
    };

    // Step 1: Reconstruct the full Green plane across the active image (with a 2-pixel apron for boundary safety)
    let apron_w = active_w + 4;
    let apron_h = active_h + 4;
    let mut green_plane = vec![0.0f32; apron_w * apron_h];

    for ay in 0..apron_h {
        let fy = (active_y as isize + ay as isize - 2).clamp(2, (full_h - 3) as isize) as usize;
        let g_row_start = ay * apron_w;
        for ax in 0..apron_w {
            let fx = (active_x as isize + ax as isize - 2).clamp(2, (full_w - 3) as isize) as usize;
            let color = frame.color_at(fy, fx);
            if color == CFA_COLOR_GREEN {
                green_plane[g_row_start + ax] = sample(fy, fx);
            } else {
                let c = sample(fy, fx);
                let g_n = sample(fy - 1, fx);
                let g_s = sample(fy + 1, fx);
                let g_w = sample(fy, fx - 1);
                let g_e = sample(fy, fx + 1);

                let c_n2 = sample(fy - 2, fx);
                let c_s2 = sample(fy + 2, fx);
                let c_w2 = sample(fy, fx - 2);
                let c_e2 = sample(fy, fx + 2);

                let delta_h = (g_w - g_e).abs() + (2.0 * c - c_w2 - c_e2).abs();
                let delta_v = (g_n - g_s).abs() + (2.0 * c - c_n2 - c_s2).abs();

                let g_h = 0.5 * (g_w + g_e) + 0.25 * (2.0 * c - c_w2 - c_e2);
                let g_v = 0.5 * (g_n + g_s) + 0.25 * (2.0 * c - c_n2 - c_s2);

                // Continuous soft blending (C^1 continuous Hermite):
                let diff = delta_h - delta_v;
                let sum = delta_h + delta_v + 1e-5;
                let t = (diff / sum).clamp(-1.0, 1.0);
                let u = 0.5 + 0.5 * t;
                let w_v = u * u * (3.0 - 2.0 * u);
                let w_h = 1.0 - w_v;

                green_plane[g_row_start + ax] = (w_h * g_h + w_v * g_v).max(0.0);
            }
        }
    }

    let g_sample = |ay: usize, ax: usize| -> f32 {
        green_plane[ay * apron_w + ax]
    };

    // Step 2: Interpolate Red and Blue using smooth color differences
    let mut out_rgb = vec![[0.0f32; 3]; active_w * active_h];

    for y in 0..active_h {
        let ay = y + 2;
        let fy = active_y + y;
        let out_row_start = y * active_w;

        for x in 0..active_w {
            let ax = x + 2;
            let fx = active_x + x;
            let color = frame.color_at(fy, fx);
            let g = g_sample(ay, ax);

            let r: f32;
            let b: f32;

            if color == CFA_COLOR_RED {
                r = sample(fy, fx);
                let d_nw = sample(fy - 1, fx - 1) - g_sample(ay - 1, ax - 1);
                let d_ne = sample(fy - 1, fx + 1) - g_sample(ay - 1, ax + 1);
                let d_sw = sample(fy + 1, fx - 1) - g_sample(ay + 1, ax - 1);
                let d_se = sample(fy + 1, fx + 1) - g_sample(ay + 1, ax + 1);

                let grad_d1 = (sample(fy - 1, fx - 1) - sample(fy + 1, fx + 1)).abs() + (g_sample(ay - 1, ax - 1) - g_sample(ay + 1, ax + 1)).abs();
                let grad_d2 = (sample(fy - 1, fx + 1) - sample(fy + 1, fx - 1)).abs() + (g_sample(ay - 1, ax + 1) - g_sample(ay + 1, ax - 1)).abs();

                let diff_d = grad_d1 - grad_d2;
                let sum_d = grad_d1 + grad_d2 + 1e-5;
                let t_d = (diff_d / sum_d).clamp(-1.0, 1.0);
                let u_d = 0.5 + 0.5 * t_d;
                let w_d2 = u_d * u_d * (3.0 - 2.0 * u_d);
                let w_d1 = 1.0 - w_d2;

                let b_d1 = 0.5 * (d_nw + d_se);
                let b_d2 = 0.5 * (d_ne + d_sw);
                b = (g + w_d1 * b_d1 + w_d2 * b_d2).max(0.0);
            } else if color == CFA_COLOR_BLUE {
                b = sample(fy, fx);
                let d_nw = sample(fy - 1, fx - 1) - g_sample(ay - 1, ax - 1);
                let d_ne = sample(fy - 1, fx + 1) - g_sample(ay - 1, ax + 1);
                let d_sw = sample(fy + 1, fx - 1) - g_sample(ay + 1, ax - 1);
                let d_se = sample(fy + 1, fx + 1) - g_sample(ay + 1, ax + 1);

                let grad_d1 = (sample(fy - 1, fx - 1) - sample(fy + 1, fx + 1)).abs() + (g_sample(ay - 1, ax - 1) - g_sample(ay + 1, ax + 1)).abs();
                let grad_d2 = (sample(fy - 1, fx + 1) - sample(fy + 1, fx - 1)).abs() + (g_sample(ay - 1, ax + 1) - g_sample(ay + 1, ax - 1)).abs();

                let diff_d = grad_d1 - grad_d2;
                let sum_d = grad_d1 + grad_d2 + 1e-5;
                let t_d = (diff_d / sum_d).clamp(-1.0, 1.0);
                let u_d = 0.5 + 0.5 * t_d;
                let w_d2 = u_d * u_d * (3.0 - 2.0 * u_d);
                let w_d1 = 1.0 - w_d2;

                let r_d1 = 0.5 * (d_nw + d_se);
                let r_d2 = 0.5 * (d_ne + d_sw);
                r = (g + w_d1 * r_d1 + w_d2 * r_d2).max(0.0);
            } else {
                let is_r_horizontal = frame.color_at(fy, fx.saturating_sub(1)) == CFA_COLOR_RED
                    || frame.color_at(fy, fx + 1) == CFA_COLOR_RED;

                let d_w = sample(fy, fx - 1) - g_sample(ay, ax - 1);
                let d_e = sample(fy, fx + 1) - g_sample(ay, ax + 1);
                let d_n = sample(fy - 1, fx) - g_sample(ay - 1, ax);
                let d_s = sample(fy + 1, fx) - g_sample(ay + 1, ax);

                let d_h = 0.5 * (d_w + d_e);
                let d_v = 0.5 * (d_n + d_s);

                if is_r_horizontal {
                    r = (g + d_h).max(0.0);
                    b = (g + d_v).max(0.0);
                } else {
                    b = (g + d_h).max(0.0);
                    r = (g + d_v).max(0.0);
                }
            }

            out_rgb[out_row_start + x] = [r, g, b];
        }
    }

    out_rgb
}

/// Applies chromatic difference median filtering to eliminate Bayer demosaicing artifacts.
pub fn apply_chromatic_median_filter_rgb(
    pixels: &mut [[f32; 3]],
    width: usize,
    height: usize,
    passes: usize,
) {
    if passes == 0 || width < 3 || height < 3 || pixels.len() < width * height {
        return;
    }

    let mut buf = pixels.to_vec();

    for _ in 0..passes {
        for y in 0..height {
            let y_top = y.saturating_sub(1);
            let y_bot = (y + 1).min(height - 1);
            let row_top = &pixels[y_top * width..(y_top + 1) * width];
            let row_mid = &pixels[y * width..(y + 1) * width];
            let row_bot = &pixels[y_bot * width..(y_bot + 1) * width];
            let dst_offset = y * width;

            for x in 0..width {
                let x_left = x.saturating_sub(1);
                let x_right = (x + 1).min(width - 1);
                let g_mid = row_mid[x][1];

                let mut cr = [
                    row_top[x_left][0] - row_top[x_left][1],
                    row_top[x][0] - row_top[x][1],
                    row_top[x_right][0] - row_top[x_right][1],
                    row_mid[x_left][0] - row_mid[x_left][1],
                    row_mid[x][0] - g_mid,
                    row_mid[x_right][0] - row_mid[x_right][1],
                    row_bot[x_left][0] - row_bot[x_left][1],
                    row_bot[x][0] - row_bot[x][1],
                    row_bot[x_right][0] - row_bot[x_right][1],
                ];

                let mut cb = [
                    row_top[x_left][2] - row_top[x_left][1],
                    row_top[x][2] - row_top[x][1],
                    row_top[x_right][2] - row_top[x_right][1],
                    row_mid[x_left][2] - row_mid[x_left][1],
                    row_mid[x][2] - g_mid,
                    row_mid[x_right][2] - row_mid[x_right][1],
                    row_bot[x_left][2] - row_bot[x_left][1],
                    row_bot[x][2] - row_bot[x][1],
                    row_bot[x_right][2] - row_bot[x_right][1],
                ];

                cr.sort_unstable_by(|a, b| a.total_cmp(b));
                cb.sort_unstable_by(|a, b| a.total_cmp(b));

                buf[dst_offset + x] = [
                    (g_mid + cr[4]).max(0.0),
                    g_mid,
                    (g_mid + cb[4]).max(0.0),
                ];
            }
        }
        pixels.copy_from_slice(&buf);
    }
}

/// Suppresses isolated single-photosite read-noise spikes and hot-pixels on the normalized CFA grid
/// before white balance multipliers and spatial demosaicing interpolation are evaluated.
///
/// Uses the DNG specification noise variance model:
///     sigma^2(x) = S * x + O
/// Where S is shot noise slope, and O is readout noise offset variance.
///
/// Compares each photosite x to its like-colored nearest neighbors on the Bayer mosaic:
/// - Green sites: 4 diagonal neighbors (dx = +/-1, dy = +/-1) are guaranteed to be Green.
/// - Red / Blue sites: 4 cardinal neighbors (dx = +/-2, dy = 0 and dx = 0, dy = +/-2) are like-colored.
///
/// If a photosite exceeds the neighborhood maximum by more than k * sigma(x) (where k = 3.5),
/// it is clamped to v_max + k * sigma(x).
pub fn suppress_sensor_read_noise_outliers(
    pixels: &mut [f32],
    width: usize,
    height: usize,
    cfa_pattern: CfaPattern,
    noise_profile: Option<[f64; 2]>,
) {
    if width < 5 || height < 5 || pixels.len() < width * height {
        return;
    }

    // Default noise profile fallback if not specified:
    // Typical 14-bit digital camera sensor at ISO 1600 (read noise ~16 counts, full well ~15000)
    let (shot_slope, read_var) = match noise_profile {
        Some([s, o]) => (s.max(1e-7) as f32, o.max(1e-8) as f32),
        None => (0.0005f32, 1.5e-6f32),
    };

    let k = 3.5f32; // 3.5-sigma confidence interval (~99.95% single-tail)
    let orig = pixels.to_vec();

    // Determine CFA offset for site mapping
    let site_map = match cfa_pattern {
        CfaPattern::Rggb => [[0, 2], [3, 1]],
        CfaPattern::Bggr => [[1, 2], [3, 0]],
        CfaPattern::Gbrg => [[2, 1], [0, 3]],
        CfaPattern::Grbg => [[2, 0], [1, 3]],
        _ => [[0, 2], [3, 1]],
    };

    for y in 2..height.saturating_sub(2) {
        let row_off = y * width;
        let y_mod = y % 2;

        for x in 2..width.saturating_sub(2) {
            let site = site_map[y_mod][x % 2];
            let v = orig[row_off + x];

            // Only check deep shadows and midtones (v < 0.25). Highlights are photon-shot noise limited.
            if v > 0.25 {
                continue;
            }

            let (n0, n1, n2, n3) = if site == 2 || site == 3 {
                // Green site: 4 diagonal neighbors (dx = +/-1, dy = +/-1) are also Green
                (
                    orig[(y - 1) * width + (x - 1)],
                    orig[(y - 1) * width + (x + 1)],
                    orig[(y + 1) * width + (x - 1)],
                    orig[(y + 1) * width + (x + 1)],
                )
            } else {
                // Red (0) or Blue (1) site: 4 cardinal neighbors 2 pixels away (dx = +/-2, dy = 0 and dx = 0, dy = +/-2)
                (
                    orig[(y - 2) * width + x],
                    orig[(y + 2) * width + x],
                    orig[row_off + (x - 2)],
                    orig[row_off + (x + 2)],
                )
            };

            let n_max = n0.max(n1).max(n2).max(n3);
            let sigma = (shot_slope * n_max + read_var).sqrt();
            let threshold = n_max + k * sigma;

            if v > threshold {
                pixels[row_off + x] = threshold;
            }
        }
    }
}

