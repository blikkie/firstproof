# Mini-Spec: Spatial Effects Scale, Kernel Radii & Two-Tier Viewport/Render Fidelity

## Status: Proposed / Roadmap (Stories 3.17, 3.18, 3.19)
**Author**: Remco van Bree  
**Scope**: `FirstProofApp` (Metal GPU), `web` (WebGL 2), `firstproof-core` (Rust CPU Export), `firstproof-engine`

---

## 1. Executive Summary & The Two-Tier Fidelity Architecture

First Proof strictly rejects digital gimmicks and synthetic AI filters. Every adjustment must correspond to real optical, sensor, or chemical darkroom principles.

Currently, several spatial effects (effects that sample neighboring pixels) use localized 4-tap or 8-tap stencils restricted to a 1–2 texel radius. While this keeps GPU viewport rendering sub-millisecond, it creates a fundamental physical disconnect:
1. **Halation Bloom** is currently computed purely pointwise ($0\text{ surrounding pixels}$), failing to scatter light into adjacent dark pixels.
2. **Bilateral Tone Recovery** operates at a 2-pixel stride ($<0.08\%$ of the image diagonal), behaving as a localized microcontrast filter rather than decomposing true macro scene illumination.
3. **Detail Sharpening & Grain Seeding** use a 4-tap Manhattan cross stencil, introducing diamond-shaped directional anisotropy along $45^\circ$ diagonal edges and decoupling grain clump scale from sensor noise extraction.

### The Two-Tier Fidelity Principle
To achieve physical authenticity without compromising First Proof's fluid 60–120fps viewport culling and editing ergonomics:
- **Tier 1: Viewport Canvas (Directional Fidelity & Rapid Feedback)**:
  - Uses sparse, multi-stride Poisson disk kernels ($9\text{–}17\text{ taps}$) or downsampled mipmap attachments.
  - Delivers $\ge 95\%$ perceptual fidelity of the optical effect, allowing the photographer to evaluate balance, radius, and intensity in real time.
- **Tier 2: Export & Offline Render Pipeline (Full Physical Fidelity)**:
  - Evaluates full multi-pass, separable 2D convolutions or multi-scale pyramids ($25\text{–}49\text{ taps}$ or multi-grid decomposition).
  - Operates on full uncompressed 16-bit sensor pixels, guaranteeing flawless print-grade optical transitions, zero directional artifacts, and genuine physical point-spread functions (PSF).

---

## 2. Spatial Effect 1: Physical Optical Halation Scatter & Point-Spread Diffusion (Story 3.17)

### Physical Phenomenon
In silver halide films lacking an Anti-Halation Undercoat (AHU) or with thin rem-jet backing (e.g. `BaodingSilver100` / Lucky SHD 100, CineStill 800T, vintage panchromatic stocks):
1. Incident scene photons from highlights entering the film shoulder ($Y > 0.55$, Zone VII+, $+1.6\text{ EV}$ above $18\%$ middle gray) saturate primary silver grains and pass through the emulsion layers without being absorbed.
2. At the acetate/polyester film base ($120\text{–}140\,\mu\text{m}$ thick) and air boundary, light undergoes **specular Fresnel internal reflection and lateral scatter** beyond the critical angle $\theta_c = \arcsin(1/1.50) \approx 41.8^\circ$.
3. Reflected photons re-enter the emulsion layer from behind, exposing neighboring silver grains in a concentric point-spread halo around the highlight source. On a physical 35mm film negative ($36\text{ mm} \times 24\text{ mm}$), the halo spans $\approx 0.35\text{–}0.50\text{ mm}$, which corresponds to a format-invariant ratio of $\approx 1.0\%\text{–}1.5\%$ of the frame long edge ($R_{\text{norm}} \approx 0.012$).

### Mathematical Formulation
Point-spread lateral scatter follows an exponential decay function:

$$I_{\text{scatter}}(d) = I_{\text{highlight}} \cdot e^{-d / \lambda_{\text{scatter}}}$$

Where $d$ is spatial Euclidean distance from the highlight source and $\lambda_{\text{scatter}}$ is the optical diffusion length ($\approx 1.2\%$ of the 35mm canvas frame).

