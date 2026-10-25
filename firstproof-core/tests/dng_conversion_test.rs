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

    // 5. Verify embedded preview brightness / gamma encoding
    let preview_bytes = firstproof_core::raw::extract_preview_jpeg(&dng_out).expect("Extract embedded DNG preview");
    let preview_img = image::load_from_memory(&preview_bytes).expect("Decode embedded preview JPEG");
    let rgb = preview_img.to_rgb8();
    let sum_luma: u64 = rgb.pixels().map(|p| (p[0] as u64 + p[1] as u64 + p[2] as u64) / 3).sum();
    let avg_luma = sum_luma / (rgb.width() as u64 * rgb.height() as u64);
    println!("DNG embedded preview avg luma: {}", avg_luma);
    assert!(avg_luma >= 40, "Embedded preview must be sRGB gamma-encoded, not crushed linear values (avg luma: {})", avg_luma);

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

#[test]
fn test_dng_convert_on_library_ingest() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cr2_src = manifest_dir.join("tests/fixtures/_MG_2651.CR2");

    if !cr2_src.exists() {
        return;
    }

    let sandbox = std::env::temp_dir().join("firstproof_dng_ingest_test");
    let src_dir = sandbox.join("source_card");
    let dst_dir = sandbox.join("main_library");
    let _ = fs::remove_dir_all(&sandbox);
    let _ = fs::create_dir_all(&src_dir);
    let _ = fs::create_dir_all(&dst_dir);

    let raw_file = src_dir.join("_MG_2651.CR2");
    fs::copy(&cr2_src, &raw_file).expect("Copy test fixture");

    // Write source sidecar with 5-star rating
    let xmp_file = src_dir.join("_MG_2651.xmp");
    fs::write(
        &xmp_file,
        b"<x:xmpmeta xmlns:x=\"adobe:ns:meta/\"><rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"><rdf:Description xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\"><xmp:Rating>5</xmp:Rating><dc:title><rdf:Alt><rdf:li xml:lang=\"x-default\">Ingest Horizon</rdf:li></rdf:Alt></dc:title></rdf:Description></rdf:RDF></x:xmpmeta>",
    )
    .unwrap();

    let src_catalog = firstproof_core::db::Database::resolve_catalog_path(&src_dir, true);
    let src_db = firstproof_core::db::Database::open(&src_catalog).unwrap();
    src_db.index_folder(&src_dir).unwrap();

    let options = firstproof_core::db::ImportOptions {
        move_files: false,
        consolidate_sidecars: true,
        deduplicate_raw_over_jpeg: true,
        skip_duplicates: true,
        verify_checksum: true,
        subfolder_name: Some("Archival Roll".to_string()),
        convert_to_dng: true,
    };

    let report = src_db
        .import_to_library(
            &[src_dir.to_string_lossy().to_string()],
            &dst_dir,
            &options,
            None,
        )
        .expect("Import with DNG conversion must succeed");

    assert_eq!(report.imported_count, 1);
    assert_eq!(report.dng_converted_count, 1, "DNG converted count must be 1");
    assert_eq!(report.error_count, 0);

    let roll_folder = dst_dir.join("Archival Roll");
    let target_dng = roll_folder.join("_MG_2651.dng");
    assert!(target_dng.exists(), "Target DNG must exist in destination subfolder");

    let target_xmp = roll_folder.join("_MG_2651.xmp");
    assert!(target_xmp.exists(), "Consolidated XMP sidecar must exist in destination");

    let parsed_xmp = read_xmp_file(&target_dng).expect("Read XMP for ingested DNG");
    assert_eq!(parsed_xmp.rating, Some(5), "Rating must be preserved");
    assert_eq!(parsed_xmp.title.as_deref(), Some("Ingest Horizon"), "Title must be preserved");

    // Destination catalog verification
    let dst_catalog = firstproof_core::db::Database::resolve_catalog_path(&dst_dir, false);
    let dst_db = firstproof_core::db::Database::open(&dst_catalog).unwrap();
    let all_photos = dst_db.get_all_photos().unwrap();
    assert_eq!(all_photos.len(), 1);
    assert!(all_photos[0].file_path.ends_with("_MG_2651.dng"), "Catalog record must point to .dng file");

    let _ = fs::remove_dir_all(&sandbox);
}

