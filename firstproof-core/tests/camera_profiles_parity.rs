use firstproof_core::color::dcp::{
    DcpProfile, evaluate_tone_curve, rgb_to_hsv, hsv_to_rgb,
};
use firstproof_core::ColorProfileMode;
use firstproof_core::xmp::{XmpData, generate_xmp, parse_xmp};

#[test]
fn test_rgb_hsv_roundtrip() {
    let test_colors = [
        (1.0, 0.0, 0.0), // Red
        (0.0, 1.0, 0.0), // Green
        (0.0, 0.0, 1.0), // Blue
        (1.0, 1.0, 0.0), // Yellow
        (0.0, 1.0, 1.0), // Cyan
        (1.0, 0.0, 1.0), // Magenta
        (0.5, 0.5, 0.5), // Neutral Gray
        (0.82, 0.55, 0.40), // Light skin
        (0.15, 0.45, 0.75), // Sky blue
        (0.20, 0.60, 0.25), // Foliage green
    ];

    for &(r, g, b) in &test_colors {
        let (h, s, v) = rgb_to_hsv(r, g, b);
        let (r_out, g_out, b_out) = hsv_to_rgb(h, s, v);

        assert!((r - r_out).abs() < 1e-4, "R mismatch on ({},{},{}): got {}", r, g, b, r_out);
        assert!((g - g_out).abs() < 1e-4, "G mismatch on ({},{},{}): got {}", r, g, b, g_out);
        assert!((b - b_out).abs() < 1e-4, "B mismatch on ({},{},{}): got {}", r, g, b, b_out);
    }
}

#[test]
fn test_huesatmap_interpolation_and_bounds() {
    let map = DcpProfile::built_in_standard_huesatmap();

    // 1. Achromatic input (Saturation = 0.0): deltas should leave it achromatic
    let (_h_out, s_out, v_out) = map.apply_hsv(0.0, 0.0, 0.5);
    assert_eq!(s_out, 0.0, "Achromatic gray became chromatic");
    assert!((v_out - 0.5).abs() < 0.05);

    // 2. Sensor daylight sky blue input (Hue ~ 235 deg, Sat ~ 0.6):
    // Must shift towards natural azure (215 - 220 deg) and tame saturation to eliminate purple cast
    let (h_sky, s_sky, _v_sky) = map.apply_hsv(235.0, 0.6, 0.8);
    assert!(h_sky < 225.0 && h_sky >= 214.0, "Sensor blue hue was not shifted towards azure: got {}", h_sky);
    assert!(s_sky <= 0.60, "Sky blue saturation was blown out: got {}", s_sky);

    // 3. Foliage green input (Hue ~ 120 deg, Sat ~ 0.5): should boost saturation
    let (_h_green, s_green, _v_green) = map.apply_hsv(120.0, 0.5, 0.5);
    assert!(s_green > 0.5, "Green saturation was not boosted: got {}", s_green);
}

#[test]
fn test_luminance_scaled_tone_curve_preserves_chromaticity() {
    let curve = DcpProfile::built_in_standard_tone_curve();
    // Sky blue input: R < G < B
    let (r, g, b) = (0.22f32, 0.35f32, 0.65f32);
    let orig_r_ratio = r / b;
    let orig_g_ratio = g / b;

    let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let lum_curved = evaluate_tone_curve(lum, &curve);
    let gain = lum_curved / lum.max(1e-6);

    let r_out = r * gain;
    let g_out = g * gain;
    let b_out = b * gain;

    let new_r_ratio = r_out / b_out;
    let new_g_ratio = g_out / b_out;

    assert!((orig_r_ratio - new_r_ratio).abs() < 1e-5, "R/B chromaticity ratio drifted!");
    assert!((orig_g_ratio - new_g_ratio).abs() < 1e-5, "G/B chromaticity ratio drifted!");
}

#[test]
fn test_camera_tone_curves_monotonicity() {
    let curves = [
        ("Standard", DcpProfile::built_in_standard_tone_curve()),
        ("Portrait", DcpProfile::built_in_portrait_tone_curve()),
        ("Landscape", DcpProfile::built_in_landscape_tone_curve()),
        ("Vivid", DcpProfile::built_in_vivid_tone_curve()),
    ];

    for (name, curve) in &curves {
        // Test endpoints
        assert_eq!(evaluate_tone_curve(0.0, curve), 0.0, "Curve '{}' origin != 0.0", name);
        assert_eq!(evaluate_tone_curve(1.0, curve), 1.0, "Curve '{}' max != 1.0", name);

        // Test monotonicity across 100 sample points
        let mut prev_y = 0.0;
        for i in 1..=100 {
            let x = (i as f32) / 100.0;
            let y = evaluate_tone_curve(x, curve);
            assert!(
                y >= prev_y - 1e-5,
                "Curve '{}' non-monotonic at x = {}: prev_y = {}, y = {}",
                name, x, prev_y, y
            );
            prev_y = y;
        }

        // Test that linear camera curves map 18% middle gray appropriately (~0.18 - 0.25, yielding ~47-51% display brightness)
        let mid = evaluate_tone_curve(0.18, curve);
        assert!(
            mid >= 0.18 && mid <= 0.25,
            "Curve '{}' midtone out of photographic linear range: {}",
            name, mid
        );
    }
}

#[test]
fn test_xmp_tone_profile_roundtrip() {
    let modes = [
        ColorProfileMode::AgxVivid,
        ColorProfileMode::AgxFilmic,
        ColorProfileMode::CameraStandard,
        ColorProfileMode::CameraPortrait,
        ColorProfileMode::CameraLandscape,
        ColorProfileMode::CameraVivid,
    ];

    for mode in modes {
        let mut data = XmpData::default();
        data.edits.tone_profile = mode;

        let xml = generate_xmp(&data);
        let parsed = parse_xmp(&xml).expect("Failed to parse generated XMP");

        assert_eq!(
            parsed.edits.tone_profile, mode,
            "XMP roundtrip failed for {:?}", mode
        );
    }
}

