# Sensitometric Emulsion Grounding & Literature References

In strict adherence to First Proof's core rule:
> **"Physical Sensitometric Math Over Magic Numbers & Piecewise Hacks"**  
> Emulations (film stocks, recording emulsions, print media, enlarger filtration, toner chemistry) must always be grounded in continuous physical, optical, and sensitometric math—never arbitrary "magic numbers", hard thresholds, or piecewise if/else hacks.

This document records the published technical data sheets, academic papers, and sensitometric curve parameters used for each upstream recording emulsion in `firstproof-core/src/color/emulsion.rs` and `FirstProofApp/Sources/Shaders/ColorScience.metal`.

---

## 1. Mathematical Formulation

### A. Sensitometric Characteristic Curve ($C^1$ Cubic Hermite Manifold)
The tonal response is evaluated as a continuous, monotonic analytical function:
$$f(v; \gamma, s, t) = \text{Shoulder}\Big(\text{Midtone}(v; \gamma); s\Big) - \text{Toe}(v; t)$$

1. **Midtone Gamma Response**:
   Normalized midtone coordinate:
   $$u = \frac{v}{v + 0.35}, \quad u \in [0, 1]$$
   Cubic polynomial S-curve perturbation:
   $$f_{\text{mid}}(v) = v + 0.40 \cdot (\gamma - 1.0) \cdot u(1 - u)(2u - 1)$$
2. **Sensitometric Highlight Shoulder**:
   Smooth cubic Hermite compression bounding high-density highlights without clipping:
   $$\Delta D_{\text{shoulder}} = s \cdot 0.12 \cdot t^2(3 - 2t), \quad t = \left(\frac{v - 0.65}{0.35}\right)$$
3. **Shadow Toe Compression**:
   Continuous quadratic roll-off at minimum density threshold ($D_{\text{min}}$):
   $$\Delta D_{\text{toe}} = t_{\text{toe}} \cdot 0.06 \cdot (1 - \tau)^2, \quad \tau = \left(\frac{v}{0.35}\right)$$

### B. Active Inter-Layer DIR Chemical Couplers
Evaluated in optical density space ($D = -\log_{10}(T)$):
$$\mathbf{D}_{\text{out}} = \mathbf{C}_{\text{DIR}} \cdot \mathbf{D}_{\text{in}}$$

With the achromatic invariant strictly enforced:
$$\sum_{j} C_{ij} = 1.0 \implies \mathbf{C}_{\text{DIR}} \cdot \begin{pmatrix} D \\ D \\ D \end{pmatrix} = \begin{pmatrix} D \\ D \\ D \end{pmatrix}$$

---

## 2. Archetype Verification & Literature Grounding

### 1. Rochester Portrait 400
- **Historical Analogue**: Kodak Professional Portra 400 (C-41 Color Negative)
- **Technical Reference**: Kodak Publication **E-4050 / E-4051** (*"KODAK PROFESSIONAL PORTRA 400 Film"*), Status M Densitometry.
- **Published Sensitometry**:
  - Exposure latitude: Over 3 decades of log exposure (> 10 stops linear response).
  - Parallel CMY highlight shoulder: Curves remain smooth and parallel past +3 EV without hard cutoff.
  - Development Inhibitor Releasing (DIR) couplers: Red layer exposure inhibits magenta dye formation, preventing cyan-magenta muddiness in skin tones.
- **Calibrated Parameters**:
  - Gamma: $\gamma_R = 0.90$, $\gamma_G = 0.90$, $\gamma_B = 0.90$ (low midtone contrast for scanning latitude).
  - Shoulder: $0.38$ (generous highlight overexposure headroom).
  - Toe: $0.08$ (open, luminous shadow response).
  - DIR Crosstalk: $c_{rg} = -0.04, c_{rb} = 0.02, c_{gr} = -0.08, c_{gb} = 0.02, c_{br} = 0.02, c_{bg} = -0.05$.

---

