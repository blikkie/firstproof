// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Archival Open DNG Converter & Lossless Compression Integration Test Suite.
//!
//! Verifies:
//! 1. Zero-loss Bayer mosaic transcoding from proprietary Canon CR2 to Adobe DNG 1.4+.
//! 2. Post-write independent disk read-back cryptographic SHA-256 verification.
//! 3. Authentic AgX preview embedding and metadata preservation.
//! 4. Full darkroom developability of the resulting DNG container.
//! 5. Sandboxed destructive path (`--delete-original`) and Darktable sidecar migration.
//! 6. Failure-safety invariants preventing deletion upon error.

use std::fs;
use std::path::{Path, PathBuf};

use firstproof_core::dng::{convert_raw_to_dng, verify_dng_file, DngConvertOptions};
use firstproof_core::export::export_photo_file_advanced;
use firstproof_core::raw_decoder::{RawDecoder, RawlerDecoder};
use firstproof_core::xmp::read_xmp_file;
use firstproof_core::{
    ColorProfileMode, DemosaicQuality, EditSettings, ExportColorSpace, ExportRenderingIntent,
    RenderTier,
};

#[test]
fn test_dng_conversion_and_disk_verification() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cr2_path = manifest_dir.join("tests/fixtures/_MG_2651.CR2");

    if !cr2_path.exists() {
        eprintln!("CR2 fixture not found at {:?}, skipping test", cr2_path);
        return;
    }

    let out_dir = std::env::temp_dir().join("firstproof_dng_test");
    let _ = fs::create_dir_all(&out_dir);
    let dng_out = out_dir.join("mg_2651_archival.dng");
    let _ = fs::remove_file(&dng_out);

    let options = DngConvertOptions {
        output_path: Some(dng_out.clone()),
        delete_original: false,
        verify: true,
        quiet: true,
    };

    let report = convert_raw_to_dng(&cr2_path, &options).expect("Conversion must succeed");

    assert!(dng_out.exists(), "Converted DNG file must exist on disk");
    assert!(report.verification_passed, "Disk read-back verification must pass");
    assert_eq!(
        report.source_cfa_sha256, report.disk_cfa_sha256,
        "Source CFA SHA-256 and DNG CFA SHA-256 must match bit-for-bit"
    );
    assert!(
        report.dng_size_bytes < report.source_size_bytes,
        "Lossless compression must reduce file size: source {} bytes vs DNG {} bytes",
        report.source_size_bytes,
        report.dng_size_bytes
    );
    println!(
        "DNG compression: {:.2} MB -> {:.2} MB ({:.1}% saved)",
        report.source_size_bytes as f64 / 1_048_576.0,
        report.dng_size_bytes as f64 / 1_048_576.0,
        report.compression_ratio
    );

    // 2. Standalone verification function test
    let verify_res = verify_dng_file(&cr2_path, &dng_out).expect("Standalone verification check");
    assert!(verify_res, "Standalone verification must confirm bit-for-bit match");

    // 3. Verify that the generated DNG loads cleanly via RawDecoder
    let decoder = RawlerDecoder;
    let dng_frame = RawDecoder::decode(&decoder, &dng_out).expect("Decode converted DNG frame");
    assert_eq!(dng_frame.camera_model, "Canon EOS 60D");
    assert_eq!(dng_frame.active_width, 5184);
    assert_eq!(dng_frame.active_height, 3456);

    // 4. Verify that the generated DNG can be developed into a JPEG
    let developed_jpg = out_dir.join("developed_from_dng.jpg");
    let _ = fs::remove_file(&developed_jpg);

    let edits = EditSettings {
        exposure: 0.5,
        tone_profile: ColorProfileMode::AgxVivid,
        ..Default::default()
    };

    let dev_res = export_photo_file_advanced(
        &dng_out,
        &developed_jpg,
        &edits,
        None,
        90,
        Some(1024),
        ExportColorSpace::Srgb,
        ExportRenderingIntent::RelativeColorimetric,
        false,
        DemosaicQuality::Standard,
        RenderTier::Viewport,
    );
    assert!(dev_res.is_ok(), "Developing converted DNG must succeed: {:?}", dev_res);
    assert!(developed_jpg.exists(), "Developed preview JPEG must exist");
    assert!(
        fs::metadata(&developed_jpg).unwrap().len() > 10_000,
        "Developed JPEG must have non-trivial size"
    );

    // Cleanup
    let _ = fs::remove_file(&developed_jpg);
    let _ = fs::remove_file(&dng_out);
    let _ = fs::remove_dir(&out_dir);
}

