# LumenBox Future Backlog & Roadmap

This document outlines the deferred features for LumenBox, detailing the math, logic, and implementation pathways so they can be picked up in future development phases.

---

## 1. Deferred Photo Editing Features (P1 & P2)

### Highlight & Shadow Recovery (P1)
Simple exposure sliders shift the entire luminance scale linearly. To recover highlights and boost shadows without clipping midtones, we need selective tonemapping:
*   **Simple Math**: Apply a parametric curve that adjusts tones based on distance from white (highlights) or black (shadows):
    \(L_{new} = L \cdot (1 + S_{Shadows} \cdot (1 - L)^2 - H_{Highlights} \cdot L^2)\)
*   **Bilateral Filtering (Advanced)**: To prevent halos (where dark lines form around bright boundaries), we will implement a fast bilateral filter on the GPU. The bilateral filter splits the image into a **base layer** (low-frequency details, containing large lighting transitions) and a **detail layer** (high-frequency details, texture). Highlight/shadow adjustments are applied *only* to the base layer, and then the detail layer is multiplied back.

### Auto-Crop & Inscribed Bounding Box on Straighten (P1)
When rotating/straightening an image by angle \(\theta\), black triangular boundary wedges appear at the image borders.
*   **Maximum Inscribed Aspect-Ratio Box**:
    To eliminate black border wedges seamlessly like Lightroom, compute the minimum zoom scaling factor \(S(\theta)\) required to inscribe the aspect ratio \(R = W / H\) inside the rotated canvas:
    \[S(\theta) = |\cos \theta| + |\sin \theta| \cdot \max(R, 1/R)\]
*   **Implementation**: In `Shaders.metal`, multiply UV coordinates by \(S(\theta)\) around \((0.5, 0.5)\) so the canvas automatically zooms to fill the entire frame without any black bars.

### Saturation & Vibrance (P1)
*   **Saturation**: Multiplies chrominance uniformly in HSL or CIELAB space.
*   **Vibrance**: Boosts saturation selectively. It targets less-saturated colors while protecting highly-saturated colors and skin tones (typically yellow/red/orange bands).
    *   *Implementation*: Convert RGB to HSV. Calculate saturation \(S\). Apply a boost multiplier \(M\) that decreases as \(S\) increases:
        \(S_{new} = S + M \cdot (1.0 - S) \cdot (1.0 - \text{is\_skin\_tone}(H))\)

### Interactive RGB & Tone Curves (P1)
Photographers rely on tone curves for surgical control over tonal contrast and stylized film grading:
*   **The Concept**: A 2D transfer function where \(X = \text{Input Luminance } [0.0, 1.0]\) and \(Y = \text{Output Luminance } [0.0, 1.0]\).
    *   **Diagonal (\(y = x\))**: Flat / neutral passthrough.
    *   **The Classic S-Curve**: Boosting upper midtones and pulling down lower midtones increases punchy micro-contrast while protecting pure black and specular white points.
    *   **The Matte / Film Fade**: Lifting the shadow black point from \((0, 0) \to (0, 0.1)\) turns harsh digital black into soft, vintage analogue shadows.
    *   **Independent RGB Channels**: Toning individual Red, Green, or Blue curves enables cinematic split-toning (e.g., adding blue/teal in the shadows while keeping highlights warm).
*   **Mathematical Implementation**:
    *   Interpolate user control points using a **Monotone Cubic Spline** (Fritsch-Carlson algorithm) to prevent overshoot/ringing.
    *   Bake the evaluated 256-sample curve into a 1D Metal texture (`MTLTextureType1D` with linear filtering) or evaluate directly in `Shaders.metal` with zero GPU latency.

### Sharpening, Detail Masking & Clarity Engine (P1)
Camera optical low-pass filters (OLPF) and 2x2 Bayer demosaic reconstruction inherently introduce subtle high-frequency smoothing. Camera internal JPEGs counteract this by baking in aggressive edge sharpening:
*   **Luminance Unsharp Masking (USM)**:
    Apply high-pass edge enhancement strictly to the perceptual Luminance (\(Y\)) channel to avoid color noise amplification:
    \[Y_{\text{sharp}} = Y + \text{Amount} \cdot (Y - \text{GaussianBlur}(Y, \sigma = \text{Radius}))\]
