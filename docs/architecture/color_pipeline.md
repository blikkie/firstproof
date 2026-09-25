# Unified Color Science & Camera Profiles Pipeline Architecture

Photographers demand both **scientific fidelity** (emulating camera manufacturer color profiles like Canon Picture Styles, Sony Creative Looks, and Nikon Picture Controls) and **analog filmic rendering** (natural highlight roll-off without digital clipping artifacts).

To prevent tone-curve collisions (double tone-mapping), out-of-gamut clipping, and unnatural shadow posterization, First Proof strictly enforces that **all develop adjustments, tone curves, optical emulsions, detail sharpening, and physical grain execute in Wide-Gamut Linear Rec.2020, with conversion to display RGB strictly as the terminal stage**:

```
                          [ RAW Sensor Linear / CFA ]
                                     │
                                     ▼
        [ 1. Dual-Illuminant DCP Calibration Matrix ] (Illuminant A / D65)
                                     │
                                     ▼
        [ 2. Wide-Gamut Linear Working Space ] (Linear Rec.2020, D65)
                                     │
                                     ▼
        [ 3. Optical Scene Adjustments ]
        • White Balance (Mired-scaled Planckian loci in Rec.2020)
        • Scene-Referred Linear Exposure (2^EV photon multiplication)
        • Bilateral Base/Detail Tone Recovery (Highlights, Shadows, Whites, Blacks)
                                     │
                                     ▼
        [ 4. Upstream Photochemical Emulsion ]
        • Color DIR Couplers (Rochester / Ashigara)
        • Silver Halide Density (Mobberley)
        • Instant Dye Diffusion (Cambridge)
                                     │
                                     ▼
        [ 5. Wide-Gamut Tone Curves & Color Grading ]
        • Parametric & Freeform Tone Curves (Master, R, G, B in Linear Rec.2020)
        • Color Adjustments (Vibrance with Rec.2020 skin-tone locus protection, Saturation, Deep Blue, Foliage)
        • Split Toning & Dual-Wheel Color Grading (Rec.2020 luminance-anchored)
                                     │
                                     ▼
        [ 6. Spatial Detail, Grain & Print Media ]
        • Laplacian Edge-Preserving Unsharp Masking & Detail Synthesis
        • Physical Subtractive Grain (Beer-Lambert optical density in wide space)
        • Fine-Art Print Media Chemistry (Paper grades, enlarger filtration, toner chemistry)
                                     │
                                     ▼
        [ 7. TERMINAL STAGE: Tone Mapping & RGB Target Transform ]
        ┌────────────────────────────────────┴────────────────────────────────────┐
        ▼                                                                         ▼
   [ Mode A: AgX Filmic ]                                              [ Mode B: Camera Matching ]
   • Gamut Inset Matrix (Rec.2020 -> AgX Inset)                         • Camera DCP Tone Curve (Anchored 0.18 Middle Gray)
   • Log2 Normalization (-10.0 to +6.5 EV)                              • Memory Color HueSatMap Deltas
   • Sigmoidal Filmic Curve & Outset Transform                          • Linear Rec.2020 -> Linear sRGB Matrix
        │                                                                         │
        └────────────────────────────────────┬────────────────────────────────────┘
                                             │
                                             ▼
        [ 8. Display & Export Encoding ]
        • Clamping to [0.0, 1.0] display gamut
        • Display Transfer Function: sRGB OETF / Gamma 2.2 (or Display P3 for EDR)
        • ICC Profile Embedding (APP2 marker in JPEG/PNG via LittleCMS2)
```

---

## Core Invariants

1. **Terminal RGB Conversion Invariant**:
   - Conversion to display/output RGB (sRGB, Display P3, or gamma encoding) must strictly be the **VERY LAST step of the entire pipeline**, both in GPU viewport shaders (`Shaders.metal`, `FragmentShader.ts`) and CPU export pipelines (`export/pipeline.rs`, `encoders.rs`).
   - Every adjustment—White Balance, Exposure, Bilateral Tone Recovery, Recording Emulsions, Tone Curves (Master + RGB), Color Adjustments (Vibrance, Saturation, Deep Blue, Foliage), Split Toning, Detail/Sharpening, Film Grain, and Print Media Chemistry—must strictly execute in Wide-Gamut Linear Rec.2020 BEFORE the terminal display transform.
   - Never apply color adjustments, saturation, tone curves, or sharpening to narrow sRGB or gamma-encoded values. Doing so truncates wide-gamut hues, distorts $R/G$ chromaticity ratios, and produces muddy skin tones or posterized clipping knees.

2. **Linear Rec.2020 Working Space**:
   - Internal image adjustments strictly execute in **Linear Rec.2020 wide-gamut** color space with a D65 white point ($Y = 0.2627 R + 0.6780 G + 0.0593 B$), NOT gamma-encoded sRGB or display P3.
   - Dual-engine math parity is enforced: `Shaders.metal` (Metal GPU), `FragmentShader.ts` (WebGL GPU), and `pipeline.rs`/`encoders.rs` (Rust CPU) evaluate mathematical transforms identically.

3. **Physical Sensitometric Grain & Emulsion Placement**:
   - Real film grain simulates the physical distribution of silver halide crystals reacting to incident scene photons before optical tonality and print transfer.
   - Applying grain post-tone-mapping in narrow gamma sRGB non-linearly distorts shadow noise floors. Grain must be evaluated in the wide-gamut optical domain.

4. **AgX Filmic Tone Mapping**:
   - Eliminates digital highlight clipping and the Abney effect (where skin tones shift to harsh yellow/salmon when red saturates before green and blue).
   - Inset matrix compresses wide gamuts safely; smooth sigmoidal curve rolls off highlights gently toward pure white.

5. **Adobe DCP Ingestion & Dual-Illuminant Calibration**:
   - Parses `ColorMatrix1/2`, `ForwardMatrix1/2`, and `CalibrationIlluminant1/2` from camera-specific DCP profiles.
   - Dynamically interpolates between Standard Illuminant A (2856K) and CIE D65 (6504K) based on reciprocal color temperature ($1/T$).

6. **Print & Web First (SDR Viewport Default)**:
   - Viewport canvas always initializes and defaults to standard dynamic range (`isHdrEnabled = false`). EDR/XDR is an explicit user opt-in inspection mode.

7. **Capture & Print Decoupling (Strict Neutral Default)**:
   - Emulsion profiles and darkroom print toners are strictly opt-in creative tools. The pipeline default is always pure optical sensor capture (Camera DCP + AgX).
   - Upstream in-camera recording chemistry (DIR couplers) and downstream darkroom printmaking (enlarger filtration, silver paper D-log H, archival toning) are orthogonal stages. Neither replaces or overrides the other.
   - For detailed manufacturer publication codes and sensitometric characteristic curve derivations, see [sensitometric_emulsion_references.md](file:///Users/blikkie/src/LumenBox/docs/architecture/sensitometric_emulsion_references.md).
