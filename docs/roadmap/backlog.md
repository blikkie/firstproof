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

## 5. Digital Camera Profiles (DCP), Camera Matching & 3D LUT Engine (P1)

Photographers often want the option to toggle between a pure neutral scientific baseline and the camera manufacturer's signature color science (e.g., Canon Picture Styles, Sony Creative Looks, Fujifilm Film Simulations):

*   **Adobe DCP (Digital Camera Profile) Architecture**:
    *   **Dual-Illuminant Matrix Interpolation**:
        DCP files store two separate color matrices calibrated under Standard Illuminant A ($2856\text{K}$ tungsten) and CIE Standard Illuminant D65 ($6504\text{K}$ daylight). During demosaicing, interpolate between both matrices based on the active Planckian white balance temperature ($1/T$).
    *   **HueSatMap (3D Hue-Saturation-Value Tables)**:
        Subtle 3D color remapping tables that selectively enhance memory colors (sky cyan-to-deep-blue shifts, warm portrait skin tone hue preservation, and foliage greens).
    *   **Camera-Matching Tone Curves**:
        Stores the manufacturer's exact non-linear S-curve to match the in-camera JPEG rendering with 100% fidelity.
*   **3D LUT Acceleration in Metal**:
    *   Parse standard `.cube` and `.dcp` 3D look-up tables (e.g. $33\times 33\times 33$ or $65\times 65\times 65$ lattices).
    *   Upload to Metal as a 3D Texture (`MTLTextureType3D` with hardware trilinear filtering).
    *   Sample in `Shaders.metal` with single-cycle trilinear texture sampling for real-time 120 FPS grading.
*   **Inspector Profile Selector**:
    *   Provide a dropdown in the Inspector under *Color & Tone*:
        *   **"LumenBox Neutral"** (Linear filmic with maximum grading headroom)
        *   **"Camera Standard"** (Matches camera JPEG rendering)
        *   **"Camera Portrait / Landscape / Monochrome"**
        *   **"Custom 3D LUT (.cube)..."** (Cinema & artistic film emulation)

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
        3. Applies geometric resampling and encoding (JPEG / 16-bit TIFF).
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
