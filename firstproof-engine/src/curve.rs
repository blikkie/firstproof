use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CurvePointRecord {
    pub x: f32,
    pub y: f32,
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToneCurvesRecord {
    pub master: Vec<CurvePointRecord>,
    pub red: Vec<CurvePointRecord>,
    pub green: Vec<CurvePointRecord>,
    pub blue: Vec<CurvePointRecord>,
}

impl Default for ToneCurvesRecord {
    fn default() -> Self {
        Self {
            master: vec![
                CurvePointRecord { x: 0.0, y: 0.0 },
                CurvePointRecord { x: 1.0, y: 1.0 },
            ],
            red: vec![
                CurvePointRecord { x: 0.0, y: 0.0 },
                CurvePointRecord { x: 1.0, y: 1.0 },
            ],
            green: vec![
                CurvePointRecord { x: 0.0, y: 0.0 },
                CurvePointRecord { x: 1.0, y: 1.0 },
            ],
            blue: vec![
                CurvePointRecord { x: 0.0, y: 0.0 },
                CurvePointRecord { x: 1.0, y: 1.0 },
            ],
        }
    }
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AllCurvesLutRecord {
    pub master: Vec<f32>,
    pub red: Vec<f32>,
    pub green: Vec<f32>,
    pub blue: Vec<f32>,
}

/// Checks if a tone curve deviates from the linear identity transfer line.
#[inline(always)]
pub fn is_curve_active(pts: &[CurvePointRecord]) -> bool {
    pts.len() > 2
        || (pts.len() == 2
            && ((pts[0].x - pts[0].y).abs() > 1e-4 || (pts[1].x - pts[1].y).abs() > 1e-4))
}

/// Evaluates a 256-sample lookup table [0.0, 1.0] using Fritsch-Carlson monotone cubic spline interpolation.
pub fn evaluate_curve_lut(points: &[CurvePointRecord]) -> Vec<f32> {
    evaluate_curve_lut_samples(points, 256)
}

pub fn evaluate_all_curves_lut(curves: &ToneCurvesRecord) -> AllCurvesLutRecord {
    evaluate_curves_lut_ref(curves)
}

/// Evaluates the 4 channels of ToneCurvesRecord from a reference.
pub fn evaluate_curves_lut_ref(curves: &ToneCurvesRecord) -> AllCurvesLutRecord {
    AllCurvesLutRecord {
        master: evaluate_curve_lut_samples(&curves.master, 256),
        red: evaluate_curve_lut_samples(&curves.red, 256),
        green: evaluate_curve_lut_samples(&curves.green, 256),
        blue: evaluate_curve_lut_samples(&curves.blue, 256),
    }
}

/// Evaluates a spline into `num_samples` normalized floating-point samples in [0.0, 1.0].
pub fn evaluate_curve_lut_samples(points: &[CurvePointRecord], num_samples: usize) -> Vec<f32> {
    let spline = MonotoneSpline::from_points(points);
    let mut lut = Vec::with_capacity(num_samples);
    let step = 1.0 / (num_samples - 1) as f32;
    for i in 0..num_samples {
        let x = (i as f32 * step).clamp(0.0, 1.0);
        lut.push(spline.evaluate(x));
    }
    lut
}

/// Monotone Hermite Cubic Spline representation (Fritsch-Carlson algorithm).
#[derive(Clone, Debug)]
pub struct MonotoneSpline {
    xs: Vec<f32>,
    ys: Vec<f32>,
    tangents: Vec<f32>,
}

impl MonotoneSpline {
    pub fn from_points(input_points: &[CurvePointRecord]) -> Self {
        if input_points.is_empty() {
            return Self::identity();
        }

        // 1. Sanitize, clamp, and sort points by ascending X
        let mut pts: Vec<(f32, f32)> = input_points
            .iter()
            .map(|p| (p.x.clamp(0.0, 1.0), p.y.clamp(0.0, 1.0)))
            .collect();
        pts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Deduplicate adjacent points with very close X
        let mut clean_pts: Vec<(f32, f32)> = Vec::with_capacity(pts.len() + 2);
        for p in pts {
            if let Some(last) = clean_pts.last() {
                if (p.0 - last.0).abs() < 1e-5 {
                    continue;
                }
            }
            clean_pts.push(p);
        }

        // Ensure boundary endpoints x=0.0 and x=1.0 exist
        if clean_pts.is_empty() {
            return Self::identity();
        }
        if clean_pts[0].0 > 0.0 {
            clean_pts.insert(0, (0.0, clean_pts[0].1));
        }
        if clean_pts.last().unwrap().0 < 1.0 {
            let last_y = clean_pts.last().unwrap().1;
            clean_pts.push((1.0, last_y));
        }

        let n = clean_pts.len();
        if n < 2 {
            return Self::identity();
        }

        let xs: Vec<f32> = clean_pts.iter().map(|p| p.0).collect();
        let ys: Vec<f32> = clean_pts.iter().map(|p| p.1).collect();

        // 2. Compute secant slopes
        let mut deltas = Vec::with_capacity(n - 1);
        for k in 0..(n - 1) {
            let dx = xs[k + 1] - xs[k];
            let dy = ys[k + 1] - ys[k];
            deltas.push(if dx.abs() > 1e-6 { dy / dx } else { 0.0 });
        }

        // 3. Initialize tangents: average adjacent secants, 0 at flat secants or interior extrema
        let mut tangents = vec![0.0; n];
        tangents[0] = deltas[0];
        tangents[n - 1] = deltas[n - 2];
        for k in 1..(n - 1) {
            if deltas[k - 1] * deltas[k] <= 0.0 {
                tangents[k] = 0.0;
            } else {
                tangents[k] = (deltas[k - 1] + deltas[k]) * 0.5;
            }
        }

        // 4. Fritsch-Carlson monotonicity check & adjustment
        for k in 0..(n - 1) {
            let delta = deltas[k];
            if delta.abs() < 1e-7 {
                tangents[k] = 0.0;
                tangents[k + 1] = 0.0;
            } else {
                let alpha = tangents[k] / delta;
                let beta = tangents[k + 1] / delta;
                if alpha < 0.0 {
                    tangents[k] = 0.0;
                }
                if beta < 0.0 {
                    tangents[k + 1] = 0.0;
                }
                let norm_sq = alpha * alpha + beta * beta;
                if norm_sq > 9.0 {
                    let tau = 3.0 / norm_sq.sqrt();
                    tangents[k] = tau * alpha * delta;
                    tangents[k + 1] = tau * beta * delta;
                }
            }
        }

        Self { xs, ys, tangents }
    }

    pub fn identity() -> Self {
        Self {
            xs: vec![0.0, 1.0],
            ys: vec![0.0, 1.0],
            tangents: vec![1.0, 1.0],
        }
    }

    /// Evaluates the cubic spline at input value `x`.
    pub fn evaluate(&self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        let n = self.xs.len();
        if n == 0 {
            return x;
        }
        if x <= self.xs[0] {
            return self.ys[0];
        }
        if x >= self.xs[n - 1] {
            return self.ys[n - 1];
        }

        // Binary search to find interval [x_k, x_{k+1}]
        let mut low = 0;
        let mut high = n - 1;
        while high - low > 1 {
            let mid = (low + high) / 2;
            if self.xs[mid] <= x {
                low = mid;
            } else {
                high = mid;
            }
        }
        let k = low;

        let h = self.xs[k + 1] - self.xs[k];
        if h.abs() < 1e-6 {
            return self.ys[k];
        }

        let t = (x - self.xs[k]) / h;
        let t2 = t * t;
        let t3 = t2 * t;

        // Hermite basis functions
        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + t;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;

        let y = h00 * self.ys[k]
            + h10 * h * self.tangents[k]
            + h01 * self.ys[k + 1]
            + h11 * h * self.tangents[k + 1];

        y.clamp(0.0, 1.0)
    }
}
