use firstproof_engine::shader::{canonical_wgsl, compile_to_glsl, compile_to_msl};

#[test]
fn test_canonical_wgsl_cross_compilation() {
    let wgsl = canonical_wgsl();
    assert!(!wgsl.is_empty(), "Canonical WGSL must not be empty");
    assert!(wgsl.contains("struct ShaderUniforms"), "Must contain ShaderUniforms");
    assert!(wgsl.contains("fn vertexShader"), "Must contain vertexShader");
    assert!(wgsl.contains("fn fragmentShader"), "Must contain fragmentShader");

    // 1. Cross-compile to Metal Shading Language
    let msl = compile_to_msl().expect("Failed to compile canonical WGSL to MSL");
    assert!(msl.contains("vertexShader"), "MSL must contain vertexShader");
    assert!(msl.contains("fragmentShader"), "MSL must contain fragmentShader");
    assert!(msl.contains("buffer(1)"), "MSL must bind uniforms to buffer(1)");
    assert!(msl.contains("texture(0)"), "MSL must bind u_texture to texture(0)");
    assert!(msl.contains("sampler(0)"), "MSL must bind u_sampler to sampler(0)");
    assert!(msl.contains("texture(1)"), "MSL must bind u_lut_3d to texture(1)");
    assert!(msl.contains("sampler(1)"), "MSL must bind u_lut_sampler to sampler(1)");
    assert!(msl.contains("texture(2)"), "MSL must bind u_curve_master_lut to texture(2)");
    assert!(msl.contains("texture(3)"), "MSL must bind u_curve_rgb_lut to texture(3)");
    assert!(msl.contains("sampler(2)"), "MSL must bind u_curve_sampler to sampler(2)");
    assert!(msl.contains("texture(4)"), "MSL must bind u_gain_map to texture(4)");
    assert!(msl.contains("sampler(3)"), "MSL must bind u_gain_map_sampler to sampler(3)");

    // 2. Cross-compile to WebGL 2 GLSL ES 300
    let glsl = compile_to_glsl().expect("Failed to compile canonical WGSL to GLSL");
    assert!(glsl.contains("#version 300 es"), "GLSL must be version 300 es");
    assert!(glsl.contains("precision highp float;"), "GLSL must specify highp precision");
    assert!(glsl.contains("layout(std140) uniform"), "GLSL must contain std140 uniform block for ShaderUniforms");
    assert!(glsl.contains("void main()"), "GLSL must contain void main()");

    // 3. Pipeline Invariants
    assert!(msl.contains("0.2627"), "MSL must evaluate Rec.2020 Red coeff 0.2627");
    assert!(msl.contains("0.678"), "MSL must evaluate Rec.2020 Green coeff 0.678");
    assert!(msl.contains("0.0593"), "MSL must evaluate Rec.2020 Blue coeff 0.0593");

    assert!(glsl.contains("0.2627"), "GLSL must evaluate Rec.2020 Red coeff 0.2627");
    assert!(glsl.contains("0.678"), "GLSL must evaluate Rec.2020 Green coeff 0.678");
    assert!(glsl.contains("0.0593"), "GLSL must evaluate Rec.2020 Blue coeff 0.0593");

    // 4. Stage Ordering Invariant in Generated MSL
    let frag_start = msl.find("fragmentShader(").expect("fragmentShader must exist in MSL");
    let frag_body = &msl[frag_start..];
    for upstream in &[
        "applyBilateralToneRecovery",
        "applyRecordingEmulsion",
        "applyColorAdjustments",
        "applySplitToning",
        "applyDetailAndSharpening",
    ] {
        let u_pos = frag_body.find(upstream).unwrap_or_else(|| panic!("Missing {} in MSL", upstream));
        let agx_pos = frag_body.find("agxToneMapVivid").expect("Missing agxToneMapVivid in MSL");
        assert!(u_pos < agx_pos, "MSL stage {} must precede agxToneMapVivid", upstream);
    }
    let agx_pos = frag_body.find("agxToneMapVivid").unwrap();
    let gamma_pos = frag_body.find("linearToSrgbGamma3").expect("Missing linearToSrgbGamma3 in MSL");
    assert!(agx_pos < gamma_pos, "MSL agxToneMapVivid must precede linearToSrgbGamma3");

    // 5. Stage Ordering Invariant in Generated GLSL
    let glsl_main = glsl.find("void main()").expect("void main() must exist in GLSL");
    let glsl_body = &glsl[glsl_main..];
    for upstream in &[
        "applyBilateralToneRecovery",
        "applyRecordingEmulsion",
        "applyColorAdjustments",
        "applySplitToning",
        "applyDetailAndSharpening",
    ] {
        let u_pos = glsl_body.find(upstream).unwrap_or_else(|| panic!("Missing {} in GLSL", upstream));
        let agx_pos = glsl_body.find("agxToneMapVivid").expect("Missing agxToneMapVivid in GLSL");
        assert!(u_pos < agx_pos, "GLSL stage {} must precede agxToneMapVivid", upstream);
    }
    let glsl_agx_pos = glsl_body.find("agxToneMapVivid").unwrap();
    let glsl_gamma_pos = glsl_body.find("linearToSrgbGamma3").expect("Missing linearToSrgbGamma3 in GLSL");
    assert!(glsl_agx_pos < glsl_gamma_pos, "GLSL agxToneMapVivid must precede linearToSrgbGamma3");
}