*   **Edge Masking & Thresholding (Sobel Gradient)**:
    To prevent sharpening background noise in flat skies or skin tones, calculate the local luminance gradient magnitude \(|\nabla Y| = \sqrt{G_x^2 + G_y^2}\). Sharpening gain is modulated by an edge weight:
    \[W_{\text{edge}} = \text{smoothstep}(\text{Threshold}, \text{Threshold} + 0.1, |\nabla Y|)\]
*   **Mid-tone Clarity (Local Contrast)**:
    A wide-radius (\(\sigma \approx 20\text{--}50\text{px}\)) unsharp mask that increases midtone local contrast, adding punch, definition, and tactile texture without harsh halos.
*   **Chrominance Denoising (Color Noise Reduction)**:
    A fast bilateral or median filter on \(Cb / Cr\) chroma channels that eliminates speckled color noise at high ISO while leaving luminance edge sharpness intact.

### Eyedropper & Selection Tools (P2)
*   **White Balance Picker**:
    1. The user clicks a point or small pixel region (e.g., 5x5 average) in the viewport.
    2. Read the RGB values of this region from the raw demosaiced texture buffer.
    3. Calculate the multipliers needed to make the red, green, and blue values equal:
       \(w_r = G_{avg} / R_{avg}, \quad w_b = G_{avg} / B_{avg}\)
    4. Apply these multipliers as the new white balance coefficients.
*   **White/Black Point Selectors**:
    *   Let users click absolute highlights/shadows. Map the selected pixel's luminance to 1.0 (white point) or 0.0 (black point) in the tone curve shader.

---

## 2. Google Photos Sync & Sharing (Post-v1)

### Google Pixel Computational DNG Ingestion
Google Pixel phones (Pixel 3 through Pixel 9 Pro) capture computational RAWs in Adobe DNG format (`PXL_YYYYMMDD_HHMMSS.dng` alongside `PXL_YYYYMMDD_HHMMSS.jpg`):
*   **Computational Linear DNG Format**:
    Pixel DNGs contain multi-frame HDR+ merged sensor data (9 to 15 burst frames combined in 16-bit linear space) with embedded lens shading gain maps and color calibration matrices.
*   **Takeout & Sync Ingestion**:
    *   **Google Takeout**: Takeout archives contain both `PXL_...dng` and `PXL_...jpg`, each with its paired `.json` sidecar. Our scanner automatically indexes the `.dng` file and links GPS, captions, and creation timestamps from the JSON metadata.
    *   **Google Photos API**: In API sync queries, identify `mimeType == "image/x-adobe-dng"` and download the uncompressed master DNG via `${baseUrl}=d` to preserve full computational sensor headroom.

