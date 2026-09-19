# First Proof Active Backlog & Roadmap

This document strictly tracks pending and deferred features calibrated against our core philosophy: **"Pure capture. Honest color. Zero digital plastic. Print & web first."**

> **Note**: For completed stories, see [completed.md](file:///Users/blikkie/src/LumenBox/docs/roadmap/completed.md). For color pipeline specifications, see [color_pipeline.md](file:///Users/blikkie/src/LumenBox/docs/architecture/color_pipeline.md).

---

## 🚀 Immediate Next Up
1. **Curated Fine-Art Print Media Archetypes: Color & Monochrome (P1)**: Optical print media (RA-4, Cibachrome, Silver Gelatin, Platinum) and spectral filtration as an optional fine-art treat.
2. **Upstream Recording Emulsions & Photochemical Grain Mechanics (P1)**: Non-linear film stock modeling, DIR chemical couplers, subtractive Beer-Lambert grain, and mock-LUT/standalone grain de-bloat.
3. **Archival Open DNG Converter & Lossless Compression Engine (P2)**: Lossless Bayer CFA transcoding and Deflate tile compression.
4. **Multi-Camera Color Calibration & Sensor Matching Engine (P2)**: Target chart profiling and spectral sensitivity matching.

---

## Phase 3: Fine-Art Darkroom Chemistry & Archival Formats (P1 / P2)

> **Product Invariant: The Primary Engine vs. The Fine-Art "Treat"**  
> First Proof's primary workhorse default is **pure optical RAW capture via AgX filmic tone mapping**. The core workspaces (**Light**, **Color**, **Detail**, **Optics**) are dedicated strictly to sensor reality: standard White Balance (Temperature & Tint), exposure, tone recovery, and sensor sharpness. Photos look 90% finished immediately without any analog styling.  
> 
> Film stock and print media emulation are **not** the main workflow and **not** a complex simulator. They are an exquisite, 100% opt-in **fine-art treat and differentiator** housed in a dedicated **Emulsion** workspace tab (`InspectorTab.emulsion`). Default state is always `None / Bypass`. When activated, the engine does all the heavy lifting under the hood so the photographer simply selects a curated stock or print medium without fiddling with chemical dials.

### 3.1 Curated Fine-Art Print Media Archetypes: Color & Monochrome (P1)
- **Downstream Darkroom Print Medium (The Fine-Art Treat)**:
  - Treats the captured frame (clean digital RAW or chemical emulsion) as an archival gallery print exposed onto authentic fine-art paper chemistry.
  - Quarantined inside the optional **Emulsion** tab; default is always clean digital display mapping.
- **Curated Color Print Media Archetypes**:
  - *Endura / Fujicolor Type-C (RA-4 Chromogenic)*: The classic gallery C-print standard. Creamy highlight roll-off, natural skin tone density, and organic cyan/magenta/yellow dye coupling that prevents digital clipping.
  - *Ilfochrome / Cibachrome Classic (Silver Dye-Bleach Reversal)*: Legendary high-gloss slide print process with pure azo dyes. Ultra-deep shadow density ($D_{\text{max}} > 3.0$), high micro-contrast, and glowing gem-like jewel tones.
  - *Archival Fine-Art Cotton Rag (Mineral Pigment)*: 100% unbleached cotton base, velvety matte shadow floor ($D_{\text{max}} \approx 1.65$), warm natural paper reflectance ($W_{\text{base}}$), and glare-free tonal breadth.
- **Curated Monochrome Print Media Archetypes & Toning**:
  - *Baryta Silver Gelatin FB*: Neutral barium sulfate ($\text{BaSO}_4$) base, deep $D_{\text{max}} \approx 2.2$, crisp midtone silver separation, and pure neutral silver blacks.
  - *Warmtone Chlorobromide (Selenium Toned)*: Subtle warm fiber base, deep velvety shadow D-max ($D_{\text{max}} \approx 2.3$) with cool plum-black depths, warm charcoal midtones, and soft highlights.
  - *Platinum / Palladium (Hand-Coated Cotton Rag)*: Noble metal embedded directly into cotton fibers. Long, delicate midtone graduation with non-clipping highlight roll-off ($D_{\text{max}} \approx 1.6$).
  - *Historic Cyanotype*: Authentic 1842 Prussian blue iron-salt spectral response and tonal scale.
- **Physical Enlarger Spectral Filtration (B&W Conversion)**:
  - When printing to monochrome media, replaces harsh digital desaturation with physical optical filtration:
    - *Neutral Panchromatic*: Standard wide-gamut linear luminance ($Y = 0.2627 R + 0.6780 G + 0.0593 B$).
    - *Yellow Filter (K2)*: Darkens blue skies subtly to emphasize clouds; clears facial haze.
    - *Orange Filter*: Moderate contrast, cuts atmospheric haze, smooths portraits.
    - *Red Filter (25A)*: High-contrast drama, near-black dramatic skies, architectural separation.
    - *Green Filter (X1)*: Lightens foliage and flora, separates skin tones under warm light.
    - *Orthochromatic (Vintage)*: Blind to red ($w_r = 0$), sensitive strictly to blue/green for historic 19th/early-20th century look.
- **Restrained "Rule of Three" Darkroom Controls (Zero Overload)**:
  - When an archetype is selected, it renders 90% finished immediately at default `0`. The user has exactly three high-leverage physical darkroom sliders:
    1. *Paper Grade / Hardness* (`-2.0` to `+2.0`, centered at `0` / Grade 2 Normal): Adjusts the slope and shoulder/toe steepness of the paper D-log H curve. Lower grades soften contrast to preserve delicate specular highlights; higher grades add graphic midtone punch.
    2. *Base Warmth* (`-100` to `+100`, centered at `0`): Subtle trim of the paper substrate reflectance ($W_{\text{base}}$)—from cool, crisp optical-brightened baryta to warm, unbleached natural cotton rag.
    3. *Process Depth* (`0%` to `100%`, default `100%`): Governs chemical toner bath immersion (untuned silver $\rightarrow$ deep archival selenium/platinum) for monochrome, or dye saturation depth for color.
  - *No Simulator Bloat*: Absolutely zero dichroic YMC dials, agitation rates, or 20-slider chemical chemistry panels.
- **Anti-Kitsch Guardrail**:
  - Absolutely zero fake torn paper edges, faux white frames, artificial canvas textures, or digital grunge. Strictly physical sensitometric D-log H paper curves, base paper spectral reflectance ($W_{\text{base}}$), and authentic darkroom toner chemistry.

### 3.2 Upstream Recording Emulsions & Photochemical Grain Mechanics (P1)
- **Upstream In-Camera Recording Medium**:
  - Models the photochemical behavior of the latent image on multi-layer camera negative/reversal and instant diffusion film prior to any print enlargement or display mapping.
- **Legacy Mock-LUT & Standalone Grain Rip-Out**:
  - *Mock-LUT Deprecation*: Purge hardcoded procedural pseudo-LUTs (`classic_chrome`, `warm_portra`, `cine_teal_orange`, `tri_x_bw`, `vintage_fade`) from `lut.rs`. Retain strictly the neutral `.cube` file parser for user-supplied technical calibration LUTs, cleanly separating technical looks from chemical darkroom science.
  - *Detail Panel De-Bloat*: Rip out standalone 2D additive noise sliders (`grainAmount`, `grainSize`, `grainRoughness`, `grainChroma`) from `DetailWorkspaceView.swift`, `Detail.metal`, and `pipeline.rs`. Detail workspace is restored to pure sensor optical acutance (Sharpening & Noise Reduction).
- **Photochemical Inter-Image Physics (DIR Couplers & Chemical Crosstalk)**:
  - Replace static LUT warping with active chemical cross-talk matrices ($3 \times 3$ $\mathbf{C}_{\text{DIR}}$) evaluated in linear Rec.2020 density space.
  - Models vertical diffusion of Development Inhibitor Releasing (DIR) couplers where intense exposure in one layer suppresses dye development in adjacent layers, creating dynamic color separation, glowing skin tones, and pastel highlight desaturation.
  - Formulated as a closed-form SIMD matrix transform across Metal GPU and Rust CPU pipelines (100% bit-parity, $O(1)$ per fragment, zero texture lookups, zero 60/120fps viewport latency penalty).
- **Subtractive Optical Density Grain Model (Beer-Lambert Absorption)**:
  - Transition from additive RGB noise (`color + grain`) to physical optical density transmission:
    $$T = 10^{-(D + \Delta D_{\text{grain}})}$$
  - Naturally integrates grain into the physical tone curve—dense in midtones, soft in highlights, and optically absorbed into deep shadow blacks.
- **Crystal Morphology & Dye Cloud Synthesis**:
  - *Tabular (T-Grain) vs. Cubic Halide*: Modern tabular planar crystals (high-frequency needle grain) vs. classic organic multi-octave cubic silver clumping.
  - *Colloidal Dye Clouds vs. Opaque Filaments*: C-41/E-6 renders soft overlapping dye clouds (bleached silver simulation); B&W renders crisp metallic silver filaments.
  - *Lateral Dye Diffusion*: Instant film models dye migration through aqueous gelatin spacing layers rather than metallic halide clumping.
  - *Physical Format Scaling*: Grain scale is driven by authentic photographic format (35mm, 120 6×7, 4×5 Large Format) rather than arbitrary digital sliders.
- **Curated Geographic Archetypes (Historic Plant Homage & Sensitometric Grounding)**:
  - Grounded strictly in published manufacturer sensitometric data sheets (characteristic D-log H curves, spectral dye density, and layer sensitivity curves):
    - *Rochester Portrait 400* (C-41, Kodak Park, Rochester NY): Gentle portrait contrast, luminous skin tone separation via DIR couplers, pastel highlight roll-off.
    - *Ashigara Chrome 50* (E-6 Reversal, Minamiashigara, Japan): High micro-contrast, rich saturated primaries with clean neutral shadows, inspired by slide chemistry.
    - *Rochester K-14 Chrome 64* (K-14 Non-Substantive Reversal, Rochester NY): Ultra-deep ink-black D-max ($D_{\text{max}} > 3.2$), vivid warm red/yellow separation, and razor edge acutance modeled from Kodak Pub E-88.
    - *Rochester Cinema 500* (ECN-2 Tungsten, Rochester NY): Low-contrast linear negative with gentle highlight compression designed for wide-gamut digital grading.
    - *Rochester Daylight 200* (C-41, Rochester NY): Warm, earthy golden-hour daylight response with restrained blue density.
    - *Mobberley Silver 400* (Panchromatic B&W, Mobberley, Cheshire, UK): Classic silver halide negative with linear midtone acutance and cubic crystal clumping.
    - *Cambridge Diffusion 100* (Instant Dye Diffusion, Kendall Square, Cambridge MA): Reversal dye-developer immobilization with reflective $\text{TiO}_2$ base tone, luminous pastel greens, and soft dye diffusion acutance (Ansel Adams / Edwin Land lineage; strictly borderless and pure capture).
- **Curated Restraint & Anti-Gimmick Invariants**:
  - *Strict Neutral Default*: Emulsion profiles are 100% opt-in creative tools. The pipeline default is always unadorned, neutral optical sensor capture (Camera DCP + AgX).
  - *Anti-Preset Restraint*: Strictly bounded to 7 foundational, deeply researched chemical archetypes across the major 20th-century processes. Zero bloated "preset packs" or arbitrary digital styling.
  - *Anti-Kitsch Guardrail*: Absolutely zero faux borders, white frames, fake light leaks, or artificial dust/scratches. All archetypes are pure optical and chemical image transformations.

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
  - Cleanly separate general optical RAW demosaicing precision from physical 3D film simulation in `ExportSheetView.swift`:
    1. *RAW Demosaicing Precision*: `Standard (Fast)` vs. `Archival (Edge-Adaptive AMaZE / RCD)`.
    2. *Volumetric 3D Emulsion Simulation*: A distinct, dedicated toggle (active when a film stock or print medium is applied).
- **Control 1: Multi-Pass Edge-Directed Demosaicing (AMaZE / RCD)**:
  - *Standard (Fast)*: Near-instant batch exports (150–250ms per RAW) using optimized cubic Bayer demosaicing.
  - *Archival (Edge-Adaptive)*: Directional edge-adaptive demosaicing (AMaZE / RCD) with chromatic moiré suppression passes (~500ms per RAW). Eliminates "zipper" artifacts, maze patterns, and false color fringing on high-frequency fabrics, architectural grids, and bird feathers on 45–61MP sensors.
- **Control 2: Deep 3D Volumetric Film Simulation (Distinct Toggle)**:
  - *Offline Rayon Simulation*: When enabled on photos with active emulsion/print media, the engine switches from the real-time analytical projection to full numerical physics:
    - *Volumetric Gelatin Depth-Integration*: Multi-slice Beer-Lambert ray transmission through a 3D Boolean Random Field of crystal populations or colloidal dye cloud spheres ($z \in [0, z_{\text{emulsion}}]$). Captures inter-crystal 3D shadowing, turbidity (optical depth scattering), and exposure-dependent volumetric clumping.
    - *Spatial Lateral Chemical Diffusion*: Extends vertical DIR coupling to spatial inhibitor diffusion across contrasting borders, recreating authentic photochemical Mackie lines / developer exhaustion boundaries for edge acutance without digital unsharp masking.
    - *Sub-Pixel Jittered Supersampling*: 2×2 jittered sub-pixel integration across the volumetric emulsion layer, preventing spatial beat frequencies (aliasing) on 300+ DPI large-format archival prints.
  - *Independence*: Photographers can export high-quality edge-demosaiced files with fast analytical film grain (fast batch turnaround), or opt into the full 3D volumetric simulation when preparing exhibition masters.
- **Strict WYSIWYG Tonal & Color Parity Invariant**:
  - The live 120fps viewport shader remains the first-order analytical integral of the deep 3D simulation. Viewport and archival export match bit-for-bit in global tone curve, AgX mapping, color separation, and White Balance.
- **Dignified UI Simplicity**:
  - Clean, uncluttered layout in the export sheet: one segmented picker for demosaicing quality, and one clear toggle for 3D volumetric simulation when an emulsion is active. Zero slider bloat or esoteric algorithm lists.

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