### Two-Tier Implementation
1. **Viewport (Metal & WebGL 2)**:
   - 12-tap Poisson disk sparse convolution kernel scaled format-invariantly to $1.2\%$ of the canvas frame: $R = (0.012 / A, 0.012)$ in normalized UV space, where $A$ is the aspect ratio.
   - Samples surrounding highlight excess luminance $Y_{\text{excess}} = (Y - 0.55)^2 / (0.25 + (Y - 0.55))$ for $Y > 0.55$ ($C^1$ smooth onset, zero clipping knee).
   - Injects diffused scatter into the center fragment's linear Rec.2020 optical density.
2. **Export Render (`pipeline.rs`, `encoders.rs`)**:
   - 2-pass separable exponential blur on the highlight threshold mask ($Y > 0.55$) with radius scaled to $1.2\%$ of the image diagonal ($r = \text{round}(0.012 \times \max(W, H))$) at native resolution.
   - For color emulsions (Rochester Cinema 500 Stripped), models the spectral red-shift of halation (red light scatters furthest through the emulsion stack: $0.50 R, 0.12 G, 0.02 B$).

---

## 3. Spatial Effect 2: Multi-Scale Hierarchical Bilateral Tone Recovery (Story 3.18)

### Physical Phenomenon
Retinex theory and bilateral tone mapping (Durand & Dorsey 2002) state that natural scene radiance $I(x, y)$ decomposes into:

$$I(x, y) = L(x, y) \cdot R(x, y)$$

- $L(x, y)$: Low-frequency macro **illumination layer** (ambient light, shadows cast by large objects), spanning $2\%\text{–}5\%$ of the image diagonal.
- $R(x, y)$: High-frequency **reflectance / texture layer** (skin pores, fabric weave, foliage edges).

Evaluating tone recovery (Highlights/Shadows) on a 2-pixel stencil applies gains directly to local surface reflectance, crushing texture microcontrast or causing edge halos.

### Mathematical Formulation
Bilateral filter on log-luminance $\text{ev} = \log_2(Y / 0.18)$:

$$L_{\text{base}}(p) = \frac{1}{W_p} \sum_{q \in \Omega} G_{\sigma_s}(\|p - q\|) \cdot G_{\sigma_r}(|\text{ev}(p) - \text{ev}(q)|) \cdot \text{ev}(q)$$

Where spatial variance $\sigma_s \gg 2\text{ pixels}$ and range variance $\sigma_r \approx 0.85\text{ EV}$.

### Two-Tier Implementation
1. **Viewport (Metal & WebGL 2)**:
   - Dual-concentric ring stencil (16 taps total):
     - Inner ring (8 taps at $r_1 = 2\text{ texels}$): Guards immediate optical edges against boundary bleed.
     - Outer ring (8 taps at $r_2 = 16\text{–}24\text{ texels}$): Captures broad ambient illumination.
   - Aligns WebGL 2 (currently pointwise) to 1:1 parity with Metal.
2. **Export Render (`pipeline.rs`)**:
   - Multi-scale bilateral grid / Gaussian pyramid ($1/8\times$ base representation) evaluating true large-scale scene illumination.
   - Recovers deep shadow detail and highlight roll-off with 100% preservation of micro-texture bite.

---

## 4. Spatial Effect 3: Isotropic 8-Tap Detail Stencil, Decoupled Grain & Film Format Scaling (Story 3.19)

### Physical Phenomenon
1. **Rotational Invariance**: Optical lenses produce circularly symmetric point-spread functions (Airy disks). A 4-tap Manhattan cross $(\Delta x, \Delta y)$ is non-isotropic: diagonal edges at $45^\circ$ receive $\sqrt{2}\times$ less Laplacian sharpening than orthogonal edges, creating star/cross artifacts.
2. **Decoupled Grain Physics (Crystal Size vs. Development Contrast)**:
   - **Grain Size (Cluster Diameter)**: Determined by physical crystal morphology (tabular T-grain vs coarse cubic silver halide, ISO 25 to ISO 3200) multiplied by optical enlargement magnification $M$.
   - **Grain Amount (RMS Granularity / Contrast $\Delta D$)**: Governed by development chemistry (agitation, dilution, push/pull processing, and developer acutance). Coupling them into a single scalar artificially forces coarse grain to blow out or fine grain to be invisible.
