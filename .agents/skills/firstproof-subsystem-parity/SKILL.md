---
name: firstproof-subsystem-parity
description: >-
  Workflow for porting, rebuilding, or extending darkroom subsystems (viewport geometry, zoom/pan, interactive overlay tools, metadata intake, filmstrip culling) from native FirstProofApp (macOS Swift/Metal) to secondary targets (web darkroom, WASM, Linux, CLI) with guaranteed architectural, mathematical, and behavioral parity.
---

# Subsystem Parity & Reference Deconstruction Protocol

Whenever implementing, porting, or refactoring a subsystem in a secondary target (`web/`, `firstproof-wasm/`, Linux, CLI) where a primary reference implementation exists in `FirstProofApp/` (macOS Swift/Metal) or `firstproof-core/` (Rust):

**NEVER design or write the target subsystem from first principles or generic web/platform heuristics.**
Every edge case (orientation, display vs sensor space, full-bleed zoom, stationary HUD anchoring, gesture hit testing) has already been solved in the native app. Designing from scratch inevitably causes regressions that re-discover solved problems one symptom at a time.

---

## Core Rule: Extraction to Shared Library Over Duplication

Whenever analyzing whether a pattern, mathematical model, coordinate mapping, or state transition is being reimplemented or ported:

**The DEFAULT architectural preference must be to extract the logic into the centralized shared library (`firstproof-types` or `firstproof-core`) rather than duplicating it across Swift and TypeScript.**

1. **Test for Shared Extraction**:
   - Can this logic (aspect ratio calculation, viewport quad fitting, pan offset clamping, EXIF orientation decoding, curve evaluation, color transforms) be expressed as a pure function without UI framework dependencies?
   - If **YES**: Extract it into Rust in `firstproof-types`.
     - UniFFI generates Swift bindings for native macOS (`FirstProofBridge`).
     - WASM exports the exact same compiled binary to TypeScript (`WasmBridge`).
     - Pure Rust unit tests (`cargo test`) verify bounds, inversions, and invariants once for all platforms.
     - Both native macOS and Web darkroom become thin consumers of the identical compiled source of truth, eliminating the possibility of behavioral divergence.
2. **What Stays Platform-Specific**:
   - Only view hierarchy layout (`SwiftUI.View`, DOM HTML), CSS/AppKit visual styling, and window server event monitors (`NSEvent`, `PointerEvent`) remain in platform code.
   - All underlying sensitometric math, geometry, and schema rules belong to the single shared source of truth.

---

Follow this systematic 4-phase protocol.

---

## Phase 1: Exhaustive Reference File Identification

Before writing a single line of target code, locate and inspect the authoritative reference files in `FirstProofApp/` and `firstproof-core/`:

| Subsystem | Primary Native Reference Files | Key Responsibilities |
| :--- | :--- | :--- |
| **Viewport & Canvas** | `MetalCanvasView.swift`, `MetalRenderer.swift`, `Geometry.metal` | Viewport sizing, vertex quad NDC transforms, aspect fitting, zoom & pan |
| **Interactive Tools (Crop/Straighten)** | `CropOverlayView.swift`, `AppViewModel+Editor.swift` | Overlay coordinates, handle shapes, aspect inversion, stationary HUDs |
| **Metadata & Intake** | `FolderScanner.swift`, `raw.rs`, `xmp.rs` | 64KB header probing, EXIF orientation, sidecar precedence |
| **Filmstrip & Culling** | `FilmstripView.swift`, `ThumbnailCache.swift` | Upright orientation, GPU snapshot caching, selection promotion |
| **Color & Sensitometry** | `Shaders.metal`, `export.rs`, `color/` | Linear Rec.2020 transforms, AgX, film emulsions, print media |

---

## Phase 2: Extract the 4 Structural Primitives

Read the reference code and answer these 4 questions explicitly before designing the target solution:

