// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Continuous Edge-Adaptive & Archival Residual Demosaicing Engine
//!
//! Provides mathematically rigorous, standards-compliant demosaicing for Bayer CFA sensors:
//! 1. Soft-Adaptive Hamilton-Adams demosaicing with smooth Hermite gradient blending.
//! 2. Archival Residual Color Difference (RCD) demosaicing with iterative green refinement.
//! 3. 2D Chromatic difference median filtering on (R - G) and (B - G) difference planes.

use crate::raw::{RawSensorFrame, CFA_COLOR_BLUE, CFA_COLOR_GREEN, CFA_COLOR_RED};

/// Soft-Adaptive Hamilton-Adams Bayer demosaicing with smooth Hermite directional blending.
///
/// Blends horizontal and vertical directional estimates using a continuous Hermite
/// interpolation curve ($u^2(3 - 2u)$) to eliminate zipper and maze artifacts.
pub fn soft_adaptive_demosaic(
    norm_pixels: &[f32],
    full_w: usize,
    full_h: usize,
    active_x: usize,
    active_y: usize,
    active_w: usize,
    active_h: usize,
    frame: &RawSensorFrame,
) -> Vec<[f32; 3]> {
    let sample = |fy: isize, fx: isize| -> f32 {
        let y_clamped = fy.clamp(0, (full_h - 1) as isize) as usize;
        let x_clamped = fx.clamp(0, (full_w - 1) as isize) as usize;
        norm_pixels[y_clamped * full_w + x_clamped]
    };

    let apron_w = active_w + 4;
    let apron_h = active_h + 4;
    let mut green_plane = vec![0.0f32; apron_w * apron_h];

    // Pass 1: Interpolate Green at Red and Blue photosites
    for ay in 0..apron_h {
        let fy = (active_y as isize + ay as isize - 2).clamp(0, (full_h - 1) as isize);
        let row_start = ay * apron_w;
        for ax in 0..apron_w {
            let fx = (active_x as isize + ax as isize - 2).clamp(0, (full_w - 1) as isize);
            let color = frame.color_at(fy as usize, fx as usize);
            if color == CFA_COLOR_GREEN {
                green_plane[row_start + ax] = sample(fy, fx);
            } else {
                let c = sample(fy, fx);
                let g_n = sample(fy - 1, fx);
                let g_s = sample(fy + 1, fx);
                let g_w = sample(fy, fx - 1);
                let g_e = sample(fy, fx + 1);

                let c_n2 = sample(fy - 2, fx);
                let c_s2 = sample(fy + 2, fx);
                let c_w2 = sample(fy, fx - 2);
                let c_e2 = sample(fy, fx + 2);

                let delta_h = (g_w - g_e).abs() + (2.0 * c - c_w2 - c_e2).abs();
                let delta_v = (g_n - g_s).abs() + (2.0 * c - c_n2 - c_s2).abs();

                let g_h = 0.5 * (g_w + g_e) + 0.25 * (2.0 * c - c_w2 - c_e2);
                let g_v = 0.5 * (g_n + g_s) + 0.25 * (2.0 * c - c_n2 - c_s2);

                let diff = delta_h - delta_v;
                let sum = delta_h + delta_v + 1e-5;
                let t = (diff / sum).clamp(-1.0, 1.0);
                let u = 0.5 + 0.5 * t;
                let w_v = u * u * (3.0 - 2.0 * u);
                let w_h = 1.0 - w_v;

                green_plane[row_start + ax] = (w_h * g_h + w_v * g_v).max(0.0);
            }
        }
    }

    let g_sample = |ay: isize, ax: isize| -> f32 {
        let y_clamped = ay.clamp(0, (apron_h - 1) as isize) as usize;
        let x_clamped = ax.clamp(0, (apron_w - 1) as isize) as usize;
        green_plane[y_clamped * apron_w + x_clamped]
    };

    // Pass 2: Reconstruct Red and Blue across the active region
    let mut out_rgb = vec![[0.0f32; 3]; active_w * active_h];

    for y in 0..active_h {
        let ay = (y + 2) as isize;
        let fy = (active_y + y) as isize;
        let row_start = y * active_w;

        for x in 0..active_w {
            let ax = (x + 2) as isize;
            let fx = (active_x + x) as isize;
            let color = frame.color_at(
                fy.clamp(0, (full_h - 1) as isize) as usize,
                fx.clamp(0, (full_w - 1) as isize) as usize,
            );
            let g = g_sample(ay, ax);

            let r: f32;
            let b: f32;

            if color == CFA_COLOR_RED {
                r = sample(fy, fx);
                let d_nw = sample(fy - 1, fx - 1) - g_sample(ay - 1, ax - 1);
                let d_ne = sample(fy - 1, fx + 1) - g_sample(ay - 1, ax + 1);
                let d_sw = sample(fy + 1, fx - 1) - g_sample(ay + 1, ax - 1);
                let d_se = sample(fy + 1, fx + 1) - g_sample(ay + 1, ax + 1);

                let grad_d1 = (sample(fy - 1, fx - 1) - sample(fy + 1, fx + 1)).abs()
                    + (g_sample(ay - 1, ax - 1) - g_sample(ay + 1, ax + 1)).abs();
                let grad_d2 = (sample(fy - 1, fx + 1) - sample(fy + 1, fx - 1)).abs()
                    + (g_sample(ay - 1, ax + 1) - g_sample(ay + 1, ax - 1)).abs();

                let diff_d = grad_d1 - grad_d2;
                let sum_d = grad_d1 + grad_d2 + 1e-5;
                let t_d = (diff_d / sum_d).clamp(-1.0, 1.0);
                let u_d = 0.5 + 0.5 * t_d;
                let w_d2 = u_d * u_d * (3.0 - 2.0 * u_d);
                let w_d1 = 1.0 - w_d2;

                let b_d1 = 0.5 * (d_nw + d_se);
                let b_d2 = 0.5 * (d_ne + d_sw);
                b = (g + w_d1 * b_d1 + w_d2 * b_d2).max(0.0);
            } else if color == CFA_COLOR_BLUE {
                b = sample(fy, fx);
                let d_nw = sample(fy - 1, fx - 1) - g_sample(ay - 1, ax - 1);
                let d_ne = sample(fy - 1, fx + 1) - g_sample(ay - 1, ax + 1);
                let d_sw = sample(fy + 1, fx - 1) - g_sample(ay + 1, ax - 1);
                let d_se = sample(fy + 1, fx + 1) - g_sample(ay + 1, ax + 1);

                let grad_d1 = (sample(fy - 1, fx - 1) - sample(fy + 1, fx + 1)).abs()
                    + (g_sample(ay - 1, ax - 1) - g_sample(ay + 1, ax + 1)).abs();
                let grad_d2 = (sample(fy - 1, fx + 1) - sample(fy + 1, fx - 1)).abs()
                    + (g_sample(ay - 1, ax + 1) - g_sample(ay + 1, ax - 1)).abs();

                let diff_d = grad_d1 - grad_d2;
                let sum_d = grad_d1 + grad_d2 + 1e-5;
                let t_d = (diff_d / sum_d).clamp(-1.0, 1.0);
                let u_d = 0.5 + 0.5 * t_d;
                let w_d2 = u_d * u_d * (3.0 - 2.0 * u_d);
                let w_d1 = 1.0 - w_d2;

                let r_d1 = 0.5 * (d_nw + d_se);
                let r_d2 = 0.5 * (d_ne + d_sw);
                r = (g + w_d1 * r_d1 + w_d2 * r_d2).max(0.0);
            } else {
                let fy_u = fy.clamp(0, (full_h - 1) as isize) as usize;
                let fx_prev = (fx - 1).clamp(0, (full_w - 1) as isize) as usize;
                let fx_next = (fx + 1).clamp(0, (full_w - 1) as isize) as usize;
                let is_r_horizontal = frame.color_at(fy_u, fx_prev) == CFA_COLOR_RED
                    || frame.color_at(fy_u, fx_next) == CFA_COLOR_RED;

                let d_w = sample(fy, fx - 1) - g_sample(ay, ax - 1);
                let d_e = sample(fy, fx + 1) - g_sample(ay, ax + 1);
                let d_n = sample(fy - 1, fx) - g_sample(ay - 1, ax);
                let d_s = sample(fy + 1, fx) - g_sample(ay + 1, ax);

                let d_h = 0.5 * (d_w + d_e);
                let d_v = 0.5 * (d_n + d_s);

                if is_r_horizontal {
                    r = (g + d_h).max(0.0);
                    b = (g + d_v).max(0.0);
                } else {
                    b = (g + d_h).max(0.0);
                    r = (g + d_v).max(0.0);
                }
            }

            out_rgb[row_start + x] = [r, g, b];
        }
    }

    out_rgb
}

