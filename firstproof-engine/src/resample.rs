//! High-performance bilinear resampling for 16-bit RGB image buffers.
//!
//! Supports arbitrary affine crop boundaries, 90-degree orthogonal rotations,
//! horizontal/vertical reflections, and proportional dimensions.

#[inline(always)]
pub fn resample_rgb16(
    src_w: usize,
    src_h: usize,
    src_data: &[u16],
    rotation: i32,
    crop_left: f32,
    crop_top: f32,
    crop_right: f32,
    crop_bottom: f32,
    flip_h: bool,
    flip_v: bool,
    target_w: usize,
    target_h: usize,
) -> Vec<u16> {
    let mut out = vec![0u16; target_w * target_h * 3];
    if src_w == 0 || src_h == 0 || target_w == 0 || target_h == 0 || src_data.is_empty() {
        return out;
    }

    let norm_rot = rotation.rem_euclid(360);
    let crop_l = crop_left.clamp(0.0, 0.99);
    let crop_t = crop_top.clamp(0.0, 0.99);
    let crop_r = crop_right.clamp(crop_l + 0.001, 1.0);
    let crop_b = crop_bottom.clamp(crop_t + 0.001, 1.0);
    let crop_w = crop_r - crop_l;
    let crop_h = crop_b - crop_t;

    let src_w_f = src_w as f32;
    let src_h_f = src_h as f32;
    let max_x_f = (src_w - 1) as f32;
    let max_y_f = (src_h - 1) as f32;

    // Fast-path: Standard 0° orientation without flips
    if norm_rot == 0 && !flip_h && !flip_v {
        for dy in 0..target_h {
            let v = (dy as f32 + 0.5) / (target_h as f32);
            let v_crop = crop_t + v * crop_h;
            let sy = (v_crop * src_h_f - 0.5).clamp(0.0, max_y_f);

            let y0 = sy.floor() as usize;
            let y1 = (y0 + 1).min(src_h - 1);
            let fy = sy - (y0 as f32);
            let w_y0 = 1.0 - fy;
            let w_y1 = fy;

            let row0 = y0 * src_w * 3;
            let row1 = y1 * src_w * 3;
            let mut dst_idx = dy * target_w * 3;

            for dx in 0..target_w {
                let u = (dx as f32 + 0.5) / (target_w as f32);
                let u_crop = crop_l + u * crop_w;
                let sx = (u_crop * src_w_f - 0.5).clamp(0.0, max_x_f);

                let x0 = sx.floor() as usize;
                let x1 = (x0 + 1).min(src_w - 1);
                let fx = sx - (x0 as f32);

                let w00 = (1.0 - fx) * w_y0;
                let w10 = fx * w_y0;
                let w01 = (1.0 - fx) * w_y1;
                let w11 = fx * w_y1;

                let i00 = row0 + x0 * 3;
                let i10 = row0 + x1 * 3;
                let i01 = row1 + x0 * 3;
                let i11 = row1 + x1 * 3;

                let r = w00 * (src_data[i00] as f32)
                    + w10 * (src_data[i10] as f32)
                    + w01 * (src_data[i01] as f32)
                    + w11 * (src_data[i11] as f32);
                let g = w00 * (src_data[i00 + 1] as f32)
                    + w10 * (src_data[i10 + 1] as f32)
                    + w01 * (src_data[i01 + 1] as f32)
                    + w11 * (src_data[i11 + 1] as f32);
                let b = w00 * (src_data[i00 + 2] as f32)
                    + w10 * (src_data[i10 + 2] as f32)
                    + w01 * (src_data[i01 + 2] as f32)
                    + w11 * (src_data[i11 + 2] as f32);

                out[dst_idx] = r.round().clamp(0.0, 65535.0) as u16;
                out[dst_idx + 1] = g.round().clamp(0.0, 65535.0) as u16;
                out[dst_idx + 2] = b.round().clamp(0.0, 65535.0) as u16;
                dst_idx += 3;
            }
        }
        return out;
    }

    // General path: Rotations (90°, 180°, 270°) and arbitrary horizontal / vertical flips
    for dy in 0..target_h {
        let mut v = (dy as f32 + 0.5) / (target_h as f32);
        if flip_v {
            v = 1.0 - v;
        }

        let mut dst_idx = dy * target_w * 3;

        for dx in 0..target_w {
            let mut u = (dx as f32 + 0.5) / (target_w as f32);
            if flip_h {
                u = 1.0 - u;
            }

            let u_crop = crop_l + u * crop_w;
            let v_crop = crop_t + v * crop_h;

            let (sx, sy) = match norm_rot {
                90 => (v_crop * src_w_f - 0.5, (1.0 - u_crop) * src_h_f - 0.5),
                180 => ((1.0 - u_crop) * src_w_f - 0.5, (1.0 - v_crop) * src_h_f - 0.5),
                270 => ((1.0 - v_crop) * src_w_f - 0.5, u_crop * src_h_f - 0.5),
                _ => (u_crop * src_w_f - 0.5, v_crop * src_h_f - 0.5),
            };

            let sx_c = sx.clamp(0.0, max_x_f);
            let sy_c = sy.clamp(0.0, max_y_f);

            let x0 = sx_c.floor() as usize;
            let y0 = sy_c.floor() as usize;
            let x1 = (x0 + 1).min(src_w - 1);
            let y1 = (y0 + 1).min(src_h - 1);

            let fx = sx_c - (x0 as f32);
            let fy = sy_c - (y0 as f32);

            let w00 = (1.0 - fx) * (1.0 - fy);
            let w10 = fx * (1.0 - fy);
            let w01 = (1.0 - fx) * fy;
            let w11 = fx * fy;

            let idx00 = (y0 * src_w + x0) * 3;
            let idx10 = (y0 * src_w + x1) * 3;
            let idx01 = (y1 * src_w + x0) * 3;
            let idx11 = (y1 * src_w + x1) * 3;

            let r = w00 * (src_data[idx00] as f32)
                + w10 * (src_data[idx10] as f32)
                + w01 * (src_data[idx01] as f32)
                + w11 * (src_data[idx11] as f32);
            let g = w00 * (src_data[idx00 + 1] as f32)
                + w10 * (src_data[idx10 + 1] as f32)
                + w01 * (src_data[idx01 + 1] as f32)
                + w11 * (src_data[idx11 + 1] as f32);
            let b = w00 * (src_data[idx00 + 2] as f32)
                + w10 * (src_data[idx10 + 2] as f32)
                + w01 * (src_data[idx01 + 2] as f32)
                + w11 * (src_data[idx11 + 2] as f32);

            out[dst_idx] = r.round().clamp(0.0, 65535.0) as u16;
            out[dst_idx + 1] = g.round().clamp(0.0, 65535.0) as u16;
            out[dst_idx + 2] = b.round().clamp(0.0, 65535.0) as u16;
            dst_idx += 3;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resample_rgb16_identity() {
        let src = vec![
            1000, 2000, 3000, 4000, 5000, 6000,
            7000, 8000, 9000, 10000, 11000, 12000,
        ];
        let out = resample_rgb16(2, 2, &src, 0, 0.0, 0.0, 1.0, 1.0, false, false, 2, 2);
        assert_eq!(out.len(), 12);
        for i in 0..12 {
            assert!((out[i] as i32 - src[i] as i32).abs() <= 1);
        }
    }

    #[test]
    fn test_resample_rgb16_downscale() {
        let src = vec![
            1000, 2000, 3000, 1000, 2000, 3000,
            1000, 2000, 3000, 1000, 2000, 3000,
        ];
        let out = resample_rgb16(2, 2, &src, 0, 0.0, 0.0, 1.0, 1.0, false, false, 1, 1);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0], 1000);
        assert_eq!(out[1], 2000);
        assert_eq!(out[2], 3000);
    }
}
