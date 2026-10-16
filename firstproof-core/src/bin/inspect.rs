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
                } else if args[i] == "--temperature" && i + 1 < args.len() {
                    if let Ok(t) = args[i + 1].parse::<f32>() {
                        edits.temperature = t;
                    }
                } else if args[i] == "--tint" && i + 1 < args.len() {
                    if let Ok(t) = args[i + 1].parse::<f32>() {
                        edits.tint = t;
                    }
                } else if args[i] == "--as-shot" {
                    if let Ok(frame) = firstproof_core::raw_decoder::RawDecoder::decode(&firstproof_core::raw_decoder::RawlerDecoder, Path::new(&args[2])) {
                        edits.temperature = frame.as_shot_temperature();
                        edits.tint = frame.as_shot_tint();
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
        "compare-passes" => {
            let path = &args[2];
            let (bw, bh, binned_bytes) = firstproof_core::export::decode_raw_preview_binned_buffer(path).expect("decode binned");
            let (fw, fh, full_bytes) = firstproof_core::export::decode_raw_preview_buffer(path, None).expect("decode full");
            println!("Binned: {}x{}, Full: {}x{}", bw, bh, fw, fh);

            let points = [
                ("Center", bw / 2, bh / 2),
                ("Sky / Cloud", (4560 / 2) as u32, (680 / 2) as u32),
                ("Trellis Sky", (3600 / 2) as u32, (800 / 2) as u32),
                ("Top-Left Sky", (560 / 2) as u32, (160 / 2) as u32),
                ("Shadow Grass", (1000 / 2) as u32, (3000 / 2) as u32),
                ("Midtone Wall", (2000 / 2) as u32, (2000 / 2) as u32),
            ];

            for (label, bx, by) in points {
                let b_idx = ((by * bw + bx) * 8) as usize;
                let rb = f16_to_f32(u16::from_le_bytes([binned_bytes[b_idx], binned_bytes[b_idx + 1]]));
                let gb = f16_to_f32(u16::from_le_bytes([binned_bytes[b_idx + 2], binned_bytes[b_idx + 3]]));
                let bb = f16_to_f32(u16::from_le_bytes([binned_bytes[b_idx + 4], binned_bytes[b_idx + 5]]));

                let fx = bx * 2;
                let fy = by * 2;
                let f_idx = ((fy * fw + fx) * 8) as usize;
                let rf = f16_to_f32(u16::from_le_bytes([full_bytes[f_idx], full_bytes[f_idx + 1]]));
                let gf = f16_to_f32(u16::from_le_bytes([full_bytes[f_idx + 2], full_bytes[f_idx + 3]]));
                let bf = f16_to_f32(u16::from_le_bytes([full_bytes[f_idx + 4], full_bytes[f_idx + 5]]));

                let (_hb, _sb, _) = firstproof_core::color::rgb_to_hsv(rb, gb, bb);
                let (_hf, _sf, _) = firstproof_core::color::rgb_to_hsv(rf, gf, bf);

                // Convert both to display sRGB via CameraStandard
                let cs_disp = |r_lin: f32, g_lin: f32, b_lin: f32| -> (u8, u8, u8, f32, f32) {
                    let c_srgb = firstproof_core::color::REC2020_TO_LINEAR_SRGB.multiply_vector(firstproof_core::color::Color3::new(r_lin, g_lin, b_lin));
                    let dcp = firstproof_core::color::dcp::DcpProfile::built_in_for_camera("Canon EOS 60D");
                    let curve = dcp.tone_curve.as_deref().unwrap_or(&[]);
                    let curved_r = firstproof_core::color::dcp::evaluate_tone_curve(c_srgb.r().max(0.0), curve);
                    let curved_g = firstproof_core::color::dcp::evaluate_tone_curve(c_srgb.g().max(0.0), curve);
                    let curved_b = firstproof_core::color::dcp::evaluate_tone_curve(c_srgb.b().max(0.0), curve);
                    let dr = (firstproof_core::color::linear_to_srgb_gamma(curved_r) * 255.0).round().clamp(0.0, 255.0) as u8;
                    let dg = (firstproof_core::color::linear_to_srgb_gamma(curved_g) * 255.0).round().clamp(0.0, 255.0) as u8;
                    let db = (firstproof_core::color::linear_to_srgb_gamma(curved_b) * 255.0).round().clamp(0.0, 255.0) as u8;
                    let (h, s, _) = firstproof_core::color::rgb_to_hsv(dr as f32 / 255.0, dg as f32 / 255.0, db as f32 / 255.0);
                    (dr, dg, db, h, s)
                };
                let (dr_b, dg_b, db_b, dh_b, ds_b) = cs_disp(rb, gb, bb);
                let (dr_f, dg_f, db_f, dh_f, ds_f) = cs_disp(rf, gf, bf);

                let lr_path = Path::new("/Users/blikkie/Pictures/Lightroom Export/2016/2016-03-26/_MG_3717_edited.jpg");
                let lr_info = if let Ok(lr_img) = image::open(lr_path).map(|i| i.to_rgb8()) {
                    let lp = lr_img.get_pixel(fx.min(lr_img.width() - 1), fy.min(lr_img.height() - 1));
                    let (lh, ls, _) = firstproof_core::color::rgb_to_hsv(lp[0] as f32 / 255.0, lp[1] as f32 / 255.0, lp[2] as f32 / 255.0);
                    format!(" | LR: RGB=({}, {}, {}), H={:.1}°, S={:.3}", lp[0], lp[1], lp[2], lh, ls)
                } else {
                    String::new()
                };

                println!("=== {} (binned bx={}, by={} | full fx={}, fy={}) ===", label, bx, by, fx, fy);
                println!("  Pass 1 (Binned): Rec=({:.4}, {:.4}, {:.4}) -> CamStd RGB=({}, {}, {}), H={:.1}°, S={:.3}", rb, gb, bb, dr_b, dg_b, db_b, dh_b, ds_b);
                println!("  Pass 2 (Full):   Rec=({:.4}, {:.4}, {:.4}) -> CamStd RGB=({}, {}, {}), H={:.1}°, S={:.3}{}", rf, gf, bf, dr_f, dg_f, db_f, dh_f, ds_f, lr_info);
            }
        }
        "lr-slice" => {
            let lr_path = Path::new("/Users/blikkie/Pictures/Lightroom Export/2016/2016-03-26/_MG_3717_edited.jpg");
            let lr_img = image::open(lr_path).unwrap().to_rgb8();
            println!("Lightroom JPEG: {}x{}", lr_img.width(), lr_img.height());
            for x in (4400..4900).step_by(20) {
                let p = lr_img.get_pixel(x, 0);
                let (h, s, v) = firstproof_core::color::rgb_to_hsv(p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0);
                println!("x={}: RGB=({}, {}, {}), H={:.1}°, S={:.3}, V={:.3}", x, p[0], p[1], p[2], h, s, v);
            }
        }
        "fp-slice" => {
            let dng_path = &args[2];
            let decoder = firstproof_core::raw_decoder::RawlerDecoder;
            use firstproof_core::raw_decoder::RawDecoder;
            let frame = decoder.decode(Path::new(dng_path)).expect("decode frame");
            println!("Frame: {}x{}, pattern={:?}, wb={:?}, black={:?}, white={:?}",
                frame.active_width, frame.active_height, frame.cfa_pattern, frame.wb_coeffs, frame.black_levels, frame.white_levels);
            let limit = frame.linear_response_limit;
            println!("Linear response limit: {}", limit);

            // Print raw RGGB for each 2x2 binned block along y=0
            let raw_data = &frame.sensor_data;
            let _w = frame.active_width;
            for x_full in (4400..4900).step_by(20) {
                let bx = (x_full / 2) as usize;
                let by = 0usize;
                let sy = frame.active_y + by * 2;
                let sx = frame.active_x + bx * 2;
                let p00 = raw_data[sy * frame.full_width + sx] as f32;
                let p01 = raw_data[sy * frame.full_width + sx + 1] as f32;
                let p10 = raw_data[(sy + 1) * frame.full_width + sx] as f32;
                let p11 = raw_data[(sy + 1) * frame.full_width + sx + 1] as f32;
                let norm = |p: f32, ch: usize| -> f32 {
                    let b = frame.black_levels[ch];
                    let w = frame.white_levels[ch];
                    ((p - b) / (w - b)).clamp(0.0, 1.0)
                };
                let norm_00 = norm(p00, 0);
                let norm_01 = norm(p01, 1);
                let norm_10 = norm(p10, 2);
                let norm_11 = norm(p11, 3);
                let (r_norm, g_norm, b_norm) = match frame.cfa_pattern {
                    firstproof_engine::raw::CfaPattern::Gbrg => (norm_10, (norm_00 + norm_11) * 0.5, norm_01),
                    _ => (norm_00, (norm_01 + norm_10) * 0.5, norm_11),
                };
                let (r_bl, g_bl, b_bl) = firstproof_engine::raw::blend_sensor_highlights(
                    r_norm, g_norm, b_norm, frame.wb_coeffs[0], frame.wb_coeffs[2], limit
                );
                let sensor_to_rec = frame.compute_sensor_to_rec2020();
                let baseline_gain = 2.0f32.powf(frame.baseline_exposure);
                let r_lin = (sensor_to_rec[0][0] * r_bl + sensor_to_rec[0][1] * g_bl + sensor_to_rec[0][2] * b_bl) * baseline_gain;
                let g_lin = (sensor_to_rec[1][0] * r_bl + sensor_to_rec[1][1] * g_bl + sensor_to_rec[1][2] * b_bl) * baseline_gain;
                let b_lin = (sensor_to_rec[2][0] * r_bl + sensor_to_rec[2][1] * g_bl + sensor_to_rec[2][2] * b_bl) * baseline_gain;
                let tm = firstproof_core::color::agx_tone_map_vivid(firstproof_core::color::Color3::new(r_lin, g_lin, b_lin), 1.0);
                let dr = (tm.r().clamp(0.0, 1.0) * 255.0).round() as u8;
                let dg = (tm.g().clamp(0.0, 1.0) * 255.0).round() as u8;
                let db = (tm.b().clamp(0.0, 1.0) * 255.0).round() as u8;
                let (h, s, _v) = firstproof_core::color::rgb_to_hsv(dr as f32 / 255.0, dg as f32 / 255.0, db as f32 / 255.0);
                let c_srgb = firstproof_core::color::REC2020_TO_LINEAR_SRGB.multiply_vector(firstproof_core::color::Color3::new(r_lin, g_lin, b_lin));
                let lum = 0.2126 * c_srgb.r().max(0.0) + 0.7152 * c_srgb.g().max(0.0) + 0.0722 * c_srgb.b().max(0.0);
                let std_curve = firstproof_core::color::dcp::DcpProfile::built_in_standard_tone_curve();
                let lum_curved = firstproof_core::color::evaluate_tone_curve(lum, &std_curve);
                let gain = lum_curved / lum.max(1e-6);
                let srgb_gamma = |c: f32| -> f32 {
                    if c <= 0.0031308 { c * 12.92 } else { 1.055 * c.max(0.0).powf(1.0 / 2.4) - 0.055 }
                };
                let dr_cs = (srgb_gamma(c_srgb.r().max(0.0) * gain).clamp(0.0, 1.0) * 255.0).round() as u8;
                let dg_cs = (srgb_gamma(c_srgb.g().max(0.0) * gain).clamp(0.0, 1.0) * 255.0).round() as u8;
                let db_cs = (srgb_gamma(c_srgb.b().max(0.0) * gain).clamp(0.0, 1.0) * 255.0).round() as u8;
                let (h_cs, s_cs, _v_cs) = firstproof_core::color::rgb_to_hsv(dr_cs as f32 / 255.0, dg_cs as f32 / 255.0, db_cs as f32 / 255.0);

                let ref_path = Path::new("target/mg3731_camera_ref.jpg");
                let ref_str = if let Ok(ref_img) = image::open(ref_path).map(|i| i.to_rgb8()) {
                    let rx = ((x_full as f32) * (ref_img.width() as f32 / frame.active_width as f32)) as u32;
                    let rp = ref_img.get_pixel(rx.min(ref_img.width() - 1), 0);
                    let (rh, rs, rv) = firstproof_core::color::rgb_to_hsv(rp[0] as f32 / 255.0, rp[1] as f32 / 255.0, rp[2] as f32 / 255.0);
                    format!(" | Ref=({}, {}, {}), H={:.1}°, S={:.2}, V={:.2}", rp[0], rp[1], rp[2], rh, rs, rv)
                } else {
                    String::new()
                };
                println!("x={}: Agx=({}, {}, {}), H={:.1}°, S={:.2} | CamStd=({}, {}, {}), H={:.1}°, S={:.2}{}",
                    x_full, dr, dg, db, h, s, dr_cs, dg_cs, db_cs, h_cs, s_cs, ref_str);
            }
        }
        "trace-point" => {
            let dng_path = &args[2];
            let decoder = firstproof_core::raw_decoder::RawlerDecoder;
            use firstproof_core::raw_decoder::RawDecoder;
            let frame = decoder.decode(Path::new(dng_path)).expect("decode frame");
            let raw_data = &frame.sensor_data;
            let x: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(4560);
            let y: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(680);
            let sy = frame.active_y + y;
            let sx = frame.active_x + x;
            let p00 = raw_data[sy * frame.full_width + sx] as f32;
            let p01 = raw_data[sy * frame.full_width + sx + 1] as f32;
            let p10 = raw_data[(sy + 1) * frame.full_width + sx] as f32;
            let p11 = raw_data[(sy + 1) * frame.full_width + sx + 1] as f32;
            let norm = |p: f32, ch: usize| -> f32 {
                let b = frame.black_levels[ch];
                let w = frame.white_levels[ch];
                ((p - b) / (w - b)).clamp(0.0, 1.0)
            };
            let (r_raw, g_raw, b_raw) = match frame.cfa_pattern {
                firstproof_engine::raw::CfaPattern::Gbrg => (norm(p10, 2), (norm(p00, 0) + norm(p11, 3)) * 0.5, norm(p01, 1)),
                _ => (norm(p00, 0), (norm(p01, 1) + norm(p10, 2)) * 0.5, norm(p11, 3)),
            };
            println!("Point ({}, {}):", x, y);
            println!("  Raw normalized: R={:.4}, G={:.4}, B={:.4}", r_raw, g_raw, b_raw);
            let wb_r = frame.wb_coeffs[0];
            let wb_b = frame.wb_coeffs[2];
            println!("  WB coeffs: wb_r={:.4}, wb_b={:.4}", wb_r, wb_b);
            let pr = r_raw * wb_r;
            let pg = g_raw;
            let pb = b_raw * wb_b;
            println!("  After WB: P_R={:.4}, P_G={:.4}, P_B={:.4}", pr, pg, pb);
            let m = frame.compute_sensor_to_rec2020();
            println!("  Sensor to Rec2020 matrix:\n    {:?}\n    {:?}\n    {:?}", m[0], m[1], m[2]);
            let r_rec = m[0][0] * pr + m[0][1] * pg + m[0][2] * pb;
            let g_rec = m[1][0] * pr + m[1][1] * pg + m[1][2] * pb;
            let b_rec = m[2][0] * pr + m[2][1] * pg + m[2][2] * pb;
            println!("  Rec2020: R={:.4}, G={:.4}, B={:.4}", r_rec, g_rec, b_rec);
            let cm2 = glam::Mat3::from_cols_array(&[
                0.6719, -0.4408, -0.0887,
                -0.0994, 1.2426, 0.2129,
                -0.0925, 0.2211, 0.6051,
            ]);
            let cam2xyz = cm2.inverse();
            println!("  ColorMatrix2 inverse (Camera -> XYZ_D65):\n    {:?}\n    {:?}\n    {:?}",
                cam2xyz.row(0), cam2xyz.row(1), cam2xyz.row(2));
            let xyz_to_rec = firstproof_core::color::ColorSpacePrimaries::REC2020.xyz_to_rgb_matrix();
            let xyz_mat = glam::Mat3::from_cols_array(&[
                xyz_to_rec.0[0][0], xyz_to_rec.0[1][0], xyz_to_rec.0[2][0],
                xyz_to_rec.0[0][1], xyz_to_rec.0[1][1], xyz_to_rec.0[2][1],
                xyz_to_rec.0[0][2], xyz_to_rec.0[1][2], xyz_to_rec.0[2][2],
            ]);
            let cam2rec = xyz_mat * cam2xyz;
            println!("  cam2rec via ColorMatrix:\n    {:?}\n    {:?}\n    {:?}",
                cam2rec.row(0), cam2rec.row(1), cam2rec.row(2));
            let r_cm = cam2rec.row(0).x * pr + cam2rec.row(0).y * pg + cam2rec.row(0).z * pb;
            let g_cm = cam2rec.row(1).x * pr + cam2rec.row(1).y * pg + cam2rec.row(1).z * pb;
            let b_cm = cam2rec.row(2).x * pr + cam2rec.row(2).y * pg + cam2rec.row(2).z * pb;
            println!("  Rec2020 via ColorMatrix: R={:.4}, G={:.4}, B={:.4}", r_cm, g_cm, b_cm);
            let srgb_cm = firstproof_core::color::REC2020_TO_LINEAR_SRGB.multiply_vector(firstproof_core::color::Color3::new(r_cm, g_cm, b_cm));
            println!("  Linear sRGB via ColorMatrix: R={:.4}, G={:.4}, B={:.4}", srgb_cm.r(), srgb_cm.g(), srgb_cm.b());

            let ref_path = Path::new("target/mg3731_camera_ref.jpg");
            if let Ok(ref_img) = image::open(ref_path).map(|i| i.to_rgb8()) {
                let rx = ((x as f32) * (ref_img.width() as f32 / frame.active_width as f32)) as u32;
                let ry = ((y as f32) * (ref_img.height() as f32 / frame.active_height as f32)) as u32;
                let rp = ref_img.get_pixel(rx.min(ref_img.width() - 1), ry.min(ref_img.height() - 1));
                let (rh, rs, rv) = firstproof_core::color::rgb_to_hsv(rp[0] as f32 / 255.0, rp[1] as f32 / 255.0, rp[2] as f32 / 255.0);
                println!("  Camera Ref JPEG: RGB=({}, {}, {}), H={:.1}°, S={:.3}, V={:.3}", rp[0], rp[1], rp[2], rh, rs, rv);
            }
        }
        "inspect-trellis" => {
            let fp_path = args.get(2).map(|s| Path::new(s.as_str())).unwrap_or(Path::new("target/cam_std.jpg"));
            let ref_path = Path::new("target/mg3731_camera_ref.jpg");
            let fp_img = image::open(fp_path).unwrap().to_rgb8();
            let ref_img = image::open(ref_path).unwrap().to_rgb8();
            println!("Scanning right side trellis region (x=3500..5100, y=500..1800)...");
            let mut fp_magenta_count = 0;
            let mut ref_magenta_count = 0;
            let sx = ref_img.width() as f32 / fp_img.width() as f32;
            let sy = ref_img.height() as f32 / fp_img.height() as f32;
            for y in (500..1800).step_by(20) {
                for x in (3500..5100).step_by(20) {
                    let fp = fp_img.get_pixel(x, y);
                    let rx = ((x as f32) * sx) as u32;
                    let ry = ((y as f32) * sy) as u32;
                    let rp = ref_img.get_pixel(rx, ry);
                    let (fh, fs, _fv) = firstproof_core::color::rgb_to_hsv(fp[0] as f32 / 255.0, fp[1] as f32 / 255.0, fp[2] as f32 / 255.0);
                    let (rh, rs, _rv) = firstproof_core::color::rgb_to_hsv(rp[0] as f32 / 255.0, rp[1] as f32 / 255.0, rp[2] as f32 / 255.0);
                    if fh >= 235.0 && fh <= 340.0 && fs > 0.05 {
                        fp_magenta_count += 1;
                        if fp_magenta_count <= 8 {
                            println!("Trellis Magenta at (x={}, y={}): FP=({}, {}, {}), H={:.1}°, S={:.3} | Ref=({}, {}, {}), H={:.1}°, S={:.3}",
                                x, y, fp[0], fp[1], fp[2], fh, fs, rp[0], rp[1], rp[2], rh, rs);
                        }
                    }
                    if rh >= 235.0 && rh <= 340.0 && rs > 0.05 {
                        ref_magenta_count += 1;
                    }
                }
            }
            println!("Trellis region: FP magenta samples={}, Ref magenta samples={}", fp_magenta_count, ref_magenta_count);
        }
        "find-magenta" => {
            let dng_path = &args[2];
            let (bw, bh, binned_bytes) = decode_raw_preview_binned_buffer(dng_path).expect("decode binned");
            println!("Scanning binned image {}x{} for magenta (H in 260..340, S > 0.05)...", bw, bh);
            let mut magenta_count = 0;
            for by in (0..bh).step_by(10) {
                for bx in (0..bw).step_by(10) {
                    let b_idx = ((by * bw + bx) * 8) as usize;
                    let r_f16 = u16::from_ne_bytes([binned_bytes[b_idx], binned_bytes[b_idx + 1]]);
                    let g_f16 = u16::from_ne_bytes([binned_bytes[b_idx + 2], binned_bytes[b_idx + 3]]);
                    let b_f16 = u16::from_ne_bytes([binned_bytes[b_idx + 4], binned_bytes[b_idx + 5]]);
                    let r_lin = f16_to_f32(r_f16);
                    let g_lin = f16_to_f32(g_f16);
                    let b_lin = f16_to_f32(b_f16);
                    let tm = firstproof_core::color::agx_tone_map_vivid(firstproof_core::color::Color3::new(r_lin, g_lin, b_lin), 1.0);
                    let dr = tm.r().clamp(0.0, 1.0);
                    let dg = tm.g().clamp(0.0, 1.0);
                    let db = tm.b().clamp(0.0, 1.0);
                    let (h, s, _v) = firstproof_core::color::rgb_to_hsv(dr, dg, db);
                    if h >= 235.0 && h <= 340.0 && s > 0.05 {
                        magenta_count += 1;
                        if magenta_count <= 10 {
                            println!("Magenta at (x={}, y={}): FP AgxVivid RGB=({:.0}, {:.0}, {:.0}), H={:.1}°, S={:.3}, Rec2020=({:.3}, {:.3}, {:.3})",
                                bx * 2, by * 2, dr * 255.0, dg * 255.0, db * 255.0, h, s, r_lin, g_lin, b_lin);
                        }
                    }
                    if (bx * 2 == 560 && by * 2 == 160) || (bx * 2 == 360 && by * 2 == 600) {
                        let lr_path = Path::new("/Users/blikkie/Pictures/Lightroom Export/2016/2016-03-26/_MG_3717_edited.jpg");
                        if let Ok(lr_img) = image::open(lr_path).map(|i| i.to_rgb8()) {
                            let lp = lr_img.get_pixel(bx * 2 as u32, by * 2 as u32);
                            let (lh, ls, _lv) = firstproof_core::color::rgb_to_hsv(lp[0] as f32 / 255.0, lp[1] as f32 / 255.0, lp[2] as f32 / 255.0);

                            println!("POINT (x={}, y={}):", bx * 2, by * 2);
                            println!("  FP AgxVivid:   RGB=({:.0}, {:.0}, {:.0}), H={:.1}°, S={:.3}", dr * 255.0, dg * 255.0, db * 255.0, h, s);
                            println!("  LR (Reference):RGB=({}, {}, {}), H={:.1}°, S={:.3}", lp[0], lp[1], lp[2], lh, ls);
                        }
                    }
                }
            }
            println!("Total magenta sample points found: {}", magenta_count);
        }
        "audit-sky" => {
            let dng_path = &args[2];
            let ref_path = Path::new("target/mg3731_camera_ref.jpg");
            let ref_img = image::open(ref_path).ok().map(|i| i.to_rgb8());
            let (bw, bh, binned_bytes) = decode_raw_preview_binned_buffer(dng_path).expect("decode binned");
            println!("Auditing Sky Pixels in {} (binned {}x{})...", dng_path, bw, bh);
            let mut total_sky_points = 0;
            let mut agx_magenta_points = 0;
            let mut camstd_magenta_points = 0;
            let mut ref_magenta_points = 0;
            let mut sum_agx_hue = 0.0f32;
            let mut sum_camstd_hue = 0.0f32;
            let mut sum_ref_hue = 0.0f32;

            let std_curve = firstproof_core::color::dcp::DcpProfile::built_in_standard_tone_curve();

            for by in (0..bh / 2).step_by(5) {
                for bx in (0..bw).step_by(5) {
                    let b_idx = ((by * bw + bx) * 8) as usize;
                    let r_lin = f16_to_f32(u16::from_ne_bytes([binned_bytes[b_idx], binned_bytes[b_idx + 1]]));
                    let g_lin = f16_to_f32(u16::from_ne_bytes([binned_bytes[b_idx + 2], binned_bytes[b_idx + 3]]));
                    let b_lin = f16_to_f32(u16::from_ne_bytes([binned_bytes[b_idx + 4], binned_bytes[b_idx + 5]]));

                    // Sky pixel criteria in Linear Rec.2020: Blue dominant over Red, reasonable brightness
                    if b_lin > r_lin * 1.15 && b_lin > 0.08 {
                        total_sky_points += 1;

                        // 1. AgX Vivid
                        let tm = firstproof_core::color::agx_tone_map_vivid(firstproof_core::color::Color3::new(r_lin, g_lin, b_lin), 1.0);
                        let (h_agx, s_agx, _) = firstproof_core::color::rgb_to_hsv(tm.r().clamp(0.0, 1.0), tm.g().clamp(0.0, 1.0), tm.b().clamp(0.0, 1.0));
                        sum_agx_hue += h_agx;
                        if h_agx >= 230.0 && h_agx <= 340.0 && s_agx > 0.05 {
                            agx_magenta_points += 1;
                        }

                        // 2. CameraStandard
                        let c_srgb = firstproof_core::color::REC2020_TO_LINEAR_SRGB.multiply_vector(firstproof_core::color::Color3::new(r_lin, g_lin, b_lin));
                        let lum = 0.2126 * c_srgb.r().max(0.0) + 0.7152 * c_srgb.g().max(0.0) + 0.0722 * c_srgb.b().max(0.0);
                        let lum_curved = firstproof_core::color::evaluate_tone_curve(lum, &std_curve);
                        let gain = lum_curved / lum.max(1e-6);
                        let srgb_gamma = |c: f32| -> f32 {
                            if c <= 0.0031308 { c * 12.92 } else { 1.055 * c.max(0.0).powf(1.0 / 2.4) - 0.055 }
                        };
                        let dr_cs = srgb_gamma(c_srgb.r().max(0.0) * gain).clamp(0.0, 1.0);
                        let dg_cs = srgb_gamma(c_srgb.g().max(0.0) * gain).clamp(0.0, 1.0);
                        let db_cs = srgb_gamma(c_srgb.b().max(0.0) * gain).clamp(0.0, 1.0);
                        let (h_cs, s_cs, _) = firstproof_core::color::rgb_to_hsv(dr_cs, dg_cs, db_cs);
                        sum_camstd_hue += h_cs;
                        if h_cs >= 230.0 && h_cs <= 340.0 && s_cs > 0.05 {
                            camstd_magenta_points += 1;
                        }

                        // 3. Camera Reference JPEG
                        if let Some(ref r_img) = ref_img {
                            let rx = ((bx * 2) as f32 * (r_img.width() as f32 / (bw * 2) as f32)) as u32;
                            let ry = ((by * 2) as f32 * (r_img.height() as f32 / (bh * 2) as f32)) as u32;
                            let rp = r_img.get_pixel(rx.min(r_img.width() - 1), ry.min(r_img.height() - 1));
                            let (h_ref, s_ref, _) = firstproof_core::color::rgb_to_hsv(rp[0] as f32 / 255.0, rp[1] as f32 / 255.0, rp[2] as f32 / 255.0);
                            sum_ref_hue += h_ref;
                            if h_ref >= 230.0 && h_ref <= 340.0 && s_ref > 0.05 {
                                ref_magenta_points += 1;
                            }
                        }

                        if total_sky_points <= 5 || total_sky_points == 200 || total_sky_points == 27400 || total_sky_points == 39600 {
                            println!("Sky Point #{} (x={}, y={}): Rec2020=({:.3}, {:.3}, {:.3}) | AgxVivid H={:.1}°, S={:.2} | CamStd H={:.1}°, S={:.2}",
                                total_sky_points, bx * 2, by * 2, r_lin, g_lin, b_lin, h_agx, s_agx, h_cs, s_cs);
                        }
                    }
                }
            }
            if total_sky_points > 0 {
                let n = total_sky_points as f32;
                println!("\n=== SKY AUDIT RESULTS across {} sky sample points ===", total_sky_points);
                println!("AgX Vivid:        Mean Hue = {:.1}°, Points with H >= 230°: {} ({:.1}%)",
                    sum_agx_hue / n, agx_magenta_points, (agx_magenta_points as f32 / n) * 100.0);
                println!("CameraStandard:   Mean Hue = {:.1}°, Points with H >= 230°: {} ({:.1}%)",
                    sum_camstd_hue / n, camstd_magenta_points, (camstd_magenta_points as f32 / n) * 100.0);
                println!("Camera Ref JPEG:  Mean Hue = {:.1}°, Points with H >= 230°: {} ({:.1}%)",
                    sum_ref_hue / n, ref_magenta_points, (ref_magenta_points as f32 / n) * 100.0);

                println!("\n=== TINT SWEEP AUDIT (AgX Vivid & CameraStandard) ===");
                for test_tint in [-15.0, -12.0, -10.0, -8.0, -6.0, -4.0, -2.0, 0.0, 2.0, 5.0] {
                    let tint_factor = test_tint as f32 * 0.002f32;
                    let g_scale = (1.0f32 - tint_factor).max(0.01);
                    let r_scale = (1.0f32 + tint_factor * 0.5).max(0.01);
                    let b_scale = (1.0f32 + tint_factor * 0.5).max(0.01);

                    let mut agx_mag = 0;
                    let mut cam_mag = 0;
                    let mut agx_sum_h = 0.0f32;
                    let mut cam_sum_h = 0.0f32;

                    for by in (0..bh / 2).step_by(10) {
                        for bx in (0..bw).step_by(10) {
                            let b_idx = ((by * bw + bx) * 8) as usize;
                            let r_lin = f16_to_f32(u16::from_ne_bytes([binned_bytes[b_idx], binned_bytes[b_idx + 1]]));
                            let g_lin = f16_to_f32(u16::from_ne_bytes([binned_bytes[b_idx + 2], binned_bytes[b_idx + 3]]));
                            let b_lin = f16_to_f32(u16::from_ne_bytes([binned_bytes[b_idx + 4], binned_bytes[b_idx + 5]]));

                            if b_lin > r_lin * 1.15 && b_lin > 0.08 {
                                let r_adj = r_lin * r_scale;
                                let g_adj = g_lin * g_scale;
                                let b_adj = b_lin * b_scale;

                                // AgX Vivid
                                let tm = firstproof_core::color::agx_tone_map_vivid(firstproof_core::color::Color3::new(r_adj, g_adj, b_adj), 1.0);
                                let (h_a, s_a, _) = firstproof_core::color::rgb_to_hsv(tm.r().clamp(0.0, 1.0), tm.g().clamp(0.0, 1.0), tm.b().clamp(0.0, 1.0));
                                agx_sum_h += h_a;
                                if h_a >= 230.0 && h_a <= 340.0 && s_a > 0.05 {
                                    agx_mag += 1;
                                }

                                // CamStd
                                let c_srgb = firstproof_core::color::REC2020_TO_LINEAR_SRGB.multiply_vector(firstproof_core::color::Color3::new(r_adj, g_adj, b_adj));
                                let lum = 0.2126 * c_srgb.r().max(0.0) + 0.7152 * c_srgb.g().max(0.0) + 0.0722 * c_srgb.b().max(0.0);
                                let lum_curved = firstproof_core::color::evaluate_tone_curve(lum, &std_curve);
                                let gain = lum_curved / lum.max(1e-6);
                                let srgb_gamma = |c: f32| -> f32 {
                                    if c <= 0.0031308 { c * 12.92 } else { 1.055 * c.max(0.0).powf(1.0 / 2.4) - 0.055 }
                                };
                                let (h_c, s_c, _) = firstproof_core::color::rgb_to_hsv(
                                    srgb_gamma(c_srgb.r().max(0.0) * gain).clamp(0.0, 1.0),
                                    srgb_gamma(c_srgb.g().max(0.0) * gain).clamp(0.0, 1.0),
                                    srgb_gamma(c_srgb.b().max(0.0) * gain).clamp(0.0, 1.0),
                                );
                                cam_sum_h += h_c;
                                if h_c >= 230.0 && h_c <= 340.0 && s_c > 0.05 {
                                    cam_mag += 1;
                                }
                            }
                        }
                    }
                    let count = (n / 4.0).max(1.0);
                    println!("Tint {:+5.1}: AgX Mean H={:5.1}°, Mag={:4} ({:4.1}%) | CamStd Mean H={:5.1}°, Mag={:4} ({:4.1}%)",
                        test_tint, agx_sum_h / count, agx_mag, (agx_mag as f32 / count) * 100.0,
                        cam_sum_h / count, cam_mag, (cam_mag as f32 / count) * 100.0);
                }
            }
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
                    let d50_to_d65 = firstproof_core::color::bradford_adaptation_matrix((0.3457, 0.3585), (0.3127, 0.3290));
                    let xyz_to_rec2020 = firstproof_core::color::ColorSpacePrimaries::REC2020.xyz_to_rgb_matrix();
                    let d50_to_rec2020 = xyz_to_rec2020.multiply_matrix(&d50_to_d65);
                    println!("Universal Bradford XYZ_D50 -> Linear Rec.2020 Matrix:\n{:?}", d50_to_rec2020.0);
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


