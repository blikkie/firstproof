use std::path::Path;
pub use firstproof_types::{
    f32_to_f16, CfaPattern, RawSensorFrame, CFA_COLOR_BLUE, CFA_COLOR_GREEN, CFA_COLOR_RED,
};

pub trait RawSensorFrameExt {
    fn sensor_to_rec2020_matrix_at(&self, temp_kelvin: f32) -> crate::color::Matrix3x3;
}

impl RawSensorFrameExt for RawSensorFrame {
    /// Computes the camera sensor to Linear Rec.2020 matrix dynamically interpolated
    /// for the specified white balance temperature (in Kelvin).
    fn sensor_to_rec2020_matrix_at(&self, temp_kelvin: f32) -> crate::color::Matrix3x3 {
        let mut dcp = crate::color::dcp::DcpProfile::built_in_for_camera(&self.camera_model);
        if let (Some(m_tungsten), Some(m_daylight)) = (self.color_matrix_tungsten, self.color_matrix) {
            dcp.color_matrix1 = crate::color::Matrix3x3(m_tungsten);
            dcp.color_matrix2 = Some(crate::color::Matrix3x3(m_daylight));
        } else if let Some(m_daylight) = self.color_matrix {
            dcp.color_matrix1 = crate::color::Matrix3x3(m_daylight);
            dcp.color_matrix2 = None;
        }
        dcp.interpolate_sensor_to_rec2020(temp_kelvin)
    }
}

/// Common trait for all RAW decoders
pub trait RawDecoder: Send + Sync {
    fn decode(&self, path: &Path) -> Result<RawSensorFrame, String>;
}

/// Default modern pure-Rust decoder powered by `rawler`
pub struct RawlerDecoder;

impl RawDecoder for RawlerDecoder {
    fn decode(&self, path: &Path) -> Result<RawSensorFrame, String> {
        let raw = rawler::decode_file(path).map_err(|e| format!("Rawler decode error: {}", e))?;

        let (crop_top, crop_left, active_w, active_h) = if let Some(crop) = raw.crop_area {
            let x = (crop.p.x as usize) & !1;
            let y = (crop.p.y as usize) & !1;
            let w = (crop.d.w as usize).max(2) & !1;
            let h = (crop.d.h as usize).max(2) & !1;
            (y, x, w, h)
        } else {
            (0, 0, raw.width & !1, raw.height & !1)
        };

        // Determine 2x2 CFA pattern
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

        // Extract both Illuminant A (tungsten ~2856K) and Illuminant D65 (daylight ~6504K)
        let find_matrix = |pattern: &str| -> Option<[[f32; 3]; 3]> {
            raw.color_matrix.iter()
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

        let color_matrix_tungsten = find_matrix("a");
        let color_matrix = find_matrix("d65")
            .or_else(|| find_matrix("d50"))
            .or_else(|| raw.color_matrix.iter().next().and_then(|(_, v)| {
                if v.len() >= 9 {
                    Some([
                        [v[0], v[1], v[2]],
                        [v[3], v[4], v[5]],
                        [v[6], v[7], v[8]],
                    ])
                } else {
                    None
                }
            }));

        let camera_model = format!("{} {}", raw.clean_make, raw.clean_model).trim().to_string();

        let mut black_levels = [0.0f32; 4];
        if raw.blacklevel.levels.is_empty() {
            // default 0.0
        } else if raw.blacklevel.levels.len() == 1 {
            let r = &raw.blacklevel.levels[0];
            let val = if r.d != 0 { r.n as f32 / r.d as f32 } else { r.n as f32 };
            black_levels = [val; 4];
        } else {
            for i in 0..4.min(raw.blacklevel.levels.len()) {
                let r = &raw.blacklevel.levels[i];
                black_levels[i] = if r.d != 0 { r.n as f32 / r.d as f32 } else { r.n as f32 };
            }
        }

        let mut white_levels = [65535.0f32; 4];
        if raw.whitelevel.0.is_empty() {
            // default 65535.0
        } else if raw.whitelevel.0.len() == 1 {
            white_levels = [raw.whitelevel.0[0] as f32; 4];
        } else {
            for i in 0..4.min(raw.whitelevel.0.len()) {
                white_levels[i] = raw.whitelevel.0[i] as f32;
            }
        }

        let wb = raw.wb_coeffs;
        let wb_coeffs = [wb[0], wb[1], wb[2], wb[3]];

        let baseline_exposure = std::fs::File::open(path)
            .ok()
            .and_then(|mut f| {
                use std::io::Read;
                let mut header = vec![0u8; 65536];
                let n = f.read(&mut header).ok()?;
                header.truncate(n);
                Some(firstproof_types::resolve_baseline_exposure(&header, &camera_model))
            })
            .unwrap_or_else(|| firstproof_types::resolve_baseline_exposure(&[], &camera_model));

        let sensor_data = match raw.data {
            rawler::RawImageData::Integer(cfa) => cfa,
            _ => return Err("Unsupported rawler data format (non-integer sensor CFA)".to_string()),
        };

        Ok(RawSensorFrame {
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
            camera_model,
            baseline_exposure,
            sensor_data,
        })
    }
}
