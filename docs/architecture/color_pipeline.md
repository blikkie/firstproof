# Unified Color Science & Camera Profiles Pipeline Architecture

Photographers demand both **scientific fidelity** (emulating camera manufacturer color profiles like Canon Picture Styles, Sony Creative Looks, and Nikon Picture Controls) and **analog filmic rendering** (natural highlight roll-off without digital clipping artifacts).

To prevent tone-curve collisions (double tone-mapping) and dynamic range clipping, First Proof decouples camera sensor calibration from display-rendering transforms into a modular, **5-stage scene-referred color pipeline**:

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
         [ 3. Scene-Referred Adjustments ] (Exposure EV, WB, Tone Recovery, Contrast)
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
      [ 4. Analog Emulsion & Darkroom Printmaking ]
      • Recording: Rochester/Ashigara/Mobberley DIR Coupler Chemistry
      • Printing: Silver Halide Paper D-log H & Archival Selenium/Platinum Toning
                                    │
                                    ▼
      [ 5. Procedural Organic Film Grain ] (Physical Zone V Midtone Clumping & Tri-Layer Chemistry)
                                    │
                                    ▼
         [ 6. Display & Print Target ] (Display P3, sRGB, or Print ICC)
```

---

## Core Invariants

1. **Linear Rec.2020 Working Space**:
   - Internal image adjustments (Exposure, Contrast, Tone Curves, Saturation, Film Grain) strictly execute in **Linear Rec.2020 wide-gamut** color space with a D65 white point, NOT gamma-encoded sRGB or display P3.
   - Dual-engine math parity is enforced: `Shaders.metal` (Metal GPU) and `export.rs` (Rust CPU) evaluate mathematical transforms identically.

2. **AgX Filmic Tone Mapping**:
   - Eliminates digital highlight clipping and the Abney effect (where skin tones shift to harsh yellow/salmon when red saturates before green and blue).
   - Inset matrix compresses wide gamuts safely; smooth sigmoidal curve rolls off highlights gently toward pure white.

3. **Adobe DCP Ingestion & Dual-Illuminant Calibration**:
   - Parses `ColorMatrix1/2`, `ForwardMatrix1/2`, and `CalibrationIlluminant1/2` from camera-specific DCP profiles.
   - Dynamically interpolates between Standard Illuminant A (2856K) and CIE D65 (6504K) based on reciprocal color temperature ($1/T$).

4. **Print & Web First (SDR Default)**:
   - Viewport initializes to standard dynamic range (`isHdrEnabled = false`). EDR/XDR is an explicit user opt-in inspection mode.

5. **Capture & Print Decoupling (Strict Neutral Default)**:
   - Emulsion profiles and darkroom print toners are strictly opt-in creative tools. The pipeline default is always pure optical sensor capture (Camera DCP + AgX).
   - Upstream in-camera recording chemistry (DIR couplers) and downstream darkroom printmaking (enlarger filtration, silver paper D-log H, archival toning) are orthogonal stages. Neither replaces or overrides the other.
