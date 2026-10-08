//! First Proof Headless Inspection & Parity Audit CLI
//!
//! Provides automated, mathematical verification between the GPU viewport pipeline
//! (simulated Metal shader math on binned raw) and the CPU export pipeline (full demosaic).

use std::env;
use std::path::{Path, PathBuf};
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
    eprintln!("================================================================================");
    eprintln!("🔬 First Proof Darkroom Inspection & Color Science Analysis Suite");
    eprintln!("================================================================================");
    eprintln!("Usage: inspect <command> [args...]");
    eprintln!("\n📷 Photographic & Sensor Inspection:");
    eprintln!("  inspect-photo <path>                        Show metadata, XMP edits, and lens calibration");
    eprintln!("  dump-raw-details <path>                     Show sensor CFA, black/white levels, and WB multipliers");
    eprintln!("  dump-dng-profile <dng_path>                 Extract and dump embedded DNG profiles and matrices");
    eprintln!("  trace-pixel <path> <x> <y>                  Trace single pixel through all 5 pipeline stages");
    eprintln!("  extract-preview <raw_path> <out.jpg>        Extract embedded camera manufacturer JPEG");
    eprintln!("  export-photo <input> <output> [opts...]     Render photo with specific profile and exposure");
    eprintln!("\n🧪 Color Science & Parity Benchmarks:");
    eprintln!("  compare-images <img1> <img2>                Full-frame pixel delta and chromaticity audit");
    eprintln!("  compare-camera-preview <raw_path>           Benchmark CameraStandard render against camera JPEG");
    eprintln!("  audit-parity <path> [--diff-map <out.png>]  Run dual-engine (Metal VP vs CPU export) audit");
    eprintln!("  audit-synthetic [--json]                    Audit synthetic 24-patch Macbeth chart parity");
    eprintln!("  json-parity <path>                          JSON parity audit for automated test harnesses");
    eprintln!("================================================================================");
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
        "compare-camera-preview" => {
            if args.len() < 3 {
                eprintln!("Error: missing raw path. Usage: inspect compare-camera-preview <raw_path>");
                std::process::exit(1);
            }
            cmd_compare_camera_preview(&args[2]);
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
                eprintln!("Usage: inspect export-photo <input> <output> [--profile <AgxVivid|AgxFilmic|CameraStandard|CameraPortrait|CameraLandscape|CameraVivid>]");
                std::process::exit(1);
            }
            let mut edits = read_xmp_file(Path::new(&args[2])).map(|x| x.edits).unwrap_or_default();
            for i in 4..args.len() {
                if args[i] == "--profile" && i + 1 < args.len() {
                    match args[i + 1].as_str() {
                        "AgxVivid" => edits.tone_profile = firstproof_core::ColorProfileMode::AgxVivid,
                        "AgxFilmic" => edits.tone_profile = firstproof_core::ColorProfileMode::AgxFilmic,
                        "CameraStandard" => edits.tone_profile = firstproof_core::ColorProfileMode::CameraStandard,
                        "CameraPortrait" => edits.tone_profile = firstproof_core::ColorProfileMode::CameraPortrait,
                        "CameraLandscape" => edits.tone_profile = firstproof_core::ColorProfileMode::CameraLandscape,
                        "CameraVivid" => edits.tone_profile = firstproof_core::ColorProfileMode::CameraVivid,
                        other => eprintln!("Unknown profile: {}", other),
                    }
                } else if args[i] == "--exposure" && i + 1 < args.len() {
                    if let Ok(ev) = args[i + 1].parse::<f32>() {
                        edits.exposure = ev;
                    }
                }
            }
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
            println!("Exported successfully to {} with profile {:?}", args[3], edits.tone_profile);
        }
        "extract-preview" => {
            if args.len() < 4 {
                eprintln!("Usage: inspect extract-preview <input> <output>");
                std::process::exit(1);
            }
            let jpeg_bytes = firstproof_core::raw::extract_preview_jpeg(&args[2]).expect("Failed to extract preview");
            std::fs::write(&args[3], jpeg_bytes).expect("Failed to write preview JPEG");
            println!("Extracted embedded preview ({} bytes) to {}", std::fs::metadata(&args[3]).unwrap().len(), args[3]);
        }
        "dump-dng-profile" => {
            let path = Path::new(&args[2]);
            let bytes = std::fs::read(path).unwrap();
            match firstproof_core::color::dcp::DcpProfile::from_bytes(&bytes) {
                Ok(profile) => {
                    println!("Profile Name: {}", profile.profile_name);
                    println!("Illuminant 1: {}", profile.illuminant1);
                    println!("Illuminant 2: {:?}", profile.illuminant2);
                    println!("Color Matrix 1:\n{:?}", profile.color_matrix1.0);
                    println!("Color Matrix 2:\n{:?}", profile.color_matrix2.map(|m| m.0));
                    println!("Forward Matrix 1:\n{:?}", profile.forward_matrix1.map(|m| m.0));
                    println!("Forward Matrix 2:\n{:?}", profile.forward_matrix2.map(|m| m.0));
                    let m_fwd = profile.interpolate_sensor_to_rec2020(2856.0);
                    println!("Derived Sensor to Rec.2020 at 2856K with ForwardMatrix:\n{:?}", m_fwd.0);
                    let d50_to_d65 = firstproof_core::color::cat16_adaptation_matrix((0.3457, 0.3585), (0.3127, 0.3290));
                    let xyz_to_rec2020 = firstproof_core::color::ColorSpacePrimaries::REC2020.xyz_to_rgb_matrix();
                    let d50_to_rec2020 = xyz_to_rec2020.multiply_matrix(&d50_to_d65);
                    println!("Universal XYZ_D50 -> Linear Rec.2020 Matrix:\n{:?}", d50_to_rec2020.0);
                    println!("Tone curve present: {}", profile.tone_curve.is_some());
                    println!("HueSatMap 1 present: {}", profile.hue_sat_map1.is_some());
                    println!("HueSatMap 2 present: {}", profile.hue_sat_map2.is_some());
                }
                Err(e) => println!("Error parsing DcpProfile from DNG: {:?}", e),
            }
        }
        "compare-pixel-matrices" => {
            use firstproof_core::raw_decoder::{RawDecoder, RawlerDecoder};
            let path = Path::new(&args[2]);
            let frame = RawDecoder::decode(&RawlerDecoder, path).unwrap();
            let dng_path = if args.len() > 3 {
                PathBuf::from(&args[3])
            } else {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/_MG_2651.dng")
            };
            let dng_bytes = std::fs::read(&dng_path).unwrap();
            let dcp = firstproof_core::color::dcp::DcpProfile::from_bytes(&dng_bytes).unwrap();
            let temp = frame.as_shot_temperature();
            let m_fwd = dcp.interpolate_sensor_to_rec2020(temp);

            let (wb_r, wb_b) = frame.as_shot_wb();
            let srgb_gamma = |c: f32| -> u8 {
                let v = if c <= 0.0031308 { c * 12.92 } else { 1.055 * c.max(0.0).powf(1.0 / 2.4) - 0.055 };
                (v.clamp(0.0, 1.0) * 255.0).round() as u8
            };
            let std_curve = firstproof_core::color::dcp::DcpProfile::built_in_standard_tone_curve();

            println!("Derived ForwardMatrix at {:.1}K:\n{:?}", temp, m_fwd.0);

            for &(cx, cy, label) in &[(362, 1347, "Shadow/Child"), (1503, 1416, "Midtone/Clothing")] {
                println!("\n--- Testing Pixel ({}, {}) - {} ---", cx, cy, label);
                let full_x = frame.active_x + cx;
                let full_y = frame.active_y + cy;
                let black = frame.black_levels[0].max(0.0);
                let white = frame.white_levels[0].max(black + 1.0);
                let scale = 1.0 / (white - black);

                // Sample 3x3 surrounding photosites to get accurate local raw color
                let mut r_sum = 0.0f32; let mut r_cnt = 0.0f32;
                let mut g_sum = 0.0f32; let mut g_cnt = 0.0f32;
                let mut b_sum = 0.0f32; let mut b_cnt = 0.0f32;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let px = (full_x as isize + dx) as usize;
                        let py = (full_y as isize + dy) as usize;
                        let val = frame.sensor_data[py * frame.full_width + px] as f32;
                        let norm = ((val - black) * scale).max(0.0);
                        match frame.color_at(py, px) {
                            0 => { r_sum += norm; r_cnt += 1.0; }
                            1 => { g_sum += norm; g_cnt += 1.0; }
                            2 => { b_sum += norm; b_cnt += 1.0; }
                            _ => {}
                        }
                    }
                }
                let r_norm = if r_cnt > 0.0 { r_sum / r_cnt } else { 0.0 };
                let g_norm = if g_cnt > 0.0 { g_sum / g_cnt } else { 0.0 };
                let b_norm = if b_cnt > 0.0 { b_sum / b_cnt } else { 0.0 };

                let (r_wb, g_wb, b_wb) = firstproof_engine::blend_sensor_highlights(
                    r_norm, g_norm, b_norm, wb_r, wb_b, frame.linear_response_limit,
                );
                println!("Raw Norm: R={:.5}, G={:.5}, B={:.5}", r_norm, g_norm, b_norm);
                println!("WB Signal: R={:.5}, G={:.5}, B={:.5}", r_wb, g_wb, b_wb);

                // 1. Current Inverted ColorMatrix at +0.0 EV
                let m_inv = frame.sensor_to_rec2020_at(temp);
                let r_inv = (m_inv[0][0] * r_wb + m_inv[0][1] * g_wb + m_inv[0][2] * b_wb).max(0.0);
                let g_inv = (m_inv[1][0] * r_wb + m_inv[1][1] * g_wb + m_inv[1][2] * b_wb).max(0.0);
                let b_inv = (m_inv[2][0] * r_wb + m_inv[2][1] * g_wb + m_inv[2][2] * b_wb).max(0.0);
                let c_inv_srgb = firstproof_core::color::REC2020_TO_LINEAR_SRGB.multiply_vector(Color3::new(r_inv, g_inv, b_inv));
                let lum_inv = 0.2126 * c_inv_srgb.r().max(0.0) + 0.7152 * c_inv_srgb.g().max(0.0) + 0.0722 * c_inv_srgb.b().max(0.0);
                let lum_curved_inv = firstproof_core::color::evaluate_tone_curve(lum_inv, &std_curve);
                let gain_inv = lum_curved_inv / lum_inv.max(1e-6);
                let disp_inv = (
                    srgb_gamma(c_inv_srgb.r().max(0.0) * gain_inv),
                    srgb_gamma(c_inv_srgb.g().max(0.0) * gain_inv),
                    srgb_gamma(c_inv_srgb.b().max(0.0) * gain_inv),
                );

                // 2. ForwardMatrix at +0.75 EV (Standard Canon EOS baseline exposure)
                let base_gain = 2.0f32.powf(0.75);
                let r_fwd = (m_fwd.0[0][0] * r_wb + m_fwd.0[0][1] * g_wb + m_fwd.0[0][2] * b_wb).max(0.0) * base_gain;
                let g_fwd = (m_fwd.0[1][0] * r_wb + m_fwd.0[1][1] * g_wb + m_fwd.0[1][2] * b_wb).max(0.0) * base_gain;
                let b_fwd = (m_fwd.0[2][0] * r_wb + m_fwd.0[2][1] * g_wb + m_fwd.0[2][2] * b_wb).max(0.0) * base_gain;
                let c_fwd_srgb = firstproof_core::color::REC2020_TO_LINEAR_SRGB.multiply_vector(Color3::new(r_fwd, g_fwd, b_fwd));
                let lum_fwd = 0.2126 * c_fwd_srgb.r().max(0.0) + 0.7152 * c_fwd_srgb.g().max(0.0) + 0.0722 * c_fwd_srgb.b().max(0.0);
                let lum_curved_fwd = firstproof_core::color::evaluate_tone_curve(lum_fwd, &std_curve);
                let gain_fwd = lum_curved_fwd / lum_fwd.max(1e-6);
                let disp_fwd = (
                    srgb_gamma(c_fwd_srgb.r().max(0.0) * gain_fwd),
                    srgb_gamma(c_fwd_srgb.g().max(0.0) * gain_fwd),
                    srgb_gamma(c_fwd_srgb.b().max(0.0) * gain_fwd),
                );

                println!("1. Old Inverted ColorMatrix (+0.0 EV): Display RGB = {:?}", disp_inv);
                println!("2. New ForwardMatrix        (+0.75 EV): Display RGB = {:?}", disp_fwd);

                let agx_fwd = firstproof_core::color::agx_tone_map_vivid(Color3::new(r_fwd, g_fwd, b_fwd), 1.0);
                let disp_agx_fwd = (
                    srgb_gamma(agx_fwd.r()),
                    srgb_gamma(agx_fwd.g()),
                    srgb_gamma(agx_fwd.b()),
                );
                println!("3. New ForwardMatrix + AgX Vivid:        Display RGB = {:?}", disp_agx_fwd);
            }
        }
        "dump-raw-details" => {
            let path = Path::new(&args[2]);
            let raw_file = rawler::decode_file(path).unwrap();
            println!("Rawler clean_make: {}", raw_file.clean_make);
            println!("Rawler clean_model: {}", raw_file.clean_model);
            println!("Rawler blacklevel: {:?}", raw_file.blacklevel.levels);
            println!("Rawler whitelevel: {:?}", raw_file.whitelevel);
            println!("Rawler wb_coeffs: {:?}", raw_file.wb_coeffs);
            println!("Rawler color_matrix: {:?}", raw_file.color_matrix);
            println!("Rawler cfa: {:?}", raw_file.camera.cfa);
        }
        "dump-sensor-patch" => {
            use firstproof_core::raw_decoder::{RawDecoder, RawlerDecoder};
            let path = Path::new(&args[2]);
            let cx: usize = args[3].parse().unwrap();
            let cy: usize = args[4].parse().unwrap();
            let frame = RawlerDecoder.decode(path).unwrap();
            let (wb_r, wb_b) = frame.as_shot_wb();
            let temp = frame.as_shot_temperature();
            let m = frame.compute_sensor_to_rec2020();
            let dcp = firstproof_core::color::dcp::DcpProfile::built_in_for_camera(&frame.camera_model);
            let dcp_m = dcp.interpolate_sensor_to_rec2020(temp);
            let black = frame.black_levels[0];
            let white = frame.white_levels[0];
            let scale = 1.0 / (white - black);
            println!("Camera: {}, Temp: {:.1}K", frame.camera_model, temp);
            println!("CFA: {:?}, Black: {}, White: {}, Scale: {}", frame.cfa_pattern, black, white, scale);
            println!("Active Crop: x={}, y={}, w={}, h={} (Full: {}x{})",
                frame.active_x, frame.active_y, frame.active_width, frame.active_height, frame.full_width, frame.full_height);
            println!("WB: R={}, B={}", wb_r, wb_b);
            println!("Rawler color_matrix: {:?}", frame.color_matrix);
            println!("Rawler color_matrix_tungsten: {:?}", frame.color_matrix_tungsten);
            println!("Frame ColorMatrix Inverted:\n{:?}", m);
            println!("DCP Profile (with ForwardMatrix):\n{:?}", dcp_m.0);
            use firstproof_core::raw_decoder::RawSensorFrameExt;
            println!("RawSensorFrameExt sensor_to_rec2020:\n{:?}", frame.sensor_to_rec2020_matrix_at(temp).0);
            println!("Raw Sensor Values around ({}, {}):", cx, cy);
            for dy in -2i32..=2 {
                let y = (cy as i32 + dy) as usize;
                let mut row_str = String::new();
                for dx in -2i32..=2 {
                    let x = (cx as i32 + dx) as usize;
                    let val = frame.sensor_data[y * frame.full_width + x];
                    let col_name = match frame.color_at(y, x) {
                        0 => "R",
                        1 => "G",
                        2 => "B",
                        _ => "?",
                    };
                    row_str.push_str(&format!("{}:{:5} ", col_name, val));
                }
                println!("{}", row_str);
            }
        }
        "trace-pixel" => {
            use firstproof_core::raw_decoder::{RawDecoder, RawlerDecoder, RawSensorFrameExt};
            let path = Path::new(&args[2]);
            let cx: usize = args[3].parse().unwrap();
            let cy: usize = args[4].parse().unwrap();
            let frame = RawlerDecoder.decode(path).unwrap();
            let (wb_r, wb_b) = frame.as_shot_wb();
            let temp = frame.as_shot_temperature();
            let black = frame.black_levels[0];
            let white = frame.white_levels[0];
            let scale = 1.0 / (white - black);

            // Compute both matrices
            let m_hardcoded = frame.compute_sensor_to_rec2020();
            let m_real_dng = frame.sensor_to_rec2020_matrix_at(temp);

            println!("============================================================");
            println!("🔬 First Proof Pipeline Tracer at ({}, {})", cx, cy);
            println!("============================================================");
            println!("📷 Camera: {}, Temp: {:.1}K, CFA: {:?}", frame.camera_model, temp, frame.cfa_pattern);
            println!("🎨 Black: {}, White: {}, Scale: {:.8}", black, white, scale);
            println!("🎨 As-Shot WB Multipliers: R={:.4}, B={:.4}", wb_r, wb_b);
            println!("🎨 ColorMatrix Tungsten: {:?}", frame.color_matrix_tungsten);
            println!("🎨 ColorMatrix Daylight: {:?}", frame.color_matrix);

            // Sample raw sensor 2x2 block
            let full_x = frame.active_x + cx;
            let full_y = frame.active_y + cy;
            let val = frame.sensor_data[full_y * frame.full_width + full_x] as f32;
            let c_type = match frame.color_at(full_y, full_x) {
                0 => "Red",
                1 => "Green",
                2 => "Blue",
                _ => "Unknown",
            };
            println!("1. CFA Pixel ({}, {}): Raw Count = {} ({})", full_x, full_y, val, c_type);
            println!("   4x4 Raw Sensor Neighborhood:");
            for dy in -2..=2 {
                let py = full_y as isize + dy;
                let mut line = String::new();
                for dx in -2..=2 {
                    let px = full_x as isize + dx;
                    let pval = frame.sensor_data[py as usize * frame.full_width + px as usize];
                    let cname = match frame.color_at(py as usize, px as usize) {
                        0 => "R", 1 => "G", 2 => "B", _ => "?",
                    };
                    line.push_str(&format!("{}:{:5} ", cname, pval));
                }
                println!("     {}", line);
            }

            // Let's also decode the full demosaiced image to inspect the interpolated pixel
            let img = firstproof_core::export::decode_image(path).unwrap();
            let idx = (cy * img.width + cx) * 3;
            let r_rec = img.data[idx];
            let g_rec = img.data[idx + 1];
            let b_rec = img.data[idx + 2];
            let c_rec = Color3::new(r_rec, g_rec, b_rec);
            println!("\n2. Demosaiced Linear Rec.2020 (Current Engine):");
            println!("   R={:.6}, G={:.6}, B={:.6}", r_rec, g_rec, b_rec);
            let inv_m = m_real_dng.inverse().unwrap_or(firstproof_core::color::Matrix3x3::IDENTITY);
            let c_sensor_est = inv_m.multiply_vector(c_rec);
            println!("   Estimated Sensor (wb applied): R={:.6}, G={:.6}, B={:.6}", c_sensor_est.r(), c_sensor_est.g(), c_sensor_est.b());

            println!("\n3. Dual-Engine Matrix Comparison:");
            println!("   Hardcoded Matrix:\n   {:?}", m_hardcoded);
            println!("   Real DNG Matrix:\n   {:?}", m_real_dng.0);

            let c_srgb = firstproof_core::color::REC2020_TO_LINEAR_SRGB.multiply_vector(c_rec);
            println!("\n4. Linear sRGB Working Space:");
            println!("   R={:.6}, G={:.6}, B={:.6}", c_srgb.r(), c_srgb.g(), c_srgb.b());
            let (h, s, v) = firstproof_core::color::rgb_to_hsv(c_srgb.r().max(0.0), c_srgb.g().max(0.0), c_srgb.b().max(0.0));
            println!("   HSV in sRGB: H={:.1}°, S={:.4}, V={:.4}", h, s, v);

            let agx_vivid = firstproof_core::color::agx_tone_map_vivid(c_rec, 1.0);
            let srgb_gamma = |c: f32| -> u8 {
                let v = if c <= 0.0031308 { c * 12.92 } else { 1.055 * c.max(0.0).powf(1.0 / 2.4) - 0.055 };
                (v.clamp(0.0, 1.0) * 255.0).round() as u8
            };
            println!("\n5. Output Profiles:");
            println!("   AgX Vivid (Linear sRGB):  R={:.6}, G={:.6}, B={:.6} -> Display RGB: ({}, {}, {})",
                agx_vivid.r(), agx_vivid.g(), agx_vivid.b(),
                srgb_gamma(agx_vivid.r()), srgb_gamma(agx_vivid.g()), srgb_gamma(agx_vivid.b())
            );

            let std_curve = firstproof_core::color::dcp::DcpProfile::built_in_standard_tone_curve();
            let cam_r = firstproof_core::color::evaluate_tone_curve(c_srgb.r().max(0.0), &std_curve);
            let cam_g = firstproof_core::color::evaluate_tone_curve(c_srgb.g().max(0.0), &std_curve);
            let cam_b = firstproof_core::color::evaluate_tone_curve(c_srgb.b().max(0.0), &std_curve);
            println!("   Camera Standard (sRGB):   R={:.6}, G={:.6}, B={:.6} -> Display RGB: ({}, {}, {})",
                cam_r, cam_g, cam_b,
                srgb_gamma(cam_r), srgb_gamma(cam_g), srgb_gamma(cam_b)
            );
        }
        "find-highlights" => {
            use firstproof_core::raw_decoder::{RawDecoder, RawlerDecoder};
            let path = Path::new(&args[2]);
            let frame = RawlerDecoder.decode(path).unwrap();
            let img = firstproof_core::export::decode_image(path).unwrap();
            let (_wb_r, _wb_b) = frame.as_shot_wb();
            let _black = frame.black_levels[0];
            let _white = frame.white_levels[0];
            let _scale = 1.0 / (_white - _black);

            println!("Searching highlights in {} ({}x{})...", path.display(), img.width, img.height);
            let srgb_gamma = |c: f32| -> u8 {
                let v = if c <= 0.0031308 { c * 12.92 } else { 1.055 * c.max(0.0).powf(1.0 / 2.4) - 0.055 };
                (v.clamp(0.0, 1.0) * 255.0).round() as u8
            };

            // Sample grid every 100 pixels
            for y in (100..img.height - 100).step_by(100) {
                for x in (100..img.width - 100).step_by(100) {
                    let idx = (y * img.width + x) * 3;
                    let r_rec = img.data[idx];
                    let g_rec = img.data[idx + 1];
                    let b_rec = img.data[idx + 2];
                    let c_rec = Color3::new(r_rec, g_rec, b_rec);
                    let agx = firstproof_core::color::agx_tone_map_vivid(c_rec, 1.0);
                    let rgb = (srgb_gamma(agx.r()), srgb_gamma(agx.g()), srgb_gamma(agx.b()));
                    
                    // Look for bright skin or blown highlights: high green or blue, or high brightness
                    if (rgb.0 > 120 || rgb.1 > 120 || rgb.2 > 120) && (rgb.2 > rgb.0 || rgb.1 > rgb.0) {
                        let full_x = frame.active_x + x;
                        let full_y = frame.active_y + y;
                        let raw_val = frame.sensor_data[full_y * frame.full_width + full_x];
                        println!("At ({:4}, {:4}): RawCount={:5} | Rec2020: ({:.3}, {:.3}, {:.3}) | Display: ({:3}, {:3}, {:3})",
                            x, y, raw_val, r_rec, g_rec, b_rec, rgb.0, rgb.1, rgb.2);
                    }
                }
            }
        }
        "test-types-full-demosaic" => {
            use firstproof_core::raw_decoder::{RawDecoder, RawlerDecoder};
            let path = Path::new(&args[2]);
            let cx: usize = args[3].parse().unwrap();
            let cy: usize = args[4].parse().unwrap();
            let frame = RawlerDecoder.decode(path).unwrap();
            println!("Decoding with frame.to_full_demosaiced_linear_f16()...");
            let (f16_buf, w, h) = frame.to_full_demosaiced_linear_f16();
            println!("Decoded {}x{} RGBA f16 buffer", w, h);
            let idx = (cy * (w as usize) + cx) * 4;
            let r = firstproof_core::export::f16_to_f32(f16_buf[idx]);
            let g = firstproof_core::export::f16_to_f32(f16_buf[idx + 1]);
            let b = firstproof_core::export::f16_to_f32(f16_buf[idx + 2]);
            println!("Linear Rec.2020 at ({}, {}): R={:.6}, G={:.6}, B={:.6}", cx, cy, r, g, b);
            let c_rec = Color3::new(r, g, b);
            let agx_vivid = firstproof_core::color::agx_tone_map_vivid(c_rec, 1.0);
            let srgb_gamma = |c: f32| -> u8 {
                let v = if c <= 0.0031308 { c * 12.92 } else { 1.055 * c.max(0.0).powf(1.0 / 2.4) - 0.055 };
                (v.clamp(0.0, 1.0) * 255.0).round() as u8
            };
            println!("AgX Vivid Display: R={}, G={}, B={}", srgb_gamma(agx_vivid.r()), srgb_gamma(agx_vivid.g()), srgb_gamma(agx_vivid.b()));
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
        println!("🎨 Frame black_levels: {:?}", frame.black_levels);
        println!("🎨 Frame white_levels: {:?}", frame.white_levels);
        println!("🎨 Frame wb_coeffs: {:?}", frame.wb_coeffs);
        println!("🎨 Frame as_shot_wb: {:?}", frame.as_shot_wb());
        println!("🎨 Frame as_shot_temperature: {:.1}K", frame.as_shot_temperature());
        println!("🎨 Frame baseline_exposure: {:.3} EV (gain: {:.3}x)", frame.baseline_exposure, 2.0f32.powf(frame.baseline_exposure));
        println!("🎨 Frame linear_response_limit: {:.3} (anti-blooming knee: {:.1}%)", frame.linear_response_limit, frame.linear_response_limit * 100.0);
        println!("🎨 Frame noise_profile: {:?}", frame.noise_profile);
        println!("🎨 Frame profile_tone_curve: {}", if let Some(ref c) = frame.profile_tone_curve { format!("Present ({} points)", c.len()) } else { "None".to_string() });
        println!("🎨 Frame warp_rectilinear: {:?}", frame.warp_rectilinear);
        let m = frame.compute_sensor_to_rec2020();
        println!("🎨 Sensor to Rec.2020: {:?}", m);
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
            total_diff_r += (p1.0[0] as i32 - p2.0[0] as i32).unsigned_abs() as u64;
            total_diff_g += (p1.0[1] as i32 - p2.0[1] as i32).unsigned_abs() as u64;
            total_diff_b += (p1.0[2] as i32 - p2.0[2] as i32).unsigned_abs() as u64;
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

fn cmd_compare_camera_preview(raw_path_str: &str) {
    let raw_path = Path::new(raw_path_str);
    println!("Extracting camera manufacturer preview from {}...", raw_path_str);
    let preview_bytes = firstproof_core::raw::extract_preview_jpeg(raw_path)
        .expect("Failed to extract embedded preview JPEG");
    let preview_img = image::load_from_memory(&preview_bytes)
        .expect("Failed to decode embedded camera preview")
        .to_rgb8();

    println!("Rendering First Proof CameraStandard export from {}...", raw_path_str);
    let temp_export = std::env::temp_dir().join(format!("fp_cam_std_{}.jpg", std::process::id()));
    let mut edits = read_xmp_file(raw_path).map(|x| x.edits).unwrap_or_default();
    edits.tone_profile = firstproof_core::ColorProfileMode::CameraStandard;

    firstproof_core::export::export_photo_file(
        raw_path,
        &temp_export,
        &edits,
        None,
        92,
        None,
        firstproof_core::ExportColorSpace::Srgb,
        firstproof_core::ExportRenderingIntent::Perceptual,
        false,
    ).expect("Failed to export First Proof CameraStandard preview");

    let export_img = image::open(&temp_export)
        .expect("Failed to open exported image")
        .to_rgb8();
    let _ = std::fs::remove_file(&temp_export);

    println!("\nComparing Camera Preview vs First Proof CameraStandard Export:");
    println!("  Camera Preview: {}x{}", preview_img.width(), preview_img.height());
    println!("  CameraStandard: {}x{}", export_img.width(), export_img.height());

    let target_w = preview_img.width();
    let target_h = preview_img.height();
    let export_resized = if export_img.width() != target_w || export_img.height() != target_h {
        image::imageops::resize(&export_img, target_w, target_h, image::imageops::FilterType::Lanczos3)
    } else {
        export_img
    };

    let mut total_diff_r = 0u64;
    let mut total_diff_g = 0u64;
    let mut total_diff_b = 0u64;
    let count = (target_w as u64) * (target_h as u64);

    for y in 0..target_h {
        for x in 0..target_w {
            let p1 = preview_img.get_pixel(x, y);
            let p2 = export_resized.get_pixel(x, y);
            total_diff_r += (p1.0[0] as i32 - p2.0[0] as i32).unsigned_abs() as u64;
            total_diff_g += (p1.0[1] as i32 - p2.0[1] as i32).unsigned_abs() as u64;
            total_diff_b += (p1.0[2] as i32 - p2.0[2] as i32).unsigned_abs() as u64;
        }
    }

    println!("\nOverall Pixel Deltas across {} pixels:", count);
    println!("  Mean RGB Delta: R={:.2}, G={:.2}, B={:.2} (out of 255)",
        total_diff_r as f64 / count as f64,
        total_diff_g as f64 / count as f64,
        total_diff_b as f64 / count as f64);
    let mean_rgb = (total_diff_r + total_diff_g + total_diff_b) as f64 / (3.0 * count as f64);
    println!("  Mean Total Error: {:.2}/255 ({:.2}%)", mean_rgb, (mean_rgb / 255.0) * 100.0);

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
        "Region", "Camera Preview", "FP CameraStd", "Delta [R,G,B]", "G/R (Cam vs FP)");
    println!("--------------------------------------------------------------------------------");
    for (label, rx, ry) in test_points {
        let px = ((target_w as f32) * rx).min((target_w - 1) as f32) as u32;
        let py = ((target_h as f32) * ry).min((target_h - 1) as f32) as u32;
        let p1 = preview_img.get_pixel(px, py);
        let p2 = export_resized.get_pixel(px, py);
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


