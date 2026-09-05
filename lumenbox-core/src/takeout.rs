use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct GeoData {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TimeInfo {
    pub timestamp: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TakeoutJson {
    pub title: String,
    pub description: Option<String>,
    #[serde(rename = "geoData")]
    pub geo_data: Option<GeoData>,
    #[serde(rename = "geoDataExif")]
    pub geo_data_exif: Option<GeoData>,
    #[serde(rename = "photoTakenTime")]
    pub photo_taken_time: Option<TimeInfo>,
}

#[derive(Debug, Clone)]
pub struct TakeoutMetadata {
    pub title: String,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub taken_time: Option<i64>, // Unix timestamp
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TakeoutMatch {
    pub image_path: PathBuf,
    pub json_path: PathBuf,
}

/// Helper function to strip copy suffix like `(1)`, `(2)`, ` (1)` before file extension.
fn strip_copy_suffix(filename: &str) -> String {
    if let Some(pos) = filename.rfind('(') {
        if let Some(close_pos) = filename[pos..].find(')') {
            let inner = &filename[pos + 1..pos + close_pos];
            if inner.chars().all(|c| c.is_ascii_digit()) {
                let mut before = &filename[..pos];
                if before.ends_with(' ') {
                    before = before.trim_end();
                }
                let after = &filename[pos + close_pos + 1..];
                return format!("{}{}", before, after);
            }
        }
    }
    filename.to_string()
}

/// Matches an image file to its corresponding Google Takeout JSON sidecar file.
/// Handles edge cases like duplicate suffix `(1)`, filename truncation, and special characters.
pub fn match_takeout_json<P: AsRef<Path>>(image_path: P) -> Option<PathBuf> {
    let image_path = image_path.as_ref();
    let parent = image_path.parent()?;
    let filename = image_path.file_name()?.to_str()?;
    
    // 1. Fast Path: Check standard [filename].[ext].json
    let mut guess = parent.join(format!("{}.json", filename));
    if guess.exists() {
        return Some(guess);
    }
    
    // 2. Secondary Fast Path: Check [filename_without_ext].json
    if let Some(stem) = image_path.file_stem()?.to_str() {
        guess = parent.join(format!("{}.json", stem));
        if guess.exists() {
            return Some(guess);
        }
    }

    // 3. Duplicate Suffix Fast Path: e.g. image(1).jpg -> image.jpg.json or image.json or image.jpg(1).json
    let stripped_filename = strip_copy_suffix(filename);
    if stripped_filename != filename {
        guess = parent.join(format!("{}.json", stripped_filename));
        if guess.exists() {
            return Some(guess);
        }
        if let Some(stem) = Path::new(&stripped_filename).file_stem()?.to_str() {
            guess = parent.join(format!("{}.json", stem));
            if guess.exists() {
                return Some(guess);
            }
        }
        if let Some(ext) = image_path.extension()?.to_str() {
            if let Some(stem) = image_path.file_stem()?.to_str() {
                let stripped_stem = strip_copy_suffix(stem);
                let suffix_match = parent.join(format!("{}.{}(1).json", stripped_stem, ext));
                if suffix_match.exists() {
                    return Some(suffix_match);
                }
            }
        }
    }
    
    // 4. Truncation Fast Path: Google Takeout truncates filenames so total JSON filename is 45-52 chars
    for total_len in 45..=52 {
        let base_len = total_len - 5; // length before ".json"
        if filename.len() > base_len {
            if filename.is_char_boundary(base_len) {
                let candidate = parent.join(format!("{}.json", &filename[..base_len]));
                if candidate.exists() {
                    return Some(candidate);
                }
            }
            if let Some(stem) = image_path.file_stem().and_then(|s| s.to_str()) {
                if let Some(pos) = stem.rfind('(') {
                    let copy_suffix = &filename[pos..]; // e.g. "(1).jpg"
                    if base_len > copy_suffix.len() {
                        let p_len = base_len - copy_suffix.len();
                        if p_len <= pos && stem.is_char_boundary(p_len) {
                            let candidate = parent.join(format!("{}{}.json", &stem[..p_len], copy_suffix));
                            if candidate.exists() {
                                return Some(candidate);
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}

/// Helper to parse the "title" string from raw JSON text snippet
fn extract_json_title(content: &str) -> Option<String> {
    let key = "\"title\"";
    let key_idx = content.find(key)?;
    let rest = &content[key_idx + key.len()..];
    let colon_idx = rest.find(':')?;
    let quote_start = rest[colon_idx + 1..].find('"')? + colon_idx + 1;
    let quote_end = rest[quote_start + 1..].find('"')? + quote_start + 1;
    Some(rest[quote_start + 1..quote_end].to_string())
}

/// Recursively scans a directory for images/media and returns matched (image, JSON) sidecar pairs.
pub fn scan_directory_for_takeout<P: AsRef<Path>>(dir_path: P) -> Vec<TakeoutMatch> {
    let mut matches = Vec::new();
    let mut stack = vec![dir_path.as_ref().to_path_buf()];

    while let Some(current_dir) = stack.pop() {
        let entries = match fs::read_dir(&current_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !name.starts_with('.') {
                        stack.push(path);
                    }
                }
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if ext_lower == "json" || ext_lower == "xmp" {
                        continue;
                    }
                }
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') {
                        continue;
                    }
                }

                if let Some(json_path) = match_takeout_json(&path) {
                    matches.push(TakeoutMatch {
                        image_path: path,
                        json_path,
                    });
                }
            }
        }
    }

    matches
}

/// Parses a Google Takeout JSON file and extracts metadata.
pub fn parse_takeout_json<P: AsRef<Path>>(json_path: P) -> std::io::Result<TakeoutMetadata> {
    let file = File::open(json_path)?;
    let reader = BufReader::new(file);
    let raw: TakeoutJson = serde_json::from_reader(reader)?;
    
    let taken_time = raw.photo_taken_time
        .and_then(|t| t.timestamp.parse::<i64>().ok());
        
    let mut latitude = None;
    let mut longitude = None;
    let mut altitude = None;
    
    if let Some(geo) = raw.geo_data {
        if geo.latitude != 0.0 || geo.longitude != 0.0 {
            latitude = Some(geo.latitude);
            longitude = Some(geo.longitude);
            altitude = Some(geo.altitude);
        }
    }
    
    if latitude.is_none() {
        if let Some(geo_exif) = raw.geo_data_exif {
            if geo_exif.latitude != 0.0 || geo_exif.longitude != 0.0 {
                latitude = Some(geo_exif.latitude);
                longitude = Some(geo_exif.longitude);
                altitude = Some(geo_exif.altitude);
            }
        }
    }
    
    let description = raw.description
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty());
        
    Ok(TakeoutMetadata {
        title: raw.title,
        description,
        latitude,
        longitude,
        altitude,
        taken_time,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_copy_suffix() {
        assert_eq!(strip_copy_suffix("photo(1).jpg"), "photo.jpg");
        assert_eq!(strip_copy_suffix("photo (2).jpg"), "photo.jpg");
        assert_eq!(strip_copy_suffix("photo.jpg"), "photo.jpg");
    }

    #[test]
    fn test_takeout_matching_and_parsing() {
        let temp_dir = std::env::temp_dir().join("lumenbox_test_takeout");
        let _ = fs::create_dir_all(&temp_dir);

        let image_filename = "my_holiday_photo_with_long_name_and_stuff(1).jpg";
        let json_filename = "my_holiday_photo_with_long_name_an(1).jpg.json";
        
        let json_path = temp_dir.join(json_filename);
        let image_path = temp_dir.join(image_filename);
        
        let json_content = r#"{
  "title": "my_holiday_photo_with_long_name_and_stuff(1).jpg",
  "description": "A beautiful sunset over the hills",
  "geoData": {
    "latitude": 37.7749,
    "longitude": -122.4194,
    "altitude": 10.0
  },
  "geoDataExif": {
    "latitude": 0.0,
    "longitude": 0.0,
    "altitude": 0.0
  },
  "photoTakenTime": {
    "timestamp": "1626468000"
  }
}"#;
        
        fs::write(&json_path, json_content).unwrap();
        fs::write(&image_path, "dummy image content").unwrap();
        
        let matched = match_takeout_json(&image_path).expect("Should find matching JSON");
        assert_eq!(matched, json_path);
        
        let metadata = parse_takeout_json(&matched).unwrap();
        assert_eq!(metadata.title, image_filename);
        assert_eq!(metadata.description.as_deref(), Some("A beautiful sunset over the hills"));
        assert_eq!(metadata.latitude, Some(37.7749));
        assert_eq!(metadata.longitude, Some(-122.4194));
        assert_eq!(metadata.taken_time, Some(1626468000));

        // Test scanner
        let scanned_matches = scan_directory_for_takeout(&temp_dir);
        assert_eq!(scanned_matches.len(), 1);
        assert_eq!(scanned_matches[0].image_path, image_path);
        assert_eq!(scanned_matches[0].json_path, json_path);

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
