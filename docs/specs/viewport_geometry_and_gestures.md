# Mini Spec: Viewport Geometry, Gestures & Texture Lifecycle

## Status: Approved Architecture Specification
**Author**: Remco van Bree  
**Scope**: `FirstProofApp` (macOS Swift/Metal), `web` (TypeScript / WebGL 2)

---

## 1. Executive Summary & Core Invariants

The darkroom viewport canvas is responsible for real-time presentation, pan/zoom navigation, and interactive geometric manipulations (crop, straighten, rotation, orientation flips). 

To ensure identical behavior across macOS native Metal and WebGL, three core invariants govern this subsystem:
1. **GPU-Accelerated Coordinate Projection**: Never mutate vertex buffer geometry on the CPU. Viewport scaling, crop zooming, straighten rotation, and orientation transforms are executed entirely within vertex and fragment shaders.
2. **Stationary Full-Canvas Gesture Container**: Never attach drag gestures or event monitors to dynamically repositioned or animated child view elements. Gestures are captured by a stationary container covering the entire viewport, resolving hit targets at `startLocation`.
3. **Intermediate Texture Bypass & Idempotence**: Rendered high-resolution intermediate textures have geometry and tone curves already baked in. Shaders must bypass vertex rotation and fragment color transforms when `isIntermediate == 1`.

---

## 2. The 4 Viewport Coordinate Spaces

All interactive tools and shaders translate coordinates across four well-defined spaces:

```
[Sensor RAW Frame] ──(Demosaic / Bayer Bin)──► [Normalized Image UV]
   (0 to W_raw, 0 to H_raw)                       ([0.0, 1.0] x [0.0, 1.0])
                                                           │
                                                           ▼ (transformGeometry)
[GPU Clip Space] ◄──(Projection / Scale)────── [Viewport Canvas Points]
   ([-1.0, 1.0] x [-1.0, 1.0])                   (0 to W_canvas, 0 to H_canvas)
```

1. **Sensor RAW Frame**: Integer pixel dimensions ($W_{\text{raw}} \times H_{\text{raw}}$) defined in sensor metadata. Represents immutable physical photosites.
2. **Normalized Image UV Space**: Continuous coordinate frame $[0.0, 1.0] \times [0.0, 1.0]$ where $(0.0, 0.0)$ is top-left and $(1.0, 1.0)$ is bottom-right of the unrotated sensor image. All edit parameters (`crop_left`, `crop_top`, `crop_right`, `crop_bottom`) are stored in this space.
3. **Viewport Canvas Points**: Logical screen points (macOS AppKit points / Web CSS pixels) of the rendering container. Computes aspect-ratio containment (letterboxing/pillarboxing) relative to window dimensions.
4. **GPU Clip Space**: Normalized device coordinates $[-1.0, 1.0] \times [-1.0, 1.0]$ for the full-screen vertex quad.

---

## 3. Shader Geometry Transformation (`transformGeometry`)

In both `Geometry.metal` and `WebGLRenderer.ts`, vertex/fragment coordinates are projected via a single analytical function:

```
Inputs:
- uv: Original fragment texture coordinate [0.0, 1.0]
- straightenAngle: Rotation angle in radians
- cropRect: [cropLeft, cropTop, cropRight, cropBottom] in normalized UV
- cropModeActive: Boolean flag (1 = user is editing crop; 0 = user is viewing final photo)
- zoomLevel, panOffset: Viewport navigation coordinates
```

### The Transform Pipeline
1. **Straighten Rotation**: Rotate coordinates around the center point $(0.5, 0.5)$:
   $$\begin{pmatrix} u' \\ v' \end{pmatrix} = \begin{pmatrix} \cos\theta & -\sin\theta \\ \sin\theta & \cos\theta \end{pmatrix} \begin{pmatrix} u - 0.5 \\ v - 0.5 \end{pmatrix} + \begin{pmatrix} 0.5 \\ 0.5 \end{pmatrix}$$
2. **Crop Inscription**:
   - When `cropModeActive == 0` (normal viewing mode), the cropped bounding box $[u_{\text{left}}, u_{\text{top}}, u_{\text{right}}, u_{\text{bottom}}]$ is mapped to fill the entire canvas:
     $$u'' = u_{\text{left}} + u' \cdot (u_{\text{right}} - u_{\text{left}})$$
     $$v'' = u_{\text{top}} + v' \cdot (u_{\text{bottom}} - u_{\text{top}})$$
   - When `cropModeActive == 1` (interactive crop tool mode), the full image remains visible, dimmed outside the crop box by a dark scrim ($70\%$ opacity).
