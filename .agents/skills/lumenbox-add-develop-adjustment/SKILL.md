---
name: lumenbox-add-develop-adjustment
description: >-
  Guide for adding or modifying a develop adjustment (sliders, curves, toggles) across the 6-layer LumenBox architecture: Rust core EditSettings, UniFFI bridge, Metal shader uniforms, AppViewModel state, Inspector UI, and export pipeline parity.
---

# Adding a Develop Adjustment across the 6 Layers of LumenBox

Whenever adding a new image adjustment (e.g., Dehaze, Clarity, Color Grading Wheels, Vignette, Texture) or modifying existing adjustment parameters, follow this systematic 6-layer workflow.

---

## Layer 1: Rust Core `EditSettings` Record

1. In `lumenbox-core/src/lib.rs`:
   Add the new field to `pub struct EditSettings`:
   ```rust
   pub struct EditSettings {
       // ... existing fields ...
       pub my_parameter: f32,
   }
   ```
2. Update default factory:
   ```rust
   impl Default for EditSettings { ... }
   ```
3. Update `export.rs` CPU processing pipeline so offline full-resolution JPEG export applies the exact same math as the GPU viewport preview.

---

## Layer 2: Metal Shaders & Uniform Layout

1. In `LumenBoxApp/Sources/Shaders/ShaderTypes.h`:
   Add the matching member to `struct Uniforms`:
   ```objc
   struct Uniforms {
       // ... existing uniforms ...
       float myParameter;
   };
   ```
   *(Ensure 4-byte / 16-byte SIMD alignment rules in Metal C headers)*.
2. In `LumenBoxApp/Sources/Shaders/Shaders.metal`:
   Implement the color or tone transformation in the fragment shader:
   ```metal
   half3 processed = applyMyAdjustment(color, uniforms.myParameter);
   ```
3. In `LumenBoxApp/Sources/Metal/MetalRenderer.swift`:
   Populate `uniforms.myParameter = edits.myParameter`.

---

## Layer 3: Regenerate UniFFI Bridge

Run:
```bash
make bundle
```
Or manually run the UniFFI code generator:
```bash
cargo run --manifest-path lumenbox-core/Cargo.toml --target aarch64-apple-darwin \
  --features uniffi/cli --bin uniffi-bindgen -- generate \
  --library target/aarch64-apple-darwin/release/liblumenbox_core.dylib \
  --language swift \
  --out-dir LumenBoxBridge/Sources/LumenBoxBridge/
```

---

## Layer 4: `AppViewModel` State & Value Semantics

1. In `LumenBoxApp/Sources/ViewModels/AppViewModel.swift`:
   Add published property:
   ```swift
   public var myParameter: Float = 0.0
   ```
2. In `LumenBoxApp/Sources/ViewModels/AppViewModel+Editor.swift`:
   - Update `currentEdits`: pass `self.myParameter`.
   - Update `applyEdits(_:)`: assign `self.myParameter = edits.myParameter`.
   - Update `EditSettings.defaultIdentity`.
   - Update `handleSliderEditing(_:slider:)`: add human-readable action label (e.g. `"My Param (\(Int(myParameter)))"`).
   - Update `resetSlider(_:)`: reset to default value.

---

## Layer 5: Inspector Sidebar UI Component

1. In the relevant workspace view (e.g. `LightWorkspaceView.swift`, `ColorWorkspaceView.swift`, or `DetailWorkspaceView.swift`):
   ```swift
   SliderRow(
       label: "My Param",
       value: $viewModel.myParameter,
       range: -100.0...100.0,
       format: "%.0f",
       onReset: { viewModel.resetSlider(.myParameter) },
       onEditingChanged: { viewModel.handleSliderEditing($0, slider: .myParameter) }
   )
   ```

---

## Layer 6: Verification & Dual-Engine Parity

1. Add a test in `lumenbox-core/tests/camera_profiles_parity.rs` or `agx_shader_parity.rs` comparing the output of the Rust CPU function against known benchmark values.
2. Run:
   ```bash
   cargo test --manifest-path lumenbox-core/Cargo.toml
   swift build --package-path LumenBoxApp
   ```
