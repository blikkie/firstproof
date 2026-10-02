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

/// Evaluates a 256-sample lookup table [0.0, 1.0] using Fritsch-Carlson monotone cubic spline interpolation.
pub fn evaluate_curve_lut(points: &[CurvePointRecord]) -> Vec<f32> {
    if points.is_empty() {
        return (0..256).map(|i| i as f32 / 255.0).collect();
    }

    let mut pts = points.to_vec();
    pts.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    pts.dedup_by(|a, b| (a.x - b.x).abs() < 1e-6);

    if pts.len() == 1 {
        let val = pts[0].y.clamp(0.0, 1.0);
        return vec![val; 256];
    }

    let n = pts.len();
    let mut h = vec![0.0; n - 1];
    let mut delta = vec![0.0; n - 1];
    for i in 0..n - 1 {
        h[i] = pts[i + 1].x - pts[i].x;
        delta[i] = if h[i].abs() > 1e-6 {
            (pts[i + 1].y - pts[i].y) / h[i]
        } else {
            0.0
        };
    }

    let mut d = vec![0.0; n];
    d[0] = delta[0];
    d[n - 1] = delta[n - 2];
    for i in 1..n - 1 {
        d[i] = (delta[i - 1] + delta[i]) * 0.5;
    }

    for i in 0..n - 1 {
        if delta[i].abs() < 1e-9 {
            d[i] = 0.0;
            d[i + 1] = 0.0;
        } else {
            let alpha = d[i] / delta[i];
            let beta = d[i + 1] / delta[i];
            let dist = alpha * alpha + beta * beta;
            if dist > 9.0 {
                let tau = 3.0 / dist.sqrt();
                d[i] = tau * alpha * delta[i];
                d[i + 1] = tau * beta * delta[i];
            }
        }
    }

    let mut lut = Vec::with_capacity(256);
    for i in 0..256 {
        let x = i as f32 / 255.0;
        if x <= pts[0].x {
            lut.push(pts[0].y.clamp(0.0, 1.0));
            continue;
        }
        if x >= pts[n - 1].x {
            lut.push(pts[n - 1].y.clamp(0.0, 1.0));
            continue;
        }

        let mut seg = 0;
        for k in 0..n - 1 {
            if x >= pts[k].x && x <= pts[k + 1].x {
                seg = k;
                break;
            }
        }

        let seg_h = h[seg];
        let t = if seg_h > 1e-6 {
            (x - pts[seg].x) / seg_h
        } else {
            0.0
        };
        let t2 = t * t;
        let t3 = t2 * t;

        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + t;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;

        let y = h00 * pts[seg].y
            + h10 * seg_h * d[seg]
            + h01 * pts[seg + 1].y
            + h11 * seg_h * d[seg + 1];
        lut.push(y.clamp(0.0, 1.0));
    }
    lut
}

pub fn evaluate_all_curves_lut(curves: &ToneCurvesRecord) -> AllCurvesLutRecord {
    AllCurvesLutRecord {
        master: evaluate_curve_lut(&curves.master),
        red: evaluate_curve_lut(&curves.red),
        green: evaluate_curve_lut(&curves.green),
        blue: evaluate_curve_lut(&curves.blue),
    }
}
