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
- **Phase 2.1: Extended White Balance & Physical Illuminant Presets**: Expanded tint headroom to $\pm 200$ with floor scale protection ($\ge 0.01$) preventing color inversion or clipping under intense industrial/subway spectral mercury and sodium spikes. Added unobtrusive dropdown menu in Color Inspector supporting physical illuminants (`Daylight`, `Cloudy`, `Shade`, `Tungsten`, `Fluorescent` $+45$, `Sodium Vapor` $2200\text{K}/+160$, `Flash`, `As Shot`, `Auto`, `Custom`). Implemented Gray-World auto white balance solver in `auto.rs` accessible via dedicated preset selection, keeping `Auto Tone` strictly focused on conservative exposure/contrast leveling to avoid false color casts.
- **Story 2.2: Split Toning & Color Grading with 2D Color Wheels & Chemical Presets**: Highlights and shadows split toning anchored in Linear Rec.2020 working space with pure chrominance vectors ($\Delta Y = 0$) and cubic Hermite optical balance transition. Dual-engine bit-level parity verified between Metal GPU shader and Rust CPU export engine (`agx_shader_parity`). Interactive 2D Color Wheels (`ColorWheelView.swift`) manipulating polar Hue/Saturation coordinates. Authentic darkroom chemical presets (`SplitToningPreset.swift`: Selenium, Sulfide Sepia, Cyanotype, Platinum/Palladium, Photographic Print). Per values alignment guardrail, demoted to a collapsed opt-in DisclosureGroup in the Color Inspector to preserve First Proof's lean, restraint-first aesthetic, with the core math and UI directly slated for reuse in Phase 3.1 (Fine-Art Monochromatic Darkroom & Archival Print Toning).
- **Phase 2.2: Advanced Bilateral Base/Detail Tone Recovery (Zero-Halo Engine)**: Upgraded four-zone selective tone recovery (`Highlights`, `Shadows`, `Whites`, `Blacks`) from pointwise parametric curves to an edge-preserving Bilateral Base/Detail Separation Engine. Decomposes scene luminance into a low-frequency base illumination layer and high-frequency detail layer via 8-neighbor range-weighted spatial filtering ($\sigma_r = 0.85$ EV). Evaluates tone curves on the base illumination layer and applies smooth gain, preserving 100% of micro-texture contrast in deep shadows and recovering highlight roll-off without glowing edge halos or boundary bleeding. Verified with automated bit-level parity tests between Metal GPU shaders (`ColorScience.metal`, `Shaders.metal`) and Rust CPU export engine (`pipeline.rs`, `camera_profiles_parity.rs`).

---

## 4. Ingestion, Sidecars & Batch Export
- **Story 2.3: Catalog Ingestion & Deduplication Precedence Engine**:
  - *Deterministic Deduplication Hierarchy*: Implemented in `db.rs` (`import_to_library`) prioritizing RAW master negatives over redundant JPEGs when sharing stems (`raw_over_jpeg_suppressed`), active edits/ratings over untouched originals, DNG standard over proprietary raw, and deterministic tie-breaking.
  - *Duplicate Import Suppression*: Prevents duplicate catalog entries based on matching file size and timestamp/content.
  - *Sidecar Consolidation Engine*: Discovers and merges disparate third-party sidecars (`.xmp`, `.CR2.xmp`, Google Takeout `.json`) into a single canonical First Proof XMP sidecar (`https://firstproof.photo/ns/pm/1.0/`), eliminating destination sidecar clutter and cleaning up source sidecars on move.
  - *Safe Checksum Verification Before Deletion*: Zero-dependency cryptographic SHA-256 streaming verification (`checksum.rs`) comparing source and destination bits before removing source master negatives, strictly preventing data loss.
  - *Removable Media & Memory Card Ingestion*: `MediaDetectionService.swift` detecting camera memory card insertion (DCIM volumes) with safe non-destructive defaults ("Safe Ingest: Keeps camera card intact as primary field backup").
  - *Native macOS Settings Window (`SettingsView.swift`)*: Preferences window (`Cmd+,`) allowing user customization of library location, memory card auto-detection, and ingestion rules.
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

---

## 6. Modern SwiftUI & macOS Architecture Modernization
- **Phase 2.5.1: Desktop Drag-and-Drop Ingestion (`.dropDestination`)**:
  - Implemented macOS 14 native `.dropDestination(for: URL.self)` on both `LibrarySidebarView` and `CanvasView`.
  - Folders and camera cards dropped anywhere on the sidebar or canvas instantly open as active sessions with visual drop highlighting and window activation focus transfer (`NSApplication.shared.activate(ignoringOtherApps: true)`).
  - Dropping loose RAW/JPEG master negatives opens the Import Sheet pre-populated with target files.
  - Added "Show in Main Library" action in Import Sheet navigating cleanly to Grid mode and focusing the first imported photo.
- **Phase 2.5.2: Dependency Injection via `@Environment(AppViewModel.self)`**:
  - Injected `@Observable` `AppViewModel` once at the scene level in `FirstProofApp.swift` (`WindowGroup` and `Settings`).
  - Completely stripped 25+ view `init(viewModel:)` prop-drilling initializers across all workspace, sidebar, canvas, inspector, and sheet views.
  - Subviews adopt fine-grained `@Environment(AppViewModel.self) private var viewModel` and view-local `@Bindable var viewModel = viewModel`, minimizing SwiftUI redraw scope.
  - Eliminated keyboard shortcut conflict on <kbd>⌘⌥2</kbd> (removed redundant blind 200% zoom button to preserve dedicated <kbd>⌘⌥2</kbd> switching for *Color & Look* tab, while keeping double-click contextual loupe).
- **Phase 2.5.3: Native macOS 14 `.inspector(isPresented:)` Container & `NavigationSplitView`**:
  - Replaced legacy AppKit `HSplitView` in `MainView.swift` with SwiftUI's native `NavigationSplitView` and `.inspector(isPresented:)`.
  - Configured native column boundaries: `.navigationSplitViewColumnWidth(min: 200, ideal: 220, max: 280)` on the leading sidebar and `.inspectorColumnWidth(min: 240, ideal: 270, max: 320)` on the trailing darkroom inspector.
  - Removed hardcoded min/max width frame constraints across sidebar and inspector child views to eliminate layout conflicts.
  - Wired native toolbar inspector toggle button (<kbd>sidebar.trailing</kbd>), `InspectorCommands()`, and standard <kbd>⌘⌥I</kbd> menu toggle for full-bleed proofing on 14" screens.
- **Phase 2.5.4: Swift Structured Concurrency & Cooperative Cancellation**:
  - Annotated `AppViewModel` with `@MainActor` for compile-time verified main-thread UI state isolation and safe `@Observable` state binding.
  - Replaced legacy Grand Central Dispatch callbacks (`DispatchQueue.global(qos:).async`) with structured `Task` and `Task.detached(priority:)` off-actor work units.
  - Implemented cooperative cancellation with `activeDecodeTask?.cancel()` and `Task.isCancelled` guards in `loadPreview(for:)`, cleanly cancelling obsolete Bayer decodes during rapid arrow-key culling.
  - Structured catalog indexing in `openDirectory` via `activeCatalogTask?.cancel()`, ensuring fast folder switching never allows stale background directory scans to overwrite newly selected session photos.
  - Modernized metadata saving debouncing with a cancellable structured `metadataSaveTask` and `Task.sleep(nanoseconds: 400_000_000)`, with flush-on-exit semantics and non-blocking background XMP sidecar persistence.
