# ADR-0002: Linear Rec.2020 Working Gamut & Terminal Display Encoding

- **Status**: Accepted
- **Date**: 2026-03-15
- **Deciders**: blikkie, Antigravity
- **Consulted / Impacted Subsystems**: `firstproof-engine`, `firstproof-core/src/export`, `FirstProofApp/Shaders.metal`, `web/src/render`

## Context & Problem Statement
Digital photo editing systems must process high dynamic range scene-referred data from RAW sensors. Operating in display spaces (like sRGB) or using intermediate gamma encoding causes hue shifts, highlight clipping, unnatural saturation behavior, and digital posterization when applying tone curves and color adjustments.

## Decision Drivers
- **Photographic Creed**: "Pure capture. Honest color. Zero digital plastic."
- **Optical realism**: Adjustments must emulate physical photons and chemical darkroom mechanics (subtractive response, optical falloff, sensitometric grain).
- **Gamut volume**: Must encompass modern wide-gamut displays (Display P3) and archival fine-art print substrates without gamut clipping during editing passes.

## Considered Options
- **Option 1**: Edit directly in non-linear sRGB (conventional lightweight web approach).
- **Option 2**: Edit in ACEScc / ACEScg (motion-picture cinema standard).
- **Option 3**: Edit in Linear Rec.2020 wide-gamut ($Y = 0.2627 R + 0.6780 G + 0.0593 B$) with terminal display/output conversion as strictly the last step.

## Decision Outcome
Chosen option: **Option 3 (Linear Rec.2020 Working Space)**. All develop adjustments (exposure, contrast, tone curves, vibrance, saturation, split toning, detail sharpening, film grain, print media) execute in unbounded Linear Rec.2020. Conversion to display RGB (sRGB, Display P3, gamma encoding) is strictly the terminal pass of GPU shaders and CPU export pipelines.

### Positive Consequences
- Mathematically consistent luminance calculation ($Y = 0.2627 R + 0.6780 G + 0.0593 B$) preserves true perceptual brightness.
- Sensitometric film grain and blur operations behave as physical scene photon interactions.
- Zero gamut clipping during intermediate adjustment passes.
- AgX filmic tone mapper gracefully maps open-domain HDR highlights into display gamut without plastic clipping or hue shifts.

### Negative Consequences & Accepted Trade-offs
- Intermediate textures cannot be sampled directly as display colors without passing through the display transform or intermediate bypass.
- Developers and shaders must strictly maintain floating point linear math and remember never to inject ad-hoc gamma or color space conversions mid-pipeline.

## Implementation Links & Code Anchors
- Pipeline specification: [docs/specs/pipeline_stages_and_color_spaces.md](../specs/pipeline_stages_and_color_spaces.md)
- GPU Metal Shader: [FirstProofApp/Sources/Shaders/Shaders.metal](../../FirstProofApp/Sources/Shaders/Shaders.metal)
- Export Pipeline: [firstproof-core/src/export/pipeline.rs](../../firstproof-core/src/export/pipeline.rs)

