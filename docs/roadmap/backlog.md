# First Proof Active Backlog & Roadmap

This document strictly tracks pending and deferred features calibrated against our core philosophy: **"Pure capture. Honest color. Zero digital plastic. Print & web first."**

> **Note**: For completed stories, see [completed.md](completed.md). For color pipeline specifications, see [color_pipeline.md](../architecture/color_pipeline.md). For web architecture, see [web_architecture.md](../architecture/web_architecture.md).

---

## 🚀 Immediate Next Up
1. **Story 3.8.2: Shader Geometry & Viewport Transformation Parity** (P0)
2. **Story 3.8.3: Crop Controller & Desktop Interaction Parity** (P0)
3. **Story 3.8.4: Zero-Flashing Preview & Transitional Staging Parity** (P0)
4. **Story 3.10: Architecture Clean-Up: Rename `firstproof-types` to `firstproof-engine` & Core De-Duplication** (P1)
5. **Story 3.8.7: Deconstruct the `main.ts` Monolith (2,586 Lines)** (P1)
6. **Story 3.9: Unified RAW Container Intake & Camera Calibration Registry Engine** (P1)
7. **Story 4.2 & 4.5: Path Sanitization, Test Hygiene & Parameter Schema Bridge** (P2)


> **Environment Note**: Native macOS desktop binary packaging (Xcode, Swift packages, UniFFI `.xcframework`, macOS code signing, and notarization) is decoupled into a dedicated initiative (**Phase 4.4**) on hold while working on Linux. Current active development focuses on the WebAssembly web darkroom and cross-platform headless Linux engine.

---

## Phase 3: Fine-Art Darkroom Chemistry, Archival Formats & Web Playground (P1 / P2)


> **Product Invariant: The Primary Engine vs. The Fine-Art "Treat"**  
> First Proof's primary workhorse default is **pure optical RAW capture via AgX filmic tone mapping**. The core workspaces (**Light**, **Color**, **Detail**, **Optics**) are dedicated strictly to sensor reality: standard White Balance (Temperature & Tint), exposure, tone recovery, and sensor sharpness. Photos look 90% finished immediately without any analog styling.  
> 
> Film stock and print media emulation are **not** the main workflow and **not** a complex simulator. They are an exquisite, 100% opt-in **fine-art treat and differentiator** housed in a dedicated **Emulsion** workspace tab (`InspectorTab.emulsion`). Default state is always `None / Bypass`. When activated, the engine does all the heavy lifting under the hood so the photographer simply selects a curated stock or print medium without fiddling with chemical dials.
>
> **Roadmap Reconsideration: Physical Film Grain in Wide-Gamut Linear Space**  
> Previous attempts at simulating film grain recoiled because grain was evaluated post-tone-mapping in narrow gamma-encoded sRGB, artificially amplifying shadow noise floors and clipping non-linearly. In real physical film, silver halide crystals and dye clouds react to incident scene photons in the optical domain before optical tone-mapping and print transfer. When re-evaluating grain down the roadmap, it must strictly execute inside the Wide-Gamut Linear Rec.2020 domain using physical sensitometric optical density ($D = \log_{10}(1/T)$) prior to the terminal display transform.

### 3.3 Archival Open DNG Converter & Lossless Compression Engine (P2)
- **Lossless Bayer CFA Transcoding**:
  - Convert proprietary camera RAWs (`.cr2`, `.nef`, `.arw`, `.rw2`) into open standard Adobe DNG (1.4+ / TIFF-EP).
  - Encapsulate raw Bayer mosaic unaltered, preserving exact sensor CFA layout, active sensor crop, black/white saturation levels, and daylight/tungsten color calibration matrices.
- **Deflate (ZIP) Tile Compression**:
  - Apply lossless Deflate / ZIP tile compression (`flate2`) across raw sensor data, reclaiming 20–35% disk storage with mathematically identical (zero-loss) reconstruction.
- **Bit-for-Bit Verification Safeguard**:
  - Validate decompressed CFA sensor payload against the original proprietary RAW via SHA-256 / CRC32 before committing conversion, ensuring zero risk of archive corruption.
- **Metadata & Sidecar Integration**:
  - Embed existing First Proof XMP edits, color adjustments, IPTC tags, and ratings into the DNG IFDs, or preserve adjacent `.xmp` according to catalog preferences.

### 3.4 ColorSync Soft Proofing (P2)
- **Canvas Soft Proofing Mode**:
  - Canvas toolbar mode simulating sRGB and standard print gamut clipping (e.g., ISO Coated v2 / US Web Coated SWOP) on wide-gamut displays.
  - Gamut warning overlay highlighting out-of-gamut pixels in high-contrast overlay color.

### 3.5 Archival High-Precision Multi-Pass Export Engine (P2)
- **Decoupled Quality Controls in Export Sheet**:
  - Cleanly separate general optical RAW demosaicing precision from spatial chemical edge diffusion in `ExportSheetView.swift`:
    1. *RAW Demosaicing Precision*: `Standard (PPG Demosaic)` vs. `Archival (Edge-Adaptive AMaZE / RCD with Chromatic Median Filtering)`.
    2. *Spatial Chemical Edge Diffusion (Mackie Lines)*: Dedicated toggle when an emulsion is active.
