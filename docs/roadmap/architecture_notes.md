# LumenBox Architecture Notes: UniFFI & Metal rendering

This document provides technical design guidelines for integrating the Rust Core library with the macOS SwiftUI application, covering the UniFFI interface definition and the GPU rendering pipeline.

---

## 1. Rust-Swift Bridge: UniFFI Specification

We will use UniFFI to auto-generate the binding layers. The bridge exposes the file system scanner, database indexer, RAW reader, and XMP manager from Rust to Swift.

### Proposed Bridge interface (Rust Side)

We will use macro-based exports (`#[uniffi::export]`) or a UDL file defining the objects:

```rust
// Core structs exposed to Swift
#[derive(uniffi::Record)]
pub struct ImageMetadata {
    pub file_path: String,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub iso: Option<u32>,
    pub aperture: Option<f32>,
    pub shutter_speed: Option<String>,
    pub focal_length: Option<f32>,
    pub taken_time: Option<i64>, // Unix timestamp
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
}

#[derive(uniffi::Record, Clone, Copy)]
pub struct EditSettings {
    pub exposure: f32,       // -5.0 to +5.0 EV
    pub contrast: f32,       // -100 to +100
    pub temperature: f32,    // Kelvin value (e.g. 5000.0)
    pub tint: f32,           // -150.0 to +150.0 (Green to Magenta)
    pub crop_left: f32,      // 0.0 to 1.0 (normalized)
    pub crop_top: f32,       // 0.0 to 1.0
    pub crop_right: f32,     // 0.0 to 1.0
    pub crop_bottom: f32,    // 0.0 to 1.0
    pub crop_angle: f32,     // Straightening angle in degrees (-45.0 to 45.0)
}

#[derive(uniffi::Object)]
pub struct LumenBoxLibrary {
    // Manages SQLite database and folder monitoring
}

#[uniffi::export]
impl LumenBoxLibrary {
    pub fn open(db_path: String) -> Self { ... }
    pub fn scan_folder(&self, folder_path: String, portable_catalog: bool) -> Vec<String> { ... }
    pub fn get_metadata(&self, file_path: String) -> Option<ImageMetadata> { ... }
    pub fn get_edits(&self, file_path: String) -> EditSettings { ... }
    pub fn save_edits(&self, file_path: String, edits: EditSettings) { ... }
}
```

### Swift Integration Lifecycle
1. The Rust core library (`lumenbox_core`) compiles into a static binary library (`liblumenbox_core.a`).
2. Run `uniffi-bindgen` to generate the C-FFI header file (`lumenbox_coreFFI.h`), module map, and Swift interface wrapper (`LumenBoxCore.swift`).
3. Bundle these assets inside an Xcode **Swift Package** (`LumenBoxBridge`) containing a binary target linking to `liblumenbox_core.a` (using a script target or manually mapping).
4. Import `LumenBoxBridge` in the main SwiftUI app.

---

## 2. Real-Time Metal Rendering Pipeline

To maintain 60fps responsiveness as the user drags exposure and temperature sliders, image operations must happen on the GPU.

```
+------------------+     Decoded 16-bit RGB Buffer       +-----------------------+
|  Rust Core (CPU) | ----------------------------------> | Swift MTKView (Metal) |
|  - LibRaw Decode |                                     | - Load texture        |
|  - WB matrices   |                                     | - Bind EditSettings   |
+------------------+                                     +-----------------------+
                                                                     |
                                                                     v
                                                         +-----------------------+
                                                         |  Metal Shader (GPU)   |
                                                         | - Crop/Rotate Coord   |
                                                         | - Apply Sliders       |
                                                         +-----------------------+
```

### Data Pipeline
1. **Demosaicing (CPU)**: When the edit view opens, Rust decodes the full RAW file using `rawloader` or `LibRaw`. It runs demosaicing (converting the Bayer grid to RGB pixels) and returns a flat float array (16-bit half-float or 32-bit float RGB values).
2. **Texture Creation (Metal/Swift)**: Swift reads the float array pointer from the Rust bridge and loads it into an `MTLTexture`:
   ```swift
   let descriptor = MTLTextureDescriptor.texture2DDescriptor(
       pixelFormat: .rgba16Float, // or .rgb16Float
       width: rawWidth,
       height: rawHeight,
       mipmapped: false
   )
   let texture = device.makeTexture(descriptor: descriptor)
   texture.replace(
       region: MTLRegionMake2D(0, 0, rawWidth, rawHeight),
       mipmapLevel: 0,
       withBytes: pointerToFloatBuffer,
       bytesPerRow: rawWidth * 8 // 16-bit half is 2 bytes per channel, 4 channels = 8 bytes
   )
   ```
3. **Shader Parameters (Uniforms)**: Pass `EditSettings` (mapped to a struct matching the Metal shader declaration) in an arguments buffer to the fragment shader.

### Metal Shader Specification (`Shaders.metal`)

The fragment shader performs pixel mapping (crop/rotate) and color modifications:

```metal
#include <metal_stdlib>
using namespace metal;

struct EditSettings {
    float exposure;
    float contrast;
    float temperature;
    float tint;
    float cropLeft;
    float cropTop;
    float cropRight;
    float cropBottom;
    float cropAngle;
};

// Fragment shader applying P0 adjustments
fragment float4 editFragmentShader(
    VertexOut input [[stage_in]],
    texture2d<float, access::sample> rawTexture [[texture(0)]],
    constant EditSettings &settings [[buffer(0)]]
) {
    // 1. Transform coordinates for rotation & crop
    float2 uv = input.texCoords;
    
    // Rotate uv coordinates around center (0.5, 0.5) by Settings.cropAngle
    float rad = settings.cropAngle * (M_PI_F / 180.0);
    float2 centered = uv - 0.5;
    float2 rotated = float2(
        centered.x * cos(rad) - centered.y * sin(rad),
        centered.x * sin(rad) + centered.y * cos(rad)
    ) + 0.5;
    
    // Crop interpolation
    float2 croppedUV = float2(
        mix(settings.cropLeft, settings.cropRight, rotated.x),
        mix(settings.cropTop, settings.cropBottom, rotated.y)
    );
    
    constexpr sampler s(address::clamp_to_edge, filter::linear);
    float4 pixel = rawTexture.sample(s, croppedUV);
    
    // 2. Apply Exposure
    // Exposure is applied as a power-of-two multiplier in linear space
    pixel.rgb *= pow(2.0, settings.exposure);
    
    // 3. Apply Contrast
    // Contrast expands or compresses values around middle gray (0.18 in linear space)
    float middleGray = 0.18;
    float contrastFactor = (settings.contrast + 100.0) / 100.0;
    pixel.rgb = middleGray + (pixel.rgb - middleGray) * contrastFactor;
    
    // Clamp values to prevent out-of-bounds artifacts
    pixel.rgb = clamp(pixel.rgb, 0.0, 1.0);
    
    return pixel;
}
```