/// Archival Multi-Pass Edge-Adaptive AMaZE / RCD Demosaicing
///
/// Implements Residual Color Difference (RCD) demosaicing with iterative green refinement
/// and multi-directional edge gradient estimation.
pub fn rcd_archival_demosaic(
    norm_pixels: &[f32],
    full_w: usize,
    full_h: usize,
    active_x: usize,
    active_y: usize,
    active_w: usize,
    active_h: usize,
    frame: &RawSensorFrame,
) -> Vec<[f32; 3]> {
    let sample = |fy: isize, fx: isize| -> f32 {
        let y_clamped = fy.clamp(0, (full_h - 1) as isize) as usize;
        let x_clamped = fx.clamp(0, (full_w - 1) as isize) as usize;
        norm_pixels[y_clamped * full_w + x_clamped]
    };

    let apron_w = active_w + 4;
    let apron_h = active_h + 4;
    let mut green_plane = vec![0.0f32; apron_w * apron_h];

    // Pass 1: Multi-directional initial green reconstruction
    for ay in 0..apron_h {
        let fy = (active_y as isize + ay as isize - 2).clamp(0, (full_h - 1) as isize);
        let row_start = ay * apron_w;
        for ax in 0..apron_w {
            let fx = (active_x as isize + ax as isize - 2).clamp(0, (full_w - 1) as isize);
            let color = frame.color_at(fy as usize, fx as usize);
            if color == CFA_COLOR_GREEN {
                green_plane[row_start + ax] = sample(fy, fx);
            } else {
                let c = sample(fy, fx);
                let g_n = sample(fy - 1, fx);
                let g_s = sample(fy + 1, fx);
                let g_w = sample(fy, fx - 1);
                let g_e = sample(fy, fx + 1);

                let c_n2 = sample(fy - 2, fx);
                let c_s2 = sample(fy + 2, fx);
                let c_w2 = sample(fy, fx - 2);
                let c_e2 = sample(fy, fx + 2);

                let delta_h = (g_w - g_e).abs() + (2.0 * c - c_w2 - c_e2).abs();
                let delta_v = (g_n - g_s).abs() + (2.0 * c - c_n2 - c_s2).abs();

                let g_h = 0.5 * (g_w + g_e) + 0.25 * (2.0 * c - c_w2 - c_e2);
                let g_v = 0.5 * (g_n + g_s) + 0.25 * (2.0 * c - c_n2 - c_s2);

                let diff = delta_h - delta_v;
                let sum = delta_h + delta_v + 1e-5;
                let t = (diff / sum).clamp(-1.0, 1.0);
                let u = 0.5 + 0.5 * t;
                let w_v = u * u * (3.0 - 2.0 * u);
                let w_h = 1.0 - w_v;

                green_plane[row_start + ax] = (w_h * g_h + w_v * g_v).max(0.0);
            }
        }
    }

    // Pass 2: Iterative Residual Color Difference (RCD) green refinement
    let mut refined_green = green_plane.clone();
    for ay in 2..(apron_h.saturating_sub(2)) {
        let fy = (active_y as isize + ay as isize - 2).clamp(0, (full_h - 1) as isize);
        let row_start = ay * apron_w;
        for ax in 2..(apron_w.saturating_sub(2)) {
            let fx = (active_x as isize + ax as isize - 2).clamp(0, (full_w - 1) as isize);
            let color = frame.color_at(fy as usize, fx as usize);
            if color != CFA_COLOR_GREEN {
                let c = sample(fy, fx);
                let d_c = c - green_plane[row_start + ax];
                let d_nw = sample(fy - 2, fx - 2) - green_plane[(ay - 2) * apron_w + (ax - 2)];
                let d_ne = sample(fy - 2, fx + 2) - green_plane[(ay - 2) * apron_w + (ax + 2)];
                let d_sw = sample(fy + 2, fx - 2) - green_plane[(ay + 2) * apron_w + (ax - 2)];
                let d_se = sample(fy + 2, fx + 2) - green_plane[(ay + 2) * apron_w + (ax + 2)];

                let mut med_d = [d_nw, d_ne, d_c, d_sw, d_se];
                med_d.sort_unstable_by(|a, b| a.total_cmp(b));
                let residual_correction = med_d[2];

                refined_green[row_start + ax] = (c - residual_correction).max(0.0);
            }
        }
    }

    let g_sample = |ay: isize, ax: isize| -> f32 {
        let y_clamped = ay.clamp(0, (apron_h - 1) as isize) as usize;
        let x_clamped = ax.clamp(0, (apron_w - 1) as isize) as usize;
        refined_green[y_clamped * apron_w + x_clamped]
    };

    // Step 3: Interpolate Red and Blue using refined residual differences
    let mut out_rgb = vec![[0.0f32; 3]; active_w * active_h];

    for y in 0..active_h {
        let ay = (y + 2) as isize;
        let fy = (active_y + y) as isize;
        let row_start = y * active_w;

        for x in 0..active_w {
            let ax = (x + 2) as isize;
            let fx = (active_x + x) as isize;
            let color = frame.color_at(
                fy.clamp(0, (full_h - 1) as isize) as usize,
                fx.clamp(0, (full_w - 1) as isize) as usize,
            );
            let g = g_sample(ay, ax);

            let r: f32;
            let b: f32;

            if color == CFA_COLOR_RED {
                r = sample(fy, fx);
                let d_nw = sample(fy - 1, fx - 1) - g_sample(ay - 1, ax - 1);
                let d_ne = sample(fy - 1, fx + 1) - g_sample(ay - 1, ax + 1);
                let d_sw = sample(fy + 1, fx - 1) - g_sample(ay + 1, ax - 1);
                let d_se = sample(fy + 1, fx + 1) - g_sample(ay + 1, ax + 1);

                let grad_d1 = (sample(fy - 1, fx - 1) - sample(fy + 1, fx + 1)).abs()
                    + (g_sample(ay - 1, ax - 1) - g_sample(ay + 1, ax + 1)).abs();
                let grad_d2 = (sample(fy - 1, fx + 1) - sample(fy + 1, fx - 1)).abs()
                    + (g_sample(ay - 1, ax + 1) - g_sample(ay + 1, ax - 1)).abs();

                let diff_d = grad_d1 - grad_d2;
                let sum_d = grad_d1 + grad_d2 + 1e-5;
                let t_d = (diff_d / sum_d).clamp(-1.0, 1.0);
                let u_d = 0.5 + 0.5 * t_d;
                let w_d2 = u_d * u_d * (3.0 - 2.0 * u_d);
                let w_d1 = 1.0 - w_d2;

                let b_d1 = 0.5 * (d_nw + d_se);
                let b_d2 = 0.5 * (d_ne + d_sw);
                b = (g + w_d1 * b_d1 + w_d2 * b_d2).max(0.0);
            } else if color == CFA_COLOR_BLUE {
                b = sample(fy, fx);
                let d_nw = sample(fy - 1, fx - 1) - g_sample(ay - 1, ax - 1);
                let d_ne = sample(fy - 1, fx + 1) - g_sample(ay - 1, ax + 1);
                let d_sw = sample(fy + 1, fx - 1) - g_sample(ay + 1, ax - 1);
                let d_se = sample(fy + 1, fx + 1) - g_sample(ay + 1, ax + 1);

                let grad_d1 = (sample(fy - 1, fx - 1) - sample(fy + 1, fx + 1)).abs()
                    + (g_sample(ay - 1, ax - 1) - g_sample(ay + 1, ax + 1)).abs();
                let grad_d2 = (sample(fy - 1, fx + 1) - sample(fy + 1, fx - 1)).abs()
                    + (g_sample(ay - 1, ax + 1) - g_sample(ay + 1, ax - 1)).abs();

                let diff_d = grad_d1 - grad_d2;
                let sum_d = grad_d1 + grad_d2 + 1e-5;
                let t_d = (diff_d / sum_d).clamp(-1.0, 1.0);
                let u_d = 0.5 + 0.5 * t_d;
                let w_d2 = u_d * u_d * (3.0 - 2.0 * u_d);
                let w_d1 = 1.0 - w_d2;

                let r_d1 = 0.5 * (d_nw + d_se);
                let r_d2 = 0.5 * (d_ne + d_sw);
                r = (g + w_d1 * r_d1 + w_d2 * r_d2).max(0.0);
            } else {
                let fy_u = fy.clamp(0, (full_h - 1) as isize) as usize;
                let fx_prev = (fx - 1).clamp(0, (full_w - 1) as isize) as usize;
                let fx_next = (fx + 1).clamp(0, (full_w - 1) as isize) as usize;
                let is_r_horizontal = frame.color_at(fy_u, fx_prev) == CFA_COLOR_RED
                    || frame.color_at(fy_u, fx_next) == CFA_COLOR_RED;

                let d_w = sample(fy, fx - 1) - g_sample(ay, ax - 1);
                let d_e = sample(fy, fx + 1) - g_sample(ay, ax + 1);
                let d_n = sample(fy - 1, fx) - g_sample(ay - 1, ax);
                let d_s = sample(fy + 1, fx) - g_sample(ay + 1, ax);

                let d_h = 0.5 * (d_w + d_e);
                let d_v = 0.5 * (d_n + d_s);

                if is_r_horizontal {
                    r = (g + d_h).max(0.0);
                    b = (g + d_v).max(0.0);
                } else {
                    b = (g + d_h).max(0.0);
                    r = (g + d_v).max(0.0);
                }
            }

            out_rgb[row_start + x] = [r, g, b];
        }
    }

    out_rgb
}

