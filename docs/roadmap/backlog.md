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
