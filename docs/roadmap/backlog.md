# First Proof Active Backlog & Roadmap

This document strictly tracks pending and deferred features calibrated against our core philosophy: **"Pure capture. Honest color. Zero digital plastic. Print & web first."**

> **Note**: For completed stories, see [completed.md](completed.md). For color pipeline specifications, see [color_pipeline.md](../architecture/color_pipeline.md). For web architecture, see [web_architecture.md](../architecture/web_architecture.md).

---

## 🚀 Immediate Next Up (v0.1 Release Gate)
1. **Story 4.0: Comprehensive AI Slop Audit & Codebase De-Slop Pass** (P0) - COMPLETED
2. **Story 4.5: Swift Desktop Parameter Schema & Behavioral Interaction Parity** (P0) - COMPLETED
3. **Story 4.6: Comprehensive Permissive RAW Decoding Matrix (CR2/CR3, ARW, NEF, RAF X-Trans, RW2, ORF, PEF)** (P0) - COMPLETED
4. **Story 4.4: Standalone Desktop & CLI Release Packaging Pipeline** (P1) - COMPLETED
5. **Story 4.1: Open Source IP, Security & License Compliance Gate** (P1) - COMPLETED
6. **Story 4.3: Web Distribution & Release Site (`firstproof.photo`)** (P1) - COMPLETED
7. **Story 4.8: Unified Shader Pipeline via WGSL & Naga Cross-Compilation** (P1)
8. **Story 4.9: Robust Headless Command Bus, CLI Parity & Unified MCP Bridge** (P1)

---

## Phase 3: Fine-Art Darkroom Chemistry, Archival Formats & Web Playground (P1 / P2)


> **Product Invariant: The Primary Engine vs. The Fine-Art "Treat"**  
> First Proof's primary workhorse default is **pure optical RAW capture via AgX filmic tone mapping**. The core workspaces (**Light**, **Color**, **Detail**, **Optics**) are dedicated strictly to sensor reality: standard White Balance (Temperature & Tint), exposure, tone recovery, and sensor sharpness. Photos look 90% finished immediately without any analog styling.  
> 
> Film stock and print media emulation are **not** the main workflow and **not** a complex simulator. They are an exquisite, 100% opt-in **fine-art treat and differentiator** housed in a dedicated **Emulsion** workspace tab (`InspectorTab.emulsion`). Default state is always `None / Bypass`. When activated, the engine does all the heavy lifting under the hood so the photographer simply selects a curated stock or print medium without fiddling with chemical dials.
>
> **Roadmap Reconsideration: Physical Film Grain in Wide-Gamut Linear Space**  
> Previous attempts at simulating film grain recoiled because grain was evaluated post-tone-mapping in narrow gamma-encoded sRGB, artificially amplifying shadow noise floors and clipping non-linearly. In real physical film, silver halide crystals and dye clouds react to incident scene photons in the optical domain before optical tone-mapping and print transfer. When re-evaluating grain down the roadmap, it must strictly execute inside the Wide-Gamut Linear Rec.2020 domain using physical sensitometric optical density ($D = \log_{10}(1/T)$) prior to the terminal display transform.

### 3.4 ColorSync Soft Proofing (P2)
- **Canvas Soft Proofing Mode**:
  - Canvas toolbar mode simulating sRGB and standard print gamut clipping (e.g., ISO Coated v2 / US Web Coated SWOP) on wide-gamut displays.
  - Gamut warning overlay highlighting out-of-gamut pixels in high-contrast overlay color.
- **Paper Substrate & Media Proofing (Decoupled from Develop Adjustments)**:
  - Substrate white-point reflectance and ink absorption limits (e.g. unbleached cotton rag matte, Hahnemühle Photo Rag, baryta surface sheen) simulate the physical print media viewing environment via LittleCMS2 ICC proofing transforms.
  - Enforces the architectural invariant that develop adjustments never apply additive digital floor lifts to simulate paper—true carbon black ($D_{\max}$) is always preserved at $f(0.0) \equiv 0.0$ for output print fidelity.