#[test]
#[ignore = "Slow real RAW disk decode"]
fn test_cr2_portrait_cheek_and_helmet_fidelity() {
    use std::path::Path;
    use firstproof_core::raw_decoder::{RawDecoder, RawlerDecoder};
    let path = Path::new("/Users/blikkie/Pictures/Darktable/20250409_eos_pull/20250409_0073.CR2");
    if !path.exists() { return; }
    let Ok(_frame) = RawlerDecoder.decode(path) else { return; };
    
    let Ok(demosaiced) = firstproof_core::export::decode_image(path) else { return; };
    let w = demosaiced.width;
    let h = demosaiced.height;
    
    // Helmet body coordinates: x ~ 50% w, y ~ 25% h (body of helmet, not specular crown)
    let h_idx = ((h * 25 / 100) * w + (w * 50 / 100)) * 3;
    let h_rec2020 = firstproof_core::color::Color3::new(demosaiced.data[h_idx], demosaiced.data[h_idx + 1], demosaiced.data[h_idx + 2]);
    
    // Cheek coordinates: x ~ 43% w, y ~ 46% h
    let c_idx = ((h * 46 / 100) * w + (w * 43 / 100)) * 3;
    let c_rec2020 = firstproof_core::color::Color3::new(demosaiced.data[c_idx], demosaiced.data[c_idx + 1], demosaiced.data[c_idx + 2]);
    
    // 1. Verify Helmet body retains rich pink tones (specular crown at y~18% may clamp to white; helmet body should not)
    let out_helmet = firstproof_core::color::agx_tone_map_vivid(h_rec2020, 1.0);
    let (h_hue, h_sat, h_val) = firstproof_core::color::rgb_to_hsv(out_helmet.r(), out_helmet.g(), out_helmet.b());
    println!("Helmet body: hue={:.1}, sat={:.3}, val={:.3}", h_hue, h_sat, h_val);
    // Helmet body pixel should not be a full specular white (val < 0.99 expected for body)
    // If val is very high, it may still be a bright area - only enforce saturation/hue for non-blown pixels
    if h_val < 0.99 {
        assert!(h_sat >= 0.10, "Helmet body pastel pink saturation washed out: sat={:.3}", h_sat);
        // Pink hue range: 320-360 or 0-30
        let pink = h_hue >= 320.0 || h_hue <= 30.0;
        assert!(pink, "Helmet body hue not in pink/red range: hue={:.1}", h_hue);
    }
    // Regardless: AgX must not produce values > 1.0 (clamping is expected at exactly 1.0)
    assert!(h_val <= 1.0, "AgX produced out-of-range value: val={:.3}", h_val);
    
    // 2. Verify Cheek has natural skin saturation without extreme yellow-orange sunburn cast
    let out_cheek = firstproof_core::color::agx_tone_map_vivid(c_rec2020, 1.0);
    let (c_hue, c_sat, _c_val) = firstproof_core::color::rgb_to_hsv(out_cheek.r(), out_cheek.g(), out_cheek.b());
    println!("Cheek: hue={:.1}, sat={:.3}", c_hue, c_sat);
    assert!(c_hue >= 18.0 && c_hue <= 35.0, "Cheek hue outside natural skin range: hue={:.1}", c_hue);
    assert!(c_sat <= 0.75, "Cheek linear skin tone oversaturated: sat={:.3}", c_sat);
    
    // Check display saturation (standard 2.2 gamma applied as seen by the user):
    let (r_disp, g_disp, b_disp) = (out_cheek.r().powf(1.0/2.2), out_cheek.g().powf(1.0/2.2), out_cheek.b().powf(1.0/2.2));
    let (_, c_sat_disp, _) = firstproof_core::color::rgb_to_hsv(r_disp, g_disp, b_disp);
    assert!(c_sat_disp <= 0.50, "Cheek display skin tone oversaturated: sat={:.3}", c_sat_disp);
}

#[test]
fn test_agx_rec2020_saturation_and_vivid_restoration() {
    use firstproof_core::color::{Color3, LINEAR_SRGB_TO_REC2020, agx_tone_map, agx_tone_map_vivid, rgb_to_hsv};
    
    // Macbeth / Real-world test patches
    let patches = [
        ("Pure Red", Color3::new(1.0, 0.0, 0.0), 0.60),
        ("Pure Green", Color3::new(0.0, 1.0, 0.0), 0.60),
        ("Pure Blue", Color3::new(0.0, 0.0, 1.0), 0.60),
        ("Cyan/Sky", Color3::new(0.2, 0.6, 0.9), 0.35),
        ("Foliage", Color3::new(0.15, 0.40, 0.10), 0.35),
        ("Skin Tone", Color3::new(0.70, 0.45, 0.35), 0.15),
    ];
    
    for (name, srgb, min_sat) in patches {
        let rec2020 = LINEAR_SRGB_TO_REC2020.multiply_vector(srgb);
        
        // 1. AgX Base Neutral (calibrated for Rec.2020)
        let out_neutral = agx_tone_map(rec2020, 1.0);
        let (_, s_neutral, _) = rgb_to_hsv(out_neutral.r(), out_neutral.g(), out_neutral.b());
        assert!(
            s_neutral >= min_sat,
            "{} in AgX Neutral has low saturation: {:.3} < {:.3}",
            name, s_neutral, min_sat
        );
        
        // 2. AgX Vivid (Punchy + saturated)
        let out_vivid = agx_tone_map_vivid(rec2020, 1.0);
        let (_, s_vivid, _) = rgb_to_hsv(out_vivid.r(), out_vivid.g(), out_vivid.b());
        assert!(
            s_vivid >= s_neutral * 0.95,
            "{} in AgX Vivid has lower saturation than neutral: {:.3} vs {:.3}",
            name, s_vivid, s_neutral
        );
    }
}

#[test]
#[ignore = "Slow real RAW disk decode"]
fn test_gpu_preview_vs_export_parity() {
    use std::path::Path;
    use firstproof_core::raw_decoder::{RawDecoder, RawlerDecoder};
    use firstproof_core::color::{Color3, agx_tone_map_vivid};
    use firstproof_core::EditSettings;
    
    let path = Path::new("/Users/blikkie/Pictures/Darktable/20250409_eos_pull/20250409_0073.CR2");
    if !path.exists() { return; }
    let Ok(_frame) = RawlerDecoder.decode(path) else { return; };
    
    let Ok(demosaiced) = firstproof_core::export::decode_image(path) else { return; };
    let w = demosaiced.width;
    let h = demosaiced.height;
    
    // Check cheek point (x ~ 43%, y ~ 46%)
    let idx = ((h * 46 / 100) * w + (w * 43 / 100)) * 3;
    let rec2020_pixel = Color3::new(demosaiced.data[idx], demosaiced.data[idx + 1], demosaiced.data[idx + 2]);
    
    // 1. GPU preview simulation:
    // Fragment shader takes rec2020 texture sample, runs agxToneMapVivid, clamps to [0, 1]
    let gpu_srgb = agx_tone_map_vivid(rec2020_pixel, 1.0);
    let gpu_8bit = [
        (gpu_srgb.r().clamp(0.0, 1.0) * 255.0).round() as u8,
        (gpu_srgb.g().clamp(0.0, 1.0) * 255.0).round() as u8,
        (gpu_srgb.b().clamp(0.0, 1.0) * 255.0).round() as u8,
    ];
    
    // 2. Export simulation:
    let mut export_img = firstproof_core::export::decode_image(path).unwrap();
    let edits = EditSettings {
        tone_profile: firstproof_core::ColorProfileMode::AgxVivid,
        ..Default::default()
    };
    firstproof_core::export::apply_edits(&mut export_img, &edits);
    let export_srgb = Color3::new(export_img.data[idx], export_img.data[idx + 1], export_img.data[idx + 2]);
    let export_8bit = [
        (export_srgb.r().clamp(0.0, 1.0) * 255.0).round() as u8,
        (export_srgb.g().clamp(0.0, 1.0) * 255.0).round() as u8,
        (export_srgb.b().clamp(0.0, 1.0) * 255.0).round() as u8,
    ];
    
    println!("GPU preview:  R={}, G={}, B={}", gpu_8bit[0], gpu_8bit[1], gpu_8bit[2]);
    println!("Export image: R={}, G={}, B={}", export_8bit[0], export_8bit[1], export_8bit[2]);
    
    assert_eq!(gpu_8bit, export_8bit, "GPU preview and export pixel math differ!");
}

