# Mini Spec: Global Emulsion & Darkroom Print Roster

## Status: Approved for Phase 3 (Stories 3.11 & 3.12)
**Author**: Remco van Bree  
**Scope**: `firstproof-engine`, `firstproof-core` (Rust), `FirstProofApp` (Metal), `web` (WebGL 2)

---

## 1. Executive Summary & Anti-Slop Principle

Commercial film simulation tools (e.g., VSCO, RNI, Dehancer) succumbed to feature bloat by accumulating 150+ arbitrary LUTs that are essentially minor gamma tweaks branded with vintage film names.

In First Proof, an emulsion or print media archetype **must never be an aesthetic preset**. It must represent a **fundamentally distinct chemical, spectral, or physical manufacturing patent**:
1. Distinct dye-coupling chemistry (substantive vs. non-substantive couplers).
2. Unique spectral sensitivity (e.g., 4th color layer, orthopanchromatic sensitization, absence of anti-halation undercoat).
3. Authentic characteristic D-log E density curves (toe curvature, shoulder roll-off, push/pull gamma behavior).
4. Physical darkroom printmaking chemistry (ferrotype silver gelatin contact prints, paper base reflectance, enlarger filtration).

Furthermore, the roster must reflect the **global history of photographic chemistry**, eliminating the North American bias (which previously had 5 of 7 archetypes from Rochester and Cambridge) while incorporating the major European, Asian, and African darkroom traditions.

---

## 2. Rationalized World Emulsion Roster

### North American Consolidation
To avoid redundancy, **`RochesterDaylight200`** (Kodak Gold 200) is merged into **`RochesterPortrait400`** (Kodak Portra 400). Both are Rochester C-41 color negatives; consumer daylight saturation and contrast are achieved through standard exposure and vibrance controls rather than duplicate emulsion archetypes.

### The Curated World Emulsions (8 Distinct Traditions)

| Archetype | Identifier | Origin | Primary Technical Reference / Patent | Chemical & Sensitometric Distinctive |
| :--- | :--- | :--- | :--- | :--- |
| **Rochester Portrait 400** | `RochesterPortrait400` | Rochester, NY (Kodak) | Kodak Publication E-4050 / E-4051 ("KODAK PROFESSIONAL PORTRA 400 Film"), Status M Densitometry. | Modern C-41 wide-latitude portraiture; warm golden-amber bias; soft highlight shoulder ($\gamma=0.90$, shoulder $0.38$, toe $0.08$). |
| **Rochester Chrome 64** | `RochesterK14Chrome64` | Rochester, NY (Kodak) | Kodak Publication E-88 / CIS-57 ("KODAK EKTACHROME and KODACHROME Professional Films") & ResearchGate K-14 Layer Analysis. | K-14 non-substantive dye reversal; couplers added in liquid developer; asymmetric per-channel gammas ($\gamma_R=1.05, \gamma_G=1.06, \gamma_B=1.08$). |
| **Rochester Cinema 500** | `RochesterCinema500` | Rochester, NY (Kodak) | Kodak Technical Data Sheet H-1-5219 / TI2647 ("KODAK VISION3 500T 5219/7219"), Status M Densitometry. | ECN-2 motion picture negative; flat telecine gamma ($\gamma=0.82$, broad highlight shoulder $0.45$, toe $0.08$). |
| **Cambridge Diffusion 100** | `CambridgeDiffusion100` | Cambridge, MA (Polaroid) | Polaroid Corporation Technical Data Sheet (Polacolor ER / Type 669) & Edwin H. Land PNAS Publication (*"Retinex Theory"* / *"Dye-Developer Transfer"*). | Instant dye-diffusion pod chemistry; metallized dyes; soft pastel toe; reflective $\text{TiO}_2$ base tone ($0.025 \cdot (1 - Y)$). |
| **Leverkusen Portrait 160** | `LeverkusenPortrait160` | Leverkusen, DE (Agfa) | Agfa-Gevaert Technical Data Sheet F-PF4-E ("Agfacolor Portrait 160 / Optima 160"), Status M Densitometry; Agfa Fischer Coupler Patent (DE 253332 / US 2,179,244). | European substantive dye couplers; earthy ochres, soft pastel greens, muted shadow saturation ($\alpha=+0.06, \beta=-0.02$). |
| **Mobberley Silver 400** | `MobberleySilver400` | Mobberley, UK (Ilford) | Ilford Technical Information Sheet ("ILFORD HP5 PLUS Professional Black and White Film"), ID-11/D-76 Developer (1+1). | Classic British documentary cubic silver halide; wide grey latitude; rich midtone separation ($\gamma=1.08$, shoulder $0.30$, toe $0.25$). |
| **Ashigara Chrome 50** | `AshigaraChrome50` | Minamiashigara, JP (Fujifilm) | Fujifilm Technical Data Sheet AF3-960E ("FUJICHROME Velvia 50 Professional [RVP 50]"), Status A Densitometry. | E-6 ultra-saturated landscape reversal; deep inky blacks; vivid cyan-greens ($\gamma=1.10\dots 1.12$, shoulder $0.10\dots 0.12$, toe $0.14\dots 0.16$). |
| **Ashigara Portrait 400** | `AshigaraPortrait400` | Minamiashigara, JP (Fujifilm) | Fujifilm Technical Data Sheet AF3-0242E ("Fujicolor PRO 400H Professional") & US Patent 4,840,880 ("Fourth Color-Sensitive Layer", Takada et al.). | Patented **4th color layer** (cyan-sensitive inter-layer); fluorescent green suppression; airy Japanese skin tones ($\alpha=+0.08, \beta=+0.06$). |
| **Liguria Cinema 80** | `LiguriaCinema80` | Cairo Montenotte, IT (Ferrania) | FILM Ferrania Technical Data Sheet P30 ("P30 Alpha 80 ISO Panchromatic Motion Picture Film"), Ferrania Research Laboratories. | Legendary 1960s Italian Neorealist cinema stock (P30); high silver density; velvety Dmax; dramatic microcontrast ($\alpha=+0.32, \beta=-0.08$). |
| **Baoding Silver 100** | `BaodingSilver100` | Baoding, CN (Lucky Film) | China Lucky Film Corporation Technical Specification ("Lucky SHD 100 Panchromatic Black and White Film", GB/T 13982 standard). | Iconic Chinese domestic cubic silver halide; **absence of anti-halation undercoat** producing natural specular halation bloom ($\alpha=+0.18, \beta=+0.10$, bloom above $Y=0.85$). |

