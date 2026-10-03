use wasm_bindgen::prelude::*;

pub mod scratchpad;

pub use firstproof_engine::{
    all_adjustments, evaluate_all_curves_lut, evaluate_curve_lut, extract_preview_slice,
    generate_xmp_sidecar, get_adjustment, get_jpeg_dimensions, kelvin_to_mired_norm,
    mired_norm_to_kelvin, parse_xmp_sidecar, AdjustmentDescriptor, AllCurvesLutRecord,
    AutoAdjustments, AutoWhiteBalanceResult, CfaPattern, ColorProfileMode, CurvePointRecord,
    EditSettings, EmulsionArchetype, EmulsionFormat, EnlargerFilterMode, ExifOrientationRecord,
    ExportColorSpace, ExportRenderingIntent, f16_to_f32, ImageMetadataRecord, PreviewResultRecord,
    PrintMediaMode, RawLinearResultRecord, RawSensorFrame, ScaleMode, ToneCurvesRecord,
    WorkspaceCategory, XmpData,
};
use scratchpad::with_scratchpad;

#[wasm_bindgen]
pub fn init_scratchpad(capacity_mb: u32) -> *mut u8 {
    let bytes = (capacity_mb as usize) * 1024 * 1024;
    with_scratchpad(|buf| {
        buf.resize(bytes);
        buf.ptr()
    })
}

#[wasm_bindgen]
pub fn get_scratchpad_ptr() -> *mut u8 {
    with_scratchpad(|buf| buf.ptr())
}

#[wasm_bindgen]
pub fn get_scratchpad_len() -> usize {
    with_scratchpad(|buf| buf.len())
}

#[wasm_bindgen]
pub fn get_scratchpad_u16_slice(offset: usize, u16_len: usize) -> js_sys::Uint16Array {
    with_scratchpad(|buf| {
        let byte_len = u16_len * 2;
        if let Some(slice) = buf.slice(offset, byte_len) {
            let u16_slice = unsafe {
                std::slice::from_raw_parts(slice.as_ptr() as *const u16, u16_len)
            };
            js_sys::Uint16Array::from(u16_slice)
        } else {
            js_sys::Uint16Array::new_with_length(0)
        }
    })
}

#[wasm_bindgen]
pub fn get_scratchpad_u8_slice(offset: usize, len: usize) -> js_sys::Uint8Array {
    with_scratchpad(|buf| {
        if let Some(slice) = buf.slice(offset, len) {
            js_sys::Uint8Array::from(slice)
        } else {
            js_sys::Uint8Array::new_with_length(0)
        }
    })
}

#[wasm_bindgen]
pub fn copy_into_scratchpad(offset: usize, bytes: &js_sys::Uint8Array) -> usize {
    with_scratchpad(|buf| {
        let len = bytes.byte_length() as usize;
        if offset + len > buf.len() {
            buf.resize(offset + len);
        }
        if let Some(slice) = buf.slice_mut(offset, len) {
            bytes.copy_to(slice);
            len
        } else {
            0
        }
    })
}

fn decode_raw_sensor_frame(slice: &[u8]) -> Option<RawSensorFrame> {
    RawSensorFrame::decode_from_bytes(slice)
}

fn decode_raw_binned_linear(slice: &[u8]) -> Option<(Vec<u16>, u32, u32)> {
    let frame = decode_raw_sensor_frame(slice)?;
    Some(frame.to_binned_linear_f16())
}

