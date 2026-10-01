# Mini Spec: Pipeline Stages & Color Space Working Invariants

## Status: Approved Architecture Specification
**Author**: Remco van Bree  
**Scope**: `firstproof-core` (Rust CPU export & CLI), `FirstProofApp` (macOS Metal), `web` (WebGL 2 GLSL)

---

## 1. Executive Summary & Core Invariant

The First Proof color engine is calibrated around a fundamental optical principle:
**All physical image adjustments (exposure, white balance, tone recovery, curves, color tuning, film emulsion chemistry, grain, and print media) must occur in the wide-gamut optical domain before display encoding.**

```
[RAW Sensor CFA]
       │
       ▼ (Stage 1: Decode & Linearize)
[Camera Native Linear RGB]
       │
       ▼ (Stage 2: Adobe DCP Forward Matrix + CAT02)
[Wide-Gamut Linear Rec.2020] ◄─── ALL INTERNAL ADJUSTMENTS OCCUR HERE
       │
       ├─► Stage 3: Linear Exposure & White Balance
       ├─► Stage 4: Bilateral Base/Detail Tone Recovery
       ├─► Stage 5: Luminance-Scaled Tone Curves
       ├─► Stage 6: Color Balance & Split Toning
       ├─► Stage 7: Recording Emulsion (D-log E & DIR Couplers)
       ├─► Stage 8: Physical Grain Synthesis (Nutting Modulation)
       └─► Stage 9: Darkroom Print Media & Enlarger Chemistry
       │
       ▼ (Stage 10: AgX Filmic / DCP Curve Tone-Mapping)
[Display-Mapped Rec.2020 / sRGB Linear]
       │
       ▼ (Terminal Conversion Invariant)
[Target Output Color Space (sRGB / Display P3 / Gamma Encoded)]
```

### The Terminal RGB Conversion Invariant
> [!IMPORTANT]
> **Conversion to display or file output RGB (sRGB, Display P3, or gamma encoding) must strictly and solely be the VERY LAST step of the entire pipeline.**
> It must never occur prior to or inside internal adjustments. Performing adjustments (saturation, tone curves, grain, or print media) on narrow sRGB or gamma-encoded values truncates wide-gamut hues, distorts $R/G$ chromaticity ratios, and produces muddy skin tones and posterized clipping knees.

---

## 2. The 10 Sequential Pipeline Stages

### Stage 1: Sensor RAW Decoding & Linearization
- **Input**: Raw Bayer CFA pattern (`.cr2`, `.nef`, `.arw`, `.dng`) or 16-bit binned raw sensor frame.
- **Operation**:
  - Subtract sensor black levels per channel; divide by white saturation ceiling.
  - Bayer binning (2×2 box average for 10–15ms viewport preview) or multi-pass edge-adaptive demosaicing (AMaZE / PPG with chromatic median filtering for export).
- **Working Space**: Normalized $[0.0, 1.0]$ Camera Native Linear RGB.

### Stage 2: Camera Calibration & Working Space Mapping
- **Operation**:
  - Parse camera-specific Adobe DCP profile (`ColorMatrix1/2`, `ForwardMatrix1/2`, `AnalogBalance`, `AsShotNeutral`).
  - Interpolate dual-illuminant matrices (Standard Illuminant A 2856K and CIE D65 6504K) based on reciprocal color temperature ($1/T$).
  - Transform native sensor chromaticities into **Wide-Gamut Linear Rec.2020** primaries:
    $$Y = 0.2627 R + 0.6780 G + 0.0593 B$$
- **Working Space**: Linear Rec.2020 (unbounded, floats $\ge 0.0$).

### Stage 3: Linear Optical Exposure & White Balance
- **Operation**:
  - Optical exposure gain applied as a clean linear multiplier:
    $$\text{gain} = 2.0^{\Delta \text{EV}}$$
  - Chromatic adaptation via Bradford / CAT02 transform modulated by Temperature (Kelvin) and Tint sliders relative to sensor baseline.
- **Working Space**: Linear Rec.2020.

### Stage 4: Bilateral Base / Detail Tone Recovery
- **Operation**:
  - Edge-preserving bilateral decomposition separating image into low-frequency base illumination ($L_{\text{base}}$) and high-frequency textural detail ($L_{\text{detail}}$).
  - Highlights, Shadows, Whites, and Blacks sliders modulate $L_{\text{base}}$ using strictly monotonic, bounded cubic Hermite curves without halo artifacts.
- **Working Space**: Linear Rec.2020.

### Stage 5: Luminance-Scaled Tone Curves
- **Operation**:
  - Evaluates Fritsch-Carlson monotone cubic spline LUTs (RGB Composite, Red, Green, Blue).
  - **Chromaticity Protection Invariant**: Curve is evaluated on photopic luminance ($Y$). The resulting luminance gain $Y_{\text{curved}} / \max(Y, 10^{-6})$ scales the linear RGB vector, preserving chromaticity angles and preventing hue shifts across contrast adjustments.