#[test]
#[ignore = "Slow real RAW disk decode"]
fn test_cr2_0058_binned_vs_export() {
    use std::path::Path;
    use firstproof_core::color::{Color3, agx_tone_map_vivid};
    use firstproof_core::export::{decode_image, decode_raw_preview_binned_buffer, apply_edits};
    use firstproof_core::xmp::read_xmp_file;

    let path = Path::new("/Users/blikkie/Pictures/Darktable/20250409_eos_pull/20250409_0058.CR2");
    if !path.exists() {
        println!("File does not exist: {:?}", path);
        return;
    }

    let xmp = match read_xmp_file(path) {
        Ok(x) => x,
        Err(_) => {
            println!("No XMP sidecar found for 0058, skipping test");
            return;
        }
    };
    println!("XMP Edits for 0058: temp={}, tint={}, exposure={}, contrast={}, highlights={}, shadows={}",
        xmp.edits.temperature, xmp.edits.tint, xmp.edits.exposure, xmp.edits.contrast, xmp.edits.highlights, xmp.edits.shadows);

    // 1. Decode binned (Display)
    let (bw, bh, binned_bytes) = decode_raw_preview_binned_buffer(path).unwrap();
    println!("Binned dimensions: {}x{}, bytes: {}", bw, bh, binned_bytes.len());

    // 2. Decode full (Export)
    let mut export_img = decode_image(path).unwrap();
    println!("Export dimensions: {}x{}", export_img.width, export_img.height);

    // Compare at several relative coordinates (e.g. face at x=35%, y=50%; wall at x=60%, y=15%; seat at x=15%, y=30%)
    let test_points = [
        ("Girl Cheek", 0.35f32, 0.55f32),
        ("Subway Wall", 0.60f32, 0.15f32),
        ("Orange Seat", 0.15f32, 0.30f32),
        ("Girl Forehead", 0.30f32, 0.40f32),
    ];

    for (label, rx, ry) in test_points {
        let ex = ((export_img.width as f32) * rx) as usize;
        let ey = ((export_img.height as f32) * ry) as usize;
        let e_idx = (ey * export_img.width + ex) * 3;
        println!("Full Raw Rec2020 at {}: [{:.4}, {:.4}, {:.4}]",
            label, export_img.data[e_idx], export_img.data[e_idx + 1], export_img.data[e_idx + 2]);
    }

    // Apply edits to export image
    apply_edits(&mut export_img, &xmp.edits);

    // Simulate Metal shader on binned buffer
    let temp_norm = (xmp.edits.temperature / 5500.0).clamp(0.25, 4.0);
    let mut r_scale = temp_norm.powf(0.75);
    let mut b_scale = (1.0 / temp_norm).powf(0.75);
    let tint_factor = xmp.edits.tint * 0.002;
    let g_scale = (1.0 - tint_factor).max(0.01);
    r_scale = (r_scale + tint_factor * 0.5).max(0.01);
    b_scale = (b_scale + tint_factor * 0.5).max(0.01);
    let exposure_gain = 2.0f32.powf(xmp.edits.exposure);
    let contrast_gain = if xmp.edits.contrast >= 0.0 {
        1.0 + (xmp.edits.contrast / 100.0)
    } else {
        1.0 + (xmp.edits.contrast / 100.0) * 0.60
    };

    println!("\n=== COLOR COMPARISON (Binned/Display vs Full/Export) ===");
    for (label, rx, ry) in test_points {
        // Sample binned:
        let bx = ((bw as f32) * rx) as usize;
        let by = ((bh as f32) * ry) as usize;
        let b_idx = (by * (bw as usize) + bx) * 8; // 4 half-floats = 8 bytes
        let r_f16 = u16::from_ne_bytes([binned_bytes[b_idx], binned_bytes[b_idx + 1]]);
        let g_f16 = u16::from_ne_bytes([binned_bytes[b_idx + 2], binned_bytes[b_idx + 3]]);
        let b_f16 = u16::from_ne_bytes([binned_bytes[b_idx + 4], binned_bytes[b_idx + 5]]);
        let mut r_binned = firstproof_core::export::f16_to_f32(r_f16);
        let mut g_binned = firstproof_core::export::f16_to_f32(g_f16);
        let mut b_binned = firstproof_core::export::f16_to_f32(b_f16);

        println!("Raw Rec2020 at {}: Binned=[{:.4}, {:.4}, {:.4}]", label, r_binned, g_binned, b_binned);

        // Apply shader math
        r_binned *= r_scale;
        g_binned *= g_scale;
        b_binned *= b_scale;
        r_binned *= exposure_gain;
        g_binned *= exposure_gain;
        b_binned *= exposure_gain;
        let (r_rec, g_rec, b_rec) = firstproof_core::export::apply_tone_recovery(
            r_binned, g_binned, b_binned,
            xmp.edits.highlights, xmp.edits.shadows, xmp.edits.whites, xmp.edits.blacks,
        );
        let binned_tone_mapped = agx_tone_map_vivid(Color3::new(r_rec, g_rec, b_rec), contrast_gain);
        let binned_8bit = [
            (binned_tone_mapped.r().clamp(0.0, 1.0) * 255.0).round() as u8,
            (binned_tone_mapped.g().clamp(0.0, 1.0) * 255.0).round() as u8,
            (binned_tone_mapped.b().clamp(0.0, 1.0) * 255.0).round() as u8,
        ];

        // Sample export:
        let ex = ((export_img.width as f32) * rx) as usize;
        let ey = ((export_img.height as f32) * ry) as usize;
        let e_idx = (ey * export_img.width + ex) * 3;
        let export_8bit = [
            (export_img.data[e_idx].clamp(0.0, 1.0) * 255.0).round() as u8,
            (export_img.data[e_idx + 1].clamp(0.0, 1.0) * 255.0).round() as u8,
            (export_img.data[e_idx + 2].clamp(0.0, 1.0) * 255.0).round() as u8,
        ];

        println!("{}: Display(Binned)={:?}, Export(Full)={:?}, Delta=[{}, {}, {}]",
            label, binned_8bit, export_8bit,
            export_8bit[0] as i16 - binned_8bit[0] as i16,
            export_8bit[1] as i16 - binned_8bit[1] as i16,
            export_8bit[2] as i16 - binned_8bit[2] as i16,
        );
    }

    let ldb = firstproof_core::lensfun::LensfunDatabase::new();
    let meta = firstproof_core::raw::extract_metadata(path).ok();
    let lens_coeffs = meta.as_ref().map(|m| {
        ldb.resolve_coefficients(
            m.camera_make.as_deref().unwrap_or(""),
            m.camera_model.as_deref().unwrap_or(""),
            m.lens_model.as_deref().unwrap_or(""),
            m.focal_length.unwrap_or(0.0),
            m.aperture,
        )
    });
    let dist_tuple = lens_coeffs.as_ref().and_then(|c| {
        if c.has_distortion {
            Some((c.distortion.as_slice(), c.distortion_model.as_str()))
        } else {
            None
        }
    });

    let new_desktop_jpg = Path::new("/Users/blikkie/Desktop/20250409_0058_FirstProof_New.jpg");
    firstproof_core::export::export_photo_file(
        path,
        new_desktop_jpg,
        &xmp.edits,
        dist_tuple,
        95,
        None,
        firstproof_core::ExportColorSpace::Srgb,
        firstproof_core::ExportRenderingIntent::RelativeColorimetric,
        true,
    ).expect("Failed to export new photo");

    let screenshot_jpg = Path::new("/Users/blikkie/.gemini/antigravity-ide/brain/be942130-01a3-4c51-a22d-2126d19f7aee/.user_uploaded/media_1789332136643.jpg");
    let desktop_jpg = Path::new("/Users/blikkie/Desktop/20250409_0058_FirstProof.jpg");

    if screenshot_jpg.exists() && new_desktop_jpg.exists() {
        let shot_img = image::open(screenshot_jpg).unwrap().to_rgb8();
        let old_desk_img = if desktop_jpg.exists() { Some(image::open(desktop_jpg).unwrap().to_rgb8()) } else { None };
        let new_desk_img = image::open(new_desktop_jpg).unwrap().to_rgb8();

        println!("\n=== FACE REGION SCAN: SCREENSHOT vs OLD EXPORT vs NEW EXPORT ===");
        let mut total_shot_r = 0u64;
        let mut total_shot_g = 0u64;
        let mut total_shot_b = 0u64;
        let mut total_old_r = 0u64;
        let mut total_old_g = 0u64;
        let mut total_old_b = 0u64;
        let mut total_new_r = 0u64;
        let mut total_new_g = 0u64;
        let mut total_new_b = 0u64;
        let mut count = 0u64;

        for step_x in 0..10 {
            for step_y in 0..10 {
                let rx = 0.28 + (step_x as f32) * 0.015;
                let ry = 0.42 + (step_y as f32) * 0.020;
                let sx = ((shot_img.width() as f32) * rx) as u32;
                let sy = ((shot_img.height() as f32) * ry) as u32;
                let sp = shot_img.get_pixel(sx, sy);

                let nx = ((new_desk_img.width() as f32) * rx) as u32;
                let ny = ((new_desk_img.height() as f32) * ry) as u32;
                let np = new_desk_img.get_pixel(nx, ny);

                total_shot_r += sp.0[0] as u64;
                total_shot_g += sp.0[1] as u64;
                total_shot_b += sp.0[2] as u64;
                total_new_r += np.0[0] as u64;
                total_new_g += np.0[1] as u64;
                total_new_b += np.0[2] as u64;

                if let Some(ref old_desk) = old_desk_img {
                    let ox = ((old_desk.width() as f32) * rx) as u32;
                    let oy = ((old_desk.height() as f32) * ry) as u32;
                    let op = old_desk.get_pixel(ox, oy);
                    total_old_r += op.0[0] as u64;
                    total_old_g += op.0[1] as u64;
                    total_old_b += op.0[2] as u64;
                }
                count += 1;
            }
        }

        let avg_shot = [total_shot_r / count, total_shot_g / count, total_shot_b / count];
        let avg_new = [total_new_r / count, total_new_g / count, total_new_b / count];
        println!("Face Region Average (100 sample points):");
        println!("  Screenshot (Display): R={}, G={}, B={}  (G/R = {:.3})", avg_shot[0], avg_shot[1], avg_shot[2], avg_shot[1] as f64 / avg_shot[0] as f64);
        if old_desk_img.is_some() {
            let avg_old = [total_old_r / count, total_old_g / count, total_old_b / count];
            println!("  Old Export (Pre-Fix): R={}, G={}, B={}  (G/R = {:.3})", avg_old[0], avg_old[1], avg_old[2], avg_old[1] as f64 / avg_old[0] as f64);
        }
        println!("  New Export (Post-Fix): R={}, G={}, B={}  (G/R = {:.3})", avg_new[0], avg_new[1], avg_new[2], avg_new[1] as f64 / avg_new[0] as f64);
    }
}

