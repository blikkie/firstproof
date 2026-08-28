use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use exif::{Exif, In, Reader, Tag, Value};

#[derive(Clone)]
pub struct ImageMetadata {
    pub file_path: String,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub iso: Option<u32>,
    pub aperture: Option<f32>,
    pub shutter_speed: Option<String>,
    pub focal_length: Option<f32>,
    pub taken_time: Option<i64>, // Unix timestamp
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
}

/// Helper function to parse EXIF date string "YYYY:MM:DD HH:MM:SS" into a Unix timestamp.
fn parse_exif_date(date_str: &str) -> Option<i64> {
    let parts: Vec<&str> = date_str.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let date_parts: Vec<&str> = parts[0].split(':').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();
    
    if date_parts.len() < 3 || time_parts.len() < 3 {
        return None;
    }
    
    let year = date_parts[0].parse::<i32>().ok()?;
    let month = date_parts[1].parse::<i32>().ok()?;
    let day = date_parts[2].parse::<i32>().ok()?;
    
    let hour = time_parts[0].parse::<i32>().ok()?;
    let min = time_parts[1].parse::<i32>().ok()?;
    let sec = time_parts[2].parse::<i32>().ok()?;
    
    // Simple Unix time calculation (ignores leap seconds, but standard)
    // Formula source: standard calendar algorithms
    let mut days = (year - 1970) * 365 + (year - 1969) / 4 - (year - 1901) / 100 + (year - 1601) / 400;
    
    let month_days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    
    for m in 1..month {
        days += month_days[m as usize];
        if m == 2 && is_leap {
            days += 1;
        }
    }
    
    days += day - 1;
    
    let seconds = (days as i64 * 86400) + (hour as i64 * 3600) + (min as i64 * 60) + sec as i64;
    Some(seconds)
}

/// Helper function to parse EXIF GPS values (degrees, minutes, seconds rational array) to decimal degrees.
fn parse_gps(exif: &Exif, tag: Tag, ref_tag: Tag) -> Option<f64> {
    let field = exif.get_field(tag, In::PRIMARY)?;
    let ref_field = exif.get_field(ref_tag, In::PRIMARY)?;
    
    let rationals = match &field.value {
        Value::Rational(v) => v,
        _ => return None,
    };
    
    if rationals.len() < 3 {
        return None;
    }
    
    let deg = rationals[0].num as f64 / rationals[0].denom as f64;
    let min = rationals[1].num as f64 / rationals[1].denom as f64;
    let sec = rationals[2].num as f64 / rationals[2].denom as f64;
    
    let mut val = deg + (min / 60.0) + (sec / 3600.0);
    
    if let Value::Ascii(ref v) = ref_field.value {
        if let Some(s) = v.get(0) {
            if let Ok(string) = std::str::from_utf8(s) {
                let sign = string.trim().to_uppercase();
                if sign.starts_with('S') || sign.starts_with('W') {
                    val = -val;
                }
            }
        }
    }
    
    Some(val)
}

