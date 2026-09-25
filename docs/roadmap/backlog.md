# First Proof Active Backlog & Roadmap

This document strictly tracks pending and deferred features calibrated against our core philosophy: **"Pure capture. Honest color. Zero digital plastic. Print & web first."**

> **Note**: For completed stories, see [completed.md](file:///home/remco/src/firstproof/docs/roadmap/completed.md). For color pipeline specifications, see [color_pipeline.md](file:///home/remco/src/firstproof/docs/architecture/color_pipeline.md). For web architecture, see [web_architecture.md](file:///home/remco/src/firstproof/docs/architecture/web_architecture.md).

---

## 🚀 Immediate Next Up
1. **Web Darkroom Subsystem Parity & Anti-Slop Initiative (P0 / P1)**: 6 remaining stories establishing shader geometry/crop parity, desktop gesture interaction parity, zero-flashing preview staging, and eliminating the duplicate TypeScript oracle and fake 16-bit export slop.
2. **Cross-Platform Linux Toolchain & Headless Development Engine (P1)**: Kubuntu/Linux compilation, headless `firstproof-cli` developer testbed, and cross-platform CI gate.
3. **Archival Open DNG Converter & Lossless Compression Engine (P2)**: Lossless Bayer CFA transcoding and Deflate tile compression.
4. **Multi-Camera Color Calibration & Sensor Matching Engine (P2)**: Target chart profiling and spectral sensitivity matching.
5. **Print Proofing & Gamut Warning Engine (P2)**: Soft proofing against ICC profiles and out-of-gamut clipping alerts.
6. **Archival High-Precision Multi-Pass Export Engine (P2)**: Multi-pass AMaZE/RCD demosaicing and 2D spatial developer exhaustion diffusion.

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
    1. *RAW Demosaicing Precision*: `Standard (Fast)` vs. `Archival (Edge-Adaptive AMaZE / RCD)`.
    2. *Spatial Chemical Edge Diffusion (Mackie Lines)*: Dedicated toggle when an emulsion is active.
- **Control 1: Multi-Pass Edge-Directed Demosaicing (AMaZE / RCD)**:
  - *Standard (Fast)*: Near-instant batch exports (150–250ms per RAW) using optimized cubic Bayer demosaicing.
  - *Archival (Edge-Adaptive)*: Directional edge-adaptive demosaicing (AMaZE / RCD) with chromatic moiré suppression passes (~500ms per RAW). Eliminates "zipper" artifacts, maze patterns, and false color fringing on high-frequency fabrics, architectural grids, and bird feathers on 45–61MP sensors.
- **Control 2: Spatial Lateral Chemical Diffusion (Mackie Lines / Developer Exhaustion)**:
  - Extends vertical DIR coupling to 2D spatial inhibitor diffusion across contrasting borders, recreating authentic photochemical Mackie lines and developer exhaustion boundaries for organic edge acutance without digital unsharp masking.
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

A systematic initiative executed strictly under the [Subsystem Parity & Reference Deconstruction Protocol](file:///home/remco/src/firstproof/.agents/skills/firstproof-subsystem-parity/SKILL.md) to eliminate duplicated logic, establish single sources of truth in `firstproof-types`, achieve 1:1 mathematical and behavioral parity with native macOS Metal, and purge AI slop and fake parity.

- **Story 3.8.2: Shader Geometry & Viewport Transformation Parity**:
  - Port `transformGeometry()` from `Geometry.metal` to GLSL in `WebGLRenderer.ts`.
  - Wire uniforms for `u_cropLeft`, `u_cropTop`, `u_cropRight`, `u_cropBottom`, `u_cropModeActive`, `u_straightenAngle`, and `u_cropConstrain`.
  - Delete dead CPU-side vertex buffer mutation (`updateQuadGeometry`) in favor of GPU vertex/fragment coordinate transforms.
  - Implement `isIntermediate` bypass invariant (pass through sampled color directly without re-applying tone curves or geometry).
  - Optimize `sampleDisplayLuma` in WebGL to reuse `y_c` and avoid running the entire pipeline 5× per pixel for sharpening.
  - *Verification*: Viewport inspection confirming cropped image fills canvas when exiting crop mode, straighten angle rotates cleanly, and intermediate textures bypass transforms.

- **Story 3.8.3: Crop Controller & Desktop Interaction Parity**:
  - Refactor `CropController.ts` into a stationary full-canvas gesture container covering the entire viewport, dimming outside the active crop box (fixing unscrimmed letterbox bars).
  - Change outside-crop drag behavior: dragging outside the crop box initiates **straighten rotation** with a floating readout HUD matching `CropOverlayView.swift` (replacing `'create'` mode).
  - Fix aspect ratio inversion for portrait images and verify the <kbd>X</kbd> flip shortcut.
  - *Verification*: Interactive crop adjustments, outside-drag rotation, HUD placement, and keyboard shortcut parity.

- **Story 3.8.4: Zero-Flashing Preview & Transitional Staging Parity**:
  - Update `RawDecodeWorker.ts` and `firstproof-wasm` to stream RAW files directly into 16-bit linear binned frames (`decode_raw_linear_from_scratchpad`) without extracting camera JPEGs first.
  - Up-front edit staging: resolve baseline EXIF orientation and XMP edits *before* updating preview state.
  - Immediately clear to the neutral dark canvas with the developing spinner during the 10–15ms photo switch to eliminate stale pixel flashing.
  - *Verification*: Burst culling test with RAW files showing zero manufacturer JPEG flashes and clean dark transitions.

- **Story 3.8.5: De-Slopping Parity Harness & Deleting the TypeScript Oracle**:
  - Expose reference pixel and frame evaluation directly from Rust WASM in `firstproof-wasm`.
  - Delete the 1,073-line pure TypeScript twin (`web/src/parity/ReferenceExport.ts`).
  - Re-point `ParityHarness.ts` and `web/parity.html` directly to compiled Rust WASM as the sole ground truth.
  - *Verification*: Automated parity test battery running in `parity.html` comparing WebGL shaders against compiled Rust WASM.

- **Story 3.8.6: Client-Side Export Clean-Up & De-Slopping**:
  - Replace the fake bitshifted 16-bit TIFF encoder (`tiffEncoder.ts`) with authentic 8-bit or real WASM-backed export.
  - Fix Display P3 color transform (or cleanly scope to calibrated sRGB).
  - Remove unimplemented "Ultra HDR" and "Rendering Intent" promises from `ExportModal.ts`.
  - Decouple export execution from the live on-screen `<canvas>` so batch export does not resize the viewport during editing.
  - *Verification*: Export single and batch photos to disk with verified color without viewport resizing.

- **Story 3.8.7: Deconstruct the `main.ts` Monolith (2,495 Lines)**:
  - Extract dedicated modules from `main.ts`:
    - `KeyboardShortcutController.ts` (first-responder shielded desktop shortcuts)
    - `InspectorSyncController.ts` (slider bindings, curve listeners, tab navigation)
    - `FilmstripController.ts` (thumbnail card rendering, DOM recycling, selection)
    - `PhotoLoaderService.ts` (worker dispatch, debounce staging, cache coordinator)
  - Reduce `main.ts` to a lean application coordinator (~200 lines).
  - *Verification*: Full end-to-end regression check across all tools, keyboard shortcuts, and inspector tabs.

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
  - Implement the identical interaction state machine in native AppKit / SwiftUI matching [ui_architecture.md](file:///home/remco/src/firstproof/docs/architecture/ui_architecture.md):
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
  - Serializes coordinates directly into open-standard [MWG (Metadata Working Group) / IPTC XMP Face Regions](file:///Users/blikkie/src/LumenBox/docs/values.md#L43) (`mwg-rs:Regions`), avoiding proprietary database lock-in across all platforms.

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
