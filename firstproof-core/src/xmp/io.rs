use std::path::{Path, PathBuf};
use firstproof_engine::xmp::{generate_xmp, parse_xmp, XmpData};

/// Checks if an XMP record contains user edits or metadata adjustments beyond default identity.
pub fn has_custom_adjustments(data: &XmpData) -> bool {
    let default_5500 = firstproof_engine::settings::EditSettings::default();
    let mut default_5000 = default_5500.clone();
    default_5000.temperature = 5000.0;

    (data.edits != default_5500 && data.edits != default_5000)
        || data.rating.is_some()
        || (data.status != "None" && !data.status.is_empty())
        || data.title.is_some()
        || data.description.is_some()
        || data.creator.is_some()
        || data.copyright.is_some()
        || !data.keywords.is_empty()
        || data.latitude.is_some()
        || data.longitude.is_some()
}

/// Resolves the sidecar file path for reading XMP.
/// Supports path directly (if it exists), foo.CR2.xmp (Darktable), and foo.xmp (Lightroom/First Proof standard).
/// If both foo.CR2.xmp and foo.xmp exist, intelligently selects the one containing custom First Proof edits.
pub fn resolve_xmp_read_path<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    let p = path.as_ref();
    if p.extension().map(|e| e.to_string_lossy().eq_ignore_ascii_case("xmp")).unwrap_or(false)
        && p.exists() {
            return Some(p.to_path_buf());
        }
    let p_str = p.to_string_lossy();
    let xmp_sidecar = PathBuf::from(format!("{}.xmp", p_str));
    let xmp_stem = p.with_extension("xmp");

    if xmp_sidecar.exists() && xmp_stem.exists() {
        let sidecar_has_edits = std::fs::read_to_string(&xmp_sidecar).ok()
            .and_then(|c| parse_xmp(&c).ok())
            .map(|d| has_custom_adjustments(&d))
            .unwrap_or(false);
        let stem_has_edits = std::fs::read_to_string(&xmp_stem).ok()
            .and_then(|c| parse_xmp(&c).ok())
            .map(|d| has_custom_adjustments(&d))
            .unwrap_or(false);

        if stem_has_edits && !sidecar_has_edits {
            return Some(xmp_stem);
        }
        if sidecar_has_edits && !stem_has_edits {
            return Some(xmp_sidecar);
        }
        // If both have edits, pick the one modified more recently
        let stem_mtime = std::fs::metadata(&xmp_stem).and_then(|m| m.modified()).ok();
        let sidecar_mtime = std::fs::metadata(&xmp_sidecar).and_then(|m| m.modified()).ok();
        if stem_mtime >= sidecar_mtime {
            return Some(xmp_stem);
        }
        return Some(xmp_sidecar);
    }

    if xmp_sidecar.exists() {
        return Some(xmp_sidecar);
    }
    if xmp_stem.exists() {
        return Some(xmp_stem);
    }
    None
}

/// Helper to merge non-destructive metadata from secondary sidecar into primary if missing.
pub fn merge_metadata(primary: &mut XmpData, secondary: &XmpData) {
    if primary.rating.is_none() && secondary.rating.is_some() {
        primary.rating = secondary.rating;
    }
    if (primary.status == "None" || primary.status.is_empty()) && (secondary.status != "None" && !secondary.status.is_empty()) {
        primary.status = secondary.status.clone();
    }
    if primary.title.is_none() && secondary.title.is_some() {
        primary.title = secondary.title.clone();
    }
    if primary.description.is_none() && secondary.description.is_some() {
        primary.description = secondary.description.clone();
    }
    if primary.creator.is_none() && secondary.creator.is_some() {
        primary.creator = secondary.creator.clone();
    }
    if primary.copyright.is_none() && secondary.copyright.is_some() {
        primary.copyright = secondary.copyright.clone();
    }
    if primary.keywords.is_empty() && !secondary.keywords.is_empty() {
        primary.keywords = secondary.keywords.clone();
    }
    if primary.latitude.is_none() && secondary.latitude.is_some() {
        primary.latitude = secondary.latitude;
    }
    if primary.longitude.is_none() && secondary.longitude.is_some() {
        primary.longitude = secondary.longitude;
    }
}

