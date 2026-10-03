use std::path::{Path, PathBuf};
use firstproof_engine::xmp::{generate_xmp, parse_xmp, XmpData};

/// Checks if an XMP record contains user edits or metadata adjustments beyond default identity.
pub fn has_custom_adjustments(data: &XmpData) -> bool {
    data.edits.exposure != 0.0
        || data.edits.contrast != 0.0
        || data.edits.highlights != 0.0
        || data.edits.shadows != 0.0
        || data.edits.whites != 0.0
        || data.edits.blacks != 0.0
        || data.edits.vibrance != 0.0
        || data.edits.saturation != 0.0
        || data.edits.deep_blue != 0.0
        || data.edits.foliage != 0.0
        || data.edits.crop_angle != 0.0
        || data.edits.crop_left != 0.0
        || data.edits.crop_top != 0.0
        || data.edits.crop_right != 1.0
        || data.edits.crop_bottom != 1.0
        || data.edits.rotation != 0
        || data.edits.flip_horizontal
        || data.edits.flip_vertical
        || (data.edits.temperature != 5000.0 && data.edits.temperature != 0.0)
        || data.edits.tint != 0.0
        || data.edits.lut_name.is_some()
        || data.edits.emulsion_archetype != crate::EmulsionArchetype::None
        || data.edits.curves != crate::curve::ToneCurvesRecord::default()
        || data.rating.is_some()
        || (data.status != "None" && !data.status.is_empty())
        || data.title.is_some()
        || data.description.is_some()
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
fn merge_metadata(primary: &mut XmpData, secondary: &XmpData) {
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
        (None, None) => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No XMP sidecar found")),
    }
}

/// Writes XMP to a file path or photo path by resolving sidecars.
pub fn write_xmp_file<P: AsRef<Path>>(path: P, data: &XmpData) -> std::io::Result<()> {
    let target = resolve_xmp_write_path(&path);
    let xml_str = generate_xmp(data);
    std::fs::write(target, xml_str)
}
