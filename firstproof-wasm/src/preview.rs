use exif::{In, Reader, Tag, Value};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageMetadataRecord {
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub iso: Option<u32>,
    pub aperture: Option<f32>,
    pub shutter_speed: Option<String>,
    pub focal_length: Option<f32>,
    pub taken_time: Option<i64>,
    pub orientation: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreviewResultRecord {
    pub preview_offset: usize,
    pub preview_len: usize,
    pub width: u32,
    pub height: u32,
    pub is_jpeg: bool,
    pub metadata: ImageMetadataRecord,
}

pub fn is_displayable_rgb_jpeg(bytes: &[u8]) -> bool {
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
        // SOF0 (0xC0) = Baseline DCT
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
        if (0xE0..=0xEF).contains(&marker)
            || marker == 0xDB
            || marker == 0xC4
            || marker == 0xDD
            || marker == 0xFE
        {
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

pub fn get_jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return None;
    }
    let mut pos = 2;
    while pos + 8 < bytes.len() {
        if bytes[pos] != 0xFF {
            pos += 1;
            continue;
        }
        let marker = bytes[pos + 1];
        if marker == 0xC0 || marker == 0xC1 || marker == 0xC2 {
            let h = ((bytes[pos + 5] as u32) << 8) | (bytes[pos + 6] as u32);
            let w = ((bytes[pos + 7] as u32) << 8) | (bytes[pos + 8] as u32);
            return Some((w, h));
        }
        if (0xE0..=0xEF).contains(&marker)
            || marker == 0xDB
            || marker == 0xC4
            || marker == 0xDD
            || marker == 0xFE
        {
            let len = ((bytes[pos + 2] as usize) << 8) | (bytes[pos + 3] as usize);
            if len < 2 {
                break;
            }
            pos += 2 + len;
        } else {
            pos += 2;
        }
    }
    None
}

pub fn extract_preview_slice(bytes: &[u8]) -> Option<(usize, usize)> {
    let len = bytes.len();
    if len < 4 {
        return None;
    }
    // Fast path: Whole file is already a JPEG
    if bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF && is_displayable_rgb_jpeg(bytes) {
        return Some((0, len));
    }

    let mut jpegs = Vec::new();
    let mut i = 0;
    while i < len - 1 {
        if bytes[i] == 0xFF && bytes[i + 1] == 0xD8 {
            let start = i;
            let mut j = i + 2;
            let mut found_eoi = false;
            while j < len - 1 {
                if bytes[j] == 0xFF && bytes[j + 1] == 0xD9 {
                    found_eoi = true;
                    let end = j + 2;
                    let size = end - start;
                    if size > 10 * 1024 {
                        jpegs.push((start, size));
                    }
                    i = j;
                    break;
                }
                j += 1;
            }
            if !found_eoi {
                let size = len - start;
                if size > 10 * 1024 {
                    jpegs.push((start, size));
                }
            }
        }
        i += 1;
    }

    jpegs
        .into_iter()
        .filter(|&(start, size)| is_displayable_rgb_jpeg(&bytes[start..start + size]))
        .max_by_key(|&(_, size)| size)
}

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

    let mut days =
        (year - 1970) * 365 + (year - 1969) / 4 - (year - 1901) / 100 + (year - 1601) / 400;
    let month_days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    for m in 1..month {
        days += month_days[m as usize];
        if m == 2 && is_leap {
            days += 1;
        }
    }
    days += day - 1;
    Some((days as i64) * 86400 + (hour as i64) * 3600 + (min as i64) * 60 + (sec as i64))
}

pub fn extract_metadata_from_bytes(bytes: &[u8]) -> ImageMetadataRecord {
    let mut cursor = Cursor::new(bytes);
    let exif_reader = Reader::new();
    let exif = match exif_reader.read_from_container(&mut cursor) {
        Ok(e) => Some(e),
        Err(_) => None,
    };

    let mut camera_make = None;
    let mut camera_model = None;
    let mut lens_model = None;
    let mut iso = None;
    let mut aperture = None;
    let mut shutter_speed = None;
    let mut focal_length = None;
    let mut taken_time = None;
    let mut orientation = None;

    if let Some(ref exif) = exif {
        if let Some(field) = exif.get_field(Tag::Make, In::PRIMARY) {
            if let Value::Ascii(ref v) = field.value {
                if let Some(s) = v.first() {
                    if let Ok(string) = std::str::from_utf8(s) {
                        camera_make = Some(string.trim().to_string());
                    }
                }
            }
        }
        if let Some(field) = exif.get_field(Tag::Model, In::PRIMARY) {
            if let Value::Ascii(ref v) = field.value {
                if let Some(s) = v.first() {
                    if let Ok(string) = std::str::from_utf8(s) {
                        camera_model = Some(string.trim().to_string());
                    }
                }
            }
        }
        if let Some(field) = exif.get_field(Tag::LensModel, In::PRIMARY) {
            if let Value::Ascii(ref v) = field.value {
                if let Some(s) = v.first() {
                    if let Ok(string) = std::str::from_utf8(s) {
                        lens_model = Some(string.trim().to_string());
                    }
                }
            }
        }
        if let Some(field) = exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY) {
            if let Value::Short(ref v) = field.value {
                if let Some(&iso_val) = v.first() {
                    iso = Some(iso_val as u32);
                }
            }
        }
        if let Some(field) = exif.get_field(Tag::FNumber, In::PRIMARY) {
            if let Value::Rational(ref v) = field.value {
                if let Some(ratio) = v.first() {
                    if ratio.denom > 0 {
                        aperture = Some(ratio.num as f32 / ratio.denom as f32);
                    }
                }
            }
        }
        if let Some(field) = exif.get_field(Tag::FocalLength, In::PRIMARY) {
            if let Value::Rational(ref v) = field.value {
                if let Some(ratio) = v.first() {
                    if ratio.denom > 0 {
                        focal_length = Some(ratio.num as f32 / ratio.denom as f32);
                    }
                }
            }
        }
        if let Some(field) = exif.get_field(Tag::ExposureTime, In::PRIMARY) {
            if let Value::Rational(ref v) = field.value {
                if let Some(ratio) = v.first() {
                    if ratio.denom > 0 {
                        if ratio.num >= ratio.denom {
                            shutter_speed = Some(format!("{}", ratio.num / ratio.denom));
                        } else {
                            shutter_speed = Some(format!("{}/{}", ratio.num, ratio.denom));
                        }
                    }
                }
            }
        }
        if let Some(field) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
            if let Value::Ascii(ref v) = field.value {
                if let Some(s) = v.first() {
                    if let Ok(string) = std::str::from_utf8(s) {
                        taken_time = parse_exif_date(string);
                    }
                }
            }
        }
        if let Some(field) = exif
            .get_field(Tag::Orientation, In::PRIMARY)
            .or_else(|| exif.fields().find(|f| f.tag == Tag::Orientation))
        {
            if let Value::Short(ref v) = field.value {
                if let Some(&val) = v.first() {
                    orientation = Some(val as u32);
                }
            }
        }
    }

    ImageMetadataRecord {
        camera_make,
        camera_model,
        lens_model,
        iso,
        aperture,
        shutter_speed,
        focal_length,
        taken_time,
        orientation,
    }
}