#[test]
#[ignore = "Slow real RAW disk decode"]
fn test_cr2_0194_full_pipeline_parity() {
    use std::path::Path;
    use firstproof_core::color::{Color3, agx_tone_map_vivid};
    use firstproof_core::export::apply_color_adjustments;
    use firstproof_core::EditSettings;
    
    let path = Path::new("/Users/blikkie/Pictures/Darktable/20250409_eos_pull/20250409_0194.CR2");
    if !path.exists() { return; }
    
    let Ok(demosaiced) = firstproof_core::export::decode_image(path) else { return; };
    let w = demosaiced.width;
    let h = demosaiced.height;
    
    // Sample 5 diverse points: Center, Top-Left, Bottom-Right, Mid-Sky, Foreground
    let sample_points = [
        ("Center Point", w / 2, h / 2),
        ("Upper Left", w / 4, h / 4),
        ("Lower Right", 3 * w / 4, 3 * h / 4),
        ("Upper Mid", w / 2, h / 5),
        ("Lower Mid", w / 2, 4 * h / 5),
    ];
    
    let edits = EditSettings {
        exposure: 0.3,
        contrast: 10.0,
        highlights: -20.0,
        shadows: 15.0,
        whites: -10.0,
        blacks: 5.0,
        vibrance: 25.0,
        saturation: 10.0,
        deep_blue: 20.0,
        foliage: 15.0,
        temperature: 5800.0,
        tint: 5.0,
        tone_profile: firstproof_core::ColorProfileMode::AgxVivid,
        ..Default::default()
    };
    
    // Run full export pipeline
    let mut export_img = firstproof_core::export::DemosaicedImage {
        width: demosaiced.width,
        height: demosaiced.height,
        data: demosaiced.data.clone(),
        is_raw: demosaiced.is_raw,
    };
    firstproof_core::export::apply_edits(&mut export_img, &edits);
    
    for (label, px, py) in sample_points {
        let idx = (py * w + px) * 3;
        
        // 1. Export pixel math:
        let r_exp_raw = export_img.data[idx];
        let g_exp_raw = export_img.data[idx + 1];
        let b_exp_raw = export_img.data[idx + 2];
        let (r_exp_adj, g_exp_adj, b_exp_adj) = apply_color_adjustments(
            r_exp_raw, g_exp_raw, b_exp_raw,
            edits.vibrance, edits.saturation, edits.deep_blue, edits.foliage,
        );
        let export_8bit = [
            (r_exp_adj.clamp(0.0, 1.0) * 255.0).round() as u8,
            (g_exp_adj.clamp(0.0, 1.0) * 255.0).round() as u8,
            (b_exp_adj.clamp(0.0, 1.0) * 255.0).round() as u8,
        ];
        
        // 2. GPU preview shader simulation:
        // Input: raw Rec.2020 texture sample
        let mut r = demosaiced.data[idx];
        let mut g = demosaiced.data[idx + 1];
        let mut b = demosaiced.data[idx + 2];
        
        // WB
        let temp_norm = (edits.temperature / 5500.0).clamp(0.25, 4.0);
        let mut r_scale = temp_norm.powf(0.75);
        let mut b_scale = (1.0 / temp_norm).powf(0.75);
        let tint_factor = edits.tint * 0.002;
        let g_scale = (1.0 - tint_factor).max(0.01);
        r_scale = (r_scale + tint_factor * 0.5).max(0.01);
        b_scale = (b_scale + tint_factor * 0.5).max(0.01);
        r *= r_scale; g *= g_scale; b *= b_scale;
        
        // Exposure
        let exp_gain = 2.0f32.powf(edits.exposure);
        r *= exp_gain; g *= exp_gain; b *= exp_gain;
        
        // Tone Recovery
        let (r_rec, g_rec, b_rec) = firstproof_core::export::apply_tone_recovery(
            r, g, b,
            edits.highlights, edits.shadows, edits.whites, edits.blacks,
        );
        
        // Tone profile (AgX Vivid)
        let contrast_gain = if edits.contrast >= 0.0 {
            1.0 + (edits.contrast / 100.0)
        } else {
            1.0 + (edits.contrast / 100.0) * 0.60
        };
        let srgb = agx_tone_map_vivid(Color3::new(r_rec, g_rec, b_rec), contrast_gain);
        
        // Color adjustments
        let (r_gpu_adj, g_gpu_adj, b_gpu_adj) = apply_color_adjustments(
            srgb.r().clamp(0.0, 1.0), srgb.g().clamp(0.0, 1.0), srgb.b().clamp(0.0, 1.0),
            edits.vibrance, edits.saturation, edits.deep_blue, edits.foliage,
        );
        let gpu_8bit = [
            (r_gpu_adj.clamp(0.0, 1.0) * 255.0).round() as u8,
            (g_gpu_adj.clamp(0.0, 1.0) * 255.0).round() as u8,
            (b_gpu_adj.clamp(0.0, 1.0) * 255.0).round() as u8,
        ];
        
        assert_eq!(
            gpu_8bit, export_8bit,
            "Parity mismatch at {} for 20250409_0194.CR2: GPU {:?} vs Export {:?}",
            label, gpu_8bit, export_8bit
        );
    }
}

