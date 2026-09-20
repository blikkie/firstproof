//! First Proof Headless Inspection & Parity Audit CLI
//!
//! Provides automated, mathematical verification between the GPU viewport pipeline
//! (simulated Metal shader math on binned raw) and the CPU export pipeline (full demosaic).

use std::env;
use std::path::Path;
use firstproof_core::color::{Color3, agx_tone_map_vivid, ciede2000::linear_srgb_to_lab, ciede2000::ciede2000};
use firstproof_core::export::{
    decode_image, decode_raw_preview_binned_buffer, apply_edits, f16_to_f32, apply_tone_recovery,
};
use firstproof_core::lensfun::LensfunDatabase;
use firstproof_core::raw::extract_metadata;
use firstproof_core::xmp::read_xmp_file;
use serde::Serialize;

#[derive(Serialize)]
struct SamplePointParity {
    label: String,
    uv: [f32; 2],
    display_rgb: [u8; 3],
    export_rgb: [u8; 3],
    delta_rgb: [i16; 3],
    gr_display: f32,
    gr_export: f32,
    delta_e: f32,
}

#[derive(Serialize)]
struct ParityAuditReport {
    photo_path: String,
    lens_model: Option<String>,
    focal_length: Option<f32>,
    distortion_model: String,
    distortion_coeffs: Vec<f32>,
    has_distortion: bool,
    edits_summary: String,
    samples: Vec<SamplePointParity>,
    max_delta_e: f32,
    mean_delta_e: f32,
    pass: bool,
}

