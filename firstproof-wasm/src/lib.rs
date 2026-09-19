use wasm_bindgen::prelude::*;

pub mod curve;
pub mod preview;
pub mod scratchpad;
pub mod xmp;

use curve::{evaluate_curves_lut_ref, ToneCurvesRecord};
use preview::{
    extract_metadata_from_bytes, extract_preview_slice, get_jpeg_dimensions, PreviewResultRecord,
};
use scratchpad::with_scratchpad;
use xmp::{generate_xmp_sidecar, parse_xmp_sidecar, WebXmpData};

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
pub fn copy_into_scratchpad(offset: usize, bytes: &[u8]) -> usize {
    with_scratchpad(|buf| {
        if offset + bytes.len() > buf.len() {
            buf.resize(offset + bytes.len());
        }
        if let Some(slice) = buf.slice_mut(offset, bytes.len()) {
            slice.copy_from_slice(bytes);
            bytes.len()
        } else {
            0
        }
    })
}

#[wasm_bindgen]
pub fn decode_preview_from_scratchpad(input_len: usize) -> Result<JsValue, JsValue> {
    with_scratchpad(|buf| {
        let slice = buf
            .slice(0, input_len)
            .ok_or_else(|| JsValue::from_str("Input length exceeds scratchpad buffer capacity"))?;

        let metadata = extract_metadata_from_bytes(slice);
        let preview_info = extract_preview_slice(slice).ok_or_else(|| {
            JsValue::from_str("Failed to find embedded RGB preview in RAW/JPEG stream")
        })?;

        let (preview_offset, preview_len) = preview_info;
        let preview_bytes = &slice[preview_offset..preview_offset + preview_len];
        let (width, height) = get_jpeg_dimensions(preview_bytes).unwrap_or((0, 0));

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
    let lut = evaluate_curves_lut_ref(&curves);
    serde_wasm_bindgen::to_value(&lut).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn parse_xmp(xml: &str) -> Result<JsValue, JsValue> {
    let xmp = parse_xmp_sidecar(xml).map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&xmp).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn generate_xmp(xmp_json: &str) -> Result<String, JsValue> {
    let xmp: WebXmpData = serde_json::from_str(xmp_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid XMP JSON: {}", e)))?;
    Ok(generate_xmp_sidecar(&xmp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use curve::CurvePointRecord;

    #[test]
    fn test_scratchpad_allocation() {
        with_scratchpad(|buf| {
            assert!(buf.len() >= 64 * 1024 * 1024);
            let ptr = buf.ptr();
            assert!(!ptr.is_null());
            buf.resize(70 * 1024 * 1024);
            assert_eq!(buf.len(), 70 * 1024 * 1024);
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
        let lut = evaluate_curves_lut_ref(&curves);
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
        let mut xmp = WebXmpData::default();
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
}
