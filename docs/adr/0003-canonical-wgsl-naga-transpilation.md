# ADR-0003: Canonical WGSL with Naga Transpilation for Cross-Platform Shaders

- **Status**: Accepted
- **Date**: 2026-04-10
- **Deciders**: blikkie, Antigravity
- **Consulted / Impacted Subsystems**: `firstproof-engine/shaders`, `FirstProofApp`, `web/src/render`, `firstproof-cli`

## Context & Problem Statement
First Proof renders real-time darkroom adjustments on both macOS native (Metal via Apple Silicon GPU) and Web (WebGL 2.0 / WebGPU in browsers). Maintaining parallel shader implementations in MSL (`Shaders.metal`) and GLSL/WGSL (`FragmentShader.ts` / `GeneratedGLSL.ts`) causes mathematical drift, subtle rendering parity discrepancies, and doubled maintenance cost for every new develop slider or curve adjustment.

## Decision Drivers
- **Universal Parity Invariant**: Every slider, tone curve, and optical transform must render identically on macOS and Web.
- **Single Source of Truth**: New develop adjustments must be defined once in shader code.
- **Tooling stability**: Must integrate into the fast inner loop and pre-commit verification gates.

## Considered Options
- **Option 1**: Hand-write MSL for Metal and hand-write GLSL for WebGL with manual visual audits.
- **Option 2**: Author in MSL and use SPIRV-Cross / external C++ transpilers to generate WebGL GLSL.
- **Option 3**: Author canonical shaders once in WebGPU Shading Language (WGSL) in `firstproof-engine/shaders/` and use Mozilla's pure Rust `naga` crate via `firstproof-cli shaders` to cross-compile into MSL, GLSL, and WebGPU.

## Decision Outcome
Chosen option: **Option 3 (Canonical WGSL via Naga)**. All viewport fragment processing is authored in `firstproof-engine/shaders/darkroom.wgsl`. The CLI command `cargo run -p firstproof-core --bin firstproof-cli -- shaders` automatically regenerates `Shaders.metal` for macOS and `GeneratedGLSL.ts` for the web darkroom.

### Positive Consequences
- Zero mathematical drift between native Metal and web WebGL outputs.
- Single edit location for all develop shader adjustments.
- Automated sensor check (`firstproof-cli shaders --check` in `./scripts/check.sh`) guarantees committed shaders never diverge from canonical WGSL source.

### Negative Consequences & Accepted Trade-offs
- Shader features are constrained to the common subset supported by Naga's WGSL frontend and MSL/GLSL backends.
- Developers must run the CLI shader generator whenever editing `darkroom.wgsl`.

## Implementation Links & Code Anchors
- Canonical WGSL Shader: [firstproof-engine/shaders/darkroom.wgsl](../../firstproof-engine/shaders/darkroom.wgsl)
- Shader Transpiler CLI: [firstproof-core/src/bin/cli.rs](../../firstproof-core/src/bin/cli.rs)
- Generated MSL: [FirstProofApp/Sources/Shaders/Shaders.metal](../../FirstProofApp/Sources/Shaders/Shaders.metal)
- Generated GLSL: [web/src/shaders/GeneratedGLSL.ts](../../web/src/shaders/GeneratedGLSL.ts)

