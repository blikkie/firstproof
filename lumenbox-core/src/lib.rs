uniffi::setup_scaffolding!();

pub mod raw;
pub mod xmp;

#[derive(uniffi::Record, Clone, Copy, Debug)]
pub struct EditSettings {
    pub exposure: f32,
    pub contrast: f32,
    pub temperature: f32,
    pub tint: f32,
    pub crop_left: f32,
    pub crop_top: f32,
    pub crop_right: f32,
    pub crop_bottom: f32,
    pub crop_angle: f32,
}

#[derive(uniffi::Record)]
pub struct ImageMetadata {
    pub file_path: String,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub iso: Option<u32>,
    pub aperture: Option<f32>,
    pub shutter_speed: Option<String>,
    pub focal_length: Option<f32>,
    pub taken_time: Option<i64>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
}

#[derive(uniffi::Record)]
pub struct XmpDataRecord {
    pub rating: Option<i32>,
    pub status: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub edits: EditSettings,
}

#[uniffi::export]
pub fn extract_metadata(path: String) -> Option<ImageMetadata> {
    match raw::extract_metadata(&path) {
        Ok(m) => Some(ImageMetadata {
            file_path: m.file_path,
            camera_make: m.camera_make,
            camera_model: m.camera_model,
            lens_model: m.lens_model,
            iso: m.iso,
            aperture: m.aperture,
            shutter_speed: m.shutter_speed,
            focal_length: m.focal_length,
            taken_time: m.taken_time,
            latitude: m.latitude,
            longitude: m.longitude,
            description: m.description,
        }),
        Err(_) => None,
    }
}

#[uniffi::export]
pub fn extract_preview(path: String) -> Option<Vec<u8>> {
    raw::extract_preview_jpeg(&path).ok()
}

#[uniffi::export]
pub fn read_xmp(path: String) -> Option<XmpDataRecord> {
    match xmp::read_xmp_file(&path) {
        Ok(x) => Some(XmpDataRecord {
            rating: x.rating,
            status: x.status,
            title: x.title,
            description: x.description,
            latitude: x.latitude,
            longitude: x.longitude,
            edits: EditSettings {
                exposure: x.edits.exposure,
                contrast: x.edits.contrast,
                temperature: x.edits.temperature,
                tint: x.edits.tint,
                crop_left: x.edits.crop_left,
                crop_top: x.edits.crop_top,
                crop_right: x.edits.crop_right,
                crop_bottom: x.edits.crop_bottom,
                crop_angle: x.edits.crop_angle,
            },
        }),
        Err(_) => None,
    }
}

#[uniffi::export]
pub fn write_xmp(path: String, record: XmpDataRecord) -> bool {
    let data = xmp::XmpData {
        rating: record.rating,
        status: record.status,
        title: record.title,
        description: record.description,
        latitude: record.latitude,
        longitude: record.longitude,
        edits: EditSettings {
            exposure: record.edits.exposure,
            contrast: record.edits.contrast,
            temperature: record.edits.temperature,
            tint: record.edits.tint,
            crop_left: record.edits.crop_left,
            crop_top: record.edits.crop_top,
            crop_right: record.edits.crop_right,
            crop_bottom: record.edits.crop_bottom,
            crop_angle: record.edits.crop_angle,
        },
    };
    xmp::write_xmp_file(&path, &data).is_ok()
}

#[uniffi::export]
pub fn check_connection() -> String {
    "LumenBox Rust Core Connected!".to_string()
}