fn print_usage() {
    eprintln!("Usage: inspect <command> [args...]");
    eprintln!("Commands:");
    eprintln!("  inspect-photo <path>              Show metadata, XMP edits, and lens calibration");
    eprintln!("  audit-parity <path> [--diff-map <out.png>]  Run full dual-engine parity audit");
    eprintln!("  json-parity <path>                Run parity audit and output JSON report for agents");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];
    match command.as_str() {
        "inspect-photo" => {
            if args.len() < 3 {
                eprintln!("Error: missing photo path");
                std::process::exit(1);
            }
            cmd_inspect_photo(&args[2]);
        }
        "audit-parity" => {
            if args.len() < 3 {
                eprintln!("Error: missing photo path");
                std::process::exit(1);
            }
            let diff_map_path = if args.len() >= 5 && args[3] == "--diff-map" {
                Some(args[4].as_str())
            } else {
                None
            };
            cmd_audit_parity(&args[2], diff_map_path, false);
        }
        "audit-synthetic" => {
            let json_mode = args.len() >= 3 && args[2] == "--json";
            cmd_audit_synthetic(json_mode);
        }
        "compare-images" => {
            if args.len() < 4 {
                eprintln!("Error: missing image paths. Usage: inspect compare-images <img1> <img2>");
                std::process::exit(1);
            }
            cmd_compare_images(&args[2], &args[3]);
        }
        "json-parity" => {
            if args.len() < 3 {
                eprintln!("Error: missing photo path");
                std::process::exit(1);
            }
            cmd_audit_parity(&args[2], None, true);
        }
        "export-photo" => {
            if args.len() < 4 {
                eprintln!("Usage: inspect export-photo <input> <output>");
                std::process::exit(1);
            }
            let edits = read_xmp_file(Path::new(&args[2])).map(|x| x.edits).unwrap_or_default();
            firstproof_core::export::export_photo_file(
                &args[2],
                &args[3],
                &edits,
                None,
                90,
                None,
                firstproof_core::ExportColorSpace::Srgb,
                firstproof_core::ExportRenderingIntent::Perceptual,
                false,
            ).unwrap();
            println!("Exported successfully to {}", args[3]);
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn cmd_inspect_photo(path_str: &str) {
    let path = Path::new(path_str);
    if !path.exists() {
        eprintln!("File not found: {:?}", path);
        std::process::exit(1);
    }

    println!("============================================================");
    println!("🔍 First Proof Photo Inspector: {:?}", path.file_name().unwrap());
    println!("============================================================");

    // 1. Metadata
    let meta = extract_metadata(path).ok();
    if let Some(ref m) = meta {
        println!("📷 Camera: {} {}", m.camera_make.as_deref().unwrap_or("Unknown"), m.camera_model.as_deref().unwrap_or("Unknown"));
        println!("🔭 Lens:   {}", m.lens_model.as_deref().unwrap_or("(Unknown)"));
        println!("📏 Focal:  {:.1}mm, ISO: {}, Shutter: {}, Aperture: f/{:.1}",
            m.focal_length.unwrap_or(0.0), m.iso.unwrap_or(0), m.shutter_speed.as_deref().unwrap_or("-"), m.aperture.unwrap_or(0.0));
    } else {
        println!("⚠️  Could not extract EXIF metadata");
    }

    if let Ok(frame) = firstproof_core::raw_decoder::RawDecoder::decode(&firstproof_core::raw_decoder::RawlerDecoder, path) {
        use firstproof_core::raw_decoder::RawSensorFrameExt;
        println!("\n🖼️  Dimensions: full={}x{}, active={}x{}, origin=({}, {})", frame.full_width, frame.full_height, frame.active_width, frame.active_height, frame.active_x, frame.active_y);
        println!("🧩 CFA Pattern: {:?}, Black: {:?}, White: {:?}, As-Shot WB: {:?}", frame.cfa_pattern, frame.black_levels, frame.white_levels, frame.as_shot_wb());
        println!("🎨 Frame color_matrix: {:?}", frame.color_matrix);
        println!("🎨 Frame color_matrix_tungsten: {:?}", frame.color_matrix_tungsten);
        let m_core = frame.sensor_to_rec2020_at(5500.0);
        println!("🎨 Core sensor_to_rec2020_at(5500K): {:?}", m_core.0);
        let m_types = frame.compute_sensor_to_rec2020();
        println!("🎨 Types compute_sensor_to_rec2020(): {:?}", m_types);
    }

    // 2. Lensfun Lookup
    let ldb = LensfunDatabase::new();
    let lens_coeffs = if let Some(ref m) = meta {
        let make = m.camera_make.as_deref().unwrap_or("");
        let model = m.camera_model.as_deref().unwrap_or("");
        let lens = m.lens_model.as_deref().unwrap_or("");
        let focal = m.focal_length.unwrap_or(0.0);
        let coeffs = ldb.resolve_coefficients(make, model, lens, focal, m.aperture);
        if coeffs.has_distortion || coeffs.has_tca || coeffs.has_vignetting {
            Some(coeffs)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(ref c) = lens_coeffs {
        println!("\n🔍 Lensfun Calibration:");
        println!("   Model: {}", c.distortion_model);
        println!("   Distortion Coeffs: {:?}", c.distortion);
        println!("   Has Distortion: {}", c.has_distortion);
    } else {
        println!("\n⚠️  No Lensfun calibration match found");
    }

    // 3. XMP Edits
    match read_xmp_file(path) {
        Ok(xmp) => {
            let e = &xmp.edits;
            println!("\n🎨 Active XMP Edits:");
            println!("   Temp: {:.0}K, Tint: {:.0}", e.temperature, e.tint);
            println!("   Exposure: {:+.2} EV, Contrast: {:+.0}", e.exposure, e.contrast);
            println!("   Highlights: {:+.1}, Shadows: {:+.1}, Whites: {:+.1}, Blacks: {:+.1}",
                e.highlights, e.shadows, e.whites, e.blacks);
            println!("   Saturation: {:+.1}, Vibrance: {:+.1}", e.saturation, e.vibrance);
            println!("   Crop: [left={:.3}, top={:.3}, right={:.3}, bottom={:.3}], Rotation: {:.1}°",
                e.crop_left, e.crop_top, e.crop_right, e.crop_bottom, e.rotation);
            println!("   Tone Profile: {:?}", e.tone_profile);
        }
        Err(e) => {
            println!("\nℹ️  No XMP sidecar found ({})", e);
        }
    }
}

fn cmd_audit_parity(path_str: &str, _diff_map: Option<&str>, json_mode: bool) {
    let path = Path::new(path_str);
    if !path.exists() {
        eprintln!("File not found: {:?}", path);
        std::process::exit(1);
    }

    let meta = extract_metadata(path).ok();
    let ldb = LensfunDatabase::new();
    let lens_coeffs = if let Some(ref m) = meta {
        let make = m.camera_make.as_deref().unwrap_or("");
        let model = m.camera_model.as_deref().unwrap_or("");
        let lens = m.lens_model.as_deref().unwrap_or("");
        let focal = m.focal_length.unwrap_or(0.0);
        let coeffs = ldb.resolve_coefficients(make, model, lens, focal, m.aperture);
        if coeffs.has_distortion || coeffs.has_tca || coeffs.has_vignetting {
            Some(coeffs)
        } else {
            None
        }
    } else {
        None
    };

    let (dist_model, dist_coeffs, has_dist) = if let Some(ref c) = lens_coeffs {
        (c.distortion_model.clone(), c.distortion.to_vec(), c.has_distortion)
    } else {
        ("none".to_string(), vec![], false)
    };

    let edits = read_xmp_file(path).map(|x| x.edits).unwrap_or_default();

    // 1. Decode Binned (Display Sim)
    let (bw, bh, binned_bytes) = match decode_raw_preview_binned_buffer(path) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Failed to decode binned preview buffer: {}", e);
            std::process::exit(1);
        }
    };

    // 2. Decode Full (Export)
    let mut export_img = match decode_image(path) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Failed to decode full RAW image: {}", e);
            std::process::exit(1);
        }
    };

    // Apply edits to export image
    apply_edits(&mut export_img, &edits);

    // Calculate simulated Metal shader parameters
    let temp_norm = (edits.temperature / 5500.0).clamp(0.25, 4.0);
    let mut r_scale = temp_norm.powf(0.75);
    let mut b_scale = (1.0 / temp_norm).powf(0.75);
    let tint_factor = edits.tint * 0.002;
    let g_scale = (1.0 - tint_factor).max(0.01);
    r_scale = (r_scale + tint_factor * 0.5).max(0.01);
    b_scale = (b_scale + tint_factor * 0.5).max(0.01);
    let exposure_gain = 2.0f32.powf(edits.exposure);
    let contrast_gain = if edits.contrast >= 0.0 {
        1.0 + (edits.contrast / 100.0)
    } else {
        1.0 + (edits.contrast / 100.0) * 0.60
    };

    let test_points = [
        ("Center", 0.50f32, 0.50f32),
        ("Top-Left Corner", 0.15f32, 0.15f32),
        ("Top-Right Corner", 0.85f32, 0.15f32),
        ("Bottom-Left Corner", 0.15f32, 0.85f32),
        ("Bottom-Right Corner", 0.85f32, 0.85f32),
        ("Mid-Top Edge", 0.50f32, 0.15f32),
        ("Mid-Bottom Edge", 0.50f32, 0.85f32),
        ("Subject / Face Focus", 0.35f32, 0.55f32),
        ("Background Wall", 0.60f32, 0.15f32),
        ("Seat / Foreground", 0.15f32, 0.30f32),
    ];

    let mut samples = Vec::new();
    let mut total_delta_e = 0.0f32;
    let mut max_delta_e = 0.0f32;

    for (label, rx, ry) in test_points {
        // Sample binned:
        let bx = ((bw as f32) * rx).clamp(0.0, (bw - 1) as f32) as usize;
        let by = ((bh as f32) * ry).clamp(0.0, (bh - 1) as f32) as usize;
        let b_idx = (by * (bw as usize) + bx) * 8; // 4 half-floats = 8 bytes
        let r_f16 = u16::from_ne_bytes([binned_bytes[b_idx], binned_bytes[b_idx + 1]]);
        let g_f16 = u16::from_ne_bytes([binned_bytes[b_idx + 2], binned_bytes[b_idx + 3]]);
        let b_f16 = u16::from_ne_bytes([binned_bytes[b_idx + 4], binned_bytes[b_idx + 5]]);
        let mut r_binned = f16_to_f32(r_f16);
        let mut g_binned = f16_to_f32(g_f16);
        let mut b_binned = f16_to_f32(b_f16);

        // Apply shader math
        r_binned *= r_scale * exposure_gain;
        g_binned *= g_scale * exposure_gain;
        b_binned *= b_scale * exposure_gain;
        let (r_rec, g_rec, b_rec) = apply_tone_recovery(
            r_binned, g_binned, b_binned,
            edits.highlights, edits.shadows, edits.whites, edits.blacks,
        );
        let binned_tone_mapped = agx_tone_map_vivid(Color3::new(r_rec, g_rec, b_rec), contrast_gain);
        let disp_r = binned_tone_mapped.r().clamp(0.0, 1.0);
        let disp_g = binned_tone_mapped.g().clamp(0.0, 1.0);
        let disp_b = binned_tone_mapped.b().clamp(0.0, 1.0);
        let disp_8bit = [
            (disp_r * 255.0).round() as u8,
            (disp_g * 255.0).round() as u8,
            (disp_b * 255.0).round() as u8,
        ];

        // Sample export:
        let ex = ((export_img.width as f32) * rx).clamp(0.0, (export_img.width - 1) as f32) as usize;
        let ey = ((export_img.height as f32) * ry).clamp(0.0, (export_img.height - 1) as f32) as usize;
        let e_idx = (ey * export_img.width + ex) * 3;
        let exp_r = export_img.data[e_idx].clamp(0.0, 1.0);
        let exp_g = export_img.data[e_idx + 1].clamp(0.0, 1.0);
        let exp_b = export_img.data[e_idx + 2].clamp(0.0, 1.0);
        let exp_8bit = [
            (exp_r * 255.0).round() as u8,
            (exp_g * 255.0).round() as u8,
            (exp_b * 255.0).round() as u8,
        ];

        let lab_disp = linear_srgb_to_lab(Color3::new(disp_r, disp_g, disp_b));
        let lab_exp = linear_srgb_to_lab(Color3::new(exp_r, exp_g, exp_b));
        let de = ciede2000(lab_disp, lab_exp);
        total_delta_e += de;
        if de > max_delta_e {
            max_delta_e = de;
        }

        let gr_disp = if disp_8bit[0] > 0 { disp_8bit[1] as f32 / disp_8bit[0] as f32 } else { 0.0 };
        let gr_exp = if exp_8bit[0] > 0 { exp_8bit[1] as f32 / exp_8bit[0] as f32 } else { 0.0 };

        samples.push(SamplePointParity {
            label: label.to_string(),
            uv: [rx, ry],
            display_rgb: disp_8bit,
            export_rgb: exp_8bit,
            delta_rgb: [
                exp_8bit[0] as i16 - disp_8bit[0] as i16,
                exp_8bit[1] as i16 - disp_8bit[1] as i16,
                exp_8bit[2] as i16 - disp_8bit[2] as i16,
            ],
            gr_display: gr_disp,
            gr_export: gr_exp,
            delta_e: de,
        });
    }

    let mean_de = if !samples.is_empty() { total_delta_e / (samples.len() as f32) } else { 0.0 };
    let pass = mean_de < 2.0 && max_delta_e < 5.0;

    let report = ParityAuditReport {
        photo_path: path_str.to_string(),
        lens_model: meta.as_ref().and_then(|m| m.lens_model.clone()),
        focal_length: meta.as_ref().and_then(|m| m.focal_length),
        distortion_model: dist_model,
        distortion_coeffs: dist_coeffs,
        has_distortion: has_dist,
        edits_summary: format!("Temp={}K, Tint={}, Exp={:+.2}, Cont={:+.0}",
            edits.temperature, edits.tint, edits.exposure, edits.contrast),
        samples,
        max_delta_e,
        mean_delta_e: mean_de,
        pass,
    };

    if json_mode {
        let json = serde_json::to_string_pretty(&report).unwrap();
        println!("{}", json);
    } else {
        println!("================================================================================");
        println!("🔬 First Proof Dual-Engine Parity Audit: {:?}", path.file_name().unwrap());
        println!("Edits: {}", report.edits_summary);
        println!("Lens Calibration: {} (coeffs: {:?})", report.distortion_model, report.distortion_coeffs);
        println!("--------------------------------------------------------------------------------");
        println!("{:<22} {:<15} {:<15} {:<12} {:<10} {:<8}",
            "Sample Region", "Display (Binned)", "Export (Full)", "Delta [R,G,B]", "G/R Delta", "ΔE₀₀");
        println!("--------------------------------------------------------------------------------");
        for s in &report.samples {
            println!("{:<22} {:<15?} {:<15?} {:<12?} {:+.3}      {:.2}",
                s.label, s.display_rgb, s.export_rgb, s.delta_rgb, s.gr_export - s.gr_display, s.delta_e);
        }
        println!("--------------------------------------------------------------------------------");
        println!("Summary: Mean ΔE₀₀ = {:.2}, Max ΔE₀₀ = {:.2} -> Result: {}",
            report.mean_delta_e, report.max_delta_e, if report.pass { "✅ PASSED" } else { "❌ FAILED" });
        println!("================================================================================");
    }
}