3. **Zoom & Pan Offset**:
   - Modulates coordinates by `zoomLevel` anchored at the cursor/tap point.

---

## 4. Stationary Full-Canvas Gesture Architecture

### The Anti-Pattern: Moving Interactive Handles
In UI frameworks (SwiftUI / AppKit / Web DOM), attaching `DragGesture` or pointer event handlers to dynamically repositioned child views (e.g. `<div style="left: ${x}px">` or `.position(point)`) causes severe tracking failures:
- Moving the view under the pointer causes the window server to drop pointer tracking or fire cancellation events (`pointercancel` / `.onEnded` silently dropped).
- Results in dropped undo records and "stuck" dragging states.

### The Correct Architecture: The Stationary Container
```
┌────────────────────────────────────────────────────────┐
│  STATIONARY GESTURE CONTAINER (Full Viewport Canvas)   │
│  - Captures onPointerDown / DragGesture at (x, y)      │
│  - Evaluates hit target once at startLocation:         │
│    • Distance to Corners (within 16px) ──► Resize Corner│
│    • Distance to Edges (within 8px)   ──► Resize Edge  │
│    • Inside Crop Rect                ──► Pan Crop Box │
│    • Outside Crop Rect               ──► Straighten   │
│                                                        │
│  ┌─────────────────────────┐                           │
│  │ Active Crop Rect        │ (Pure visual elements     │
│  │ (allowsHitTesting:false)│  no event monitors)       │
│  └─────────────────────────┘                           │
└────────────────────────────────────────────────────────┘
```

### Outside-Crop Drag: Straighten Rotation with HUD
Dragging outside the active crop box must **never** create a new crop box (which destroys the user's carefully adjusted framing). Instead, dragging outside initiates **straighten rotation**:
- Pointer movement tangential to the center rotates the image.
- A stationary HUD readout appears at the bottom center displaying the exact rotation angle: `+1.4°` with a reset indicator.
- Pressing <kbd>Escape</kbd> reverts the straighten angle without pushing an undo step.

### Portrait Aspect Ratio Inversion (<kbd>X</kbd> Shortcut)
When a photo is portrait ($H > W$) or the user presses <kbd>X</kbd>:
- The aspect ratio constraint ($W / H$) inverts automatically: e.g. $3:2 \to 2:3$, $4:3 \to 3:4$.
- The crop container clamps within image boundaries without collapsing to zero width.

---

## 5. Intermediate Preview & Texture Lifecycle

```
[RAW File Selection]
        │
        ▼ (Stage 0: Up-Front Edit Staging)
[Resolve EXIF Orientation & XMP Edits from Disk]
        │
        ▼ (Atomic Canvas Transition: 10-15ms)
[Dark Neutral Canvas with Developing Spinner] (Prevents stale pixel flash)
        │
        ▼ (2x2 Bayer Box Binning Engine)
[16-bit Linear Rec.2020 Binned Frame]
        │
        ▼ (Metal / WebGL Render)
[High-Res Intermediate Rendered Image] ──► Cached in IntermediateTextureCache
```

### Invariant 1: Intermediate Bypass & Idempotence
When the user switches back to a photo that has a cached high-resolution intermediate (`intermediateImage` / `renderThumbnailSnapshot`):
- High-res intermediates **already have crop, rotation, exposure, and AgX tone-mapping baked into their pixels**.
- When uploaded to the GPU canvas:
  ```metal
  if (uniforms.isIntermediate == 1) {
      // Bypass vertex rotation and crop projection (use identity UV)
      // Bypass fragment color transforms (sample sampledColor directly)
      return sampledColor;
  }
  ```
- **Rule**: Never pass an intermediate texture through AgX or geometry transforms a second time.

### Invariant 2: Zero Camera-JPEG Flashing
- The viewport canvas must **never** display or flash the camera manufacturer's baked JPEG (`extractPreview`).
- RAW selection streams directly into the 10–15ms Bayer box binning engine (`decodeRawPreviewBinned`).
- Intermediate lookups strictly query First Proof's authentic Linear Rec.2020 intermediate textures, preserving authentic color from frame 0.

### Invariant 3: Up-Front Edit Staging & Clean Dark Transitions
- Baseline EXIF orientation and XMP edit records are resolved and applied *before* updating canvas state.
- If a photo has no cached intermediate, immediately clear to the neutral dark canvas with the developing spinner for the 10–15ms until atomic promotion, avoiding visual confusion from retaining previous photo pixels.
