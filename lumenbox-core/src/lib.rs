uniffi::setup_scaffolding!();

use std::path::Path;

pub mod raw;
pub mod xmp;
pub mod takeout;
pub mod db;

#[derive(uniffi::Record, Clone, Debug)]
pub struct PhotoRecord {
    pub id: i64,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub modified_time: i64,
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
    pub rating: Option<i32>,
    pub status: String,
    pub has_xmp: bool,
    pub has_takeout: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<db::PhotoRecord> for PhotoRecord {
    fn from(r: db::PhotoRecord) -> Self {
        Self {
            id: r.id,
            file_path: r.file_path,
            file_name: r.file_name,
            file_size: r.file_size,
            modified_time: r.modified_time,
            camera_make: r.camera_make,
            camera_model: r.camera_model,
            lens_model: r.lens_model,
            iso: r.iso,
            aperture: r.aperture,
            shutter_speed: r.shutter_speed,
            focal_length: r.focal_length,
            taken_time: r.taken_time,
            latitude: r.latitude,
            longitude: r.longitude,
            description: r.description,
            rating: r.rating,
            status: r.status,
            has_xmp: r.has_xmp,
            has_takeout: r.has_takeout,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

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

#[derive(uniffi::Record)]
pub struct TakeoutMetadataRecord {
    pub title: String,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub taken_time: Option<i64>,
}

#[derive(uniffi::Record)]
pub struct TakeoutMatchRecord {
    pub image_path: String,
    pub json_path: String,
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
pub fn match_takeout_json(image_path: String) -> Option<String> {
    takeout::match_takeout_json(Path::new(&image_path))
        .map(|p| p.to_string_lossy().into_owned())
}

#[uniffi::export]
pub fn parse_takeout_json(json_path: String) -> Option<TakeoutMetadataRecord> {
    match takeout::parse_takeout_json(Path::new(&json_path)) {
        Ok(t) => Some(TakeoutMetadataRecord {
            title: t.title,
            description: t.description,
            latitude: t.latitude,
            longitude: t.longitude,
            altitude: t.altitude,
            taken_time: t.taken_time,
        }),
        Err(_) => None,
    }
}

#[uniffi::export]
pub fn scan_directory_for_takeout(dir_path: String) -> Vec<TakeoutMatchRecord> {
    takeout::scan_directory_for_takeout(Path::new(&dir_path))
        .into_iter()
        .map(|m| TakeoutMatchRecord {
            image_path: m.image_path.to_string_lossy().into_owned(),
            json_path: m.json_path.to_string_lossy().into_owned(),
        })
        .collect()
}

use std::sync::Arc;

#[derive(uniffi::Object)]
pub struct LumenBoxDatabase {
    inner: db::Database,
}

#[uniffi::export]
impl LumenBoxDatabase {
    #[uniffi::constructor]
    pub fn open(db_path: String) -> Option<Arc<Self>> {
        db::Database::open(db_path).ok().map(|inner| Arc::new(Self { inner }))
    }

    pub fn index_folder(&self, folder_path: String) -> u32 {
        self.inner.index_folder(folder_path).unwrap_or(0)
    }

    pub fn get_photo(&self, file_path: String) -> Option<PhotoRecord> {
        self.inner
            .get_photo(&file_path)
            .ok()
            .flatten()
            .map(Into::into)
    }

    pub fn get_all_photos(&self) -> Vec<PhotoRecord> {
        self.inner
            .get_all_photos()
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn update_rating_status(&self, file_path: String, rating: Option<i32>, status: String) -> bool {
        self.inner
            .update_rating_status(&file_path, rating, &status)
            .unwrap_or(false)
    }
}

#[uniffi::export]
pub fn resolve_catalog_path(folder_path: String, portable: bool) -> String {
    db::Database::resolve_catalog_path(folder_path, portable)
        .to_string_lossy()
        .into_owned()
}

#[uniffi::export]
pub fn check_connection() -> String {
    "LumenBox Rust Core Connected!".to_string()
}
