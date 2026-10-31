# First Proof Core FFI API Reference

> **Agent Invariant**: Never open `firstproof_core.swift` (4,400+ lines). Refer to this document or `firstproof-core/src/lib.rs` directly.

## Enums & Core Types
- `ExportColorSpace`: `Srgb`, `DisplayP3`
- `ExportRenderingIntent`: `Perceptual`, `RelativeColorimetric`
- `ProofIntent`: `RelativeColorimetric`, `Perceptual`, `AbsoluteColorimetric`
- `IccProfileRecord`: ColorSync / ICC profile descriptor (`name`, `description`, `path`, `is_display`).
- `EmulsionArchetype`: Film stock simulation presets (`Neutral`, `RochesterDaylight200`, `RochesterChrome64`, `TokyoSuperia400`, `AshigaraVelvia50`, `AshigaraPro400H`, `MobberleyHP5`, `MobberleyPanF`, `BaodingSilver100`).
- `PrintMediaMode`: Fine-art print media archetypes (`Bypass`, `ChromogenicTypeC`, `SilverDyeBleachClassic`, `BarytaSilverGelatinFb`, `WarmtoneChlorobromide`, `PlatinumPalladium`, `HistoricCyanotype`, `BamakoSilverContact`).
- `EmulsionFormat`: Film formats for optical grain and MTF scaling (`Format35mm`, `Format120Medium`, `FormatLargeFormat4x5`, `FormatSuper16`).
- `EmulsionGrainMode`: Legacy grain synthesis modes (`SeedFromSensor`, `SynthesizePhysical`).
- `RenderTier`: Pipeline processing tiers (`InteractiveViewport`, `ArchivalExport`).
- `DemosaicQuality`: RAW demosaicing algorithms (`BilinearBinned`, `HamiltonAdams`, `Rcd`).
- `EditSettings`: Complete adjustment state (exposure, contrast, highlights, shadows, whites, blacks, saturation, vibrance, temp, tint, crop rect/angle, tone curves, LUT settings, grain, emulsion, print media, lens correction, perspective).
- `ToneCurvesRecord` / `CurvePointRecord`: Parametric master + RGB curve points.
- `CubeLutRecord`: Loaded 3D LUT (title, size, `data_rgba32f`).
- `RawPreviewRecord`: 16-bit linear decoded preview (`width`, `height`, `data`, `is_linear_float`).
- `LensCorrectionCoefficientsRecord`: Lensfun calibration values (distortion, tca, vignetting).

## Export & Pipeline Functions
- `export_photo(input_path, output_path, edits, lens_coeffs, quality, max_dimension, color_space, rendering_intent, ultra_hdr) -> bool`
- `export_batch_photos(items: Vec<BatchExportItemRecord>) -> Vec<BatchExportResultRecord>`
- `decode_raw_preview(path, max_dimension) -> Option<RawPreviewRecord>`
- `compute_white_balance_sampled(path, sample_x, sample_y, radius) -> Option<WhiteBalanceResult>`
- `compute_auto_adjustments(path) -> Option<AutoAdjustments>`
- `compute_auto_adjustments_from_buffer(w, h, data) -> Option<AutoAdjustments>`

## XMP & Metadata Functions
- `read_xmp(image_path) -> Option<XmpData>`
- `write_xmp(image_path, data: XmpData) -> bool`
- `find_xmp_path(image_path) -> Option<String>`
- `extract_xmp_edits(image_path) -> Option<EditSettings>`
- `save_xmp_edits(image_path, edits: EditSettings) -> bool`
- `save_xmp_metadata(image_path, rating, status, title, desc, creator, copyright, keywords, lat, lon) -> bool`

## Database & Catalog (`FirstProofDatabase`)
- `FirstProofDatabase::open(db_path) -> Arc<Self>`
- `.index_folder(folder_path) -> u32`
- `.get_photo(file_path) -> Option<PhotoRecord>`
- `.get_all_photos() -> Vec<PhotoRecord>`
- `.update_rating_status(file_path, rating, status) -> bool`
- `.update_photo_metadata(file_path, title, desc, creator, copyright, keywords) -> bool`
- `.update_rating_status_batch(file_paths, rating, update_rating, status) -> u32`
- `.export_portable_package(source_paths, dest_folder, move_files) -> Result<ExportPackageReport>`
- `resolve_catalog_path(folder_path, portable) -> String`

## Optics & Film Emulation
- `FirstProofLensfun::new() -> Arc<Self>`
  - `.load_directory(dir_path) -> u32`
  - `.load_json_file(path) -> bool`
  - `.resolve_coefficients(camera_make, camera_model, lens_model, focal_length, aperture) -> LensCorrectionCoefficientsRecord`
- `get_builtin_cube_lut_titles() -> Vec<String>`
- `get_builtin_cube_lut(title, size) -> Option<CubeLutRecord>`
- `load_cube_lut(path) -> Option<CubeLutRecord>`