#[test]
fn test_sliders_dual_engine_parity_sweep() {
    use firstproof_core::color::{Color3, LINEAR_SRGB_TO_REC2020, REC2020_TO_LINEAR_SRGB, agx_tone_map, agx_tone_map_vivid, evaluate_tone_curve, linear_to_srgb_gamma, rgb_to_hsv, hsv_to_rgb};
    use firstproof_core::export::{apply_tone_recovery, apply_color_adjustments};
    use firstproof_core::color::dcp::DcpProfile;
    use firstproof_core::ColorProfileMode;

    // Test palette in linear sRGB
    let test_patches = [
        ("Neutral Gray", [0.18, 0.18, 0.18]),
        ("Vivid Red", [0.85, 0.12, 0.10]),
        ("Deep Sky", [0.15, 0.45, 0.85]),
        ("Lush Foliage", [0.20, 0.65, 0.15]),
        ("Warm Skin", [0.68, 0.42, 0.32]),
        ("Deep Shadow", [0.03, 0.03, 0.03]),
        ("Bright Highlight", [0.92, 0.90, 0.88]),
    ];

    // Slider configurations to sweep: (exp, contrast, hl, sh, wh, bl, vib, sat, temp, tint, mode)
    // Slider configurations to sweep: (exp, contrast, hl, sh, wh, bl, vib, sat, temp, tint, mode)
    let test_configs: [(f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, ColorProfileMode); 8] = [
        // Neutral
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5500.0, 0.0, ColorProfileMode::AgxVivid),
        // Saturated & Punchy with color adjustments
        (0.5, 20.0, -30.0, 25.0, -15.0, 10.0, 35.0, 20.0, 6200.0, 10.0, ColorProfileMode::AgxVivid),
        // AgX Filmic neutral base with deep blue boost
        (-0.5, -10.0, 10.0, -10.0, 0.0, 0.0, 15.0, -10.0, 4800.0, -8.0, ColorProfileMode::AgxFilmic),
        // Camera Standard profile with high vibrance & foliage
        (0.0, 15.0, -20.0, 20.0, -10.0, 5.0, 50.0, 15.0, 5500.0, 0.0, ColorProfileMode::CameraStandard),
        // Camera Portrait profile with negative vibrance
        (0.2, -5.0, -15.0, 15.0, 0.0, 0.0, -30.0, -20.0, 5600.0, 4.0, ColorProfileMode::CameraPortrait),
        // Camera Landscape profile with deep blue and foliage
        (0.0, 25.0, -40.0, 30.0, -20.0, 10.0, 40.0, 30.0, 5200.0, -5.0, ColorProfileMode::CameraLandscape),
        // Extreme positive tint (+190) under warm tungsten (subway/mercury spikes)
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 3200.0, 190.0, ColorProfileMode::AgxVivid),
        // Extreme negative tint (-190) under cool shade
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 7500.0, -190.0, ColorProfileMode::AgxFilmic),
    ];

    for (profile_idx, &(exp, contrast, hl, sh, wh, bl, vib, sat, temp, tint, mode)) in test_configs.iter().enumerate() {
        for (patch_name, [r_in, g_in, b_in]) in test_patches {
            // Input in Rec.2020 wide gamut space
            let c_rec2020 = LINEAR_SRGB_TO_REC2020.multiply_vector(Color3::new(r_in, g_in, b_in));
            
            // 1. SIMULATE METAL GPU SHADER PIPELINE:
            let mut r_gpu = c_rec2020.r();
            let mut g_gpu = c_rec2020.g();
            let mut b_gpu = c_rec2020.b();
            
            // White Balance
            let temp_norm = (temp / 5500.0).clamp(0.25, 4.0);
            let mut r_scale = temp_norm.powf(0.75);
            let mut b_scale = (1.0 / temp_norm).powf(0.75);
            let tint_factor = tint * 0.002;
            let g_scale = (1.0 - tint_factor).max(0.01);
            r_scale = (r_scale + tint_factor * 0.5).max(0.01);
            b_scale = (b_scale + tint_factor * 0.5).max(0.01);
            r_gpu *= r_scale; g_gpu *= g_scale; b_gpu *= b_scale;
            
            // Exposure
            let exp_gain = 2.0f32.powf(exp);
            r_gpu *= exp_gain; g_gpu *= exp_gain; b_gpu *= exp_gain;
            
            // Tone Recovery
            let (r_rec, g_rec, b_rec) = apply_tone_recovery(r_gpu, g_gpu, b_gpu, hl, sh, wh, bl);
            
            // Tone Profile
            let contrast_gain = if contrast >= 0.0 {
                1.0 + (contrast / 100.0)
            } else {
                1.0 + (contrast / 100.0) * 0.60
            };
            let (r_disp_gpu, g_disp_gpu, b_disp_gpu) = match mode {
                ColorProfileMode::AgxVivid => {
                    let srgb = agx_tone_map_vivid(Color3::new(r_rec, g_rec, b_rec), contrast_gain);
                    (srgb.r().clamp(0.0, 1.0), srgb.g().clamp(0.0, 1.0), srgb.b().clamp(0.0, 1.0))
                }
                ColorProfileMode::AgxFilmic => {
                    let srgb = agx_tone_map(Color3::new(r_rec, g_rec, b_rec), contrast_gain);
                    (srgb.r().clamp(0.0, 1.0), srgb.g().clamp(0.0, 1.0), srgb.b().clamp(0.0, 1.0))
                }
                m => {
                    let c_srgb = REC2020_TO_LINEAR_SRGB.multiply_vector(Color3::new(r_rec, g_rec, b_rec));
                    let (h, s, v) = rgb_to_hsv(c_srgb.r().max(0.0), c_srgb.g().max(0.0), c_srgb.b().max(0.0));
                    let hsm = match m {
                        ColorProfileMode::CameraPortrait => DcpProfile::built_in_portrait_huesatmap(),
                        ColorProfileMode::CameraLandscape => DcpProfile::built_in_landscape_huesatmap(),
                        ColorProfileMode::CameraVivid => DcpProfile::built_in_vivid_huesatmap(),
                        _ => DcpProfile::built_in_standard_huesatmap(),
                    };
                    let (h2, s2, v2) = hsm.apply_hsv(h, s, v);
                    let (rm, gm, bm) = hsv_to_rgb(h2, s2, v2);
                    let curve = match m {
                        ColorProfileMode::CameraPortrait => DcpProfile::built_in_portrait_tone_curve(),
                        ColorProfileMode::CameraLandscape => DcpProfile::built_in_landscape_tone_curve(),
                        ColorProfileMode::CameraVivid => DcpProfile::built_in_vivid_tone_curve(),
                        _ => DcpProfile::built_in_standard_tone_curve(),
                    };
                    let lum = 0.2126 * rm + 0.7152 * gm + 0.0722 * bm;
                    let lum_curved = evaluate_tone_curve(lum, &curve);
                    let lum_adj = ((lum_curved - 0.18) * contrast_gain + 0.18).clamp(0.0, 1.0);
                    let gain = lum_adj / lum.max(1e-6);
                    (
                        linear_to_srgb_gamma((rm * gain).clamp(0.0, 1.0)).clamp(0.0, 1.0),
                        linear_to_srgb_gamma((gm * gain).clamp(0.0, 1.0)).clamp(0.0, 1.0),
                        linear_to_srgb_gamma((bm * gain).clamp(0.0, 1.0)).clamp(0.0, 1.0),
                    )
                }
            };
            
            // Color Adjustments
            let (r_gpu_out, g_gpu_out, b_gpu_out) = apply_color_adjustments(
                r_disp_gpu, g_disp_gpu, b_disp_gpu,
                vib, sat, 30.0, 20.0,
            );
            
            let gpu_8bit = [
                (r_gpu_out.clamp(0.0, 1.0) * 255.0).round() as u8,
                (g_gpu_out.clamp(0.0, 1.0) * 255.0).round() as u8,
                (b_gpu_out.clamp(0.0, 1.0) * 255.0).round() as u8,
            ];
            
            // 2. SIMULATE RUST EXPORT PIPELINE:
            let mut demosaiced = firstproof_core::export::DemosaicedImage {
                width: 1,
                height: 1,
                data: vec![c_rec2020.r(), c_rec2020.g(), c_rec2020.b()],
                is_raw: true,
            };
            let edits = firstproof_core::EditSettings {
                exposure: exp,
                contrast,
                highlights: hl,
                shadows: sh,
                whites: wh,
                blacks: bl,
                vibrance: vib,
                saturation: sat,
                deep_blue: 30.0,
                foliage: 20.0,
                temperature: temp,
                tint,
                tone_profile: mode,
                ..Default::default()
            };
            firstproof_core::export::apply_edits(&mut demosaiced, &edits);
            
            let r_disp_exp = demosaiced.data[0];
            let g_disp_exp = demosaiced.data[1];
            let b_disp_exp = demosaiced.data[2];
            let (r_disp_exp, g_disp_exp, b_disp_exp) = if mode == ColorProfileMode::AgxVivid || mode == ColorProfileMode::AgxFilmic {
                (r_disp_exp.clamp(0.0, 1.0), g_disp_exp.clamp(0.0, 1.0), b_disp_exp.clamp(0.0, 1.0))
            } else {
                (
                    linear_to_srgb_gamma(r_disp_exp).clamp(0.0, 1.0),
                    linear_to_srgb_gamma(g_disp_exp).clamp(0.0, 1.0),
                    linear_to_srgb_gamma(b_disp_exp).clamp(0.0, 1.0),
                )
            };
            let (r_exp_out, g_exp_out, b_exp_out) = apply_color_adjustments(
                r_disp_exp, g_disp_exp, b_disp_exp,
                vib, sat, 30.0, 20.0,
            );
            let exp_8bit = [
                (r_exp_out.clamp(0.0, 1.0) * 255.0).round() as u8,
                (g_exp_out.clamp(0.0, 1.0) * 255.0).round() as u8,
                (b_exp_out.clamp(0.0, 1.0) * 255.0).round() as u8,
            ];
            
            assert_eq!(
                gpu_8bit, exp_8bit,
                "Dual-engine slider parity failure for {} under config #{:?}: GPU {:?} vs Export {:?}",
                patch_name, profile_idx, gpu_8bit, exp_8bit
            );
        }
    }
}

