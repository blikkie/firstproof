//! Synthetic Test Chart Generator for Zero-Disk-I/O Parity Audits
//!
//! Generates deterministic, wide-gamut Linear Rec.2020 image buffers containing:
//! 1. Standardized 24-patch Macbeth ColorChecker (including calibrated skin tones, skies, primaries).
//! 2. 16-step achromatic neutral gray ramp (for white balance, exposure, and tone curve testing).
//! 3. High-contrast geometric grid lines and corner fiducials (for lens distortion warp testing).
//!
//! Enables high-speed dual-engine parity regression testing without checking in large proprietary RAW files.

use crate::color::{Color3, LINEAR_SRGB_TO_REC2020, srgb_gamma_to_linear};
use crate::export::{DemosaicedImage, f32_to_f16};

#[derive(Clone, Debug)]
pub struct SyntheticSamplePoint {
    pub name: &'static str,
    pub uv: [f32; 2],
    pub expected_rec2020: Color3,
}

pub struct SyntheticTestChart {
    pub width: u32,
    pub height: u32,
    pub rec2020_data: Vec<f32>, // RGB floats in Linear Rec.2020
    pub sample_points: Vec<SyntheticSamplePoint>,
}

const MACBETH_24_SRGB: [(&str, [u8; 3]); 24] = [
    ("Dark Skin",     [115,  82,  68]),
    ("Light Skin",    [194, 150, 130]),
    ("Blue Sky",      [ 98, 122, 157]),
    ("Foliage",       [ 87, 108,  67]),
    ("Blue Flower",   [133, 128, 177]),
    ("Bluish Green",  [103, 189, 170]),
    ("Orange",        [214, 126,  44]),
    ("Purplish Blue", [ 80,  91, 166]),
    ("Moderate Red",  [193,  90,  99]),
    ("Purple",        [ 94,  60, 108]),
    ("Yellow Green",  [157, 188,  64]),
    ("Orange Yellow", [224, 163,  46]),
    ("Blue",          [ 56,  61, 150]),
    ("Green",         [ 70, 148,  73]),
    ("Red",           [175,  54,  60]),
    ("Yellow",        [231, 199,  31]),
    ("Magenta",       [187,  86, 149]),
    ("Cyan",          [  8, 133, 161]),
    ("White 9.5",     [243, 243, 242]),
    ("Neutral 8",     [200, 200, 200]),
    ("Neutral 6.5",   [160, 160, 160]),
    ("Neutral 5",     [122, 122, 121]),
    ("Neutral 3.5",   [ 85,  85,  85]),
    ("Black 2",       [ 52,  52,  52]),
];

