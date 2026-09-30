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
  - *Literature Grounding*: Characteristic curves systematically verified against published manufacturer technical data sheets (Kodak E-4050, Fuji AF3-960E, Kodak E-88, Kodak H-1-5219, Kodak E-7022, Ilford HP5 Plus, and Polaroid Land PNAS); documented in [sensitometric_emulsion_references.md](file:///Users/blikkie/src/LumenBox/docs/architecture/sensitometric_emulsion_references.md).


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