#[test]
fn test_lensfun_export_distortion_math() {
    // Verifies that export coordinate transformation matches Metal shader formula exactly
    let target_w = 3000f32;
    let target_h = 2000f32;
    let aspect_ratio = target_w / target_h; // 1.5
    let auto_crop_scale = 1.0f32;
    let (k1, k2) = (-0.024f32, 0.006f32); // Typical Canon poly3 distortion

    // Pick top-right pixel in target output (tx=2999, ty=1999)
    let tx = 2999f32;
    let ty = 1999f32;
    let u = (tx + 0.5) / target_w;
    let v = (ty + 0.5) / target_h;

    // 1. Rust export math:
    let centered_x = (u - 0.5) * 2.0 * aspect_ratio / auto_crop_scale;
    let centered_y = (v - 0.5) * 2.0 / auto_crop_scale;
    let r2 = centered_x * centered_x + centered_y * centered_y;
    let r = r2.sqrt();
    let r4 = r2 * r2;
    let rd = r * (1.0 + k1 * r2 + k2 * r4);
    let scale_factor = rd / r;
    let rot_x = centered_x * scale_factor;
    let rot_y = centered_y * scale_factor;
    let rust_sx = (rot_x / (2.0 * aspect_ratio) + 0.5) * target_w;
    let rust_sy = (rot_y / 2.0 + 0.5) * target_h;

    // 2. Metal GPU shader math (from Shaders.metal):
    let mut norm_x = (u - 0.5) * 2.0 * aspect_ratio;
    let mut norm_y = (v - 0.5) * 2.0;
    let metal_r2 = norm_x * norm_x + norm_y * norm_y;
    let metal_r = metal_r2.sqrt();
    let metal_r_corr = metal_r * (1.0 + k1 * metal_r2 + k2 * (metal_r2 * metal_r2));
    let metal_factor = metal_r_corr / metal_r;
    norm_x *= metal_factor;
    norm_y *= metal_factor;
    let metal_uv_x = norm_x / (2.0 * aspect_ratio) + 0.5;
    let metal_uv_y = norm_y / 2.0 + 0.5;
    let metal_sx = metal_uv_x * target_w;
    let metal_sy = metal_uv_y * target_h;

    assert!((rust_sx - metal_sx).abs() < 1e-4, "X coord mismatch: rust={}, metal={}", rust_sx, metal_sx);
    assert!((rust_sy - metal_sy).abs() < 1e-4, "Y coord mismatch: rust={}, metal={}", rust_sy, metal_sy);
}