/// Extracts metadata from the RAW/DNG file path.
pub fn extract_metadata<P: AsRef<Path>>(path: P) -> std::io::Result<ImageMetadata> {
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let exif_reader = Reader::new();
    
    let exif = exif_reader.read_from_container(&mut reader)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        
    let mut camera_make = None;
    let mut camera_model = None;
    let mut lens_model = None;
    let mut iso = None;
    let mut aperture = None;
    let mut shutter_speed = None;
    let mut focal_length = None;
    let mut taken_time = None;
    let mut description = None;
    
    if let Some(field) = exif.get_field(Tag::Make, In::PRIMARY) {
        if let Value::Ascii(ref v) = field.value {
            if let Some(s) = v.get(0) {
                if let Ok(string) = std::str::from_utf8(s) {
                    camera_make = Some(string.trim().to_string());
                }
            }
        }
    }
    
    if let Some(field) = exif.get_field(Tag::Model, In::PRIMARY) {
        if let Value::Ascii(ref v) = field.value {
            if let Some(s) = v.get(0) {
                if let Ok(string) = std::str::from_utf8(s) {
                    camera_model = Some(string.trim().to_string());
                }
            }
        }
    }
    
    if let Some(field) = exif.get_field(Tag::LensModel, In::PRIMARY) {
        if let Value::Ascii(ref v) = field.value {
            if let Some(s) = v.get(0) {
                if let Ok(string) = std::str::from_utf8(s) {
                    lens_model = Some(string.trim().to_string());
                }
            }
        }
    }
    
    if let Some(field) = exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY) {
        if let Value::Short(ref v) = field.value {
            if let Some(&iso_val) = v.get(0) {
                iso = Some(iso_val as u32);
            }
        }
    }
    
    if let Some(field) = exif.get_field(Tag::FNumber, In::PRIMARY) {
        if let Value::Rational(ref v) = field.value {
            if let Some(ratio) = v.get(0) {
                aperture = Some(ratio.num as f32 / ratio.denom as f32);
            }
        }
    }
    
    if let Some(field) = exif.get_field(Tag::FocalLength, In::PRIMARY) {
        if let Value::Rational(ref v) = field.value {
            if let Some(ratio) = v.get(0) {
                focal_length = Some(ratio.num as f32 / ratio.denom as f32);
            }
        }
    }
    
    if let Some(field) = exif.get_field(Tag::ExposureTime, In::PRIMARY) {
        if let Value::Rational(ref v) = field.value {
            if let Some(ratio) = v.get(0) {
                if ratio.num >= ratio.denom {
                    shutter_speed = Some(format!("{}", ratio.num / ratio.denom));
                } else {
                    shutter_speed = Some(format!("{}/{}", ratio.num, ratio.denom));
                }
            }
        }
    }
    
    if let Some(field) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
        if let Value::Ascii(ref v) = field.value {
            if let Some(s) = v.get(0) {
                if let Ok(string) = std::str::from_utf8(s) {
                    taken_time = parse_exif_date(string);
                }
            }
        }
    }
    
    if let Some(field) = exif.get_field(Tag::ImageDescription, In::PRIMARY) {
        if let Value::Ascii(ref v) = field.value {
            if let Some(s) = v.get(0) {
                if let Ok(string) = std::str::from_utf8(s) {
                    description = Some(string.trim().to_string());
                }
            }
        }
    }
    
    let latitude = parse_gps(&exif, Tag::GPSLatitude, Tag::GPSLatitudeRef);
    let longitude = parse_gps(&exif, Tag::GPSLongitude, Tag::GPSLongitudeRef);
    
    Ok(ImageMetadata {
        file_path: path.as_ref().to_string_lossy().into_owned(),
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
        description,
    })
}

/// Checks if a candidate JPEG stream contains a displayable RGB Start-of-Frame (SOF0/SOF1/SOF2)
/// and is NOT a 14-bit Lossless Bayer RAW sensor stream (SOF3).
fn is_displayable_rgb_jpeg(bytes: &[u8]) -> bool {
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return false;
    }
    
    let mut pos = 2;
    while pos + 3 < bytes.len() {
        if bytes[pos] != 0xFF {
            pos += 1;
            continue;
        }
        let marker = bytes[pos + 1];
        if marker == 0xFF || marker == 0x00 {
            pos += 1;
            continue;
        }
        // SOF0 (0xC0) = Baseline DCT (Standard displayable preview)
        // SOF1 (0xC1) = Extended Sequential DCT
        // SOF2 (0xC2) = Progressive DCT
        if marker == 0xC0 || marker == 0xC1 || marker == 0xC2 {
            return true;
        }
        // SOF3 (0xC3) = Lossless Huffman (Canon RAW sensor Bayer data) -> Reject
        if marker == 0xC3 {
            return false;
        }
        // Start of Scan (SOS): image scan begins, header markers ended
        if marker == 0xDA {
            break;
        }
        // Variable-length markers (APPn, DQT, DHT, DRI, COM)
        if (0xE0..=0xEF).contains(&marker) || marker == 0xDB || marker == 0xC4 || marker == 0xDD || marker == 0xFE {
            if pos + 3 >= bytes.len() {
                break;
            }
            let len = ((bytes[pos + 2] as usize) << 8) | (bytes[pos + 3] as usize);
            if len < 2 {
                break;
            }
            pos += 2 + len;
        } else {
            pos += 2;
        }
    }
    false
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
    
    // Fast path: If the entire file is already an RGB JPEG image
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