### 3.23 Physical Proof Run Calibration & Real-Paper Dmax Verification (P2)
- **Physical Test Batch Print Proofing**:
  - Execute a comprehensive lab print batch run on authentic fine-art substrates: unbleached cotton rag (*Hahnemühle Photo Rag 308*, *Canson Rag Photographique*), warmtone chlorobromide selenium, and baryta fiber-base.
  - Evaluate physical densitometry ($D_{\text{max}}$) under standard gallery illumination ($D_{50}$ / $D_{65}$) against digital screen rendering.
  - Benchmark shadow floor formulations across physical prints: compare physical reflective carbon ink depth vs emissive display floor lift (Option A: natural restrained floor $\sim 0.004$; Option B: micro-contrast compression without additive lift; Option C: current localized Hermite toe).
  - Tune paper grade and process depth default presets to guarantee 1:1 perceptual fidelity between darkroom viewport and physical framed gallery print.

### 3.24 Sensitometric EV Zone Histogram & Sensor Clipping HUD (P2 - Post v0.1)
- **Log₂ EV Stop Dynamic Range Scale**:
  - Horizontal axis calibrated in physical EV stops relative to 18% middle grey ($-8\text{ EV}$ to $+4\text{ EV}$) rather than an arbitrary 0–255 integer scale.
  - Evaluates linear scene radiance in Linear Rec.2020 prior to display tone-mapping, measuring true sensor exposure and optical dynamic range.
- **Ansel Adams Zone System Reference Overlay**:
  - Discrete, understated tick markers indicating Zones 0 through X (Zone 0 = pure print carbon black $D_{\max}$, Zone V = 18% middle grey, Zone VIII = textured high-value white, Zone X = specular clip).
- **Sensor Saturation vs. Display Roll-Off Telemetry**:
  - Distinct clipping sensors distinguishing between true hardware Bayer photosite saturation (blown raw sensor highlights) and AgX filmic shoulder compression (preserved highlights rolled off into display white).
- **Dual-Domain Performance Asymmetry & Decoupled Execution**:
  - Never runs in the fast 10–15ms burst culling or carousel navigation loop.
  - Evaluated asynchronously on a background worker or via GPU texture downsampling only when paused in the develop inspector or when toggled on demand via shortcut (`H`).
- **Quiet Darkroom Visuals**:
  - Monochrome tone density fill or muted desaturated channel traces rendered in dark slate (`#1E1E1E`); zero primary-colored mountain graphs or dancing waveforms.
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


### 3.8 Web Darkroom Subsystem Parity & Anti-Slop Initiative (P0 / P1) - COMPLETED
All sub-stories (3.8.1 through 3.8.7) completed and archived in [completed.md](completed.md).



---



## Phase 4: Pre-Release IP Audit, Code Cleanliness & Distribution (P0 / P1)

### 4.0 Comprehensive AI Slop Audit & Codebase De-Slop Pass (P0) - COMPLETED
All 7 lenses (Lenses 1 through 7) completed and archived in [completed.md](completed.md).

