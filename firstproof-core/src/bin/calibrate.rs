use std::env;
use image::{RgbImage, Rgb, imageops};
use firstproof_core::export::decode_raw_preview_buffer;
use firstproof_core::raw::extract_preview_jpeg;

fn main() {
    let args: Vec<String> = env::args().collect();
    let raw_path = if args.len() > 1 {
        &args[1]
    } else {
        "/Users/blikkie/Pictures/Darktable/20250409_eos_pull/20250409_0359.CR2"
    };

    println!("============================================================");
    println!("🔬 First Proof RAW vs Camera JPEG Calibration Diagnostics Tool");
    println!("Analyzing: {}", raw_path);
    println!("============================================================\n");

    // 1. Extract Camera Reference JPEG
    let preview_bytes = match extract_preview_jpeg(raw_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error extracting embedded preview JPEG: {}", e);
            return;
        }
    };

    let cam_dyn = match image::load_from_memory(&preview_bytes) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Error loading embedded JPEG memory: {}", e);
            return;
        }
    };
    let cam_rgb = cam_dyn.to_rgb8();
    println!("📸 Camera Reference JPEG: {}x{}", cam_rgb.width(), cam_rgb.height());

    // 2. Decode RAW Sensor Pipeline
    let max_dim = 1200u32;
    let (raw_w, raw_h, raw_rgba) = match decode_raw_preview_buffer(raw_path, Some(max_dim)) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error decoding RAW sensor preview: {}", e);
            return;
        }
    };
    println!("⚙️  Decoded RAW Sensor:    {}x{}", raw_w, raw_h);

    let raw_rgb = RgbImage::from_fn(raw_w, raw_h, |x, y| {
        let idx = ((y * raw_w + x) * 4) as usize;
        Rgb([raw_rgba[idx], raw_rgba[idx + 1], raw_rgba[idx + 2]])
    });

    // 3. Resize Camera JPEG to match RAW dimensions for pixel-to-pixel comparison
    let target_w = raw_w;
    let target_h = raw_h;
    let cam_resized = imageops::resize(&cam_rgb, target_w, target_h, imageops::FilterType::Lanczos3);

    // 4. Compute Statistical & Color Science Metrics
    let mut total_sq_err: f64 = 0.0;
    let mut total_abs_diff_r: f64 = 0.0;
    let mut total_abs_diff_g: f64 = 0.0;
    let mut total_abs_diff_b: f64 = 0.0;

    let mut cam_sum_r: f64 = 0.0;
    let mut cam_sum_g: f64 = 0.0;
    let mut cam_sum_b: f64 = 0.0;
    let mut raw_sum_r: f64 = 0.0;
    let mut raw_sum_g: f64 = 0.0;
    let mut raw_sum_b: f64 = 0.0;

    let pixel_count = (target_w * target_h) as f64;

    // Create Difference Heatmap
    let mut diff_img = RgbImage::new(target_w, target_h);

    for y in 0..target_h {
        for x in 0..target_w {
            let cp = cam_resized.get_pixel(x, y);
            let rp = raw_rgb.get_pixel(x, y);

            let cr = cp[0] as f64;
            let cg = cp[1] as f64;
            let cb = cp[2] as f64;

            let rr = rp[0] as f64;
            let rg = rp[1] as f64;
            let rb = rp[2] as f64;

            cam_sum_r += cr;
            cam_sum_g += cg;
            cam_sum_b += cb;

            raw_sum_r += rr;
            raw_sum_g += rg;
            raw_sum_b += rb;

            let dr = (cr - rr).abs();
            let dg = (cg - rg).abs();
            let db = (cb - rb).abs();

            total_abs_diff_r += dr;
            total_abs_diff_g += dg;
            total_abs_diff_b += db;

            let sq = (cr - rr).powi(2) + (cg - rg).powi(2) + (cb - rb).powi(2);
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

    let cam_avg_r = cam_sum_r / pixel_count;
    let cam_avg_g = cam_sum_g / pixel_count;
    let cam_avg_b = cam_sum_b / pixel_count;

    let raw_avg_r = raw_sum_r / pixel_count;
    let raw_avg_g = raw_sum_g / pixel_count;
    let raw_avg_b = raw_sum_b / pixel_count;

    println!("\n📊 STATISTICAL METRICS:");
    println!("------------------------------------------------------------");
    println!("  • Mean Squared Error (MSE):  {:.2}", mse);
    println!("  • Peak SNR (PSNR):           {:.2} dB", psnr);
    println!("  • Mean Channel Errors:        ΔR={:.1} | ΔG={:.1} | ΔB={:.1} (out of 255)", mean_diff_r, mean_diff_g, mean_diff_b);
    println!("  • Camera Reference Averages: R={:.1}  | G={:.1}  | B={:.1}", cam_avg_r, cam_avg_g, cam_avg_b);
    println!("  • RAW Demosaic Averages:     R={:.1}  | G={:.1}  | B={:.1}", raw_avg_r, raw_avg_g, raw_avg_b);
    println!("------------------------------------------------------------");

    // 5. Generate Side-by-Side 3-Panel Diagnostic Image
    let combined_w = target_w * 3;
    let combined_h = target_h;
    let mut combined_img = RgbImage::new(combined_w, combined_h);

    imageops::overlay(&mut combined_img, &cam_resized, 0, 0);
    imageops::overlay(&mut combined_img, &raw_rgb, target_w as i64, 0);
    imageops::overlay(&mut combined_img, &diff_img, (target_w * 2) as i64, 0);

    let output_comparison_path = "raw_calibration_comparison.jpg";
    if let Err(e) = combined_img.save(output_comparison_path) {
        eprintln!("Failed to save comparison image: {}", e);
    } else {
        println!("\n✅ Side-by-Side Diagnostic Triptych Generated: {}", output_comparison_path);
        println!("   [Panel 1: Camera Reference JPEG | Panel 2: Decoded RAW | Panel 3: Amplified Error Heatmap]");
    }
}