### 2. Ashigara Chrome 50
- **Historical Analogue**: Fujichrome Velvia 50 Professional [RVP] (E-6 Color Reversal)
- **Technical Reference**: Fujifilm Technical Data Sheet **AF3-960E** (*"FUJICHROME Velvia 50 Professional [RVP 50]"*), Status A Densitometry.
- **Published Sensitometry**:
  - Contrast Index: Exceptionally steep midtone gamma ($\gamma \approx 1.6 - 1.8$ in optical density space; $\gamma \approx 1.10 - 1.12$ in positive transmission space).
  - Maximum Density ($D_{\text{max}}$): Exceeds $3.6$, yielding ultra-deep neutral blacks.
  - DIR Inter-Image Effects: Strongest inter-layer suppression of any slide emulsion, preventing yellow/cyan cross-contamination in emerald foliage and cobalt skies.
- **Calibrated Parameters**:
  - Gamma: $\gamma_R = 1.10$, $\gamma_G = 1.12$, $\gamma_B = 1.11$.
  - Shoulder: $0.12$ (tight highlight threshold; requires precise exposure).
  - Toe: $0.14$ (R), $0.14$ (G), $0.16$ (B) (clean, dense shadow floor).
  - DIR Crosstalk: $c_{rg} = -0.15, c_{rb} = -0.08, c_{gr} = -0.18, c_{gb} = -0.08, c_{br} = -0.06, c_{bg} = -0.12$.

---

### 3. Rochester K-14 Chrome 64
- **Historical Analogue**: Kodachrome 64 Professional [KR] (K-14 Non-Substantive Color Reversal)
- **Technical Reference**: Kodak Technical Publication **E-88 / CIS-57** (*"KODAK EKTACHROME and KODACHROME Professional Films"*) & ResearchGate Sensitometric Layer Analysis.
- **Published Sensitometry**:
  - Chemistry: Non-substantive reversal emulsion; coupler-free silver halide layers developed with external dye couplers in separate red, blue, and green developer baths.
  - Spectral Asymmetry: Published Status A curves confirm the red-sensitive layer possesses higher effective speed and lower midtone slope than the blue layer, imparting the signature warm golden-amber highlight glow.
  - Maximum Density ($D_{\text{max}}$): $D_{\text{max}} > 3.2$ with archival ink-black shadow depths.
- **Calibrated Parameters**:
  - Gamma: $\gamma_R = 1.05$, $\gamma_G = 1.06$, $\gamma_B = 1.08$.
  - Shoulder: $0.18$ (R), $0.16$ (G), $0.14$ (B) (extended red highlight roll-off).
  - Toe: $0.08$ (R), $0.10$ (G), $0.14$ (B) (warm shadow bias).
  - DIR Crosstalk: $c_{rg} = -0.08, c_{rb} = 0.06, c_{gr} = -0.10, c_{gb} = 0.04, c_{br} = 0.08, c_{bg} = -0.04$.

---

### 4. Rochester Cinema 500
- **Historical Analogue**: Kodak VISION3 500T Color Negative Film 5219 / 7219 (ECN-2 Motion Picture Negative)
- **Technical Reference**: Kodak Technical Data Sheet **H-1-5219 / TI2647** (*"KODAK VISION3 500T Color Negative Film"*), Status M Densitometry.
- **Published Sensitometry**:
  - Color Balance: 3200K tungsten balanced.
  - Exposure Latitude: Extreme 14+ stops (spanning $-8$ to $+8$ camera stops on published Kodak H-1-5219 axes).
  - Linear Contrast Index: $\Delta D / \Delta \text{stop} \approx 0.18 - 0.20$, resulting in a flat linear negative specifically formulated for telecine and digital intermediate color grading.
  - Highlight Headroom: Soft shoulder extending 5+ stops above 18% neutral gray.
- **Calibrated Parameters**:
  - Gamma: $\gamma_R = 0.82$, $\gamma_G = 0.82$, $\gamma_B = 0.82$.
  - Shoulder: $0.45$ (broadest shoulder in portfolio, modeling 5+ stops of overexposure latitude).
  - Toe: $0.08$.
  - DIR Crosstalk: $c_{rg} = -0.02, c_{rb} = 0.01, c_{gr} = -0.03, c_{gb} = 0.01, c_{br} = 0.01, c_{bg} = -0.02$ (restrained inter-image effects for linear grading fidelity).