impl SyntheticTestChart {
    /// Generates a standardized synthetic test chart at the given dimensions (e.g. 1200 x 800).
    pub fn new(width: u32, height: u32) -> Self {
        let mut rec2020_data = vec![0.08f32; (width * height * 3) as usize];
        let mut sample_points = Vec::new();

        // 1. Render 24 Macbeth Patches in top half (4 rows x 6 cols)
        let chart_left = (width as f32 * 0.05) as usize;
        let chart_top = (height as f32 * 0.08) as usize;
        let chart_w = (width as f32 * 0.90) as usize;
        let chart_h = (height as f32 * 0.42) as usize;
        let patch_w = chart_w / 6;
        let patch_h = chart_h / 4;

        for row in 0..4 {
            for col in 0..6 {
                let idx = row * 6 + col;
                let (name, srgb_8bit) = MACBETH_24_SRGB[idx];
                
                let r_lin = srgb_gamma_to_linear((srgb_8bit[0] as f32) / 255.0);
                let g_lin = srgb_gamma_to_linear((srgb_8bit[1] as f32) / 255.0);
                let b_lin = srgb_gamma_to_linear((srgb_8bit[2] as f32) / 255.0);
                let rec2020 = LINEAR_SRGB_TO_REC2020.multiply_vector(Color3::new(r_lin, g_lin, b_lin));

                let px_start = chart_left + col * patch_w;
                let py_start = chart_top + row * patch_h;

                // Center of patch for sampling
                let center_u = (px_start + patch_w / 2) as f32 / width as f32;
                let center_v = (py_start + patch_h / 2) as f32 / height as f32;
                sample_points.push(SyntheticSamplePoint {
                    name,
                    uv: [center_u, center_v],
                    expected_rec2020: rec2020,
                });

                // Fill patch pixels with 2px border
                for y in (py_start + 2)..(py_start + patch_h - 2) {
                    for x in (px_start + 2)..(px_start + patch_w - 2) {
                        let p_idx = (y * width as usize + x) * 3;
                        rec2020_data[p_idx] = rec2020.r();
                        rec2020_data[p_idx + 1] = rec2020.g();
                        rec2020_data[p_idx + 2] = rec2020.b();
                    }
                }
            }
        }

        // 2. Render 16-Step Neutral Gray Ramp in mid band
        let ramp_left = (width as f32 * 0.05) as usize;
        let ramp_top = (height as f32 * 0.54) as usize;
        let ramp_w = (width as f32 * 0.90) as usize;
        let ramp_h = (height as f32 * 0.12) as usize;
        let step_w = ramp_w / 16;

        for step in 0..16 {
            // EV levels from -6 EV (0.0028) to +1.5 EV (0.509)
            let ev = -6.0 + (step as f32) * 0.5;
            let linear_luma = (0.18 * 2.0f32.powf(ev)).clamp(0.001, 10.0);
            let rec2020 = Color3::new(linear_luma, linear_luma, linear_luma);

            let px_start = ramp_left + step * step_w;
            for y in ramp_top..(ramp_top + ramp_h) {
                for x in px_start..(px_start + step_w) {
                    let p_idx = (y * width as usize + x) * 3;
                    rec2020_data[p_idx] = rec2020.r();
                    rec2020_data[p_idx + 1] = rec2020.g();
                    rec2020_data[p_idx + 2] = rec2020.b();
                }
            }
        }

        // 3. Render High-Contrast Geometric Grid Lines & Corner Fiducials in bottom band
        let grid_top = (height as f32 * 0.70) as usize;
        let grid_bottom = (height as f32 * 0.95) as usize;

        for y in grid_top..grid_bottom {
            for x in 0..(width as usize) {
                let is_grid_line = (x % 40 == 0) || (y % 40 == 0);
                let val = if is_grid_line { 0.85f32 } else { 0.15f32 };
                let p_idx = (y * width as usize + x) * 3;
                rec2020_data[p_idx] = val;
                rec2020_data[p_idx + 1] = val;
                rec2020_data[p_idx + 2] = val;
            }
        }

        // Corner fiducials (top-left, top-right, bottom-left, bottom-right)
        let corners = [
            ("Corner TL", 0.05f32, 0.05f32),
            ("Corner TR", 0.95f32, 0.05f32),
            ("Corner BL", 0.05f32, 0.95f32),
            ("Corner BR", 0.95f32, 0.95f32),
        ];
        for (name, u, v) in corners {
            let cx = (u * width as f32) as usize;
            let cy = (v * height as f32) as usize;
            sample_points.push(SyntheticSamplePoint {
                name,
                uv: [u, v],
                expected_rec2020: Color3::new(0.85, 0.85, 0.85),
            });
            for dy in -3..=3 {
                for dx in -3..=3 {
                    let x = (cx as i32 + dx).clamp(0, width as i32 - 1) as usize;
                    let y = (cy as i32 + dy).clamp(0, height as i32 - 1) as usize;
                    let p_idx = (y * width as usize + x) * 3;
                    rec2020_data[p_idx] = 0.85;
                    rec2020_data[p_idx + 1] = 0.85;
                    rec2020_data[p_idx + 2] = 0.85;
                }
            }
        }

        Self {
            width,
            height,
            rec2020_data,
            sample_points,
        }
    }

    /// Converts into a DemosaicedImage suitable for export pipeline processing.
    pub fn to_decoded_image(&self) -> DemosaicedImage {
        DemosaicedImage {
            width: self.width as usize,
            height: self.height as usize,
            data: self.rec2020_data.clone(),
            is_raw: true,
            as_shot_temperature: Some(5500.0),
            as_shot_tint: Some(0.0),
            baseline_sharpness: 1.0,
            profile_tone_curve: None,
            warp_rectilinear: None,
        }
    }

    /// Converts into a half-float (f16) RGBA buffer simulating GPU binned preview input.
    pub fn to_binned_f16_rgba(&self) -> Vec<u8> {
        let pixel_count = (self.width * self.height) as usize;
        let mut buffer = vec![0u8; pixel_count * 8]; // 4 channels x 2 bytes (f16)

        for i in 0..pixel_count {
            let r = self.rec2020_data[i * 3];
            let g = self.rec2020_data[i * 3 + 1];
            let b = self.rec2020_data[i * 3 + 2];
            let a = 1.0f32;

            let r_f16 = f32_to_f16(r);
            let g_f16 = f32_to_f16(g);
            let b_f16 = f32_to_f16(b);
            let a_f16 = f32_to_f16(a);

            let buf_idx = i * 8;
            buffer[buf_idx..buf_idx + 2].copy_from_slice(&r_f16.to_ne_bytes());
            buffer[buf_idx + 2..buf_idx + 4].copy_from_slice(&g_f16.to_ne_bytes());
            buffer[buf_idx + 4..buf_idx + 6].copy_from_slice(&b_f16.to_ne_bytes());
            buffer[buf_idx + 6..buf_idx + 8].copy_from_slice(&a_f16.to_ne_bytes());
        }

        buffer
    }
}