### 4.1 Open Source IP, Security & License Compliance Gate (P1)
- **License Compliance & Copyleft Gate (`cargo-deny`)**:
  - Whitelist permissive licenses (`MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `MPL-2.0`, `CC0-1.0`); hard-block copyleft `GPL` / `AGPL`.
  - Audit `rawler` (LGPL-2.1) to ensure standalone dynamic linking or clean-room replacement without copyleft contamination.
  - Standardize First Proof application code and Rust engine under the **MIT License**.
  - **Bundled Lensfun Database CC-BY-SA 3.0 Guardrail**:
    - Quarantines the bundled Lensfun lens calibration database under `assets/lensfun/` as a distinct dataset licensed under Creative Commons Attribution-ShareAlike 3.0 (CC-BY-SA 3.0).
    - As a collective work / mere aggregation, the database license does not affect First Proof's MIT application code.
    - Explicitly call out this dataset carve-out in root `LICENSE` and `THIRDPARTY.md`.
    - Provide visible in-app attribution under the Optics inspector tab lens profile selector (*"Lens profiles provided by the open-source Lensfun Project (CC-BY-SA 3.0)"*) and in the About/Attribution view.
- **Code Cleanliness & Provenance Scanning (`scancode` / `license-eye`)**:
  - Scan codebase for accidental copy-pasted GPL snippets, proprietary headers, or unlicensed third-party code.
  - Standardize SPDX license headers (`SPDX-License-Identifier: MIT`) across all Rust, Swift, and Metal files.
- **Secret & Credential Detection (`gitleaks`)**:
  - Pre-commit and CI scans to guarantee no private keys, signing certificates, local absolute paths, or API tokens exist in repository git history.
- **Dependency Vulnerability Auditing (`cargo-audit`)**:
  - Scan dependency graph against the RustSec Advisory Database for known CVEs and memory safety advisories.
- **Memory Safety & Undefined Behavior (`cargo miri`)**:
  - Validate unsafe FFI pointer conversions and raw byte slicing in `raw_decoder` and `export` modules under Miri.

### 4.2 Pre-Release Git History Scrubbing Pass (P2 - Pre-Push Gate)
- **Git History Rewrite & Scrubbing Pass (`git-filter-repo` / BFG)**:
  - *(Note: Active codebase path sanitization and test scratch cleanup completed; see completed.md)*
  - Perform an automated history rewrite prior to public release to scrub local paths, developer usernames, private emails, and transient binary blobs from git commit history.
  - Followed by aggressive reflog expiration and garbage collection (`git reflog expire --expire=now --all && git gc --prune=now --aggressive`).

### 4.3 Web Distribution & Release Site (`firstproof.photo`) (P1)
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

### 4.4 Standalone Desktop & CLI Release Packaging Pipeline (P1)
- **Release Packaging Milestone**:
  - Packaging the standalone native macOS application (compiling `libfirstproof_core.a`, running `uniffi-bindgen`, building the SwiftUI `.app` bundle, Developer ID code signing, and Apple `notarytool` stapling) and headless multi-arch `firstproof-cli` release binaries (`x86_64` and `aarch64` Linux and macOS).
  - Cross-platform GitHub Actions release runner compiling statically linked artifacts with LittleCMS2, shell completions (`bash`, `zsh`, `fish`), and man pages.
  - **Automated Hardware & Lens Calibration Harvester (`scripts/harvest_hardware_data.sh`)**:
    - Build a deterministic headless extraction script (`scripts/harvest_hardware_data.sh`) executed during release preparation or CI that pulls both camera profiles and lens correction data directly from authoritative upstream distributions:
      1. **Camera Calibration**: Pulls Adobe DNG Converter archive packages (extracted headlessly via `7z`) or RawTherapee `camconst.json`, parsing Adobe `.dcp` profiles and DNG tags (ForwardMatrix, Illuminant matrices, `BaselineSharpness`, `LinearResponseLimit`, and sensor black/white saturation margins) directly into `firstproof-engine::calibration::registry` or a compact binary table.
      2. **Bundled Lens Calibration (Lensfun DB)**: Ingests upstream Lensfun CC-BY-SA 3.0 XML database snapshots (`data/db/*.xml`), compiling and indexing them into a compact pre-indexed binary blob (`lensfun.bin` / JSON) embedded directly in native macOS desktop resources and WebAssembly assets.
    - **Deprecation of Legacy External Host Scanning**:
      - Deprecates the old runtime scanning of local Homebrew `/opt/homebrew/share/lensfun` and Linux `/usr/share/lensfun` paths.
      - Guarantees instant (< 2ms) optical distortion, TCA, and vignetting lookups operate 100% out of the box across desktop and browser darkrooms on day 1 without requiring user OS package installs.
    - Keeps both new camera and lens support a simple automated version bump away without requiring manual downloads or hand-crafted registry entries.
  - Native macOS `.app` bundle verification active on macOS workstation via `make bundle`.
  - Direct download links will be posted openly when binaries are ready—no email gates or signups.

### 4.5 Swift Desktop Parameter Schema & Behavioral Interaction Parity (P0)
- **Swift View & Model Schema Consumption**:
  - *(Note: Rust core UniFFI & WASM parameter schema bridge completed; see completed.md)*
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


### 4.7 Native macOS Desktop Archival DNG Ingest & Batch Action (P2)
- **Ingest Flow Integration (`ImportSheetView.swift`, `AppViewModel.swift`)**:
  - Add "Convert to Archival DNG on Ingest" toggle in the intake dialog.
  - Streams copied photos through `firstproof_core::dng::convert_raw_to_dng` with post-write disk read-back verification before adding to catalog SQLite.
- **Multi-Selection Batch Action**:
  - Context menu and `Photo -> Convert to Archival DNG` menu action on selected filmstrip thumbnails.
  - Smart filtering: Silently ignores JPEGs, PNGs, TIFFs, and already-DNG files; processes strictly proprietary camera RAWs (`.cr2`, `.nef`, `.arw`, etc.).
  - Non-blocking background worker with progress sheet and cancel safety.
- **Sidecar & Catalog Sync**:
  - Automatically updates catalog database records (`catalog.db`) pointing photo paths to the new `.dng` file without dropping user ratings, flags, or undo history.

### 4.8 Unified Shader Pipeline via WGSL & Naga Cross-Compilation (P1)
- **Problem Statement & Maintenance Friction**:
  - First Proof currently maintains identical color science and rendering algorithms across two divergent shader dialects: Metal Shading Language (`Shaders.metal`, `ColorScience.metal`, `Detail.metal`) for native macOS AppKit and WebGL 2 GLSL (`FragmentShader.ts`, `ColorScience.ts`, `Detail.ts`) for the web darkroom.
  - Every update to color science, print media, tone curves, or optical grain requires parallel manual implementations, constant synchronization, and defensive parity testing.
- **Single Source of Truth in WGSL (WebGPU Shading Language)**:
  - Migrate core fragment and compute shaders into canonical, strongly typed `.wgsl` source modules in `firstproof-engine/shaders/`.
  - Author all internal optical adjustments (Exposure, Bilateral Tone Recovery, White Balance, Hurter-Driffield curves, AgX filmic tone mapping, and print media) once.
- **Headless Naga Cross-Compilation Toolchain**:
  - Leverage `naga` (pure Rust shader compiler from the `wgpu` ecosystem) at build time:
    - Transpiles canonical WGSL $\to$ Metal Shading Language (`.metal`) for macOS desktop bundle compilation.
    - Transpiles canonical WGSL $\to$ GLSL 300 ES for WebGL 2 web darkroom runtime, with direct WGSL pass-through for WebGPU targets.
- **Deterministic Compile-Time Parity Verification**:
  - Build-time fitness assertions ensure generated MSL and GLSL expose bit-identical uniform buffer structures, scalar alignment, and coordinate math.
  - Eliminates human transcription error, divergent math, and manual multi-dialect shader maintenance permanently.

### 4.9 Robust Headless Command Bus, CLI Parity & Unified MCP Bridge (P1)
- **Problem Statement**:
  - First Proof possesses a high-performance CLI (`firstproof-cli` with `develop`, `inspect`, and `dng` subcommands) and an AppKit debug socket / MCP server (`tools/mcp/firstproof_mcp.py`), but the underlying command and state schemas are partially ad-hoc and decoupled from the web darkroom.
- **Canonical Command Schema in Shared Rust Engine (`firstproof-engine::command`)**:
  - Formalize a strongly typed, serializable JSON command enum (`DarkroomCommand`, `DarkroomAction`, `EditDelta`):
    - Viewport inspection (`GetState`, `SamplePixel`, `InspectMetadata`).
    - Non-destructive edit commands (`SetExposure`, `ApplyToneCurve`, `SetEmulsion`, `ResetWorkspace`).
    - Archival batch operations (`DevelopPhoto`, `ConvertToDng`, `BatchExport`).
- **Triple-Target Command Parity**:
  - **Native macOS Desktop**: AppKit debug socket (`/tmp/firstproof-debug.sock`) executes the canonical command enum directly, updating UI state without tearing.
  - **Headless CLI**: `firstproof-cli` accepts piped JSON command streams or flag-driven commands for headless studio automation, ingestion pipelines, and CI regression suites.
  - **Web Darkroom**: `firstproof-wasm` exposes `execute_command(json_str)` directly to JavaScript, enabling browser automation, test harness orchestration, and web extension integration.
- **Production MCP Server Refinement & Documentation**:
  - Elevate `firstproof_mcp.py` to a production-grade Model Context Protocol server exposing formal tools for darkroom automation, live diagnostics, and color parity verification.
  - Author complete documentation in `docs/architecture/cli_and_mcp.md` detailing CLI usage, socket IPC protocols, and MCP integration for photographers and studio engineers.

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

### 5.6 First Proof Mobile Capture App: Pure Optical Camera & Tactile Darkroom (Exploratory / Post-v2)
- **The Joy of Pure Capture on Mobile**:
  - Brings the quiet presence, tactile pleasure, and deliberate pace of manual camera craft to mobile devices: *"Pure capture. Honest color. Zero digital plastic."*
  - Direct hardware sensor stream: bypasses manufacturer ISP processing via low-level native camera APIs (`ImageFormat.RAW_SENSOR` / `RAW16` on Android `Camera2` & NDK, `kCVPixelFormatType_14Bayer_RGGB` on iOS `AVCapturePhotoOutput`).
  - Pure single-exposure capture: disables computational multi-frame fusion, sets `NOISE_REDUCTION_MODE = OFF`, `EDGE_MODE = OFF`, and `TONEMAP_MODE = OFF` / linear contrast curve, honoring the raw optical photon noise floor and authentic lens character.
- **The AgX Viewfinder (Real-Time 60fps GPU Pipeline)**:
  - Renders the live camera preview directly through First Proof's open-domain **AgX filmic tone mapper in Linear Rec.2020** wide-gamut color space at 60fps on mobile GPUs (Vulkan / OpenGL ES 3.0 on Android, Metal on iOS).
  - Highlights rolloff organically with natural photopic desaturation, preventing electric cyan/blue blowout on skies and specular highlights.
  - Live composition modes: compose directly in authentic chemical emulsions (e.g. Tri-X 400, Portra 400, Rochester Cinema 500) or historical print media (Baryta, Bamako Silver Contact, Cyanotype).
- **Tactile Manual Control Surface & Priority Exposure Modes (Leica/Hasselblad Ergonomics)**:
  - **Exposure Modes**: Full Manual (M), Aperture Priority (Av), Shutter Priority (Tv), and Program Auto (P) with prominent, weighted haptic Exposure Compensation dial ($\pm 3.0\text{ EV}$ in $1/3\text{ EV}$ increments).
  - **Auto-ISO with Minimum Shutter Floor**: Configurable minimum shutter speed ceiling ($1/125\text{s}$ or $1/\text{focal length}$) preventing motion blur before analog gain floats.
  - **Tactile Viewfinder Controls**: Weighted haptic dials for Shutter Speed ($1/32,000\text{s}$ to $30\text{s}$), native ISO analog gain, Kelvin White Balance, and Focus Distance (diopters with live high-contrast focus peaking and 100% split-prism zoom assist).
  - Quick hyperfocal snap shortcut for street photography.
- **Sensitometric Metering & The Zone System Assistant (Anti-Plastic Guardrail)**:
  - **Meter Modes**: Multi-Zone Evaluative, Center-Weighted, Spot, and **Highlight-Weighted Metering** (meters specifically to anchor brightest scene highlights just under `LinearResponseLimit`, letting AgX handle natural filmic shoulder roll-off).
  - **Zone System Telemetry**: Minimalist, non-intrusive live readout showing scene dynamic range distribution across Ansel Adams Zones 0–X.
  - **Strict Anti-Slop Invariant (Zero Synthetic Scene-Coloring AI)**: Hard rejection of consumer AI "scene styles" (no artificial sky saturation, no food warming, no synthetic skin smoothing, no hallucinatory filters). On-device intelligence functions strictly as an optical exposure meter and advisory tool, never painting over or second-guessing raw scene photons.
- **Archival Open DNG Output & Immutable Master Invariant**:
  - Writes standard Adobe DNG 1.4+ containers directly via First Proof's 4-stage atomic archival writer (`firstproof-core::dng`), embedding First Proof's authentic, un-baked retina preview (Linear Rec.2020 $\to$ AgX) in SubIFD1.
  - Edits, ratings, and capture metadata persist in standard `.xmp` sidecars (`photo.dng.xmp`), traveling seamlessly to the macOS desktop and web darkroom via local folder sync.
- **Instant Frictionless Master-Grade Sharing (Flagship Speed)**:
  - One-tap invocation of the native OS share sheet (`Intent.ACTION_SEND` on Android, `UIActivityViewController` on iOS) directly from the viewfinder review strip or photo inspect view.
  - Flagship mobile SoCs (Snapdragon 8 / Apple A-series / Google Tensor) leverage SIMD/NEON and GPU compute to crunch full-resolution Bayer RAW $\to$ Linear Rec.2020 $\to$ AgX filmic tone curve into an archival-quality JPEG in well under a second.
  - Exports a finished, wide-gamut sRGB or Display P3 JPEG with authentic tone curves and embedded ICC profiles, letting photographers share finished work instantly with zero tedious export micro-management.
- **Cross-Platform Shared Rust Engine**:
  - Core capture orchestration, DNG serialization, and color processing compiled directly from `firstproof-engine` via `cargo-ndk` (`aarch64-linux-android`) for Android and UniFFI for iOS.

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
