use exif::{In, Reader, Tag, Value};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExifOrientationRecord {
    pub rotation: i32,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

/// Decodes an EXIF orientation tag (1-8) into canonical rotation degrees (0, 90, 180, 270)
/// and horizontal/vertical flip booleans.
pub fn decode_exif_orientation(orientation: u32) -> ExifOrientationRecord {
    let (rotation, flip_horizontal, flip_vertical) = match orientation {
        1 => (0, false, false),    // Normal
        2 => (0, true, false),     // Flip horizontal
        3 => (180, false, false),  // Rotate 180
        4 => (0, false, true),     // Flip vertical
        5 => (270, true, false),   // Transpose: Flip horizontal + Rotate 270 CW
        6 => (90, false, false),   // Rotate 90 CW
        7 => (90, true, false),    // Transverse: Flip horizontal + Rotate 90 CW
        8 => (270, false, false),  // Rotate 270 CW (90 CCW)
        _ => (0, false, false),
    };
    ExifOrientationRecord {
        rotation,
        flip_horizontal,
        flip_vertical,
    }
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageMetadataRecord {
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
    pub raw_width: Option<u32>,
    pub raw_height: Option<u32>,
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreviewResultRecord {
    pub preview_offset: u64,
    pub preview_len: u64,
    pub width: u32,
    pub height: u32,
    pub is_jpeg: bool,
    pub metadata: ImageMetadataRecord,
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RawLinearResultRecord {
    pub linear_offset: u64,
    pub linear_len: u64,
    pub width: u32,
    pub height: u32,
    pub is_linear: bool,
    pub metadata: ImageMetadataRecord,
}

#[derive(Clone, Copy, Debug)]
pub struct JpegStreamInfo {
    pub start: usize,
    pub length: usize,
    pub width: u32,
    pub height: u32,
    pub is_displayable_rgb: bool,
}

pub fn parse_jpeg_stream_at(bytes: &[u8], start: usize) -> Option<JpegStreamInfo> {
    if start + 4 > bytes.len() || bytes[start] != 0xFF || bytes[start + 1] != 0xD8 {
        return None;
    }

    let mut pos = start + 2;
    let mut width = 0u32;
    let mut height = 0u32;
    let mut is_displayable_rgb = false;

    while pos + 1 < bytes.len() {
        if bytes[pos] != 0xFF {
            return None;
        }

        while pos < bytes.len() && bytes[pos] == 0xFF {
            pos += 1;
        }
        if pos >= bytes.len() {
            return None;
        }

        let marker = bytes[pos];
        pos += 1;

        if marker == 0xD9 {
            let length = pos - start;
            if is_displayable_rgb && length >= 512 {
                return Some(JpegStreamInfo {
                    start,
                    length,
                    width,
                    height,
                    is_displayable_rgb,
                });
            }
            return None;
        }
        if marker == 0xD8 {
            return None;
        }
        if (0xD0..=0xD7).contains(&marker) || marker == 0x01 {
            continue;
        }

        if (0xC0..=0xCF).contains(&marker) && marker != 0xC3 && marker != 0xC4 && marker != 0xC8 && marker != 0xCC {
            if pos + 7 <= bytes.len() {
                height = ((bytes[pos + 3] as u32) << 8) | (bytes[pos + 4] as u32);
                width = ((bytes[pos + 5] as u32) << 8) | (bytes[pos + 6] as u32);
                let num_components = bytes[pos + 7];
                if num_components >= 1 {
                    is_displayable_rgb = true;
                }
            }
        } else if marker == 0xC3 {
            // Lossless JPEG (SOF3) -> raw Bayer data, not browser displayable standard JPEG!
            is_displayable_rgb = false;
        }

        if marker == 0xDA { // SOS
            if pos + 2 > bytes.len() {
                return None;
            }
            let sos_len = ((bytes[pos] as usize) << 8) | (bytes[pos + 1] as usize);
            if sos_len < 2 || pos + sos_len > bytes.len() {
                return None;
            }
            pos += sos_len;

            while pos + 1 < bytes.len() {
                if bytes[pos] == 0xFF {
                    let next_b = bytes[pos + 1];
                    if next_b == 0x00 {
                        pos += 2;
                        continue;
                    }
                    if next_b == 0xD9 {
                        pos += 2;
                        let length = pos - start;
                        if is_displayable_rgb && length >= 512 {
                            return Some(JpegStreamInfo {
                                start,
                                length,
                                width,
                                height,
                                is_displayable_rgb,
                            });
                        }
                        return None;
                    }
                    if (0xD0..=0xD7).contains(&next_b) {
                        pos += 2;
                        continue;
                    }
                    if next_b == 0xFF {
                        pos += 1;
                        continue;
                    }
                    pos += 1;
                    continue;
                }
                pos += 1;
            }
            return None;
        }

        if pos + 2 > bytes.len() {
            return None;
        }
        let payload_len = ((bytes[pos] as usize) << 8) | (bytes[pos + 1] as usize);
        if payload_len < 2 {
            return None;
        }
        pos += payload_len;
    }

    None
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

pub fn get_jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if let Some(info) = parse_jpeg_stream_at(bytes, 0) {
        if info.width > 0 && info.height > 0 {
            return Some((info.width, info.height));
        }
    }
    None
}

pub fn extract_preview_slice(bytes: &[u8]) -> Option<(usize, usize)> {
    let len = bytes.len();
    if len < 4 {
        return None;
    }
    if bytes[0] == 0xFF && bytes[1] == 0xD8 {
        if let Some(info) = parse_jpeg_stream_at(bytes, 0) {
            if info.is_displayable_rgb {
                return Some((0, info.length.min(len)));
            }
        }
    }

    let mut candidates: Vec<JpegStreamInfo> = Vec::new();

    // 1. TIFF / DNG IFD Tag Inspection
    let is_le = if len >= 4 {
        if &bytes[0..2] == b"II" {
            Some(true)
        } else if &bytes[0..2] == b"MM" {
            Some(false)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(le) = is_le {
        let mut visited_ifds = std::collections::HashSet::new();
        let mut ifd_queue = Vec::new();

        if let Some(ifd0) = read_u32(bytes, 4, le) {
            ifd_queue.push(ifd0 as usize);
        }

        while let Some(ifd_off) = ifd_queue.pop() {
            if ifd_off == 0 || ifd_off + 2 > bytes.len() || !visited_ifds.insert(ifd_off) {
                continue;
            }
            let entry_count = match read_u16(bytes, ifd_off, le) {
                Some(c) => c as usize,
                None => continue,
            };

            let mut next_ifd_off = 0usize;
            if ifd_off + 2 + entry_count * 12 + 4 <= bytes.len() {
                if let Some(n) = read_u32(bytes, ifd_off + 2 + entry_count * 12, le) {
                    next_ifd_off = n as usize;
                }
            }
            if next_ifd_off > 0 {
                ifd_queue.push(next_ifd_off);
            }

            let mut ifd_w = 0u32;
            let mut ifd_h = 0u32;

            for k in 0..entry_count {
                let entry_off = ifd_off + 2 + k * 12;
                if entry_off + 12 > bytes.len() {
                    break;
                }
                let tag = match read_u16(bytes, entry_off, le) { Some(t) => t, None => continue };
                let val = match read_u32(bytes, entry_off + 8, le) { Some(v) => v as usize, None => continue };

                if tag == 0x0100 {
                    ifd_w = val as u32;
                } else if tag == 0x0101 {
                    ifd_h = val as u32;
                } else if tag == 0x0201 || tag == 0x0111 {
                    let count = read_u32(bytes, entry_off + 4, le).unwrap_or(1) as usize;
                    if count == 1 {
                        if val + 4 < bytes.len() {
                            if let Some(mut info) = parse_jpeg_stream_at(bytes, val) {
                                if info.is_displayable_rgb && info.length >= 512 {
                                    if ifd_w > info.width { info.width = ifd_w; }
                                    if ifd_h > info.height { info.height = ifd_h; }
                                    candidates.push(info);
                                }
                            }
                        }
                    } else if count > 1 && count < 100 {
                        for i in 0..count {
                            let off_pos = val + i * 4;
                            if let Some(off) = read_u32(bytes, off_pos, le) {
                                let off_u = off as usize;
                                if off_u + 4 < bytes.len() {
                                    if let Some(mut info) = parse_jpeg_stream_at(bytes, off_u) {
                                        if info.is_displayable_rgb && info.length >= 512 {
                                            if ifd_w > info.width { info.width = ifd_w; }
                                            if ifd_h > info.height { info.height = ifd_h; }
                                            candidates.push(info);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if tag == 0x014A || tag == 0x8769 { // SubIFDs (0x014A) or ExifIFD (0x8769)
                    let count = read_u32(bytes, entry_off + 4, le).unwrap_or(1) as usize;
                    if count == 1 {
                        if val + 2 <= bytes.len() {
                            ifd_queue.push(val);
                        }
                    } else if count > 1 && count < 100 {
                        // `val` is a file offset pointing to an array of `count` SubIFD offsets
                        for i in 0..count {
                            let sub_off_pos = val + i * 4;
                            if let Some(sub_off) = read_u32(bytes, sub_off_pos, le) {
                                let sub_u = sub_off as usize;
                                if sub_u + 2 <= bytes.len() {
                                    ifd_queue.push(sub_u);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fast-path: If TIFF IFD parsing found a high-res preview (>= 640x480), return immediately
        if let Some(best_ifd) = candidates
            .iter()
            .cloned()
            .max_by_key(|info| (info.width * info.height, info.length))
        {
            if best_ifd.width * best_ifd.height >= 640 * 480 {
                return Some((best_ifd.start, best_ifd.length));
            }
        }
    }

    // 2. Stream byte scan fallback only if TIFF IFD tags didn't yield a high-res preview
    let mut i = 0;
    while i < len - 1 {
        if bytes[i] == 0xFF && bytes[i + 1] == 0xD8 {
            if let Some(info) = parse_jpeg_stream_at(bytes, i) {
                if info.is_displayable_rgb && info.length >= 512 {
                    candidates.push(info);
                }
                i += info.length.max(1);
                continue;
            } else {
                // Brute-force SOF scanner for JPEG streams where marker parsing failed
                let search_limit = (i + 4096).min(len);
                let mut p = i + 2;
                let mut stream_w = 0u32;
                let mut stream_h = 0u32;
                let mut found_sof = false;

                while p + 8 < search_limit {
                    if bytes[p] == 0xFF {
                        let m = bytes[p + 1];
                        if (0xC0..=0xCF).contains(&m) && m != 0xC3 && m != 0xC4 && m != 0xC8 && m != 0xCC {
                            stream_h = ((bytes[p + 5] as u32) << 8) | (bytes[p + 6] as u32);
                            stream_w = ((bytes[p + 7] as u32) << 8) | (bytes[p + 8] as u32);
                            found_sof = true;
                            break;
                        }
                        if m == 0xDA { break; }
                    }
                    p += 1;
                }

                if found_sof && stream_w >= 640 && stream_h >= 480 {
                    let mut stream_len = len - i;
                    for eoi_pos in (i + 100)..len - 1 {
                        if bytes[eoi_pos] == 0xFF && bytes[eoi_pos + 1] == 0xD8 && eoi_pos > i + 100 {
                            stream_len = eoi_pos - i;
                            break;
                        }
                        if bytes[eoi_pos] == 0xFF && bytes[eoi_pos + 1] == 0xD9 {
                            stream_len = (eoi_pos + 2) - i;
                            break;
                        }
                    }
                    candidates.push(JpegStreamInfo {
                        start: i,
                        length: stream_len,
                        width: stream_w,
                        height: stream_h,
                        is_displayable_rgb: true,
                    });
                    i += stream_len.max(1);
                    continue;
                }
            }
        }
        i += 1;
    }

    // Pick candidate with largest pixel resolution area (width * height), or largest length if resolutions match
    candidates
        .into_iter()
        .max_by_key(|info| (info.width * info.height, info.length))
        .map(|info| (info.start, info.length))
}

fn read_u16(bytes: &[u8], off: usize, is_le: bool) -> Option<u16> {
    if off + 2 > bytes.len() { return None; }
    if is_le {
        Some(u16::from_le_bytes([bytes[off], bytes[off + 1]]))
    } else {
        Some(u16::from_be_bytes([bytes[off], bytes[off + 1]]))
    }
}

fn read_u32(bytes: &[u8], off: usize, is_le: bool) -> Option<u32> {
    if off + 4 > bytes.len() { return None; }
    if is_le {
        Some(u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]]))
    } else {
        Some(u32::from_be_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]]))
    }
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

fn parse_gps(exif: &exif::Exif, tag: Tag, ref_tag: Tag) -> Option<f64> {
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
        if let Some(s) = v.first() {
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

pub fn extract_metadata_from_bytes(bytes: &[u8]) -> ImageMetadataRecord {
    let mut cursor = Cursor::new(bytes);
    let exif_reader = Reader::new();
    let exif = exif_reader.read_from_container(&mut cursor).ok();

    let mut camera_make = None;
    let mut camera_model = None;
    let mut lens_model = None;
    let mut iso = None;
    let mut aperture = None;
    let mut shutter_speed = None;
    let mut focal_length = None;
    let mut taken_time = None;
    let mut latitude = None;
    let mut longitude = None;
    let mut description = None;
    let mut creator = None;
    let mut copyright = None;
    let mut orientation = None;
    let mut raw_width = None;
    let mut raw_height = None;

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
        latitude = parse_gps(exif, Tag::GPSLatitude, Tag::GPSLatitudeRef);
        longitude = parse_gps(exif, Tag::GPSLongitude, Tag::GPSLongitudeRef);

        if let Some(field) = exif.get_field(Tag::ImageDescription, In::PRIMARY) {
            if let Value::Ascii(ref v) = field.value {
                if let Some(s) = v.first() {
                    if let Ok(string) = std::str::from_utf8(s) {
                        description = Some(string.trim().to_string());
                    }
                }
            }
        }

        if let Some(field) = exif.get_field(Tag::Artist, In::PRIMARY) {
            if let Value::Ascii(ref v) = field.value {
                if let Some(s) = v.first() {
                    if let Ok(string) = std::str::from_utf8(s) {
                        creator = Some(string.trim().to_string());
                    }
                }
            }
        }

        if let Some(field) = exif.get_field(Tag::Copyright, In::PRIMARY) {
            if let Value::Ascii(ref v) = field.value {
                if let Some(s) = v.first() {
                    if let Ok(string) = std::str::from_utf8(s) {
                        copyright = Some(string.trim().to_string());
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

        let mut max_w = 0u32;
        let mut max_h = 0u32;
        for field in exif.fields() {
            let tag_num = field.tag.number();
            if tag_num == 0xC620 {
                // DefaultCropSize in DNG (Tag 50720): 2 values [width, height]
                match &field.value {
                    Value::Rational(ref v) if v.len() >= 2 => {
                        if v[0].denom > 0 { max_w = max_w.max(v[0].num / v[0].denom); }
                        if v[1].denom > 0 { max_h = max_h.max(v[1].num / v[1].denom); }
                    }
                    Value::Long(ref v) if v.len() >= 2 => {
                        max_w = max_w.max(v[0]);
                        max_h = max_h.max(v[1]);
                    }
                    Value::Short(ref v) if v.len() >= 2 => {
                        max_w = max_w.max(v[0] as u32);
                        max_h = max_h.max(v[1] as u32);
                    }
                    _ => {}
                }
            } else if tag_num == 0xA002 || field.tag == Tag::PixelXDimension {
                match field.value {
                    Value::Short(ref v) => { if let Some(&w) = v.first() { max_w = max_w.max(w as u32); } }
                    Value::Long(ref v) => { if let Some(&w) = v.first() { max_w = max_w.max(w); } }
                    _ => {}
                }
            } else if tag_num == 0xA003 || field.tag == Tag::PixelYDimension {
                match field.value {
                    Value::Short(ref v) => { if let Some(&h) = v.first() { max_h = max_h.max(h as u32); } }
                    Value::Long(ref v) => { if let Some(&h) = v.first() { max_h = max_h.max(h); } }
                    _ => {}
                }
            }
        }
        if max_w > 0 { raw_width = Some(max_w); }
        if max_h > 0 { raw_height = Some(max_h); }
    }

    // Direct TIFF IFD sweep to extract full sensor resolution (e.g. SubIFD1 raw dimensions 5184x3456)
    let len = bytes.len();
    let is_le = if len >= 4 {
        if &bytes[0..2] == b"II" { Some(true) }
        else if &bytes[0..2] == b"MM" { Some(false) }
        else { None }
    } else { None };

    if let Some(le) = is_le {
        let mut visited_ifds = std::collections::HashSet::new();
        let mut ifd_queue = Vec::new();
        if let Some(ifd0) = read_u32(bytes, 4, le) {
            ifd_queue.push(ifd0 as usize);
        }

        let mut max_tiff_w = raw_width.unwrap_or(0);
        let mut max_tiff_h = raw_height.unwrap_or(0);

        while let Some(ifd_off) = ifd_queue.pop() {
            if ifd_off == 0 || ifd_off + 2 > bytes.len() || !visited_ifds.insert(ifd_off) {
                continue;
            }
            let entry_count = match read_u16(bytes, ifd_off, le) {
                Some(c) => c as usize,
                None => continue,
            };

            let mut next_ifd_off = 0usize;
            if ifd_off + 2 + entry_count * 12 + 4 <= bytes.len() {
                if let Some(n) = read_u32(bytes, ifd_off + 2 + entry_count * 12, le) {
                    next_ifd_off = n as usize;
                }
            }
            if next_ifd_off > 0 {
                ifd_queue.push(next_ifd_off);
            }

            for k in 0..entry_count {
                let entry_off = ifd_off + 2 + k * 12;
                if entry_off + 12 > bytes.len() { break; }
                let tag = match read_u16(bytes, entry_off, le) { Some(t) => t, None => continue };
                let val = match read_u32(bytes, entry_off + 8, le) { Some(v) => v as usize, None => continue };

                if tag == 0x0100 { // ImageWidth
                    max_tiff_w = max_tiff_w.max(val as u32);
                } else if tag == 0x0101 { // ImageLength
                    max_tiff_h = max_tiff_h.max(val as u32);
                } else if tag == 0x0112 && orientation.is_none() { // Orientation
                    if let Some(orient_val) = read_u16(bytes, entry_off + 8, le) {
                        if (1..=8).contains(&orient_val) {
                            orientation = Some(orient_val as u32);
                        }
                    }
                } else if tag == 0x014A || tag == 0x8769 { // SubIFDs or ExifIFD
                    let count = read_u32(bytes, entry_off + 4, le).unwrap_or(1) as usize;
                    if count == 1 {
                        if val + 2 <= bytes.len() { ifd_queue.push(val); }
                    } else if count > 1 && count < 100 {
                        for i in 0..count {
                            let sub_off_pos = val + i * 4;
                            if let Some(sub_off) = read_u32(bytes, sub_off_pos, le) {
                                let sub_u = sub_off as usize;
                                if sub_u + 2 <= bytes.len() { ifd_queue.push(sub_u); }
                            }
                        }
                    }
                }
            }
        }

        if max_tiff_w > 0 { raw_width = Some(max_tiff_w); }
        if max_tiff_h > 0 { raw_height = Some(max_tiff_h); }
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
        latitude,
        longitude,
        description,
        creator,
        copyright,
        orientation,
        raw_width,
        raw_height,
    }
}
