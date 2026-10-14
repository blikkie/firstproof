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
    /// Delegates directly to firstproof_engine's canonical model to ensure 1:1 parity
    /// between fast viewport binned previews and full archival demosaic exports.
    fn sensor_to_rec2020_matrix_at(&self, temp_kelvin: f32) -> crate::color::Matrix3x3 {
        crate::color::Matrix3x3(self.sensor_to_rec2020_at(temp_kelvin))
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

        let bytes = std::fs::read(path).unwrap_or_default();

        RawSensorFrame::decode_from_raw_image(raw, &bytes)
            .ok_or_else(|| "Failed to construct RawSensorFrame from decoded RAW".to_string())
    }
}