- **Control 1: Multi-Pass Edge-Directed Demosaicing & Chromatic Median Filtering**:
  - *Standard (Soft-Adaptive Hamilton-Adams Demosaic)*: Integrated in Story 5.8 with smooth Hermite gradient blending and 2 passes of chromatic difference median filtering ($(R - G)$ and $(B - G)$), eliminating checkerboard, zipper, and maze artifacts while preserving 100% of spatial green luminance sharpness ($G$).
  - *Archival (Multi-pass AMaZE / RCD)*: Multi-pass iterative demosaicing for extreme fine fabric moiré.
- **Control 2: Spatial Lateral Chemical Diffusion (Mackie Lines / Developer Exhaustion)**:
  - Extends vertical DIR coupling to 2D spatial inhibitor diffusion across contrasting borders, recreating authentic photochemical Mackie lines and developer exhaustion boundaries for organic edge acutance without digital unsharp masking.
- **True 16-Bit Archival TIFF Export (Cross-Platform & Web)**:
  - Generate authentic 16-bit uncompressed RGB TIFF 6.0 files with genuine 65,536 tonal levels per channel ($0\dots65,535$) for downstream Photoshop editing and fine-art print RIPs.
  - *Web Darkroom Architecture*: Render via an `RGBA16F` half-float framebuffer attachment in WebGL 2 (or route full-resolution RAW frames directly through the compiled `f32` Rust WASM export pipeline), capturing true high-precision continuous gradients with zero comb-tooth histogram banding.
- **Strict WYSIWYG Tonal & Color Parity Invariant**:
  - Viewport and archival export match bit-for-bit in global tone curve, AgX mapping, color separation, and White Balance.

### 3.6 Cross-Platform Linux Toolchain & Headless Development Engine (P1 - Pulled Forward)
- **Kubuntu & Multi-Arch Linux Toolchain (x86_64 / aarch64)**:
  - Validate and guarantee that `firstproof-core` compiles and passes all unit tests natively on Linux (tested on Kubuntu / Ubuntu 24.04 LTS).
  - Document required Linux development packages (`build-essential`, `libclang-dev`, `liblcms2-dev`, `pkg-config`).
  - Purge any remaining macOS/Darwin-specific assumptions from the Rust core (e.g. hardcoded `/Users/...` path expectations, case-insensitivity assumptions, or Darwin linker flags).
- **Headless `firstproof-cli` Developer Binary**:
  - Implement an ergonomic CLI developer tool (`firstproof-cli` in `firstproof-core/src/bin/cli.rs`) that allows running the full RAW darkroom pipeline directly from the Linux terminal:
    ```bash
    cargo run --bin firstproof-cli -- develop input.cr2 --out preview.tiff --exposure +0.5 --profile agx-vivid
    ```
  - Enables direct, headless development and visual verification of color science, raw demosaicing, AgX tone mapping, and upcoming film emulsions on Linux (opening output in KDE Gwenview or GIMP) without needing a macOS desktop or GPU canvas.
- **Cross-Platform CI Gate (GitHub Actions)**:
  - Add a GitHub Actions workflow running `cargo test --all-targets` across both `ubuntu-latest` and `macos-latest`.
  - Enforces cross-platform portability as a hard merge gate for all future core Rust development.
- **Clean Architectural Decoupling (Keeping Us Honest)**:
  - Keeps the core color science, SQLite catalog database, XMP sidecar parser, and export pipeline 100% platform-agnostic and decoupled from macOS AppKit/SwiftUI.
  - Lays the architectural groundwork for a future native Linux desktop GUI (e.g. Qt/KDE or Rust/WGPU) sharing the exact same `firstproof-core` engine.

### 3.7 WebAssembly Darkroom Engine & Static Web Launch (P0 / P1 - Pulled Forward)
- **`firstproof-wasm` Core Crate Scaffolding**:
  - Extract lightweight WebAssembly bindings target using `wasm-bindgen` and `wasm-pack`.
  - Compiles pure Rust color science (`color/`), AgX filmic tonemapping, Fritsch-Carlson tone curves (`curve.rs`), and fast RAW preview extraction (`raw_decoder.rs`).
  - Pre-allocated RAM scratchpad buffer to eliminate browser garbage collection (GC) stutter and memory fragmentation.
- **Zero-Copy WebGL 2 Shader Pipeline**:
  - GLSL 3.00 ES fragment shader port of `ColorScience.metal` and `Shaders.metal`.
  - Uniform Buffer Objects (UBO) matching `EditSettings` (Exposure, Contrast, WB Temp/Tint, Bilateral Tone Recovery, Curves, Emulsion presets).
  - Direct pointer access into WASM linear memory (`wasm.memory.buffer`) for zero-copy texture upload via `gl.texSubImage2D`.