#[test]
fn test_dng_destructive_path_and_sidecar_migration() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cr2_src = manifest_dir.join("tests/fixtures/_MG_2651.CR2");

    if !cr2_src.exists() {
        eprintln!("CR2 fixture not found at {:?}, skipping test", cr2_src);
        return;
    }

    let sandbox_dir = std::env::temp_dir().join("firstproof_dng_destructive_test");
    let _ = fs::create_dir_all(&sandbox_dir);

    let sandboxed_cr2 = sandbox_dir.join("sample.CR2");
    let sandboxed_xmp = sandbox_dir.join("sample.CR2.xmp"); // Darktable naming convention
    let target_dng = sandbox_dir.join("sample.dng");
    let expected_dng_xmp = sandbox_dir.join("sample.dng.xmp");
    let expected_stem_xmp = sandbox_dir.join("sample.xmp");

    // Copy fixture to sandbox
    fs::copy(&cr2_src, &sandboxed_cr2).expect("Copy CR2 to sandbox");

    // Create synthetic Darktable XMP sidecar with 4-star rating and title
    let xmp_content = r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:xmp="http://ns.adobe.com/xap/1.0/"
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    xmp:Rating="4">
   <dc:title>
    <rdf:Alt>
     <rdf:li xml:lang="x-default">Archival Test Capture</rdf:li>
    </rdf:Alt>
   </dc:title>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;
    fs::write(&sandboxed_xmp, xmp_content).expect("Write synthetic Darktable XMP");

    assert!(sandboxed_cr2.exists());
    assert!(sandboxed_xmp.exists());

    // Execute destructive conversion with --delete-original
    let options = DngConvertOptions {
        output_path: Some(target_dng.clone()),
        delete_original: true,
        verify: true,
        quiet: true,
    };

    let report = convert_raw_to_dng(&sandboxed_cr2, &options).expect("Destructive conversion must succeed");

    // Assertions:
    // 1. Target DNG exists and passed verification
    assert!(target_dng.exists(), "Target DNG must exist on disk");
    assert!(report.verification_passed, "Verification must pass");
    assert!(report.deleted_original, "Report must reflect that original was deleted");

    // 2. Source CR2 is safely deleted
    assert!(
        !sandboxed_cr2.exists(),
        "Source CR2 must be safely deleted after verified conversion"
    );

    // 3. Sidecars were migrated to preserve Darktable & Lightroom continuity
    assert!(
        expected_dng_xmp.exists() || expected_stem_xmp.exists(),
        "Sidecar must be migrated to sample.dng.xmp or sample.xmp"
    );

    // 4. Reading XMP for the new DNG preserves the 4-star rating and title
    let dng_xmp = read_xmp_file(&target_dng).expect("Read XMP for new DNG");
    assert_eq!(dng_xmp.rating, Some(4), "4-star rating must be preserved across migration");
    assert_eq!(
        dng_xmp.title.as_deref(),
        Some("Archival Test Capture"),
        "Title must be preserved across migration"
    );

    // Cleanup
    let _ = fs::remove_file(&target_dng);
    let _ = fs::remove_file(&expected_dng_xmp);
    let _ = fs::remove_file(&expected_stem_xmp);
    let _ = fs::remove_file(&sandboxed_xmp);
    let _ = fs::remove_dir(&sandbox_dir);
}

#[test]
fn test_dng_failure_safety_negative_case() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cr2_src = manifest_dir.join("tests/fixtures/_MG_2651.CR2");

    if !cr2_src.exists() {
        return;
    }

    let sandbox_dir = std::env::temp_dir().join("firstproof_dng_safety_test");
    let _ = fs::create_dir_all(&sandbox_dir);
    let sandboxed_cr2 = sandbox_dir.join("safety_sample.CR2");
    fs::copy(&cr2_src, &sandboxed_cr2).expect("Copy CR2 to safety test");

    // Pass an invalid / unwriteable output destination to trigger a failure
    let invalid_out = PathBuf::from("/proc/forbidden/invalid.dng");
    let options = DngConvertOptions {
        output_path: Some(invalid_out),
        delete_original: true,
        verify: true,
        quiet: true,
    };

    let result = convert_raw_to_dng(&sandboxed_cr2, &options);
    assert!(result.is_err(), "Conversion to invalid path must return Err");

    // The original file MUST NOT be deleted
    assert!(
        sandboxed_cr2.exists(),
        "Source RAW must NEVER be deleted when conversion or verification fails"
    );

    let _ = fs::remove_file(&sandboxed_cr2);
    let _ = fs::remove_dir(&sandbox_dir);
}
