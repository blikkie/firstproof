# ADR-0004: Dual-Domain Performance Asymmetry ("Wall for 10 Years")

- **Status**: Accepted
- **Date**: 2026-03-20
- **Deciders**: blikkie, Antigravity
- **Consulted / Impacted Subsystems**: `firstproof-engine`, `FirstProofApp`, `web`, `firstproof-core/src/export`

## Context & Problem Statement
Interactive photo darkroom applications face two conflicting computational goals:
1. High interactivity during culling and editing: users switch between 30–60MB RAW files and scrub sliders expecting instant 60 FPS feedback and 10–15ms photo transitions.
2. Archival print & export fidelity: fine-art prints demand uncompromising optical demosaicing, continuous $C^1$ tone mapping, full floating-point precision, and zero computational shortcutting.

Attempting to run full AHD/DCB/AMAZE demosaicing at 60 FPS on 45MP RAW files causes thermal throttling, dropped frames, and laggy slider response. Conversely, using fast binned approximations for archival export produces visible labyrinth artifacts, aliasing, and degraded print resolution.

## Decision Drivers
- **Product Creed**: *"You only export it once, and then it hangs on your wall for 10 years."* Archival output must never be compromised for vanity export benchmarks.
- **Viewport Ergonomics**: Friction in the viewport is fatal. Culling and slider scrubbing must feel instantaneous.
- **Zero Camera-JPEG Viewport Flashing**: Never show the embedded camera JPEG preview, which has bake-in 8-bit clipping and inconsistent manufacturer picture styles.

## Considered Options
- **Option 1**: Symmetric lightweight pipeline—use fast binned approximations everywhere (including export).
- **Option 2**: Symmetric heavy pipeline—run full multi-pass demosaicing on every viewport frame and image switch.
- **Option 3**: Asymmetric dual-domain pipeline—2×2 Bayer box binning on GPU/WASM for 10–15ms viewport rendering; full multi-pass 32-bit float CPU demosaicing and archival ICC injection on export.

## Decision Outcome
Chosen option: **Option 3 (Asymmetric Dual-Domain Pipeline)**.
- **Viewport Domain**: Selection streams directly into 2×2 Bayer box binning (`decodeRawPreviewBinned`). Viewport textures are cached, and fragment shaders apply develop adjustments at 60 FPS in real time.
- **Export & High-Fidelity Domain**: Export pipeline loads full CFA sensor data, executes multi-pass floating-point demosaicing, applies full linear optical pipelines, and encodes to archival TIFF/JPEG with embedded ICC profiles.

### Positive Consequences
- Viewport achieves locked 60 FPS and 10–15ms photo switching latency.
- Export fidelity is maximized without compromise.
- No reliance on proprietary or clipped camera JPEGs during RAW editing.

### Negative Consequences & Accepted Trade-offs
- Codebases must maintain two demosaicing paths: fast box-binned preview and full-resolution demosaicing.
- Very high zoom levels in the viewport (beyond 100%) reflect binned resolution until high-fidelity 1:1 preview tiles are computed.

## Implementation Links & Code Anchors
- Binned Preview Decoder: [firstproof-engine/src/raw.rs](../../firstproof-engine/src/raw.rs)
- Full Archival Export: [firstproof-core/src/export/pipeline.rs](../../firstproof-core/src/export/pipeline.rs)

