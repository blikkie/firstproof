use image::{GrayImage, Luma};
use image::codecs::jpeg::JpegEncoder;
use img_parts::jpeg::Jpeg;
use img_parts::Bytes;

/// Metadata parameters for Ultra HDR / ISO 21496-1 and Google/Adobe Gain Maps.
#[derive(Debug, Clone, PartialEq)]
pub struct GainMapMetadata {
    pub min_gain: f32,
    pub max_gain: f32,
    pub gamma: f32,
    pub offset_sdr: f32,
    pub offset_hdr: f32,
    pub hdr_capacity_min: f32,
    pub hdr_capacity_max: f32,
}

impl Default for GainMapMetadata {
    fn default() -> Self {
        Self {
            min_gain: 0.0,
            max_gain: 3.0, // Up to 3.0 EV = 8x highlight boost (1,600 nits)
            gamma: 1.0,
            offset_sdr: 1.0 / 64.0, // 0.015625
            offset_hdr: 1.0 / 64.0, // 0.015625
            hdr_capacity_min: 0.0,
            hdr_capacity_max: 3.0,
        }
    }
}

/// Computes a downsampled 8-bit grayscale gain map from SDR and HDR luminance arrays.
pub fn generate_gain_map(
    sdr_luma: &[f32],
    hdr_luma: &[f32],
    width: usize,
    height: usize,
    meta: &GainMapMetadata,
) -> (GrayImage, Vec<u8>) {
    // Standard downsampling factor: 1/2 resolution in each axis for compact footprint
    let gm_w = (width / 2).max(1);
    let gm_h = (height / 2).max(1);
    let mut gm_img = GrayImage::new(gm_w as u32, gm_h as u32);
    
    let range = (meta.max_gain - meta.min_gain).max(1e-4);
    
    for gy in 0..gm_h {
        let sy = (gy * 2).min(height - 1);
        for gx in 0..gm_w {
            let sx = (gx * 2).min(width - 1);
            let idx = sy * width + sx;
            
            let y_sdr = sdr_luma.get(idx).copied().unwrap_or(0.0).clamp(0.0, 1.0);
            let y_hdr = hdr_luma.get(idx).copied().unwrap_or(0.0).max(0.0);
            
            let numerator = y_hdr + meta.offset_hdr;
            let denominator = y_sdr + meta.offset_sdr;
            let ratio = (numerator / denominator.max(1e-6)).max(1e-4);
            let log2_gain = ratio.log2();
            
            let normalized = ((log2_gain - meta.min_gain) / range).clamp(0.0, 1.0);
            let byte_val = (normalized * 255.0).round() as u8;
            
            gm_img.put_pixel(gx as u32, gy as u32, Luma([byte_val]));
        }
    }
    
    // Encode downsampled gain map to 8-bit JPEG
    let mut jpeg_bytes = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, 85);
    let _ = encoder.encode(
        gm_img.as_raw(),
        gm_w as u32,
        gm_h as u32,
        image::ColorType::L8,
    );
    
    (gm_img, jpeg_bytes)
}

/// Generates the GContainer and Adobe hdrgm XMP metadata XML string.
pub fn generate_xmp_gain_map_metadata(gain_map_length: usize, meta: &GainMapMetadata) -> String {
    format!(
        concat!(
            "<x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"Adobe XMP Core 5.1.2\">\n",
            " <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n",
            "  <rdf:Description rdf:about=\"\"\n",
            "    xmlns:Container=\"http://ns.google.com/photos/1.0/container/\"\n",
            "    xmlns:Item=\"http://ns.google.com/photos/1.0/container/item/\"\n",
            "    xmlns:hdrgm=\"http://ns.adobe.com/hdr-gain-map/1.0/\"\n",
            "    hdrgm:Version=\"1.0\"\n",
            "    hdrgm:GainMapMin=\"{:.4}\"\n",
            "    hdrgm:GainMapMax=\"{:.4}\"\n",
            "    hdrgm:Gamma=\"{:.4}\"\n",
            "    hdrgm:OffsetSDR=\"{:.6}\"\n",
            "    hdrgm:OffsetHDR=\"{:.6}\"\n",
            "    hdrgm:HDRCapacityMin=\"{:.4}\"\n",
            "    hdrgm:HDRCapacityMax=\"{:.4}\"\n",
            "    hdrgm:BaseRenditionIsHDR=\"False\">\n",
            "   <Container:Directory>\n",
            "    <rdf:Seq>\n",
            "     <rdf:li rdf:parseType=\"Resource\">\n",
            "      <Item:Mime>image/jpeg</Item:Mime>\n",
            "      <Item:Semantic>Primary</Item:Semantic>\n",
            "      <Item:Length>0</Item:Length>\n",
            "      <Item:Padding>0</Item:Padding>\n",
            "     </rdf:li>\n",
            "     <rdf:li rdf:parseType=\"Resource\">\n",
            "      <Item:Mime>image/jpeg</Item:Mime>\n",
            "      <Item:Semantic>GainMap</Item:Semantic>\n",
            "      <Item:Length>{}</Item:Length>\n",
            "      <Item:Padding>0</Item:Padding>\n",
            "     </rdf:li>\n",
            "    </rdf:Seq>\n",
            "   </Container:Directory>\n",
            "  </rdf:Description>\n",
            " </rdf:RDF>\n",
            "</x:xmpmeta>"
        ),
        meta.min_gain,
        meta.max_gain,
        meta.gamma,
        meta.offset_sdr,
        meta.offset_hdr,
        meta.hdr_capacity_min,
        meta.hdr_capacity_max,
        gain_map_length
    )
}

