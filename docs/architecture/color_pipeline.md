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
        [ 7. TERMINAL STAGE: Canonical AgX Tone Mapping & Gamut Transform ]
        • Gamut Inset Matrix (Linear Rec.2020 -> AgX Inset)
        • Log2 Normalization (-10.0 to +6.5 EV dynamic range scaling)
        • Sigmoidal Filmic Contrast Curve & Chromatic Outset Transform (AgX Vivid formulation)
        • Linear Rec.2020 / Target Gamut Mapping
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
   - Multi-engine math parity is enforced: `Shaders.metal` (Metal GPU), `FragmentShader.ts` (WebGL GPU), and `pipeline.rs`/`encoders.rs` (Rust CPU) evaluate mathematical transforms identically.

3. **Physical Sensitometric Grain & Emulsion Placement**:
   - Real film grain simulates the physical distribution of silver halide crystals reacting to incident scene photons before optical tonality and print transfer.
   - Applying grain post-tone-mapping in narrow gamma sRGB non-linearly distorts shadow noise floors. Grain must be evaluated in the wide-gamut optical domain.

4. **Canonical AgX Tone Mapping (Vivid Formulation)**:
   - First Proof standardizes universally on Troy Sobotka's canonical AgX (Vivid formulation) as its single terminal tone mapper, eliminating confusing mode toggles (`AgxFilmic`, `AgxPhotographic`, and closed-domain camera-matching profiles).
   - Eliminates digital highlight clipping, highlight solarization, and the Abney effect (where skin tones shift to harsh yellow/salmon when red saturates before green and blue).
   - The Vivid formulation provides rich, deep primary chromaticity preservation and natural highlight roll-off without muddy tonal shifts or plastic digital clipping.
   - An AgX Inset matrix compresses wide-gamut scene radiance into the AgX working gamut; normalized $\log_2$ values $[-10.0, +6.5]$ are transformed via an analytical sigmoidal curve, and restored via the AgX Outset matrix before display encoding.

5. **Adobe DCP Ingestion & Dual-Illuminant Calibration**:
   - Parses `ColorMatrix1/2`, `ForwardMatrix1/2`, and `CalibrationIlluminant1/2` from camera-specific DCP profiles.
   - Dynamically interpolates between Standard Illuminant A (2856K) and CIE D65 (6504K) based on reciprocal color temperature ($1/T$).

6. **Print & Web First (SDR Viewport Default)**:
   - Viewport canvas always initializes and defaults to standard dynamic range (`isHdrEnabled = false`). EDR/XDR is an explicit user opt-in inspection mode.

7. **Capture & Print Decoupling (Strict Neutral Default)**:
   - Emulsion profiles and darkroom print toners are strictly opt-in creative tools. The pipeline default is always pure optical sensor capture (Camera DCP + AgX).
   - Upstream in-camera recording chemistry (DIR couplers) and downstream darkroom printmaking (enlarger filtration, silver paper D-log H, archival toning) are orthogonal stages. Neither replaces or overrides the other.
   - For detailed manufacturer publication codes and sensitometric characteristic curve derivations, see [sensitometric_emulsion_references.md](sensitometric_emulsion_references.md).

8. **Print Media Sensitometry, Boundary Anchoring & Substrate Decoupling**:
   - Darkroom print media archetypes model authentic photochemical and noble-metal processes:
     1. **Chromogenic Type-C**: RA-4 cyan/magenta/yellow dye coupling on resin-coated base with highlight compression.
     2. **Silver Dye-Bleach Classic**: Subtractive azo dye destruction reversal on polyester base ($D_{\max} > 3.0$).
     3. **Baryta Silver Gelatin FB**: Neutral barium sulfate ($\text{BaSO}_4$) fiber base holding metallic silver filaments.
     4. **Warmtone Chlorobromide**: Colloidal selenium dye deposition shifting silver to warm charcoal and plum tones.
     5. **Platinum / Palladium**: UV reduction of ferric oxalate to elemental noble metal with luminous open shadow toe and warm colloidal palladium midtones.
     6. **Historic Cyanotype**: 1842 Sir John Herschel ferric ferrocyanide Prussian blue salts.
     7. **Bamako Silver Contact**: High-silver contact paper ferrotyped to glass (Keïta / Sidibé).
   - **Strict Boundary Anchoring Invariant ($f(0.0) \equiv 0.0, f(1.0) \equiv 1.0$)**:
     - Additive digital black floor lifts (`+ 0.016` to `+ 0.035`) are forbidden in develop adjustments. They violate physical printer color management: causing inkjet drivers to starve print heads of carbon black pigment ($D_{\max}$), double-washing out physical prints, and casting unnatural cloudy veils on wide-gamut displays.
     - Luminous open shadow graduation in noble metals (Platinum/Palladium) is formulated via analytical cubic toe cushioning $f(x) = x + \alpha \cdot x(1-x)^2$ and harmonic colloidal dye toning, preserving absolute terminal black.
   - **Substrate Decoupling**:
     - Paper substrates (unbleached cotton rag, alpha-cellulose, base reflectance, OBA white points, substrate absorption limits) belong exclusively in **Soft Proofing via ICC profiles (Story 3.4)**, not develop adjustments.
