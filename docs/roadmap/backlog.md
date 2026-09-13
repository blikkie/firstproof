# First Proof Active Backlog & Roadmap

This document strictly tracks pending and deferred features calibrated against our core philosophy: **"Pure capture. Honest color. Zero digital plastic. Print & web first."**

> **Note**: For completed stories, see [completed.md](file:///Users/blikkie/src/LumenBox/docs/roadmap/completed.md). For color pipeline specifications, see [color_pipeline.md](file:///Users/blikkie/src/LumenBox/docs/architecture/color_pipeline.md).

---

## 🚀 Immediate Next Up
1. **Highlight & Shadow Recovery (P1)**: Parametric selective tonemapping curve + bilateral base/detail separation on GPU to rescue dynamic range without midtone clipping or halo artifacts.
2. **Auto-Crop & Inscribed Bounding Box on Straighten (P1)**: Inscribed aspect-ratio zoom scaling $S(\theta) = |\cos\theta| + |\sin\theta|\cdot\max(R, 1/R)$ in Metal shader to eliminate black border wedges during rotation.

---

## 1. Darkroom & Develop Adjustments (P1)

### Highlight & Shadow Recovery
- **Parametric Selective Tonemapping**: Curve adjusting tones based on distance from white (highlights) or black (shadows):
  $$L_{new} = L \cdot (1 + S_{Shadows} \cdot (1 - L)^2 - H_{Highlights} \cdot L^2)$$
- **GPU Bilateral Filtering**: Splits image into base layer (low-frequency lighting transitions) and detail layer (texture). Highlight/shadow adjustments apply strictly to base layer to eliminate halo artifacts.

### Auto-Crop & Inscribed Bounding Box on Straighten
- When rotating canvas by angle $\theta$, eliminate black boundary wedges by computing the minimum inscribed scale factor:
  $$S(\theta) = |\cos \theta| + |\sin \theta| \cdot \max(R, 1/R)$$
- In `Shaders.metal`, multiply UV coordinates by $S(\theta)$ around $(0.5, 0.5)$ so the canvas automatically zooms to fill the entire frame without black bars.

### Saturation & Vibrance
- **Saturation**: Multiplies chrominance uniformly in linear Rec.2020 space.
- **Vibrance**: Selectively boosts less-saturated colors while protecting skin tones and already-saturated channels.

### Sharpening & Sensor Detail Engine
- **Luminance Unsharp Masking (USM)**: Applied strictly to perceptual Luminance ($Y$) to avoid color noise amplification.
- **Sobel Edge Masking & Thresholding**: Protects flat skies and smooth skin tones from noise amplification.
- **Directional RCD / VNG Demosaicing**: Upgrade RAW Bayer demosaic pipeline via `bayer` crate.
- **Anti-Slop Invariant**: Wide-radius clarity and heuristic dehaze remain permanently banned to prevent plastic HDR halos.

---

## 2. Google Liberation Bridge (Post-v1)
- **Pixel Computational Linear DNG Ingestion**: Ingest 16-bit multi-frame linear DNGs (`PXL_*.dng`) with embedded lens shading gain maps.
- **Takeout & Local Archive Extraction**: Process Google Takeout archive folders directly, linking `.dng`/`.jpg` files with accompanying `.json` metadata sidecars.
- **Storage Reclamation Manifest**: Generate deletion manifests of confirmed "Rejects" and redundant JPEGs so users can reclaim Google cloud storage while safely keeping master DNGs locally.

---

## 3. Universal Non-Destructive Sidecar Architecture
- Treat all source formats (RAW, JPEG, HEIC, PNG, TIFF) as immutable read-only master files.
- Store user adjustments in adjacent `.xmp` sidecars conforming to `https://firstproof.photo/ns/pm/1.0/`.
- Export pristine derivative images on demand using `img-parts` without generation loss.

---

## 4. Bulk Export & Soft Proofing (P1 & P2)
- **Multi-Selection Batch Export UI**: Export *Selected Photos*, *Filtered Picks*, or *Entire Catalog* (`Cmd+Shift+E`).
- **Rayon Multi-Core Thread Pool**: Parallel batch exports across Apple Silicon performance & efficiency cores.
- **ColorSync Soft Proofing**: Canvas toolbar mode simulating sRGB and print gamut clipping on wide-gamut displays.
- **Rendering Intent Control**: Integrate LittleCMS2 (`lcms2`) in Rust export for Perceptual vs. Relative Colorimetric rendering intents.
