//! Canonical Unified Shader Pipeline (WGSL -> Metal & WebGL 2 GLSL)
//!
//! Provides single-source-of-truth shaders authored in WGSL, validated with `naga`,
//! and cross-compiled deterministically into Metal Shading Language and WebGL 2 GLSL ES 300.

use std::collections::BTreeMap;
use naga::back::{glsl, msl};
use naga::front::wgsl;
use naga::valid::{Capabilities, ValidationFlags, Validator};

/// Error during shader parsing, validation, or cross-compilation.
#[derive(Debug, Clone)]
pub enum ShaderError {
    Parse(String),
    Validation(String),
    Msl(String),
    Glsl(String),
}

impl std::fmt::Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(msg) => write!(f, "WGSL parse error: {}", msg),
            Self::Validation(msg) => write!(f, "Naga validation error: {}", msg),
            Self::Msl(msg) => write!(f, "MSL compilation error: {}", msg),
            Self::Glsl(msg) => write!(f, "GLSL compilation error: {}", msg),
        }
    }
}

impl std::error::Error for ShaderError {}

/// Returns the concatenated, single-source-of-truth canonical WGSL darkroom shader.
pub fn canonical_wgsl() -> &'static str {
    concat!(
        include_str!("../../../firstproof-engine/shaders/types.wgsl"),
        "\n",
        include_str!("../../../firstproof-engine/shaders/constants.wgsl"),
        "\n",
        include_str!("../../../firstproof-engine/shaders/geometry.wgsl"),
        "\n",
        include_str!("../../../firstproof-engine/shaders/detail.wgsl"),
        "\n",
        include_str!("../../../firstproof-engine/shaders/color_science.wgsl"),
        "\n",
        include_str!("../../../firstproof-engine/shaders/darkroom.wgsl"),
    )
}

/// Compiles canonical WGSL to Metal Shading Language (.metal) for macOS AppKit viewport rendering.
pub fn compile_to_msl() -> Result<String, ShaderError> {
    let source = canonical_wgsl();
    let module = wgsl::parse_str(source).map_err(|e| ShaderError::Parse(e.emit_to_string(source)))?;
    
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    let info = validator.validate(&module).map_err(|e| ShaderError::Validation(e.emit_to_string(source)))?;
    
    let mut msl_options = msl::Options::default();
    
    // Map resource bindings for fragmentShader to exact Metal slots
    let mut fs_res = msl::EntryPointResources::default();
    // buffer(1): ShaderUniforms
    fs_res.resources.insert(
        naga::ResourceBinding { group: 0, binding: 0 },
        msl::BindTarget {
            buffer: Some(1),
            texture: None,
            sampler: None,
            external_texture: None,
            mutable: false,
        },
    );
    // texture(0): u_texture
    fs_res.resources.insert(
        naga::ResourceBinding { group: 0, binding: 1 },
        msl::BindTarget {
            buffer: None,
            texture: Some(0),
            sampler: None,
            external_texture: None,
            mutable: false,
        },
    );
    // sampler(0): u_sampler
    fs_res.resources.insert(
        naga::ResourceBinding { group: 0, binding: 2 },
        msl::BindTarget {
            buffer: None,
            texture: None,
            sampler: Some(msl::BindSamplerTarget::Resource(0)),
            external_texture: None,
            mutable: false,
        },
    );
    // texture(1): u_lut_3d
    fs_res.resources.insert(
        naga::ResourceBinding { group: 0, binding: 3 },
        msl::BindTarget {
            buffer: None,
            texture: Some(1),
            sampler: None,
            external_texture: None,
            mutable: false,
        },
    );
    // sampler(1): u_lut_sampler
    fs_res.resources.insert(
        naga::ResourceBinding { group: 0, binding: 4 },
        msl::BindTarget {
            buffer: None,
            texture: None,
            sampler: Some(msl::BindSamplerTarget::Resource(1)),
            external_texture: None,
            mutable: false,
        },
    );
    // texture(2): u_curve_master_lut
    fs_res.resources.insert(
        naga::ResourceBinding { group: 0, binding: 5 },
        msl::BindTarget {
            buffer: None,
            texture: Some(2),
            sampler: None,
            external_texture: None,
            mutable: false,
        },
    );
    // texture(3): u_curve_rgb_lut
    fs_res.resources.insert(
        naga::ResourceBinding { group: 0, binding: 6 },
        msl::BindTarget {
            buffer: None,
            texture: Some(3),
            sampler: None,
            external_texture: None,
            mutable: false,
        },
    );
    // sampler(2): u_curve_sampler
    fs_res.resources.insert(
        naga::ResourceBinding { group: 0, binding: 7 },
        msl::BindTarget {
            buffer: None,
            texture: None,
            sampler: Some(msl::BindSamplerTarget::Resource(2)),
            external_texture: None,
            mutable: false,
        },
    );
    // texture(4): u_gain_map
    fs_res.resources.insert(
        naga::ResourceBinding { group: 0, binding: 8 },
        msl::BindTarget {
            buffer: None,
            texture: Some(4),
            sampler: None,
            external_texture: None,
            mutable: false,
        },
    );
    // sampler(3): u_gain_map_sampler
    fs_res.resources.insert(
        naga::ResourceBinding { group: 0, binding: 9 },
        msl::BindTarget {
            buffer: None,
            texture: None,
            sampler: Some(msl::BindSamplerTarget::Resource(3)),
            external_texture: None,
            mutable: false,
        },
    );
    // texture(5): u_proof_lut
    fs_res.resources.insert(
        naga::ResourceBinding { group: 0, binding: 11 },
        msl::BindTarget {
            buffer: None,
            texture: Some(5),
            sampler: None,
            external_texture: None,
            mutable: false,
        },
    );
    // sampler(4): u_proof_sampler
    fs_res.resources.insert(
        naga::ResourceBinding { group: 0, binding: 12 },
        msl::BindTarget {
            buffer: None,
            texture: None,
            sampler: Some(msl::BindSamplerTarget::Resource(4)),
            external_texture: None,
            mutable: false,
        },
    );
    
    msl_options.per_entry_point_map.insert("fragmentShader".to_string(), fs_res);
    
    // Map resource bindings for vertexShader: buffer(0) for vertices array
    let mut vs_res = msl::EntryPointResources::default();
    vs_res.resources.insert(
        naga::ResourceBinding { group: 0, binding: 10 },
        msl::BindTarget {
            buffer: Some(0),
            texture: None,
            sampler: None,
            external_texture: None,
            mutable: false,
        },
    );
    msl_options.per_entry_point_map.insert("vertexShader".to_string(), vs_res);
    
    let mut msl_out = String::new();
    let msl_pipeline_options = msl::PipelineOptions::default();
    let mut msl_writer = msl::Writer::new(&mut msl_out);
    msl_writer
        .write(&module, &info, &msl_options, &msl_pipeline_options)
        .map_err(|e| ShaderError::Msl(e.to_string()))?;
        
    Ok(msl_out)
}