/// Applies chromatic difference median filtering to eliminate Bayer demosaicing
/// checkerboard moiré and false-color maze artifacts along subtle gradients.
pub fn apply_chromatic_median_filter(
    pixels: &mut Vec<[f32; 3]>,
    width: usize,
    height: usize,
    passes: usize,
) {
    if passes == 0 || width < 3 || height < 3 || pixels.len() < width * height {
        return;
    }

    let mut src = std::mem::take(pixels);
    let mut dst = src.clone();

    for _ in 0..passes {
        for y in 0..height {
            let y_top = y.saturating_sub(1);
            let y_bot = (y + 1).min(height - 1);
            let row_top = &src[y_top * width..(y_top + 1) * width];
            let row_mid = &src[y * width..(y + 1) * width];
            let row_bot = &src[y_bot * width..(y_bot + 1) * width];
            let dst_row = &mut dst[y * width..(y + 1) * width];

            for x in 0..width {
                let x_left = x.saturating_sub(1);
                let x_right = (x + 1).min(width - 1);

                let g_mid = row_mid[x][1];

                // 3x3 window on R - G
                let mut cr = [
                    row_top[x_left][0] - row_top[x_left][1],
                    row_top[x][0] - row_top[x][1],
                    row_top[x_right][0] - row_top[x_right][1],
                    row_mid[x_left][0] - row_mid[x_left][1],
                    row_mid[x][0] - g_mid,
                    row_mid[x_right][0] - row_mid[x_right][1],
                    row_bot[x_left][0] - row_bot[x_left][1],
                    row_bot[x][0] - row_bot[x][1],
                    row_bot[x_right][0] - row_bot[x_right][1],
                ];

                // 3x3 window on B - G
                let mut cb = [
                    row_top[x_left][2] - row_top[x_left][1],
                    row_top[x][2] - row_top[x][1],
                    row_top[x_right][2] - row_top[x_right][1],
                    row_mid[x_left][2] - row_mid[x_left][1],
                    row_mid[x][2] - g_mid,
                    row_mid[x_right][2] - row_mid[x_right][1],
                    row_bot[x_left][2] - row_bot[x_left][1],
                    row_bot[x][2] - row_bot[x][1],
                    row_bot[x_right][2] - row_bot[x_right][1],
                ];

                cr.sort_unstable_by(|a, b| a.total_cmp(b));
                cb.sort_unstable_by(|a, b| a.total_cmp(b));

                dst_row[x] = [
                    (g_mid + cr[4]).max(0.0),
                    g_mid,
                    (g_mid + cb[4]).max(0.0),
                ];
            }
        }
        std::mem::swap(&mut src, &mut dst);
    }

    *pixels = src;
}
