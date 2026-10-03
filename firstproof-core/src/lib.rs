uniffi::setup_scaffolding!("firstproof_core");

use std::path::Path;

pub mod raw;
pub mod raw_decoder;
pub mod xmp;
pub mod takeout;
pub mod db;
pub mod lensfun;
pub mod export;
pub mod color;
pub mod auto;
pub mod lut;
pub mod curve;
pub mod gainmap;
pub mod synthetic;
pub mod checksum;

pub use firstproof_engine::{
    all_adjustments, get_adjustment, kelvin_to_mired_norm, mired_norm_to_kelvin,
    AdjustmentDescriptor, ScaleMode, WorkspaceCategory,
    evaluate_all_curves_lut, evaluate_curve_lut, AllCurvesLutRecord, ColorProfileMode,
    CurvePointRecord, DemosaicQuality, EditSettings, EmulsionArchetype, EmulsionFormat,
    EmulsionGrainMode, EnlargerFilterMode, ExifOrientationRecord, ExportColorSpace,
    ExportRenderingIntent, PrintMediaMode, RemjetBackingMode, RenderTier, ToneCurvesRecord, WhiteBalanceResult,
};

pub use lensfun::LensCorrectionCoefficients;

#[derive(uniffi::Record, Clone, Debug)]
pub struct BatchExportItemRecord {
    pub input_path: String,
    pub output_path: String,
    pub edits: EditSettings,
    pub lens_coeffs: Option<LensCorrectionCoefficients>,
    pub quality: u32,
    pub max_dimension: Option<u32>,
    pub color_space: ExportColorSpace,
    pub rendering_intent: ExportRenderingIntent,
    pub ultra_hdr: bool,
    pub demosaic_quality: Option<DemosaicQuality>,
    pub chemical_edge_diffusion: Option<bool>,
    pub render_tier: Option<RenderTier>,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct BatchExportResultRecord {
    pub input_path: String,
    pub output_path: String,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct CubeLutRecord {
    pub title: String,
    pub size: u32,
    pub data_rgba32f: Vec<u8>,
}



/// Returns a deterministic signature of all edit settings for cache keying and invalidation.
/// Automatically serializes all current and future fields without manual tracking.
#[uniffi::export]
pub fn edits_cache_signature(edits: &EditSettings) -> String {
    serde_json::to_string(edits).unwrap_or_else(|_| format!("{:?}", edits))
}

/// Checks whether an EditSettings instance differs from default/as-shot settings.
#[uniffi::export]
pub fn edits_has_modifications(
    edits: &EditSettings,
    as_shot_temp: Option<f32>,
    as_shot_tint: Option<f32>,
) -> bool {
    let mut default = EditSettings::default();
    if let Some(temp) = as_shot_temp {
        default.temperature = temp;
    }
    if let Some(tint) = as_shot_tint {
        default.tint = tint;
    }
    edits != &default
}

/// Calculates photographic Kelvin temperature and tint from an unadjusted neutral pixel sample.
#[uniffi::export]
pub fn calculate_white_balance_from_sample(r: f32, g: f32, b: f32) -> WhiteBalanceResult {
    if r <= 1e-5 || g <= 1e-5 || b <= 1e-5 {
        return WhiteBalanceResult {
            temperature: 5500.0,
            tint: 0.0,
        };
    }

    let temp_ratio = (b / r).max(1e-4);
    let temp_norm = temp_ratio.powf(2.0 / 3.0);
    let target_temp = (5500.0 * temp_norm).clamp(2000.0, 20000.0);

    let r_scaled = r * temp_norm.powf(0.75);
    let b_scaled = b * (1.0 / temp_norm).powf(0.75);
    let rb_avg = (r_scaled + b_scaled) * 0.5;

    let tint_factor = 1.0 - (rb_avg / g.max(1e-4));
    let target_tint = (tint_factor / 0.002).clamp(-200.0, 200.0);

    WhiteBalanceResult {
        temperature: (target_temp / 50.0).round() * 50.0,
        tint: target_tint.round(),
    }
}

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
    pub title: Option<String>,
    pub description: Option<String>,
    pub creator: Option<String>,
    pub copyright: Option<String>,
    pub keywords: Option<String>,
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
            title: r.title,
            description: r.description,
            creator: r.creator,
            copyright: r.copyright,
            keywords: r.keywords,
            rating: r.rating,
            status: r.status,
            has_xmp: r.has_xmp,
            has_takeout: r.has_takeout,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
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
    pub creator: Option<String>,
    pub copyright: Option<String>,
    pub orientation: Option<u32>,
}

#[uniffi::export]
pub fn decode_exif_orientation(orientation: u32) -> ExifOrientationRecord {
    firstproof_engine::decode_exif_orientation(orientation)
}

#[uniffi::export]
pub fn get_all_adjustments() -> Vec<AdjustmentDescriptor> {
    all_adjustments()
}

#[uniffi::export]
pub fn get_adjustment_descriptor(id: String) -> Option<AdjustmentDescriptor> {
    get_adjustment(&id)
}

#[uniffi::export]
pub fn convert_kelvin_to_mired_norm(kelvin: f64) -> f64 {
    kelvin_to_mired_norm(kelvin)
}

#[uniffi::export]
pub fn convert_mired_norm_to_kelvin(norm: f64) -> f64 {
    mired_norm_to_kelvin(norm)
}

#[derive(uniffi::Record)]
pub struct XmpDataRecord {
    pub rating: Option<i32>,
    pub status: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub creator: Option<String>,
    pub copyright: Option<String>,
    pub keywords: Vec<String>,
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
            file_path: path,
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
            creator: m.creator,
            copyright: m.copyright,
            orientation: m.orientation,
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
            creator: x.creator,
            copyright: x.copyright,
            keywords: x.keywords,
            latitude: x.latitude,
            longitude: x.longitude,
            edits: x.edits,
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
        creator: record.creator,
        copyright: record.copyright,
        keywords: record.keywords,
        latitude: record.latitude,
        longitude: record.longitude,
        edits: record.edits,
    };
    xmp::write_xmp_file(&path, &data).is_ok()
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct GainMapMetadataRecord {
    pub min_gain: f32,
    pub max_gain: f32,
    pub gamma: f32,
    pub offset_sdr: f32,
    pub offset_hdr: f32,
    pub hdr_capacity_min: f32,
    pub hdr_capacity_max: f32,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct GainMapRecord {
    pub metadata: GainMapMetadataRecord,
    pub jpeg_bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[uniffi::export]
pub fn has_ultra_hdr_gain_map(file_path: String) -> bool {
    if let Ok(bytes) = std::fs::read(&file_path) {
        gainmap::has_gain_map(&bytes)
    } else {
        false
    }
}

#[uniffi::export]
pub fn extract_ultra_hdr_gain_map(file_path: String) -> Option<GainMapRecord> {
    let bytes = std::fs::read(&file_path).ok()?;
    let (meta, jpeg_bytes, width, height) = gainmap::extract_gain_map(&bytes)?;
    Some(GainMapRecord {
        metadata: GainMapMetadataRecord {
            min_gain: meta.min_gain,
            max_gain: meta.max_gain,
            gamma: meta.gamma,
            offset_sdr: meta.offset_sdr,
            offset_hdr: meta.offset_hdr,
            hdr_capacity_min: meta.hdr_capacity_min,
            hdr_capacity_max: meta.hdr_capacity_max,
        },
        jpeg_bytes,
        width,
        height,
    })
}

#[uniffi::export]
pub fn export_photo(
    input_path: String,
    output_path: String,
    edits: EditSettings,
    lens_coeffs: Option<LensCorrectionCoefficientsRecord>,
    quality: u32,
    max_dimension: Option<u32>,
    color_space: Option<ExportColorSpace>,
    rendering_intent: Option<ExportRenderingIntent>,
    ultra_hdr: Option<bool>,
    demosaic_quality: Option<DemosaicQuality>,
    chemical_edge_diffusion: Option<bool>,
    render_tier: Option<RenderTier>,
) -> bool {
    let dist_tuple = lens_coeffs.as_ref().and_then(|c| {
        if c.has_distortion {
            Some((c.distortion.as_slice(), c.distortion_model.as_str()))
        } else {
            None
        }
    });
    
    let cs = color_space.unwrap_or_default();
    let ri = rendering_intent.unwrap_or_default();
    let uhdr = ultra_hdr.unwrap_or(false);
    let dq = demosaic_quality.unwrap_or_default();
    let ced = chemical_edge_diffusion.unwrap_or(false);
    let rt = render_tier.unwrap_or(RenderTier::ArchivalExport);
    
    match export::export_photo_file_advanced(
        &input_path,
        &output_path,
        &edits,
        dist_tuple,
        quality as u8,
        max_dimension,
        cs,
        ri,
        uhdr,
        dq,
        ced,
        rt,
    ) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Export error for {}: {}", input_path, e);
            false
        }
    }
}

#[uniffi::export]
pub fn export_batch_photos(
    items: Vec<BatchExportItemRecord>,
) -> Vec<BatchExportResultRecord> {
    export::export_batch_photos(items)
}

#[derive(uniffi::Record)]
pub struct RawPreviewRecord {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub is_linear_float: bool,
}

#[uniffi::export]
pub fn decode_raw_preview(path: String, max_dimension: Option<u32>) -> Option<RawPreviewRecord> {
    match export::decode_raw_preview_buffer(&path, max_dimension) {
        Ok((width, height, data)) => Some(RawPreviewRecord {
            width,
            height,
            data,
            is_linear_float: true,
        }),
        Err(e) => {
            eprintln!("Failed to decode RAW preview for {}: {}", path, e);
            None
        }
    }
}

#[uniffi::export]
pub fn decode_raw_preview_binned(path: String) -> Option<RawPreviewRecord> {
    match export::decode_raw_preview_binned_buffer(&path) {
        Ok((width, height, data)) => Some(RawPreviewRecord {
            width,
            height,
            data,
            is_linear_float: true,
        }),
        Err(e) => {
            eprintln!("Failed to decode binned RAW preview for {}: {}", path, e);
            None
        }
    }
}

#[uniffi::export]
pub fn compute_auto_adjustments(path: String) -> Option<auto::AutoAdjustments> {
    auto::compute_auto_adjustments_from_path(&path)
}

#[uniffi::export]
pub fn compute_auto_adjustments_from_buffer(width: u32, height: u32, data: Vec<u8>) -> Option<auto::AutoAdjustments> {
    Some(auto::compute_auto_adjustments_from_f16_buffer(width, height, &data))
}

#[uniffi::export]
pub fn compute_auto_white_balance(path: String) -> Option<WhiteBalanceResult> {
    auto::compute_auto_white_balance_from_path(&path)
}

#[uniffi::export]
pub fn compute_auto_white_balance_from_buffer(width: u32, height: u32, data: Vec<u8>) -> Option<WhiteBalanceResult> {
    Some(auto::compute_auto_white_balance_from_f16_buffer(width, height, &data))
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

#[derive(Debug, uniffi::Error)]
pub enum FirstProofError {
    DatabaseError { reason: String },
}

impl std::fmt::Display for FirstProofError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FirstProofError::DatabaseError { reason } => write!(f, "Database error: {}", reason),
        }
    }
}

