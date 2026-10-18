// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

//! Architecture Fitness Functions (Story 4.6.2)
//!
//! Enforces the computational steering loop and Martin Fowler's "Agent = Model + Harness"
//! framework by programmatically verifying architectural invariants:
//! 1. Engine Dependency Purity: `firstproof-engine` maintains zero platform OS, windowing,
//!    or C-bound library dependencies.
//! 2. Terminal Display Transform Invariant: Tone mapping and gamma encoding strictly execute
//!    as the final stage of rendering pipelines across Metal, WebGL, and Rust CPU export.
//! 3. Rec.2020 Sensitometric Luminance Invariant: Wide-gamut photopic luminance operations
//!    strictly evaluate ITU-R BT.2020 coefficients [0.2627, 0.6780, 0.0593].

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().expect("Repository root").to_path_buf()
}

/// Evaluates dependency purity on a Cargo.toml string.
pub fn check_engine_dependencies_purity(toml_content: &str) -> Result<(), String> {
    let prohibited_crates = [
        "rusqlite",
        "sqlite",
        "lcms2",
        "lcms2-sys",
        "libraw-sys",
        "raw-loader",
        "web-sys",
        "js-sys",
        "wasm-bindgen", // WASM host adapter belongs in firstproof-wasm, not engine
        "metal",
        "cocoa",
        "objc",
        "appkit",
        "core-graphics",
        "winit",
        "tokio",
        "rayon", // Multithreading host adapter belongs in firstproof-core
    ];

    let in_deps = toml_content
        .lines()
        .skip_while(|line| !line.trim().starts_with("[dependencies]"))
        .skip(1)
        .take_while(|line| !line.trim().starts_with('['));

    for line in in_deps {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let crate_name = trimmed
            .split('=')
            .next()
            .unwrap_or("")
            .trim()
            .trim_matches('"');

        for prohibited in &prohibited_crates {
            if crate_name == *prohibited {
                return Err(format!(
                    "Architectural Fitness Violation: 'firstproof-engine' must remain a pure algorithmic domain kernel. Found prohibited dependency '{}'.",
                    prohibited
                ));
            }
        }
    }
    Ok(())
}