- **Local Folder Intake & Bi-Directional XMP Sync**:
  - **Folder Selection**: Integrate File System Access API (`window.showDirectoryPicker()`) with fallback to `<input type="file" webkitdirectory>`.
  - **Lazy Streaming Ingestion**: Handle 500+ RAW folders without loading multi-gigabyte payloads into browser RAM.
  - **Direct Sidecar Write-Back**: When granted directory permission, edits and star ratings write back directly to disk as adjacent `.xmp` sidecars.
  - **In-Memory Filmstrip**: Horizontal thumbnail carousel with single-key rating shortcuts (`1`–`5`, `P`/`X`) and keyboard navigation.
- **Static Web Distribution & Privacy-First Ethos (`firstproof.photo`)**:
  - Host statically on GitHub Pages via GitHub Actions CI. Zero backend servers, 100% private on-device processing.
  - Clean, typography-driven landing page embedding the interactive web darkroom.
  - Strict anti-marketing principles: Zero email address capture, zero tracking scripts, zero waitlists. Direct link to open source repo and binaries when ready.


### 3.8 Web Darkroom Subsystem Parity & Anti-Slop Initiative (P0 / P1)

A systematic initiative executed strictly under the [Subsystem Parity & Reference Deconstruction Protocol](../../.agents/skills/firstproof-subsystem-parity/SKILL.md) to eliminate duplicated logic, establish single sources of truth in `firstproof-types`, achieve 1:1 mathematical and behavioral parity with native macOS Metal, and purge AI slop and fake parity.

> **Authoritative Technical Mini Specs**:
> - [Viewport Geometry, Gestures & Texture Lifecycle](../specs/viewport_geometry_and_gestures.md)
> - [Pipeline Stages & Color Space Working Invariants](../specs/pipeline_stages_and_color_spaces.md)

