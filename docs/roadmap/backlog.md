# First Proof Active Backlog & Roadmap

This document strictly tracks pending and deferred features calibrated against our core philosophy: **"Pure capture. Honest color. Zero digital plastic. Print & web first."**

> **Note**: For completed stories, see [completed.md](file:///Users/blikkie/src/LumenBox/docs/roadmap/completed.md). For color pipeline specifications, see [color_pipeline.md](file:///Users/blikkie/src/LumenBox/docs/architecture/color_pipeline.md).

---

## 🚀 Immediate Next Up
1. **Fine-Art Monochromatic Darkroom & Spectral Filtration Engine (P1)**: Physical lens filtration and archival print toning (selenium, sepia, platinum) in Linear Rec.2020.
2. **Archival Open DNG Converter & Lossless Compression Engine (P1)**: Lossless Bayer CFA transcoding and Deflate tile compression.
3. **Chemical Emulsion Profiles (C-41, E-6 & ECN-2 Chemistry) (P2)**: Non-linear film stock modeling with subtractive dye layer response.

---

## Phase 3: Fine-Art Darkroom Chemistry & Archival Formats (P1 / P2)

### 3.1 Fine-Art Monochromatic Darkroom & Spectral Filtration Engine (P1)
- **Spectral Filtration in Linear Rec.2020 (Physical Lens Filters)**:
  - Replace harsh digital desaturation with physical optical filtration:
    - *Neutral Panchromatic*: Standard wide-gamut linear luminance ($Y = 0.2627 R + 0.6780 G + 0.0593 B$).
    - *Yellow Filter (K2)*: Darkens blue skies subtly to emphasize clouds; clears facial haze.
    - *Orange Filter*: Moderate contrast, cuts atmospheric haze, smooths portraits.
    - *Red Filter (25A)*: High-contrast drama, near-black dramatic skies, architectural separation.
    - *Green Filter (X1)*: Lightens foliage and flora, separates skin tones under warm light.
    - *Orthochromatic (Vintage)*: Blind to red ($w_r = 0$), sensitive strictly to blue/green for historic 19th/early-20th century look.
- **Silver Halide Emulsion Characteristic Curve (Smooth Tonal Roll-Off)**:
  - Model the continuous logarithmic shoulder of analog silver gelatin paper (Hurter & Driffield / D-log H curve) through our AgX tonemapper, preventing harsh digital highlight blowout and preserving velvety shadow detail without muddy midtones.
- **Archival Chemical Print Toning**:
  - *Pure Silver Gelatin*: Achromatic neutral silver.
  - *Selenium Toning*: Increases shadow D-max with subtle cool plum-black tones while keeping highlights clean silver-white.
  - *Warm Platinum*: Subtle warm paper base tone with soft, luminous highlights.
  - *Architecture Reusability*: Directly reuses the bit-parity split toning engine (`apply_split_toning`, `applySplitToning`) and interactive 2D color wheels (`ColorWheelView.swift`) implemented in Story 2.2, applied specifically across the achromatic silver density curve where chemical toning authentically belongs.
- **Anti-Slop Invariant**:
  - Strictly grounded in optical physics and chemical darkroom print making. Zero arbitrary digital "filter presets" or crushed-black Instagram effects.

### 3.2 Archival Open DNG Converter & Lossless Compression Engine (P1)
- **Lossless Bayer CFA Transcoding**:
  - Convert proprietary camera RAWs (`.cr2`, `.nef`, `.arw`, `.rw2`) into open standard Adobe DNG (1.4+ / TIFF-EP).
  - Encapsulate raw Bayer mosaic unaltered, preserving exact sensor CFA layout, active sensor crop, black/white saturation levels, and daylight/tungsten color calibration matrices.
- **Deflate (ZIP) Tile Compression**:
  - Apply lossless Deflate / ZIP tile compression (`flate2`) across raw sensor data, reclaiming 20–35% disk storage with mathematically identical (zero-loss) reconstruction.
- **Bit-for-Bit Verification Safeguard**:
  - Validate decompressed CFA sensor payload against the original proprietary RAW via SHA-256 / CRC32 before committing conversion, ensuring zero risk of archive corruption.
- **Metadata & Sidecar Integration**:
  - Embed existing First Proof XMP edits, color adjustments, IPTC tags, and ratings into the DNG IFDs, or preserve adjacent `.xmp` according to catalog preferences.

### 3.3 Chemical Emulsion Profiles (C-41, E-6 & ECN-2 Chemistry) (P2)
- **Spectral Inter-Layer Inhibition (DIR Coupler Physics)**:
  - Model Development Inhibitor Releasing (DIR) couplers via 3×3 spectral cross-talk matrices directly in linear Rec.2020 to produce luminous, non-clashing skin tones and soft highlight desaturation.
- **Physical Chemistry Archetypes (Trademark-Clean)**:
  - *CN-400 Portrait (C-41)*: Gentle contrast, luminous skin tone separation via inter-layer dye coupling, soft pastel highlight roll-off.
  - *CR-50 Vivid Chrome (E-6 Reversal)*: High micro-contrast, rich saturated primaries with clean neutral shadows, inspired by slide chemistry.
  - *ECN-500 Cinema (ECN-2)*: Low-contrast linear negative with gentle highlight compression designed for wide-gamut digital grading.
  - *Gold-200 Daylight*: Warm, earthy golden-hour daylight response with restrained blue density.
- **Organic Synergy with Procedural Grain**:
  - Bind each chemical profile directly to its corresponding physically-modeled `FilmGrainPreset` (e.g. `CN-400` paired with `P-400 Portrait` grain) for coherent, integrated analog physics.

### 3.4 ColorSync Soft Proofing (P2)
- **Canvas Soft Proofing Mode**:
  - Canvas toolbar mode simulating sRGB and standard print gamut clipping (e.g., ISO Coated v2 / US Web Coated SWOP) on wide-gamut displays.
  - Gamut warning overlay highlighting out-of-gamut pixels in high-contrast overlay color.

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
  - Unadorned native macOS application screenshots showing the actual darkroom interface and workflow.
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
- **Direct, Quiet Download**:
  - Simple Apple Silicon `.dmg` / `.zip` download link with SHA-256 checksums and clear macOS 14+ (Apple Silicon) compatibility requirements.

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

### 5.4 Offline Search, Time-Proximity Geotagging & Spatial Discovery (Post-v1 / Post-v2)
- **Time-Proximity Geotagging ("Borrow GPS from Mobile")**:
  - Correlate GPS coordinates from companion mobile photos within an adjustable time window (e.g. $\pm 30$–$60$ min) for camera RAWs lacking onboard GPS hardware.
  - Linear spatial trajectory interpolation when a RAW shot is bracketed between two mobile GPS locations.
  - Camera clock drift calibration slider (`±MM:SS`) with reference photo alignment.
  - Transparent metadata auditing: Distinguish exact hardware coordinates from approximate inferred locations (`FIRSTPROOF_TIME_PROXIMITY_INFERRED` in XMP sidecars and muted pin badge in inspector).
- **Invisible Concrete Object Search (Apple Neural Engine `VNClassifyImageRequest`)**:
  - Sub-millisecond background classification on Apple Silicon during folder index; categorizes 1,300+ concrete objects (e.g. "dog", "bicycle", "boat", "car", "mountain") into SQLite `photo_classifications`.
  - Zero UI clutter: no bounding boxes or tags drawn on the artist's canvas; full-text search bar returns matching photos instantly.
- **Restrained Archival MapKit Bounding Box Search**:
  - Muted, dark-slate native macOS MapKit canvas matching First Proof's `#1E1E1E` darkroom palette.
  - Dynamic viewport bounding-box synchronization: panning/zooming the map instantly updates the contact sheet grid.
- **Archival Portrait Clustering (MWG/IPTC Standard Regions)**:
  - 100% offline, on-device portrait face clustering via Apple `Vision` framework.
  - Unidentified sitter triage deck: *"Found X photos of this person. Who is this sitter?"*
  - Serializes coordinates directly into open-standard [MWG (Metadata Working Group) / IPTC XMP Face Regions](file:///Users/blikkie/src/LumenBox/docs/values.md#L43) (`mwg-rs:Regions`), avoiding proprietary database lock-in.

### 5.5 Optical Notch & Spectral Illuminant Modeling (Deep Backlog / Exploratory R&D)
- **Physical Context & The Multi-Source Problem**:
  - Gas-discharge lighting (low/high-pressure sodium streetlamps, industrial mercury/fluorescent tubes) produces discrete atomic line emission spikes (e.g., $589\,\text{nm}$ sodium doublet, $546\,\text{nm}$ mercury green) rather than continuous thermal Planckian radiation.
  - In real-world mixed-source captures (e.g., subway platforms combining tungsten, fluorescent tubes, and ambient exterior light), automatic illuminant deconvolution is mathematically ill-posed and prone to false color artifacts.
- **Exploratory Darkroom & Optical Emulations (Preserving Anti-Slop Invariants)**:
  - **Virtual Optical Notch Filtering**: Emulate physical chemical glass filters (such as didymium/neodymium glass that sharply absorbs the $580\text{–}590\,\text{nm}$ sodium doublet, or analog FL-D/FL-W magenta glass) to selectively suppress line-emission spikes without distorting continuous adjacent spectral channels.
  - **Targeted Hue-Band De-Spiking**: Parametric narrow-band chromaticity qualifiers in the color engine that can gently attenuate saturated line-emission spikes toward natural warm amber/tungsten, anchored to known physical emission wavelengths.
  - **Strict Anti-Slop Guardrail**: Never attempt synthetic multi-source AI auto-hallucination. Any future tool must remain a conscious, physical optical emulation with predictable, manual photographer control.