---

## 3. Darkroom Print Media: West African Studio Chemistry

### Historical Background: The "Shirley Card" Problem vs. Bamako Masters
Standard Western photographic paper and commercial printing curves (e.g. Kodak's default gamma and "Shirley" calibration card) were historically calibrated for Caucasian skin tones. Under standard commercial processing, dark skin tones compressed into flat, muddy, shadow-choked zones.

In mid-century West Africa, legendary portrait masters **Seydou Keïta & Malick Sidibé** (*Bamako, Mali*) and **Sanlé Sory** (*Bobo-Dioulasso, Burkina Faso*) established an iconic darkroom tradition:
- Negatives were printed as **1:1 contact prints** using natural daylight frames or tungsten lamps.
- Processed in high-concentration hydroquinone/metol developers (e.g. Kodak D-72 / Dektol) under high ambient studio heat.
- Dried on **ferrotype tins**, producing an ultra-smooth, mirror-like specular surface.

### The Sensitometric Archetype: `BamakoSilverContact`
Added to `PrintMediaMode`:
1. **Zone III–V Microcontrast & Luminosity**: Specifically expands tonal separation across rich dark skin tones and shadow textures, eliminating muddy compression and rendering deep melanin tones luminous and specular ($\alpha=+0.22, \beta=+0.14$).
2. **High-Key Textile Acutance**: Retains brilliant, non-blooming edge contrast in Zone VII–VIII highlights for complex woven textiles (Ankara wax prints, bogolanfini/mudcloth, lace) and metallic jewelry.
3. **Ferrotype Mirror Dmax**: Deep, inky maximum black density ($D_{\max} \ge 2.25$) with neutral-to-cool specular silver reflectance ($Y^{1.0 + 0.15 \cdot \text{depth}}$).

---

## 4. Sensitometric Formulations

### Mathematical Models

In accordance with First Proof's anti-magic-number invariants, all emulsion curves are formulated as continuous, strictly monotonic functions without slope discontinuities or hard clipping knees:

#### 1. Multi-Layer D-log H Characteristic Curves (`apply_film_characteristic_curve`)
Evaluated on foundational multi-layer negative and reversal films:
1. **Monotonic Cubic Midtone Response**:
   $$u = \frac{v}{v + 0.35}, \quad f_{\text{base}}(v) = v + 0.40(\gamma - 1.0) \cdot u(1-u)(2u-1)$$
2. **Smooth $C^1$ Hermite Highlight Shoulder** (active for $v > 0.65$):
   $$t_s = \frac{v - 0.65}{0.35}, \quad \Delta_{\text{shoulder}} = 0.12 \cdot \text{shoulder} \cdot t_s^2(3 - 2t_s)$$
3. **Smooth $C^1$ Quadratic Shadow Toe** (active for $v < 0.35$):
   $$t_t = \frac{v}{0.35}, \quad \Delta_{\text{toe}} = 0.06 \cdot \text{toe} \cdot (1 - t_t)^2$$

| Archetype | Gamma ($\gamma$) | Shoulder Roll-Off | Toe Compression | DIR Coupler Crosstalk |
| :--- | :--- | :--- | :--- | :--- |
| `RochesterPortrait400` | $0.90$ | $0.38$ | $0.08$ | Cyan-magenta softening ($C_{rg}=-0.04, C_{gr}=-0.08$) |
| `AshigaraChrome50` | $1.10 / 1.12 / 1.11$ | $0.12 / 0.10 / 0.12$ | $0.14 / 0.14 / 0.16$ | High edge-effect emerald isolation ($C_{rg}=-0.15, C_{gr}=-0.18$) |
| `RochesterK14Chrome64` | $1.05 / 1.06 / 1.08$ | $0.18 / 0.16 / 0.14$ | $0.08 / 0.10 / 0.14$ | Asymmetric warm golden highlight separation |
| `RochesterCinema500` | $0.82$ | $0.45$ | $0.08$ | Flat telecine grading latitude |
| `MobberleySilver400` | $1.08$ | $0.30$ | $0.25$ | Panchromatic enlarger filtration collapse |
| `CambridgeDiffusion100` | $0.98$ | $0.35$ | $0.12$ | Phthalocyanine pastel + $\text{TiO}_2$ base reflection |

#### 2. Continuous $C^1$ Analytical S-Curves (`apply_emulsion_s_curve`)
Evaluated on regional and specialty world archetypes, anchored at $f(0.0) \equiv 0.0$ and $f(1.0) \equiv 1.0$:

$$f(x) = x + \alpha \cdot x(1-x)(2x-1) + \beta \cdot x(1-x)$$

| Archetype | $\alpha$ (S-Curve / Contrast) | $\beta$ (Toe / Shoulder Bias) | Special Chemical Transform |
| :--- | :--- | :--- | :--- |
| `LeverkusenPortrait160` | $+0.06$ | $-0.02$ (Soft Shoulder) | Substantive ochre/earthy dye balance ($C_{rg}=-0.05, C_{gr}=-0.06$) |
| `AshigaraPortrait400` | $+0.08$ | $+0.06$ (Airy Highlights) | 4th-layer cyan inter-layer suppression ($C_{gr}=-0.12, C_{bg}=-0.08$) |
| `LiguriaCinema80` | $+0.32$ | $-0.08$ (High Contrast) | Pure silver monochrome collapse ($R=G=B$) |
| `BaodingSilver100` | $+0.18$ | $+0.10$ (Halation Bloom) | Lateral light scatter $0.08 \cdot ((Y-0.85)/0.15)^2$ for $Y > 0.85$ |
| `BamakoSilverContact` | $+0.22$ | $+0.14$ (Zone III–V Expansion) | High-Dmax ferrotype curve with textile acutance |

---

## 5. Architectural Integration

### Data Models (`firstproof-engine`)
- [`modes.rs`](../../firstproof-engine/src/modes.rs):
  - Update `EmulsionArchetype` enum:
    - Retain `RochesterPortrait400`, `AshigaraChrome50`, `RochesterK14Chrome64`, `RochesterCinema500`, `MobberleySilver400`, `CambridgeDiffusion100`.
    - Deprecate/merge `RochesterDaylight200` $\to$ `RochesterPortrait400`.
    - Add `LeverkusenPortrait160`, `AshigaraPortrait400`, `LiguriaCinema80`, `BaodingSilver100`.
  - Update `PrintMediaMode` enum:
    - Add `BamakoSilverContact`.
- [`xmp.rs`](../../firstproof-engine/src/xmp.rs):
  - Serialization/deserialization for new enum variants with backward-compatibility aliases.

### Shader & Pipeline Parity
- Implement identical analytical math across:
  1. `firstproof-core/src/color/emulsion.rs`
  2. `firstproof-core/src/color/print_media.rs`
  3. `FirstProofApp/Sources/Shaders/Shaders.metal`
  4. `web/src/shaders/FragmentShader.ts`
- Verified via `tests/dual_engine_synthetic_parity.rs` and `tests/print_media_parity.rs`.