#[wasm_bindgen]
pub fn decode_raw_linear_from_scratchpad(input_len: usize) -> Result<JsValue, JsValue> {
    with_scratchpad(|buf| {
        let metadata = {
            let slice = buf
                .slice(0, input_len)
                .ok_or_else(|| JsValue::from_str("Input length exceeds scratchpad buffer capacity"))?;
            firstproof_engine::extract_metadata_from_bytes(slice)
        };

        let linear_res = {
            if let Some(slice) = buf.slice(0, input_len) {
                decode_raw_binned_linear(slice)
            } else {
                None
            }
        };

        if let Some((u16_pixels, bin_w, bin_h)) = linear_res {
            let bytes_needed = u16_pixels.len() * 2;
            let aligned_offset = (input_len + 15) & !15;
            let scratch_len = buf.len();
            if aligned_offset + bytes_needed > scratch_len {
                buf.resize(aligned_offset + bytes_needed);
            }
            if let Some(target_slice) = buf.slice_mut(aligned_offset, bytes_needed) {
                let u8_view = unsafe {
                    std::slice::from_raw_parts(
                        u16_pixels.as_ptr() as *const u8,
                        bytes_needed,
                    )
                };
                target_slice.copy_from_slice(u8_view);
                let res = RawLinearResultRecord {
                    linear_offset: aligned_offset as u64,
                    linear_len: bytes_needed as u64,
                    width: bin_w,
                    height: bin_h,
                    is_linear: true,
                    metadata,
                };
                web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                    "⚡ [RAW Linear Rec.2020 Event] Decoded uncompressed 16-bit linear RAW frame: {}x{} pixels ({} MB)",
                    bin_w, bin_h, (bytes_needed as f32 / 1024.0 / 1024.0)
                )));
                return serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()));
            }
        }

        Err(JsValue::from_str("Failed to decode 16-bit linear RAW frame"))
    })
}

fn decode_raw_full_linear(slice: &[u8]) -> Option<(Vec<u16>, u32, u32)> {
    let frame = decode_raw_sensor_frame(slice)?;
    Some(frame.to_full_demosaiced_linear_f16())
}

#[wasm_bindgen]
pub fn decode_raw_full_linear_from_scratchpad(input_len: usize) -> Result<JsValue, JsValue> {
    with_scratchpad(|buf| {
        let metadata = {
            let slice = buf
                .slice(0, input_len)
                .ok_or_else(|| JsValue::from_str("Input length exceeds scratchpad buffer capacity"))?;
            firstproof_engine::extract_metadata_from_bytes(slice)
        };

        let linear_res = {
            if let Some(slice) = buf.slice(0, input_len) {
                decode_raw_full_linear(slice)
            } else {
                None
            }
        };

        if let Some((u16_pixels, full_w, full_h)) = linear_res {
            let bytes_needed = u16_pixels.len() * 2;
            let aligned_offset = (input_len + 15) & !15;
            let scratch_len = buf.len();
            if aligned_offset + bytes_needed > scratch_len {
                buf.resize(aligned_offset + bytes_needed);
            }
            if let Some(target_slice) = buf.slice_mut(aligned_offset, bytes_needed) {
                let u8_view = unsafe {
                    std::slice::from_raw_parts(
                        u16_pixels.as_ptr() as *const u8,
                        bytes_needed,
                    )
                };
                target_slice.copy_from_slice(u8_view);
                let res = RawLinearResultRecord {
                    linear_offset: aligned_offset as u64,
                    linear_len: bytes_needed as u64,
                    width: full_w,
                    height: full_h,
                    is_linear: true,
                    metadata,
                };
                web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                    "⚡ [RAW Full Demosaic Rec.2020 Event] Decoded full-resolution 16-bit linear RAW frame: {}x{} pixels ({} MB)",
                    full_w, full_h, (bytes_needed as f32 / 1024.0 / 1024.0)
                )));
                return serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()));
            }
        }

        Err(JsValue::from_str("Failed to decode full-resolution 16-bit linear RAW frame"))
    })
}