/// Generates the standardized ISO 21496-1 APP2 marker segment payload.
pub fn generate_iso_21496_1_app2_payload(meta: &GainMapMetadata) -> Vec<u8> {
    let mut payload = Vec::with_capacity(128);
    // Header string terminated with null byte
    payload.extend_from_slice(b"urn:iso:std:iso:ts:21496:-1\0");
    // Standard binary serialization (version 0):
    // min_gain, max_gain, gamma, offset_sdr, offset_hdr as big-endian 32-bit floats
    payload.extend_from_slice(&meta.min_gain.to_be_bytes());
    payload.extend_from_slice(&meta.max_gain.to_be_bytes());
    payload.extend_from_slice(&meta.gamma.to_be_bytes());
    payload.extend_from_slice(&meta.offset_sdr.to_be_bytes());
    payload.extend_from_slice(&meta.offset_hdr.to_be_bytes());
    payload.extend_from_slice(&meta.hdr_capacity_min.to_be_bytes());
    payload.extend_from_slice(&meta.hdr_capacity_max.to_be_bytes());
    payload
}

/// Packages a primary SDR JPEG and a secondary gain map JPEG into an Ultra HDR / ISO 21496-1 single file.
pub fn package_ultra_hdr_jpeg(
    primary_sdr_jpeg: &[u8],
    gain_map_jpeg: &[u8],
    meta: &GainMapMetadata,
) -> Result<Vec<u8>, String> {
    let xmp_xml = generate_xmp_gain_map_metadata(gain_map_jpeg.len(), meta);
    let iso_payload = generate_iso_21496_1_app2_payload(meta);
    
    // Parse primary JPEG container using img-parts
    let mut jpeg = Jpeg::from_bytes(Bytes::copy_from_slice(primary_sdr_jpeg))
        .map_err(|e| format!("Failed to parse primary JPEG: {:?}", e))?;
    
    // 1. Inject XMP container marker (APP1 with http://ns.adobe.com/xap/1.0/\0 prefix)
    let mut xmp_segment_payload = Vec::with_capacity(32 + xmp_xml.len());
    xmp_segment_payload.extend_from_slice(b"http://ns.adobe.com/xap/1.0/\0");
    xmp_segment_payload.extend_from_slice(xmp_xml.as_bytes());
    let xmp_segment = img_parts::jpeg::markers::APP1;
    jpeg.segments_mut().push(img_parts::jpeg::JpegSegment::new_with_contents(
        xmp_segment,
        Bytes::from(xmp_segment_payload),
    ));
    
    // 2. Inject ISO 21496-1 marker (APP2)
    let app2_marker = img_parts::jpeg::markers::APP2;
    jpeg.segments_mut().push(img_parts::jpeg::JpegSegment::new_with_contents(
        app2_marker,
        Bytes::from(iso_payload),
    ));
    
    // Write out primary JPEG
    let mut output = Vec::with_capacity(primary_sdr_jpeg.len() + gain_map_jpeg.len() + 1024);
    jpeg.encoder()
        .write_to(&mut output)
        .map_err(|e| format!("Failed to encode Ultra HDR primary JPEG: {:?}", e))?;
        
    // 3. Concatenate secondary gain map JPEG payload
    output.extend_from_slice(gain_map_jpeg);
    
    Ok(output)
}

