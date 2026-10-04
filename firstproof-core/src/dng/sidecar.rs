// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Sidecar continuity and migration logic for DNG conversion.
//!
//! Handles both Lightroom-standard (`photo.xmp`) and Darktable-standard (`photo.<ext>.xmp`)
//! sidecars, ensuring edits and ratings are never orphaned when original RAW files are deleted.

use std::path::{Path, PathBuf};

/// Discovers and reads the raw XMP packet string from existing sidecar files.
pub fn load_source_xmp_packet<P: AsRef<Path>>(source_path: P) -> Option<Vec<u8>> {
    let source_path = source_path.as_ref();
    let p_str = source_path.to_string_lossy();

    // 1. Try standard Lightroom sidecar: photo.xmp
    let stem_xmp = source_path.with_extension("xmp");
    if stem_xmp.exists() {
        if let Ok(bytes) = std::fs::read(&stem_xmp) {
            return Some(bytes);
        }
    }

    // 2. Try Darktable sidecar: photo.CR2.xmp
    let ext_xmp = PathBuf::from(format!("{}.xmp", p_str));
    if ext_xmp.exists() {
        if let Ok(bytes) = std::fs::read(&ext_xmp) {
            return Some(bytes);
        }
    }

    None
}

/// Migrates on-disk sidecars when the source RAW file is being safely unlinked.
///
/// If only a Darktable-style `photo.<ext>.xmp` sidecar was present, migrates it
/// to `photo.dng.xmp` and `photo.xmp` so third-party software continues to find it.
pub fn migrate_sidecars_on_deletion<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    dng_path: Q,
) -> Vec<PathBuf> {
    let mut migrated = Vec::new();
    let source_path = source_path.as_ref();
    let dng_path = dng_path.as_ref();

    let p_str = source_path.to_string_lossy();
    let dng_str = dng_path.to_string_lossy();

    let source_stem_xmp = source_path.with_extension("xmp");
    let source_ext_xmp = PathBuf::from(format!("{}.xmp", p_str));

    let dng_stem_xmp = dng_path.with_extension("xmp");
    let dng_ext_xmp = PathBuf::from(format!("{}.xmp", dng_str));

    // If Darktable-style photo.<ext>.xmp existed:
    if source_ext_xmp.exists() {
        // Copy to photo.dng.xmp (Darktable convention for the new DNG)
        if !dng_ext_xmp.exists() && std::fs::copy(&source_ext_xmp, &dng_ext_xmp).is_ok() {
            migrated.push(dng_ext_xmp.clone());
        }
        // Also ensure universal photo.xmp exists
        if !dng_stem_xmp.exists() && std::fs::copy(&source_ext_xmp, &dng_stem_xmp).is_ok() {
            migrated.push(dng_stem_xmp.clone());
        }
        // Unlink the old photo.<ext>.xmp
        let _ = std::fs::remove_file(&source_ext_xmp);
    }

    // If photo.xmp existed and dng is in a different directory or stem:
    if source_stem_xmp.exists()
        && source_stem_xmp != dng_stem_xmp
        && !dng_stem_xmp.exists()
        && std::fs::copy(&source_stem_xmp, &dng_stem_xmp).is_ok()
    {
        migrated.push(dng_stem_xmp.clone());
        let _ = std::fs::remove_file(&source_stem_xmp);
    }

    migrated
}
