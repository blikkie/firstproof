# First Proof Active Backlog & Roadmap

This document strictly tracks pending and deferred features calibrated against our core philosophy: **"Pure capture. Honest color. Zero digital plastic. Print & web first."**

> **Note**: For completed stories, see [completed.md](completed.md). For color pipeline specifications, see [color_pipeline.md](../architecture/color_pipeline.md). For web architecture, see [web_architecture.md](../architecture/web_architecture.md).

---

## 🚀 Immediate Next Up (v0.1 Release Gate)
1. **Story 3.4: ColorSync Soft Proofing & Lab ICC Profile Management** (P2)
2. **Story 3.26: Global 3D LUT Library Management & Default Look Preset in Settings** (P2)
3. **Story 3.23: Commercial Fine-Art Lab Print Run (Printique) & Real-Paper Dmax Verification** (P2)

---

## Phase 3: Fine-Art Darkroom Chemistry, Archival Formats & Web Playground (P1 / P2)


> **Product Invariant: The Primary Engine vs. The Fine-Art "Treat"**  
> First Proof's primary workhorse default is **pure optical RAW capture via AgX filmic tone mapping**. The core workspaces (**Light**, **Color**, **Detail**, **Optics**) are dedicated strictly to sensor reality: standard White Balance (Temperature & Tint), exposure, tone recovery, and sensor sharpness. Photos look 90% finished immediately without any analog styling.  
> 
> Film stock and print media emulation are **not** the main workflow and **not** a complex simulator. They are an exquisite, 100% opt-in **fine-art treat and differentiator** housed in a dedicated **Emulsion** workspace tab (`InspectorTab.emulsion`). Default state is always `None / Bypass`. When activated, the engine does all the heavy lifting under the hood so the photographer simply selects a curated stock or print medium without fiddling with chemical dials.

### 3.4 ColorSync Soft Proofing & Lab ICC Profile Management (P2)
- **Canvas Soft Proofing Mode**:
  - Canvas toolbar mode simulating sRGB and standard print gamut clipping (e.g., ISO Coated v2 / US Web Coated SWOP, Printique / WhiteWall paper profiles) on wide-gamut displays.
  - Gamut warning overlay highlighting out-of-gamut pixels in high-contrast overlay color.
- **Paper Substrate & Media Proofing (Decoupled from Develop Adjustments)**:
  - Substrate white-point reflectance and ink absorption limits (e.g. unbleached cotton rag matte, Hahnemühle Photo Rag, baryta surface sheen, commercial Lustre / Deep Matte) simulate the physical print media viewing environment via LittleCMS2 ICC proofing transforms.
  - Enforces the architectural invariant that develop adjustments never apply additive digital floor lifts to simulate paper—true carbon black ($D_{\max}$) is always preserved at $f(0.0) \equiv 0.0$ for output print fidelity.
- **Lab & Output ICC Profile Management in Settings (`SettingsView.swift`)**:
  - Dedicated "Color & Proofing" section in Settings to manage installed printer and paper ICC profiles adhering strictly to native OS conventions.
  - On macOS, scans standard user and system ColorSync directories (`~/Library/ColorSync/Profiles/`, `/Library/ColorSync/Profiles/`).
  - Import custom print lab profiles (e.g. Printique, WhiteWall, or custom lab-calibrated sRGB/printer profiles) with 1-click import into `~/Library/ColorSync/Profiles/` (or platform equivalent, e.g. `/usr/share/color/icc/` on Linux).
  - Set a default proofing profile for rapid single-key toggle (`Cmd+Y` / toolbar button) during culling and editing.

### 3.23 Commercial Fine-Art Lab Print Run (Printique) & Real-Paper Dmax Verification (P2)
- **Practical Commercial Lab Print Batch (Printique / Standard Lab Offerings)**:
  - Instead of requiring unobtainable historical darkroom processes, execute a calibration test print run using accessible commercial fine-art lab options: e.g. Printique's Lustre, Glossy, Deep Matte, and Fine Art Cotton Rag (Hahnemühle Photo Rag).
  - Evaluate how First Proof's AgX filmic tone curve, film emulsions, and print media archetypes translate to real-world commercial photo papers under standard gallery illumination ($D_{50}$ / $D_{65}$).
  - Pair with Printique's downloadable ICC printer/paper profiles in Soft Proofing (Story 3.4) to benchmark on-screen soft-proof simulation against the physical delivered prints.
  - Benchmark shadow floor formulations across physical prints: verify that carbon black ink ($D_{\max}$) remains deep and rich without milky digital floor lifts or premature shadow crushing.
  - Tune paper grade and process depth default presets to guarantee 1:1 perceptual fidelity between darkroom viewport and physical framed gallery print.