#[test]
fn test_dng_batch_convert_catalog_photos() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cr2_src = manifest_dir.join("tests/fixtures/_MG_2651.CR2");

    if !cr2_src.exists() {
        return;
    }

    let sandbox = std::env::temp_dir().join("firstproof_dng_batch_test");
    let _ = fs::remove_dir_all(&sandbox);
    let _ = fs::create_dir_all(&sandbox);

    let raw_file = sandbox.join("batch_01.CR2");
    fs::copy(&cr2_src, &raw_file).expect("Copy test fixture");

    let jpg_dummy = sandbox.join("raster_dummy.jpg");
    fs::write(&jpg_dummy, b"Fake JPEG dummy file").unwrap();

    let catalog_path = firstproof_core::db::Database::resolve_catalog_path(&sandbox, false);
    let db = firstproof_core::db::Database::open(&catalog_path).unwrap();
    db.index_folder(&sandbox).unwrap();

    // Set a custom 4-star rating on the RAW in catalog
    db.update_rating_status(&raw_file.to_string_lossy(), Some(4), "pick").unwrap();

    let photo_paths = vec![
        raw_file.to_string_lossy().to_string(),
        jpg_dummy.to_string_lossy().to_string(),
    ];

    let batch_report = firstproof_core::db::Database::batch_convert_catalog_to_dng(
        &sandbox,
        &photo_paths,
        true, // delete original RAW after verified read-back
        None,
    )
    .expect("Batch conversion must succeed");

    assert_eq!(batch_report.total_requested, 2);
    assert_eq!(batch_report.converted_count, 1);
    assert_eq!(batch_report.skipped_count, 1, "Raster dummy must be safely skipped");
    assert_eq!(batch_report.failed_count, 0);
    assert_eq!(
        batch_report.space_reclaimed_bytes,
        batch_report.total_source_bytes - batch_report.total_dng_bytes,
        "Net space reclaimed must equal size difference when new DNG is written"
    );

    let target_dng = sandbox.join("batch_01.dng");
    assert!(target_dng.exists(), "Target batch_01.dng must exist on disk");
    assert!(!raw_file.exists(), "Source batch_01.CR2 must be safely deleted");

    // Catalog verification: in-place update
    let updated_rec = db.get_photo(&target_dng.to_string_lossy()).unwrap().expect("Record for DNG must exist in catalog");
    assert_eq!(updated_rec.rating, Some(4), "Rating must be preserved in catalog SQLite");
    assert_eq!(updated_rec.status, "pick", "Status must be preserved in catalog SQLite");
    assert!(updated_rec.file_size > 0);

    let _ = fs::remove_dir_all(&sandbox);
}