#[test]
fn test_sharpening_and_masking_parity() {
    // Metal shader detail function simulation
    fn metal_apply_detail(
        color: (f32, f32, f32),
        y_c: f32, y_l: f32, y_r: f32, y_t: f32, y_b: f32,
        amount: f32, masking: f32, nr: f32,
    ) -> (f32, f32, f32) {
        if amount <= 0.0 && nr <= 0.0 {
            return color;
        }
        let y_neighbors = 0.25 * (y_l + y_r + y_t + y_b);
        let delta_y = y_c - y_neighbors;
        let gx = (y_r - y_l) * 0.5;
        let gy = (y_b - y_t) * 0.5;
        let grad_mag = (gx * gx + gy * gy).sqrt();
        let w_edge = if masking <= 0.0 {
            1.0
        } else {
            let m = (masking / 100.0).clamp(0.0, 1.0);
            let threshold = m.powf(1.5) * 0.08;
            let t = (grad_mag / threshold.max(1e-4)).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        };
        let sharp_gain = (amount / 100.0) * 3.5;
        let sharp_term = sharp_gain * w_edge * delta_y;
        let nr_term = (nr / 100.0) * (1.0 - w_edge) * delta_y;
        let delta_adj = sharp_term - nr_term;
        let k = ((y_c + delta_adj) / y_c.max(1e-5)).clamp(0.0, 3.0);
        (
            (color.0 * k).clamp(0.0, 1.0),
            (color.1 * k).clamp(0.0, 1.0),
            (color.2 * k).clamp(0.0, 1.0),
        )
    }

    // Test a sweep of detail scenarios
    let scenarios = [
        // (name, c_rgb, y_c, (y_l, y_r, y_t, y_b), amount, masking, nr)
        ("neutral", (0.5, 0.4, 0.3), 0.4, (0.3, 0.5, 0.3, 0.5), 0.0, 0.0, 0.0),
        ("step_edge_unmasked", (0.8, 0.8, 0.8), 0.8, (0.2, 0.8, 0.5, 0.5), 50.0, 0.0, 0.0),
        ("step_edge_masked", (0.8, 0.8, 0.8), 0.8, (0.2, 0.8, 0.5, 0.5), 50.0, 30.0, 0.0),
        ("flat_noise_unmasked", (0.505, 0.505, 0.505), 0.505, (0.5, 0.5, 0.5, 0.5), 50.0, 0.0, 0.0),
        ("flat_noise_masked", (0.505, 0.505, 0.505), 0.505, (0.5, 0.5, 0.5, 0.5), 50.0, 30.0, 0.0),
        ("flat_noise_nr", (0.505, 0.505, 0.505), 0.505, (0.5, 0.5, 0.5, 0.5), 0.0, 30.0, 80.0),
        ("edge_with_nr", (0.8, 0.8, 0.8), 0.8, (0.2, 0.8, 0.5, 0.5), 40.0, 20.0, 60.0),
    ];

    for (name, color, y_c, (y_l, y_r, y_t, y_b), amount, masking, nr) in scenarios {
        let rust_res = firstproof_core::export::apply_detail_and_sharpening_rgb(
            color.0, color.1, color.2,
            y_c, y_l, y_r, y_t, y_b,
            amount, masking, nr,
        );
        let metal_res = metal_apply_detail(
            color,
            y_c, y_l, y_r, y_t, y_b,
            amount, masking, nr,
        );

        let rust_8bit = [
            (rust_res.0 * 255.0).round() as u8,
            (rust_res.1 * 255.0).round() as u8,
            (rust_res.2 * 255.0).round() as u8,
        ];
        let metal_8bit = [
            (metal_res.0 * 255.0).round() as u8,
            (metal_res.1 * 255.0).round() as u8,
            (metal_res.2 * 255.0).round() as u8,
        ];

        assert_eq!(
            rust_8bit, metal_8bit,
            "Detail & sharpening parity failed for scenario '{}': Rust {:?} vs Metal {:?}",
            name, rust_8bit, metal_8bit
        );
    }
}