### 3.26 Global 3D LUT Library Management & Default Look Preset in Settings (P2)
- **Account / App-Level 3D LUT Library (Native OS Conventions)**:
  - Elevate `.cube` LUTs from transient, per-photo imports into a persistent global LUT library honoring native OS filesystem conventions:
    - **macOS**: `~/Library/Application Support/FirstProof/LUTs/` with a native "Show in Finder" button.
    - **Linux / CLI**: `$XDG_DATA_HOME/firstproof/luts/` (or `~/.config/firstproof/luts/`).
  - All imported LUTs are permanently available across all photos, sessions, and catalog folders without needing to re-browse the filesystem.
- **LUT Management in Settings (`SettingsView.swift`)**:
  - Dedicated "Look & 3D LUTs" management pane in Settings allowing users to add, rename, preview, and remove custom 3D LUTs.
- **Default Signature Look Setting**:
  - Allow users to set a "Default Look LUT" (with customizable default intensity, e.g. 100% or subtle 40%) in Settings.
  - When enabled, newly imported photos or photos reset to default automatically inherit the photographer's signature LUT look, eliminating repetitive manual application across photo shoots.
- **Inspector UI Integration**:
  - Develop inspector's LUT dropdown automatically aggregates both First Proof built-in looks and all globally managed user LUTs, with an inline "+ Import..." button that automatically registers new LUTs into the persistent global library.
---





## Phase 5: Post-v1 & Long-Term Vision

### 5.1 First Proof Local Peer-to-Peer Sync Bridge & Cloud Liberation (Post-v1)
- **Google Photos Reality & Liberation via Takeout**:
  - Acknowledge Google Photos as a locked, read-restricted walled garden: official APIs forbid background library crawling and remote media deletion/trashing.
  - Liberation relies on the high-fidelity **Google Takeout Ingest Bridge** (Story 5): parsing JSON sidecars, resolving truncated filenames, and consolidating metadata into canonical `.xmp` sidecars on the local disk.
- **Interactive Google Photos Picker Ingest**:
  - On-demand modal import invoking the official Google Photos Picker API (`photospicker.mediaitems.readonly`), allowing the photographer to pull specific shoots, albums, or date ranges into First Proof.
  - Streams original binary assets directly to catalog storage via `baseUrl=d`, ingesting full in-camera embedded EXIF (shutter, aperture, ISO, lens) and native GPS directly into SQLite and canonical `.xmp` sidecars.
  - **Quiet Transcode / Storage-Saver Telemetry**: If Google returns an asset as a lossy JPEG rather than the camera's original RAW format (indicating Google "Storage Saver" downsampling), display a subtle, quiet badge in the import summary and metadata inspector (e.g., `Transcoded JPEG` in muted slate `#888888`), maintaining honest optical transparency without intrusive modal warnings.
- **Serverless Local Peer-to-Peer Mobile Bridge (Future First Proof Mobile Companion)**:
  - **Zero-Cloud, Zero-Hosting Invariant**: No intermediary relay servers, central databases, or cloud accounts. Transfers occur at line speed over the local network (LAN / Wi-Fi).
  - **Cryptographic Device Pairing (Same-User Authentication)**:
    - Desktop app generates an ephemeral QR code encoding a local Ed25519 public key fingerprint and LAN connection parameters.
    - Mobile app scans the QR code to establish mutual TLS (mTLS) with public key pinning saved in the system Keychain.
    - Local network discovery via Bonjour / mDNS (`_firstproof._tcp`); sync only proceeds if both endpoints mutually prove ownership of the paired keypair.
  - **Differential Sync**:
    - Streams new RAW captures from mobile camera roll to desktop catalog storage.
    - Synchronizes non-destructive culling state (star ratings `1`..`5`, pick/reject flags `P`/`X`) bidirectionally directly into catalog records and XMP sidecars without touching cloud storage.

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
