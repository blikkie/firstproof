use std::env;
use std::path::Path;
use image::{RgbImage, Rgb, imageops};
use firstproof_core::raw::extract_preview_jpeg;
use firstproof_core::xmp::read_xmp_file;
use firstproof_core::{ExportColorSpace, ExportRenderingIntent};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("============================================================");
        eprintln!("🔬 First Proof RAW vs Camera JPEG Calibration Diagnostics");
        eprintln!("============================================================");
        eprintln!("Usage: calibrate <raw_path> [options]");
        eprintln!("\nOptions:");
        eprintln!("  --exposure <ev>         Exposure compensation offset (e.g. +0.0, +0.5)");
        eprintln!("  --crop <x,y,w,h>        1:1 pixel crop for detail inspection (e.g. 1400,1500,800,800)");
        eprintln!("  --out-prefix <prefix>   Output filename prefix (default: raw_calibration)");
        std::process::exit(1);
    }

    let raw_path = &args[1];
    let path = Path::new(raw_path);
    if !path.exists() {
        eprintln!("Error: file not found: {}", raw_path);
        std::process::exit(1);
    }

    let mut custom_crop: Option<(u32, u32, u32, u32)> = None;
    let mut exposure_override: Option<f32> = None;
    let mut out_prefix = "raw_calibration".to_string();
    let mut use_xmp = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--crop" if i + 1 < args.len() => {
                let parts: Vec<&str> = args[i + 1].split(',').collect();
                if parts.len() == 4 {
                    if let (Ok(x), Ok(y), Ok(w), Ok(h)) = (
                        parts[0].parse::<u32>(),
                        parts[1].parse::<u32>(),
                        parts[2].parse::<u32>(),
                        parts[3].parse::<u32>(),
                    ) {
                        custom_crop = Some((x, y, w, h));
                    }
                }
                i += 2;
            }
            "--exposure" if i + 1 < args.len() => {
                if let Ok(ev) = args[i + 1].parse::<f32>() {
                    exposure_override = Some(ev);
                }
                i += 2;
            }
            "--out-prefix" if i + 1 < args.len() => {
                out_prefix = args[i + 1].clone();
                i += 2;
            }
            "--use-xmp" => {
                use_xmp = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    println!("============================================================");
    println!("🔬 First Proof RAW vs Camera JPEG Calibration Diagnostics Tool");
    println!("Analyzing: {}", raw_path);
    println!("============================================================\n");

    // 1. Extract Camera Reference JPEG
    println!("📸 1. Extracting Camera Reference JPEG...");
    let preview_bytes = match extract_preview_jpeg(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error extracting embedded preview JPEG: {}", e);
            std::process::exit(1);
        }
    };

    let cam_rgb = match image::load_from_memory(&preview_bytes) {
        Ok(img) => img.to_rgb8(),
        Err(e) => {
            eprintln!("Error decoding embedded JPEG: {}", e);
            std::process::exit(1);
        }
    };
    println!("   Camera Reference JPEG: {}x{}", cam_rgb.width(), cam_rgb.height());

    // 2. Render First Proof Export
    println!("⚙️  2. Rendering First Proof Export (AgX Filmic)...");
    let temp_export_path = std::env::temp_dir().join(format!("fp_cal_{}_{}.jpg", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)));
    let mut edits = if use_xmp {
        println!("   Using edits from sidecar XMP...");
        read_xmp_file(path).map(|x| x.edits).unwrap_or_default()
    } else {
        println!("   Evaluating clean camera defaults (sidecar XMP ignored)...");
        firstproof_core::EditSettings::default()
    };
    if let Some(ev) = exposure_override {
        edits.exposure = ev;
    }

    if let Err(e) = firstproof_core::export::export_photo_file(
        path,
        &temp_export_path,
        &edits,
        None,
        92,
        None,
        ExportColorSpace::Srgb,
        ExportRenderingIntent::Perceptual,
        false,
    ) {
        eprintln!("Error rendering First Proof export: {}", e);
        std::process::exit(1);
    }

    let fp_rgb = match image::open(&temp_export_path) {
        Ok(img) => img.to_rgb8(),
        Err(e) => {
            eprintln!("Error reading rendered export: {}", e);
            let _ = std::fs::remove_file(&temp_export_path);
            std::process::exit(1);
        }
    };
    let _ = std::fs::remove_file(&temp_export_path);
    println!("   First Proof Render:    {}x{}", fp_rgb.width(), fp_rgb.height());

    // 3. Match dimensions for pixel-to-pixel comparison
    let target_w = cam_rgb.width();
    let target_h = cam_rgb.height();
    let fp_matched = if fp_rgb.width() != target_w || fp_rgb.height() != target_h {
        println!("   Resizing First Proof render ({}x{} -> {}x{}) for alignment...",
            fp_rgb.width(), fp_rgb.height(), target_w, target_h);
        imageops::resize(&fp_rgb, target_w, target_h, imageops::FilterType::Lanczos3)
    } else {
        fp_rgb
    };

    // 4. Compute Statistical & Color Science Metrics
    let mut total_sq_err: f64 = 0.0;
    let mut total_abs_diff_r: f64 = 0.0;
    let mut total_abs_diff_g: f64 = 0.0;
    let mut total_abs_diff_b: f64 = 0.0;

    let mut cam_sum_r: f64 = 0.0;
    let mut cam_sum_g: f64 = 0.0;
    let mut cam_sum_b: f64 = 0.0;
    let mut fp_sum_r: f64 = 0.0;
    let mut fp_sum_g: f64 = 0.0;
    let mut fp_sum_b: f64 = 0.0;

    let pixel_count = (target_w * target_h) as f64;

    // Full-frame Difference Heatmap (3.5x amplified)
    let mut diff_img = RgbImage::new(target_w, target_h);

    for y in 0..target_h {
        for x in 0..target_w {
            let cp = cam_rgb.get_pixel(x, y);
            let fp = fp_matched.get_pixel(x, y);

            let cr = cp[0] as f64;
            let cg = cp[1] as f64;
            let cb = cp[2] as f64;

            let fr = fp[0] as f64;
            let fg = fp[1] as f64;
            let fb = fp[2] as f64;

            cam_sum_r += cr;
            cam_sum_g += cg;
            cam_sum_b += cb;

            fp_sum_r += fr;
            fp_sum_g += fg;
            fp_sum_b += fb;

            let dr = (cr - fr).abs();
            let dg = (cg - fg).abs();
            let db = (cb - fb).abs();

            total_abs_diff_r += dr;
            total_abs_diff_g += dg;
            total_abs_diff_b += db;

            let sq = (cr - fr).powi(2) + (cg - fg).powi(2) + (cb - fb).powi(2);
            total_sq_err += sq / 3.0;

            // Amplify difference by 3.5x for heatmap visualization
            let hr = (dr * 3.5).min(255.0) as u8;
            let hg = (dg * 3.5).min(255.0) as u8;
            let hb = (db * 3.5).min(255.0) as u8;
            diff_img.put_pixel(x, y, Rgb([hr, hg, hb]));
        }
    }

    let mse = total_sq_err / pixel_count;
    let psnr = if mse > 0.0 { 10.0 * (255.0 * 255.0 / mse).log10() } else { 99.0 };

    let mean_diff_r = total_abs_diff_r / pixel_count;
    let mean_diff_g = total_abs_diff_g / pixel_count;
    let mean_diff_b = total_abs_diff_b / pixel_count;
    let mean_total_err = (mean_diff_r + mean_diff_g + mean_diff_b) / 3.0;

    let cam_avg_r = cam_sum_r / pixel_count;
    let cam_avg_g = cam_sum_g / pixel_count;
    let cam_avg_b = cam_sum_b / pixel_count;

    let fp_avg_r = fp_sum_r / pixel_count;
    let fp_avg_g = fp_sum_g / pixel_count;
    let fp_avg_b = fp_sum_b / pixel_count;

    println!("\n📊 STATISTICAL METRICS (Full Frame - 17.9M pixels):");
    println!("------------------------------------------------------------");
    println!("  • Mean Squared Error (MSE):  {:.2}  [Baseline: 299.11 | Gate: < 180.00 -> {}]", mse, if mse < 180.0 { "✅ PASS" } else { "❌ FAIL" });
    println!("  • Peak SNR (PSNR):           {:.2} dB [Baseline:  23.37 | Gate: >  25.50 -> {}]", psnr, if psnr > 25.50 { "✅ PASS" } else { "❌ FAIL" });
    println!("  • Mean Channel Errors:        ΔR={:.2} | ΔG={:.2} | ΔB={:.2} (out of 255)", mean_diff_r, mean_diff_g, mean_diff_b);
    println!("  • Mean Total Error:          {:.2}/255 ({:.2}%) [Baseline: 13.56 | Gate: < 9.00 -> {}]", mean_total_err, (mean_total_err / 255.0) * 100.0, if mean_total_err < 9.0 { "✅ PASS" } else { "❌ FAIL" });
    println!("  • Camera Reference Averages: R={:.1}  | G={:.1}  | B={:.1}", cam_avg_r, cam_avg_g, cam_avg_b);
    println!("  • First Proof Averages:      R={:.1}  | G={:.1}  | B={:.1}", fp_avg_r, fp_avg_g, fp_avg_b);
    println!("------------------------------------------------------------");

    // Key Region Spot Audit
    let test_points = [
        ("Girl Cheek / Face", 0.35f32, 0.55f32),
        ("Subway Wall", 0.60f32, 0.15f32),
        ("Center Midtone", 0.50f32, 0.50f32),
        ("Metal Handrail", 0.75f32, 0.25f32),
        ("Orange Seat", 0.15f32, 0.30f32),
        ("Top-Right Shadow", 0.85f32, 0.15f32),
        ("Bottom-Left Bright", 0.15f32, 0.85f32),
    ];

    println!("\n🎯 KEY REGION SPOT AUDIT:");
    println!("{:<22} {:<15} {:<15} {:<15} {:<15}",
        "Region", "Camera Preview", "First Proof", "Delta [R,G,B]", "G/R (Cam vs FP)");
    println!("--------------------------------------------------------------------------------");
    for (label, rx, ry) in test_points {
        let px = ((target_w as f32) * rx).min((target_w - 1) as f32) as u32;
        let py = ((target_h as f32) * ry).min((target_h - 1) as f32) as u32;
        let p_cam = cam_rgb.get_pixel(px, py);
        let p_fp = fp_matched.get_pixel(px, py);
        let gr_cam = p_cam.0[1] as f32 / p_cam.0[0].max(1) as f32;
        let gr_fp = p_fp.0[1] as f32 / p_fp.0[0].max(1) as f32;
        println!("{:<22} {:<15?} {:<15?} [{:+3}, {:+3}, {:+3}]     {:.3} vs {:.3}",
            label, p_cam.0, p_fp.0,
            p_fp.0[0] as i16 - p_cam.0[0] as i16,
            p_fp.0[1] as i16 - p_cam.0[1] as i16,
            p_fp.0[2] as i16 - p_cam.0[2] as i16,
            gr_cam, gr_fp);
    }

    // 5. Generate Full-Frame 3-Panel Diagnostic Image (scaled to 1600px width per panel for fast viewing)
    let panel_w = 1600u32;
    let panel_h = ((target_h as f32) * (panel_w as f32) / (target_w as f32)).round() as u32;
    let cam_scaled = imageops::resize(&cam_rgb, panel_w, panel_h, imageops::FilterType::Lanczos3);
    let fp_scaled = imageops::resize(&fp_matched, panel_w, panel_h, imageops::FilterType::Lanczos3);
    let diff_scaled = imageops::resize(&diff_img, panel_w, panel_h, imageops::FilterType::Lanczos3);

    let mut full_triptych = RgbImage::new(panel_w * 3, panel_h);
    imageops::overlay(&mut full_triptych, &cam_scaled, 0, 0);
    imageops::overlay(&mut full_triptych, &fp_scaled, panel_w as i64, 0);
    imageops::overlay(&mut full_triptych, &diff_scaled, (panel_w * 2) as i64, 0);

    let full_triptych_path = format!("{}_full_triptych.jpg", out_prefix);
    if let Err(e) = full_triptych.save(&full_triptych_path) {
        eprintln!("Error saving full triptych: {}", e);
    } else {
        println!("\n🖼️  Saved Full Triptych ({}x{}): {}",
            full_triptych.width(), full_triptych.height(), full_triptych_path);
        println!("   [Panel 1: Camera Reference | Panel 2: First Proof | Panel 3: 3.5x Amplified Diff Heatmap]");
    }

    // 6. Generate 1:1 Pixel Crop Detail Comparison (Default: Child's Face)
    let (cx, cy, cw, ch) = custom_crop.unwrap_or_else(|| {
        if raw_path.contains("2332") {
            // Child's face region in _MG_2332.CR2
            (1400, 1500, 800, 800)
        } else {
            // Center crop default
            (
                target_w.saturating_sub(800) / 2,
                target_h.saturating_sub(800) / 2,
                800.min(target_w),
                800.min(target_h),
            )
        }
    });

    let safe_cx = cx.min(target_w.saturating_sub(cw));
    let safe_cy = cy.min(target_h.saturating_sub(ch));
    let safe_cw = cw.min(target_w - safe_cx);
    let safe_ch = ch.min(target_h - safe_cy);

    if safe_cw > 0 && safe_ch > 0 {
        let cam_crop = imageops::crop_imm(&cam_rgb, safe_cx, safe_cy, safe_cw, safe_ch).to_image();
        let fp_crop = imageops::crop_imm(&fp_matched, safe_cx, safe_cy, safe_cw, safe_ch).to_image();

        // 1:1 Pixel Difference Heatmap for the crop (4.0x amplified)
        let mut crop_diff = RgbImage::new(safe_cw, safe_ch);
        for y in 0..safe_ch {
            for x in 0..safe_cw {
                let cp = cam_crop.get_pixel(x, y);
                let fp = fp_crop.get_pixel(x, y);
                let dr = ((cp[0] as i16 - fp[0] as i16).abs() as f32 * 4.0).min(255.0) as u8;
                let dg = ((cp[1] as i16 - fp[1] as i16).abs() as f32 * 4.0).min(255.0) as u8;
                let db = ((cp[2] as i16 - fp[2] as i16).abs() as f32 * 4.0).min(255.0) as u8;
                crop_diff.put_pixel(x, y, Rgb([dr, dg, db]));
            }
        }

        let mut crop_triptych = RgbImage::new(safe_cw * 3, safe_ch);
        imageops::overlay(&mut crop_triptych, &cam_crop, 0, 0);
        imageops::overlay(&mut crop_triptych, &fp_crop, safe_cw as i64, 0);
        imageops::overlay(&mut crop_triptych, &crop_diff, (safe_cw * 2) as i64, 0);

        let crop_triptych_path = format!("{}_crop_triptych.jpg", out_prefix);
        if let Err(e) = crop_triptych.save(&crop_triptych_path) {
            eprintln!("Error saving crop triptych: {}", e);
        } else {
            println!("🔍 Saved 1:1 Detail Crop Triptych ({}x{} at x={}, y={}): {}",
                crop_triptych.width(), crop_triptych.height(), safe_cx, safe_cy, crop_triptych_path);
            println!("   [Panel 1: Camera Reference 100% | Panel 2: First Proof 100% | Panel 3: 4x Diff Heatmap]");
        }
    }

    println!("\n✅ Calibration Diagnostics Complete.");
}
