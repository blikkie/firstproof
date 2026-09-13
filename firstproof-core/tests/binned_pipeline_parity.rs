use firstproof_core::raw_decoder::{CfaPattern, RawSensorFrame};
use firstproof_core::export::decode::{f16_to_f32, f32_to_f16};
use firstproof_core::export::arena::with_decode_arena;

#[test]
fn test_binned_quad_patterns_math() {
    let patterns = [
        CfaPattern::Rggb,
        CfaPattern::Bggr,
        CfaPattern::Gbrg,
        CfaPattern::Grbg,
    ];

    for &pattern in &patterns {
        // Construct a synthetic 2x2 sensor frame
        let full_width = 2;
        let full_height = 2;
        // Photosite values:
        // [1000, 2000]
        // [3000, 4000]
        let sensor_data = vec![1000u16, 2000u16, 3000u16, 4000u16];

        let frame = RawSensorFrame {
            full_width,
            full_height,
            active_x: 0,
            active_y: 0,
            active_width: 2,
            active_height: 2,
            cfa_pattern: pattern,
            black_levels: [0.0; 4],
            white_levels: [10000.0; 4],
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            color_matrix: Some([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]),
            color_matrix_tungsten: None,
            camera_model: "Synthetic Test".to_string(),
            sensor_data,
        };

        // Expected (R, G, B) raw for each pattern:
        // RGGB: R=1000, G=(2000+3000)/2=2500, B=4000
        // BGGR: B=1000, G=(2000+3000)/2=2500, R=4000
        // GBRG: G1=1000, B=2000, R=3000, G2=4000 -> R=3000, G=2500, B=2000
        // GRBG: G1=1000, R=2000, B=3000, G2=4000 -> R=2000, G=2500, B=3000
        let (expected_r, expected_g, expected_b) = match pattern {
            CfaPattern::Rggb => (1000.0 / 10000.0, 2500.0 / 10000.0, 4000.0 / 10000.0),
            CfaPattern::Bggr => (4000.0 / 10000.0, 2500.0 / 10000.0, 1000.0 / 10000.0),
            CfaPattern::Gbrg => (3000.0 / 10000.0, 2500.0 / 10000.0, 2000.0 / 10000.0),
            CfaPattern::Grbg => (2000.0 / 10000.0, 2500.0 / 10000.0, 3000.0 / 10000.0),
            _ => unreachable!(),
        };

        let binned_w = frame.active_width / 2;
        let binned_h = frame.active_height / 2;
        assert_eq!(binned_w, 1);
        assert_eq!(binned_h, 1);

        let black = frame.black_levels[0];
        let white = frame.white_levels[0];
        let scale = 1.0 / (white - black);

        let s00 = frame.sensor_data[0] as f32;
        let s01 = frame.sensor_data[1] as f32;
        let s10 = frame.sensor_data[2] as f32;
        let s11 = frame.sensor_data[3] as f32;

        let (r_raw, g_raw, b_raw) = match frame.cfa_pattern {
            CfaPattern::Rggb => (s00, (s01 + s10) * 0.5, s11),
            CfaPattern::Bggr => (s11, (s01 + s10) * 0.5, s00),
            CfaPattern::Gbrg => (s10, (s00 + s11) * 0.5, s01),
            CfaPattern::Grbg => (s01, (s00 + s11) * 0.5, s10),
            _ => (s00, (s01 + s10) * 0.5, s11),
        };

        let r_norm = ((r_raw - black) * scale).clamp(0.0, 1.0);
        let g_norm = ((g_raw - black) * scale).clamp(0.0, 1.0);
        let b_norm = ((b_raw - black) * scale).clamp(0.0, 1.0);

        assert!((r_norm - expected_r).abs() < 1e-5, "R mismatch for {:?}", pattern);
        assert!((g_norm - expected_g).abs() < 1e-5, "G mismatch for {:?}", pattern);
        assert!((b_norm - expected_b).abs() < 1e-5, "B mismatch for {:?}", pattern);
    }
}

#[test]
fn test_half_float_alpha_invariants() {
    let a16 = f32_to_f16(1.0);
    assert_eq!(a16, 0x3c00);
    let a_back = f16_to_f32(a16);
    assert!((a_back - 1.0).abs() < 1e-6);
}

#[test]
fn test_scratch_arena_reusability() {
    with_decode_arena(|arena| {
        // Initial decode size: 10MB
        let initial_size = 10 * 1024 * 1024;
        let _ = arena.acquire_raw_bytes(initial_size);
        let initial_cap = arena.rgb_raw_bytes.capacity();
        assert!(initial_cap >= initial_size);

        // Next photo is smaller (e.g. 6MB) -> should not reallocate
        let _ = arena.acquire_raw_bytes(6 * 1024 * 1024);
        assert_eq!(arena.rgb_raw_bytes.capacity(), initial_cap, "Arena capacity grew unexpectedly when requesting smaller buffer");

        // Next photo is same size -> should not reallocate
        let _ = arena.acquire_raw_bytes(initial_size);
        assert_eq!(arena.rgb_raw_bytes.capacity(), initial_cap, "Arena capacity must be reused without heap churn");
    });
}
