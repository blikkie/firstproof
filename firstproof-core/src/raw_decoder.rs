use std::path::Path;
pub use firstproof_engine::{
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

        let header = std::fs::File::open(path)
            .ok()
            .and_then(|mut f| {
                use std::io::Read;
                let mut buf = vec![0u8; 65536];
                let n = f.read(&mut buf).ok()?;
                buf.truncate(n);
                Some(buf)
            })
            .unwrap_or_default();

        RawSensorFrame::decode_from_raw_image(raw, &header)
            .ok_or_else(|| "Failed to construct RawSensorFrame from decoded RAW".to_string())
    }
}