/// Resolves the sidecar file path for writing XMP.
/// If an existing sidecar file already exists (e.g. foo.xmp or foo.CR2.xmp), updates that file.
/// If foo.xmp exists, prioritizes foo.xmp to maintain standard interoperability.
/// Otherwise defaults to foo.xmp.
pub fn resolve_xmp_write_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();
    if p.extension().map(|e| e.to_string_lossy().eq_ignore_ascii_case("xmp")).unwrap_or(false) {
        return p.to_path_buf();
    }
    let p_str = p.to_string_lossy();
    let xmp_stem = p.with_extension("xmp");
    if xmp_stem.exists() {
        return xmp_stem;
    }
    let xmp_sidecar = PathBuf::from(format!("{}.xmp", p_str));
    if xmp_sidecar.exists() {
        return xmp_sidecar;
    }
    xmp_stem
}

/// Reads XMP from a file path or photo path by resolving sidecars.
/// When both `foo.xmp` and `foo.CR2.xmp` exist, merges them so no metadata or edits are dropped.
pub fn read_xmp_file<P: AsRef<Path>>(path: P) -> std::io::Result<XmpData> {
    let p = path.as_ref();
    if p.extension().map(|e| e.to_string_lossy().eq_ignore_ascii_case("xmp")).unwrap_or(false) {
        let content = std::fs::read_to_string(p)?;
        return parse_xmp(&content);
    }

    let p_str = p.to_string_lossy();
    let xmp_sidecar = PathBuf::from(format!("{}.xmp", p_str));
    let xmp_stem = p.with_extension("xmp");

    let sidecar_data = if xmp_sidecar.exists() {
        std::fs::read_to_string(&xmp_sidecar).ok().and_then(|c| parse_xmp(&c).ok())
    } else {
        None
    };

    let stem_data = if xmp_stem.exists() {
        std::fs::read_to_string(&xmp_stem).ok().and_then(|c| parse_xmp(&c).ok())
    } else {
        None
    };

    match (sidecar_data, stem_data) {
        (Some(sidecar), Some(stem)) => {
            let sidecar_has_edits = has_custom_adjustments(&sidecar);
            let stem_has_edits = has_custom_adjustments(&stem);

            let (mut primary, secondary) = if stem_has_edits && !sidecar_has_edits {
                (stem, sidecar)
            } else if sidecar_has_edits && !stem_has_edits {
                (sidecar, stem)
            } else {
                // Both or neither have edits: pick the file modified more recently
                let stem_mtime = std::fs::metadata(&xmp_stem).and_then(|m| m.modified()).ok();
                let sidecar_mtime = std::fs::metadata(&xmp_sidecar).and_then(|m| m.modified()).ok();
                if stem_mtime >= sidecar_mtime {
                    (stem, sidecar)
                } else {
                    (sidecar, stem)
                }
            };

            merge_metadata(&mut primary, &secondary);
            Ok(primary)
        }
        (Some(data), None) => Ok(data),
        (None, Some(data)) => Ok(data),
        (None, None) => {
            // Sibling search if path is a DNG or converted container:
            // If neither photo.dng.xmp nor photo.xmp exists, check for sibling photo.<ext>.xmp (e.g. photo.CR2.xmp)
            if let (Some(parent), Some(stem)) = (p.parent(), p.file_stem()) {
                let stem_str = stem.to_string_lossy();
                let stem_dot = format!("{}.", stem_str);
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with(&stem_dot) && name.ends_with(".xmp") {
                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                if let Ok(parsed) = parse_xmp(&content) {
                                    return Ok(parsed);
                                }
                            }
                        }
                    }
                }
            }

            // Embedded XMP packet extraction fallback (e.g. DNG or TIFF with embedded XMP)
            if let Some(embedded) = extract_embedded_xmp(p) {
                return Ok(embedded);
            }

            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No XMP sidecar found"))
        }
    }
}

/// Extracts embedded XMP packet directly from an image container (DNG, TIFF, JPEG).
pub fn extract_embedded_xmp<P: AsRef<Path>>(path: P) -> Option<XmpData> {
    let bytes = std::fs::read(path).ok()?;
    let start_needle = b"<?xpacket begin=";
    let end_needle = b"<?xpacket end=";
    let start = bytes.windows(start_needle.len()).position(|w| w == start_needle)?;
    let end_rel = bytes[start..].windows(end_needle.len()).position(|w| w == end_needle)?;
    let end_close = bytes[start + end_rel..].iter().position(|&b| b == b'>')?;
    let xmp_bytes = &bytes[start..start + end_rel + end_close + 1];
    let xmp_str = std::str::from_utf8(xmp_bytes).ok()?;
    parse_xmp(xmp_str).ok()
}

/// Writes XMP to a file path or photo path by resolving sidecars.
pub fn write_xmp_file<P: AsRef<Path>>(path: P, data: &XmpData) -> std::io::Result<()> {
    let target = resolve_xmp_write_path(&path);
    let xml_str = generate_xmp(data);
    std::fs::write(target, xml_str)
}