#[test]
fn test_dng_batch_convert_catalog_photos_keep_original() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cr2_src = manifest_dir.join("tests/fixtures/_MG_2651.CR2");

    if !cr2_src.exists() {
        return;
    }

    let sandbox = std::env::temp_dir().join("firstproof_dng_batch_keep_original_test");
    let _ = fs::remove_dir_all(&sandbox);
    let _ = fs::create_dir_all(&sandbox);

    let raw_file = sandbox.join("batch_keep.CR2");
    fs::copy(&cr2_src, &raw_file).expect("Copy test fixture");

    let catalog_path = firstproof_core::db::Database::resolve_catalog_path(&sandbox, false);
    let db = firstproof_core::db::Database::open(&catalog_path).unwrap();
    db.index_folder(&sandbox).unwrap();

    // Set a custom 5-star rating on the RAW in catalog
    db.update_rating_status(&raw_file.to_string_lossy(), Some(5), "pick").unwrap();

    let photo_paths = vec![raw_file.to_string_lossy().to_string()];

    let batch_report = firstproof_core::db::Database::batch_convert_catalog_to_dng(
        &sandbox,
        &photo_paths,
        false, // do NOT delete original RAW
        None,
    )
    .expect("Batch conversion must succeed");

    assert_eq!(batch_report.total_requested, 1);
    assert_eq!(batch_report.converted_count, 1);
    assert_eq!(batch_report.failed_count, 0);

    let target_dng = sandbox.join("batch_keep.dng");
    assert!(raw_file.exists(), "Source batch_keep.CR2 must remain on disk");
    assert!(target_dng.exists(), "Target batch_keep.dng must exist on disk");

    // Catalog verification: BOTH records must exist in SQLite catalog
    let all_photos = db.get_all_photos().unwrap();
    assert_eq!(all_photos.len(), 2, "Catalog must retain both original RAW and new DNG");

    let original_rec = db
        .get_photo(&raw_file.to_string_lossy())
        .unwrap()
        .expect("Record for original RAW must remain in catalog");
    assert_eq!(original_rec.rating, Some(5));
    assert_eq!(original_rec.status, "pick");

    let dng_rec = db
        .get_photo(&target_dng.to_string_lossy())
        .unwrap()
        .expect("Record for DNG must be inserted into catalog");
    assert_eq!(dng_rec.rating, Some(5));
    assert_eq!(dng_rec.status, "pick");
    assert!(dng_rec.file_size > 0);

    let _ = fs::remove_dir_all(&sandbox);
}

#[test]
fn test_dng_collision_reconciles_existing_dng_and_reclaims_space() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cr2_src = manifest_dir.join("tests/fixtures/_MG_2651.CR2");

    if !cr2_src.exists() {
        return;
    }

    let sandbox = std::env::temp_dir().join("firstproof_dng_reconcile_test");
    let _ = fs::remove_dir_all(&sandbox);
    let _ = fs::create_dir_all(&sandbox);

    let raw_file = sandbox.join("coexisting.CR2");
    fs::copy(&cr2_src, &raw_file).expect("Copy test fixture");
    let raw_size = fs::metadata(&raw_file).unwrap().len();

    // 1. Provide pre-converted coexisting.dng alongside coexisting.CR2 to simulate co-existing library
    let target_dng = sandbox.join("coexisting.dng");
    let dng_fixture = manifest_dir.join("tests/fixtures/_MG_2651.dng");
    if dng_fixture.exists() {
        fs::copy(&dng_fixture, &target_dng).expect("Copy DNG fixture");
    } else {
        let init_opts = DngConvertOptions {
            output_path: None,
            delete_original: false,
            verify: true,
            quiet: true,
        };
        let initial_rep = convert_raw_to_dng(&raw_file, &init_opts).expect("Initial DNG generation");
        assert!(!initial_rep.already_existed, "First conversion creates new DNG");
    }
    assert!(target_dng.exists());

    // 2. Index both files into catalog.db
    let catalog_path = firstproof_core::db::Database::resolve_catalog_path(&sandbox, false);
    let db = firstproof_core::db::Database::open(&catalog_path).unwrap();
    db.index_folder(&sandbox).unwrap();

    // The user performed culling/rating on the CR2: rating 5, status "Pick"
    db.update_rating_status(&raw_file.to_string_lossy(), Some(5), "Pick").unwrap();

    // 3. Batch convert coexisting.CR2 with delete_original: true
    let batch_report = firstproof_core::db::Database::batch_convert_catalog_to_dng(
        &sandbox,
        &[raw_file.to_string_lossy().to_string()],
        true, // delete original RAW
        None,
    )
    .expect("Batch conversion must succeed");

    assert_eq!(batch_report.converted_count, 1);
    assert_eq!(batch_report.reconciled_existing_count, 1, "Existing DNG must be identified and reconciled");
    assert_eq!(batch_report.space_reclaimed_bytes, raw_size, "Original RAW size must be reported as reclaimed space");
    assert!(batch_report.reports[0].already_existed, "Report must indicate DNG was already present");
    assert!(batch_report.reports[0].deleted_original, "Source CR2 must be marked as deleted");

    // File assertions
    assert!(target_dng.exists(), "Reconciled DNG must still exist on disk");
    assert!(!raw_file.exists(), "Duplicate source CR2 must be safely deleted from disk");

    // Catalog assertions: merged record with winning rating and pick
    let dng_rec = db.get_photo(&target_dng.to_string_lossy()).unwrap().expect("DNG record must exist in catalog");
    assert_eq!(dng_rec.rating, Some(5), "Winning 5-star rating from CR2 must be reconciled into DNG record");
    assert_eq!(dng_rec.status, "Pick", "Winning Pick status from CR2 must be reconciled into DNG record");

    let old_rec = db.get_photo(&raw_file.to_string_lossy()).unwrap();
    assert!(old_rec.is_none(), "Obsolete CR2 catalog row must be removed to prevent UNIQUE constraint collisions");

    let _ = fs::remove_dir_all(&sandbox);
}