/// Compiles canonical WGSL to GLSL 300 ES fragment shader for WebGL 2 web darkroom rendering.
pub fn compile_to_glsl() -> Result<String, ShaderError> {
    let source = canonical_wgsl();
    let module = wgsl::parse_str(source).map_err(|e| ShaderError::Parse(e.emit_to_string(source)))?;
    
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    let info = validator.validate(&module).map_err(|e| ShaderError::Validation(e.emit_to_string(source)))?;
    
    let mut glsl_binding_map = BTreeMap::new();
    glsl_binding_map.insert(naga::ResourceBinding { group: 0, binding: 0 }, 0); // UBO
    glsl_binding_map.insert(naga::ResourceBinding { group: 0, binding: 1 }, 0); // texture 0
    glsl_binding_map.insert(naga::ResourceBinding { group: 0, binding: 3 }, 1); // lut 1
    glsl_binding_map.insert(naga::ResourceBinding { group: 0, binding: 5 }, 2); // master curve 2
    glsl_binding_map.insert(naga::ResourceBinding { group: 0, binding: 6 }, 3); // rgb curve 3
    glsl_binding_map.insert(naga::ResourceBinding { group: 0, binding: 8 }, 4); // gain map 4
    
    let glsl_options = glsl::Options {
        version: glsl::Version::Embedded { version: 300, is_webgl: true },
        binding_map: glsl_binding_map,
        ..Default::default()
    };
    
    let pipeline_options = glsl::PipelineOptions {
        shader_stage: naga::ShaderStage::Fragment,
        entry_point: "fragmentShader".to_string(),
        multiview: None,
    };
    
    let mut glsl_out = String::new();
    let mut glsl_writer = glsl::Writer::new(
        &mut glsl_out,
        &module,
        &info,
        &glsl_options,
        &pipeline_options,
        naga::proc::BoundsCheckPolicies::default(),
    ).map_err(|e| ShaderError::Glsl(e.to_string()))?;
    
    glsl_writer.write().map_err(|e| ShaderError::Glsl(e.to_string()))?;
    Ok(glsl_out)
}