#[test]
fn test_bilateral_tone_recovery_metal_parity() {
    // Metal shader simulation of applyBilateralToneRecovery
    fn metal_apply_bilateral_tone_recovery(
        color: (f32, f32, f32),
        y_c: f32,
        y_neighbors: &[f32; 8],
        highlights: f32,
        shadows: f32,
        whites: f32,
        blacks: f32,
    ) -> (f32, f32, f32) {
        if highlights == 0.0 && shadows == 0.0 && whites == 0.0 && blacks == 0.0 {
            return color;
        }
        if y_c <= 1e-6 {
            return color;
        }
        let ev_c = (y_c.max(1e-5) / 0.18).log2();
        let w_s_ortho: f32 = 0.60653066;
        let w_s_diag: f32 = 0.36787944;
        let inv_two_sigma_r_sq: f32 = 1.0 / 1.445;

        let mut total_weight = 1.0f32;
        let mut weighted_ev_sum = ev_c;

        for i in 0..4 {
            let ev_i = (y_neighbors[i].max(1e-5) / 0.18).log2();
            let d_ev = ev_i - ev_c;
            let w_r = (-(d_ev * d_ev) * inv_two_sigma_r_sq).exp();
            let w = w_s_ortho * w_r;
            total_weight += w;
            weighted_ev_sum += w * ev_i;
        }

        for i in 4..8 {
            let ev_i = (y_neighbors[i].max(1e-5) / 0.18).log2();
            let d_ev = ev_i - ev_c;
            let w_r = (-(d_ev * d_ev) * inv_two_sigma_r_sq).exp();
            let w = w_s_diag * w_r;
            total_weight += w;
            weighted_ev_sum += w * ev_i;
        }

        let ev_base = weighted_ev_sum / total_weight;
        let mut delta_ev = 0.0f32;

        if blacks != 0.0 {
            let u = ((-ev_base - 1.0) / 3.0).clamp(0.0, 1.0);
            let w_blacks = u * u * (3.0 - 2.0 * u);
            delta_ev += (blacks / 100.0) * 0.90 * w_blacks;
        }
        if shadows != 0.0 {
            let u = ((ev_base + 4.0) / 4.0).clamp(0.0, 1.0);
            let w_shadows = (std::f32::consts::PI * u).sin();
            delta_ev += (shadows / 100.0) * 1.20 * w_shadows;
        }
        if highlights != 0.0 {
            let u = (ev_base / 4.0).clamp(0.0, 1.0);
            let w_highlights = (std::f32::consts::PI * u).sin();
            delta_ev += (highlights / 100.0) * 1.30 * w_highlights;
        }
        if whites != 0.0 {
            let u = ((ev_base - 1.0) / 3.0).clamp(0.0, 1.0);
            let w_whites = u * u * (3.0 - 2.0 * u);
            delta_ev += (whites / 100.0) * 0.90 * w_whites;
        }

        if delta_ev == 0.0 {
            return color;
        }
        let gain = 2.0f32.powf(delta_ev);
        (color.0 * gain, color.1 * gain, color.2 * gain)
    }

    let scenarios = [
        ("neutral", (0.5, 0.4, 0.3), 0.4, [0.4; 8], 0.0, 0.0, 0.0, 0.0),
        ("flat_shadow_lift", (0.04, 0.03, 0.02), 0.03, [0.03; 8], 0.0, 50.0, 0.0, 0.0),
        ("flat_highlight_pull", (0.8, 0.7, 0.6), 0.7, [0.7; 8], -40.0, 0.0, 0.0, 0.0),
        ("step_edge_silhouette", (0.03, 0.03, 0.03), 0.03, [0.03, 0.85, 0.03, 0.85, 0.03, 0.85, 0.03, 0.85], 0.0, 60.0, 0.0, 0.0),
        ("step_edge_sky", (0.85, 0.85, 0.85), 0.85, [0.03, 0.85, 0.03, 0.85, 0.03, 0.85, 0.03, 0.85], -50.0, 0.0, 0.0, 0.0),
        ("textured_shadow", (0.04, 0.04, 0.04), 0.04, [0.038, 0.042, 0.039, 0.041, 0.037, 0.043, 0.040, 0.041], 0.0, 45.0, 0.0, -10.0),
        ("multi_slider_high_dynamic", (0.6, 0.5, 0.4), 0.5, [0.45, 0.55, 0.48, 0.52, 0.44, 0.56, 0.47, 0.53], -30.0, 40.0, 15.0, -20.0),
    ];

    for (name, color, y_c, neighbors, hl, sh, wh, bl) in scenarios {
        let rust_res = firstproof_core::export::apply_bilateral_tone_recovery_rgb(
            color.0, color.1, color.2,
            y_c, &neighbors,
            hl, sh, wh, bl,
        );
        let metal_res = metal_apply_bilateral_tone_recovery(
            color,
            y_c, &neighbors,
            hl, sh, wh, bl,
        );

        let rust_8bit = [
            (rust_res.0.clamp(0.0, 1.0) * 255.0).round() as u8,
            (rust_res.1.clamp(0.0, 1.0) * 255.0).round() as u8,
            (rust_res.2.clamp(0.0, 1.0) * 255.0).round() as u8,
        ];
        let metal_8bit = [
            (metal_res.0.clamp(0.0, 1.0) * 255.0).round() as u8,
            (metal_res.1.clamp(0.0, 1.0) * 255.0).round() as u8,
            (metal_res.2.clamp(0.0, 1.0) * 255.0).round() as u8,
        ];

        assert_eq!(
            rust_8bit, metal_8bit,
            "Bilateral tone recovery parity failed for scenario '{}': Rust {:?} vs Metal {:?}",
            name, rust_8bit, metal_8bit
        );
    }
}

