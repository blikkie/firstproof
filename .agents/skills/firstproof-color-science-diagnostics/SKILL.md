---
name: firstproof-color-science-diagnostics
description: >-
  Systematic protocol for debugging color science, transfer functions, and tone mapping operators in First Proof. Enforces boundary anchoring, strict monotonicity, 21-step grey wedge verification, stage-by-stage pipeline telemetry, and strict separation between camera matching profiles and filmic tone mappers to prevent random curve fitting.
---

# First Proof Color Science & Transfer Function Diagnostics

This guide establishes the mandatory scientific protocol for diagnosing, auditing, and debugging color rendering, tone curves, clipping, and gamut mapping across Rust CPU, Metal GPU, and WebGL targets.

---

## 1. Core Principles & Anti-Slop Invariants

1. **First-Principles Transfer Invariants Over Downstream Curve Fitting**:
   - Never apply localized scalar multipliers, arbitrary hue offsets, or downstream saturation tweaks to mask visual clipping or muddy tones.
   - **Boundary Anchoring**: Every transfer curve must satisfy $f(0.0) \equiv 0.0$ and $f(1.0) \equiv 1.0$ exactly.
   - **Strict Monotonicity**: $f'(x) > 0$ for all $x \in (0, 1)$. Curves must never cross zero early, plunge negative, or flatline before domain bounds.
   - **$C^1$ Smoothness**: Piecewise segments must have continuous first derivatives at transition points.

2. **Global Field & Wedge Verification (Never Spot-Check in Isolation)**:
   - Spot-checking single pixels creates severe confirmation bias.
   - Every color adjustment must be verified against the **21-step Photometric Grey Wedge** spanning $-10\text{ EV}$ to $+6.5\text{ EV}$ to ensure zero stops are crushed.

3. **Strict Separation of Color Models**:
   - **Camera Matching (`CameraStandard`)**: Benchmarked strictly against camera manufacturer reference JPEGs (DCP ForwardMatrix + Canon/Sony/Nikon curves).
   - **AgX Filmic (`AgxFilmic`, `AgxVivid`)**: Troy Sobotka's open-domain cinematic tone mapper for 16.5 stops of scene radiance. Never hack AgX with arbitrary "memory color" offsets to mimic a consumer camera JPEG.

---

## 2. Step-by-Step Diagnostic Protocol

Whenever investigating an image defect (e.g. shadow crushing, highlight blowout, unnatural color cast):

### Step 1: Analytical Transfer Curve Probe
Before touching any pipeline code, evaluate the transfer curve $f(x)$ on a synthetic log wedge:
```python
# Evaluate f(x) and its derivative across -10 EV to +6.5 EV
for ev in range(-10, 7):
    norm = (ev - (-10.0)) / 16.5
    y = curve(norm)
    # Check: y >= 0.0, y <= 1.0, dy/dx > 0
```
- If $y = 0.0$ for any $\text{EV} > -9.5\text{ EV}$, the curve has an **early zero-crossing bug**.
- If $y = 1.0$ for any $\text{EV} < +6.0\text{ EV}$, the curve has a **premature highlight clipping bug**.

### Step 2: Multi-Stage Telemetry Probe
Trace pixels or histograms through each physical boundary using `inspect trace-pixel`:
1. **Sensor RAW Photosites**: Check raw ADC counts against black level (`2042` on Canon 60D).
2. **White-Balanced Sensor Space**: Verify that pre-demosaic multipliers ($wb_r, wb_b$) don't clip before demosaicing.
3. **Camera Matrix $\to$ Linear Rec.2020**: Check for negative coordinates ($min(R, G, B) < 0$) indicating out-of-gamut spectral response.
4. **Tone Mapping Operator**: Check input EV relative to 0.18 mid-gray and output display luminance.
5. **Terminal sRGB / Display P3**: Check for quantization clipping ($< 0.5/255$ or $> 254.5/255$).

### Step 3: Decompose the Error (CIELAB $\Delta L^*, \Delta C^*, \Delta h$)
Do not rely on scalar MSE. Convert to CIELAB and inspect:
- **$\Delta L^*$ high, $\Delta C^*$ low**: The issue is pure tone reproduction / exposure curve. Do not touch color sliders.
- **$\Delta C^*$ high, $\Delta L^*$ low**: The issue is saturation / gamut compression.
- **$\Delta h$ high**: The issue is matrix white balance or spectral mismatch.

---

## 3. Automated Test Gates

Before committing any tone mapping or color science change:
1. Run `cargo test --test agx_shader_parity` (bit-level parity across Rust and Metal).
2. Run `cargo test --test camera_profiles_parity` (Macbeth chart & curve monotonicity).
3. Verify the 21-step wedge test: all 21 stops from $-10\text{ EV}$ to $+6.5\text{ EV}$ must yield strictly increasing display values without clamping.