### Syncing Phone Library to LumenBox
1.  **OAuth2 Authentication**: Integrate macOS authentication loop (ASWebAuthenticationSession) to obtain a Google Photos API token.
2.  **Background Sync Service**:
    *   Implement background polling in Rust that queries the Google Photos API (`mediaItems.list`).
    *   Verify if the item is already indexed in the SQLite DB (using Google's unique `id` and local filename checks).
    *   If missing, download the media item (Google provides a base URL with size specifiers, e.g., `=d` for original resolution) and place it in the user's managed directory.
3.  **Reverse Syncing (Sharing Back)**:
    *   Allow users to select photos and export them back to Google Photos using `mediaItems.batchCreate`.
    *   Pass the updated description, tags, and rating metadata.

---

## 3. Universal Non-Destructive Sidecar Architecture (JPEG, HEIC, PNG)

*   **The Non-Destructive Principle**:
    Standard photo viewers re-encode JPEGs on save, causing irreversible generation loss (JPEG compression artifacts, color quantization, and highlight/shadow clipping).
*   **LumenBox Sidecar Architecture**:
    *   Treat all source image formats (RAW, JPEG, HEIC, PNG, TIFF) as **immutable read-only master files**.
    *   Store all user adjustments (Exposure, WB, Contrast, Tone Curves, Crop/Straighten, Ratings, Pick/Reject flags, and EXIF/Takeout titles) in adjacent `.xmp` sidecars (`image.jpg.xmp` or `image.xmp`).
    *   **Benefits**:
        1. **Zero Quality Loss**: The original JPEG is never recompressed or degraded.
        2. **Infinite Undo & Repurposing**: Users can adjust edits years later or export in different aspect ratios, resolutions, or color grades on demand.
        3. **Interoperability**: Sidecars are fully compatible with Darktable, Lightroom, Adobe Bridge, and external archival systems.
        4. **Export Engine**: Generates a pristine, freshly rendered derivative image (JPEG, WebP, or 16-bit TIFF) only when exporting or sharing.

---

## 4. Native macOS Polish Pass (Desktop Experience)

*   **Activation Policy & Process Lifecycle**:
    Set `NSApplication.shared.setActivationPolicy(.regular)` on startup to guarantee standard macOS windowing behavior (`Cmd+Tab` app switcher, Dock tile icon, and native menu bar).
*   **Session Persistence**:
    Save and restore `lastOpenedDirectoryPath` in `UserDefaults` / `@AppStorage` so the app automatically re-opens the user's active library on launch.
*   **Window State & Free Resizing**:
    Enable `NSWindow` frame auto-saving (`window.setFrameAutosaveName`) and configure flexible min/max split pane constraints.
*   **App Bundle Packaging (`LumenBox.app`)**:
    Add `make bundle` target to package the Swift executable, `Info.plist`, Metal libraries, and `AppIcon.icns` into a distributable `LumenBox.app` bundle.

---

## 5. Unified Color Science & Camera Profiles Pipeline (P1)

Photographers demand both **scientific fidelity** (emulating camera manufacturer color profiles like Canon Picture Styles, Sony Creative Looks, and Nikon Picture Controls) and **analog filmic rendering** (natural highlight roll-off without digital clipping artifacts).

To prevent tone-curve collisions (double tone-mapping) and dynamic range clipping, we decouple camera sensor calibration from display-rendering transforms into a modular, **5-stage scene-referred color pipeline**:

```
                          [ RAW Sensor Linear ]
                                    │
                                    ▼
       [ 1. Dual-Illuminant DCP Calibration Matrix ] (Illuminant A / D65)
                                    │
                                    ▼
       [ 2. Wide-Gamut Linear Working Space ] (Linear Rec.2020 / ACEScg)
                                    │
                                    ▼
         [ 3. Scene-Referred Adjustments ] (Exposure EV, WB, Contrast)
                                    │
                  ┌─────────────────┴─────────────────┐
                  ▼                                   ▼
      [ Mode A: AgX Filmic (Default) ]    [ Mode B: Camera Matching ]
      • AgX Inset Matrix                  • DCP HueSatMap
      • Highlight Filmic Desaturation     • DCP Manufacturer Tone Curve
      • AgX Sigmoid Curve (Scene -> SDR)  • Camera Matrix to Display
                  │                                   │
                  └─────────────────┬─────────────────┘
                                    │
                                    ▼
            [ 4. Optional Post-Tone 3D LUT (.cube) ] (Display-Referred Creative Looks)
                                    │
                                    ▼
       [ 5. Display / EDR Target ] (Display P3, sRGB, or Ultra HDR Gain Map)
```

---

### Story 5.1: Linear Wide-Gamut Working Space & Pipeline Foundation
*   **The Foundation**: Prior to applying filmic tone mapping or DCP profiles, sensor data must transition through a standardized, wide-gamut scene-referred working space rather than ad-hoc sRGB clipping.
*   **Implementation**:
    *   Standardize on **Linear Rec.2020** (or ACEScg) with a D65 white point in both `lumenbox-core` (Rust) and `Shaders.metal` (Metal).
    *   Implement baseline matrix conversions: $\text{RAW Sensor} \to \text{CIE XYZ D50} \to \text{Linear Rec.2020}$.
    *   Execute Exposure (EV), White Balance, and linear Contrast adjustments strictly in this wide-gamut space before any display mapping.

---

### Story 5.2: AgX Filmic Tone Mapping Engine & Parity Guard
*   **The Problem: Digital Clipping & The Abney Effect**:
    *   When bright skin tones approach high luminance in naive digital pipelines, the red channel clips first to $1.0$. As green and blue rise, $(R, G, B)$ shifts toward yellow, creating unnatural "salmon" skin tones and plastic highlight clipping.
*   **The Solution: AgX Color Science**:
    *   **Purity / Inset Matrix**: Compresses wide-gamut linear colors into an inset working color space with a protective guard against negative and out-of-gamut values.
    *   **Hue Preservation & Filmic Desaturation**: As exposure reaches the shoulder, color vectors blend smoothly toward the achromatic luminance axis, producing clean, organic highlight roll-off.
    *   **Sigmoidal Tone Curve**: Smooth analog contrast without harsh digital knee boundaries.
*   **Strict CPU (Rust) & GPU (Metal) Parity**:
    *   **Live Viewport (Metal)**: Real-time 120 FPS evaluation in `Shaders.metal`.
    *   **Headless Export (Rust)**: High-quality evaluation in `lumenbox-core/src/export.rs` for batch exports.
    *   **Parity Test Guard**: An automated integration test in `lumenbox-core/tests/agx_shader_parity.rs` verifies shader constants and polynomial curves against the Rust CPU model to $\epsilon < 10^{-6}$.

---

### Story 5.3: Adobe DCP Ingestion & Dual-Illuminant Camera Calibration
*   **Camera-Specific Colorimetry**: Replace generic D65 matrices with exact camera sensor calibration extracted from Adobe `.dcp` profiles.
*   **Implementation**:
    *   **Rust DCP Parser**: Parse TIFF/DNG IFD structures in `lumenbox-core/src/dcp.rs` to extract `ColorMatrix1/2`, `ForwardMatrix1/2`, and `CalibrationIlluminant1/2`.
    *   **Dual-Illuminant Planckian Interpolation**: Dynamically interpolate between Standard Illuminant A ($2856\text{K}$ tungsten) and CIE Illuminant D65 ($6504\text{K}$ daylight) using reciprocal temperature ($1/T$) derived from the active White Balance slider.
    *   **Pipeline Integration**: Feed this dynamic calibration matrix directly into Story 5.1's scene-referred conversion stage, providing AgX (Story 5.2) with true camera-calibrated sensor data.

---

### Story 5.4: Camera-Matching Tone Profiles & DCP HueSatMap Engine
*   **The Feature**: Give photographers 1:1 parity with in-camera JPEGs (e.g. Canon Picture Styles, Sony Creative Looks, Nikon Picture Controls) alongside AgX Filmic.
*   **Modular Branching Transform**:
    *   **DCP HueSatMap**: 3D HSV delta tables that selectively enhance memory colors (sky blues, skin tones, foliage).
    *   **Camera-Matching Tone Curves**: Non-linear S-curves matching camera manufacturer rendering.
    *   **Inspector Profile Selector**:
        *   **"LumenBox Neutral (AgX Filmic)"** [Default]: Uses DCP sensor calibration $\to$ AgX Inset $\to$ AgX Highlight Desaturation & Sigmoidal curve. Bypasses the DCP `ToneCurve` to prevent double tone-mapping.
        *   **"Camera Standard"**: Uses DCP sensor calibration $\to$ DCP HueSatMap $\to$ DCP Manufacturer Tone Curve $\to$ Output sRGB.
        *   **"Camera Portrait / Landscape / Vivid"**: Swappable DCP profiles for alternate manufacturer looks.

---

### Story 5.5: 3D LUT Engine (.cube) & Headless SIMD Tetrahedral Interpolator
*   **Creative Film Simulation**: Support industry-standard `.cube` look-up tables (e.g., $17\times 17\times 17$, $33\times 33\times 33$, $65\times 65\times 65$) for cinema grading and artistic film emulation.
*   **Architectural Placement**:
    *   Evaluated in the **display-referred stage (post-tone-mapping)** with an opacity/mix slider ($0\%\text{--}100\%$). Applying LUTs post-tone-mapping ensures $[0, 1]$ creative LUTs do not hard-clamp high dynamic range sensor highlights.
*   **Metal & Rust Parity**:
    *   **Metal GPU**: Uploaded as a 3D texture (`MTLTextureType3D`) with single-cycle hardware trilinear texture filtering.
    *   **Rust CPU**: Implemented in `lumenbox-core/src/export.rs` with SIMD-accelerated tetrahedral or trilinear interpolation for deterministic headless batch exports.
*   **UI Integration**: Custom `.cube` file picker and blend slider in the Inspector under *Color & Tone*.

---

## 6. Parallel Multi-Core Bulk Export Engine (P1)

Photographers regularly need to batch export dozens or hundreds of culled photos after a shoot with zero UI stalling:

*   **Multi-Selection & Filtered Batch Export UI**:
    *   Support `Cmd+A` (Select All in Grid), `Shift+Click` (Contiguous Range), and `Cmd+Click` (Discontiguous Selection).
    *   Export Modal Sheet (`Cmd+Shift+E`): Option to export *Selected Photos*, *All Filtered Photos (e.g. 5-Star Picks)*, or the *Entire Catalog*.
*   **Rayon Multi-Core Thread Pool (Rust Core)**:
    *   Process exports in parallel across all Apple Silicon performance & efficiency cores using `rayon::par_iter`.
    *   Each worker thread autonomously:
        1. Reads the photo's individual `.xmp` sidecar for custom Exposure, WB, Contrast, Crop, and Lensfun parameters.
        2. Demosaics the 14-bit Bayer sensor data.
        3. Applies the configured color pipeline (AgX Filmic, Camera Matching DCP, or 3D LUT).
        4. Applies geometric resampling and encoding (JPEG / 16-bit TIFF).
*   **Background Export Queue & Progress Tracking**:
    *   Non-blocking background job with real-time SwiftUI progress bar (`Exporting 42 of 180 photos...`).
    *   Native macOS Notification Center banner upon batch completion with a "Show in Finder" action.
*   **Batch File Naming Templates**:
    *   Support dynamic token patterns: `{original_name}_edited.{ext}`, `{seq_3}_{YYYYMMDD}_{title}.{ext}`, or custom prefix/suffix rules.
    *   Target size presets: *Original Full-Resolution*, *Web 2048px*, *4K UHD 3840px*, or custom max dimension constraints.

---

## 7. Apple EDR & Ultra HDR (ISO 21496-1) Gain Map Pipeline (P1)

HDR photos are now the standard on the web, Google Photos, Instagram, and modern smartphone/laptop screens (iPhone Super Retina, MacBook Liquid Retina XDR):

*   **How Ultra HDR Works (Universal JPEG Compatibility)**:
    *   An Ultra HDR image is a standard, 100% backward-compatible `.jpg` file.
    *   **Base Image**: Standard SDR tone-mapped image (safe for legacy screens and fine art printing).
    *   **Embedded Gain Map (ISO 21496-1 / Adobe HDR)**: A compact secondary grayscale map embedded in the JPEG's metadata (MPF/XMP) storing the exact per-pixel gain multiplier:
        \[G(x, y) = \frac{\log_2(I_{\text{HDR}}(x, y) / I_{\text{SDR}}(x, y)) - \text{MinGain}}{\text{MaxGain} - \text{MinGain}}\]
    *   **Rendering**: SDR screens show the base image; HDR screens (Google Photos web, Instagram, Apple XDR) multiply the base image by the gain map to physically radiate specular highlights at up to 1,600 nits.
*   **Metal EDR Live Viewport (SwiftUI + Metal)**:
    *   Configure `MTKView` with `.rgba16Float` and `metalLayer.wantsExtendedDynamicRangeContent = true`.
    *   Query `NSScreen.main?.maximumExtendedDynamicRangeColorComponentValue` to dynamically read available display headroom ($1.0 \to 4.0+$).
    *   Add an **"HDR"** toggle badge in the Canvas toolbar to toggle between standard SDR preview and full 1,600-nit XDR radiance.
*   **Ultra HDR Export Option**:
    *   Add an **"Ultra HDR (Gain Map JPEG)"** checkbox in the Export Sheet (`Cmd+E`).
    *   Rust export engine generates the SDR baseline, computes the downsampled gain map, and embeds the ISO 21496-1 metadata tag.
*   **Google Photos Takeout & Ingestion**:
    *   Detect incoming Google Pixel & Google Photos Ultra HDR JPEGs and extract their gain map channels for instant high-dynamic-range playback.

---

## 8. Core Refactoring & Library Modernization Audit (P2 Tech Debt)

An architectural audit of existing `lumenbox-core` code revealed several areas where algorithms were hand-rolled from scratch rather than leveraging mature, community-tested Rust crates. While some hand-rolled code was intentionally defensive (e.g. avoiding LGPL C++ bindings), other areas represent maintenance surface area and should be modernized:

### Story 8.1: Migrate Hand-Rolled CIEDE2000 & Color Conversions to `palette`
*   **Current State**: `lumenbox-core/src/color/ciede2000.rs` (163 lines) implements ISO/CIE 11664-6:2014 CIEDE2000 perceptual difference math by hand, along with custom `Lab` and `Color3` conversions.
*   **The Modernization**:
    *   Adopt the **`palette`** crate (`palette = "0.7"`, MIT / Apache-2.0).
    *   `palette` natively supports SIMD-friendly `Lab`, `Xyz`, `Srgb`, `LinSrgb`, `Rec2020`, `Oklab`, and provides the `Difference` trait (`ciede2000`) out of the box with zero boilerplate.
    *   Eliminates 160+ lines of fragile trigonometric color difference math while guaranteeing compliance with official CIE test vectors.

### Story 8.2: Adopt `glam` for High-Performance SIMD Matrix & Vector Math
*   **Current State**: `lumenbox-core/src/color/mod.rs` implements a custom `struct Matrix3x3([[f32; 3]; 3])` with hand-rolled 3x3 cofactor inversion, matrix-matrix multiplication, and matrix-vector transforms.
*   **The Modernization**:
    *   Adopt **`glam`** (`glam = "0.27"`, MIT / Apache-2.0).
    *   Replaces hand-rolled matrix operations with `glam::Mat3` and `glam::Vec3`.
    *   Compiles to hardware NEON SIMD instructions on Apple Silicon ARM64, and shares a bit-identical 1:1 memory layout with Apple Metal's `simd_float3x3` and `simd_float3`.

### Story 8.3: Adopt `xmp-writer` for Robust Sidecar Generation
*   **Current State**: `lumenbox-core/src/xmp.rs` uses manual format strings and XML string concatenation to write RDF/XMP sidecar files.
*   **The Modernization**:
    *   Adopt **`xmp-writer`** (used by Typst, MIT / Apache-2.0).
    *   Provides a type-safe, validated builder pattern for standard Dublin Core (`dc:`), TIFF (`tiff:`), EXIF (`exif:`), and custom (`pm:`) namespace properties without risk of XML formatting regressions or escaping bugs.

### Story 8.4: Upgrade from Naive Bilinear Demosaicing to Directional (RCD / VNG)
*   **Current State**: `lumenbox-core/src/export.rs` uses simple 2x2 bilinear interpolation over the Bayer grid. Bilinear demosaicing is fast, but causes "zipper" edge artifacts, chromatic aberration false colors, and high-frequency resolution loss.
*   **The Modernization**:
    *   Adopt or port a permissive (MIT/BSD-licensed) directional demosaicer such as **RCD (Ratio Color Difference)** or **VNG (Variable Number of Gradients)** for high-quality batch exports, while retaining bilinear only for fast-path scratch previews.

### Story 8.5: Lensfun Database Licensing & Pure-Rust Engine Isolation
*   **Current State**: `lumenbox-core/src/lensfun.rs` (560 lines) parses Lensfun XML database files using `roxmltree`.
*   **The Audit Finding**: Hand-rolling this pure Rust XML parser was a necessary architectural defense to avoid linking against the official `liblensfun` C++ library (which is LGPL-3.0).
*   **Actionable Task**:
    *   Factor `lensfun.rs` into an internal standalone sub-crate (`lumenbox-lensfun`).
---

## 9. Procedural Organic Film Grain Simulation (P3)

Physical film grain simulation (distinct from pointwise 3D LUTs) models the stochastic, physical behavior of silver halide crystal clumping across photographic emulsions (e.g. Kodak Portra, Tri-X, CineStill):

*   **Non-Linear Luminance-Dependent Clumping**:
    Unlike digital sensor noise which dominates dark shadows, silver halide grain is an optical density phenomenon that is **most pronounced in the midtones (Zone V)**, diminishes in the shadows (where few crystals were exposed), and completely washes out in specular highlights (where the emulsion is fully saturated and opaque).
    \[G(Y) = 4 \cdot Y \cdot (1.0 - Y) \cdot \text{Amount}\]
*   **Tri-Layer Emulsion Chemistry**:
    Color negative film consists of three physical gelatin emulsion layers (Yellow, Magenta, Cyan). The top blue-sensitive/yellow layer has physically coarser silver halide crystals than the lower layers. The simulation provides per-channel stochastic weight to emulate natural chromatic grain rather than flat digital noise.
*   **Metal GPU & Rust Export Parity**:
    *   **Metal Viewport**: Evaluated in `Shaders.metal` using procedural high-frequency noise (blue noise or simplex noise texture) with zero runtime FPS cost.
    *   **Rust Export**: Deterministic evaluation in `export.rs` for full-resolution batch exports.
*   **Inspector Controls**:
    *   *Grain Amount* ($0\%\dots 100\%$)
    *   *Grain Size / Roughness* (35mm fine grain vs 16mm/Super-8 coarse grain)
    *   *Color Variation / Chroma Noise*

---

## 10. Third-Party Libraries & Licensing Guardrails

To preserve LumenBox's ability to be distributed as a proprietary, commercial macOS application on the Mac App Store and direct download, the project enforces strict IP and licensing guardrails:

| Technology / Component | Recommended Library | License | Licensing Risk & Policy |
| :--- | :--- | :--- | :--- |
| **Color Spaces & Metrics** | `palette` | MIT / Apache-2.0 | **Safe**. Fully permissive. Replaces hand-rolled CIEDE2000. |
| **Vector & Matrix Math** | `glam` | MIT / Apache-2.0 | **Safe**. High-speed SIMD math matching Metal shaders. |
| **ICC Profile Management** | `lcms2` / `lcms2-sys` | MIT | **Safe**. Safe bindings to LittleCMS2. Used by Safari/Firefox. |
| **Filmic Tone Mapping** | Internal AgX (Troy Sobotka) | CC0-1.0 / MIT | **Safe**. Standard public-domain affine matrices & polynomial sigmoid. |
| **RAW Unpacking** | `rawler` | MPL-2.0 / MIT | **Safe**. Pure Rust RAW container reader. |
| **EXIF Parsing** | `kamadak-exif` | BSD-2-Clause | **Safe**. Permissive EXIF metadata reader. |
| **XMP Writing** | `xmp-writer` | MIT / Apache-2.0 | **Safe**. Type-safe standard RDF/XMP generation. |
| **Darktable / RawTherapee** | *None (Reference Only)* | **GPL-3.0 (Strictly Prohibited)** | **High Risk**. Never copy or translate source code directly. Would trigger viral copyleft obligations on Swift/Rust codebase. |
| **Adobe ICC Profiles** | *Runtime LittleCMS generation* | Adobe Proprietary | **High Risk**. Do not copy `/Library/ColorSync/Profiles` or Adobe ICCs into the app bundle. Generate profiles programmatically via `lcms2` or bundle Elle Stone's CC-BY ICCs. |
| **Adobe DCP Camera Profiles** | Native `.dcp` parser (`dcp.rs`) | Adobe Open Specification | **Safe for Parsing**. Reading DNG/DCP metadata is permitted under Adobe's royalty-free patent grant. Do not bundle copyrighted creative DCP profiles; let users supply them. |
| **Compressed RAW Patents** | *Apple ImageIO Stills Only* | RED / Apple Patents | **High Risk**. Never attempt to implement reverse-engineered REDCODE or ProRes RAW decompression. |
| **Lensfun Calibration Data** | Pure-Rust XML Parser | CC-BY-SA 3.0 / LGPL-3.0 | **Medium Risk**. Keep calibration data decoupled from binary code; do not statically bake database into executable. |
