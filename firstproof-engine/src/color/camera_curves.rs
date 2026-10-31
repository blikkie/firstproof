//! Camera tone curves and HSV/RGB color utilities.
//!
//! Monotonic cubic Hermite spline interpolation (Fritsch-Carlson algorithm)
//! and standard built-in camera matching tone curves.

/// Converts RGB to HSV (H in degrees [0, 360), S in [0, 1], V in [0, 1]).
#[inline]
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;
    let s = if max > 1e-6 { delta / max } else { 0.0 };

    let h = if delta < 1e-6 {
        0.0
    } else if (max - r).abs() < 1e-6 {
        60.0 * (((g - b) / delta).rem_euclid(6.0))
    } else if (max - g).abs() < 1e-6 {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    (h, s, v)
}

/// Converts HSV (H in degrees [0, 360), S in [0, 1], V in [0, 1]) back to RGB.
#[inline]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h_prime = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - ((h_prime.rem_euclid(2.0)) - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r1 + m, g1 + m, b1 + m)
}

/// Evaluates a tone curve defined by (x, y) control knots using a C^1 continuous
/// monotonic cubic Hermite spline (Fritsch-Carlson algorithm).
#[inline]
pub fn evaluate_tone_curve(x: f32, curve: &[(f32, f32)]) -> f32 {
    let n = curve.len();
    if n == 0 {
        return x;
    }
    if n == 1 {
        return curve[0].1;
    }
    let x_clamped = x.clamp(0.0, 1.0);
    if x_clamped <= curve[0].0 {
        return curve[0].1;
    }
    if x_clamped >= curve[n - 1].0 {
        return curve[n - 1].1;
    }

    let n_segs = n - 1;
    let mut d_stack = [0.0f32; 32];
    let mut m_stack = [0.0f32; 33];
    let mut d_heap;
    let mut m_heap;
    let (d, m): (&mut [f32], &mut [f32]) = if n_segs <= 32 {
        (&mut d_stack[..n_segs], &mut m_stack[..n])
    } else {
        d_heap = vec![0.0f32; n_segs];
        m_heap = vec![0.0f32; n];
        (&mut d_heap[..], &mut m_heap[..])
    };

    // 1. Calculate secant slopes d_k = (y_{k+1} - y_k) / (x_{k+1} - x_k)
    for k in 0..n_segs {
        let dx = curve[k + 1].0 - curve[k].0;
        let dy = curve[k + 1].1 - curve[k].1;
        d[k] = if dx.abs() > 1e-6 { dy / dx } else { 0.0 };
    }

    // 2. Initialize tangents as averages of adjacent secants
    m[0] = d[0];
    for k in 1..n_segs {
        m[k] = 0.5 * (d[k - 1] + d[k]);
    }
    m[n_segs] = d[n_segs - 1];

    // 3. Fritsch & Carlson monotonicity constraints
    for k in 0..n_segs {
        if d[k].abs() < 1e-6 {
            m[k] = 0.0;
            m[k + 1] = 0.0;
        } else {
            let alpha = m[k] / d[k];
            let beta = m[k + 1] / d[k];
            let s = alpha * alpha + beta * beta;
            if s > 9.0 {
                let tau = 3.0 / s.sqrt();
                m[k] = tau * alpha * d[k];
                m[k + 1] = tau * beta * d[k];
            }
        }
    }

    // 4. Find the active segment via binary search
    let seg = match curve.binary_search_by(|p| p.0.partial_cmp(&x_clamped).unwrap_or(std::cmp::Ordering::Equal)) {
        Ok(idx) => idx.min(n_segs - 1),
        Err(idx) => idx.saturating_sub(1).min(n_segs - 1),
    };

    // 5. Evaluate Hermite cubic polynomial
    let h = curve[seg + 1].0 - curve[seg].0;
    if h.abs() < 1e-6 {
        return curve[seg].1;
    }
    let t = (x_clamped - curve[seg].0) / h;
    let t2 = t * t;
    let t3 = t2 * t;

    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;

    let y = h00 * curve[seg].1
        + h10 * h * m[seg]
        + h01 * curve[seg + 1].1
        + h11 * h * m[seg + 1];

    y.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dense_257_point_curve_monotonicity_and_anchoring() {
        // Construct a synthetic 257-point sigmoid S-curve
        let points: Vec<(f32, f32)> = (0..=256)
            .map(|i| {
                let x = i as f32 / 256.0;
                // Sigmoidal Hermite tone curve
                let y = x * x * (3.0 - 2.0 * x);
                (x, y)
            })
            .collect();

        assert_eq!(evaluate_tone_curve(0.0, &points), 0.0);
        assert_eq!(evaluate_tone_curve(1.0, &points), 1.0);

        let mut prev_y = -1.0f32;
        for i in 0..=1000 {
            let x = i as f32 / 1000.0;
            let y = evaluate_tone_curve(x, &points);
            assert!(y >= prev_y, "Non-monotonic step at x={}: prev={}, curr={}", x, prev_y, y);
            assert!((0.0..=1.0).contains(&y));
            prev_y = y;
        }

        // Verify knot pass-through
        for &(kx, ky) in points.iter().step_by(16) {
            let evaluated = evaluate_tone_curve(kx, &points);
            assert!(
                (evaluated - ky).abs() < 1e-4,
                "Failed knot pass-through at x={}: expected {}, got {}",
                kx,
                ky,
                evaluated
            );
        }
    }
}