### 1. Coordinate Spaces & Transforms
* What space do coordinate values live in?
  * *Sensor space* (raw unrotated sensor pixels, e.g. 6000×4000)
  * *Container space* (normalized 0.0..1.0 relative to unrotated raw file)
  * *Display space* (normalized 0.0..1.0 or pixel coordinates relative to the rotated/oriented on-screen image)
* **Invariant**: Interactive overlay tools (Crop, Loupe) always operate in **display space**. The vertex shader or rendering pipeline maps display space back to container/texture space.

### 2. Viewport & Sizing Negotiation
* Does the render surface size to the image or to the window?
* **Invariant**: The viewport rendering surface (`MTKView`, WebGL `<canvas>`) must **always span 100% of the viewport container** (`width: 100%, height: 100%`). Aspect fitting (`fitScaleX`, `fitScaleY`), zoom scale, and pan offsets must strictly execute in the **vertex stage** (NDC quad bounds).
* **Anti-Pattern**: Never delegate aspect containment to CSS `object-fit: contain` or native fixed-frame container bounds. Doing so traps the image in letterbox/pillarbox bars when zoomed.

### 3. State Lifecycle & HUD Placement
* Where are floating tool controls anchored?
* **Invariant**: Tool controls (Crop HUD, Straighten readout, Loupe controls) must reside in **stationary overlay containers** pinned to the viewport periphery (e.g. floating at bottom center above the filmstrip). They must never be nested inside or transform with the image/crop layer.
* What state is saved on exit?
* **Invariant**: Committing a tool (via "Done" button, Return, Escape, or unmount) must commit pending drag scrubs (`endSliderScrub`) and trigger sidecar serialization.

### 4. Orientation & Edge Cases
* How does the reference handle:
  * Native portrait photos ($w < h$)?
  * Orthogonal 90° / 270° camera orientation or manual rotation?
  * Flipped / mirrored orientation?
* **Invariant**: When displayed in portrait orientation ($w_{\text{disp}} < h_{\text{disp}}$), aspect presets must automatically invert ($3:2 \to 2:3$, $4:3 \to 3:4$, $16:9 \to 9:16$). Single-key shortcut <kbd>X</kbd> and the **Flip (X)** action must invert this ratio symmetrically.

---

## Phase 3: Construct the Porting Contract Matrix

Write down a 1:1 translation table mapping the native Swift/Metal constructs to their target equivalents:

```markdown
| Native Swift/Metal Construct | Target Equivalent (e.g. Web/WASM) | Behavioral Parity Guarantee |
| :--- | :--- | :--- |
| `fitScaleX / fitScaleY` in `MetalRenderer` | `u_fitScale` in WebGL vertex shader | Quad fills viewport on zoom; no letterbox bars |
| `CornerBracketShape` (18x18, 3.5px) | SVG `<path class="crop-handle-corner">` | Identical desktop visual affordance |
| `floatingActionBar` at bottom of `ZStack` | Stationary `#crop-toolbar` at bottom center | Floating HUD never covers image center |
| `isPortrait ? (2/3) : (3/2)` | `getTargetAspectRatio()` | Aspect presets match displayed orientation |
| Fast header EXIF probing | Sub-0.1ms 64KB byte-slice reader | Upright filmstrip thumbnails from frame 0 |
```

---

## Phase 4: Mechanical Transcription & Parity Verification

1. **Transcribe Directly**: Implement the target components following the matrix. Preserve exact dimensions, stroke widths, easing curves, and math.
2. **Parity Check Against Desktop**: Compare side-by-side with the macOS app running or reference code:
   - Does Fit view look identical?
   - When double-clicking to zoom 100%, does the image expand across the entire window without black bars?
   - Do handles, scrim opacity (55%), and rule-of-thirds dash styles (4 4) match?
   - Do keyboard shortcuts (<kbd>C</kbd>, <kbd>X</kbd>, <kbd>[</kbd>, <kbd>]</kbd>, <kbd>Return</kbd>, <kbd>Esc</kbd>) behave identically?