fn cmd_compare_images(path1_str: &str, path2_str: &str) {
    let img1 = image::open(path1_str).expect("Failed to open image 1").to_rgb8();
    let img2 = image::open(path2_str).expect("Failed to open image 2").to_rgb8();
    println!("Comparing:\n  Img1 (Viewport): {} ({}x{})\n  Img2 (Export):   {} ({}x{})",
        path1_str, img1.width(), img1.height(), path2_str, img2.width(), img2.height());
    
    let target_w = img1.width();
    let target_h = img1.height();
    let img2_resized = if img2.width() != target_w || img2.height() != target_h {
        image::imageops::resize(&img2, target_w, target_h, image::imageops::FilterType::Lanczos3)
    } else {
        img2
    };

    let mut total_diff_r = 0u64;
    let mut total_diff_g = 0u64;
    let mut total_diff_b = 0u64;
    let count = (target_w as u64) * (target_h as u64);

    for y in 0..target_h {
        for x in 0..target_w {
            let p1 = img1.get_pixel(x, y);
            let p2 = img2_resized.get_pixel(x, y);
            total_diff_r += (p1.0[0] as i32 - p2.0[0] as i32).abs() as u64;
            total_diff_g += (p1.0[1] as i32 - p2.0[1] as i32).abs() as u64;
            total_diff_b += (p1.0[2] as i32 - p2.0[2] as i32).abs() as u64;
        }
    }

    println!("\nOverall Pixel Deltas across {} pixels:", count);
    println!("  Mean RGB Delta: R={:.2}, G={:.2}, B={:.2} (out of 255)",
        total_diff_r as f64 / count as f64,
        total_diff_g as f64 / count as f64,
        total_diff_b as f64 / count as f64);

    let test_points = [
        ("Center", 0.50f32, 0.50f32),
        ("Girl Cheek", 0.35f32, 0.55f32),
        ("Subway Wall", 0.60f32, 0.15f32),
        ("Metal Handrail", 0.75f32, 0.25f32),
        ("Orange Seat", 0.15f32, 0.30f32),
        ("Top-Right Corner", 0.85f32, 0.15f32),
        ("Bottom-Left Corner", 0.15f32, 0.85f32),
    ];

    println!("\nKey Point Verification:");
    println!("{:<20} {:<15} {:<15} {:<15} {:<15}",
        "Region", "Viewport", "Export", "Delta [R,G,B]", "G/R (VP vs EXP)");
    println!("--------------------------------------------------------------------------------");
    for (label, rx, ry) in test_points {
        let px = ((target_w as f32) * rx).min((target_w - 1) as f32) as u32;
        let py = ((target_h as f32) * ry).min((target_h - 1) as f32) as u32;
        let p1 = img1.get_pixel(px, py);
        let p2 = img2_resized.get_pixel(px, py);
        let gr1 = p1.0[1] as f32 / p1.0[0].max(1) as f32;
        let gr2 = p2.0[1] as f32 / p2.0[0].max(1) as f32;
        println!("{:<20} {:<15?} {:<15?} [{:+3}, {:+3}, {:+3}]     {:.3} vs {:.3}",
            label, p1.0, p2.0,
            p2.0[0] as i16 - p1.0[0] as i16,
            p2.0[1] as i16 - p1.0[1] as i16,
            p2.0[2] as i16 - p1.0[2] as i16,
            gr1, gr2);
    }
}