3. **Film Gate Magnification Factor ($M$)**:
   - In optical enlargement, apparent grain cluster diameter and halation bloom scale inversely with gate size relative to 35mm ($36 \times 24\,\text{mm}$, diag $\approx 43.3\,\text{mm}$):
     - **Super 8** ($4.8 \times 3.5\,\text{mm}$): $M \approx 7.3\times$ (heavy, coarse grain clumps; massive highlight halation spill across adjacent subjects).
     - **16mm** ($10.3 \times 7.5\,\text{mm}$): $M \approx 3.4\times$ (classic vintage cinema texture).
     - **35mm Still Frame** ($36 \times 24\,\text{mm}$): $M \equiv 1.0\times$ (baseline reference).
     - **Medium Format / 120** ($56 \times 70\,\text{mm}$): $M \approx 0.48\times$ (silky, ultra-fine micro-grain; tight, crisp halation perimeter).
     - **Large Format / 4×5** ($95 \times 120\,\text{mm}$): $M \approx 0.28\times$ (virtually imperceptible grain at standard viewing; razor-thin, delicate halation rim).
4. **The Polaroid / Instant Invariant (Direct Positive Contact Scale)**:
   - Integral instant film (SX-70, 600, `CambridgeDiffusion100`) has **no negative and no enlarger lens**:
     - Image formation is Dye Diffusion Transfer (DDT) through reagent pods at direct $1:1$ contact scale ($M \equiv 1.0$).
     - Replaces silver halide crystal clumping with microscopic chemical dye cloud diffusion and subtle reagent mottling.
     - **Zero Halation**: Emulsion sandwich contains an opaque white titanium dioxide ($\text{TiO}_2$) reflective layer and carbon black opacifying layers, preventing internal Fresnel specular reflections. Halation scatter is physically zero.
5. **Format-Proportional Halation Scaling**:
   - Film base thickness ($t \approx 120\text{–}140\,\mu\text{m}$) is nearly constant across roll formats; hence, absolute physical halo width on film is $\sim 0.4\,\text{mm}$.
   - Relative to the frame, halation scatter radius $\lambda_{\text{scatter}}$ scales proportionally with format magnification: $\lambda_{\text{scatter}}(M) = \lambda_{35} \times M$.

### Mathematical Formulation
Isotropic 9-point discrete Laplacian stencil:

$$\nabla^2 Y(p) = \frac{1}{6} \sum_{i \in \text{ortho}} Y(q_i) + \frac{1}{12} \sum_{j \in \text{diag}} Y(q_j) - \frac{1}{2} Y(p)$$

Low-pass filter radius for sensor noise seeding & grain cluster sampling:

$$R_{\text{sample}} = \max\left(\text{texelSize} \cdot \text{grainSize} \cdot M_{\text{format}} \cdot 1.5, \, \text{texelSize}\right)$$

RMS density modulation (Nutting's Law):

$$\Delta D = \eta \cdot 4Y(1-Y) \cdot \text{grainAmount}$$

### Two-Tier Implementation
1. **Viewport & Export Engine Parity**:
   - Consolidate USM sharpening and grain seeding into a unified 8-tap isotropic fetch ring $(y_{\text{ortho}}, y_{\text{diag}})$.
   - Dynamic stride linked to `grain_size * M_format`, ensuring medium/large format and coarse-grain simulations capture genuine multi-pixel variance.
   - Zero added texture bandwidth: USM and grain share the exact same 8 neighbor fetches in `Detail.metal` and `FragmentShader.ts`.
   - Scale halation blur radius dynamically by $M_{\text{format}}$, with instant film (`CambridgeDiffusion100`) bypassing halation scatter entirely.

---

## 5. Downstream Spatial Effects (Stories 3.12 & 3.16)

1. **Story 3.12: Dual-Pass Spatial Lateral Chemical Diffusion (Mackie Lines) [Dropped]**:
   - *Status*: Dropped / Struck (YAGNI & Anti-Slop). Dilute stand development developer exhaustion (Mackie lines / Eberhard effect) visually mimics synthetic digital clarity / unsharp mask halos; dropped to maintain photographic restraint and prevent slider bloat.
2. **Story 3.16: Adaptive OLPF Deconvolution & Optical Sharpness Tuning**:
   - Consumes the 8-tap isotropic stencil and format MTF scaling to reverse birefringent sensor anti-aliasing filters while preventing diffraction-induced ringing.

