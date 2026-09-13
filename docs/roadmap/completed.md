# First Proof Completed Stories Archive

This archive tracks completed stories and features moved from the active backlog.

---

## 1. Core Workflow & Triage
- **Comprehensive Undo/Redo & History Engine**: Coalesced rapid slider scrubbing into atomic undo steps based on drag lifecycle (`beginScrub` / `endScrub`). Value-type `EditSettings` snapshots (~250 bytes). Expandable Inspector `HistorySectionView` allowing instant time-travel.
- **Single-Key Professional Muscle Memory**: Single-key shortcuts (`1`..`5` ratings, `P`/`X`/`U` flags, `C` crop, `Z` zoom, `\` original peek) guarded by first-responder shielding against active text fields.
- **Native macOS Polish**: Regular activation policy (`NSApplication.shared.setActivationPolicy(.regular)`), session persistence of `lastOpenedDirectoryPath`, and distributable `FirstProof.app` bundle via `make bundle`.
- **Portable Catalogs on External Drives (`.firstproof/`)**: Folder indexing on external storage volumes stores catalogs inside `<volume>/.firstproof/catalog.db` so the catalog travels with external media.

---

## 2. Color Science & Camera Profiles
- **Story 5.1: Linear Wide-Gamut Working Space & Pipeline Foundation**: Standardized on Linear Rec.2020 with D65 white point in both Rust and Metal. Baseline matrix conversions: RAW Sensor $\to$ CIE XYZ D50 $\to$ Linear Rec.2020.
- **Story 5.2: AgX Filmic Tone Mapping Engine & Parity Guard**: AgX Inset matrix, hue preservation, filmic desaturation shoulder, and sigmoidal tone curve. Verified with automated `agx_shader_parity` integration tests ($\epsilon < 10^{-6}$).
- **Story 5.3: Adobe DCP Ingestion & Dual-Illuminant Camera Calibration**: Rust DCP parser (`dcp.rs`) extracting matrices and dual-illuminant Planckian interpolation between Illuminant A (2856K) and D65 (6504K).
- **Story 5.4: Camera-Matching Tone Profiles & DCP HueSatMap Engine**: DCP HueSatMap 3D HSV delta tables and manufacturer tone curves. Swappable profiles ("First Proof Neutral (AgX)", "Camera Standard", "Camera Portrait", "Camera Landscape", "Camera Vivid"). Verified with `camera_profiles_parity`.
- **Story 5.5: Secondary 3D LUT Engine (.cube) & Color Nuance**: Evaluated post-tone-mapping in collapsible Fine Tuning disclosure panel. 3D Metal texture with trilinear filtering; SIMD tetrahedral interpolation in Rust CPU export.
- **Story 5.6: Dual-Target Export (Display P3 & sRGB)**: Export color space selector with embedded ICC profiles generated via `lcms2` and injected via `img-parts`.

---

## 3. Darkroom & Develop Adjustments
- **Story 6.4: White Balance Picker**: 5×5 neighborhood sampling in Metal canvas viewport, Planckian Kelvin / Tint computation, and XMP persistence.
- **Conservative Auto-Leveling**: Scene exposure offset anchoring optical midtones at 18% reflectance based on middle-luminance percentiles.
- **Sub-millisecond Thumbnail Invalidation**: Viewport snapshots captured into `ThumbnailCache` without re-triggering disk demosaicing.
- **Auto-Crop & Inscribed Bounding Box on Straighten**: Implemented minimum inscribed aspect ratio scale factor $S(\theta) = |\cos\theta| + |\sin\theta|\cdot\max(R, 1/R)$ in `Geometry.metal` and `encoders.rs` to automatically eliminate black rotation wedges.
- **Selective Tone Recovery (Highlights, Shadows, Whites, Blacks)**: Implemented in `ColorScience.metal` and `pipeline.rs` with EV-based bell curves anchoring midtones at 18% gray with dual-engine parity.
- **Saturation & Vibrance Engine**: Linear Rec.2020 chrominance scaling and smart vibrance protecting skin tones in `ColorScience.metal` and `pipeline.rs`.
- **Sharpening & Sensor Detail Engine (USM, Radius, Masking)**: Perceptual luminance unsharp masking with Sobel edge thresholding in `Shaders.metal` and `pipeline.rs` with automated parity tests.
- **Navigation Boundary Guard**: Fixed boundary navigation reload bug in `AppViewModel+Selection.swift` to prevent redundant decodes when arrowing past catalog boundaries.
- **Phase 2.1: Extended White Balance & Physical Illuminant Presets**: Expanded tint headroom to $\pm 200$ with floor scale protection ($\ge 0.01$) preventing color inversion or clipping under intense industrial/subway spectral mercury and sodium spikes. Added unobtrusive dropdown menu in Color Inspector supporting physical illuminants (`Daylight`, `Cloudy`, `Shade`, `Tungsten`, `Fluorescent` $+45$, `Flash`, `As Shot`, `Auto`, `Custom`). Implemented Gray-World auto white balance solver in `auto.rs` accessible via dedicated preset selection, keeping `Auto Tone` strictly focused on conservative exposure/contrast leveling to avoid false color casts.

---

## 4. Ingestion, Sidecars & Batch Export
- **Universal Non-Destructive Sidecar Architecture**: Immutable master files with firstproof-native `.xmp` sidecars (`https://firstproof.photo/ns/pm/1.0/`), Adobe CRS ingestion, and `img-parts` ICC/metadata injection.
- **Multi-Selection Batch Export UI & Rayon Thread Pool**: Parallel batch export sheet (`ExportSheetView.swift`) powered by Rayon multi-core pool, sRGB/Display-P3 profiles, and LittleCMS2 rendering intent control.
- **Google Takeout Archive Extraction**: Rust parser in `takeout.rs` matching `.json` sidecars to images and populating EXIF/GPS/timestamps during catalog indexing.

---

## 5. Performance & RAW Pipeline Architecture
- **Phase 1.1: High-Throughput RAW Pipeline & Viewport Decimation**:
  - *Pre-Allocated Scratch Arenas*: Reusable thread-local `DecodeScratchArena` retaining intermediate demosaic and linear memory buffers, completely eliminating 600MB+ per-photo heap churn across culling sessions.
  - *Viewport-Aware 2×2 Bayer Box Binning*: Direct Bayer quad collapse into Linear Rec.2020 half-float RGBA buffers for instant 10–15ms viewport preview decode. Zero demosaic interpolation overhead.
  - *ARM NEON 128-Bit SIMD Alignment*: Vectorized Planckian sensor-to-Rec.2020 matrix transformation using `fmla.4s` (`vmlaq_n_f32`), hardware half-float conversions (`vcvt_f16_f32`), and 4-way interleaved vector store (`vst4_u16`).
  - *Rayon Multi-Core Parallel Demosaic*: Chunked multi-core parallel processing across performance and efficiency cores for both binned preview and full-resolution demosaic.
  - *Progressive Hybrid Viewport Streaming*: Instant 10–15ms binned preview promotion for fluid culling navigation with background full-res stream for 100% zoom pixel peeping.
- **Phase 1.2: Directional Neighbor Preload & LRU Ring Buffer**:
  - *Thread-Safe In-Memory Ring Buffer (`PreviewRingBuffer`)*: 5-slot bounded LRU cache for pre-decoded `RawPreviewRecord` instances protected by `NSLock`, bounding working memory to ~240–450MB and achieving 0ms perceived arrow navigation.
  - *Directional Lookahead Worker (`LookaheadPreloadService`)*: Asynchronously warms up $[-1, +1, -2, +2]$ neighbor frames on a dedicated low-priority background queue (`qos: .utility`) with token-based cancellation during rapid scrubbing.
  - *Zero Dark Transition on Pre-Warmed Frames*: Synchronous promotion of cached binned records in `loadPreview(for:)` eliminates dark canvas transitions and spinners during burst navigation.
- **Phase 1.3: Per-Photo In-Memory Edit History Stack**:
  - *Isolated Darkroom History (`PhotoEditHistory`)*: Dedicated per-photo undo/redo stacks capped at 50 steps (<15 KB RAM per photo). Completely eliminates accidental off-screen mutations when pressing `Cmd+Z` in Detail view.
  - *Darkroom vs. Catalog Undo Routing*: Detail view `Cmd+Z` strictly targets the active photo's adjustment history; Grid view `Cmd+Z` targets the catalog session stack (batch ratings, flags).
  - *Session-Only Memory Invariant*: History lives in memory for fluid session workflow; non-destructive edits persist permanently to XMP sidecars without cross-restart history tree serialization bloat.
