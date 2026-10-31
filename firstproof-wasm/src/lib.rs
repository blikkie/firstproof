use wasm_bindgen::prelude::*;
use image::ImageEncoder;

pub mod scratchpad;

pub use firstproof_engine::{
    all_adjustments, develop_raster_image_archival, evaluate_all_curves_lut, evaluate_curve_lut,
    extract_preview_slice, generate_xmp_sidecar, get_adjustment, get_jpeg_dimensions,
    kelvin_to_mired_norm, mired_norm_to_kelvin, parse_xmp_sidecar, AdjustmentDescriptor,
    AllCurvesLutRecord, AutoAdjustments, AutoWhiteBalanceResult, CfaPattern,
    CropAspectRatioPreset, CurvePointRecord, DemosaicQuality, EditSettings, EmulsionArchetype,
    EmulsionFormat, EnlargerFilterMode, EvHistogramRecord, ExifOrientationRecord, ExportColorSpace,
    ExportRenderingIntent, f16_to_f32, f32_to_f16, ImageMetadataRecord, LensCorrectionCoefficients,
    LensfunDatabase, PreviewResultRecord, PrintMediaMode, RawLinearResultRecord, RawSensorFrame,
    RemjetBackingMode, RenderTier, ScaleMode, ToneCurvesRecord, WhiteBalancePreset,
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

pub fn copy_into_scratchpad_bytes(offset: usize, bytes: &[u8]) -> usize {
    with_scratchpad(|buf| {
        let len = bytes.len();
        if offset + len > buf.len() {
            buf.resize(offset + len);
        }
        if let Some(slice) = buf.slice_mut(offset, len) {
            slice.copy_from_slice(bytes);
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ArchivalExportResultRecord {
    pub linear_offset: u64,
    pub linear_len: u64,
    pub width: u32,
    pub height: u32,
}

#[wasm_bindgen]
pub fn develop_raw_archival_from_scratchpad(
    input_len: usize,
    edits_json: &str,
    quality: u32,
) -> Result<JsValue, JsValue> {
    let edits: EditSettings = serde_json::from_str(edits_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid edits JSON: {}", e)))?;

    let quality_mode = if quality == 1 {
        DemosaicQuality::Archival
    } else {
        DemosaicQuality::Standard
    };

    with_scratchpad(|buf| {
        let slice = buf
            .slice(0, input_len)
            .ok_or_else(|| JsValue::from_str("Input length exceeds scratchpad buffer capacity"))?;

        let (w, h, u16_data) = if let Some(frame) = RawSensorFrame::decode_from_bytes(slice) {
            firstproof_engine::develop_raw_sensor_frame_archival(
                &frame,
                &edits,
                quality_mode,
            )
        } else if let Ok(dyn_img) = image::load_from_memory(slice) {
            let rgb = dyn_img.to_rgb8();
            firstproof_engine::develop_raster_image_archival(
                rgb.as_raw(),
                rgb.width() as usize,
                rgb.height() as usize,
                &edits,
            )
        } else {
            return Err(JsValue::from_str("Failed to decode image from bytes"));
        };

        let bytes_needed = u16_data.len() * 2;
        let aligned_offset = (input_len + 15) & !15;
        let scratch_len = buf.len();
        if aligned_offset + bytes_needed > scratch_len {
            buf.resize(aligned_offset + bytes_needed);
        }
        if let Some(target_slice) = buf.slice_mut(aligned_offset, bytes_needed) {
            let u8_view = unsafe {
                std::slice::from_raw_parts(
                    u16_data.as_ptr() as *const u8,
                    bytes_needed,
                )
            };
            target_slice.copy_from_slice(u8_view);
        }

        let res = ArchivalExportResultRecord {
            linear_offset: aligned_offset as u64,
            linear_len: bytes_needed as u64,
            width: w as u32,
            height: h as u32,
        };

        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ExportResultRecord {
    pub output_offset: u64,
    pub output_len: u64,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[wasm_bindgen]
pub fn export_raw_from_scratchpad(
    input_len: usize,
    edits_json: &str,
    format_str: &str,
    quality: u32,
    size_option: &str,
    custom_dim: u32,
    demosaic_quality: u32,
) -> Result<JsValue, JsValue> {
    let edits: EditSettings = serde_json::from_str(edits_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid edits JSON: {}", e)))?;

    let quality_mode = if demosaic_quality == 1 {
        DemosaicQuality::Archival
    } else {
        DemosaicQuality::Standard
    };

    with_scratchpad(|buf| {
        let slice = buf
            .slice(0, input_len)
            .ok_or_else(|| JsValue::from_str("Input length exceeds scratchpad buffer capacity"))?;

        // Multi-pass archival development (RAW sensor frame or standard raster image)
        let (dev_w, dev_h, dev_u16) = if let Some(frame) = RawSensorFrame::decode_from_bytes(slice) {
            firstproof_engine::develop_raw_sensor_frame_archival(
                &frame,
                &edits,
                quality_mode,
            )
        } else if let Ok(dyn_img) = image::load_from_memory(slice) {
            let rgb = dyn_img.to_rgb8();
            firstproof_engine::develop_raster_image_archival(
                rgb.as_raw(),
                rgb.width() as usize,
                rgb.height() as usize,
                &edits,
            )
        } else {
            return Err(JsValue::from_str("Unsupported image format: neither RAW sensor nor valid raster image"));
        };

        // 3. Compute geometry & target dimensions
        let rot = edits.rotation.rem_euclid(360);
        let is_orthogonal = rot == 90 || rot == 270;
        let base_w = if is_orthogonal { dev_h } else { dev_w };
        let base_h = if is_orthogonal { dev_w } else { dev_h };

        let crop_l = edits.crop_left.clamp(0.0, 0.99);
        let crop_t = edits.crop_top.clamp(0.0, 0.99);
        let crop_r = edits.crop_right.clamp(crop_l + 0.01, 1.0);
        let crop_b = edits.crop_bottom.clamp(crop_t + 0.01, 1.0);

        let effective_w = ((base_w as f32) * (crop_r - crop_l)).round().max(1.0) as usize;
        let effective_h = ((base_h as f32) * (crop_b - crop_t)).round().max(1.0) as usize;

        let max_dim: Option<usize> = match size_option {
            "2048" => Some(2048),
            "3840" => Some(3840),
            "Custom" if custom_dim > 0 => Some(custom_dim as usize),
            _ => None,
        };

        let (target_w, target_h) = if let Some(max_d) = max_dim {
            if effective_w > max_d || effective_h > max_d {
                if effective_w >= effective_h {
                    let tw = max_d;
                    let th = ((effective_h * max_d) as f32 / effective_w as f32).round().max(1.0) as usize;
                    (tw, th)
                } else {
                    let th = max_d;
                    let tw = ((effective_w * max_d) as f32 / effective_h as f32).round().max(1.0) as usize;
                    (tw, th)
                }
            } else {
                (effective_w, effective_h)
            }
        } else {
            (effective_w, effective_h)
        };

        // 4. Bilinear geometry resampling in Rust (including crop_angle, perspective, and crop_constrain)
        let resampled_u16 = firstproof_engine::resample_rgb16_with_geometry(
            dev_w,
            dev_h,
            &dev_u16,
            rot,
            crop_l,
            crop_t,
            crop_r,
            crop_b,
            edits.crop_angle,
            edits.perspective_vertical,
            edits.perspective_horizontal,
            edits.crop_constrain,
            edits.flip_horizontal,
            edits.flip_vertical,
            target_w,
            target_h,
        );

        // 5. Encode output file (JPEG or TIFF)
        let is_tiff = format_str.eq_ignore_ascii_case("TIFF");
        let encoded_bytes = if is_tiff {
            let mut bytes = Vec::with_capacity(resampled_u16.len() * 2);
            for &val in &resampled_u16 {
                bytes.extend_from_slice(&val.to_le_bytes());
            }
            let mut out = Vec::new();
            let encoder = image::codecs::tiff::TiffEncoder::new(std::io::Cursor::new(&mut out));
            encoder
                .write_image(&bytes, target_w as u32, target_h as u32, image::ColorType::Rgb16.into())
                .map_err(|e| JsValue::from_str(&format!("Failed to encode TIFF: {}", e)))?;
            out
        } else {
            let mut rgb8 = Vec::with_capacity(target_w * target_h * 3);
            for &val in &resampled_u16 {
                rgb8.push((val >> 8) as u8);
            }
            let mut out = Vec::new();
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality.clamp(1, 100) as u8);
            encoder
                .encode(&rgb8, target_w as u32, target_h as u32, image::ColorType::Rgb8.into())
                .map_err(|e| JsValue::from_str(&format!("Failed to encode JPEG: {}", e)))?;
            out
        };

        // 6. Write output into scratchpad
        let bytes_needed = encoded_bytes.len();
        let aligned_offset = (input_len + 15) & !15;
        let scratch_len = buf.len();
        if aligned_offset + bytes_needed > scratch_len {
            buf.resize(aligned_offset + bytes_needed);
        }
        if let Some(target_slice) = buf.slice_mut(aligned_offset, bytes_needed) {
            target_slice.copy_from_slice(&encoded_bytes);
        }

        let res = ExportResultRecord {
            output_offset: aligned_offset as u64,
            output_len: bytes_needed as u64,
            width: target_w as u32,
            height: target_h as u32,
            format: if is_tiff { "TIFF".to_string() } else { "JPEG".to_string() },
        };

        serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
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
                if encoder.encode(&rgb_bytes, bin_w, bin_h, image::ColorType::Rgb8.into()).is_ok() {
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

        // 4. Fallback to image::load_from_memory for standard raster images (PNG, TIFF, standalone JPEG, WebP)
        if width == 0 || height == 0 {
            if let Some(slice) = buf.slice(0, input_len) {
                if let Ok(dyn_img) = image::load_from_memory(slice) {
                    let rgb = dyn_img.to_rgb8();
                    let orig_w = rgb.width();
                    let orig_h = rgb.height();
                    if orig_w > 0 && orig_h > 0 {
                        let max_dim = 1920u32;
                        let (prev_w, prev_h, prev_rgb) = if orig_w > max_dim || orig_h > max_dim {
                            let (target_w, target_h) = if orig_w >= orig_h {
                                (max_dim, ((orig_h as f32 * max_dim as f32) / orig_w as f32).round().max(1.0) as u32)
                            } else {
                                (((orig_w as f32 * max_dim as f32) / orig_h as f32).round().max(1.0) as u32, max_dim)
                            };
                            let resized = image::imageops::resize(&rgb, target_w, target_h, image::imageops::FilterType::Triangle);
                            (target_w, target_h, resized.into_raw())
                        } else {
                            (orig_w, orig_h, rgb.into_raw())
                        };

                        let mut jpeg_buf = Vec::new();
                        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_buf, 90);
                        if encoder.encode(&prev_rgb, prev_w, prev_h, image::ColorType::Rgb8.into()).is_ok() {
                            let raw_len = jpeg_buf.len();
                            let scratch_len = buf.len();
                            let aligned_offset = (input_len + 15) & !15;
                            if aligned_offset + raw_len > scratch_len {
                                buf.resize(aligned_offset + raw_len);
                            }
                            if let Some(target_slice) = buf.slice_mut(aligned_offset, raw_len) {
                                target_slice.copy_from_slice(&jpeg_buf);
                                preview_offset = aligned_offset as u64;
                                preview_len = raw_len as u64;
                                width = orig_w;
                                height = orig_h;
                            }
                        }
                    }
                }
            }
        }

        let mut metadata = metadata;
        if metadata.raw_width.is_none() && width > 0 {
            metadata.raw_width = Some(width);
        }
        if metadata.raw_height.is_none() && height > 0 {
            metadata.raw_height = Some(height);
        }
        if metadata.orientation.is_none() {
            metadata.orientation = Some(1);
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

static LENSFUN_DB: std::sync::RwLock<Option<LensfunDatabase>> = std::sync::RwLock::new(None);

#[wasm_bindgen]
pub fn load_lensfun_database(json_str: &str) -> bool {
    match LensfunDatabase::from_json_str(json_str) {
        Ok(db) => {
            if let Ok(mut guard) = LENSFUN_DB.write() {
                *guard = Some(db);
                true
            } else {
                false
            }
        }
        Err(e) => {
            web_sys::console::warn_1(&format!("[firstproof-wasm] Failed to parse lensfun JSON: {}", e).into());
            false
        }
    }
}

#[wasm_bindgen]
pub fn resolve_lens_coefficients(
    camera_make: &str,
    camera_model: &str,
    lens_model: &str,
    focal_length: f32,
    aperture: Option<f32>,
) -> JsValue {
    if let Ok(guard) = LENSFUN_DB.read() {
        if let Some(db) = guard.as_ref() {
            let coeffs = db.resolve_coefficients(camera_make, camera_model, lens_model, focal_length, aperture);
            return serde_wasm_bindgen::to_value(&coeffs).unwrap_or(JsValue::NULL);
        }
    }
    let default_coeffs = LensCorrectionCoefficients::default();
    serde_wasm_bindgen::to_value(&default_coeffs).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn convert_kelvin_to_mired_norm(kelvin: f64) -> f64 {
    kelvin_to_mired_norm(kelvin)
}

#[wasm_bindgen]
pub fn convert_mired_norm_to_kelvin(norm: f64) -> f64 {
    mired_norm_to_kelvin(norm)
}

#[wasm_bindgen]
pub fn calculate_target_crop_ratio(
    preset_name: &str,
    image_aspect: f32,
    is_portrait: bool,
) -> Option<f32> {
    let preset = match preset_name {
        "original" | "Original" => CropAspectRatioPreset::Original,
        "1:1" | "square" | "Square" => CropAspectRatioPreset::Square,
        "4:3" | "fourThree" | "FourThree" => CropAspectRatioPreset::FourThree,
        "3:2" | "threeTwo" | "ThreeTwo" => CropAspectRatioPreset::ThreeTwo,
        "16:9" | "sixteenNine" | "SixteenNine" => CropAspectRatioPreset::SixteenNine,
        _ => CropAspectRatioPreset::Freeform,
    };
    preset.target_ratio(image_aspect, is_portrait)
}

#[wasm_bindgen]
pub fn get_white_balance_preset_values(preset_name: &str) -> Option<js_sys::Float64Array> {
    let preset = match preset_name {
        "Daylight" | "daylight" => WhiteBalancePreset::Daylight,
        "Cloudy" | "cloudy" => WhiteBalancePreset::Cloudy,
        "Shade" | "shade" => WhiteBalancePreset::Shade,
        "Tungsten" | "tungsten" => WhiteBalancePreset::Tungsten,
        "Fluorescent" | "fluorescent" => WhiteBalancePreset::Fluorescent,
        "SodiumVapor" | "sodiumVapor" | "sodium_vapor" => WhiteBalancePreset::SodiumVapor,
        "Flash" | "flash" => WhiteBalancePreset::Flash,
        _ => return None,
    };
    preset.kelvin_and_tint().map(|(t, tint)| {
        let arr = js_sys::Float64Array::new_with_length(2);
        arr.set_index(0, t);
        arr.set_index(1, tint);
        arr
    })
}

#[wasm_bindgen]
pub fn detect_white_balance_preset(
    temperature: f64,
    tint: f64,
    as_shot_temp: f64,
    as_shot_tint: f64,
) -> String {
    let preset = WhiteBalancePreset::detect(temperature, tint, as_shot_temp, as_shot_tint);
    match preset {
        WhiteBalancePreset::AsShot => "AsShot".to_string(),
        WhiteBalancePreset::Auto => "Auto".to_string(),
        WhiteBalancePreset::Daylight => "Daylight".to_string(),
        WhiteBalancePreset::Cloudy => "Cloudy".to_string(),
        WhiteBalancePreset::Shade => "Shade".to_string(),
        WhiteBalancePreset::Tungsten => "Tungsten".to_string(),
        WhiteBalancePreset::Fluorescent => "Fluorescent".to_string(),
        WhiteBalancePreset::SodiumVapor => "SodiumVapor".to_string(),
        WhiteBalancePreset::Flash => "Flash".to_string(),
        WhiteBalancePreset::Custom => "Custom".to_string(),
    }
}

#[wasm_bindgen(typescript_custom_section)]
const TS_DEFINITIONS: &'static str = r#"
export type CropAspectRatioPreset =
    | 'Original'
    | '1:1'
    | '4:3'
    | '3:2'
    | '16:9'
    | 'Free';

export type WhiteBalancePreset =
    | 'AsShot'
    | 'Auto'
    | 'Daylight'
    | 'Cloudy'
    | 'Shade'
    | 'Tungsten'
    | 'Fluorescent'
    | 'SodiumVapor'
    | 'Flash'
    | 'Custom';

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
    baseline_sharpness?: number;
    latitude?: number | null;
    longitude?: number | null;
    description?: string | null;
    creator?: string | null;
    copyright?: string | null;
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
    | 'Format120'
    | 'MediumFormat'
    | 'Format4x5'
    | 'LargeFormat4x5'
    | 'Super8'
    | 'Format16mm';

export type EmulsionGrainMode =
    | 'Off'
    | 'SeedFromSensor'
    | 'SynthesizePhysical';

export type RemjetBackingMode =
    | 'Intact'
    | 'Stripped';

export type PrintMediaMode =
    | 'None'
    | 'ChromogenicTypeC'
    | 'SilverDyeBleach'
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
    lut_name?: string;
    lut_intensity: number;
    sharpening_amount: number;
    sharpening_radius: number;
    sharpening_masking: number;
    noise_reduction: number;
    emulsion_archetype: EmulsionArchetype;
    emulsion_format: EmulsionFormat;
    emulsion_exposure: number;
    remjet_backing: RemjetBackingMode;
    emulsion_grain_mode: EmulsionGrainMode;
    grain_scale: number;
    grain_size?: number;
    grain_amount?: number;
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
    defringe_amount?: number;
    lens_correction_enabled?: boolean;
    tca_correction_enabled?: boolean;
    vignetting_correction_enabled?: boolean;
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

/// Executes a canonical [`DarkroomCommand`] in the Web darkroom via WASM.
#[wasm_bindgen]
pub fn execute_darkroom_command(json_str: &str) -> String {
    let cmd = match firstproof_engine::DarkroomCommand::from_json(json_str) {
        Ok(c) => c,
        Err(e) => return firstproof_engine::CommandResponse::error(e).to_json(),
    };

    match cmd {
        firstproof_engine::DarkroomCommand::ApplyEditDelta { delta } => {
            let mut settings = firstproof_engine::EditSettings::default();
            delta.apply_to(&mut settings);
            match serde_json::to_value(&settings) {
                Ok(val) => firstproof_engine::CommandResponse::ok(serde_json::json!({
                    "settings": val
                })).to_json(),
                Err(e) => firstproof_engine::CommandResponse::error(e.to_string()).to_json(),
            }
        }
        firstproof_engine::DarkroomCommand::ResetWorkspace { category } => {
            firstproof_engine::CommandResponse::ok(serde_json::json!({
                "action": "reset_workspace",
                "category": category,
            })).to_json()
        }
        firstproof_engine::DarkroomCommand::SetExposure { ev } => {
            firstproof_engine::CommandResponse::ok(serde_json::json!({
                "action": "set_exposure",
                "ev": ev,
            })).to_json()
        }
        firstproof_engine::DarkroomCommand::SetTemperature { kelvin } => {
            firstproof_engine::CommandResponse::ok(serde_json::json!({
                "action": "set_temperature",
                "kelvin": kelvin,
            })).to_json()
        }
        firstproof_engine::DarkroomCommand::SetTint { tint } => {
            firstproof_engine::CommandResponse::ok(serde_json::json!({
                "action": "set_tint",
                "tint": tint,
            })).to_json()
        }
        firstproof_engine::DarkroomCommand::SetEmulsion { archetype, format, exposure } => {
            firstproof_engine::CommandResponse::ok(serde_json::json!({
                "action": "set_emulsion",
                "archetype": archetype,
                "format": format,
                "exposure": exposure,
            })).to_json()
        }
        firstproof_engine::DarkroomCommand::SetPrintMedia { media, paper_grade, base_warmth, process_depth } => {
            firstproof_engine::CommandResponse::ok(serde_json::json!({
                "action": "set_print_media",
                "media": media,
                "paper_grade": paper_grade,
                "base_warmth": base_warmth,
                "process_depth": process_depth,
            })).to_json()
        }
        other => {
            firstproof_engine::CommandResponse::ok(serde_json::json!({
                "command": format!("{:?}", other),
                "status": "acknowledged"
            })).to_json()
        }
    }
}

#[wasm_bindgen]
pub fn compute_ev_histogram_from_bytes(
    data: &[u8],
    width: u32,
    height: u32,
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
) -> Result<JsValue, JsValue> {
    let record = firstproof_engine::histogram::compute_ev_histogram_from_f16_buffer(
        width,
        height,
        data,
        exposure,
        contrast,
        highlights,
        shadows,
        whites,
        blacks,
        crop_left,
        crop_top,
        crop_right,
        crop_bottom,
        lens_correction_enabled,
        tca_correction_enabled,
        vignetting_correction_enabled,
        vignetting_k1,
        vignetting_k2,
        vignetting_k3,
    )
    .ok_or_else(|| JsValue::from_str("Failed to compute EV histogram from buffer"))?;

    serde_wasm_bindgen::to_value(&record).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_compute_ev_histogram() {
        // 10x10 f16 buffer (100 pixels * 8 bytes = 800 bytes)
        let mut buf = vec![0u8; 800];
        let mid_u16 = f32_to_f16(0.18);
        for i in 0..100 {
            let offset = i * 8;
            buf[offset..offset + 2].copy_from_slice(&mid_u16.to_le_bytes());
            buf[offset + 2..offset + 4].copy_from_slice(&mid_u16.to_le_bytes());
            buf[offset + 4..offset + 6].copy_from_slice(&mid_u16.to_le_bytes());
            buf[offset + 6..offset + 8].copy_from_slice(&f32_to_f16(1.0).to_le_bytes());
        }
        let record = firstproof_engine::histogram::compute_ev_histogram_from_f16_buffer(
            10, 10, &buf, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, false, false, false, 0.0, 0.0, 0.0,
        ).unwrap();
        assert_eq!(record.num_bins, 120);
        assert!(record.bins_luminance.iter().sum::<f32>() > 0.0);
    }

    #[test]
    fn test_wasm_execute_darkroom_command() {
        let delta_json = r#"{"command":"apply_edit_delta","delta":{"exposure":1.25,"contrast":10.0}}"#;
        let resp = execute_darkroom_command(delta_json);
        assert!(resp.contains("\"status\":\"ok\""));
        assert!(resp.contains("\"exposure\":1.25"));

        let exp_json = r#"{"command":"set_exposure","ev":-0.5}"#;
        let resp_exp = execute_darkroom_command(exp_json);
        assert!(resp_exp.contains("\"status\":\"ok\""));
        assert!(resp_exp.contains("\"ev\":-0.5"));
    }

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

        // sRGB through default AgX Vivid (matching 128u8 / 255.0 frame test below):
        let rgb_neutral = evaluate_reference_pixel(128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, false, &edits_json, None).unwrap();
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

    #[test]
    fn test_raster_png_preview() {
        let mut img = image::RgbImage::new(2, 2);
        img.put_pixel(0, 0, image::Rgb([255, 0, 0]));
        img.put_pixel(1, 0, image::Rgb([0, 255, 0]));
        img.put_pixel(0, 1, image::Rgb([0, 0, 255]));
        img.put_pixel(1, 1, image::Rgb([255, 255, 255]));
        let mut png_bytes = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png).unwrap();

        let dyn_img = image::load_from_memory(&png_bytes).expect("load_from_memory png failed");
        assert_eq!(dyn_img.width(), 2);
        assert_eq!(dyn_img.height(), 2);

        let rgb = dyn_img.to_rgb8();
        let mut jpeg_buf = Vec::new();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_buf, 90);
        let enc_res = encoder.encode(rgb.as_raw(), 2, 2, image::ColorType::Rgb8.into());
        assert!(enc_res.is_ok(), "encoder.encode failed: {:?}", enc_res.err());
    }

    #[test]
    fn test_lensfun_resolution() {
        let minimal_json = r#"{
            "cameras": [
                { "maker": "Canon", "model": "EOS 5D Mark IV", "mount": "Canon EF", "crop_factor": 1.0 }
            ],
            "lenses": [
                {
                    "maker": "Canon",
                    "model": "EF 24-70mm f/2.8L II USM",
                    "mounts": ["Canon EF"],
                    "crop_factor": 1.0,
                    "min_focal": 24.0,
                    "max_focal": 70.0,
                    "min_aperture": 2.8,
                    "max_aperture": 22.0,
                    "distortion_calib": [
                        { "model": "ptlens", "focal_length": 24.0, "coefficients": [0.02964, 0.0, 0.0, 0.0] }
                    ],
                    "tca_calib": [
                        { "focal_length": 24.0, "coefficients": [1.0002104, 1.0000529, -0.0000220, 0.0, 0.0, 0.0] }
                    ],
                    "vignetting_calib": [
                        { "focal_length": 24.0, "aperture": 2.8, "distance": 1000.0, "coefficients": [-0.8, 0.2, 0.0] }
                    ]
                }
            ]
        }"#;

        assert!(load_lensfun_database(minimal_json));
        if let Ok(guard) = LENSFUN_DB.read() {
            let db = guard.as_ref().expect("LENSFUN_DB should be populated");
            let coeffs = db.resolve_coefficients("Canon", "5D Mark IV", "EF 24-70mm f/2.8L II USM", 24.0, Some(2.8));
            assert!(coeffs.has_distortion);
            assert_eq!(coeffs.distortion_model, "ptlens");
            assert_eq!(coeffs.distortion[0], 0.02964);
            assert!(coeffs.has_tca);
            assert!(coeffs.has_vignetting);
            assert_eq!(coeffs.vignetting[0], -0.8);
        }
    }
}