- **Working Space**: Linear Rec.2020.

### Stage 6: Color Balance, Vibrance & Split Toning
- **Operation**:
  - Vibrance: Non-linear chroma scaling with automatic saturation compression on warm skin-tone hue angles ($h \in [20^\circ, 55^\circ]$).
  - Saturation: Radial chromaticity scaling in Rec.2020 $uv$ space.
  - Split Toning: Highlight and Shadow tinting blended via a smooth sigmoidal midpoint balance function.
- **Working Space**: Linear Rec.2020.

### Stage 7: Recording Emulsion (Photochemical Chemistry)
- **Operation**:
  - If `emulsion_archetype != None`:
    - Evaluates continuous $C^1$ sensitometric D-log E characteristic curves per channel (toe, straight-line gamma, shoulder).
    - Vertical DIR (Development Inhibitor Releasing) inter-image coupling suppresses adjacent layer contamination.
    - Push/Pull processing applies physical development time trim ($2^{\text{EV}}$ chemical gain).
- **Working Space**: Linear Rec.2020.

### Stage 8: Physical Grain Synthesis (Stochastic Emulsion Resynthesis)
- **Operation**:
  - If `emulsion_grain_mode != Off`:
    - Extracts high-frequency residual $\delta_{\text{sensor}} = Y - Y_{\text{lowpass}}$ from the sensor's own photons (reusing 4-tap detail unsharp masking samples).
    - Consolidates high-frequency chrominance variance into luminance density.
    - Modulates grain amplitude by Nutting midtone density curve $G(Y) = 4.0 \cdot Y \cdot (1.0 - Y)$.
- **Working Space**: Linear Rec.2020.

### Stage 9: Darkroom Print Media & Enlarger Chemistry
- **Operation**:
  - If `print_media != None`:
    - Simulates physical optical print transfer (Baryta Silver Gelatin, Warmtone Chlorobromide, Cyanotype, Bamako Silver Contact).
    - Multiplicative paper base reflectance ($I \cdot W_{\text{base}}$), variable contrast paper filtration, and maximum black density ($D_{\max}$).
- **Working Space**: Linear Rec.2020.

### Stage 10: Tone Mapping & Terminal Display Conversion
- **Operation**:
  1. **Tone Mapping Engine**:
     - `AgxFilmic` / `AgxVivid`: Troy Sobotka's open-domain cinematic tone mapper with natural highlight desaturation.
     - `CameraStandard`: Proprietary manufacturer camera curve matching DIGIC / BIONZ processing.
  2. **Terminal Color Space Transform**:
     - Matrix transform from Linear Rec.2020 to destination color space:
       - *Display Viewport*: Linear sRGB or Display P3 $\to$ Hardware EDR / SDR gamma encoding.
       - *File Export*: Convert to calibrated sRGB, Display P3, Adobe RGB 1998, or ProPhoto RGB, and embed corresponding ICC profile via `lcms2`.
- **Working Space**: Target Display / File Output RGB.

---

## 3. Multi-Engine Parity Matrix (Rust CPU • Metal GPU • WebGL 2)

Every stage must produce mathematically identical results across all three targets:

| Pipeline Stage | Rust CPU Export (`pipeline.rs`) | Metal GPU Viewport (`Shaders.metal`) | WebGL 2 GLSL (`FragmentShader.ts`) |
| :--- | :--- | :--- | :--- |
| **Rec.2020 Luminance** | $Y = 0.2627R + 0.6780G + 0.0593B$ | `dot(c, float3(0.2627, 0.6780, 0.0593))` | `dot(c, vec3(0.2627, 0.6780, 0.0593))` |
| **Linear Exposure** | `c * 2.0f32.powf(exp)` | `c * exp2(exposureGain)` | `c * exp2(u_exposure)` |
| **Tone Recovery** | Monotone Hermite splines | Matching Hermite polynomial | Matching Hermite polynomial |
| **Curve Evaluation** | `evaluate_curve_lut()` | 1D LUT Texture / Spline | 1D LUT Texture / Spline |
| **Emulsion D-log E** | Analytical $C^1$ cubic curve | Matching analytical $C^1$ curve | Matching analytical $C^1$ curve |
| **Print Media** | Subtractive Beer-Lambert | Matching subtractive transfer | Matching subtractive transfer |
| **AgX Filmic** | `agx_default_contrast_approx` | Matching AgX shader implementation | Matching AgX shader implementation |
| **Terminal Output** | `lcms2` / `LINEAR_REC2020_TO_SRGB` | `LINEAR_REC2020_TO_SRGB` matrix | `LINEAR_REC2020_TO_SRGB` matrix |

Automated verification is enforced by `cargo test --test dual_engine_synthetic_parity` and `tests/agx_shader_parity.rs`.
