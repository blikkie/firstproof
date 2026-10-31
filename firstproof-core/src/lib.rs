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
pub mod dng;
pub mod proof;

pub use proof::{IccProfileRecord, ProofIntent};
pub use lut::LutRecord;

pub use firstproof_engine::{
    all_adjustments, get_adjustment, kelvin_to_mired_norm, mired_norm_to_kelvin,
    AdjustmentDescriptor, ScaleMode, WorkspaceCategory,
    evaluate_all_curves_lut, evaluate_curve_lut, AllCurvesLutRecord,
    CommandResponse, CropAspectRatioPreset, CurvePointRecord, DarkroomCommand, DemosaicQuality,
    EditDelta, EditSettings, EmulsionArchetype, EmulsionFormat, EmulsionGrainMode,
    EnlargerFilterMode, EvHistogramRecord, ExifOrientationRecord, ExportColorSpace, ExportRenderingIntent,
    PrintMediaMode, RemjetBackingMode, RenderTier, ToneCurvesRecord, WhiteBalancePreset,
    WhiteBalanceResult,
};

pub use lensfun::LensCorrectionCoefficients;

#[derive(uniffi::Record, Clone, Debug)]
pub struct BatchExportItemRecord {
    pub input_path: String,
    pub output_path: String,
    pub edits: EditSettings,
    pub lens_coeffs: Option<LensCorrectionCoefficientsRecord>,
    pub quality: u32,
    pub max_dimension: Option<u32>,
    pub color_space: ExportColorSpace,
    pub rendering_intent: ExportRenderingIntent,
    pub ultra_hdr: bool,
    pub demosaic_quality: Option<DemosaicQuality>,
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
    let res = firstproof_engine::auto::calculate_white_balance_from_sample(r, g, b);
    WhiteBalanceResult {
        temperature: res.temperature,
        tint: res.tint,
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

impl From<PhotoRecord> for db::PhotoRecord {
    fn from(r: PhotoRecord) -> Self {
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

#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct WhiteBalancePresetValuesRecord {
    pub temperature: f64,
    pub tint: f64,
}

#[uniffi::export]
pub fn calculate_target_crop_ratio(
    preset: CropAspectRatioPreset,
    image_aspect: f32,
    is_portrait: bool,
) -> Option<f32> {
    preset.target_ratio(image_aspect, is_portrait)
}

#[uniffi::export]
pub fn get_white_balance_preset_values(preset: WhiteBalancePreset) -> Option<WhiteBalancePresetValuesRecord> {
    preset.kelvin_and_tint().map(|(temperature, tint)| WhiteBalancePresetValuesRecord {
        temperature,
        tint,
    })
}

#[uniffi::export]
pub fn detect_white_balance_preset(
    temperature: f64,
    tint: f64,
    as_shot_temp: f64,
    as_shot_tint: f64,
) -> WhiteBalancePreset {
    WhiteBalancePreset::detect(temperature, tint, as_shot_temp, as_shot_tint)
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
    render_tier: Option<RenderTier>,
) -> bool {
    let cs = color_space.unwrap_or_default();
    let ri = rendering_intent.unwrap_or_default();
    let uhdr = ultra_hdr.unwrap_or(false);
    let dq = demosaic_quality.unwrap_or_default();
    let rt = render_tier.unwrap_or(RenderTier::ArchivalExport);
    
    match export::export_photo_file_advanced(
        &input_path,
        &output_path,
        &edits,
        lens_coeffs.as_ref(),
        quality as u8,
        max_dimension,
        cs,
        ri,
        uhdr,
        dq,
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
pub fn compute_ev_histogram(
    width: u32,
    height: u32,
    data: Vec<u8>,
    exposure: f32,
    contrast: f32,
    highlights: f32,
    shadows: f32,
    whites: f32,
    blacks: f32,
    crop_left: f32,
    crop_top: f32,
    crop_right: f32,
    crop_bottom: f32,
    lens_correction_enabled: bool,
    tca_correction_enabled: bool,
    vignetting_correction_enabled: bool,
    vignetting_k1: f32,
    vignetting_k2: f32,
    vignetting_k3: f32,
) -> Option<EvHistogramRecord> {
    firstproof_engine::histogram::compute_ev_histogram_from_f16_buffer(
        width, height, &data, exposure, contrast, highlights, shadows, whites, blacks,
        crop_left, crop_top, crop_right, crop_bottom,
        lens_correction_enabled, tca_correction_enabled,
        vignetting_correction_enabled, vignetting_k1, vignetting_k2, vignetting_k3,
    )
}

#[uniffi::export]
pub fn compute_ev_histogram_from_path(
    path: String,
    exposure: f32,
    contrast: f32,
    highlights: f32,
    shadows: f32,
    whites: f32,
    blacks: f32,
    crop_left: f32,
    crop_top: f32,
    crop_right: f32,
    crop_bottom: f32,
    lens_correction_enabled: bool,
    tca_correction_enabled: bool,
    vignetting_correction_enabled: bool,
    vignetting_k1: f32,
    vignetting_k2: f32,
    vignetting_k3: f32,
) -> Option<EvHistogramRecord> {
    if let Ok((width, height, data)) = export::decode_raw_preview_binned_buffer(&path) {
        firstproof_engine::histogram::compute_ev_histogram_from_f16_buffer(
            width, height, &data, exposure, contrast, highlights, shadows, whites, blacks,
            crop_left, crop_top, crop_right, crop_bottom,
            lens_correction_enabled, tca_correction_enabled,
            vignetting_correction_enabled, vignetting_k1, vignetting_k2, vignetting_k3,
        )
    } else {
        None
    }
}

#[uniffi::export]
pub fn get_as_shot_white_balance(path: String) -> Option<WhiteBalanceResult> {
    let decoder = crate::raw_decoder::RawlerDecoder;
    if let Ok(frame) = crate::raw_decoder::RawDecoder::decode(&decoder, Path::new(&path)) {
        Some(WhiteBalanceResult {
            temperature: frame.as_shot_temperature(),
            tint: frame.as_shot_tint(),
        })
    } else {
        None
    }
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
    DngError { reason: String },
    ColorSyncError { reason: String },
    LutError { reason: String },
}

impl std::fmt::Display for FirstProofError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FirstProofError::DatabaseError { reason } => write!(f, "Database error: {}", reason),
            FirstProofError::DngError { reason } => write!(f, "DNG conversion error: {}", reason),
            FirstProofError::ColorSyncError { reason } => write!(f, "ColorSync error: {}", reason),
            FirstProofError::LutError { reason } => write!(f, "LUT error: {}", reason),
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

    pub fn get_master_photos(&self) -> Vec<PhotoRecord> {
        let photos = self.inner.get_all_photos().unwrap_or_default();
        self.inner
            .filter_derivative_records(photos)
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
        progress: Option<Box<dyn ImportProgressCallback>>,
    ) -> Result<ImportReportRecord, FirstProofError> {
        let opts: db::ImportOptions = options.into();
        let progress_fn = progress.as_ref().map(|cb| {
            move |curr: u32, tot: u32, file: &str| {
                cb.on_progress(curr, tot, file.to_string());
            }
        });
        let progress_ref: Option<&dyn Fn(u32, u32, &str)> = match &progress_fn {
            Some(f) => Some(f),
            None => None,
        };
        self.inner
            .import_to_library(&source_paths_or_folders, &destination_folder, &opts, progress_ref)
            .map(Into::into)
            .map_err(|e| FirstProofError::DatabaseError { reason: e })
    }

    pub fn delete_photos(&self, file_paths: Vec<String>) -> u32 {
        self.inner.delete_photos(&file_paths).unwrap_or(0) as u32
    }

    pub fn find_duplicate_clusters(&self) -> Vec<DuplicateClusterRecord> {
        let photos = self.inner.get_all_photos().unwrap_or_default();
        db::Database::find_duplicate_clusters_in_memory(photos)
            .into_iter()
            .map(|c| DuplicateClusterRecord {
                original: c.original.into(),
                duplicates: c.duplicates.into_iter().map(Into::into).collect(),
            })
            .collect()
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
    pub convert_to_dng: bool,
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
            convert_to_dng: r.convert_to_dng,
        }
    }
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct ImportReportRecord {
    pub total_found: u32,
    pub imported_count: u32,
    pub skipped_duplicates: u32,
    pub raw_over_jpeg_suppressed: u32,
    pub duplicate_clones_suppressed: u32,
    pub consolidated_sidecars: u32,
    pub error_count: u32,
    pub destination_catalog_path: String,
    pub error_messages: Vec<String>,
    pub dng_converted_count: u32,
}

impl From<db::ImportReport> for ImportReportRecord {
    fn from(r: db::ImportReport) -> Self {
        Self {
            total_found: r.total_found,
            imported_count: r.imported_count,
            skipped_duplicates: r.skipped_duplicates,
            raw_over_jpeg_suppressed: r.raw_over_jpeg_suppressed,
            duplicate_clones_suppressed: r.duplicate_clones_suppressed,
            consolidated_sidecars: r.consolidated_sidecars,
            error_count: r.error_count,
            destination_catalog_path: r.destination_catalog_path,
            error_messages: r.error_messages,
            dng_converted_count: r.dng_converted_count,
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

#[uniffi::export(callback_interface)]
pub trait ImportProgressCallback: Send + Sync + std::fmt::Debug {
    fn on_progress(&self, current: u32, total: u32, current_file: String);
}

#[uniffi::export]
pub fn import_to_library(
    source_paths_or_folders: Vec<String>,
    destination_folder: String,
    options: ImportOptionsRecord,
    progress: Option<Box<dyn ImportProgressCallback>>,
) -> Result<ImportReportRecord, FirstProofError> {
    let progress_fn = progress.as_ref().map(|cb| {
        move |curr: u32, tot: u32, file: &str| {
            cb.on_progress(curr, tot, file.to_string());
        }
    });
    let progress_ref: Option<&dyn Fn(u32, u32, &str)> = match &progress_fn {
        Some(f) => Some(f),
        None => None,
    };
    if let Some(first_path) = source_paths_or_folders.first() {
        let p = Path::new(first_path);
        let parent = if p.is_dir() { Some(p) } else { p.parent() };
        if let Some(parent_dir) = parent {
            let cat_path = db::Database::resolve_catalog_path(parent_dir, true);
            if let Ok(db) = db::Database::open(&cat_path) {
                let opts: db::ImportOptions = options.into();
                return db
                    .import_to_library(&source_paths_or_folders, &destination_folder, &opts, progress_ref)
                    .map(Into::into)
                    .map_err(|e| FirstProofError::DatabaseError { reason: e });
            }
        }
    }
    let temp_db = db::Database::open_in_memory()
        .map_err(|e| FirstProofError::DatabaseError { reason: e.to_string() })?;
    let opts: db::ImportOptions = options.into();
    temp_db
        .import_to_library(&source_paths_or_folders, &destination_folder, &opts, progress_ref)
        .map(Into::into)
        .map_err(|e| FirstProofError::DatabaseError { reason: e })
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct DngConvertOptionsRecord {
    pub output_path: Option<String>,
    pub delete_original: bool,
    pub verify: bool,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct DngConvertReportRecord {
    pub source_path: String,
    pub dng_path: String,
    pub source_size_bytes: u64,
    pub dng_size_bytes: u64,
    pub compression_ratio: f32,
    pub verification_passed: bool,
    pub deleted_original: bool,
    pub already_existed: bool,
    pub elapsed_ms: f64,
}

impl From<crate::dng::DngConvertReport> for DngConvertReportRecord {
    fn from(r: crate::dng::DngConvertReport) -> Self {
        Self {
            source_path: r.source_path.to_string_lossy().to_string(),
            dng_path: r.dng_path.to_string_lossy().to_string(),
            source_size_bytes: r.source_size_bytes,
            dng_size_bytes: r.dng_size_bytes,
            compression_ratio: r.compression_ratio,
            verification_passed: r.verification_passed,
            deleted_original: r.deleted_original,
            already_existed: r.already_existed,
            elapsed_ms: r.elapsed_ms,
        }
    }
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct DngBatchReportRecord {
    pub total_requested: u32,
    pub converted_count: u32,
    pub reconciled_existing_count: u32,
    pub skipped_count: u32,
    pub failed_count: u32,
    pub total_source_bytes: u64,
    pub total_dng_bytes: u64,
    pub space_reclaimed_bytes: u64,
    pub reports: Vec<DngConvertReportRecord>,
    pub error_messages: Vec<String>,
}

impl From<db::DngBatchReport> for DngBatchReportRecord {
    fn from(r: db::DngBatchReport) -> Self {
        Self {
            total_requested: r.total_requested,
            converted_count: r.converted_count,
            reconciled_existing_count: r.reconciled_existing_count,
            skipped_count: r.skipped_count,
            failed_count: r.failed_count,
            total_source_bytes: r.total_source_bytes,
            total_dng_bytes: r.total_dng_bytes,
            space_reclaimed_bytes: r.space_reclaimed_bytes,
            reports: r.reports.into_iter().map(Into::into).collect(),
            error_messages: r.error_messages,
        }
    }
}

#[uniffi::export(callback_interface)]
pub trait DngProgressCallback: Send + Sync + std::fmt::Debug {
    fn on_progress(&self, current: u32, total: u32, current_file: String);
}

#[uniffi::export]
pub fn convert_raw_to_dng(
    input_path: String,
    options: DngConvertOptionsRecord,
) -> Result<DngConvertReportRecord, FirstProofError> {
    let opts = crate::dng::DngConvertOptions {
        output_path: options.output_path.map(std::path::PathBuf::from),
        delete_original: options.delete_original,
        verify: options.verify,
        quiet: true,
    };
    crate::dng::convert_raw_to_dng(input_path, &opts)
        .map(Into::into)
        .map_err(|e| FirstProofError::DngError { reason: e })
}

#[uniffi::export]
pub fn batch_convert_catalog_to_dng(
    catalog_folder: String,
    photo_paths: Vec<String>,
    delete_original: bool,
    progress: Option<Box<dyn DngProgressCallback>>,
) -> Result<DngBatchReportRecord, FirstProofError> {
    let progress_fn = progress.as_ref().map(|cb| {
        move |curr: u32, tot: u32, file: &str| {
            cb.on_progress(curr, tot, file.to_string());
        }
    });
    let progress_ref: Option<&dyn Fn(u32, u32, &str)> = match &progress_fn {
        Some(f) => Some(f),
        None => None,
    };
    db::Database::batch_convert_catalog_to_dng(
        catalog_folder,
        &photo_paths,
        delete_original,
        progress_ref,
    )
    .map(Into::into)
    .map_err(|e| FirstProofError::DngError { reason: e })
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

    pub fn load_json_file(&self, path: String) -> bool {
        if let Ok(loaded) = lensfun::LensfunDatabase::from_json_file(&path) {
            let mut guard = self.inner.lock().unwrap();
            *guard = loaded;
            true
        } else {
            false
        }
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
pub fn get_global_lut_dir_path() -> String {
    lut::get_global_lut_dir().to_string_lossy().to_string()
}

#[uniffi::export]
pub fn discover_global_luts() -> Vec<LutRecord> {
    lut::discover_global_luts()
}

#[uniffi::export]
pub fn import_global_lut(source_path: String) -> Result<LutRecord, FirstProofError> {
    lut::import_global_lut(&source_path).map_err(|reason| FirstProofError::LutError { reason })
}

#[uniffi::export]
pub fn delete_global_lut(id_or_path: String) -> Result<bool, FirstProofError> {
    lut::delete_global_lut(&id_or_path).map_err(|reason| FirstProofError::LutError { reason })
}

#[uniffi::export]
pub fn rename_global_lut(id_or_path: String, new_title: String) -> Result<LutRecord, FirstProofError> {
    lut::rename_global_lut(&id_or_path, &new_title).map_err(|reason| FirstProofError::LutError { reason })
}

#[uniffi::export]
pub fn resolve_cube_lut(name_or_path: String) -> Option<CubeLutRecord> {
    lut::load_global_or_cube_lut(&name_or_path).ok().map(|l| {
        let data_rgba32f = l.to_rgba32f_bytes();
        CubeLutRecord {
            title: l.title,
            size: l.size as u32,
            data_rgba32f,
        }
    })
}

#[uniffi::export]
pub fn load_cube_lut(path: String) -> Option<CubeLutRecord> {
    resolve_cube_lut(path)
}

#[uniffi::export]
pub fn filter_derivative_clones(photos: Vec<PhotoRecord>) -> Vec<PhotoRecord> {
    if photos.len() <= 1 {
        return photos;
    }
    let db_records: Vec<db::PhotoRecord> = photos.into_iter().map(Into::into).collect();
    db::Database::filter_derivative_records_in_memory(db_records)
        .into_iter()
        .map(Into::into)
        .collect()
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct DuplicateClusterRecord {
    pub original: PhotoRecord,
    pub duplicates: Vec<PhotoRecord>,
}

#[uniffi::export]
pub fn find_duplicate_clusters(photos: Vec<PhotoRecord>) -> Vec<DuplicateClusterRecord> {
    if photos.len() <= 1 {
        return Vec::new();
    }
    let db_records: Vec<db::PhotoRecord> = photos.into_iter().map(Into::into).collect();
    db::Database::find_duplicate_clusters_in_memory(db_records)
        .into_iter()
        .map(|c| DuplicateClusterRecord {
            original: c.original.into(),
            duplicates: c.duplicates.into_iter().map(Into::into).collect(),
        })
        .collect()
}

#[uniffi::export]
pub fn normalize_stem(stem: String) -> String {
    db::normalize_stem(&stem)
}

/// Executes a canonical [`DarkroomCommand`] in headless mode and returns a structured JSON response.
#[uniffi::export]
pub fn execute_darkroom_command_headless(json_str: String) -> String {
    let cmd = match DarkroomCommand::from_json(&json_str) {
        Ok(c) => c,
        Err(e) => return CommandResponse::error(e).to_json(),
    };

    match cmd {
        DarkroomCommand::InspectMetadata { path } => {
            let p = match path {
                Some(p) => p,
                None => return CommandResponse::error("Missing photo path for metadata inspection").to_json(),
            };
            match raw::extract_metadata(&p) {
                Ok(meta) => CommandResponse::ok(serde_json::json!({
                    "metadata": {
                        "camera_make": meta.camera_make,
                        "camera_model": meta.camera_model,
                        "lens_model": meta.lens_model,
                        "iso": meta.iso,
                        "aperture": meta.aperture,
                        "shutter_speed": meta.shutter_speed,
                        "focal_length": meta.focal_length,
                        "orientation": meta.orientation,
                        "raw_width": meta.raw_width,
                        "raw_height": meta.raw_height,
                    }
                })).to_json(),
                Err(e) => CommandResponse::error(format!("Failed to extract metadata: {:?}", e)).to_json(),
            }
        }
        DarkroomCommand::DevelopPhoto { input, output, settings, fast } => {
            let in_path = std::path::PathBuf::from(&input);
            if !in_path.exists() {
                return CommandResponse::error(format!("Input file not found: {}", input)).to_json();
            }
            let out_path = output.unwrap_or_else(|| {
                let stem = in_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
                format!("{}_developed.tiff", stem)
            });
            let edits = settings.unwrap_or_default();
            let (demosaic_quality, render_tier) = if fast.unwrap_or(false) {
                (DemosaicQuality::Standard, RenderTier::Viewport)
            } else {
                (DemosaicQuality::Archival, RenderTier::ArchivalExport)
            };
            
            match export::export_photo_file_advanced(
                &input,
                &out_path,
                &edits,
                None,
                95,
                None,
                ExportColorSpace::Srgb,
                ExportRenderingIntent::Perceptual,
                false,
                demosaic_quality,
                render_tier,
            ) {
                Ok(_) => CommandResponse::ok(serde_json::json!({
                    "input": input,
                    "output_path": out_path,
                    "render_tier": format!("{:?}", render_tier),
                })).to_json(),
                Err(e) => CommandResponse::error(format!("Develop failed: {:?}", e)).to_json(),
            }
        }
        DarkroomCommand::ConvertToDng { input, output } => {
            let in_path = std::path::PathBuf::from(&input);
            if !in_path.exists() {
                return CommandResponse::error(format!("Input file not found: {}", input)).to_json();
            }
            let options = dng::DngConvertOptions {
                output_path: output.map(std::path::PathBuf::from),
                verify: true,
                ..Default::default()
            };
            match dng::convert_raw_to_dng(&input, &options) {
                Ok(report) => CommandResponse::ok(serde_json::json!({
                    "input": input,
                    "dng_path": report.dng_path.to_string_lossy().to_string(),
                    "source_size_bytes": report.source_size_bytes,
                    "dng_size_bytes": report.dng_size_bytes,
                    "compression_ratio": report.compression_ratio,
                    "verification_passed": report.verification_passed,
                })).to_json(),
                Err(e) => CommandResponse::error(format!("DNG conversion failed: {:?}", e)).to_json(),
            }
        }
        DarkroomCommand::ApplyEditDelta { delta } => {
            let mut settings = EditSettings::default();
            delta.apply_to(&mut settings);
            match serde_json::to_value(&settings) {
                Ok(val) => CommandResponse::ok(serde_json::json!({
                    "settings": val
                })).to_json(),
                Err(e) => CommandResponse::error(format!("Serialization failed: {}", e)).to_json(),
            }
        }
        DarkroomCommand::SetExposure { ev } => {
            let mut settings = EditSettings::default();
            settings.exposure = ev;
            match serde_json::to_value(&settings) {
                Ok(val) => CommandResponse::ok(serde_json::json!({
                    "exposure": ev,
                    "settings": val
                })).to_json(),
                Err(e) => CommandResponse::error(e.to_string()).to_json(),
            }
        }
        DarkroomCommand::SetTemperature { kelvin } => {
            let mut settings = EditSettings::default();
            settings.temperature = kelvin as f32;
            match serde_json::to_value(&settings) {
                Ok(val) => CommandResponse::ok(serde_json::json!({
                    "temperature": kelvin,
                    "settings": val
                })).to_json(),
                Err(e) => CommandResponse::error(e.to_string()).to_json(),
            }
        }
        DarkroomCommand::SetTint { tint } => {
            let mut settings = EditSettings::default();
            settings.tint = tint;
            match serde_json::to_value(&settings) {
                Ok(val) => CommandResponse::ok(serde_json::json!({
                    "tint": tint,
                    "settings": val
                })).to_json(),
                Err(e) => CommandResponse::error(e.to_string()).to_json(),
            }
        }
        DarkroomCommand::SetEmulsion { archetype, format, exposure } => {
            let mut settings = EditSettings::default();
            settings.emulsion_archetype = archetype;
            if let Some(f) = format { settings.emulsion_format = f; }
            if let Some(e) = exposure { settings.emulsion_exposure = e; }
            match serde_json::to_value(&settings) {
                Ok(val) => CommandResponse::ok(serde_json::json!({
                    "archetype": archetype,
                    "format": settings.emulsion_format,
                    "exposure": settings.emulsion_exposure,
                    "settings": val
                })).to_json(),
                Err(e) => CommandResponse::error(e.to_string()).to_json(),
            }
        }
        DarkroomCommand::SetPrintMedia { media, paper_grade, base_warmth, process_depth } => {
            let mut settings = EditSettings::default();
            settings.print_media = media;
            if let Some(g) = paper_grade { settings.print_paper_grade = g; }
            if let Some(w) = base_warmth { settings.print_base_warmth = w; }
            if let Some(d) = process_depth { settings.print_process_depth = d; }
            match serde_json::to_value(&settings) {
                Ok(val) => CommandResponse::ok(serde_json::json!({
                    "media": media,
                    "paper_grade": settings.print_paper_grade,
                    "base_warmth": settings.print_base_warmth,
                    "process_depth": settings.print_process_depth,
                    "settings": val
                })).to_json(),
                Err(e) => CommandResponse::error(e.to_string()).to_json(),
            }
        }
        other => {
            CommandResponse::error(format!(
                "Command '{:?}' requires an interactive darkroom session (connect via AppKit debug socket /tmp/firstproof-debug.sock)",
                other
            )).to_json()
        }
    }
}

// MARK: - ColorSync Soft Proofing UniFFI Exports

#[uniffi::export]
pub fn discover_color_sync_profiles() -> Vec<IccProfileRecord> {
    proof::discover_icc_profiles()
}

#[uniffi::export]
pub fn import_color_sync_profile(source_path: String) -> Result<IccProfileRecord, FirstProofError> {
    proof::import_icc_profile(&source_path)
        .map_err(|e| FirstProofError::ColorSyncError { reason: e })
}

#[uniffi::export]
pub fn generate_soft_proof_3d_lut(
    profile_id_or_path: String,
    intent: ProofIntent,
    source_is_p3: bool,
    display_is_p3: bool,
    size: u32,
) -> Result<CubeLutRecord, FirstProofError> {
    proof::generate_soft_proof_lut(&profile_id_or_path, intent, source_is_p3, display_is_p3, size)
        .map_err(|e| FirstProofError::ColorSyncError { reason: e })
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

