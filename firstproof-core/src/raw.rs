use std::fs::File;
use std::io::Read;
use std::path::Path;

pub use firstproof_engine::{
    decode_exif_orientation as decode_orientation_record,
    extract_metadata_from_bytes,
    is_displayable_rgb_jpeg,
    ImageMetadataRecord,
};

/// Type alias for backward compatibility across firstproof-core
pub type ImageMetadata = ImageMetadataRecord;

/// Decodes an EXIF orientation tag (1-8) into canonical rotation degrees (0, 90, 180, 270)
/// and horizontal/vertical flip booleans.
pub fn decode_exif_orientation(orientation: u32) -> (i32, bool, bool) {
    let rec = firstproof_engine::decode_exif_orientation(orientation);
    (rec.rotation, rec.flip_horizontal, rec.flip_vertical)
}

/// Extracts metadata from the RAW/DNG file path.
pub fn extract_metadata<P: AsRef<Path>>(path: P) -> std::io::Result<ImageMetadataRecord> {
    let bytes = std::fs::read(&path)?;
    Ok(extract_metadata_from_bytes(&bytes))
}

/// Scans the file bytes to extract the largest embedded RGB JPEG preview.
pub fn extract_preview_jpeg<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    
    let len = bytes.len();
    if len < 4 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "File too small"));
    }
    
    // Fast path 1: TIFF IFD / SubIFD tag resolution via firstproof_engine (0.3ms)
    if let Some((start, len_slice)) = firstproof_engine::preview::extract_preview_slice(&bytes) {
        if start + len_slice <= bytes.len() {
            return Ok(bytes[start..start + len_slice].to_vec());
        }
    }

    // Fast path 2: Direct JPEG image
    if bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF && is_displayable_rgb_jpeg(&bytes) {
        return Ok(bytes);
    }
    
    let mut jpegs = Vec::new();
    let mut i = 0;
    
    // Scan for JPEG Start of Image (SOI) markers: 0xFFD8
    while i < len - 1 {
        if bytes[i] == 0xFF && bytes[i + 1] == 0xD8 {
            let start = i;
            // Scan forward for End of Image (EOI): 0xFFD9
            let mut j = i + 2;
            let mut found_eoi = false;
            while j < len - 1 {
                if bytes[j] == 0xFF && bytes[j + 1] == 0xD9 {
                    found_eoi = true;
                    let end = j + 2;
                    let size = end - start;
                    if size > 10 * 1024 { // Minimum 10KB to qualify as an embedded thumbnail/preview
                        jpegs.push((start, end, size));
                    }
                    i = j; // skip forward
                    break;
                }
                j += 1;
            }
            if !found_eoi {
                let size = len - start;
                if size > 10 * 1024 {
                    jpegs.push((start, len, size));
                }
            }
        }
        i += 1;
    }
    
    // Find the largest valid displayable RGB JPEG preview
    let best = jpegs
        .into_iter()
        .filter(|&(start, end, _)| is_displayable_rgb_jpeg(&bytes[start..end]))
        .max_by_key(|&(_, _, size)| size);
        
    match best {
        Some((start, end, _)) => Ok(bytes[start..end].to_vec()),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No displayable RGB preview found in file",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_preview_jpeg() {
        let mut data = vec![0xAA; 1000]; // leading junk
        
        let jpeg_size = 60 * 1024; // 60KB
        let mut jpeg_bytes = vec![0x00; jpeg_size];
        jpeg_bytes[0] = 0xFF;
        jpeg_bytes[1] = 0xD8;
        // Add SOF0 marker (Baseline RGB)
        jpeg_bytes[2] = 0xFF;
        jpeg_bytes[3] = 0xC0;
        jpeg_bytes[4] = 0x00;
        jpeg_bytes[5] = 0x11; // 17 bytes payload
        
        jpeg_bytes[jpeg_size - 2] = 0xFF;
        jpeg_bytes[jpeg_size - 1] = 0xD9;
        
        data.extend_from_slice(&jpeg_bytes);
        data.extend_from_slice(&vec![0xBB; 2000]); // trailing junk
        
        let path = "test_mock_image.cr2";
        std::fs::write(path, &data).unwrap();
        
        let extracted = extract_preview_jpeg(path).unwrap();
        
        let _ = std::fs::remove_file(path);
        
        assert_eq!(extracted.len(), jpeg_size);
        assert_eq!(extracted[0], 0xFF);
        assert_eq!(extracted[1], 0xD8);
        assert_eq!(extracted[jpeg_size - 2], 0xFF);
        assert_eq!(extracted[jpeg_size - 1], 0xD9);
    }
}
