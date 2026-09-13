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