#[test]
fn test_dng_collision_unrelated_disambiguation() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cr2_src = manifest_dir.join("tests/fixtures/_MG_2651.CR2");

    if !cr2_src.exists() {
        return;
    }

    let sandbox = std::env::temp_dir().join("firstproof_dng_disambig_test");
    let _ = fs::remove_dir_all(&sandbox);
    let _ = fs::create_dir_all(&sandbox);

    let raw_file = sandbox.join("rollover.CR2");
    fs::copy(&cr2_src, &raw_file).expect("Copy test fixture");

    // Create an unrelated rollover.dng file with different content
    let existing_unrelated_dng = sandbox.join("rollover.dng");
    fs::write(&existing_unrelated_dng, b"Different capture dummy payload").unwrap();

    let opts = DngConvertOptions {
        output_path: None, // automatic naming
        delete_original: false,
        verify: true,
        quiet: true,
    };

    let rep = convert_raw_to_dng(&raw_file, &opts).expect("Conversion must succeed with disambiguated name");
    assert_eq!(rep.dng_path, sandbox.join("rollover_1.dng"), "Must disambiguate to rollover_1.dng");
    assert!(!rep.already_existed);

    // Assert existing file was NOT overwritten
    let existing_content = fs::read(&existing_unrelated_dng).unwrap();
    assert_eq!(existing_content, b"Different capture dummy payload", "Existing unrelated DNG must not be clobbered");
    assert!(sandbox.join("rollover_1.dng").exists(), "Disambiguated DNG must be written");

    let _ = fs::remove_dir_all(&sandbox);
}

#[test]
fn test_dng_conversion_preserves_rotation_and_propagates_sidecars() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cr2_src = manifest_dir.join("tests/fixtures/_MG_2651.CR2");
    if !cr2_src.exists() {
        return;
    }

    let sandbox = std::env::temp_dir().join("firstproof_dng_rotation_test");
    let _ = fs::remove_dir_all(&sandbox);
    let _ = fs::create_dir_all(&sandbox);

    let raw_file = sandbox.join("rotated_test.CR2");
    fs::copy(&cr2_src, &raw_file).expect("Copy test fixture");

    // Write a Darktable-style sidecar with 270 degree rotation
    let cr2_xmp = sandbox.join("rotated_test.CR2.xmp");
    let xmp_xml = r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about="" xmlns:pm="https://firstproof.photo/ns/pm/1.0/">
   <pm:rotation>270</pm:rotation>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;
    fs::write(&cr2_xmp, xmp_xml).expect("Write test XMP");

    let opts = DngConvertOptions {
        output_path: None,
        delete_original: false, // non-destructive conversion
        verify: true,
        quiet: true,
    };

    let rep = convert_raw_to_dng(&raw_file, &opts).expect("Conversion must succeed");
    assert!(rep.dng_path.exists(), "DNG must exist");

    // Assert that sidecars were propagated for the DNG even though original was not deleted
    let dng_xmp = read_xmp_file(&rep.dng_path).expect("DNG must resolve sidecar or embedded edits");
    assert_eq!(dng_xmp.edits.rotation, 270, "Rotation of 270 must be preserved for converted DNG");

    let _ = fs::remove_dir_all(&sandbox);
}

