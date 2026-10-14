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

fn parse_rawler_gps(gps: &Option<rawler::exif::ExifGPS>) -> (Option<f64>, Option<f64>) {
    let gps = match gps {
        Some(g) => g,
        None => return (None, None),
    };
    let lat = gps.gps_latitude.as_ref().and_then(|r| {
        if r[0].d > 0 && r[1].d > 0 && r[2].d > 0 {
            let deg = r[0].n as f64 / r[0].d as f64;
            let min = r[1].n as f64 / r[1].d as f64;
            let sec = r[2].n as f64 / r[2].d as f64;
            let mut val = deg + (min / 60.0) + (sec / 3600.0);
            if let Some(ref r_str) = gps.gps_latitude_ref {
                if r_str.starts_with('S') || r_str.starts_with('s') {
                    val = -val;
                }
            }
            Some(val)
        } else {
            None
        }
    });
    let lon = gps.gps_longitude.as_ref().and_then(|r| {
        if r[0].d > 0 && r[1].d > 0 && r[2].d > 0 {
            let deg = r[0].n as f64 / r[0].d as f64;
            let min = r[1].n as f64 / r[1].d as f64;
            let sec = r[2].n as f64 / r[2].d as f64;
            let mut val = deg + (min / 60.0) + (sec / 3600.0);
            if let Some(ref r_str) = gps.gps_longitude_ref {
                if r_str.starts_with('W') || r_str.starts_with('w') {
                    val = -val;
                }
            }
            Some(val)
        } else {
            None
        }
    });
    (lat, lon)
}

fn raw_metadata_to_record(rm: &rawler::decoders::RawMetadata) -> ImageMetadataRecord {
    let camera_make = if !rm.make.is_empty() { Some(rm.make.clone()) } else { None };
    let camera_model = if !rm.model.is_empty() { Some(rm.model.clone()) } else { None };
    let lens_model = rm.exif.lens_model.clone()
        .or_else(|| rm.lens.as_ref().and_then(|l| l.identifiers.name.clone()))
        .or_else(|| rm.exif.lens_make.clone());
    let iso = rm.exif.iso_speed.or_else(|| rm.exif.iso_speed_ratings.map(|v| v as u32));
    let aperture = rm.exif.fnumber.as_ref().and_then(|r| {
        if r.d > 0 { Some(r.n as f32 / r.d as f32) } else { None }
    }).or_else(|| {
        rm.exif.aperture_value.as_ref().and_then(|r| {
            if r.d > 0 { Some(2.0f32.powf((r.n as f32 / r.d as f32) * 0.5)) } else { None }
        })
    });
    let shutter_speed = rm.exif.exposure_time.as_ref().and_then(|r| {
        if r.d > 0 {
            if r.n >= r.d {
                Some(format!("{}", r.n / r.d))
            } else {
                Some(format!("{}/{}", r.n, r.d))
            }
        } else {
            None
        }
    });
    let focal_length = rm.exif.focal_length.as_ref().and_then(|r| {
        if r.d > 0 { Some(r.n as f32 / r.d as f32) } else { None }
    });
    let taken_time = rm.exif.date_time_original.as_deref().and_then(firstproof_engine::preview::parse_exif_date_str);
    let (latitude, longitude) = parse_rawler_gps(&rm.exif.gps);
    let creator = rm.exif.artist.clone();
    let copyright = rm.exif.copyright.clone();
    let orientation = rm.exif.orientation.map(|o| o as u32);

    ImageMetadataRecord {
        camera_make,
        camera_model,
        lens_model,
        iso,
        aperture,
        shutter_speed,
        focal_length,
        taken_time,
        latitude,
        longitude,
        description: rm.exif.user_comment.clone(),
        creator,
        copyright,
        orientation,
        raw_width: None,
        raw_height: None,
        baseline_sharpness: None,
    }
}

/// Extracts metadata from the RAW/DNG file path.
pub fn extract_metadata<P: AsRef<Path>>(path: P) -> std::io::Result<ImageMetadataRecord> {
    let path_ref = path.as_ref();
    let mut rec = if let Ok(raw_source) = rawler::rawsource::RawSource::new(path_ref) {
        if let Ok(dec) = rawler::get_decoder(&raw_source) {
            if let Ok(rm) = dec.raw_metadata(&raw_source, &rawler::decoders::RawDecodeParams::default()) {
                Some(raw_metadata_to_record(&rm))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    if let Ok(bytes) = std::fs::read(path_ref) {
        let byte_rec = extract_metadata_from_bytes(&bytes);
        if let Some(ref mut r) = rec {
            if r.camera_make.is_none() { r.camera_make = byte_rec.camera_make; }
            if r.camera_model.is_none() { r.camera_model = byte_rec.camera_model; }
            if r.lens_model.is_none() { r.lens_model = byte_rec.lens_model; }
            if r.iso.is_none() { r.iso = byte_rec.iso; }
            if r.aperture.is_none() { r.aperture = byte_rec.aperture; }
            if r.shutter_speed.is_none() { r.shutter_speed = byte_rec.shutter_speed; }
            if r.focal_length.is_none() { r.focal_length = byte_rec.focal_length; }
            if r.taken_time.is_none() { r.taken_time = byte_rec.taken_time; }
            if r.latitude.is_none() { r.latitude = byte_rec.latitude; }
            if r.longitude.is_none() { r.longitude = byte_rec.longitude; }
            if r.description.is_none() { r.description = byte_rec.description; }
            if r.creator.is_none() { r.creator = byte_rec.creator; }
            if r.copyright.is_none() { r.copyright = byte_rec.copyright; }
            if r.orientation.is_none() { r.orientation = byte_rec.orientation; }
        } else {
            rec = Some(byte_rec);
        }
    }

    rec.ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Failed to extract metadata"))
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
