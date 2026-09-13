use crate::{CurvePointRecord, EditSettings};

#[derive(Clone, Debug)]
pub struct XmpData {
    pub rating: Option<i32>,
    pub status: String, // "Pick", "Reject", "None"
    pub title: Option<String>,
    pub description: Option<String>,
    pub creator: Option<String>,
    pub copyright: Option<String>,
    pub keywords: Vec<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub edits: EditSettings,
}

impl Default for XmpData {
    fn default() -> Self {
        Self {
            rating: None,
            status: "None".to_string(),
            title: None,
            description: None,
            creator: None,
            copyright: None,
            keywords: Vec::new(),
            latitude: None,
            longitude: None,
            edits: EditSettings::default(),
        }
    }
}

/// Formats a decimal GPS coordinate into TIFF-style EXIF representation inside XMP.
/// e.g. 34.0522 -> "34,3.1320N"
pub fn format_gps_xmp(val: f64, is_lat: bool) -> String {
    let abs_val = val.abs();
    let deg = abs_val.floor() as u32;
    let min = (abs_val - deg as f64) * 60.0;
    
    let dir = if is_lat {
        if val >= 0.0 { "N" } else { "S" }
    } else {
        if val >= 0.0 { "E" } else { "W" }
    };
    
    format!("{},{:.4}{}", deg, min, dir)
}

/// Parses TIFF-style EXIF coordinate from XMP back to decimal degree float.
/// e.g. "34,3.1320N" -> 34.0522
pub fn parse_gps_xmp(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    
    let last_char = s.chars().last()?;
    let is_negative = match last_char {
        'S' | 's' | 'W' | 'w' => true,
        'N' | 'n' | 'E' | 'e' => false,
        _ => return None,
    };
    
    let coord_part = &s[..s.len() - 1];
    let parts: Vec<&str> = coord_part.split(',').collect();
    if parts.is_empty() {
        return None;
    }
    
    let deg: f64 = parts[0].trim().parse().ok()?;
    let min: f64 = if parts.len() > 1 {
        parts[1].trim().parse().ok()?
    } else {
        0.0
    };
    
    let sec: f64 = if parts.len() > 2 {
        parts[2].trim().parse().ok()?
    } else {
        0.0
    };
    
    let decimal = deg + (min / 60.0) + (sec / 3600.0);
    if is_negative {
        Some(-decimal)
    } else {
        Some(decimal)
    }
}

pub fn serialize_curve_points(pts: &[CurvePointRecord]) -> String {
    pts.iter().map(|p| format!("{:.4},{:.4}", p.x, p.y)).collect::<Vec<_>>().join(";")
}

pub fn parse_curve_points(s: &str) -> Vec<CurvePointRecord> {
    s.split(';')
        .filter_map(|pair| {
            let mut parts = pair.split(',');
            let x = parts.next()?.trim().parse::<f32>().ok()?;
            let y = parts.next()?.trim().parse::<f32>().ok()?;
            Some(CurvePointRecord { x, y })
        })
        .collect()
}