/// Verifies that an upstream adjustment stage strictly precedes a downstream terminal stage.
pub fn check_pipeline_stage_order(code: &str, upstream: &str, downstream: &str) -> Result<(), String> {
    let pos_up = code
        .find(upstream)
        .ok_or_else(|| format!("Missing expected upstream stage '{}'", upstream))?;
    let pos_down = code
        .rfind(downstream)
        .ok_or_else(|| format!("Missing expected terminal stage '{}'", downstream))?;

    if pos_up > pos_down {
        return Err(format!(
            "Architectural Fitness Violation: Upstream stage '{}' (pos {}) occurs after terminal stage '{}' (pos {}). Output transforms must be strictly terminal.",
            upstream, pos_up, downstream, pos_down
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_engine_dependency_purity() {
    let engine_cargo_path = repo_root().join("firstproof-engine/Cargo.toml");
    let content = fs::read_to_string(&engine_cargo_path)
        .unwrap_or_else(|_| panic!("Failed to read {:?}", engine_cargo_path));

    check_engine_dependencies_purity(&content).expect("firstproof-engine dependency purity verified");
}

#[test]
fn test_engine_dependency_purity_failure_mode() {
    let bad_toml = r#"
[package]
name = "firstproof-engine"
version = "0.1.0"

[dependencies]
serde = "1.0"
rusqlite = "0.31"
"#;
    let err = check_engine_dependencies_purity(bad_toml).unwrap_err();
    assert!(err.contains("Found prohibited dependency 'rusqlite'"));
}

#[test]
fn test_terminal_display_transform_ordering_metal() {
    let shader_path = repo_root().join("FirstProofApp/Sources/Shaders/Shaders.metal");
    let content = fs::read_to_string(&shader_path)
        .unwrap_or_else(|_| panic!("Failed to read {:?}", shader_path));

    let frag_start = content
        .find("fragment fragmentShaderOutput fragmentShader(")
        .expect("Found fragmentShader in Shaders.metal");
    let frag_body = &content[frag_start..];

    // Assert that upstream optical adjustments precede terminal tone mapping
    for upstream in &[
        "applyBilateralToneRecovery",
        "applyRecordingEmulsion",
        "applyColorAdjustments",
        "applySplitToning",
        "applyDetailAndSharpening",
    ] {
        check_pipeline_stage_order(frag_body, upstream, "agxToneMapVivid")
            .unwrap_or_else(|e| panic!("Metal Shader Error: {}", e));
    }

    // Terminal gamma encoding must be the very last return stage
    check_pipeline_stage_order(frag_body, "agxToneMapVivid", "linearToSrgbGamma3_")
        .unwrap_or_else(|e| panic!("Metal Shader Error: {}", e));
}

#[test]
fn test_terminal_display_transform_ordering_wgsl() {
    let wgsl = firstproof_engine::shader::canonical_wgsl();
    let frag_start = wgsl
        .find("fn fragmentShader(")
        .expect("Found fn fragmentShader in canonical WGSL");
    let frag_body = &wgsl[frag_start..];

    for upstream in &[
        "applyBilateralToneRecovery",
        "applyRecordingEmulsion",
        "applyColorAdjustments",
        "applySplitToning",
        "applyDetailAndSharpening",
    ] {
        check_pipeline_stage_order(frag_body, upstream, "agxToneMapVivid")
            .unwrap_or_else(|e| panic!("WGSL Shader Error: {}", e));
    }

    check_pipeline_stage_order(frag_body, "agxToneMapVivid", "linearToSrgbGamma3")
        .unwrap_or_else(|e| panic!("WGSL Shader Error: {}", e));
}

#[test]
fn test_terminal_display_transform_ordering_webgl() {
    let shader_path = repo_root().join("web/src/shaders/GeneratedGLSL.ts");
    let content = fs::read_to_string(&shader_path)
        .unwrap_or_else(|_| panic!("Failed to read {:?}", shader_path));

    let main_start = content.find("void main() {").expect("Found void main() in GeneratedGLSL.ts");
    let main_body = &content[main_start..];

    // Assert that upstream optical adjustments precede terminal tone mapping
    for upstream in &[
        "applyBilateralToneRecovery",
        "applyRecordingEmulsion",
        "applyColorAdjustments",
        "applySplitToning",
        "applyDetailAndSharpening",
    ] {
        check_pipeline_stage_order(main_body, upstream, "agxToneMapVivid")
            .unwrap_or_else(|e| panic!("WebGL Shader Error: {}", e));
    }

    // Terminal gamma encoding must be the very last assignment to output
    check_pipeline_stage_order(main_body, "agxToneMapVivid", "linearToSrgbGamma3_")
        .unwrap_or_else(|e| panic!("WebGL Shader Error: {}", e));
}

#[test]
fn test_terminal_display_transform_ordering_rust_pipeline() {
    let pipeline_path = repo_root().join("firstproof-core/src/export/pipeline.rs");
    let content = fs::read_to_string(&pipeline_path)
        .unwrap_or_else(|_| panic!("Failed to read {:?}", pipeline_path));

    let fn_start = content.find("pub fn apply_edits").expect("Found apply_edits in pipeline.rs");
    let fn_body = &content[fn_start..];

    // Assert that upstream optical adjustments precede terminal tone mapping
    for upstream in &[
        "apply_bilateral_tone_recovery_rgb",
        "apply_recording_emulsion",
        "apply_color_adjustments",
        "apply_split_toning",
    ] {
        check_pipeline_stage_order(fn_body, upstream, "agx_tone_map_vivid")
            .unwrap_or_else(|e| panic!("CPU Pipeline Error: {}", e));
    }
}

#[test]
fn test_terminal_display_transform_failure_mode() {
    let bad_pipeline = r#"
    let color = agx_tone_map_vivid(raw);
    let bad = apply_color_adjustments(color); // Invariant violation: modifying tone-mapped pixels
    "#;
    let err = check_pipeline_stage_order(bad_pipeline, "apply_color_adjustments", "agx_tone_map_vivid").unwrap_err();
    assert!(err.contains("occurs after terminal stage"));
}

#[test]
fn test_rec2020_luminance_coefficient_purity() {
    let metal_shader = fs::read_to_string(repo_root().join("FirstProofApp/Sources/Shaders/Shaders.metal")).unwrap();
    let webgl_shader = fs::read_to_string(repo_root().join("web/src/shaders/GeneratedGLSL.ts")).unwrap();
    let wgsl_shader = firstproof_engine::shader::canonical_wgsl();
    let rust_pipeline = fs::read_to_string(repo_root().join("firstproof-core/src/export/pipeline.rs")).unwrap();

    let r_coeff = "0.2627";
    let g_coeff = "0.678";
    let b_coeff = "0.0593";

    // 1. Metal Shader Verification
    assert!(metal_shader.contains(r_coeff), "Shaders.metal must evaluate Rec.2020 Red coeff 0.2627");
    assert!(metal_shader.contains(g_coeff), "Shaders.metal must evaluate Rec.2020 Green coeff 0.678");
    assert!(metal_shader.contains(b_coeff), "Shaders.metal must evaluate Rec.2020 Blue coeff 0.0593");

    // 2. WebGL Shader Verification (GeneratedGLSL.ts)
    assert!(webgl_shader.contains(r_coeff), "GeneratedGLSL.ts must evaluate Rec.2020 Red coeff 0.2627");
    assert!(webgl_shader.contains(g_coeff), "GeneratedGLSL.ts must evaluate Rec.2020 Green coeff 0.678");
    assert!(webgl_shader.contains(b_coeff), "GeneratedGLSL.ts must evaluate Rec.2020 Blue coeff 0.0593");

    // 3. Canonical WGSL Verification
    assert!(wgsl_shader.contains(r_coeff), "Canonical WGSL must evaluate Rec.2020 Red coeff 0.2627");
    assert!(wgsl_shader.contains(g_coeff), "Canonical WGSL must evaluate Rec.2020 Green coeff 0.678");
    assert!(wgsl_shader.contains(b_coeff), "Canonical WGSL must evaluate Rec.2020 Blue coeff 0.0593");

    // 4. Rust CPU Pipeline Verification
    assert!(rust_pipeline.contains(r_coeff), "pipeline.rs must evaluate Rec.2020 Red coeff 0.2627");
    assert!(rust_pipeline.contains(g_coeff), "pipeline.rs must evaluate Rec.2020 Green coeff 0.678");
    assert!(rust_pipeline.contains(b_coeff), "pipeline.rs must evaluate Rec.2020 Blue coeff 0.0593");
}


#[test]
fn test_dual_domain_performance_asymmetry_invariants() {
    let export_batch = fs::read_to_string(repo_root().join("firstproof-core/src/export/batch.rs")).unwrap();
    let export_lib = fs::read_to_string(repo_root().join("firstproof-core/src/lib.rs")).unwrap();
    let export_encoders = fs::read_to_string(repo_root().join("firstproof-core/src/export/encoders.rs")).unwrap();

    // 1. Export pipelines must default to RenderTier::ArchivalExport ("hangs on wall for 10 years")
    assert!(
        export_batch.contains("unwrap_or(crate::RenderTier::ArchivalExport)"),
        "Export batch must default to ArchivalExport tier"
    );
    assert!(
        export_lib.contains("unwrap_or(RenderTier::ArchivalExport)"),
        "export_photo in firstproof-core must default to ArchivalExport tier"
    );
    assert!(
        export_encoders.contains("RenderTier::ArchivalExport"),
        "export_photo_to_disk must execute with ArchivalExport tier"
    );
}
