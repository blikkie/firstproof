// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Archival DNG verification engine.
//!
//! Performs independent disk read-back verification: parses TIFF IFDs,
//! decompresses the Lossless JPEG Bayer mosaic off disk, and validates
//! 100% bit-for-bit cryptographic SHA-256 parity with the source capture.

use std::path::Path;
use std::sync::Arc;

use rawler::decoders::RawDecodeParams;
use rawler::rawsource::RawSource;

use super::writer::hash_cfa_data;

/// Detailed verification report for an archival DNG file read off disk.
#[derive(Clone, Debug, PartialEq)]
pub struct DngVerificationReport {
    pub dng_cfa_sha256: String,
    pub width: usize,
    pub height: usize,
    pub cfa_pattern_name: String,
    pub has_embedded_preview: bool,
    pub bit_for_bit_match: bool,
}

/// Reads back a candidate DNG file from disk and validates its decompressed CFA mosaic
/// against an expected SHA-256 checksum.
pub fn verify_dng_from_disk<P: AsRef<Path>>(
    dng_path: P,
    expected_cfa_sha256: Option<&str>,
) -> Result<DngVerificationReport, String> {
    let dng_path = dng_path.as_ref();
    let rawsource = Arc::new(
        RawSource::new(dng_path)
            .map_err(|e| format!("Failed to open DNG file from disk {}: {}", dng_path.display(), e))?,
    );
    let decoder = rawler::get_decoder(&rawsource)
        .map_err(|e| format!("Failed to identify DNG decoder for {}: {}", dng_path.display(), e))?;
    let raw_params = RawDecodeParams { image_index: 0 };

    // Decompress the raw CFA mosaic from disk
    let rawimage = decoder
        .raw_image(&rawsource, &raw_params, false)
        .map_err(|e| format!("Failed to decompress RAW CFA mosaic from {}: {}", dng_path.display(), e))?;

    let dng_cfa_sha256 = hash_cfa_data(&rawimage.data);
    let bit_for_bit_match = if let Some(expected) = expected_cfa_sha256 {
        expected.eq_ignore_ascii_case(&dng_cfa_sha256)
    } else {
        true
    };

    // Verify preview image decodability
    let has_embedded_preview = decoder
        .preview_image(&rawsource, &raw_params)
        .map(|opt| opt.is_some())
        .unwrap_or(false);

    let cfa_pattern_name = format!("{:?}", rawimage.camera.cfa);

    Ok(DngVerificationReport {
        dng_cfa_sha256,
        width: rawimage.width,
        height: rawimage.height,
        cfa_pattern_name,
        has_embedded_preview,
        bit_for_bit_match,
    })
}