/// Parses an existing JPEG to detect if it contains an Ultra HDR / ISO 21496-1 gain map.
pub fn has_gain_map(jpeg_bytes: &[u8]) -> bool {
    let haystack = &jpeg_bytes[..jpeg_bytes.len().min(65536)];
    
    // Check for ISO 21496-1 URN or Adobe/Google gain map namespaces
    haystack.windows(b"urn:iso:std:iso:ts:21496:-1".len()).any(|w| w == b"urn:iso:std:iso:ts:21496:-1")
        || haystack.windows(b"http://ns.adobe.com/hdr-gain-map/1.0/".len()).any(|w| w == b"http://ns.adobe.com/hdr-gain-map/1.0/")
        || haystack.windows(b"Item:Semantic=\"GainMap\"".len()).any(|w| w == b"Item:Semantic=\"GainMap\"")
}

/// Extracts a gain map from an Ultra HDR JPEG if present.
pub fn extract_gain_map(jpeg_bytes: &[u8]) -> Option<(GainMapMetadata, Vec<u8>, u32, u32)> {
    if !has_gain_map(jpeg_bytes) {
        return None;
    }
    
    // Search for Item:Length="<length>" in XMP
    let mut meta = GainMapMetadata::default();
    let text = String::from_utf8_lossy(&jpeg_bytes[..jpeg_bytes.len().min(65536)]);
    
    let mut gain_map_length = None;
    if let Some(pos) = text.find("<Item:Semantic>GainMap</Item:Semantic>") {
        let snippet = &text[pos..text.len().min(pos + 200)];
        if let Some(l_start) = snippet.find("<Item:Length>") {
            if let Some(l_end) = snippet[l_start..].find("</Item:Length>") {
                let num_str = &snippet[l_start + 13..l_start + l_end].trim();
                gain_map_length = num_str.parse::<usize>().ok();
            }
        }
    }
    
    // Parse GainMapMax if present
    if let Some(pos) = text.find("hdrgm:GainMapMax=\"") {
        let rest = &text[pos + 18..];
        if let Some(quote) = rest.find('\"') {
            if let Ok(v) = rest[..quote].parse::<f32>() {
                meta.max_gain = v;
            }
        }
    }
    
    // Extract payload from end of file
    let gm_len = gain_map_length.unwrap_or(0);
    let gm_bytes = if gm_len > 0 && gm_len < jpeg_bytes.len() {
        &jpeg_bytes[jpeg_bytes.len() - gm_len..]
    } else {
        // Fallback: search backwards for secondary JPEG SOI marker (0xFF, 0xD8)
        let mut found_offset = None;
        for i in (jpeg_bytes.len().saturating_sub(500_000)..jpeg_bytes.len().saturating_sub(4)).rev() {
            if jpeg_bytes[i] == 0xFF && jpeg_bytes[i + 1] == 0xD8 && jpeg_bytes[i + 2] == 0xFF {
                found_offset = Some(i);
                break;
            }
        }
        {
            let offset = found_offset?;
            &jpeg_bytes[offset..]
        }
    };
    
    // Decode image dimensions using the image crate
    if let Ok(img) = image::load_from_memory(gm_bytes) {
        let w = img.width();
        let h = img.height();
        Some((meta, gm_bytes.to_vec(), w, h))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gain_map_generation_and_packaging() {
        let width = 64;
        let height = 64;
        let sdr_luma = vec![0.5f32; width * height];
        let mut hdr_luma = vec![0.5f32; width * height];
        
        // Add a specular highlight
        for i in 0..10 {
            hdr_luma[i] = 4.0; // 4x brighter
        }
        
        let meta = GainMapMetadata::default();
        let (gm_img, gm_jpeg) = generate_gain_map(&sdr_luma, &hdr_luma, width, height, &meta);
        assert!(!gm_jpeg.is_empty());
        assert_eq!(gm_img.width(), 32);
        assert_eq!(gm_img.height(), 32);
        
        // Create dummy primary JPEG
        let mut primary_jpeg = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut primary_jpeg, 90);
        let dummy_rgb = vec![128u8; width * height * 3];
        encoder.encode(&dummy_rgb, width as u32, height as u32, image::ColorType::Rgb8).unwrap();
        
        // Package into Ultra HDR JPEG
        let packaged = package_ultra_hdr_jpeg(&primary_jpeg, &gm_jpeg, &meta).unwrap();
        assert!(has_gain_map(&packaged));
        
        // Extract gain map
        let extracted = extract_gain_map(&packaged);
        assert!(extracted.is_some());
        let (extracted_meta, extracted_bytes, ew, eh) = extracted.unwrap();
        assert_eq!(ew, 32);
        assert_eq!(eh, 32);
        assert!(!extracted_bytes.is_empty());
        assert!((extracted_meta.max_gain - 3.0).abs() < 1e-3);
    }
}