#[derive(uniffi::Object)]
pub struct FirstProofDatabase {
    inner: db::Database,
}

#[uniffi::export]
impl FirstProofDatabase {
    #[uniffi::constructor]
    pub fn open(db_path: String) -> Result<Arc<Self>, FirstProofError> {
        db::Database::open(db_path)
            .map(|inner| Arc::new(Self { inner }))
            .map_err(|e| FirstProofError::DatabaseError {
                reason: e.to_string(),
            })
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

    pub fn update_photo_metadata(
        &self,
        file_path: String,
        title: Option<String>,
        description: Option<String>,
        creator: Option<String>,
        copyright: Option<String>,
        keywords: Option<String>,
    ) -> bool {
        self.inner
            .update_photo_metadata(
                &file_path,
                title.as_deref(),
                description.as_deref(),
                creator.as_deref(),
                copyright.as_deref(),
                keywords.as_deref(),
            )
            .unwrap_or(false)
    }

    pub fn update_rating_status_batch(
        &self,
        file_paths: Vec<String>,
        rating: Option<i32>,
        update_rating: bool,
        status: Option<String>,
    ) -> u32 {
        self.inner
            .update_rating_status_batch(&file_paths, rating, update_rating, status.as_deref())
            .unwrap_or(0)
    }

    pub fn export_portable_package(
        &self,
        source_photo_paths: Vec<String>,
        destination_folder: String,
        move_files: bool,
    ) -> Result<ExportPackageReport, FirstProofError> {
        self.inner
            .export_portable_package(&source_photo_paths, &destination_folder, move_files)
            .map(Into::into)
            .map_err(|e| FirstProofError::DatabaseError { reason: e })
    }

    pub fn import_to_library(
        &self,
        source_paths_or_folders: Vec<String>,
        destination_folder: String,
        options: ImportOptionsRecord,
    ) -> Result<ImportReportRecord, FirstProofError> {
        let opts: db::ImportOptions = options.into();
        self.inner
            .import_to_library(&source_paths_or_folders, &destination_folder, &opts)
            .map(Into::into)
            .map_err(|e| FirstProofError::DatabaseError { reason: e })
    }

    pub fn close(&self) {
        self.inner.checkpoint_and_close();
    }
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct ImportOptionsRecord {
    pub move_files: bool,
    pub consolidate_sidecars: bool,
    pub deduplicate_raw_over_jpeg: bool,
    pub skip_duplicates: bool,
    pub verify_checksum: bool,
    pub subfolder_name: Option<String>,
}

impl From<ImportOptionsRecord> for db::ImportOptions {
    fn from(r: ImportOptionsRecord) -> Self {
        Self {
            move_files: r.move_files,
            consolidate_sidecars: r.consolidate_sidecars,
            deduplicate_raw_over_jpeg: r.deduplicate_raw_over_jpeg,
            skip_duplicates: r.skip_duplicates,
            verify_checksum: r.verify_checksum,
            subfolder_name: r.subfolder_name,
        }
    }
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct ImportReportRecord {
    pub total_found: u32,
    pub imported_count: u32,
    pub skipped_duplicates: u32,
    pub raw_over_jpeg_suppressed: u32,
    pub consolidated_sidecars: u32,
    pub error_count: u32,
    pub destination_catalog_path: String,
    pub error_messages: Vec<String>,
}

impl From<db::ImportReport> for ImportReportRecord {
    fn from(r: db::ImportReport) -> Self {
        Self {
            total_found: r.total_found,
            imported_count: r.imported_count,
            skipped_duplicates: r.skipped_duplicates,
            raw_over_jpeg_suppressed: r.raw_over_jpeg_suppressed,
            consolidated_sidecars: r.consolidated_sidecars,
            error_count: r.error_count,
            destination_catalog_path: r.destination_catalog_path,
            error_messages: r.error_messages,
        }
    }
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct ExportPackageReport {
    pub total_requested: u32,
    pub exported_count: u32,
    pub error_count: u32,
    pub destination_catalog_path: String,
    pub error_messages: Vec<String>,
}

impl From<db::ExportPackageStats> for ExportPackageReport {
    fn from(s: db::ExportPackageStats) -> Self {
        Self {
            total_requested: s.total_requested,
            exported_count: s.exported_count,
            error_count: s.error_count,
            destination_catalog_path: s.destination_catalog_path,
            error_messages: s.error_messages,
        }
    }
}

#[uniffi::export]
pub fn import_to_library(
    source_paths_or_folders: Vec<String>,
    destination_folder: String,
    options: ImportOptionsRecord,
) -> Result<ImportReportRecord, FirstProofError> {
    if let Some(first_path) = source_paths_or_folders.first() {
        let p = Path::new(first_path);
        let parent = if p.is_dir() { Some(p) } else { p.parent() };
        if let Some(parent_dir) = parent {
            let cat_path = db::Database::resolve_catalog_path(parent_dir, true);
            if let Ok(db) = db::Database::open(&cat_path) {
                let opts: db::ImportOptions = options.into();
                return db
                    .import_to_library(&source_paths_or_folders, &destination_folder, &opts)
                    .map(Into::into)
                    .map_err(|e| FirstProofError::DatabaseError { reason: e });
            }
        }
    }
    let temp_db = db::Database::open_in_memory()
        .map_err(|e| FirstProofError::DatabaseError { reason: e.to_string() })?;
    let opts: db::ImportOptions = options.into();
    temp_db
        .import_to_library(&source_paths_or_folders, &destination_folder, &opts)
        .map(Into::into)
        .map_err(|e| FirstProofError::DatabaseError { reason: e })
}

#[uniffi::export]
pub fn export_portable_package(
    source_photo_paths: Vec<String>,
    destination_folder: String,
    move_files: bool,
) -> Result<ExportPackageReport, FirstProofError> {
    if let Some(first_path) = source_photo_paths.first() {
        if let Some(parent) = Path::new(first_path).parent() {
            let cat_path = db::Database::resolve_catalog_path(parent, true);
            if let Ok(db) = db::Database::open(&cat_path) {
                return db
                    .export_portable_package(&source_photo_paths, &destination_folder, move_files)
                    .map(Into::into)
                    .map_err(|e| FirstProofError::DatabaseError { reason: e });
            }
        }
    }
    let temp_db = db::Database::open_in_memory()
        .map_err(|e| FirstProofError::DatabaseError { reason: e.to_string() })?;
    temp_db
        .export_portable_package(&source_photo_paths, &destination_folder, move_files)
        .map(Into::into)
        .map_err(|e| FirstProofError::DatabaseError { reason: e })
}

#[uniffi::export]
pub fn resolve_catalog_path(folder_path: String, portable: bool) -> String {
    db::Database::resolve_catalog_path(folder_path, portable)
        .to_string_lossy()
        .into_owned()
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct LensCorrectionCoefficientsRecord {
    pub distortion: Vec<f32>,
    pub tca: Vec<f32>,
    pub vignetting: Vec<f32>,
    pub distortion_model: String,
    pub has_distortion: bool,
    pub has_tca: bool,
    pub has_vignetting: bool,
}

impl From<lensfun::LensCorrectionCoefficients> for LensCorrectionCoefficientsRecord {
    fn from(c: lensfun::LensCorrectionCoefficients) -> Self {
        Self {
            distortion: c.distortion.to_vec(),
            tca: c.tca.to_vec(),
            vignetting: c.vignetting.to_vec(),
            distortion_model: c.distortion_model,
            has_distortion: c.has_distortion,
            has_tca: c.has_tca,
            has_vignetting: c.has_vignetting,
        }
    }
}

#[derive(uniffi::Object)]
pub struct FirstProofLensfun {
    inner: std::sync::Mutex<lensfun::LensfunDatabase>,
}

#[uniffi::export]
impl FirstProofLensfun {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: std::sync::Mutex::new(lensfun::LensfunDatabase::new()),
        })
    }

    pub fn load_directory(&self, dir_path: String) -> u32 {
        self.inner
            .lock()
            .unwrap()
            .load_directory(&dir_path)
            .unwrap_or(0)
    }

    pub fn load_xml_string(&self, xml_content: String) -> bool {
        self.inner
            .lock()
            .unwrap()
            .load_xml_string(&xml_content)
            .is_ok()
    }

    pub fn resolve_coefficients(
        &self,
        camera_make: String,
        camera_model: String,
        lens_model: String,
        focal_length: f32,
        aperture: Option<f32>,
    ) -> LensCorrectionCoefficientsRecord {
        self.inner
            .lock()
            .unwrap()
            .resolve_coefficients(&camera_make, &camera_model, &lens_model, focal_length, aperture)
            .into()
    }
}

#[uniffi::export]
pub fn check_connection() -> String {
    "First Proof Rust Core Connected!".to_string()
}

#[uniffi::export]
pub fn get_builtin_cube_lut_titles() -> Vec<String> {
    Vec::new()
}

#[uniffi::export]
pub fn get_builtin_cube_lut(_title: String, _size: u32) -> Option<CubeLutRecord> {
    None
}

#[uniffi::export]
pub fn load_cube_lut(path: String) -> Option<CubeLutRecord> {
    lut::load_cube_lut_file(&path).ok().map(|l| {
        let data_rgba32f = l.to_rgba32f_bytes();
        CubeLutRecord {
            title: l.title,
            size: l.size as u32,
            data_rgba32f,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edits_cache_signature_and_modification_detection() {
        let default_edits = EditSettings::default();
        assert!(!edits_has_modifications(&default_edits, None, None));
        assert!(!edits_has_modifications(&default_edits, Some(5500.0), Some(0.0)));

        let sig1 = edits_cache_signature(&default_edits);
        assert!(!sig1.is_empty());
        assert!(sig1.contains("perspective_vertical"));
        assert!(sig1.contains("crop_constrain"));

        // Changing perspective alters signature and triggers modification detection
        let mut modified = default_edits.clone();
        modified.perspective_vertical = 12.0;
        let sig2 = edits_cache_signature(&modified);
        assert_ne!(sig1, sig2);
        assert!(edits_has_modifications(&modified, None, None));

        // Changing crop alters signature
        let mut cropped = default_edits.clone();
        cropped.crop_left = 0.1;
        cropped.crop_right = 0.9;
        let sig3 = edits_cache_signature(&cropped);
        assert_ne!(sig1, sig3);
        assert_ne!(sig2, sig3);
        assert!(edits_has_modifications(&cropped, None, None));
    }
}