- **Story 3.8.2: Shader Geometry & Viewport Transformation Parity**:
  - Port `transformGeometry()` from `Geometry.metal` to GLSL in `WebGLRenderer.ts` matching [viewport_geometry_and_gestures.md](../specs/viewport_geometry_and_gestures.md#3-shader-geometry-transformation-transformgeometry).
  - Wire uniforms for `u_cropLeft`, `u_cropTop`, `u_cropRight`, `u_cropBottom`, `u_cropModeActive`, `u_straightenAngle`, and `u_cropConstrain`.
  - Delete dead CPU-side vertex buffer mutation (`updateQuadGeometry`) in favor of GPU vertex/fragment coordinate transforms.
  - Implement `isIntermediate` bypass invariant (pass through sampled color directly without re-applying tone curves or geometry).
  - Optimize `sampleDisplayLuma` in WebGL to reuse `y_c` and avoid running the entire pipeline 5× per pixel for sharpening.
  - *Verification*: Viewport inspection confirming cropped image fills canvas when exiting crop mode, straighten angle rotates cleanly, and intermediate textures bypass transforms.

- **Story 3.8.3: Crop Controller & Desktop Interaction Parity**:
  - Refactor `CropController.ts` into a stationary full-canvas gesture container covering the entire viewport, dimming outside the active crop box (fixing unscrimmed letterbox bars) matching [viewport_geometry_and_gestures.md](../specs/viewport_geometry_and_gestures.md#4-stationary-full-canvas-gesture-architecture).
  - Change outside-crop drag behavior: dragging outside the crop box initiates **straighten rotation** with a floating readout HUD matching `CropOverlayView.swift` (replacing `'create'` mode).
  - Fix aspect ratio inversion for portrait images and verify the <kbd>X</kbd> flip shortcut.
  - *Verification*: Interactive crop adjustments, outside-drag rotation, HUD placement, and keyboard shortcut parity.

- **Story 3.8.4: Zero-Flashing Preview & Transitional Staging Parity**:
  - Implement texture staging and bypass lifecycle matching [viewport_geometry_and_gestures.md](../specs/viewport_geometry_and_gestures.md#5-intermediate-preview--texture-lifecycle).
  - Update `RawDecodeWorker.ts` and `firstproof-wasm` to stream RAW files directly into 16-bit linear binned frames (`decode_raw_linear_from_scratchpad`) without extracting camera JPEGs first.
  - Up-front edit staging: resolve baseline EXIF orientation and XMP edits *before* updating preview state.
  - Immediately clear to the neutral dark canvas with the developing spinner during the 10–15ms photo switch to eliminate stale pixel flashing.
  - *Verification*: Burst culling test with RAW files showing zero manufacturer JPEG flashes and clean dark transitions.



- **Story 3.8.7: Deconstruct the `main.ts` Monolith (2,586 Lines) (P1)**:
  - Extract dedicated modules from `main.ts`:
    - `KeyboardShortcutController.ts` (first-responder shielded desktop shortcuts)
    - `InspectorSyncController.ts` (slider bindings, curve listeners, tab navigation)
    - `FilmstripController.ts` (thumbnail card rendering, DOM recycling, selection, and viewport-driven lazy loading via `IntersectionObserver` / scroll listeners with concurrency throttling so thumbnails continuously load as the user scrolls)
    - `PhotoLoaderService.ts` (worker dispatch, debounce staging, cache coordinator)
  - Reduce `main.ts` to a lean application coordinator (< 250 lines).
  - *Verification*: Full end-to-end regression check across all tools, keyboard shortcuts, filmstrip smooth scrolling with lazy thumbnail hydration, and inspector tabs.

### 3.9 Unified RAW Container Intake & Camera Calibration Registry Engine (P1)
- **Eliminate Ad-Hoc Tag Scanning (`firstproof-types::container`)**:
  - Replace fragmented, hand-rolled binary scanners (`preview.rs`, `raw.rs`, `dcp.rs`) with a single, strongly-typed `RawContainerManifest` parser.
  - Parse TIFF/DNG IFD hierarchies (IFD0, SubIFDs, ExifIFD) in a single pass, strongly typing all DNG specification tags: `BaselineExposure` (Tag 50728), `BaselineNoise` (Tag 50730), `BaselineSharpness` (Tag 50731), `LinearResponseLimit` (Tag 50734), `ColorMatrix1/2`, `ForwardMatrix1/2`, `AnalogBalance`, `AsShotNeutral`, and `ActiveArea`.
- **Structured Camera Calibration Database (`CameraCalibrationRegistry`)**:
  - Build an in-memory, zero-allocation calibration database mapping camera make and model to authoritative physical sensor properties.
  - Harness `rawler`'s built-in 700+ camera calibration dataset (MIT / Apache-2.0) for sensor CFA patterns, active crop areas, and dual-illuminant color matrices.
  - Augment with Adobe DNG Converter baseline exposure offsets and ForwardMatrices across major camera families (Canon EOS, Sony Alpha, Nikon Z/D, Fujifilm X, Panasonic Lumix, OM System/Olympus, Leica, Hasselblad).
  - Eliminate all fragile ad-hoc string heuristics (`if m.contains("canon")`) in favor of exact model indexing with principled manufacturer-family fallbacks.
- **Dual-Format RAW vs DNG Regression Suite**:
  - Integration test `test_raw_vs_dng_intake_parity` auditing paired proprietary RAWs and their Adobe DNG converted counterparts (e.g. `_MG_2332.CR2` vs `_MG_2332.dng`).
  - Guarantees bit-level agreement: baseline exposure delta $\Delta \text{EV} < 0.01$, identical active sensor crop, and photometric wedge continuity across both native and web darkroom targets.

### 3.10 Architecture Clean-Up: Rename `firstproof-types` to `firstproof-engine` & Core De-Duplication (P1)
- **Architectural Crate Reclassification & Rename**:
  - Rename crate `firstproof-types` to **`firstproof-engine`** (or `firstproof-kernel`), correcting the historical misnomer.
  - Formally establish the 3-tier workspace architecture:
    1. **`firstproof-engine`**: Pure, cross-platform algorithmic kernel (color science, sensitometry, AgX, demosaicing, auto-algorithms, tone curves, XMP). Zero native OS or C dependencies (`glam`, `serde`); compiles anywhere (macOS, Linux, WASM, embedded).
    2. **`firstproof-core`**: Native desktop host adapter (SQLite catalog, LibRaw/Rawler disk I/O, LCMS2 ICC profiles, Rayon multithreaded batch export, UniFFI bridge).
    3. **`firstproof-wasm`**: WebAssembly host adapter (memory scratchpad, `wasm-bindgen`, JS array buffer marshalling).
  - Update all `Cargo.toml` workspace definitions, crate references, internal imports, and documentation.
- **Deduplicate RAW Sensor Decoding**:
  - Move `decode_raw_sensor_frame` into `firstproof-engine/src/raw.rs` as a shared method on `RawSensorFrame`.
  - Both `firstproof-core/src/raw_decoder.rs` and `firstproof-wasm/src/lib.rs` consume the exact same decoder, eliminating 100 lines of copy-pasted Rawler decoding logic.
- **Deduplicate XMP Engine**:
  - Make `firstproof-engine/src/xmp.rs` the single canonical implementation of `XmpData`, `format_gps_xmp`, `parse_gps_xmp`, and XML read/write.
  - Re-export from `firstproof-core::xmp`, eliminating duplicate files (`data.rs`, `reader.rs`, `writer.rs`).
- **Deduplicate Metadata Structs**:
  - Unify `ImageMetadata` (`firstproof-core/src/raw.rs`) and `ImageMetadataRecord` (`firstproof-engine/src/lib.rs`).
- **Verification**:
  - `cargo test --all-targets` across `firstproof-engine`, `firstproof-core`, and `firstproof-wasm`.

### 3.11 Sensitometric Emulsion Balancing & Physical Optical Grain Engine (P2)
> **Technical Mini Specs**:
> - [Stochastic Emulsion Resynthesis Mini Spec](../specs/stochastic_emulsion_resynthesis.md)
> - [Global Emulsion & Darkroom Print Roster Mini Spec](../specs/global_emulsion_and_darkroom_roster.md)
- **Global Chemical & Sensitometric Roster Balancing**:
  - Rationalize North American stocks: merge `RochesterDaylight200` into `RochesterPortrait400` (eliminating redundant C-41 variants in favor of pure exposure/saturation modulation).
  - Add **`LeverkusenPortrait160`** (*Agfa Agfacolor / Vista* - Leverkusen, Germany): European substantive dye couplers, earthy ochre midtones, soft pastel greens, and gentle shadow saturation.
  - Add **`AshigaraPortrait400`** (*Fujicolor Pro 400H* - Minamiashigara, Japan): Patented 4th color layer (cyan-sensitive inter-layer) delivering airy Japanese portrait pastels and fluorescent green suppression.
  - Add **`LiguriaCinema80`** (*Ferrania P30* - Cairo Montenotte, Italy): Legendary 1960s Italian Neorealist high-silver cinema emulsion (Fellini, Pasolini), featuring deep velvety blacks, ultra-sharp acutance, and punchy microcontrast.
  - Add **`BaodingSilver100`** (*Lucky SHD 100* - Baoding, China): Iconic domestic Chinese cubic silver halide featuring authentic optical halation bloom around specular highlights due to minimal anti-halation backing.
- **Physical Grain Synthesis via Stochastic Emulsion Resynthesis**:
  - **Sensitometric Domain Placement Invariant**: Grain is simulated in the optical scene domain *before* AgX tone mapping and print transfer, avoiding shadow noise floor amplification and non-linear gamma clipping.
  - **Noise Transmutation (Zero Plastic Denoising)**: Repurpose high-ISO sensor noise as the latent nucleation seed for silver crystals or subtractive dye clouds via Chrominance Phase Consolidation and Nutting midtone density modulation ($4y(1-y)$).
  - **Stock-Specific Crystal Morphology**:
    1. *Metallic Silver Crystals* (`MobberleySilver400`, `LiguriaCinema80`, `BaodingSilver100`): Sharp, irregular crystalline silver grains with stock-calibrated acutance and density distribution.
    2. *Organic Dye Clouds* (`RochesterPortrait400`, `AshigaraPortrait400`, `LeverkusenPortrait160`): Soft, overlapping microscopic dye clouds left behind after silver bleaching in C-41/E-6 processes.
    3. *Optical Halation Scattering* (`BaodingSilver100`): Internal reflection and lateral light dispersion across the film base producing natural specular glow without digital bloom filters.
  - **Real-Time GPU Performance Budget (< 0.1ms at 4K)**:
    - Fold into existing 4-tap detail unsharp masking shader pass, requiring zero extra texture lookups and zero additional memory bandwidth.
  - **Multi-Engine Mathematical Parity**:
    - Procedural SIMD/hash-based grain synthesis matching bit-for-bit across Rust CPU export (`color/grain.rs`), Metal GPU shaders (`Shaders.metal`), and WebGL fragment shaders (`FragmentShader.ts`).
  - **Verification**:
    - Parity test suite verifying identical grain density across wedges $(-10\text{ EV}$ to $+6.5\text{ EV})$ in Rust, Metal, and WebGL.

### 3.12 West African Studio Darkroom Chemistry & Deep-Tone Sensitometry (P2)
> **Technical Mini Spec**: [Global Emulsion & Darkroom Print Roster Mini Spec](../specs/global_emulsion_and_darkroom_roster.md#3-darkroom-print-media-west-african-studio-chemistry)
- **Honoring West African Studio Print Culture (`BamakoSilverContact`)**:
  - Add **`BamakoSilverContact`** to `PrintMediaMode`, formalizing the darkroom printmaking tradition of Bamako, Mali (Seydou Keïta, Malick Sidibé) and Bobo-Dioulasso (Sanlé Sory).
  - Modeled on large-format 1:1 contact printing onto high-silver bromide paper with high-concentration hydroquinone/metol developers and ferrotype gloss finishing.
- **Sensitometric Characteristics**:
  - **Zone III–V Microcontrast & Luminosity**: Expands tonal separation across rich dark skin tones and shadow textures, eliminating the muddy compression typical of Caucasian-biased historical paper curves.
  - **High-Key Textile Acutance**: Retains brilliant, non-blooming edge contrast in Zone VII–VIII highlights for intricate patterned fabrics (Ankara wax prints, woven mudcloth, lace) and jewelry.
  - **Ferrotype Mirror Dmax**: Deep, inky maximum black density ($D_{\max} \ge 2.25$) with neutral-to-cool specular silver reflectance.
- **Deep-Tone Sensitometric Test Suite**:
  - Implement a specialized synthetic reflectance wedge in `tests/print_media_parity.rs` auditing Zones II through VI across diverse skin reflectance models.
  - Guarantees strictly monotonic derivative separation ($df/dx > 0$) without clipping, solarization, or plastic color flattening.
- **Parity Across Engines**:
  - Implement identical mathematical transfer functions across `firstproof-core/src/color/print_media.rs`, `Shaders.metal`, and `FragmentShader.ts`.

---

## Phase 4: Pre-Release IP Audit, Code Cleanliness & Distribution (P2)

### 4.1 Open Source IP, Security & Code Cleanliness Auditing
- **License Compliance & Copyleft Gate (`cargo-deny`)**:
  - Whitelist permissive licenses (`MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `MPL-2.0`, `CC0-1.0`); hard-block `GPL` / `AGPL`.
  - Audit `rawler` (LGPL-2.1) to ensure standalone dynamic linking or clean-room replacement without copyleft contamination.
- **Code Cleanliness & Provenance Scanning (`scancode` / `license-eye`)**:
  - Scan codebase for accidental copy-pasted GPL snippets, proprietary headers, or unlicensed third-party code.
  - Standardize SPDX license headers (`SPDX-License-Identifier: Apache-2.0 OR MIT`) across all Rust, Swift, and Metal files.
- **Secret & Credential Detection (`gitleaks`)**:
  - Pre-commit and CI scans to guarantee no private keys, signing certificates, local absolute paths, or API tokens exist in repository git history.
- **Dependency Vulnerability Auditing (`cargo-audit`)**:
  - Scan dependency graph against the RustSec Advisory Database for known CVEs and memory safety advisories.
- **Memory Safety & Undefined Behavior (`cargo miri`)**:
  - Validate unsafe FFI pointer conversions and raw byte slicing in `raw_decoder` and `export` modules under Miri.

### 4.2 Active Codebase Path Sanitization & History Scrubbing
- **Active Codebase Path Sanitization**:
  - Purge hardcoded `/Users/<username>/...` paths from `FirstProofApp.swift`, `calibrate.rs`, and test files (`camera_profiles_parity.rs`). Replace with dynamic `Bundle` lookups and `TEST_FIXTURES_DIR` / environment variables.
- **Committed Test Scratch Cleanup**:
  - Delete committed scratch debugging blocks in `camera_profiles_parity.rs` (lines 408–480) referencing local desktop paths and IDE attachment folders (`.user_uploaded/media_...`).
- **Git History Rewrite & Scrubbing Pass (`git-filter-repo` / BFG)**:
  - Perform an automated history rewrite prior to public release to scrub local paths, developer usernames, private emails, and transient binary blobs from git commit history.
  - Followed by aggressive reflog expiration and garbage collection (`git reflog expire --expire=now --all && git gc --prune=now --aggressive`).

### 4.3 Web Distribution & Release Site (`firstproof.photo`)
- **Craft-First, Zero Sales Pitch Design**:
  - Restrained, typography-driven static page (deep neutral darks, editorial layout, zero marketing popups, tracking cookies, or conversion fluff).
  - Unadorned application screenshots showing the actual darkroom interface and workflow.
  - Zero email harvesting or lead capture forms: pure, quiet software craft.
- **Integrated In-Browser Test Drive**:
  - Photographers can drag and drop their own RAW files or click "Open Folder" to test First Proof instantly without downloading anything.
- **Visual Proof Gallery & Optical Comparison**:
  - Curated gallery of real photographic captures demonstrating authentic color rendition, natural skin tones, and subtle tonal gradations.
  - Side-by-side inspection: In-Camera JPEG (cooked 8-bit sRGB with blown highlights) vs. First Proof (16-bit linear Rec.2020 AgX output).
- **On the Shoulders of Giants (Attribution & Lineage)**:
  - Dedicated recognition honoring the foundational pioneers whose open work makes First Proof possible:
    - **Troy Sobotka**: Groundbreaking research on AgX color science, open filmic tonemapping, and dynamic range preservation.
    - **Marti Maria**: LittleCMS (`lcms2`) open color management system.
    - **Dave Coffin & RAW Decoding Pioneers**: `dcraw`, `libraw`, and `rawler` for reversing proprietary camera Bayer sensor formats.
    - **Torsten Bronger & Lensfun Community**: Open lens database and optical distortion modeling.
    - **CIE Colorimetry & Open Standards**: CIE 1931/1976 colorimetric observers, Planckian locus math, and open standard specifications.

### 4.4 macOS Desktop Binary Release & Packaging Pipeline (Deferred / Standalone Initiative)
- **Decoupled macOS Release Milestone**:
  - Packaging the standalone native macOS application (compiling `libfirstproof_core.a`, running `uniffi-bindgen`, building the SwiftUI `.app` bundle, Developer ID code signing, and Apple `notarytool` stapling) is explicitly preserved as a dedicated release-readiness milestone.
  - While developing in the Linux environment, this track is held in place. It will be executed upon returning to a macOS environment or wired into a dedicated `macos-14` / `macos-15` GitHub Actions packaging runner.
  - Direct download links will be posted openly when binaries are ready—no email gates or signups.

### 4.5 Swift Desktop Parameter Schema & Behavioral Interaction Parity (P1 - macOS Environment)
- **UniFFI Schema Bridge**:
  - Expose `AdjustmentDescriptor`, `all_adjustments()`, and Mired conversion math from `firstproof-types` via `firstproof-core` UniFFI scaffolding.
- **Swift View & Model Schema Consumption**:
  - Refactor `SliderRow.swift`, `DevelopInspectorView.swift`, and `AppViewModel.swift` to derive slider ranges, steps, defaults, and formatters directly from the canonical Rust descriptors rather than hardcoding values in Swift views.
- **Unified Interaction Behavioral Contracts**:
  - Implement the identical interaction state machine in native AppKit / SwiftUI matching [ui_architecture.md](../architecture/ui_architecture.md):
    - Record initial value on drag gesture start.
    - <kbd>Shift</kbd>+drag fine-scrubbing ($0.1\times$ precision damper).
    - <kbd>Escape</kbd> key drag rollback (cancels pending scrub and reverts value without pushing undo).
    - Double-click / double-tap reset on slider thumb and row labels.
    - Tabular lining numbers (`.monospacedDigit()`) across all inspector readout labels to eliminate width jitter.
- **Persistent Heads-Up EXIF Badges (CAM / ISO / Aperture / Shutter)**:
  - Adopt the web darkroom's fixed 4-badge EXIF summary element (`CAM`, `ISO`, `APERTURE`, `SHUTTER`) directly in macOS `PermanentHeaderCard` (`InspectorHeaderView.swift`) positioned above the segmented tabs, so essential capture parameters remain visible at a glance across every workspace tab.
- **Cross-Platform Verification Gate**:
  - Verify 1:1 parameter boundaries, visual feedback, and keyboard/mouse interaction parity between native macOS and WebAssembly darkroom targets.

---

## Phase 5: Post-v1 & Long-Term Vision

### 5.1 Google Liberation Bridge & Cloud Quarantine (Post-v1)
- **Pixel Computational Linear DNG Ingestion**: Ingest 16-bit multi-frame linear DNGs (`PXL_*.dng`) with embedded lens shading gain maps.
- **Air-Gapped Cloud Rejects Quarantine**:
  - Decouple local reject flagging (`X`) from remote cloud deletion.
  - Dedicated "Rejected Google Photos" sidebar scope where candidates must be explicitly reviewed.
  - Batch "Remove from Cloud" action with confirmation dialog, moving remote items to Google Photos Trash rather than permanent obliteration.
  - Provider-agnostic interface (`CloudProviderBridge`) prepared for future iCloud/Dropbox integrations.

### 5.2 Light-Touch Video Cataloging & Metadata Ingestion (Post-v2)
- **Unified Mixed-Media Chronology**:
  - Index camera video clips (`.mov`, `.mp4`, `.m4v`) into the catalog database so hybrid photo/video shoots appear in exact chronological sequence alongside RAW frames.
- **Hardware-Accelerated In-Canvas Playback (AVKit / AVFoundation)**:
  - Spacebar playback, J/K/L shuttle scrubbing, video duration/frame-rate overlays, and sub-millisecond poster-frame thumbnail caching.
- **Culling & XMP Metadata Support**:
  - Full support for single-key star ratings (`1`..`5`), pick/reject flags (`P`/`X`), keywords, captions, and title metadata stored in adjacent `.xmp` sidecars (`clip.mp4.xmp`) without touching immutable master video files.
- **Strict Anti-Bloat Invariant (Zero Video Editing)**:
  - Explicit non-goal: No video grading, color adjustments, cutting, or re-encoding. Master video files remain completely read-only.
  - Dedicated "Open in External NLE / Player" shortcut (`Cmd+Return`) to hand off clips directly to QuickTime, DaVinci Resolve, or Final Cut Pro.

### 5.3 Hardware Filtering, Manual Collections & Dynamic Smart Albums (Post-v1)
- **Per-Camera Sidebar Line Items & Grid Format Triage**:
  - **Sidebar Camera List**: Dynamic list of every camera model detected in the current catalog (e.g. `Sony ILCE-7M4 (530)`, `Fujifilm X100V (412)`, `Google Pixel 8 Pro (306)`), allowing instant 1-click scoping by capture device.
  - **Grid-Level "RAWs Only" Toggle**: An unobtrusive toggle/segmented control in the contact sheet grid header (`[ All Formats | RAW Only ]` or `Hide JPEGs`) that filters out raster JPEGs when the photographer is focused purely on sensor darkroom grading, while keeping in-camera JPEGs accessible whenever needed.
- **Manual Albums & Custom Print Sequencing**:
  - Non-destructive album collections (`albums`, `album_photos`) stored in local `.firstproof/catalog.db` with dual XMP collection synchronization.
  - Support for `Sort By: Custom Sequence` with drag-and-drop reordering for photobook pagination, exhibition layouts, and print runs.
- **Dynamic Smart Albums (Tonal & Curation Predicate Engine)**:
  - Fast, SQL-compiled smart queries matching any/all rules: Rating (`>= 4★`), Status (`Pick`), Date Range, Camera Model, Lens, Focal Length, and Edit State (`Has Edits`).
  - Streamlines pre-print proofing (e.g. "All 4★+ picks taken on 50mm between Oct–Dec").

### 5.4 Cross-Platform Offline Search, Geotagging & Face Clustering (Post-v1 / Post-v2)
- **Time-Proximity Geotagging ("Borrow GPS from Mobile")**:
  - Correlate GPS coordinates from companion mobile photos within an adjustable time window (e.g. $\pm 30$–$60$ min) for camera RAWs lacking onboard GPS hardware.
  - Linear spatial trajectory interpolation when a RAW shot is bracketed between two mobile GPS locations.
  - Camera clock drift calibration slider (`±MM:SS`) with reference photo alignment.
  - Transparent metadata auditing: Distinguish exact hardware coordinates from approximate inferred locations (`FIRSTPROOF_TIME_PROXIMITY_INFERRED` in XMP sidecars and muted pin badge in inspector).
- **Platform-Agnostic Offline Concrete Object Search**:
  - Primary engine written in cross-platform Rust (`firstproof-core`), evaluating lightweight on-device vision models (e.g. MobileNet / ONNX runtime or pure Rust inference) that run identically across Linux, macOS, and WebAssembly.
  - Sub-millisecond background classification during folder indexing; categorizes 1,000+ concrete objects into SQLite `photo_classifications`.
  - Zero UI clutter: no bounding boxes or tags drawn on the artist's canvas; full-text search bar returns matching photos instantly.
  - **Optional Apple Intelligence / Vision Accelerator Plugin**: On macOS, users can optionally opt-in to Apple Neural Engine acceleration (`VNClassifyImageRequest`). Deduplication logic merges overlapping labels cleanly based on confidence scores.
- **Restrained Archival Map View Bounding Box Search**:
  - Muted, dark-slate map canvas matching First Proof's `#1E1E1E` darkroom palette (Leaflet / MapKit depending on platform).
  - Dynamic viewport bounding-box synchronization: panning/zooming the map instantly updates the contact sheet grid.
- **Cross-Platform Archival Portrait Clustering (MWG/IPTC Standard Regions)**:
  - 100% offline, on-device portrait face detection and embedding clustering in Rust (e.g. lightweight UltraFace / YuNet ONNX or pure Rust).
  - Optional macOS opt-in: Apple `Vision` face observation can be enabled as an alternative hardware accelerator.
  - Unidentified sitter triage deck: *"Found X photos of this person. Who is this sitter?"*
  - Serializes coordinates directly into open-standard [MWG (Metadata Working Group) / IPTC XMP Face Regions](../values.md#L43) (`mwg-rs:Regions`), avoiding proprietary database lock-in across all platforms.

### 5.5 Optical Notch & Spectral Illuminant Modeling (Deep Backlog / Exploratory R&D)
- **Physical Context & The Multi-Source Problem**:
  - Gas-discharge lighting (low/high-pressure sodium streetlamps, industrial mercury/fluorescent tubes) produces discrete atomic line emission spikes (e.g., $589\,\text{nm}$ sodium doublet, $546\,\text{nm}$ mercury green) rather than continuous thermal Planckian radiation.
  - In real-world mixed-source captures (e.g., subway platforms combining tungsten, fluorescent tubes, and ambient exterior light), automatic illuminant deconvolution is mathematically ill-posed and prone to false color artifacts.
- **Exploratory Darkroom & Optical Emulations (Preserving Anti-Slop Invariants)**:
  - **Virtual Optical Notch Filtering**: Emulate physical chemical glass filters (such as didymium/neodymium glass that sharply absorbs the $580\text{–}590\,\text{nm}$ sodium doublet, or analog FL-D/FL-W magenta glass) to selectively suppress line-emission spikes without distorting continuous adjacent spectral channels.
  - **Targeted Hue-Band De-Spiking**: Parametric narrow-band chromaticity qualifiers in the color engine that can gently attenuate saturated line-emission spikes toward natural warm amber/tungsten, anchored to known physical emission wavelengths.
  - **Strict Anti-Slop Guardrail**: Never attempt synthetic multi-source AI auto-hallucination. Any future tool must remain a conscious, physical optical emulation with predictable, manual photographer control.

---

## Phase 6: Modern Platform Architecture & macOS 27 Alignment (P2)

### 6.1 Minimum Deployment Target Bump (N-2 Window on macOS 27 Launch)
- **Target Bump & Toolchain Modernization**:
  - Upon general availability of macOS 27, bump minimum deployment target in `Package.swift` from `.macOS(.v14)` to `.macOS(.v25)`.
  - Preserves the standard "N-2" pro-photographer workstation support window (supporting macOS 25, 26, and 27).
  - Resolves Darwin linker skew warnings (`built for newer 'macOS' version than being linked`).
  - Enables strict Swift 6 concurrency runtime isolation checks across the bridge and view models without false positives.
- **Layout & Render Work Unlocked**:
  - **Adopt `.onGeometryChange(for:of:action:)`**: Completely eliminate `GeometryReader` across `ToneCurveView`, `ColorWheelView`, and crop interaction layers in favor of direct render-tree size observation, eliminating layout recursion and frame jitter.
  - **Native `.windowDraggable()`**: Clean canvas window movement without AppKit window tracking overrides.
- **Anti-Slop Guardrail (Rejection of "Liquid Glass" Translucency)**:
  - Proactively reject refractive translucent materials ("liquid glass") in inspector sidebars or canvas chrome.
  - Maintain First Proof's strict opaque, matte dark slate palette (`#181818` / `#1E1E1E`). Desktop wallpaper or background window radiance must never bleed into the photographer's peripheral vision and distort neutral chromatic adaptation.
