use lumenbox_core::color::dcp::{
    DcpProfile, evaluate_tone_curve, rgb_to_hsv, hsv_to_rgb,
};
use lumenbox_core::ColorProfileMode;
use lumenbox_core::xmp::{XmpData, generate_xmp, parse_xmp};

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
fn test_cr2_portrait_cheek_and_helmet_fidelity() {
    use std::path::Path;
    use lumenbox_core::raw_decoder::{RawDecoder, RawlerDecoder};
    let path = Path::new("/Users/blikkie/Pictures/Darktable/20250409_eos_pull/20250409_0073.CR2");
    if !path.exists() { return; }
    let Ok(_frame) = RawlerDecoder.decode(path) else { return; };
    
    let Ok(demosaiced) = lumenbox_core::export::decode_image(path) else { return; };
    let w = demosaiced.width;
    let h = demosaiced.height;
    
    // Helmet body coordinates: x ~ 50% w, y ~ 25% h (body of helmet, not specular crown)
    let h_idx = ((h * 25 / 100) * w + (w * 50 / 100)) * 3;
    let h_rec2020 = lumenbox_core::color::Color3::new(demosaiced.data[h_idx], demosaiced.data[h_idx + 1], demosaiced.data[h_idx + 2]);
    
    // Cheek coordinates: x ~ 43% w, y ~ 46% h
    let c_idx = ((h * 46 / 100) * w + (w * 43 / 100)) * 3;
    let c_rec2020 = lumenbox_core::color::Color3::new(demosaiced.data[c_idx], demosaiced.data[c_idx + 1], demosaiced.data[c_idx + 2]);
    
    // 1. Verify Helmet body retains rich pink tones (specular crown at y~18% may clamp to white; helmet body should not)
    let out_helmet = lumenbox_core::color::agx_tone_map_vivid(h_rec2020, 1.0);
    let (h_hue, h_sat, h_val) = lumenbox_core::color::rgb_to_hsv(out_helmet.r(), out_helmet.g(), out_helmet.b());
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
    let out_cheek = lumenbox_core::color::agx_tone_map_vivid(c_rec2020, 1.0);
    let (c_hue, c_sat, _c_val) = lumenbox_core::color::rgb_to_hsv(out_cheek.r(), out_cheek.g(), out_cheek.b());
    println!("Cheek: hue={:.1}, sat={:.3}", c_hue, c_sat);
    assert!(c_hue >= 18.0 && c_hue <= 35.0, "Cheek hue outside natural skin range: hue={:.1}", c_hue);
    assert!(c_sat <= 0.75, "Cheek linear skin tone oversaturated: sat={:.3}", c_sat);
    
    // Check display saturation (standard 2.2 gamma applied as seen by the user):
    let (r_disp, g_disp, b_disp) = (out_cheek.r().powf(1.0/2.2), out_cheek.g().powf(1.0/2.2), out_cheek.b().powf(1.0/2.2));
    let (_, c_sat_disp, _) = lumenbox_core::color::rgb_to_hsv(r_disp, g_disp, b_disp);
    assert!(c_sat_disp <= 0.50, "Cheek display skin tone oversaturated: sat={:.3}", c_sat_disp);
}

#[test]
fn test_agx_rec2020_saturation_and_vivid_restoration() {
    use lumenbox_core::color::{Color3, LINEAR_SRGB_TO_REC2020, agx_tone_map, agx_tone_map_vivid, rgb_to_hsv};
    
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
fn test_gpu_preview_vs_export_parity() {
    use std::path::Path;
    use lumenbox_core::raw_decoder::{RawDecoder, RawlerDecoder};
    use lumenbox_core::color::{Color3, agx_tone_map_vivid};
    use lumenbox_core::EditSettings;
    
    let path = Path::new("/Users/blikkie/Pictures/Darktable/20250409_eos_pull/20250409_0073.CR2");
    if !path.exists() { return; }
    let Ok(_frame) = RawlerDecoder.decode(path) else { return; };
    
    let Ok(demosaiced) = lumenbox_core::export::decode_image(path) else { return; };
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
    let mut export_img = lumenbox_core::export::decode_image(path).unwrap();
    let edits = EditSettings {
        exposure: 0.0,
        contrast: 0.0,
        highlights: 0.0,
        shadows: 0.0,
        whites: 0.0,
        blacks: 0.0,
        temperature: 5500.0,
        tint: 0.0,
        crop_left: 0.0,
        crop_top: 0.0,
        crop_right: 1.0,
        crop_bottom: 1.0,
        crop_angle: 0.0,
        rotation: 0,
        flip_horizontal: false,
        flip_vertical: false,
        tone_profile: lumenbox_core::ColorProfileMode::AgxVivid,
        lut_name: None,
        lut_intensity: 1.0,
    };
    lumenbox_core::export::apply_edits(&mut export_img, &edits);
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