#[wasm_bindgen]
pub fn decode_preview_from_scratchpad(input_len: usize) -> Result<JsValue, JsValue> {
    with_scratchpad(|buf| {
        let metadata = {
            let slice = buf
                .slice(0, input_len)
                .ok_or_else(|| JsValue::from_str("Input length exceeds scratchpad buffer capacity"))?;
            firstproof_engine::extract_metadata_from_bytes(slice)
        };

        // 1. Try extracting high-res embedded preview JPEG
        let mut preview_offset = 0u64;
        let mut preview_len = 0u64;
        let mut width = 0u32;
        let mut height = 0u32;

        {
            if let Some(slice) = buf.slice(0, input_len) {
                if let Some((off, len)) = extract_preview_slice(slice) {
                    let preview_bytes = &slice[off..off + len];
                    if let Some((w, h)) = get_jpeg_dimensions(preview_bytes) {
                        if w >= 320 && h >= 240 {
                            preview_offset = off as u64;
                            preview_len = len as u64;
                            width = w;
                            height = h;
                        }
                    }
                }
            }
        }

        // 2. If embedded preview is missing or low-res (< 1920x1080), decode full RAW Bayer sensor frame via rawler
        if width == 0 || height == 0 {
            let raw_res = {
                if let Some(slice) = buf.slice(0, input_len) {
                    decode_raw_binned_linear(slice)
                } else {
                    None
                }
            };

            if let Some((u16_pixels, bin_w, bin_h)) = raw_res {
                let mut rgb_bytes = vec![0u8; (bin_w * bin_h * 3) as usize];
                for i in 0..(bin_w * bin_h) as usize {
                    let r_lin = f16_to_f32(u16_pixels[i * 4]);
                    let g_lin = f16_to_f32(u16_pixels[i * 4 + 1]);
                    let b_lin = f16_to_f32(u16_pixels[i * 4 + 2]);
                    rgb_bytes[i * 3] = (r_lin.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0) as u8;
                    rgb_bytes[i * 3 + 1] = (g_lin.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0) as u8;
                    rgb_bytes[i * 3 + 2] = (b_lin.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0) as u8;
                }
                let mut jpeg_buf = Vec::new();
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_buf, 90);
                if encoder.encode(&rgb_bytes, bin_w, bin_h, image::ColorType::Rgb8).is_ok() {
                    let raw_len = jpeg_buf.len();
                    let scratch_len = buf.len();
                    if input_len + raw_len > scratch_len {
                        buf.resize(input_len + raw_len);
                    }
                    if let Some(target_slice) = buf.slice_mut(input_len, raw_len) {
                        target_slice.copy_from_slice(&jpeg_buf);
                        preview_offset = input_len as u64;
                        preview_len = raw_len as u64;
                        width = bin_w;
                        height = bin_h;
                    }
                }
            }
        }

        // 3. Fallback to embedded slice if rawler decoding was unavailable for this container format
        if width == 0 || height == 0 {
            if let Some(slice) = buf.slice(0, input_len) {
                if let Some((off, len)) = extract_preview_slice(slice) {
                    let preview_bytes = &slice[off..off + len];
                    if let Some((w, h)) = get_jpeg_dimensions(preview_bytes) {
                        preview_offset = off as u64;
                        preview_len = len as u64;
                        width = w;
                        height = h;
                    }
                }
            }
        }

        let res = PreviewResultRecord {
            preview_offset,
            preview_len,
            width,
            height,
            is_jpeg: true,
            metadata,
        };

        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

#[wasm_bindgen]
pub fn evaluate_curves(curves_json: &str) -> Result<JsValue, JsValue> {
    let curves: ToneCurvesRecord = serde_json::from_str(curves_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid curves JSON: {}", e)))?;
    let lut = evaluate_all_curves_lut(&curves);
    serde_wasm_bindgen::to_value(&lut).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn parse_xmp(xml: &str) -> Result<JsValue, JsValue> {
    let xmp = parse_xmp_sidecar(xml).map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&xmp).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn generate_xmp(xmp_json: &str) -> Result<String, JsValue> {
    let xmp: XmpData = serde_json::from_str(xmp_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid XMP JSON: {}", e)))?;
    Ok(generate_xmp_sidecar(&xmp))
}

fn is_curve_active(pts: &[CurvePointRecord]) -> bool {
    pts.len() > 2
        || (pts.len() == 2
            && ((pts[0].x - pts[0].y).abs() > 1e-4 || (pts[1].x - pts[1].y).abs() > 1e-4))
}

fn resolve_curves_lut(
    edits: &EditSettings,
    curves_lut_json: Option<String>,
) -> Result<Option<AllCurvesLutRecord>, JsValue> {
    if let Some(s) = curves_lut_json {
        if !s.is_empty() {
            let lut: AllCurvesLutRecord = serde_json::from_str(&s)
                .map_err(|e| JsValue::from_str(&format!("Invalid curves LUT JSON: {}", e)))?;
            return Ok(Some(lut));
        }
    }
    if is_curve_active(&edits.curves.master)
        || is_curve_active(&edits.curves.red)
        || is_curve_active(&edits.curves.green)
        || is_curve_active(&edits.curves.blue)
    {
        Ok(Some(evaluate_all_curves_lut(&edits.curves)))
    } else {
        Ok(None)
    }
}

#[wasm_bindgen]
pub fn evaluate_reference_frame(
    input_data: &[u8],
    is_linear_rec2020: bool,
    width: u32,
    height: u32,
    edits_json: &str,
    curves_lut_json: Option<String>,
) -> Result<Vec<u8>, JsValue> {
    let edits: EditSettings = serde_json::from_str(edits_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid edits JSON: {}", e)))?;
    let curves_lut = resolve_curves_lut(&edits, curves_lut_json)?;
    Ok(firstproof_engine::develop_frame_pipeline(
        input_data,
        is_linear_rec2020,
        width,
        height,
        &edits,
        curves_lut.as_ref(),
    ))
}

#[wasm_bindgen]
pub fn evaluate_reference_pixel(
    r: f32,
    g: f32,
    b: f32,
    is_linear_rec2020: bool,
    edits_json: &str,
    curves_lut_json: Option<String>,
) -> Result<Vec<u8>, JsValue> {
    let edits: EditSettings = serde_json::from_str(edits_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid edits JSON: {}", e)))?;
    let curves_lut = resolve_curves_lut(&edits, curves_lut_json)?;
    let rgb = firstproof_engine::develop_pixel_pipeline(
        r,
        g,
        b,
        is_linear_rec2020,
        &edits,
        curves_lut.as_ref(),
        None,
    );
    Ok(rgb.to_vec())
}

#[wasm_bindgen]
pub fn evaluate_display_luma(
    r: f32,
    g: f32,
    b: f32,
    is_linear_rec2020: bool,
    edits_json: &str,
) -> Result<f32, JsValue> {
    let edits: EditSettings = serde_json::from_str(edits_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid edits JSON: {}", e)))?;
    Ok(firstproof_engine::evaluate_display_luma(
        r,
        g,
        b,
        is_linear_rec2020,
        &edits,
    ))
}

#[wasm_bindgen]
pub fn compute_auto_adjustments_from_u16(data: &js_sys::Uint16Array, width: u32, height: u32) -> JsValue {
    let u16_len = (width * height * 4) as usize;
    let target_len = u16_len.min(data.length() as usize);
    let mut vec = vec![0u16; target_len];
    data.copy_to(&mut vec);
    let adj = firstproof_engine::auto::compute_auto_adjustments_from_u16_slice(width, height, &vec);
    serde_wasm_bindgen::to_value(&adj).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn compute_auto_adjustments_from_luminances(lums: &[f32]) -> JsValue {
    let adj = firstproof_engine::auto::compute_auto_adjustments_from_luminances(lums.to_vec());
    serde_wasm_bindgen::to_value(&adj).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn compute_auto_white_balance_from_u16(data: &js_sys::Uint16Array, width: u32, height: u32) -> JsValue {
    let u16_len = (width * height * 4) as usize;
    let target_len = u16_len.min(data.length() as usize);
    let mut vec = vec![0u16; target_len];
    data.copy_to(&mut vec);
    let wb = firstproof_engine::auto::compute_auto_white_balance_from_u16_slice(width, height, &vec);
    serde_wasm_bindgen::to_value(&wb).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn calculate_white_balance_from_sample(r: f32, g: f32, b: f32) -> JsValue {
    let wb = firstproof_engine::auto::calculate_white_balance_from_sample(r, g, b);
    serde_wasm_bindgen::to_value(&wb).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn decode_exif_orientation(orientation: u32) -> JsValue {
    let rec = firstproof_engine::decode_exif_orientation(orientation);
    serde_wasm_bindgen::to_value(&rec).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn extract_metadata_from_bytes(bytes: &[u8]) -> JsValue {
    let meta = firstproof_engine::extract_metadata_from_bytes(bytes);
    serde_wasm_bindgen::to_value(&meta).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn get_all_adjustments() -> JsValue {
    serde_wasm_bindgen::to_value(&all_adjustments()).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn get_adjustment_descriptor(id: &str) -> JsValue {
    serde_wasm_bindgen::to_value(&get_adjustment(id)).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn convert_kelvin_to_mired_norm(kelvin: f64) -> f64 {
    kelvin_to_mired_norm(kelvin)
}

#[wasm_bindgen]
pub fn convert_mired_norm_to_kelvin(norm: f64) -> f64 {
    mired_norm_to_kelvin(norm)
}

#[wasm_bindgen(typescript_custom_section)]
const TS_DEFINITIONS: &'static str = r#"
export interface ExifOrientationRecord {
    rotation: number;
    flip_horizontal: boolean;
    flip_vertical: boolean;
}

export interface ImageMetadataRecord {
    camera_make?: string;
    camera_model?: string;
    lens_model?: string;
    iso?: number;
    aperture?: number;
    shutter_speed?: string;
    focal_length?: number;
    taken_time?: number;
    orientation?: number;
    raw_width?: number;
    raw_height?: number;
}

export interface PreviewResultRecord {
    preview_offset: number;
    preview_len: number;
    width: number;
    height: number;
    is_jpeg: boolean;
    metadata: ImageMetadataRecord;
}

export interface RawLinearResultRecord {
    linear_offset: number;
    linear_len: number;
    width: number;
    height: number;
    is_linear: boolean;
    metadata: ImageMetadataRecord;
}

export interface CurvePointRecord {
    x: number;
    y: number;
}

export interface ToneCurvesRecord {
    master: CurvePointRecord[];
    red: CurvePointRecord[];
    green: CurvePointRecord[];
    blue: CurvePointRecord[];
    [key: string]: CurvePointRecord[];
}

export interface AllCurvesLutRecord {
    master: number[];
    red: number[];
    green: number[];
    blue: number[];
}

export type ColorProfileMode =
    | 'AgxVivid'
    | 'AgxFilmic'
    | 'CameraStandard'
    | 'CameraPortrait'
    | 'CameraLandscape'
    | 'CameraVivid';

export type ExportColorSpace =
    | 'Srgb'
    | 'DisplayP3';

export type ExportRenderingIntent =
    | 'Perceptual'
    | 'RelativeColorimetric';

export type EmulsionArchetype =
    | 'None'
    | 'RochesterPortrait400'
    | 'AshigaraChrome50'
    | 'RochesterK14Chrome64'
    | 'RochesterCinema500'
    | 'RochesterDaylight200'
    | 'MobberleySilver400'
    | 'CambridgeDiffusion100'
    | 'LeverkusenPortrait160'
    | 'AshigaraPortrait400'
    | 'LiguriaCinema80'
    | 'BaodingSilver100';

export type EmulsionFormat =
    | 'Format35mm'
    | 'MediumFormat'
    | 'LargeFormat4x5';

export type EmulsionGrainMode =
    | 'Off'
    | 'SeedFromSensor'
    | 'SynthesizePhysical';

export type PrintMediaMode =
    | 'None'
    | 'ChromogenicTypeC'
    | 'SilverDyeBleach'
    | 'FineArtCottonRag'
    | 'BarytaSilverGelatin'
    | 'WarmtoneChlorobromide'
    | 'PlatinumPalladium'
    | 'Cyanotype'
    | 'BamakoSilverContact';

export type EnlargerFilterMode =
    | 'NeutralPanchromatic'
    | 'YellowK2'
    | 'Orange16'
    | 'Red25A'
    | 'GreenX1'
    | 'OrthochromaticVintage';

export interface EditSettings {
    exposure: number;
    contrast: number;
    highlights: number;
    shadows: number;
    whites: number;
    blacks: number;
    vibrance: number;
    saturation: number;
    deep_blue: number;
    foliage: number;
    temperature: number;
    tint: number;
    crop_left: number;
    crop_top: number;
    crop_right: number;
    crop_bottom: number;
    crop_angle: number;
    perspective_vertical: number;
    perspective_horizontal: number;
    crop_constrain: boolean;
    rotation: number;
    flip_horizontal: boolean;
    flip_vertical: boolean;
    tone_profile: ColorProfileMode;
    lut_name?: string;
    lut_intensity: number;
    sharpening_amount: number;
    sharpening_radius: number;
    sharpening_masking: number;
    noise_reduction: number;
    emulsion_archetype: EmulsionArchetype;
    emulsion_format: EmulsionFormat;
    emulsion_exposure: number;
    emulsion_grain_mode: EmulsionGrainMode;
    grain_scale: number;
    curves: ToneCurvesRecord;
    split_toning_shadow_hue: number;
    split_toning_shadow_sat: number;
    split_toning_highlight_hue: number;
    split_toning_highlight_sat: number;
    split_toning_balance: number;
    print_media: PrintMediaMode;
    enlarger_filter: EnlargerFilterMode;
    print_paper_grade: number;
    print_base_warmth: number;
    print_process_depth: number;
}

export interface XmpData {
    rating?: number;
    status?: string;
    title?: string;
    description?: string;
    creator?: string;
    copyright?: string;
    keywords?: string[];
    latitude?: number;
    longitude?: number;
    edits: EditSettings;
}

export interface AutoAdjustments {
    exposure: number;
    contrast: number;
}

export interface AutoWhiteBalanceResult {
    temperature: number;
    tint: number;
}

export type WorkspaceCategory = 'Light' | 'Color' | 'Detail' | 'Emulsion';
export type ScaleMode = 'Linear' | 'Mired';

export interface AdjustmentDescriptor {
    id: string;
    name: string;
    category: WorkspaceCategory;
    min: number;
    max: number;
    step: number;
    default: number;
    scale_mode: ScaleMode;
    unit: string;
    precision: number;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scratchpad_allocation() {
        with_scratchpad(|buf| {
            assert!(buf.len() >= 64 * 1024 * 1024);
            let ptr = buf.ptr();
            assert!(!ptr.is_null());
            buf.resize(300 * 1024 * 1024);
            assert_eq!(buf.len(), 300 * 1024 * 1024);
        });
    }

    #[test]
    fn test_curve_evaluation_parity() {
        let mut curves = ToneCurvesRecord::default();
        curves.master = vec![
            CurvePointRecord { x: 0.0, y: 0.0 },
            CurvePointRecord { x: 0.25, y: 0.20 },
            CurvePointRecord { x: 0.75, y: 0.80 },
            CurvePointRecord { x: 1.0, y: 1.0 },
        ];
        let lut = evaluate_all_curves_lut(&curves);
        assert_eq!(lut.master.len(), 256);
        assert_eq!(lut.red.len(), 256);
        assert_eq!(lut.green.len(), 256);
        assert_eq!(lut.blue.len(), 256);

        assert!((lut.master[0] - 0.0).abs() < 1e-4);
        assert!((lut.master[255] - 1.0).abs() < 1e-4);
        for i in 1..256 {
            assert!(lut.master[i] >= lut.master[i - 1] - 1e-5);
        }
    }

    #[test]
    fn test_xmp_roundtrip() {
        let mut xmp = XmpData::default();
        xmp.rating = Some(5);
        xmp.status = "Pick".to_string();
        xmp.title = Some("Alpine Sunset".to_string());
        xmp.edits.exposure = 0.65;
        xmp.edits.contrast = 15.0;
        xmp.edits.highlights = -25.0;
        xmp.edits.shadows = 20.0;
        xmp.edits.temperature = 6200.0;
        xmp.edits.tint = -8.0;
        xmp.edits.rotation = 90;
        xmp.edits.flip_horizontal = true;

        let xml = generate_xmp_sidecar(&xmp);
        assert!(xml.contains("<xmp:Rating>5</xmp:Rating>"));
        assert!(xml.contains("<pm:status>Pick</pm:status>"));
        assert!(xml.contains("<pm:exposure>0.6500</pm:exposure>"));

        let parsed = parse_xmp_sidecar(&xml).expect("Failed to parse generated XMP");
        assert_eq!(parsed.rating, Some(5));
        assert_eq!(parsed.status, "Pick");
        assert_eq!(parsed.title, Some("Alpine Sunset".to_string()));
        assert!((parsed.edits.exposure - 0.65).abs() < 1e-3);
        assert!((parsed.edits.contrast - 15.0).abs() < 1e-3);
        assert!((parsed.edits.temperature - 6200.0).abs() < 1e-3);
        assert_eq!(parsed.edits.rotation, 90);
        assert!(parsed.edits.flip_horizontal);
        assert!(!parsed.edits.flip_vertical);
    }

    #[test]
    fn test_wasm_decode_exif_orientation() {
        let r1 = firstproof_engine::decode_exif_orientation(1);
        assert_eq!(r1.rotation, 0);
        assert!(!r1.flip_horizontal);
        assert!(!r1.flip_vertical);

        let r6 = firstproof_engine::decode_exif_orientation(6);
        assert_eq!(r6.rotation, 90);
        assert!(!r6.flip_horizontal);
        assert!(!r6.flip_vertical);

        let r8 = firstproof_engine::decode_exif_orientation(8);
        assert_eq!(r8.rotation, 270);
        assert!(!r8.flip_horizontal);
        assert!(!r8.flip_vertical);
    }

    #[test]
    fn test_wasm_extract_metadata_from_bytes() {
        let mut bytes = vec![0u8; 256];
        bytes[0..4].copy_from_slice(b"II\x2A\x00");
        bytes[4..8].copy_from_slice(&8u32.to_le_bytes());

        let entry_count = 1u16;
        bytes[8..10].copy_from_slice(&entry_count.to_le_bytes());
        bytes[10..12].copy_from_slice(&0x0112u16.to_le_bytes()); // Orientation
        bytes[12..14].copy_from_slice(&3u16.to_le_bytes());      // SHORT
        bytes[14..18].copy_from_slice(&1u32.to_le_bytes());      // Count
        bytes[18..22].copy_from_slice(&8u32.to_le_bytes());      // Value = 8 (270 deg)

        let meta = firstproof_engine::extract_metadata_from_bytes(&bytes);
        assert_eq!(meta.orientation, Some(8));
    }

    #[test]
    fn test_wasm_reference_pipeline_evaluation() {
        let edits = EditSettings::default();
        let edits_json = serde_json::to_string(&edits).unwrap();

        // sRGB through default AgX Vivid:
        let rgb_neutral = evaluate_reference_pixel(0.5, 0.5, 0.5, false, &edits_json, None).unwrap();
        assert_eq!(rgb_neutral[0], rgb_neutral[1]);
        assert_eq!(rgb_neutral[1], rgb_neutral[2]);

        // Linear Rec.2020 AgX Vivid evaluation:
        let rgb_agx = evaluate_reference_pixel(0.18, 0.18, 0.18, true, &edits_json, None).unwrap();
        // 0.18 mid gray through AgX Vivid should be around 125-135 display sRGB
        assert!(rgb_agx[0] >= 120 && rgb_agx[0] <= 140);
        assert_eq!(rgb_agx[0], rgb_agx[1]);
        assert_eq!(rgb_agx[1], rgb_agx[2]);

        // Display luma evaluation:
        let luma = evaluate_display_luma(0.18, 0.18, 0.18, true, &edits_json).unwrap();
        assert!(luma > 0.15 && luma < 0.35);

        // Frame evaluation:
        let input_rgba = vec![128u8, 128u8, 128u8, 255u8];
        let frame_out = evaluate_reference_frame(&input_rgba, false, 1, 1, &edits_json, None).unwrap();
        assert_eq!(frame_out.len(), 4);
        assert_eq!(frame_out[0], rgb_neutral[0]);
        assert_eq!(frame_out[1], rgb_neutral[1]);
        assert_eq!(frame_out[2], rgb_neutral[2]);
        assert_eq!(frame_out[3], 255);
    }

    #[test]
    fn test_schema_adjustments() {
        let adjustments = all_adjustments();
        assert!(!adjustments.is_empty());
        let exp = get_adjustment("exposure").expect("exposure adjustment exists");
        assert_eq!(exp.id, "exposure");
        assert_eq!(exp.category, WorkspaceCategory::Light);
        assert_eq!(exp.unit, "EV");

        assert!((convert_kelvin_to_mired_norm(2000.0) - 0.0).abs() < 1e-4);
        assert_eq!(convert_mired_norm_to_kelvin(0.0), 2000.0);
    }
}
