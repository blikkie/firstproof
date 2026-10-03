use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use image::{ColorType, ImageBuffer, Rgb, RgbImage};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::tiff::TiffEncoder;

use crate::{EditSettings, ExportColorSpace, ExportRenderingIntent};
use crate::color::linear_to_srgb_gamma;
use super::decode::{decode_image, sample_bilinear};
use super::pipeline::{apply_detail_and_sharpening_rgb, apply_edits};
use super::icc::{create_display_p3_profile, inject_icc_profile_into_jpeg};

/// Exports an edited photo to JPEG or 16-bit TIFF on disk with optional resizing, color space management, and Lensfun distortion correction.
pub fn export_photo_file<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P,
    output_path: Q,
    edits: &EditSettings,
    lens_distortion: Option<(&[f32], &str)>,
    quality: u8,
    max_dimension: Option<u32>,
    color_space: ExportColorSpace,
    rendering_intent: ExportRenderingIntent,
    ultra_hdr: bool,
) -> Result<(), String> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    
    let mut image = decode_image(input_path)?;
    apply_edits(&mut image, edits);
    
    let orig_width = image.width as u32;
    let orig_height = image.height as u32;
    
    let crop_l = edits.crop_left.clamp(0.0, 0.99);
    let crop_t = edits.crop_top.clamp(0.0, 0.99);
    let crop_r = edits.crop_right.clamp(crop_l + 0.01, 1.0);
    let crop_b = edits.crop_bottom.clamp(crop_t + 0.01, 1.0);
    let crop_w_frac = crop_r - crop_l;
    let crop_h_frac = crop_b - crop_t;

    let effective_orig_w = ((orig_width as f32) * crop_w_frac).round().max(1.0) as u32;
    let effective_orig_h = ((orig_height as f32) * crop_h_frac).round().max(1.0) as u32;

    // Calculate target dimensions
    let (target_w, target_h) = if let Some(max_dim) = max_dimension {
        if max_dim > 0 && (effective_orig_w > max_dim || effective_orig_h > max_dim) {
            if effective_orig_w >= effective_orig_h {
                let w = max_dim;
                let h = ((effective_orig_h as f32 * max_dim as f32) / effective_orig_w as f32).round() as u32;
                (w, h.max(1))
            } else {
                let h = max_dim;
                let w = ((effective_orig_w as f32 * max_dim as f32) / effective_orig_h as f32).round() as u32;
                (w.max(1), h)
            }
        } else {
            (effective_orig_w, effective_orig_h)
        }
    } else {
        (effective_orig_w, effective_orig_h)
    };
    
    // Build 8-bit RGB image buffer with geometric warp and bilinear filtering
    let mut rgb_img: RgbImage = ImageBuffer::new(target_w, target_h);
    let mut sdr_luma = if ultra_hdr { Vec::with_capacity((target_w * target_h) as usize) } else { Vec::new() };
    let mut hdr_luma = if ultra_hdr { Vec::with_capacity((target_w * target_h) as usize) } else { Vec::new() };
    
    let aspect_ratio = (orig_width as f32) / (orig_height as f32);
    let angle_rad = edits.crop_angle * std::f32::consts::PI / 180.0;
    let cos_theta = angle_rad.cos();
    let sin_theta = angle_rad.sin();
    
    let c_v = edits.perspective_vertical * 0.005;
    let c_h = edits.perspective_horizontal * 0.005;
    
    let mut auto_crop_scale = 1.0f32;
    if edits.crop_constrain {
        let corners = [
            (aspect_ratio, 1.0),
            (aspect_ratio, -1.0),
            (-aspect_ratio, -1.0),
            (-aspect_ratio, 1.0),
        ];
        for (px, py) in corners {
            let rx = px * cos_theta - py * sin_theta;
            let ry = px * sin_theta + py * cos_theta;
            let req = (rx.abs() / aspect_ratio).max(ry.abs()) - (c_v * ry + c_h * rx);
            if req > auto_crop_scale {
                auto_crop_scale = req;
            }
        }
    }
    
    let (has_dist, k1, k2, k3, is_ptlens) = if let Some((coeffs, model)) = lens_distortion {
        if !coeffs.is_empty() {
            let k1 = coeffs[0];
            let k2 = if coeffs.len() > 1 { coeffs[1] } else { 0.0 };
            let k3 = if coeffs.len() > 2 { coeffs[2] } else { 0.0 };
            (true, k1, k2, k3, model == "ptlens")
        } else {
            (false, 0.0, 0.0, 0.0, false)
        }
    } else {
        (false, 0.0, 0.0, 0.0, false)
    };
    
    let active_lut: Option<crate::lut::CubeLut> = if let Some(ref lut_name) = edits.lut_name {
        if edits.lut_intensity > 0.0 {
            crate::lut::load_cube_lut_file(lut_name).ok()
        } else {
            None
        }
    } else {
        None
    };
    let lut_intensity = edits.lut_intensity.clamp(0.0, 1.0);



    for ty in 0..target_h {
        for tx in 0..target_w {
            let u = (tx as f32 + 0.5) / (target_w as f32);
            let v = (ty as f32 + 0.5) / (target_h as f32);
            
            // Map output (u, v) into cropped sub-box of the composition
            let u_crop = crop_l + u * crop_w_frac;
            let v_crop = crop_t + v * crop_h_frac;
            
            // 1. Straighten and auto-crop zoom (normalized to Lensfun standard coordinates)
            let centered_x = (u_crop - 0.5) * 2.0 * aspect_ratio / auto_crop_scale;
            let centered_y = (v_crop - 0.5) * 2.0 / auto_crop_scale;
            
            let rot_x = centered_x * cos_theta - centered_y * sin_theta;
            let rot_y = centered_x * sin_theta + centered_y * cos_theta;
            
            // Perspective keystone (backward projective homography)
            let denom = (1.0 + c_v * rot_y + c_h * rot_x).max(0.1);
            let mut geom_x = rot_x / denom;
            let mut geom_y = rot_y / denom;
            
            // 2. Lensfun distortion model
            if has_dist {
                let r2 = geom_x * geom_x + geom_y * geom_y;
                let r = r2.sqrt();
                if r > 0.0001 {
                    let rd = if is_ptlens {
                        let a = k1;
                        let b = k2;
                        let c = k3;
                        let d = 1.0 - (a + b + c);
                        let r3 = r2 * r;
                        let r4 = r2 * r2;
                        a * r4 + b * r3 + c * r2 + d * r
                    } else {
                        // poly3 model: r_d = r * (1 + k1*r^2 + k2*r^4)
                        let r4 = r2 * r2;
                        r * (1.0 + k1 * r2 + k2 * r4)
                    };
                    let scale_factor = rd / r;
                    geom_x *= scale_factor;
                    geom_y *= scale_factor;
                }
            }
            
            let sx = (geom_x / (2.0 * aspect_ratio) + 0.5) * (orig_width as f32);
            let sy = (geom_y / 2.0 + 0.5) * (orig_height as f32);
            
            let (mut r_lin, mut g_lin, mut b_lin) = if !edits.crop_constrain && (sx < 0.0 || sx >= (orig_width - 1) as f32 || sy < 0.0 || sy >= (orig_height - 1) as f32) {
                (0.0, 0.0, 0.0)
            } else {
                sample_bilinear(&image, sx, sy)
            };

            // 3. 3D Look-Up Table (display-referred linear stage post-tone-mapping)
            if let Some(ref lut) = active_lut {
                if lut_intensity > 0.0 {
                    let lut_out = lut.interpolate_tetrahedral([r_lin, g_lin, b_lin]);
                    r_lin = r_lin * (1.0 - lut_intensity) + lut_out[0] * lut_intensity;
                    g_lin = g_lin * (1.0 - lut_intensity) + lut_out[1] * lut_intensity;
                    b_lin = b_lin * (1.0 - lut_intensity) + lut_out[2] * lut_intensity;
                }
            }

            // 3b. Detail & Sharpening / Noise Reduction / Stochastic Emulsion Grain
            if edits.sharpening_amount > 0.0 || edits.noise_reduction > 0.0 || edits.emulsion_grain_mode != crate::EmulsionGrainMode::Off {
                let r_step = edits.sharpening_radius.max(0.5);
                let (rl, gl, bl) = sample_bilinear(&image, sx - r_step, sy);
                let (rr, gr, br) = sample_bilinear(&image, sx + r_step, sy);
                let (rt, gt, bt) = sample_bilinear(&image, sx, sy - r_step);
                let (rb, gb, bb) = sample_bilinear(&image, sx, sy + r_step);
                let y_c = 0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin;
                let y_l = 0.2126 * rl + 0.7152 * gl + 0.0722 * bl;
                let y_r = 0.2126 * rr + 0.7152 * gr + 0.0722 * br;
                let y_t = 0.2126 * rt + 0.7152 * gt + 0.0722 * bt;
                let y_b = 0.2126 * rb + 0.7152 * gb + 0.0722 * bb;
                if edits.sharpening_amount > 0.0 || edits.noise_reduction > 0.0 {
                    let (r_sharp, g_sharp, b_sharp) = apply_detail_and_sharpening_rgb(
                        r_lin, g_lin, b_lin,
                        y_c, y_l, y_r, y_t, y_b,
                        edits.sharpening_amount,
                        edits.sharpening_masking,
                        edits.noise_reduction,
                    );
                    r_lin = r_sharp;
                    g_lin = g_sharp;
                    b_lin = b_sharp;
                }
                if edits.emulsion_grain_mode != crate::EmulsionGrainMode::Off {
                    let (gr, gg, gb) = firstproof_engine::color::grain::apply_stochastic_emulsion_grain(
                        r_lin, g_lin, b_lin,
                        y_c, y_l, y_r, y_t, y_b,
                        tx as f32 / target_w as f32, ty as f32 / target_h as f32,
                        edits.emulsion_grain_mode,
                        edits.grain_scale,
                        edits.emulsion_archetype,
                    );
                    r_lin = gr;
                    g_lin = gg;
                    b_lin = gb;
                }
            }

            // TERMINAL STEP: Convert Linear to Display Gamma sRGB
            let r_disp = linear_to_srgb_gamma(r_lin).clamp(0.0, 1.0);
            let g_disp = linear_to_srgb_gamma(g_lin).clamp(0.0, 1.0);
            let b_disp = linear_to_srgb_gamma(b_lin).clamp(0.0, 1.0);

            let srgb8 = [
                (r_disp * 255.0).round() as u8,
                (g_disp * 255.0).round() as u8,
                (b_disp * 255.0).round() as u8,
            ];
            
            rgb_img.put_pixel(tx, ty, Rgb(srgb8));

            if ultra_hdr {
                let y_sdr = 0.2126 * r_disp.clamp(0.0, 1.0) + 0.7152 * g_disp.clamp(0.0, 1.0) + 0.0722 * b_disp.clamp(0.0, 1.0);
                let y_hdr = 0.2126 * r_lin.max(0.0) + 0.7152 * g_lin.max(0.0) + 0.0722 * b_lin.max(0.0);
                sdr_luma.push(y_sdr);
                hdr_luma.push(y_hdr);
            }
        }
    }
    
    // Apply 90/180/270 degree rotation and horizontal/vertical flips
    if edits.rotation != 0 || edits.flip_horizontal || edits.flip_vertical {
        match edits.rotation {
            90 => rgb_img = image::imageops::rotate90(&rgb_img),
            180 => rgb_img = image::imageops::rotate180(&rgb_img),
            270 => rgb_img = image::imageops::rotate270(&rgb_img),
            _ => {}
        }
        if edits.flip_horizontal {
            rgb_img = image::imageops::flip_horizontal(&rgb_img);
        }
        if edits.flip_vertical {
            rgb_img = image::imageops::flip_vertical(&rgb_img);
        }
    }
    
    let final_w = rgb_img.width();
    let final_h = rgb_img.height();
    
    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Color Management: convert from working sRGB to target color space (Display P3) if requested
    let (target_rgb_bytes, icc_to_embed): (Vec<u8>, Vec<u8>) = if color_space == ExportColorSpace::DisplayP3 {
        let in_prof = lcms2::Profile::new_srgb();
        if let Ok(p3_prof) = create_display_p3_profile() {
            let intent = match rendering_intent {
                ExportRenderingIntent::Perceptual => lcms2::Intent::Perceptual,
                ExportRenderingIntent::RelativeColorimetric => lcms2::Intent::RelativeColorimetric,
            };
            if let Ok(transform) = lcms2::Transform::new(
                &in_prof,
                lcms2::PixelFormat::RGB_8,
                &p3_prof,
                lcms2::PixelFormat::RGB_8,
                intent,
            ) {
                let mut p3_bytes = vec![0u8; rgb_img.as_raw().len()];
                transform.transform_pixels(rgb_img.as_raw(), &mut p3_bytes);
                let icc_bytes = p3_prof.icc().unwrap_or_else(|_| crate::color::SRGB_ICC_PROFILE.to_vec());
                (p3_bytes, icc_bytes)
            } else {
                (rgb_img.as_raw().to_vec(), crate::color::SRGB_ICC_PROFILE.to_vec())
            }
        } else {
            (rgb_img.as_raw().to_vec(), crate::color::SRGB_ICC_PROFILE.to_vec())
        }
    } else {
        (rgb_img.as_raw().to_vec(), crate::color::SRGB_ICC_PROFILE.to_vec())
    };
    
    let ext = output_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("jpg")
        .to_lowercase();
        
    if ext == "tif" || ext == "tiff" {
        let file = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
        let mut writer = BufWriter::new(file);
        let encoder = TiffEncoder::new(&mut writer);
        encoder
            .encode(&target_rgb_bytes, final_w, final_h, ColorType::Rgb8)
            .map_err(|e| format!("Failed to encode TIFF: {}", e))?;
    } else {
        let mut jpeg_buf = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_buf, quality.clamp(1, 100));
        encoder
            .encode(&target_rgb_bytes, final_w, final_h, ColorType::Rgb8)
            .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
            
        let tagged_jpeg = inject_icc_profile_into_jpeg(&jpeg_buf, &icc_to_embed);
        let final_jpeg_bytes = if ultra_hdr && !sdr_luma.is_empty() {
            let meta = crate::gainmap::GainMapMetadata::default();
            let (_gm_img, gm_jpeg) = crate::gainmap::generate_gain_map(
                &sdr_luma,
                &hdr_luma,
                target_w as usize,
                target_h as usize,
                &meta,
            );
            crate::gainmap::package_ultra_hdr_jpeg(&tagged_jpeg, &gm_jpeg, &meta)
                .unwrap_or(tagged_jpeg)
        } else {
            tagged_jpeg
        };
        let mut file = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
        file.write_all(&final_jpeg_bytes).map_err(|e| format!("Failed to write JPEG: {}", e))?;
    }
    
    Ok(())
}
