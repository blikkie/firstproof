use std::path::Path;

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
            color_matrix: None,
            sensor_data,
        })
    }
}
