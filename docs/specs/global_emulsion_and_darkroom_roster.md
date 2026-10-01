# Mini Spec: Global Emulsion & Darkroom Print Roster

## Status: Approved for Phase 3 (Stories 3.11 & 3.12)
**Author**: Remco van Bree  
**Scope**: `firstproof-types`, `firstproof-core` (Rust), `FirstProofApp` (Metal), `web` (WebGL 2)

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

| Archetype | Identifier | Origin | Chemical Distinctive |
| :--- | :--- | :--- | :--- |
| **Rochester Portrait 400** | `RochesterPortrait400` | Rochester, NY (Kodak) | Modern C-41 wide-latitude portraiture; warm golden-amber bias; soft highlight shoulder. |
| **Rochester Chrome 64** | `RochesterK14Chrome64` | Rochester, NY (Kodak) | K-14 non-substantive dye reversal; couplers added in liquid developer; razor-sharp edge acutance. |
| **Cambridge Diffusion 100** | `CambridgeDiffusion100` | Cambridge, MA (Polaroid) | Instant dye-diffusion pod chemistry; metallized dyes; soft pastel toe. |
| **Leverkusen Portrait 160** | `LeverkusenPortrait160` | Leverkusen, DE (Agfa) | European substantive dye couplers; earthy ochres, soft pastel greens, and muted shadow saturation. |
| **Mobberley Silver 400** | `MobberleySilver400` | Mobberley, UK (Ilford) | Classic British documentary cubic silver halide; wide grey latitude; rich midtone separation. |
| **Ashigara Chrome 50** | `AshigaraChrome50` | Minamiashigara, JP (Fujifilm) | E-6 ultra-saturated landscape reversal; deep inky blacks; vivid cyan-greens. |
| **Ashigara Portrait 400** | `AshigaraPortrait400` | Minamiashigara, JP (Fujifilm) | Patented **4th color layer** (cyan-sensitive inter-layer); fluorescent green suppression; airy Japanese skin tones. |
| **Liguria Cinema 80** | `LiguriaCinema80` | Cairo Montenotte, IT (Ferrania) | Legendary 1960s Italian Neorealist cinema stock (P30); high silver density; velvety Dmax; dramatic microcontrast. |
| **Baoding Silver 100** | `BaodingSilver100` | Baoding, CN (Lucky Film) | Iconic Chinese domestic cubic silver halide; **absence of anti-halation undercoat** producing natural specular halation bloom. |

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
1. **Zone III–V Microcontrast & Luminosity**: Specifically expands tonal separation across rich dark skin tones and shadow textures, eliminating muddy compression and rendering deep melanin tones luminous and specular.
2. **High-Key Textile Acutance**: Retains brilliant, non-blooming edge contrast in Zone VII–VIII highlights for complex woven textiles (Ankara wax prints, bogolanfini/mudcloth, lace) and metallic jewelry.
3. **Ferrotype Mirror Dmax**: Deep, inky maximum black density ($D_{\max} \ge 2.25$) with neutral-to-cool specular silver reflectance.

---

## 4. Sensitometric Formulations

### Continuous $C^1$ Analytical S-Curves
In accordance with First Proof's anti-magic-number invariants, all emulsion curves are formulated as continuous, strictly monotonic functions anchored at $f(0.0) \equiv 0.0$ and $f(1.0) \equiv 1.0$:

$$f(x) = x + \alpha \cdot x(1-x)(2x-1) + \beta \cdot x(1-x)$$

| Archetype | $\alpha$ (S-Curve / Contrast) | $\beta$ (Toe / Shoulder Bias) | Special Chemical Transform |
| :--- | :--- | :--- | :--- |
| `RochesterPortrait400` | $+0.12$ | $+0.04$ (Warm Toe) | DIR coupler shadow desaturation |
| `LeverkusenPortrait160` | $+0.06$ | $-0.02$ (Soft Shoulder) | Substantive ochre/earthy dye balance |
| `AshigaraPortrait400` | $+0.08$ | $+0.06$ (Airy Highlights) | 4th-layer cyan inter-layer suppression |
| `LiguriaCinema80` | $+0.32$ | $-0.08$ (High Contrast) | Pure silver monochrome collapse ($R=G=B$) |
| `BaodingSilver100` | $+0.18$ | $+0.10$ (Halation Bloom) | Lateral light scatter around $Y > 0.85$ |
| `BamakoSilverContact` | $+0.22$ | $+0.14$ (Zone III–V Expansion) | High-Dmax ferrotype curve with textile acutance |

---

## 5. Architectural Integration

### Data Models (`firstproof-types`)
- [`modes.rs`](file:///home/remco/src/firstproof/firstproof-types/src/modes.rs):
  - Update `EmulsionArchetype` enum:
    - Retain `RochesterPortrait400`, `AshigaraChrome50`, `RochesterK14Chrome64`, `RochesterCinema500`, `MobberleySilver400`, `CambridgeDiffusion100`.
    - Deprecate/merge `RochesterDaylight200` $\to$ `RochesterPortrait400`.
    - Add `LeverkusenPortrait160`, `AshigaraPortrait400`, `LiguriaCinema80`, `BaodingSilver100`.
  - Update `PrintMediaMode` enum:
    - Add `BamakoSilverContact`.
- [`xmp.rs`](file:///home/remco/src/firstproof/firstproof-types/src/xmp.rs):
  - Serialization/deserialization for new enum variants with backward-compatibility aliases.

### Shader & Pipeline Parity
- Implement identical analytical math across:
  1. `firstproof-core/src/color/emulsion.rs`
  2. `firstproof-core/src/color/print_media.rs`
  3. `FirstProofApp/Sources/Shaders/Shaders.metal`
  4. `web/src/shaders/FragmentShader.ts`
- Verified via `tests/dual_engine_synthetic_parity.rs` and `tests/print_media_parity.rs`.