---

### 5. Rochester Daylight 200
- **Historical Analogue**: Kodak Gold 200 / Kodacolor (C-41 Consumer Color Negative)
- **Technical Reference**: Kodak Publication **E-7022 / CIS-124** (*"KODAK GOLD 200 Film"*), Status M Densitometry.
- **Published Sensitometry**:
  - Consumer Daylight Formulation: Designed for outdoor daylight snapshots (beach, sunlit family photography).
  - Contrast Index: Significantly punchier than Portra 400 (green/red slope $\approx 0.70$ vs $0.58$ in density units).
  - Spectral Coupling: Yellow dye layer suppresses blue development in high densities, imparting sun-drenched golden amber warmth to midtones and highlights while keeping shadows rich and warm.
- **Calibrated Parameters**:
  - Gamma: $\gamma_R = 1.12$, $\gamma_G = 1.08$, $\gamma_B = 0.98$ (punchy warm midtones).
  - Shoulder: $0.20$ (R), $0.22$ (G), $0.26$ (B).
  - Toe: $0.12$ (R), $0.12$ (G), $0.16$ (B).
  - DIR Crosstalk: $c_{rg} = -0.06, c_{rb} = 0.03, c_{gr} = -0.07, c_{gb} = 0.02, c_{br} = 0.08, c_{bg} = -0.10$.

---

### 6. Mobberley Silver 400
- **Historical Analogue**: Ilford HP5 Plus 400 (Panchromatic Silver Halide B&W Negative)
- **Technical Reference**: Ilford Technical Information Sheet (*"ILFORD HP5 PLUS Professional Black and White Film"*), ID-11 / D-76 Developer (1+1).
- **Published Sensitometry**:
  - Spectral Sensitivity: Panchromatic response from 380 nm to 660 nm peaking around 560 nm. Daylight weights: $0.28 R + 0.62 G + 0.10 B$.
  - Contrast Index: $\bar{G} \approx 0.62$ with classic straight line extending across Zone III to Zone VIII.
  - Tonal Behavior: True metallic silver halide reduction; zero chromatic dye clouds; clean collapse to neutral luminance.
- **Calibrated Parameters**:
  - Panchromatic Luminance: $Y_{\text{pan}} = 0.28 R + 0.62 G + 0.10 B$.
  - Gamma: $\gamma = 1.08$, Shoulder: $0.30$, Toe: $0.25$.

---

### 7. Cambridge Diffusion 100
- **Historical Analogue**: Polaroid Polacolor Type 100 / 669 (Instant Dye Diffusion Reversal)
- **Technical Reference**: Polaroid Corporation Technical Data Sheet (*Polacolor ER / Type 669*) & Edwin H. Land (*"Dye-Developer Diffusion Transfer Process"*, PNAS / SPSE).
- **Published Sensitometry**:
  - Chemistry: Pre-formed metallized dye-developer molecules (phthalocyanine cyan, azo yellow/magenta) linked to hydroquinone developers, diffusing into a positive receiver sheet unless immobilized by oxidized silver halides.
  - Reflective Base Tone: Positive receiver sheet contains titanium dioxide ($\text{TiO}_2$), providing a creamy reflective highlight base ($\approx 2.5\%$ base reflection).
  - Dynamic Range: Narrow 5–6 stop latitude with soft highlight roll-off and pastel green dye fidelity.
- **Calibrated Parameters**:
  - Gamma: $\gamma_R = 0.98$, $\gamma_G = 0.98$, $\gamma_B = 0.98$.
  - Shoulder: $0.35$, Toe: $0.12$.
  - Reflective $\text{TiO}_2$ Base Tone: $0.025 \cdot \max(1 - Y, 0) \cdot [1.02, 1.00, 0.94]$.
  - DIR Crosstalk: $c_{rg} = -0.03, c_{rb} = 0.02, c_{gr} = -0.04, c_{gb} = 0.03, c_{br} = 0.01, c_{bg} = -0.03$.
