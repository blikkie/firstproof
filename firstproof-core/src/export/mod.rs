pub mod icc;
pub mod decode;
pub mod pipeline;
pub mod encoders;
pub mod batch;
pub mod arena;

pub use icc::*;
pub use decode::*;
pub use pipeline::*;
pub use encoders::*;
pub use batch::*;
pub use arena::*;

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};
    use crate::{
        BatchExportItemRecord, EditSettings, ExportColorSpace, ExportRenderingIntent,
    };
    use crate::color::Color3;

    #[test]
    fn test_export_jpeg_and_edits() {
        let tmp_in = "test_export_in.jpg";
        let tmp_out = "test_export_out.jpg";
        
        let img = RgbImage::from_fn(100, 100, |x, y| {
            Rgb([(x * 2) as u8, (y * 2) as u8, 128])
        });
        img.save(tmp_in).unwrap();
        
        let edits = EditSettings {
            exposure: 1.0,
            contrast: 20.0,
            temperature: 6500.0,
            tint: 10.0,
            tone_profile: crate::ColorProfileMode::AgxFilmic,
            ..Default::default()
        };
        
        let res = export_photo_file(
            tmp_in,
            tmp_out,
            &edits,
            None,
            90,
            Some(50),
            ExportColorSpace::Srgb,
            ExportRenderingIntent::RelativeColorimetric,
            false,
        );
        assert!(res.is_ok());
        
        let exported = image::open(tmp_out).unwrap();
        assert_eq!(exported.width(), 50);
        assert_eq!(exported.height(), 50);
        
        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out);
    }

    #[test]
    fn test_pipeline_parity_math() {
        let mut img = DemosaicedImage {
            width: 1,
            height: 1,
            data: vec![0.18, 0.18, 0.18],
            is_raw: true,
        };
        let edits = EditSettings {
            temperature: 5500.0,
            tone_profile: crate::ColorProfileMode::AgxFilmic,
            ..Default::default()
        };
        apply_edits(&mut img, &edits);
        
        let c_srgb = Color3::new(img.data[0], img.data[1], img.data[2]);
        let srgb8 = crate::color::linear_rgb_to_srgb8(c_srgb);
        
        eprintln!("DEBUG: c_srgb = {:?}, srgb8 = {:?}", c_srgb, srgb8);
        assert!(srgb8[0] >= 120 && srgb8[0] <= 135, "Expected display midtone range (120..135), got {}", srgb8[0]);
        assert_eq!(srgb8[0], srgb8[1]);
        assert_eq!(srgb8[1], srgb8[2]);
    }

    #[test]
    fn test_lut_export_pipeline() {
        let tmp_in = "test_lut_in.jpg";
        let tmp_out_no_lut = "test_lut_out_no_lut.jpg";
        let tmp_out_lut = "test_lut_out_lut.jpg";
        
        let img = RgbImage::from_fn(30, 30, |x, y| {
            Rgb([x as u8 * 8, y as u8 * 8, 128])
        });
        img.save(tmp_in).unwrap();
        
        let mut edits = EditSettings {
            tone_profile: crate::ColorProfileMode::AgxVivid,
            ..Default::default()
        };
        
        let tmp_cube = "test_mono.cube";
        let cube_content = "TITLE \"Test Mono\"\nLUT_3D_SIZE 2\nDOMAIN_MIN 0.0 0.0 0.0\nDOMAIN_MAX 1.0 1.0 1.0\n0.0 0.0 0.0\n0.5 0.5 0.5\n0.5 0.5 0.5\n1.0 1.0 1.0\n0.5 0.5 0.5\n1.0 1.0 1.0\n1.0 1.0 1.0\n1.0 1.0 1.0\n";
        std::fs::write(tmp_cube, cube_content).unwrap();

        export_photo_file(tmp_in, tmp_out_no_lut, &edits, None, 95, None, ExportColorSpace::Srgb, ExportRenderingIntent::RelativeColorimetric, false).unwrap();
        
        edits.lut_name = Some(tmp_cube.to_string());
        edits.lut_intensity = 1.0;
        export_photo_file(tmp_in, tmp_out_lut, &edits, None, 95, None, ExportColorSpace::Srgb, ExportRenderingIntent::RelativeColorimetric, false).unwrap();
        
        let img_no_lut = image::open(tmp_out_no_lut).unwrap().to_rgb8();
        let img_lut = image::open(tmp_out_lut).unwrap().to_rgb8();
        
        let p_no_lut = img_no_lut.get_pixel(15, 15);
        let p_lut = img_lut.get_pixel(15, 15);
        
        assert_eq!(p_lut[0], p_lut[1]);
        assert_eq!(p_lut[1], p_lut[2]);
        assert_ne!(p_no_lut[0], p_no_lut[2]);
        
        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out_no_lut);
        let _ = std::fs::remove_file(tmp_out_lut);
        let _ = std::fs::remove_file(tmp_cube);
    }

    #[test]
    fn test_export_icc_injection() {
        let tmp_in = "test_icc_in.jpg";
        let tmp_out = "test_icc_out.jpg";
        
        let img = RgbImage::from_fn(20, 20, |x, y| {
            Rgb([x as u8 * 10, y as u8 * 10, 128])
        });
        img.save(tmp_in).unwrap();
        
        let edits = EditSettings {
            tone_profile: crate::ColorProfileMode::AgxVivid,
            ..Default::default()
        };
        
        export_photo_file(tmp_in, tmp_out, &edits, None, 95, None, ExportColorSpace::Srgb, ExportRenderingIntent::RelativeColorimetric, false).unwrap();
        
        let bytes = std::fs::read(tmp_out).unwrap();
        let marker = [0xff, 0xe2];
        let found_app2 = bytes.windows(marker.len()).any(|w| w == marker);
        assert!(found_app2, "Exported JPEG must contain APP2 marker");
        let icc_ident = b"ICC_PROFILE\0";
        let found_ident = bytes.windows(icc_ident.len()).any(|w| w == icc_ident);
        assert!(found_ident, "Exported JPEG must contain ICC_PROFILE identifier");
        
        if cfg!(target_os = "macos") {
            let sips_out = std::process::Command::new("sips")
                .arg("-g")
                .arg("profile")
                .arg(tmp_out)
                .output();
            if let Ok(output) = sips_out {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("sips output: {}", stdout);
                assert!(stdout.contains("sRGB"), "macOS sips must report sRGB color profile, got: {}", stdout);
            }
        }
        
        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out);
    }

    #[test]
    fn test_export_rotation_and_flips() {
        let tmp_in = "test_rot_in.jpg";
        let tmp_out_90 = "test_rot_out_90.jpg";
        let tmp_out_flip = "test_rot_out_flip.jpg";

        let img = RgbImage::from_fn(60, 40, |x, _y| {
            if x < 30 {
                Rgb([255, 0, 0])
            } else {
                Rgb([0, 255, 0])
            }
        });
        img.save(tmp_in).unwrap();

        let mut edits = EditSettings::default();
        edits.rotation = 90;
        export_photo_file(tmp_in, tmp_out_90, &edits, None, 95, None, ExportColorSpace::Srgb, ExportRenderingIntent::RelativeColorimetric, false).unwrap();
        let rotated = image::open(tmp_out_90).unwrap().to_rgb8();
        assert_eq!(rotated.width(), 40);
        assert_eq!(rotated.height(), 60);

        let mut edits_flip = EditSettings::default();
        edits_flip.flip_horizontal = true;
        export_photo_file(tmp_in, tmp_out_flip, &edits_flip, None, 95, None, ExportColorSpace::Srgb, ExportRenderingIntent::RelativeColorimetric, false).unwrap();
        let flipped = image::open(tmp_out_flip).unwrap().to_rgb8();
        assert_eq!(flipped.width(), 60);
        assert_eq!(flipped.height(), 40);
        let px = flipped.get_pixel(5, 5);
        assert!(px[1] > 180 && px[1] > px[0], "Flipped pixel (5, 5) must be green from horizontal flip, got {:?}", px);

        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out_90);
        let _ = std::fs::remove_file(tmp_out_flip);
    }

    #[test]
    fn test_tone_recovery_math() {
        let (r0, g0, b0) = apply_tone_recovery(0.5, 0.4, 0.3, 0.0, 0.0, 0.0, 0.0);
        assert_eq!(r0, 0.5);
        assert_eq!(g0, 0.4);
        assert_eq!(b0, 0.3);

        let (r_in, g_in, b_in) = (0.6, 0.4, 0.2);
        let orig_ratio_rg = r_in / g_in;
        let orig_ratio_bg = b_in / g_in;

        let (r_hl, g_hl, b_hl) = apply_tone_recovery(r_in, g_in, b_in, -50.0, 0.0, 0.0, 0.0);
        assert!((r_hl / g_hl - orig_ratio_rg).abs() < 1e-5);
        assert!((b_hl / g_hl - orig_ratio_bg).abs() < 1e-5);
        assert!(g_hl < g_in);

        let (r_sh_in, g_sh_in, b_sh_in) = (0.04, 0.03, 0.02);
        let (r_sh, g_sh, _b_sh) = apply_tone_recovery(r_sh_in, g_sh_in, b_sh_in, 0.0, 50.0, 0.0, 0.0);
        assert!(g_sh > g_sh_in);
        assert!((r_sh / g_sh - (r_sh_in / g_sh_in)).abs() < 1e-5);

        let (r_bl_in, g_bl_in, b_bl_in) = (0.008, 0.008, 0.008);
        let (_, g_bl, _) = apply_tone_recovery(r_bl_in, g_bl_in, b_bl_in, 0.0, 0.0, 0.0, -50.0);
        assert!(g_bl < g_bl_in);
    }

    #[test]
    fn test_tone_recovery_monotonicity_and_plateau() {
        // Highlights recovery monotonicity: pulling highlights (-100) on increasing brightness
        // must produce non-decreasing attenuation (monotonically non-increasing gain)
        // and must NEVER drop back to zero for extreme highlights (ev >= 3.0).
        let mut prev_gain = 1.0f32;
        // Test EV from 0.0 (0.18) to +6.0 (11.52)
        for ev_step in 0..=60 {
            let ev = ev_step as f32 * 0.1;
            let lum = 0.18 * 2.0f32.powf(ev);
            let (r_rec, _, _) = apply_tone_recovery(lum, lum, lum, -100.0, 0.0, 0.0, 0.0);
            let gain = r_rec / lum;

            assert!(
                gain <= prev_gain + 1e-6,
                "Highlights attenuation must be monotonic: gain {} at ev {} was greater than prev_gain {}",
                gain, ev, prev_gain
            );
            prev_gain = gain;

            // For ev >= 3.0, the recovery must stay at maximum plateau (gain = 2^(-1.30) ~ 0.4061)
            // It must NEVER rebound towards 1.0 (which was the bug with the old half-sine lobe)
            if ev >= 3.0 {
                let expected_plateau_gain = 2.0f32.powf(-1.30);
                assert!(
                    (gain - expected_plateau_gain).abs() < 1e-4,
                    "Highlights recovery must plateau at maximum attenuation above +3 EV: got {} expected {}",
                    gain, expected_plateau_gain
                );
            }
        }

        // Shadows recovery monotonicity: lifting shadows (+100) on decreasing brightness
        // must produce monotonically non-decreasing boost and plateau below -3 EV
        let mut prev_boost = 1.0f32;
        // Test EV from 0.0 (0.18) down to -6.0 (0.0028)
        for ev_step in 0..=60 {
            let ev = -(ev_step as f32 * 0.1);
            let lum = 0.18 * 2.0f32.powf(ev);
            let (r_rec, _, _) = apply_tone_recovery(lum, lum, lum, 0.0, 100.0, 0.0, 0.0);
            let boost = r_rec / lum;

            assert!(
                boost >= prev_boost - 1e-6,
                "Shadows boost must be monotonic: boost {} at ev {} was less than prev_boost {}",
                boost, ev, prev_boost
            );
            prev_boost = boost;

            // For ev <= -3.0, the boost must stay at maximum plateau (gain = 2^(1.20) ~ 2.2974)
            // It must NEVER drop back towards 1.0
            if ev <= -3.0 {
                let expected_plateau_boost = 2.0f32.powf(1.20);
                assert!(
                    (boost - expected_plateau_boost).abs() < 1e-4,
                    "Shadows recovery must plateau at maximum boost below -3 EV: got {} expected {}",
                    boost, expected_plateau_boost
                );
            }
        }
    }

    #[test]
    fn test_bilateral_tone_recovery_flat_patch() {
        let (r, g, b) = (0.05, 0.04, 0.03);
        let lum = 0.2627 * r + 0.6780 * g + 0.0593 * b;
        let neighbors = [lum; 16];

        let bilateral = apply_bilateral_tone_recovery_rgb(r, g, b, lum, &neighbors, -30.0, 40.0, 10.0, -15.0);
        let pointwise = apply_tone_recovery(r, g, b, -30.0, 40.0, 10.0, -15.0);

        assert!((bilateral.0 - pointwise.0).abs() < 1e-6);
        assert!((bilateral.1 - pointwise.1).abs() < 1e-6);
        assert!((bilateral.2 - pointwise.2).abs() < 1e-6);
    }

    #[test]
    fn test_bilateral_tone_recovery_halo_suppression_at_contrast_edge() {
        // Dark subject (shadow) adjacent to bright sky (highlight)
        let r_shadow = 0.03;
        let g_shadow = 0.03;
        let b_shadow = 0.03;
        let y_shadow = 0.03;

        // Inner and outer rings with 8 taps in shadow (0.03) and 8 taps in sky (0.85)
        let edge_neighbors = [
            0.03, 0.85, 0.03, 0.85, 0.03, 0.85, 0.03, 0.85,
            0.03, 0.85, 0.03, 0.85, 0.03, 0.85, 0.03, 0.85,
        ];

        // Lift shadows by +60
        let (r_rec, g_rec, b_rec) = apply_bilateral_tone_recovery_rgb(
            r_shadow, g_shadow, b_shadow,
            y_shadow, &edge_neighbors,
            0.0, 60.0, 0.0, 0.0,
        );

        // The shadow should lift significantly (> 1.25x), proving the bright sky did NOT pull
        // the base illumination into midtones/highlights (which would cancel shadow lift and create halos)
        assert!(g_rec > g_shadow * 1.25, "Shadow lift at edge should remain robust: {} vs {}", g_rec, g_shadow);

        // Chromaticity should be preserved exactly
        assert!((r_rec / g_rec - 1.0).abs() < 1e-5);
        assert!((b_rec / g_rec - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_bilateral_tone_recovery_texture_contrast_preservation() {
        // Two adjacent pixels in a textured shadow: pixel A is 0.040, pixel B is 0.044 (ratio = 1.10)
        let y_a = 0.040;
        let y_b = 0.044;
        let orig_ratio = y_b / y_a;

        // Textured neighborhood with subtle variations around 0.042
        let neighbors_a = [
            0.039, 0.041, 0.043, 0.042, 0.038, 0.044, 0.040, 0.041,
            0.041, 0.042, 0.040, 0.043, 0.039, 0.044, 0.041, 0.042,
        ];
        let neighbors_b = [
            0.041, 0.043, 0.045, 0.044, 0.040, 0.046, 0.042, 0.043,
            0.041, 0.042, 0.040, 0.043, 0.039, 0.044, 0.041, 0.042,
        ];

        let (_, g_rec_a, _) = apply_bilateral_tone_recovery_rgb(y_a, y_a, y_a, y_a, &neighbors_a, 0.0, 50.0, 0.0, 0.0);
        let (_, g_rec_b, _) = apply_bilateral_tone_recovery_rgb(y_b, y_b, y_b, y_b, &neighbors_b, 0.0, 50.0, 0.0, 0.0);

        let recovered_ratio = g_rec_b / g_rec_a;
        // In bilateral base/detail tone recovery, the micro-contrast ratio should be preserved within 2%
        assert!(
            (recovered_ratio - orig_ratio).abs() < 0.020,
            "Local texture contrast ratio should be preserved: orig {}, recovered {}",
            orig_ratio, recovered_ratio
        );
    }

    #[test]
    fn test_contrast_floor_and_tonal_recovery_limits() {
        let floor_gain = if -100.0f32 >= 0.0 {
            1.0 + (-100.0 / 100.0)
        } else {
            1.0 + (-100.0 / 100.0) * 0.60
        };
        assert!((floor_gain - 0.40f32).abs() < 1e-6);

        let ceil_gain = if 100.0f32 >= 0.0 {
            1.0 + (100.0 / 100.0)
        } else {
            1.0 + (100.0 / 100.0) * 0.60
        };
        assert!((ceil_gain - 2.0f32).abs() < 1e-6);

        let exp_floor = 2.0f32.powf(-3.0);
        assert!((exp_floor - 0.125f32).abs() < 1e-6);
        let exp_ceil = 2.0f32.powf(3.0);
        assert!((exp_ceil - 8.0f32).abs() < 1e-6);

        let (_, g_max, _) = apply_tone_recovery(0.8, 0.8, 0.8, -100.0, 0.0, 0.0, 0.0);
        assert!(g_max > 0.8 * 0.35, "Highlights at -100 should not crush below photographic threshold");

        let (_, g_min, _) = apply_tone_recovery(0.05, 0.05, 0.05, 0.0, 100.0, 0.0, 0.0);
        assert!(g_min < 0.05 * 3.0, "Shadows at +100 should not blow out beyond 3x gain");
    }

    #[test]
    fn test_white_balance_from_sample() {
        use crate::calculate_white_balance_from_sample;

        let neutral = calculate_white_balance_from_sample(0.5, 0.5, 0.5);
        assert_eq!(neutral.temperature, 5500.0);
        assert_eq!(neutral.tint, 0.0);

        let warm = calculate_white_balance_from_sample(0.8, 0.6, 0.3);
        assert!(warm.temperature < 5500.0);
        assert!(warm.temperature >= 2000.0);

        let cool = calculate_white_balance_from_sample(0.3, 0.5, 0.8);
        assert!(cool.temperature > 5500.0);
        assert!(cool.temperature <= 20000.0);

        let green = calculate_white_balance_from_sample(0.4, 0.6, 0.4);
        assert!(green.tint > 0.0);
        assert!(green.tint <= 200.0);

        let extreme_blue = calculate_white_balance_from_sample(0.01, 0.1, 0.99);
        assert_eq!(extreme_blue.temperature, 20000.0);
        let extreme_red = calculate_white_balance_from_sample(0.99, 0.1, 0.01);
        assert_eq!(extreme_red.temperature, 2000.0);
        let extreme_green = calculate_white_balance_from_sample(0.01, 0.99, 0.01);
        assert_eq!(extreme_green.tint, 200.0);
    }

    #[test]
    fn test_color_adjustments_math() {
        let (r0, g0, b0) = apply_color_adjustments(0.5, 0.4, 0.3, 0.0, 0.0, 0.0, 0.0);
        assert!((r0 - 0.5).abs() < 1e-5);
        assert!((g0 - 0.4).abs() < 1e-5);
        assert!((b0 - 0.3).abs() < 1e-5);

        let (_, s_base, _) = crate::color::rgb_to_hsv(0.6, 0.4, 0.2);
        let (r_sat, g_sat, b_sat) = apply_color_adjustments(0.6, 0.4, 0.2, 0.0, 50.0, 0.0, 0.0);
        let (_, s_boosted, _) = crate::color::rgb_to_hsv(r_sat, g_sat, b_sat);
        assert!(s_boosted > s_base);

        let (r_skin, g_skin, b_skin) = (0.85, 0.65, 0.50);
        let (h_skin, s_skin_orig, _) = crate::color::rgb_to_hsv(r_skin, g_skin, b_skin);
        assert!((h_skin - 35.0).abs() < 10.0);

        let (h_blue_base, s_blue_base, v_blue_base) = (215.0, s_skin_orig, 0.85);
        let (r_blue, g_blue, b_blue) = crate::color::hsv_to_rgb(h_blue_base, s_blue_base, v_blue_base);

        let (r_skin_vib, g_skin_vib, b_skin_vib) = apply_color_adjustments(r_skin, g_skin, b_skin, 80.0, 0.0, 0.0, 0.0);
        let (_, s_skin_vib, _) = crate::color::rgb_to_hsv(r_skin_vib, g_skin_vib, b_skin_vib);

        let (r_blue_vib, g_blue_vib, b_blue_vib) = apply_color_adjustments(r_blue, g_blue, b_blue, 80.0, 0.0, 0.0, 0.0);
        let (_, s_blue_vib, _) = crate::color::rgb_to_hsv(r_blue_vib, g_blue_vib, b_blue_vib);

        let skin_delta_s = s_skin_vib - s_skin_orig;
        let blue_delta_s = s_blue_vib - s_blue_base;
        assert!(skin_delta_s < blue_delta_s * 0.7);

        let (r_sky, g_sky, b_sky) = (0.20, 0.50, 0.85);
        let (_, s_sky_orig, v_sky_orig) = crate::color::rgb_to_hsv(r_sky, g_sky, b_sky);
        let (r_db, g_db, b_db) = apply_color_adjustments(r_sky, g_sky, b_sky, 0.0, 0.0, 80.0, 0.0);
        let (_, s_db, v_db) = crate::color::rgb_to_hsv(r_db, g_db, b_db);
        assert!(s_db > s_sky_orig);
        assert!(v_db < v_sky_orig);

        let (r_grass, g_grass, b_grass) = crate::color::hsv_to_rgb(85.0, 0.6, 0.7);
        let (r_fol, g_fol, b_fol) = apply_color_adjustments(r_grass, g_grass, b_grass, 0.0, 0.0, 0.0, 80.0);
        let (h_fol, s_fol, _) = crate::color::rgb_to_hsv(r_fol, g_fol, b_fol);
        assert!(h_fol > 85.0);
        assert!(s_fol > 0.6);
    }

    #[test]
    fn test_split_toning_math() {
        // 1. Identity when saturations are zero
        let (r0, g0, b0) = apply_split_toning(0.2, 0.5, 0.8, 215.0, 0.0, 45.0, 0.0, 0.0);
        assert_eq!((r0, g0, b0), (0.2, 0.5, 0.8));

        // 2. Shadows toning with selenium/cool blue hue (215 deg)
        // Shadow tone (lum = 0.20 < 0.50 pivot)
        let (r_sh, _g_sh, b_sh) = apply_split_toning(0.2, 0.2, 0.2, 215.0, 50.0, 45.0, 0.0, 0.0);
        assert!(b_sh > r_sh, "Shadows should be tinted blue (selenium)");

        // Highlight should remain untouched when highlight_sat is 0
        let (r_hl, g_hl, b_hl) = apply_split_toning(0.8, 0.8, 0.8, 215.0, 50.0, 45.0, 0.0, 0.0);
        assert_eq!((r_hl, g_hl, b_hl), (0.8, 0.8, 0.8), "Highlights untouched when highlight_sat is 0");

        // 3. Highlight toning with warm amber/gold hue (45 deg)
        let (r_hl2, g_hl2, b_hl2) = apply_split_toning(0.8, 0.8, 0.8, 215.0, 0.0, 45.0, 50.0, 0.0);
        assert!(r_hl2 > b_hl2, "Highlights should be warm (R > B)");
        assert!(g_hl2 > b_hl2, "Highlights should be warm (G > B)");

        // 4. Optical balance anchoring: positive balance shifts pivot downward, extending highlights into midtones
        let mid = (0.4, 0.4, 0.4);
        // Neutral balance (pivot 0.50): midtone 0.4 is in shadow territory
        let (_, _, b_mid_bal0) = apply_split_toning(mid.0, mid.1, mid.2, 215.0, 50.0, 45.0, 0.0, 0.0);
        assert!(b_mid_bal0 > mid.2, "Midtone at 0.4 tinted blue with balance 0");

        // High positive balance (+100, pivot shifts down to 0.25): midtone 0.4 is now in highlight territory!
        let (r_mid_bal_plus, _, b_mid_bal_plus) = apply_split_toning(mid.0, mid.1, mid.2, 215.0, 50.0, 45.0, 50.0, 100.0);
        assert!(r_mid_bal_plus > b_mid_bal_plus, "Midtone at 0.4 now receives highlight warm toning under +100 balance");

        // 5. Photometric luminance preservation: split toning adds pure chrominance
        let gray = 0.35f32;
        let lum_before = 0.2627 * gray + 0.6780 * gray + 0.0593 * gray;
        let (rt, gt, bt) = apply_split_toning(gray, gray, gray, 215.0, 40.0, 45.0, 0.0, 0.0);
        let lum_after = 0.2627 * rt + 0.6780 * gt + 0.0593 * bt;
        assert!((lum_after - lum_before).abs() < 1e-3, "Photometric luminance should be preserved");
    }

    #[test]
    fn test_detail_and_sharpening_math() {
        let (r0, g0, b0) = apply_detail_and_sharpening_rgb(0.5, 0.4, 0.3, 0.4, 0.3, 0.5, 0.3, 0.5, 0.0, 0.0, 0.0);
        assert_eq!((r0, g0, b0), (0.5, 0.4, 0.3));

        let y_c = 0.8;
        let (y_l, y_r, y_t, y_b) = (0.2, 0.8, 0.5, 0.5);
        let (r_sharp, _g_sharp, _b_sharp) = apply_detail_and_sharpening_rgb(0.8, 0.8, 0.8, y_c, y_l, y_r, y_t, y_b, 50.0, 0.0, 0.0);
        assert!(r_sharp > 0.8);

        let y_flat_c = 0.505;
        let (y_flat_l, y_flat_r, y_flat_t, y_flat_b) = (0.500, 0.500, 0.500, 0.500);
        let (r_unmasked, _, _) = apply_detail_and_sharpening_rgb(0.505, 0.505, 0.505, y_flat_c, y_flat_l, y_flat_r, y_flat_t, y_flat_b, 50.0, 0.0, 0.0);
        let (r_masked, _, _) = apply_detail_and_sharpening_rgb(0.505, 0.505, 0.505, y_flat_c, y_flat_l, y_flat_r, y_flat_t, y_flat_b, 50.0, 30.0, 0.0);
        assert!(r_unmasked > 0.505);
        assert_eq!(r_masked, 0.505);

        let (r_nr, _, _) = apply_detail_and_sharpening_rgb(0.505, 0.505, 0.505, y_flat_c, y_flat_l, y_flat_r, y_flat_t, y_flat_b, 0.0, 30.0, 80.0);
        assert!(r_nr < 0.505);
    }

    #[test]
    fn test_tone_curves_export() {
        let tmp_in = "test_curve_in.jpg";
        let tmp_out_flat = "test_curve_flat.jpg";
        let tmp_out_curved = "test_curve_curved.jpg";

        let img = RgbImage::from_fn(20, 20, |_x, _y| {
            Rgb([128, 128, 128])
        });
        img.save(tmp_in).unwrap();

        let edits_flat = EditSettings::default();
        export_photo_file(tmp_in, tmp_out_flat, &edits_flat, None, 95, None, ExportColorSpace::Srgb, ExportRenderingIntent::RelativeColorimetric, false).unwrap();

        let mut edits_curved = EditSettings::default();
        edits_curved.curves.master = vec![
            crate::curve::CurvePointRecord { x: 0.0, y: 0.0 },
            crate::curve::CurvePointRecord { x: 0.5, y: 0.8 },
            crate::curve::CurvePointRecord { x: 1.0, y: 1.0 },
        ];
        export_photo_file(tmp_in, tmp_out_curved, &edits_curved, None, 95, None, ExportColorSpace::Srgb, ExportRenderingIntent::RelativeColorimetric, false).unwrap();

        let flat_px = image::open(tmp_out_flat).unwrap().to_rgb8().get_pixel(10, 10)[0];
        let curved_px = image::open(tmp_out_curved).unwrap().to_rgb8().get_pixel(10, 10)[0];

        assert!(curved_px > flat_px + 20);

        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out_flat);
        let _ = std::fs::remove_file(tmp_out_curved);
    }

    #[test]
    fn test_display_p3_export_and_icc() {
        let tmp_in = "test_p3_in.jpg";
        let tmp_out = "test_p3_out.jpg";

        let img = RgbImage::from_fn(30, 30, |x, y| {
            Rgb([x as u8 * 8, y as u8 * 8, 200])
        });
        img.save(tmp_in).unwrap();

        let edits = EditSettings::default();
        export_photo_file(
            tmp_in,
            tmp_out,
            &edits,
            None,
            95,
            None,
            ExportColorSpace::DisplayP3,
            ExportRenderingIntent::Perceptual,
            false,
        ).unwrap();

        let bytes = std::fs::read(tmp_out).unwrap();
        let marker = [0xff, 0xe2];
        let found_app2 = bytes.windows(marker.len()).any(|w| w == marker);
        assert!(found_app2);

        if cfg!(target_os = "macos") {
            let sips_out = std::process::Command::new("sips")
                .arg("-g")
                .arg("profile")
                .arg(tmp_out)
                .output();
            if let Ok(output) = sips_out {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains("P3") || stdout.contains("RGB"));
            }
        }

        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out);
    }

    #[test]
    fn test_batch_export_rayon() {
        let mut items = Vec::new();
        let count = 4;
        for i in 0..count {
            let in_path = format!("test_batch_in_{}.jpg", i);
            let out_path = format!("test_batch_out_{}.jpg", i);
            let img = RgbImage::from_fn(20, 20, |_x, _y| {
                Rgb([(i * 50) as u8, 120, 150])
            });
            img.save(&in_path).unwrap();

            items.push(BatchExportItemRecord {
                input_path: in_path,
                output_path: out_path,
                edits: EditSettings::default(),
                lens_coeffs: None,
                quality: 90,
                max_dimension: Some(40),
                color_space: if i % 2 == 0 { ExportColorSpace::Srgb } else { ExportColorSpace::DisplayP3 },
                rendering_intent: ExportRenderingIntent::RelativeColorimetric,
                ultra_hdr: false,
                demosaic_quality: None,
                chemical_edge_diffusion: None,
                render_tier: None,
            });
        }

        let results = export_batch_photos(items.clone());
        assert_eq!(results.len(), count);
        for res in &results {
            assert!(res.success, "Batch export item failed: {:?}", res.error_message);
            assert!(std::path::Path::new(&res.output_path).exists());
            let _ = std::fs::remove_file(&res.input_path);
            let _ = std::fs::remove_file(&res.output_path);
        }
    }

    #[test]
    fn test_ultra_hdr_export_pipeline() {
        let tmp_in = "test_uhdr_in.jpg";
        let tmp_out = "test_uhdr_out.jpg";

        let img = RgbImage::from_fn(30, 30, |_x, _y| {
            Rgb([240, 240, 255])
        });
        img.save(tmp_in).unwrap();

        let mut edits = EditSettings::default();
        edits.exposure = 2.0;

        export_photo_file(
            tmp_in,
            tmp_out,
            &edits,
            None,
            90,
            None,
            ExportColorSpace::Srgb,
            ExportRenderingIntent::RelativeColorimetric,
            true,
        ).unwrap();

        let bytes = std::fs::read(tmp_out).unwrap();
        assert!(crate::gainmap::has_gain_map(&bytes));
        let extracted = crate::gainmap::extract_gain_map(&bytes);
        assert!(extracted.is_some());

        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out);
    }

    #[test]
    fn test_recording_emulsion_and_subtractive_grain_math() {
        use crate::color::emulsion::{apply_recording_emulsion, EmulsionArchetype, EnlargerFilterMode};

        // 1. Clean bypass when None
        let (r0, g0, b0) = apply_recording_emulsion(0.5, 0.5, 0.5, EmulsionArchetype::None, 0.0, EnlargerFilterMode::NeutralPanchromatic);
        assert_eq!((r0, g0, b0), (0.5, 0.5, 0.5));

        // 2. Sensitometric toe absorption: pure black stays 0.0
        let (rb, gb, bb) = apply_recording_emulsion(0.0, 0.0, 0.0, EmulsionArchetype::RochesterPortrait400, 0.0, EnlargerFilterMode::NeutralPanchromatic);
        assert_eq!((rb, gb, bb), (0.0, 0.0, 0.0));

        // 3. Push/Pull photographic exposure trim (2^EV)
        let (r_base, _, _) = apply_recording_emulsion(0.3, 0.3, 0.3, EmulsionArchetype::RochesterPortrait400, 0.0, EnlargerFilterMode::NeutralPanchromatic);
        let (r_push, _, _) = apply_recording_emulsion(0.3, 0.3, 0.3, EmulsionArchetype::RochesterPortrait400, 1.0, EnlargerFilterMode::NeutralPanchromatic);
        assert!(r_push > r_base, "Pushed exposure must produce higher density than normal");
    }

    #[test]
    fn test_chromatic_median_filtering() {
        // 1. Uniform patch is unaffected
        let mut uniform = vec![[0.4, 0.4, 0.4]; 25];
        apply_chromatic_median_filter(&mut uniform, 5, 5, 1);
        for p in &uniform {
            assert_eq!(*p, [0.4, 0.4, 0.4]);
        }

        // 2. Spatial green step edge remains 100% sharp
        let mut edge = Vec::with_capacity(25);
        for _y in 0..5 {
            for x in 0..5 {
                let g = if x < 2 { 0.2 } else { 0.8 };
                edge.push([g, g, g]);
            }
        }
        apply_chromatic_median_filter(&mut edge, 5, 5, 1);
        for x in 0..5 {
            let expected_g = if x < 2 { 0.2 } else { 0.8 };
            assert!((edge[2 * 5 + x][1] - expected_g).abs() < 1e-6);
        }

        // 3. Isolated red/blue chromatic outlier is rejected by median filter
        let mut outlier_grid = vec![[0.5, 0.5, 0.5]; 25];
        // Center pixel (2, 2) has a +0.2 red checkerboard spike:
        outlier_grid[2 * 5 + 2][0] = 0.7; // R = 0.7, G = 0.5 -> C_r = +0.2
        apply_chromatic_median_filter(&mut outlier_grid, 5, 5, 1);
        // Center pixel's R should be filtered back to 0.5 by median filter
        assert!((outlier_grid[2 * 5 + 2][0] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_16bit_tiff_export_precision() {
        let tmp_in = "test_tiff16_in.jpg";
        let tmp_out = "test_tiff16_out.tif";

        let img = RgbImage::from_fn(40, 40, |x, y| {
            Rgb([((x * 6) % 256) as u8, ((y * 6) % 256) as u8, 128])
        });
        img.save(tmp_in).unwrap();

        let edits = EditSettings {
            exposure: 0.5,
            contrast: 15.0,
            ..Default::default()
        };

        let res = export_photo_file_advanced(
            tmp_in,
            tmp_out,
            &edits,
            None,
            90,
            Some(20),
            ExportColorSpace::DisplayP3,
            ExportRenderingIntent::RelativeColorimetric,
            false,
            crate::DemosaicQuality::Archival,
            false,
            crate::RenderTier::ArchivalExport,
        );
        assert!(res.is_ok(), "Export failed: {:?}", res.err());

        let decoded = image::open(tmp_out).expect("Failed to open exported TIFF");
        assert_eq!(decoded.width(), 20);
        assert_eq!(decoded.height(), 20);

        match decoded {
            image::DynamicImage::ImageRgb16(rgb16) => {
                let mut found_high_precision = false;
                for p in rgb16.pixels() {
                    // In 16-bit, values should span into the high range > 255
                    if p.0[0] > 255 || p.0[1] > 255 || p.0[2] > 255 {
                        found_high_precision = true;
                        break;
                    }
                }
                assert!(found_high_precision, "TIFF must contain true 16-bit samples (>255)");
            }
            other => panic!("Expected ImageRgb16 for TIFF export, got {:?}", other.color()),
        }

        let _ = std::fs::remove_file(tmp_in);
        let _ = std::fs::remove_file(tmp_out);
    }

    #[test]
    fn test_rcd_archival_demosaic_execution() {
        let w = 8;
        let h = 8;
        let mut raw = vec![0.0f32; w * h];
        for y in 0..h {
            for x in 0..w {
                raw[y * w + x] = match (y % 2, x % 2) {
                    (0, 0) => 0.6, // R
                    (0, 1) => 0.5, // Gr
                    (1, 0) => 0.5, // Gb
                    _ => 0.4,      // B
                };
            }
        }

        let frame = crate::raw_decoder::RawSensorFrame {
            full_width: w,
            full_height: h,
            active_x: 0,
            active_y: 0,
            active_width: w,
            active_height: h,
            cfa_pattern: crate::raw_decoder::CfaPattern::Rggb,
            black_levels: [0.0; 4],
            white_levels: [1.0; 4],
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            color_matrix: Some([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]),
            color_matrix_tungsten: None,
            forward_matrix_tungsten: None,
            forward_matrix_daylight: None,
            camera_model: "Synthetic Test".to_string(),
            baseline_exposure: 0.0,
            noise_profile: None,
            sensor_data: vec![1000u16; w * h],
        };

        let demosaiced = rcd_archival_demosaic(&raw, w, h, 0, 0, w, h, &frame);
        assert_eq!(demosaiced.len(), w * h);
        for p in &demosaiced {
            assert!(!p[0].is_nan() && !p[1].is_nan() && !p[2].is_nan());
            assert!(p[0] >= 0.0 && p[1] >= 0.0 && p[2] >= 0.0);
        }
    }

    #[test]
    fn test_lateral_chemical_diffusion_mackie_lines() {
        // 1. Uniform patch: zero change
        let w = 16;
        let h = 16;
        let mut uniform = vec![0.5f32; w * h * 3];
        apply_lateral_chemical_diffusion(&mut uniform, w, h, 0.15);
        for &val in &uniform {
            assert!((val - 0.5).abs() < 1e-4, "Uniform patch must remain unchanged");
        }

        // 2. Step edge: Mackie lines (border contrast enhancement / developer exhaustion)
        // Left half: 0.2 (shadow), Right half: 0.8 (highlight)
        let mut edge = vec![0.0f32; w * h * 3];
        for y in 0..h {
            for x in 0..w {
                let v = if x < 8 { 0.2 } else { 0.8 };
                let idx = (y * w + x) * 3;
                edge[idx] = v;
                edge[idx + 1] = v;
                edge[idx + 2] = v;
            }
        }

        apply_lateral_chemical_diffusion(&mut edge, w, h, 0.20);

        // In the middle row (away from top/bottom borders):
        let mid_y = h / 2;
        let p_shadow_near_edge = edge[(mid_y * w + 7) * 3]; // x=7: shadow border
        let p_shadow_far = edge[(mid_y * w + 1) * 3];       // x=1: deep shadow

        let p_highlight_near_edge = edge[(mid_y * w + 8) * 3]; // x=8: highlight border
        let p_highlight_far = edge[(mid_y * w + 14) * 3];      // x=14: uniform highlight

        // Adjacent to the bright area, developer is exhausted by high silver reduction,
        // so the shadow border receives less active developer: density drops / undershoot occurs.
        assert!(
            p_shadow_near_edge <= p_shadow_far,
            "Shadow edge must exhibit bromide inhibition / undershoot (got {} vs far {})",
            p_shadow_near_edge, p_shadow_far
        );

        // Conversely, fresh developer diffuses from the unexposed shadow across into the highlight border,
        // producing the Mackie line (overshoot / adjacency fringe):
        assert!(
            p_highlight_near_edge >= p_highlight_far,
            "Highlight edge must exhibit fresh developer boost / overshoot (got {} vs far {})",
            p_highlight_near_edge, p_highlight_far
        );
    }
}