fn cmd_audit_synthetic(_json_mode: bool) {
    use firstproof_core::synthetic::SyntheticTestChart;
    use firstproof_core::EditSettings;

    println!("================================================================================");
    println!("🧪 First Proof Synthetic Dual-Engine Parity Audit (Macbeth 24 & Neutral Ramp)");
    println!("--------------------------------------------------------------------------------");

    let chart = SyntheticTestChart::new(1200, 800);
    let binned_bytes = chart.to_binned_f16_rgba();
    let bw = chart.width;
    let bh = chart.height;

    let edits = EditSettings {
        temperature: 3800.0,
        tint: 45.0,
        exposure: 0.5,
        contrast: 15.0,
        highlights: -25.0,
        shadows: 20.0,
        ..Default::default()
    };

    let mut export_img = chart.to_decoded_image();
    apply_edits(&mut export_img, &edits);

    let temp_norm = (edits.temperature / 5500.0).clamp(0.25, 4.0);
    let mut r_scale = temp_norm.powf(0.75);
    let mut b_scale = (1.0 / temp_norm).powf(0.75);
    let tint_factor = edits.tint * 0.002;
    let g_scale = (1.0 - tint_factor).max(0.01);
    r_scale = (r_scale + tint_factor * 0.5).max(0.01);
    b_scale = (b_scale + tint_factor * 0.5).max(0.01);
    let exposure_gain = 2.0f32.powf(edits.exposure);
    let contrast_gain = if edits.contrast >= 0.0 {
        1.0 + (edits.contrast / 100.0)
    } else {
        1.0 + (edits.contrast / 100.0) * 0.60
    };

    let mut samples = Vec::new();
    let mut total_de00 = 0.0f32;
    let mut max_de00 = 0.0f32;

    for patch in &chart.sample_points {
        let u = patch.uv[0];
        let v = patch.uv[1];

        let bx = ((bw as f32 * u).round() as usize).clamp(0, bw as usize - 1);
        let by = ((bh as f32 * v).round() as usize).clamp(0, bh as usize - 1);
        let b_idx = (by * bw as usize + bx) * 8;
        let r_f16 = u16::from_ne_bytes([binned_bytes[b_idx], binned_bytes[b_idx + 1]]);
        let g_f16 = u16::from_ne_bytes([binned_bytes[b_idx + 2], binned_bytes[b_idx + 3]]);
        let b_f16 = u16::from_ne_bytes([binned_bytes[b_idx + 4], binned_bytes[b_idx + 5]]);

        let r_binned = f16_to_f32(r_f16) * r_scale * exposure_gain;
        let g_binned = f16_to_f32(g_f16) * g_scale * exposure_gain;
        let b_binned = f16_to_f32(b_f16) * b_scale * exposure_gain;

        let (r_rec, g_rec, b_rec) = apply_tone_recovery(
            r_binned, g_binned, b_binned,
            edits.highlights, edits.shadows, edits.whites, edits.blacks,
        );

        let tone_mapped = agx_tone_map_vivid(Color3::new(r_rec, g_rec, b_rec), contrast_gain);
        let disp_8bit = [
            (tone_mapped.r().clamp(0.0, 1.0) * 255.0).round() as u8,
            (tone_mapped.g().clamp(0.0, 1.0) * 255.0).round() as u8,
            (tone_mapped.b().clamp(0.0, 1.0) * 255.0).round() as u8,
        ];

        let ex = ((export_img.width as f32 * u).round() as usize).clamp(0, export_img.width - 1);
        let ey = ((export_img.height as f32 * v).round() as usize).clamp(0, export_img.height - 1);
        let e_idx = (ey * export_img.width + ex) * 3;

        let exp_8bit = [
            (export_img.data[e_idx].clamp(0.0, 1.0) * 255.0).round() as u8,
            (export_img.data[e_idx + 1].clamp(0.0, 1.0) * 255.0).round() as u8,
            (export_img.data[e_idx + 2].clamp(0.0, 1.0) * 255.0).round() as u8,
        ];

        let lab_disp = linear_srgb_to_lab(Color3::new(
            tone_mapped.r().clamp(0.0, 1.0),
            tone_mapped.g().clamp(0.0, 1.0),
            tone_mapped.b().clamp(0.0, 1.0),
        ));
        let lab_exp = linear_srgb_to_lab(Color3::new(
            export_img.data[e_idx].clamp(0.0, 1.0),
            export_img.data[e_idx + 1].clamp(0.0, 1.0),
            export_img.data[e_idx + 2].clamp(0.0, 1.0),
        ));
        let de = ciede2000(lab_disp, lab_exp);
        total_de00 += de;
        if de > max_de00 {
            max_de00 = de;
        }

        samples.push((patch.name, disp_8bit, exp_8bit, de));
    }

    let mean_de00 = total_de00 / (samples.len() as f32);

    println!("{:<22} {:<15} {:<15} {:<12} {:<8}",
        "Synthetic Patch", "GPU Display", "CPU Export", "Delta [R,G,B]", "ΔE₀₀");
    println!("--------------------------------------------------------------------------------");
    for (name, disp, exp, de) in &samples {
        let delta = [
            exp[0] as i16 - disp[0] as i16,
            exp[1] as i16 - disp[1] as i16,
            exp[2] as i16 - disp[2] as i16,
        ];
        println!("{:<22} {:<15?} {:<15?} {:<12?} {:.3}", name, disp, exp, delta, de);
    }
    println!("--------------------------------------------------------------------------------");
    println!("Summary: Mean ΔE₀₀ = {:.4}, Max ΔE₀₀ = {:.4} -> Result: {}",
        mean_de00, max_de00, if mean_de00 < 0.05 && max_de00 < 0.15 { "✅ PASSED" } else { "❌ FAILED" });
    println!("================================================================================");
}


