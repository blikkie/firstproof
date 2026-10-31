pub use firstproof_engine::curve::{
    evaluate_curve_lut_samples, evaluate_curves_lut_ref, is_curve_active, AllCurvesLutRecord,
    CurvePointRecord, MonotoneSpline, ToneCurvesRecord,
};

/// Evaluates an arbitrary set of curve control points into a 256-sample lookup table (LUT)
/// using the Fritsch-Carlson monotone cubic hermite spline algorithm.
#[uniffi::export]
pub fn evaluate_curve_lut(points: Vec<CurvePointRecord>) -> Vec<f32> {
    evaluate_curve_lut_samples(&points, 256)
}

/// Evaluates the 4 channels of ToneCurvesRecord into four 256-sample LUTs (Master, Red, Green, Blue).
#[uniffi::export]
pub fn evaluate_all_curves_lut(curves: ToneCurvesRecord) -> AllCurvesLutRecord {
    evaluate_curves_lut_ref(&curves)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_spline() {
        let spline = MonotoneSpline::identity();
        for i in 0..=100 {
            let x = i as f32 / 100.0;
            let y = spline.evaluate(x);
            assert!(
                (y - x).abs() < 1e-5,
                "Identity curve mismatch at x={}: got {}",
                x,
                y
            );
        }
    }

    #[test]
    fn test_s_curve_monotonicity() {
        // Classic photographic S-curve: pull shadows down, push highlights up
        let points = vec![
            CurvePointRecord { x: 0.0, y: 0.0 },
            CurvePointRecord { x: 0.25, y: 0.15 }, // Pull shadows down
            CurvePointRecord { x: 0.5, y: 0.5 },   // Neutral midtone
            CurvePointRecord { x: 0.75, y: 0.85 }, // Push highlights up
            CurvePointRecord { x: 1.0, y: 1.0 },
        ];
        let spline = MonotoneSpline::from_points(&points);

        let mut prev_y = -1.0;
        for i in 0..=255 {
            let x = i as f32 / 255.0;
            let y = spline.evaluate(x);
            assert!(
                y >= prev_y - 1e-6,
                "Monotonicity violation at x={}: y={} < prev={}",
                x,
                y,
                prev_y
            );
            prev_y = y;
        }

        // Check S-curve shape properties
        assert!(spline.evaluate(0.25) < 0.25);
        assert!((spline.evaluate(0.5) - 0.5).abs() < 0.01);
        assert!(spline.evaluate(0.75) > 0.75);
    }

    #[test]
    fn test_matte_fade_spline() {
        // Vintage film fade: lift shadow black point to 0.15
        let points = vec![
            CurvePointRecord { x: 0.0, y: 0.15 },
            CurvePointRecord { x: 1.0, y: 1.0 },
        ];
        let spline = MonotoneSpline::from_points(&points);
        assert!((spline.evaluate(0.0) - 0.15).abs() < 1e-4);
        assert!((spline.evaluate(1.0) - 1.0).abs() < 1e-4);
        assert!((spline.evaluate(0.5) - 0.575).abs() < 1e-4);
    }

    #[test]
    fn test_lut_evaluation_size() {
        let lut = evaluate_curve_lut(vec![
            CurvePointRecord { x: 0.0, y: 0.0 },
            CurvePointRecord { x: 1.0, y: 1.0 },
        ]);
        assert_eq!(lut.len(), 256);
        assert!((lut[0] - 0.0).abs() < 1e-5);
        assert!((lut[255] - 1.0).abs() < 1e-5);
    }
}
