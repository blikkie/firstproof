use std::fs;
use std::path::PathBuf;
use firstproof_core::lut::{
    delete_global_lut, discover_global_luts, import_global_lut, inspect_cube_file,
    load_global_or_cube_lut, rename_global_lut,
};

fn create_sample_cube(title: &str, size: usize) -> String {
    let mut s = format!("TITLE \"{}\"\nLUT_3D_SIZE {}\nDOMAIN_MIN 0.0 0.0 0.0\nDOMAIN_MAX 1.0 1.0 1.0\n", title, size);
    for b in 0..size {
        let b_val = b as f32 / (size - 1) as f32;
        for g in 0..size {
            let g_val = g as f32 / (size - 1) as f32;
            for r in 0..size {
                let r_val = r as f32 / (size - 1) as f32;
                s.push_str(&format!("{:.6} {:.6} {:.6}\n", r_val, g_val, b_val));
            }
        }
    }
    s
}

#[test]
fn test_inspect_cube_file_reads_title_and_size() {
    let tmp_dir = std::env::temp_dir().join("firstproof_test_inspect");
    fs::create_dir_all(&tmp_dir).unwrap();
    let file_path = tmp_dir.join("test_portrait.cube");
    let content = create_sample_cube("Warm Portrait 400", 2);
    fs::write(&file_path, content).unwrap();

    let (title, size) = inspect_cube_file(&file_path).expect("Failed to inspect cube file");
    assert_eq!(title, "Warm Portrait 400");
    assert_eq!(size, 2);

    let _ = fs::remove_file(file_path);
    let _ = fs::remove_dir(tmp_dir);
}

#[test]
fn test_global_lut_lifecycle_import_discover_resolve_rename_delete() {
    let tmp_src_dir = std::env::temp_dir().join("firstproof_test_src");
    fs::create_dir_all(&tmp_src_dir).unwrap();
    let src_file = tmp_src_dir.join("Cinematic_Teal_Orange.cube");
    let content = create_sample_cube("Cinematic Teal & Orange", 2);
    fs::write(&src_file, content).unwrap();

    // 1. Import
    let record = import_global_lut(src_file.to_str().unwrap())
        .expect("Failed to import global LUT");
    assert_eq!(record.title, "Cinematic Teal & Orange");
    assert_eq!(record.size, 2);
    assert!(!record.is_builtin);
    assert!(PathBuf::from(&record.file_path).exists());

    // 2. Discover
    let luts = discover_global_luts();
    let found = luts.iter().find(|l| l.title == "Cinematic Teal & Orange");
    assert!(found.is_some(), "Imported LUT should be present in discover_global_luts()");

    // 3. Resolve by title and by filename
    let resolved_by_title = load_global_or_cube_lut("Cinematic Teal & Orange")
        .expect("Should resolve LUT by title");
    assert_eq!(resolved_by_title.title, "Cinematic Teal & Orange");
    assert_eq!(resolved_by_title.size, 2);

    let resolved_by_filename = load_global_or_cube_lut("Cinematic_Teal_Orange.cube")
        .expect("Should resolve LUT by filename");
    assert_eq!(resolved_by_filename.size, 2);

    // 4. Rename
    let renamed = rename_global_lut(&record.id, "Moody Teal & Orange")
        .expect("Failed to rename global LUT");
    assert_eq!(renamed.title, "Moody Teal & Orange");
    assert!(PathBuf::from(&renamed.file_path).exists());

    // Verify resolved by new title
    let resolved_new = load_global_or_cube_lut("Moody Teal & Orange")
        .expect("Should resolve LUT by new title");
    assert_eq!(resolved_new.title, "Moody Teal & Orange");

    // 5. Delete
    let deleted = delete_global_lut(&renamed.id)
        .expect("Failed to delete global LUT");
    assert!(deleted);
    assert!(!PathBuf::from(&renamed.file_path).exists());

    // Cleanup source temp
    let _ = fs::remove_file(src_file);
    let _ = fs::remove_dir(tmp_src_dir);
}

#[test]
fn test_export_pipeline_resolves_global_lut_by_title() {
    let tmp_src_dir = std::env::temp_dir().join("firstproof_test_export_lut");
    fs::create_dir_all(&tmp_src_dir).unwrap();
    let src_file = tmp_src_dir.join("global_mono.cube");
    let content = "TITLE \"Global Monochrome\"\nLUT_3D_SIZE 2\nDOMAIN_MIN 0.0 0.0 0.0\nDOMAIN_MAX 1.0 1.0 1.0\n0.0 0.0 0.0\n0.5 0.5 0.5\n0.5 0.5 0.5\n1.0 1.0 1.0\n0.5 0.5 0.5\n1.0 1.0 1.0\n1.0 1.0 1.0\n1.0 1.0 1.0\n";
    fs::write(&src_file, content).unwrap();

    let record = import_global_lut(src_file.to_str().unwrap()).unwrap();

    // Create a colored test image
    let tmp_in = tmp_src_dir.join("in.jpg");
    let tmp_out = tmp_src_dir.join("out.jpg");
    let img = image::RgbImage::from_fn(30, 30, |_x, _y| image::Rgb([200, 100, 50]));
    img.save(&tmp_in).unwrap();

    let edits = firstproof_core::EditSettings {
        tone_profile: firstproof_core::ColorProfileMode::AgxVivid,
        lut_name: Some("Global Monochrome".to_string()),
        lut_intensity: 1.0,
        ..Default::default()
    };

    firstproof_core::export::export_photo_file(
        tmp_in.to_str().unwrap(),
        tmp_out.to_str().unwrap(),
        &edits,
        None,
        95,
        None,
        firstproof_core::ExportColorSpace::Srgb,
        firstproof_core::ExportRenderingIntent::RelativeColorimetric,
        false,
    ).expect("Export should succeed by resolving LUT title from global library");

    let out_img = image::open(&tmp_out).unwrap().to_rgb8();
    let pixel = out_img.get_pixel(15, 15);
    // Since it's monochrome LUT, R == G == B
    assert_eq!(pixel[0], pixel[1]);
    assert_eq!(pixel[1], pixel[2]);

    // Clean up
    let _ = delete_global_lut(&record.id);
    let _ = fs::remove_file(src_file);
    let _ = fs::remove_file(tmp_in);
    let _ = fs::remove_file(tmp_out);
    let _ = fs::remove_dir(tmp_src_dir);
}

#[test]
fn test_delete_prevents_directory_traversal() {
    let result = delete_global_lut("../../../etc/passwd");
    assert!(result.is_err(), "Must reject deletion outside global LUT directory");
}
