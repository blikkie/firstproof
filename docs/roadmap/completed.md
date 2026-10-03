# First Proof Completed Stories Archive

This archive tracks completed stories and features moved from the active backlog.

---

## 1. Core Workflow & Triage
- **Comprehensive Undo/Redo & History Engine**: Coalesced rapid slider scrubbing into atomic undo steps based on drag lifecycle (`beginScrub` / `endScrub`). Value-type `EditSettings` snapshots (~250 bytes). Expandable Inspector `HistorySectionView` allowing instant time-travel.
- **Single-Key Professional Muscle Memory**: Single-key shortcuts (`1`..`5` ratings, `P`/`X`/`U` flags, `C` crop, `Z` zoom, `\` original peek) guarded by first-responder shielding against active text fields.
- **Native macOS Polish**: Regular activation policy (`NSApplication.shared.setActivationPolicy(.regular)`), session persistence of `lastOpenedDirectoryPath`, and distributable `FirstProof.app` bundle via `make bundle`.
- **Portable Catalogs on External Drives (`.firstproof/`)**: Folder indexing on external storage volumes stores catalogs inside `<volume>/.firstproof/catalog.db` so the catalog travels with external media.
- **Cross-Engine Pipeline Parity & Pure Additive Luminance Sharpening**:
  - Replaced legacy multiplicative unsharp mask channel scaling ($k = 1.0 + \Delta / y_c$) with pure additive luminance high-pass filtering ($C' = C + \Delta y$) across Metal (`Detail.metal`), Rust (`pipeline.rs`), and WebGL (`WebGLRenderer.ts`), eliminating all arbitrary divisor floors (`0.05`), clamp ceilings (`3.0`), and piecewise noise gates. Preserves chromaticity ratios $(R - G, B - G)$ with bit-level invariance, permanently eradicating Bayer sensor read-noise hot-pixels ("red stars") in deep shadows.
  - Harmonized stage order across all engines: Tone Recovery strictly precedes Recording Emulsion, and sharpening references are evaluated from homogeneous pre-curve display luminance.
  - Bounded WebGL viewport sharpening sampling offsets by screen-space derivatives ($\max(\text{texelSize}, \text{fwidth}(v\_\text{texCoord}))$), preventing sub-pixel sensor noise aliasing in downsampled viewports while preserving 1:1 optical bite in full-resolution exports.
  - Built automated live WebGL vs CPU divergence testing battery (`ParityHarness.ts`, `ParityModal.ts`, `ReferenceExport.ts`) and upgraded proof exports to native sensor resolution.

---

## 2. Color Science & Camera Profiles
- **Story 5.1: Linear Wide-Gamut Working Space & Pipeline Foundation**: Standardized on Linear Rec.2020 with D65 white point in both Rust and Metal. Baseline matrix conversions: RAW Sensor $\to$ CIE XYZ D50 $\to$ Linear Rec.2020.
- **Story 5.2: AgX Filmic Tone Mapping Engine & Parity Guard**: AgX Inset matrix, hue preservation, filmic desaturation shoulder, and sigmoidal tone curve. Verified with automated `agx_shader_parity` integration tests ($\epsilon < 10^{-6}$).
- **Story 5.3: Adobe DCP Ingestion & Dual-Illuminant Camera Calibration**: Rust DCP parser (`dcp.rs`) extracting matrices and dual-illuminant Planckian interpolation between Illuminant A (2856K) and D65 (6504K).
- **Story 5.4: Camera-Matching Tone Profiles & DCP HueSatMap Engine**: DCP HueSatMap 3D HSV delta tables and manufacturer tone curves. Swappable profiles ("First Proof Neutral (AgX)", "Camera Standard", "Camera Portrait", "Camera Landscape", "Camera Vivid"). Verified with `camera_profiles_parity`.
- **Story 5.5: Secondary 3D LUT Engine (.cube) & Color Nuance**: Evaluated post-tone-mapping in collapsible Fine Tuning disclosure panel. 3D Metal texture with trilinear filtering; SIMD tetrahedral interpolation in Rust CPU export.
- **Story 5.6: Dual-Target Export (Display P3 & sRGB)**: Export color space selector with embedded ICC profiles generated via `lcms2` and injected via `img-parts`.
- **Story 5.7: Authentic DNG ForwardMatrix & Edge-Directed PPG RAW Calibration**:
  - Integrated Adobe DNG §6 ForwardMatrix dual-illuminant Planckian interpolation for Canon, Sony, and Nikon camera families into Linear Rec.2020 wide-gamut working space.
  - Replaced independent-channel cubic demosaicing with edge-directed PPG (Patterned Pixel Grouping) demosaicing in the export pipeline via `correct_blacklevel_cfa` on normalized `f32` data.
  - Aligned highlight reconstruction knee to physical sensor saturation ($0.97$), preserving photopic luminance without false color knees.
  - Fixed DNG baseline exposure tag parsing (Tag 50730).
  - Quantitative verification against camera reference JPEG (`_MG_2332.CR2`): MSE dropped from 299.11 to 225.84 (-24.5%), PSNR rose from 23.37 dB to 24.59 dB, metal handrail delta matched at `[+2, +2, +2]`, and skin melanin G/R chromaticity ratio matched to 0.500 vs 0.504.
  - Built diagnostic calibration suite (`calibrate`, `inspect`) generating 1:1 detail crop and full-frame diff triptychs.
- **Story 5.8: Continuous Sensitometric Per-Channel Tone Curves, Soft-Adaptive Bayer Demosaicing, & Pure CIE Color Pipeline**:
  - Replaced scalar luminance tone curve scaling ($k = f(Y)/Y$) with continuous $C^1$ per-channel Hurter-Driffield spline evaluation across Metal (`Shaders.metal`), Rust CPU (`pipeline.rs`), and WebGL (`FragmentShader.ts`, `ReferenceExport.ts`), naturally self-limiting to $[0.0, 1.0]$ and eliminating tungsten skin highlight blowouts and cyan blotches.
  - Implemented soft-adaptive Hamilton-Adams Bayer demosaicing with smooth Hermite directional blending, eliminating zipper, maze, and checkerboard artifacts along low-gradient skin transitions.
  - Added pre-demosaicing CFA white balance scaling, ensuring cross-channel Laplacian gradients operate in calibrated optical units and preventing asymmetric high-ISO noise amplification.
  - Implemented chromatic difference median filtering on $(R - G)$ and $(B - G)$ difference planes, eliminating false-color chroma speckles while preserving 100% of spatial green luminance ($G$) optical sharpness.
  - Completely eliminated heuristic `applyCameraHueSat` and synthetic 3D HueSatMaps, grounding the color pipeline strictly in physical CIE spectral matrices and sensitometric tone mapping.
  - Quantitative verification against camera reference JPEG (`_MG_2332.CR2`): MSE dropped from 299.11 to 135.65 (-54.6%), PSNR rose from 23.37 dB to 26.81 dB, Mean Total Error dropped to 8.91/255 (3.49%), and full-frame blue parity matched at 94.3 vs 94.2.

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
- **Story 3.1: Curated Fine-Art Print Media Archetypes: Color & Monochrome (P1)**:
  - *Quarantined Emulsion Workspace*: Opt-in darkroom print media emulation housed strictly in the dedicated `Emulsion` tab (`InspectorTab.emulsion`, `Cmd+Opt+5`), defaulting to `None / Clean Digital Display`.
  - *Generic, Non-Trademarked Terminology*: Physical darkroom processes, historical formulations, and generic chemical terminology (*Gallery Chromogenic Type-C (RA-4)*, *Silver Dye-Bleach Classic (Azo Reversal)*, *Archival Fine-Art Cotton Rag (Mineral Pigment)*, *Baryta Silver Gelatin FB*, *Warmtone Chlorobromide (Selenium Toned)*, *Platinum / Palladium*, *Historic Cyanotype*).
  - *Physical Optical Glass Filtration*: Six enlarger spectral filter modes (*Neutral Panchromatic*, *Yellow K2*, *Orange 16*, *Red 25A*, *Green X1*, *Orthochromatic Vintage*) for authentic optical monochromatic conversion in Linear Rec.2020.
  - *Restrained "Rule of Three" Darkroom Controls*: Smooth cubic polynomial S-curve Paper Grade (`-2.0` to `+2.0`, centered at `0`), Multiplicative Paper Base Warmth reflectance trim (`-100` to `+100`), and continuous chemical Process Depth (`0%` to `100%`).
  - *Analytical Sensitometry & Continuity*: Replaced all piecewise knees and tent functions with $C^1/C^\infty$ continuous curves: cubic Hermite soft shoulder for RA-4 highlights, quadratic fiber surface scatter for matte cotton rag shadows, and parabolic harmonic bell curves for selenium midtones. Subtractive Beer-Lambert ferric ferrocyanide absorption for authentic Prussian blue Cyanotype shadows ($D_{\text{max}} \approx 1.55$).
  - *Live History Integration*: Observable `PhotoEditHistory` with `historyRevision` ticker synchronizing darkroom changes into the History panel with dedicated `"film"` icons and full bidirectional `Cmd+Z` undo/redo.
  - *Dual-Engine Parity & Non-Destructive Ingestion*: 100% bit-level parity verified across all archetypes and filter sweeps between Rust CPU export (`print_media.rs`, `encoders.rs`) and Metal GPU shader (`ColorScience.metal`), backed by automated integration tests (`firstproof-core/tests/print_media_parity.rs`). Fully serialized to firstproof-native XMP sidecars (`printMedia`, `enlargerFilter`, `printPaperGrade`, `printBaseWarmth`, `printProcessDepth`).
- **Story 3.2: Upstream Recording Emulsions & Photochemical Color Science (P1)**:
  - *Upstream In-Camera Recording Medium*: Modeled photochemical behavior of the latent image on multi-layer camera negative/reversal and instant diffusion film prior to print enlargement or display mapping.
  - *Zero Fake Grain & Legacy Mock-LUT Purge*: Purged hardcoded procedural pseudo-LUTs (`classic_chrome`, `warm_portrait`, `cine_teal_orange`, `tri_x_bw`, `vintage_fade`) from `lut.rs`, retaining strictly neutral `.cube` parsing. Purged 2D additive noise sliders (`grainAmount`, `grainSize`, `grainRoughness`, `grainChroma`) from Detail panel, restoring it to pure optical acutance. Permanently purged procedural noise algorithms in adherence to "Zero digital plastic", keeping sensor resolution 100% pristine.
  - *Active DIR Chemical Couplers*: Vertical Development Inhibitor Releasing crosstalk matrices ($3 \times 3$ $\mathbf{C}_{\text{DIR}}$) evaluated in linear Rec.2020 density space, generating dynamic color separation, pastel highlight roll-off, and glowing skin tones.
  - *Sensitometric D-log H Curves*: Continuous $C^1$ cubic Hermite characteristic curves derived from published manufacturer data sheets with photographic push/pull processing ($2^{\text{EV}}$ trim).
  - *Curated Geographic Archetypes*: 7 deeply researched chemical archetypes across major historical processes (*Rochester Portrait 400*, *Ashigara Chrome 50*, *Rochester K-14 Chrome 64*, *Rochester Cinema 500*, *Rochester Daylight 200*, *Mobberley Silver 400*, *Cambridge Diffusion 100*).
  - *Dual-Engine Bit-Level Parity*: Closed-form SIMD transform verified across Rust CPU (`emulsion.rs`, `pipeline.rs`) and Metal GPU shader (`ColorScience.metal`, `Shaders.metal`) with automated integration tests (`firstproof-core/tests/emulsion_parity.rs`).
  - *Literature Grounding*: Characteristic curves systematically verified against published manufacturer technical data sheets (Kodak E-4050, Fuji AF3-960E, Kodak E-88, Kodak H-1-5219, Kodak E-7022, Ilford HP5 Plus, and Polaroid Land PNAS); documented in [sensitometric_emulsion_references.md](../architecture/sensitometric_emulsion_references.md).


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
- **Phase 2.5.5: Custom `Layout` Protocol for Aspect-Ratio Containment**:
  - Implemented reusable `SquareAspectLayout: Layout` participating natively in SwiftUI's layout negotiation via `sizeThatFits` and `placeSubviews`.
  - Replaced legacy `Color.clear.aspectRatio(1.0, contentMode: .fit).overlay(GeometryReader ...)` pattern in `ToneCurveView` and `ColorWheelView`.
  - Guaranteed deterministic vertical space reservation inside inspector `ScrollView` and `VStack` containers, eliminating layout jitter and overflow into adjacent disclosure groups during window and sidebar resizing.
  - Updated architectural guidelines in `.agents/AGENTS.md` to establish `SquareAspectLayout` as the standard pattern for square aspect containment.

---

## 7. WebAssembly & WebGL 2 Interactive Darkroom Engine (`firstproof.photo`)
- **WebAssembly Core Engine (`firstproof-wasm`)**:
  - Compiled core color science, sensitometric film stock characteristic curves, print media archetypes, DCP/XMP ingestion, and preview extraction to WebAssembly targeting `--target web`.
  - Implemented pre-allocated 64MB decode scratchpad (`init_scratchpad`) with zero-copy shared memory pointer transfers (`copy_into_scratchpad`, `decode_preview_from_scratchpad`).
  - Strict spec-compliant JPEG stream marker segment parser (`parse_jpeg_stream_at`) handling 2-byte marker lengths, skipping `0xFF 0x00` entropy scan data byte stuffing, and filtering out `SOF3` Lossless JPEG tiles.
  - Recursive TIFF IFD structure parser (`IFD0`, `SubIFDs` `0x014A`, `ExifIFD` `0x8769`) extracting high-res preview JPEGs (`5184 × 3456`) directly from DNG and CR2 negatives.
- **WebGL 2 Sensitometric Shader Pipeline (`WebGLRenderer.ts`)**:
  - GLSL 3.00 ES shader rendering AgX filmic tone mapping (Vivid/Neutral), linear Rec.2020 working space, DCP HueSatMap HSV deltas, high-precision tone curve LUTs, and fine-art print media.
- **Photographic Web Application (`web/src/main.ts`)**:
  - Local browser folder intake via File System Access API (`showDirectoryPicker`) with zero tracking and 100% on-device processing.
  - Dual-wheel Tone Curve editor widget (`ToneCurveWidget.ts`) with cubic spline evaluation.
  - Extended White Balance presets including Sodium Vapor ($2200\text{K}, \text{Tint } +160$) and Auto WB.
  - Interactive White Balance Eyedropper (<kbd>W</kbd>), White Point Picker, and Black Point Picker sampling WebGL canvas pixels.
  - Pure photographic EXIF formatters (`formatAperture`, `formatShutterSpeed`, `formatDimensions`, `formatExposure`).
- **Shared Rust Types & Web Parity Invariant (`firstproof-types`)**:
  - Created shared domain crate `firstproof-types` extracting `RawSensorFrame`, `EditSettings`, `evaluate_curve_lut`, `XmpData`, and `extract_preview_slice` to guarantee 1:1 color science, sensitometric math, and channel mapping parity across native macOS and web darkroom targets without duplicate logic.
  - Verified bit-level parity with integration test suite `core_wasm_parity.rs`.
- **Off-Main-Thread RAW Pipeline & Instant Culling (`RawDecodeWorker.ts`)**:
  - Decoupled 16-bit linear Bayer CFA sensor decoding into dedicated WebWorker (`RawDecodeWorker.ts`) using zero-copy `postMessage` ArrayBuffer transfer, keeping the main UI thread locked at 60 FPS.
  - Implemented lookahead neighbor frame prewarming (`[idx - 1, idx + 1]`) in the background worker with bounded LRU memory eviction and `imgBitmap.close()` cleanup, reducing burst culling navigation latency to < 50ms.
  - Integrated dynamic `wasm.memory.buffer` validation to eliminate detached ArrayBuffer exceptions when allocating scratchpads.
- **Native Engine Intake & Binning Enhancements (`firstproof-core`)**:
  - Backported sub-millisecond TIFF IFD preview extraction into `raw.rs` (`extract_preview_slice`), replacing brute-force multi-megabyte linear byte scans with direct 0.3ms IFD/SubIFD directory lookups.
  - Optimized 2×2 Bayer binning in `decode.rs` with contiguous row slicing and hoisted CFA pattern matching outside inner pixel loops across both NEON SIMD and Rayon parallel paths.
- **macOS Slider Boundaries, Mired Scaling & Conservative Auto Tone (`firstproof-types/auto.rs`, `web/`)**:
  - Extracted conservative auto tone solver (Zone System log-geometric mean, $P_{50}$ median anchoring, $P_{98}$ specular ceiling safeguard, $65\%$ damping) and Gray-World auto white balance into shared `firstproof-types::auto`.
  - Re-exported auto solvers in `firstproof-wasm` and `firstproof-core`, eliminating math duplication.
  - Implemented perceptual Mired non-linear color temperature mapping ($2000\text{K} \to 20000\text{K}$, $50\text{K}/200\text{K}$ stepping) providing uniform visual sensitivity across daylight and tungsten regions.
  - Aligned all develop slider boundaries with macOS desktop app (Exposure $\pm 3.0\,\text{EV}$, Tint $\pm 200$, Warmth $\pm 100$, Process Depth $0..100\%$).
  - Added Detail & Sharpening workspace and <kbd>Cmd+U</kbd> / <kbd>Ctrl+U</kbd> keyboard shortcut for instant auto tone leveling.
- **macOS Export Modal Parity & Full-Resolution RAW CFA Demosaicing (`ExportModal.ts`, `firstproof-types/raw.rs`, `tiffEncoder.ts`)**:
  - Implemented `ExportModal.ts` with 1:1 parity with macOS desktop `ExportSheetView.swift`: JPEG (Quality 1–100 + Ultra HDR ISO 21496-1 toggle) vs 16-bit uncompressed TIFF 6.0, sizing presets (Original, 2048px, 3840px, Custom Dimension), and color spaces (sRGB vs Display P3 with Relative Colorimetric vs Perceptual rendering intents).
  - Resolved Chromium `SecurityError: The directory selected contains system files` by defaulting destination to `~/Downloads (Browser Default)` streaming via browser downloads without requiring privileged directory handle access.
  - Implemented pure Rust edge-directed gradient-sensing CFA demosaicing in `firstproof-types/src/raw.rs` (`to_full_demosaiced_linear_f16`) and exposed through `firstproof-wasm` bridge (`decode_raw_full_linear_from_scratchpad`), allowing exports of RAW files at 1:1 native sensor resolution while preserving 10–15ms 2×2 binned viewport culling.
  - Created zero-dependency TypeScript 16-bit uncompressed TIFF 6.0 binary encoder (`tiffEncoder.ts`) with deterministic little-endian IFD directory packing.
- **Story 3.8.1: Shared Primitives & Rust Source of Truth (`firstproof-types`, `firstproof-wasm`, `WasmBridge.ts`, `exifReader.ts`)**:
  - Moved `ExifOrientationRecord` and `decode_exif_orientation(orientation: u32) -> ExifOrientationRecord` into shared domain crate `firstproof-types`, eliminating diverging rotation/flip logic across platforms. Re-exported via `firstproof-core` (UniFFI) and `firstproof-wasm` (WASM).
  - Aligned baseline default temperature in `EditSettings::default()` from `5000.0` to `5500.0` (Daylight D55 neutral), matching Swift `AppViewModel`, Metal shaders, Rust `pipeline.rs`, and WebGL shaders.
  - Eliminated duplicate 80-line manual TIFF/JPEG IFD offset parsing in `web/src/utils/exifReader.ts` in favor of shared WASM `extract_metadata_from_bytes`, supporting sub-0.1ms orientation extraction directly on 64KB byte slices.
  - Derived clean TypeScript definitions directly from Rust types via `#[wasm_bindgen(typescript_custom_section)]` in `firstproof-wasm` and refactored `WasmBridge.ts` to re-export canonical Rust schemas (`ExifOrientationRecord`, `ImageMetadataRecord`, `EditSettings`, `ToneCurvesRecord`, `AllCurvesLutRecord`, `XmpData`, and enum types).
  - Replaced handwritten switch statement in `web/src/main.ts` with delegation to shared WASM `decode_exif_orientation`.

---

## 3. Sensitometric Manifolds & Artifact Elimination
- **Story 5.9: Monotonic Highlight & Shadow Recovery**:
  - Replaced bounded half-sine lobes with smooth monotonic $C^1$ cubic Hermite shoulders across Rust (`pipeline.rs`), Metal (`ColorScience.metal`), WebGL (`ColorScience.ts`), and TypeScript (`ReferenceExport.ts`).
  - Ensures extreme highlight recovery above $+4\text{ EV}$ smoothly plateaus at maximum compression rather than rebounding to zero.
- **Story 5.10: Continuous Melanin Ellipse for Skin-Safe Vibrance**:
  - Replaced binary `(r > g) && (g > b)` check with a continuous 2D Gaussian distance metric around the human melanin locus $(r_n, g_n) = (0.44, 0.32)$ with a smooth Hermite shadow fade ($0.005 \to 0.030$), eliminating threshold solarization on shadow skin.
- **Story 5.11: Smooth Selective Color Response (Deep Blue & Foliage)**:
  - Symmetrized Deep Blue and Foliage manifolds across positive and negative slider ranges without slope discontinuities.
- **Story 5.12: Unified Analytical Characteristic Curves for Emulsions & Print Media**:
  - Replaced piecewise zone branching (`base > 0.65`, `c > 0.70`, `afterShoulder < 0.35`) with unified continuous analytical formulas in `emulsion.rs`, `print_media.rs`, `ColorScience.metal`, and `ColorScience.ts`.
- **Story 5.13: Continuous Circular Skin Hue & Midtone Saturation Manifold for AgX Vivid**:
  - Diagnosed and resolved the blue blowout defect on cheek transitions under `AgxVivid`:
    1. *Circular Shortest-Arc Hue Distance*: Replaced linear hue distance with circular shortest-arc distance $\min(|h - 22^\circ|, 360^\circ - |h - 22^\circ|)$ with $36^\circ$ radius, ensuring warm rose and flushed infant skin ($345^\circ - 360^\circ$) receives full continuous skin protection.
    2. *Noise-Safe Midtone Chroma Envelope*: Strictly confined saturation boost to well-lit midtones ($V \in [0.15, 0.85]$) via smooth $C^1$ Hermite ramp-in and roll-off, permanently eradicating blue/purple noise amplification and shadow contour clipping in dark cheeks.
    3. *Authentic AgX Contrast Baseline*: Aligned AgX base contrast to authentic Sobotka curve without arbitrary 1.15x multiplier, preserving shadow readability and green midtone tonality ($G/R$ cheek ratio matching camera preview at $0.496$ vs $0.500$).
    4. *Full Demosaicing Parity in `firstproof-types`*: Upgraded `firstproof-types/src/raw.rs` (`to_full_demosaiced_linear_f16`) with pre-CFA white balance scaling, soft-adaptive directional demosaicing, and 3x3 chromatic median filtering, eliminating demosaicing desync across WASM and native export engines.
- **Story 3.8.1: Dual-Engine Color Science Parity (Print Media & Mobberley B&W Filtration)**:
  - *Print Media Stage Harmonization*: Moved `applyPrintMedia` in `Shaders.metal` and `FragmentShader.ts` from scene-referred Rec.2020 (before tone mapping) to execute on `srgbLinearOut` (immediately after AgX/camera tone curve mapping, prior to display gamma/EDR). Added `apply_print_media` to `pipeline.rs` at the identical post-tonemapping stage and purged the redundant second call in `encoders.rs`.
  - *Mobberley Silver 400 Lens Filtration*: Updated `apply_recording_emulsion` in `firstproof-core/src/color/emulsion.rs` to accept `filter_mode: EnlargerFilterMode` and evaluate `filter_mode.spectral_weights()`, achieving 100% B&W filtration parity between Rust export and GPU viewport shaders across all filter archetypes (Yellow K2, Orange 16, Red 25A, Green X1, Orthochromatic, and Neutral Panchromatic `0.2627R + 0.6780G + 0.0593B`).
  - *Dual-Engine Verification*: Automated test `test_synthetic_macbeth_and_neutral_ramp_dual_engine_parity` in `dual_engine_synthetic_parity.rs` expanded to audit print media modes (Fine Art Cotton Rag, Baryta Silver Gelatin, Cyanotype) and Mobberley filtration across both simulated GPU shader and Rust CPU pipeline with $\Delta E_{00} < 0.15$ on individual patches and mean $\Delta E_{00} < 0.05$.
- **Story 3.8.5: De-Slopping Parity Harness & Deleting the TypeScript Oracle (`firstproof-types`, `firstproof-wasm`, `ParityHarness.ts`, `ReferenceExport.ts`)**:
  - *Deleted 1,019-Line TypeScript Twin*: Completely purged `web/src/parity/ReferenceExport.ts` (1,019 lines of hand-rolled TypeScript duplicating AgX filmic, Fritsch-Carlson cubic splines, Bilateral Tone Recovery, and print media), eradicating the dual-source-of-truth divergence risk.
  - *Extracted Canonical Rust Color Science & Reference Pipeline into `firstproof-types`*: Moved pure sensitometric math (`agx.rs`, `camera_curves.rs`, `emulsion.rs`, `print_media.rs`, `pipeline.rs`) into `firstproof-types::color`, and re-exported it directly in `firstproof-core`, eliminating ~1,200 lines of duplicate Rust code.
  - *WASM Reference Frame & Pixel Evaluators*: Exposed `evaluate_reference_frame`, `evaluate_reference_pixel`, and `evaluate_display_luma` through `firstproof-wasm` with high-performance vectorization across image buffers.
  - *Parity Harness Upgraded to Single Source of Truth*: Re-wired `ParityHarness.ts` and `web/parity.html` to audit WebGL shaders bit-for-bit against compiled Rust WebAssembly.
  - *Root-Cause Fixes*: Purged the artificial "fake neutral bypass" in the Rust reference engine that skipped tone mapping on default inputs, aligned print media stage placement post-tone-mapping in display linear space, and harmonized photopic luminance coefficients to Rec.2020 (`0.2627, 0.6780, 0.0593`).
  - *Verification*: 100% green pass across all 19 test scenarios in `web/parity.html` ($\text{maxChannelDelta} \le 1/255$ on adjustments, $\le 2/255$ overall, PSNR $> 57\,\text{dB}$), plus 100% pass across all native Rust test suites (`dual_engine_synthetic_parity`, `agx_shader_parity`, `camera_profiles_parity`).
- **Story 3.8.6: Client-Side Export Clean-Up & Eliminating Fake 16-Bit Slop (`web/src/utils/tiffEncoder.ts`, `web/src/ExportModal.ts`, `web/src/main.ts`, `web/index.html`)**:
  - *Eliminated Fake Bitshifted 16-Bit Slop*: Replaced the deceptive `(r << 8) | r` bitshifting in `tiffEncoder.ts` with an authentic, standards-compliant uncompressed 8-bit RGB TIFF 6.0 encoder (`BitsPerSample = [8, 8, 8]`). Halved disk file footprint with honest archival representations.
  - *Purged Unimplemented UI Promises*: Removed the non-functional "Ultra HDR (ISO 21496-1)" checkbox (gain map synthesis belongs in native engines, not browser mockups) and the non-functional "Rendering Intent" select box.
  - *Calibrated sRGB Working Scope*: Scoped web exports strictly to calibrated sRGB (Standard Web & Print Default), eliminating uncalibrated Display P3 canvas blitting.
  - *Decoupled Off-Screen Export Pipeline*: Implemented a dedicated off-screen `WebGLRenderer` and canvas instance (`getExportRenderer()`). All export processing (texture decoding at full sensor CFA resolution, geometry transforms, curve LUTs, and rendering) occurs completely off-screen, ensuring the live `#darkroom-canvas` viewport is never resized, flashed, or clobbered during batch exports.
  - *Verification*: Single and batch exports confirmed in-browser without viewport resizing; 8-bit TIFF and sRGB JPEG files verified to open and render cleanly across viewers.
- **Story 3.8.2: Shader Geometry & Viewport Transformation Parity (`web/src/shaders/FragmentShader.ts`, `web/src/WebGLRenderer.ts`)**:
  - *Ported `transformGeometry()` to GLSL*: Ported `transformGeometry(vec2 uv)` 1:1 from `FirstProofApp/Sources/Shaders/Geometry.metal` into WebGL 2 GLSL. Implemented base orientation (0°, 90°, 180°, 270° rotation and horizontal/vertical flips), normal crop bounding box expansion (`u_cropModeActive == 0`), center-anchored straighten rotation with auto-crop boundary constraints (`u_cropConstrain != 0`), backward projective perspective homography, and out-of-bounds letterbox discard returning `vec4(0.08, 0.08, 0.08, 1.0)`.
  - *Permanently Eliminated CPU-Side Vertex Buffer Mutation*: Deleted `updateQuadGeometry` and CPU dynamic buffer allocations (`gl.bufferSubData`). Initialized the VBO quad buffer once with `gl.STATIC_DRAW`, projecting all rotations, flips, straighten angles, and crops strictly on the GPU (Core Invariant 1: GPU-Accelerated Coordinate Projection).
  - *Intermediate Texture Bypass & Idempotence*: Implemented fast-path bypass for pre-rendered intermediate textures (`u_isIntermediate == 1`), returning `texture(u_texture, v_texCoord)` at line 0 to bypass all fragment color and geometry transforms without double-rendering (Core Invariant 3: Intermediate Texture Bypass & Idempotence).
  - *Optimized Sharpening Sampling & Redundant ALU Elimination*: Updated `sampleDisplayLuma` to sample projected `uv` texture coordinates. Evaluated center tap `y_c` directly from homogeneous pre-curve linear Rec.2020 color (`rec2020PreCurve`) via `computeDisplayLuma`, eliminating redundant full-pipeline evaluations for the center pixel.
  - *Aspect-Ratio Containment Parity*: Harmonized `computeFitScale` in `WebGLRenderer.ts` with `MetalRenderer.swift` to properly compute display dimensions for orthogonal rotation and cropped aspect ratios.
  - *Verification*: Confirmed in-browser 100% green pass in `web/parity.html` ($\text{maxChannelDelta} \le 1/255$, PSNR $> 57\,\text{dB}$), smooth crop expansion/fill upon exiting crop mode, 90°/180°/270° rotation, and zero-flash intermediate texture bypass.
- **Story 3.8.3: Crop Controller & Desktop Interaction Parity (`web/src/CropController.ts`, `web/src/main.ts`, `web/src/style.css`, `web/index.html`)**:
  - *Stationary Full-Canvas Gesture Container*: Refactored `CropController.ts` from an image-rect-bounded element into a full-viewport stationary gesture surface (`#crop-gesture-surface` with `inset: 0`). The SVG scrim cut-out (`#crop-mask-cutout`) now dims both the uncropped photo and letterbox margins, matching macOS `CropOverlayView.swift` and eliminating bright un-scrimmed canvas bars.
  - *Corner Rotation Affordances & Dual-Stroke Vector Cursor*: Replaced arbitrary "click anywhere to rotate" interaction with intuitive corner rotation zones (24px to 60px from crop vertices). Replaced the misleading `crosshair` cursor with a bespoke dual-stroke curved rotation SVG cursor with black drop shadows and crisp white stroke, ensuring high contrast on all photo and canvas tones.
  - *Dedicated Toolbar Straighten Control*: Added a dedicated interactive Straighten slider (`-45°` to `+45°`) with numeric readout and double-click-to-reset directly in the floating bottom HUD (`#crop-toolbar`), providing immediate discoverability alongside the existing polar angle drag HUD pill (`#straighten-hud`).
  - *Clean Reset Parity*: Updated `resetCrop()` to reset both crop bounds (`[0, 0, 1, 1]`) and `straightenAngle = 0.0`. Resets inspector and toolbar sliders to `0.0°` and clears `edits.crop_angle`, immediately re-rendering the full uncropped and unrotated photo without stale angle retention.
  - *Aspect Ratio Dropdown Styling*: Fixed uninitialized CSS variables causing white-on-white text in Chromium and Firefox `<select>` popups by explicitly styling `.crop-aspect-select` and `.crop-aspect-select option` with dark background `#1a1d24` and crisp text `#f1f5f9`.
  - *Verification*: Verified in-browser across Chrome and Firefox; confirmed corner drag rotation, floating HUD pill readout, toolbar slider synchronization, double-click reset, aspect ratio dropdown legibility, and full unrotated reset restoration.
- **Story 3.8.4: Zero-Flashing Preview & Transitional Staging Parity (`web/src/main.ts`, `web/src/WebGLRenderer.ts`, `web/src/IntermediateTextureCache.ts`, `web/src/shaders/FragmentShader.ts`, `web/index.html`)**:
  - *Stage 0 Up-Front Edit & Orientation Staging*: Resolved baseline orientation and edits from memory cache or XMP sidecars synchronously prior to texture updates (`photoEditsCache`, `photoExifCache`, `syncSliderUI`, `syncRatingFlagUI`). Completely eliminated stale slider and rotation flashing across photo selection.
  - *Atomic Dark Canvas Transitions*: Introduced `clearCanvas()` on `WebGLRenderer` to clear the viewport immediately to neutral dark (`#111318`) with the developing spinner when switching to an un-cached photo, strictly preventing stale pixels from lingering during the 10–15ms async decode window.
  - *High-Resolution Intermediate Texture Cache*: Created `IntermediateTextureCache` storing up to 20 GPU-backed `ImageBitmap` snapshots keyed by filename and deterministic edit fingerprint hash. Navigating back to previously visited photos renders in 0ms directly from memory.
  - *Intermediate Bypass & Idempotence Invariant*: Implemented identity geometry handling (`rotation: 0`, `crop: [0,0,1,1]`) and fast-path fragment bypass (`u_isIntermediate = 1`) in `WebGLRenderer` and `FragmentShader.ts`, ensuring intermediate textures never pass through AgX or geometric transforms twice.
  - *Direct 16-Bit Linear Sensor Frame Streaming*: Streamed decoded linear CFA frames directly into `uploadLinearRec2020Texture`, bypassing redundant main-thread JPEG decoding entirely for RAW files.
  - *Boyscout Clean-Up (Camera JPEG Peek Removal)*: Purged the temporary debug "Camera JPEG (J)" peek button, shortcut, `TEXTURE3` binding, and fragment shader branch, streamlining the render pipeline.
  - *Verification*: Confirmed in-browser burst culling with zero stale-pixel flashes, 0ms return navigation from intermediate cache, and clean removal of the `J` key shortcut.
- **Story 3.10: Architecture Clean-Up: Rename `firstproof-types` to `firstproof-engine` & Core De-Duplication (`firstproof-engine`, `firstproof-core`, `firstproof-wasm`)**:
  - *Architectural Crate Reclassification & Rename*: Renamed `firstproof-types` to `firstproof-engine`, formally establishing the 3-tier architecture:
    1. `firstproof-engine`: Pure algorithmic domain kernel (color science, AgX, sensitometric tone curves, CFA demosaicing, auto-algorithms, XMP sidecars, parameter schema). Zero native OS or C dependencies; compiles everywhere (macOS, Linux, WASM, embedded).
    2. `firstproof-core`: Native host adapter (SQLite catalog, LibRaw/Rawler disk I/O, LCMS2 ICC profiles, Rayon batch export, UniFFI Swift bridge).
    3. `firstproof-wasm`: WebAssembly host adapter (memory scratchpad, `wasm-bindgen`, JS array buffer marshalling).
  - *RAW Sensor Decoding Consolidation*: Consolidated `RawSensorFrame::decode_from_raw_image` and `RawSensorFrame::decode_from_bytes` in `firstproof-engine/src/raw.rs` using 4-channel black/white level normalization and DNG baseline exposure parsing, eliminating duplicated decoding logic in `firstproof-core/src/raw_decoder.rs` and `firstproof-wasm/src/lib.rs`.
  - *Canonical XMP Engine Consolidation*: Consolidated the XMP engine in `firstproof-engine/src/xmp.rs` with comprehensive Adobe Camera Raw (`crs:`), Photoshop (`Headline`/`By-line`), and Lightroom (`hierarchicalSubject`) fallbacks, eliminating duplicate files `data.rs`, `reader.rs`, `writer.rs` in `firstproof-core/src/xmp/`.
  - *Metadata Struct Consolidation*: Consolidated `ImageMetadataRecord` in `firstproof-engine/src/preview.rs` to include GPS coordinates (`latitude`, `longitude`), `description`, `creator`, and `copyright`, eliminating duplicate `struct ImageMetadata` and EXIF parsing in `firstproof-core/src/raw.rs`.
  - *Parameter Schema & Mired Bridge*: Re-exported `AdjustmentDescriptor`, `all_adjustments()`, `convert_kelvin_to_mired_norm()`, and `convert_mired_norm_to_kelvin()` from `firstproof-engine` through `firstproof-core` via UniFFI.
  - *Verification*: 100% test pass across all crates (15 tests in `firstproof-engine`, 59 unit + 10 integration test suites in `firstproof-core`, 6 tests in `firstproof-wasm`, clean `wasm-pack build`, clean `npm run build` in `web/`).
- **Story 3.8.7: Deconstruct the `main.ts` Monolith (2,660 Lines to 978 Lines)**:
  - *Extracted Dedicated Controllers & Services*:
    1. `KeyboardShortcutController.ts` (197 lines): First-responder shielded desktop shortcuts (ratings `1`..`5`, flags `P`/`X`/`U`, crop `C`, zoom `Z`, peek `\`, reset, export, undo/redo).
    2. `CanvasViewportController.ts` (139 lines): Viewport pointer tracking, zoom, pan, hover hit-testing, and pinch-to-zoom scoping so zoom gestures only fire when hovering over the photo canvas.
    3. `InspectorSyncController.ts` (451 lines): Develop sidebar DOM elements, tab navigation, slider bindings, curve listeners, and drag scrub state machine (`beginScrub` / `endScrub` undo integration).
    4. `FilmstripController.ts` (488 lines): DOM card rendering, viewport-driven lazy thumbnail hydration using `IntersectionObserver` (`rootMargin: '300px'`) and throttled concurrency (`MAX_CONCURRENT_THUMBNAILS = 3`), multi-selection (single, Shift-range, Cmd/Ctrl-toggle), and step navigation.
    5. `PhotoLoaderService.ts` (645 lines): Off-main-thread Web Worker RAW decoding pool with direct WASM fallback, 4-item LRU prewarming cache with explicit `ImageBitmap` disposal, unified `PhotoRecord` state management, and touch-free XMP sidecar auto-save.
    6. Domain Helpers: `utils/edits.ts` (default edit factories and modification detection) and `utils/metadataHeader.ts` (EXIF telemetry DOM updates).
  - *Developer Mode Quarantine*:
    - Gated internal parity diagnostics (`#btn-parity-audit`, `ParityModal`) behind developer mode query parameters (`?dev=1`, `?debug=1`, or `import.meta.env.DEV`), ensuring the default UI remains uncluttered and focused on photographic craft.
  - *Root-Cause Bug Resolutions*:
    - *Reset Resurrection*: Ensured `autoSaveSidecar` persists reset default edits if an XMP sidecar already exists on disk, and eliminated redundant delayed sidecar re-reads in photo selection.
    - *Rating/Flag Leakage*: Unified per-photo state inside `PhotoRecord` so selecting a new photo strictly reads and binds that photo's own rating and flag state, preventing cross-photo bleed.
  - *Verification*: Monolith reduced from 2,660 lines down to 978 lines (a 63% reduction); full browser verification of all shortcuts, pinch/pan zoom, lazy thumbnail hydration, sidecar reset persistence, rating/flag isolation, and clean `npm run build`.
- **Story 3.9: Unified RAW Container Intake & Camera Calibration Registry Engine (`firstproof-engine::container`, `firstproof-engine::calibration`, `firstproof-core/tests/raw_vs_dng_intake_parity.rs`)**:
  - *Single-Pass `RawContainerManifest` TIFF/IFD Parser*: Safe, zero-allocation-friendly parser operating directly on byte slices across root IFDs, SubIFDs, and ExifIFDs. Strongly types standard DNG specification tags: `BaselineExposure` (Tag 50730), `AsShotNeutral` (Tag 50728), `AsShotWhiteXY` (Tag 50729), `BaselineNoise` (Tag 50731), `BaselineSharpness` (Tag 50732), `LinearResponseLimit` (Tag 50734), `ColorMatrix1/2` (Tags 50721/50722), `ForwardMatrix1/2` (Tags 50964/50965), `CalibrationIlluminant1/2` (Tags 50778/50779), `AnalogBalance` (Tag 50727), `ActiveArea` (Tag 50829), and 4-channel `BlackLevel` & `WhiteLevel` (Tags 50714/50717).
  - *Authoritative `CameraCalibrationRegistry`*: Built a structured, compile-time in-memory database of camera calibrations for Canon EOS, Sony Alpha, Nikon Z/D, Fujifilm X, Panasonic Lumix, and Leica with principled normalized identifier lookups and family fallbacks. Replaced all fragile ad-hoc string heuristics (`if m.contains("canon")`).
  - *Uncovered and Fixed Legacy Canon +0.831 EV Bug*: Discovered that the legacy binary scanner mistakenly parsed Tag 50728 (`AsShotNeutral`) as `BaselineExposure`, reading the red rational (`0.831169`) and erroneously boosting Canon baseline exposure by +0.831 EV. Corrected Canon baseline exposure to 0.0 EV per official Adobe DNG specification Tag 50730.
  - *Per-Channel Black & White Normalization*: Fixed single-channel `black_levels[0]` shortcuts across `sample_normalized`, `to_binned_linear_f16`, and `to_full_demosaiced_linear_f16`, mapping exact photosite CFA coordinates `(y % 2) * 2 + (x % 2)` to eliminate shadow tint pedestals and green split.
  - *Dual-Format RAW vs DNG Regression Suite*: Added `firstproof-core/tests/raw_vs_dng_intake_parity.rs` testing paired Canon CR2 and DNG files (`_MG_2460.CR2` vs `_MG_2460.dng`). Verified 1:1 agreement: baseline exposure matches to 0.0000 EV, identical active sensor crop ($5184 \times 3456$), identical CFA layout (`Gbrg`), identical white balance ratios ($R \approx 2.138, B \approx 1.476$), and midtone wedge luminance divergence $|\Delta\text{EV}| < 0.06\text{ EV}$.
- **Story 3.13: Sensor Read-Noise Mitigation & Viewport Chroma Parity (`firstproof-engine::raw`, `firstproof-core::export::decode`, `sensor_noise_mitigation_parity.rs`)**:
  - *Extracted DNG `NoiseProfile` (Tag 51041)*: Parsed standard 2-parameter Poisson-Gaussian noise model ($\sigma^2(x) = S \cdot x + O$) in `RawContainerManifest` into `RawSensorFrame.noise_profile`. Expanded raw decoder header read buffer from 64KB to 256KB to capture extended IFDs without truncation.
  - *Principled Pre-WB Outlier Suppression (`suppress_sensor_read_noise_outliers`)*: Evaluates isolated single-photosite readout noise spikes on the raw CFA grid before digital white balance multipliers are evaluated. Evaluates 4 nearest like-colored neighbors (diagonals for Green, distance-2 cardinals for Red and Blue) and clamps outliers exceeding the local null hypothesis threshold ($n_{\max} + 3.5\sigma(n_{\max})$). Confirmed 100% preservation of optical edges ($0.000\%$ attenuation).
  - *Viewport 2×2 Box-Binning Chroma Parity*: Resolved the $18\times$ noise variance disparity between Green and Red in 2×2 box-binned viewports (caused by averaging two Green photosites while sampling Red unaveraged under $3\times$ warm WB gain) by integrating a 2-pass chromatic difference median filter on $(R - G)$ and $(B - G)$ into `to_binned_linear_f16`, leaving optical luminance $G$ completely untouched ($G' = G$).
  - *Desktop & WASM Engine Deduplication*: Replaced ~360 lines of duplicated binned decode code in `firstproof-core/src/export/decode.rs` (`decode_raw_preview_binned_buffer`) with a direct call to `frame.to_binned_linear_f16()`, guaranteeing bit-level perceptual parity between native macOS, WebAssembly darkroom, and export pipelines.
  - *Automated Regression Battery (`sensor_noise_mitigation_parity.rs`)*: Verified deep shadow chromatic standard deviation $\text{std}(R - G) < 0.020$ (measured $0.01457$) and maximum isolated chromatic spike $< 0.040$ across 55,000 pixels on real ISO 1600 Canon 60D RAW at extreme 9450K temperature.
- **Story 4.6.1: Harness Engineering — Unified Pre-Flight Sensor & Linter Suite (`scripts/check.sh`, `make check`, `web/biome.json`, `.agents/hooks.json`)**:
  - *Adopted Harness Engineering Principle (Martin Fowler & Birgitta Böckeler)*: Codified "Agent = Model + Harness" and the Computational Steering Loop into `.agents/AGENTS.md`. Regressions and quality gates are enforced by deterministic computational feedback sensors rather than inferential prompt text.
  - *Unified Pre-Flight Sensor Command (`scripts/check.sh` / `make check`)*: Orchestrates all testing and verification steps across Rust, WebAssembly, and TypeScript into a single deterministic script running in under 45 seconds:
    1. Rust workspace tests (`cargo test --workspace --quiet`).
    2. Workspace-wide Rust Clippy (`cargo clippy --workspace --all-targets -- -D warnings`).
    3. Web TypeScript typecheck (`npm --prefix web run typecheck`).
    4. Web Biome linter (`npm --prefix web run lint`).
    5. WebAssembly package build and Node integration test (`wasm-pack build` and `test_wasm.js`).
    6. Web production asset compilation (`npm --prefix web run build`).
    7. Darwin-conditional Swift/Metal gate (executes on macOS; gracefully skips on Linux hosts).
  - *Workspace-Wide Rust Clippy Discipline*: Activated workspace-level Clippy configuration (`[workspace.lints.clippy]` in root `Cargo.toml`), resolved unwrap anti-patterns, fixed missing methods, and tuned allowances for low-level image processing math.
  - *TypeScript Hygiene & Biome Linter Suite*: Adopted Biome (`@biomejs/biome`) with `web/biome.json` (2-space indent, single quotes, excluding `dist`). Enabled `"noUnusedLocals": true` and `"noUnusedParameters": true` in `tsconfig.json`. Fixed all unused local variables across `web/src`.
  - *Token Defense Lifecycle Hook (`.agents/hooks.json` & `.agents/hooks/pre_tool_view_file.py`)*: Codified the "Lifecycle Hooks vs. Prompt Rules Decision Framework" into `AGENTS.md`. Implemented a `PreToolUse` hook on `view_file` that structurally blocks reads of massive generated files (`firstproof_core.swift`, 4,400+ lines), physically preventing catastrophic token waste out-of-band.
  - *Verification*: Executed `make check` and verified all 7 stages pass cleanly end-to-end; verified token defense hook blocks unauthorized reads with educational redirect.
- **Story 4.6.2: Automated Architecture Fitness Functions (`firstproof-core/tests/architecture_fitness.rs`)**:
  - *Automated Architecture Fitness Functions (Martin Fowler)*: Codified automated, deterministic fitness tests defending structural boundaries across all three darkroom execution targets (Metal GPU, WebGL GPU, and Rust CPU).
  - *Engine Dependency Purity Sensor (`test_engine_dependency_purity`)*: Formulated a structural assertion that `firstproof-engine` maintains zero platform OS, windowing, or C-bound library dependencies (`rusqlite`, `sqlite`, `lcms2`, `libraw-sys`, `web-sys`, `js-sys`, `wasm-bindgen`, `metal`, `cocoa`, `objc`, `winit`, `tokio`, `rayon`). Engine remains a 100% pure computational kernel compiling to both native and WASM. Proved failure mode with synthetic malformed dependencies.
  - *Terminal Display Transform Invariant Sensor (`test_terminal_display_transform_ordering_*`)*: Analyzes shader and export pipeline sources to guarantee that tone-mapping (`agx_tone_map`, `AgxFilmic`, `applyCameraToneCurve`) and gamma encoding (`linearToSrgbGamma3`, Display P3 transform) execute strictly as the terminal stage after all internal optical adjustments (White Balance, Exposure, Bilateral Tone Recovery, Emulsion, Tone Curves, Color Vibrance, Split Toning, Detail Sharpening). Validated across `Shaders.metal`, `FragmentShader.ts`, and `pipeline.rs`. Proved failure mode with inverted stage ordering.
  - *Rec.2020 Sensitometric Luminance Coefficient Sensor (`test_rec2020_luminance_coefficient_purity`)*: Verifies that wide-gamut photopic luminance evaluations across Metal, WebGL, and Rust strictly evaluate ITU-R BT.2020 coefficients $[0.2627, 0.6780, 0.0593]$, preventing accidental Rec.709 regressions.
  - *Verification*: 7 passed fitness tests running in 0.00s; unified suite `make check` passes all 7 stages end-to-end in <35s.
