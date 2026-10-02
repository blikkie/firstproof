use firstproof_core::{
    evaluate_all_curves_lut, evaluate_curve_lut, CurvePointRecord, EditSettings,
    EmulsionArchetype, ToneCurvesRecord,
};
use firstproof_engine::xmp::{generate_xmp_sidecar, parse_xmp_sidecar, XmpData};

#[test]
fn test_core_wasm_curve_eval_parity() {
    let pts = vec![
        CurvePointRecord { x: 0.0, y: 0.0 },
        CurvePointRecord { x: 0.25, y: 0.20 },
        CurvePointRecord { x: 0.75, y: 0.85 },
        CurvePointRecord { x: 1.0, y: 1.0 },
    ];
    let lut_core = evaluate_curve_lut(&pts);
    let lut_types = firstproof_engine::evaluate_curve_lut(&pts);

    assert_eq!(lut_core.len(), 256);
    assert_eq!(lut_core, lut_types);
}

#[test]
fn test_core_wasm_all_curves_lut_parity() {
    let curves = ToneCurvesRecord {
        master: vec![
            CurvePointRecord { x: 0.0, y: 0.0 },
            CurvePointRecord { x: 0.5, y: 0.6 },
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
    };

    let lut_record = evaluate_all_curves_lut(&curves);
    assert_eq!(lut_record.master.len(), 256);
    assert_eq!(lut_record.red.len(), 256);
    assert_eq!(lut_record.green.len(), 256);
    assert_eq!(lut_record.blue.len(), 256);
    assert!((lut_record.master[128] - 0.6).abs() < 0.05);
}

#[test]
fn test_core_wasm_xmp_serialization_parity() {
    let mut xmp = XmpData::default();
    xmp.rating = Some(5);
    xmp.status = "Pick".to_string();
    xmp.title = Some("Parity Test Photo".to_string());
    xmp.edits = EditSettings {
        exposure: 0.85,
        contrast: 15.0,
        temperature: 5800.0,
        tint: -4.0,
        highlights: -25.0,
        shadows: 30.0,
        emulsion_archetype: EmulsionArchetype::RochesterPortrait400,
        ..Default::default()
    };

    let xml = generate_xmp_sidecar(&xmp);
    let parsed = parse_xmp_sidecar(&xml).expect("Failed to parse XMP XML");

    assert_eq!(parsed.rating, Some(5));
    assert_eq!(parsed.status, "Pick");
    assert_eq!(parsed.title, Some("Parity Test Photo".to_string()));
    assert!((parsed.edits.exposure - 0.85).abs() < 1e-3);
    assert!((parsed.edits.temperature - 5800.0).abs() < 1.0);
    assert_eq!(
        parsed.edits.emulsion_archetype,
        EmulsionArchetype::RochesterPortrait400
    );
}
