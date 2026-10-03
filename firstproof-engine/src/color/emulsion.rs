//! Upstream Recording Emulsions & Photochemical Color Science
//!
//! Models the photochemical behavior of the latent image on multi-layer camera
//! negative/reversal and instant diffusion film in Linear Rec.2020 prior to print
//! enlargement or display mapping.
//!
//! Key Physics:
//! 1. Sensitometric D-log H Characteristic Curves: Grounded in manufacturer data sheets
//!    (C-41 negative, E-6 reversal, K-14 non-substantive reversal, ECN-2 cinema, and panchromatic B&W).
//! 2. Active DIR Chemical Couplers: Vertical development inhibitor releasing crosstalk
//!    matrices ($3 \times 3$ $\mathbf{C}_{\text{DIR}}$) evaluated in linear Rec.2020 density space.
//! 3. Latent Image Push/Pull Exposure Index: Sensitometric $2^{\text{EV}}$ trim modulating
//!    chemical reaction potential.
//!
//! Bit-level dual-engine parity with Metal shaders is strictly enforced.


pub use crate::modes::{EmulsionArchetype, EmulsionFormat, EnlargerFilterMode};


/// Sensitometric $C^1$ continuous D-log H characteristic curve with soft shoulder and toe compression.
#[inline]
pub fn apply_film_characteristic_curve(val: f32, gamma: f32, shoulder: f32, toe: f32) -> f32 {
    let v = val.clamp(0.0, 2.0);

    // 1. Monotonic cubic midtone gamma response (f(x) = x + alpha * x(1-x)(2x-1))
    let base = if (gamma - 1.0).abs() > 0.001 {
        let u = (v / (v + 0.35)).clamp(0.0, 1.0);
        let g_mod = (gamma - 1.0) * 0.40;
        (v + g_mod * u * (1.0 - u) * (2.0 * u - 1.0)).max(0.0)
    } else {
        v
    };

    // 2. Sensitometric highlight shoulder roll-off:
    // Continuous C1 Hermite shoulder naturally vanishing at base <= 0.65 with zero derivative
    let t_s = ((base - 0.65) / 0.35).clamp(0.0, 1.0);
    let comp_s = shoulder * 0.12 * t_s * t_s * (3.0 - 2.0 * t_s);

    // 3. Shadow toe compression:
    // Continuous C1 quadratic roll-off naturally vanishing at base >= 0.35 with zero derivative
    let t_t = (base / 0.35).clamp(0.0, 1.0);
    let comp_t = toe * 0.06 * (1.0 - t_t) * (1.0 - t_t);

    (base - comp_s - comp_t).max(0.0)
}

/// Continuous $C^1$ analytical S-curve anchored at f(0.0) == 0.0 and f(1.0) == 1.0:
/// f(x) = x + alpha * x(1-x)(2x-1) + beta * x(1-x)
#[inline]
pub fn apply_emulsion_s_curve(x: f32, alpha: f32, beta: f32) -> f32 {
    let c = x.clamp(0.0, 1.0);
    let poly = c + alpha * c * (1.0 - c) * (2.0 * c - 1.0) + beta * c * (1.0 - c);
    poly.clamp(0.0, 1.0)
}

/// Active DIR Chemical Crosstalk Coupler Matrix in Optical Density Space:
/// D = -log10(T)
/// D_out = C_DIR * D_in
#[inline]
pub fn apply_dir_chemical_coupling(
    r: f32,
    g: f32,
    b: f32,
    c_rg: f32, c_rb: f32,
    c_gr: f32, c_gb: f32,
    c_br: f32, c_bg: f32,
) -> (f32, f32, f32) {
    let eps = 1e-4f32;
    let d_r = -(r.max(eps)).log10();
    let d_g = -(g.max(eps)).log10();
    let d_b = -(b.max(eps)).log10();

    let c_rr = 1.0 - c_rg - c_rb;
    let c_gg = 1.0 - c_gr - c_gb;
    let c_bb = 1.0 - c_br - c_bg;

    let d_r_out = c_rr * d_r + c_rg * d_g + c_rb * d_b;
    let d_g_out = c_gr * d_r + c_gg * d_g + c_gb * d_b;
    let d_b_out = c_br * d_r + c_bg * d_g + c_bb * d_b;

    let r_out = (10.0f32.powf(-d_r_out)).clamp(0.0, 2.0);
    let g_out = (10.0f32.powf(-d_g_out)).clamp(0.0, 2.0);
    let b_out = (10.0f32.powf(-d_b_out)).clamp(0.0, 2.0);

    (r_out, g_out, b_out)
}

/// Permuted Congruential Hash 2D
/// Primary Sensitometric Recording Emulsion Engine
/// Evaluated in Linear Rec.2020 working space prior to print enlargement/display target.
/// Pure photochemical color science: active DIR chemical crosstalk matrices and D-log H characteristic curves.
/// Strictly free of synthetic procedural noise or digital grain artifacts.
pub fn apply_recording_emulsion(
    r: f32,
    g: f32,
    b: f32,
    archetype: EmulsionArchetype,
    push_pull_ev: f32,
    filter_mode: EnlargerFilterMode,
) -> (f32, f32, f32) {
    if archetype == EmulsionArchetype::None {
        return (r, g, b);
    }

    // 1. Photographic Exposure Index / Push-Pull Trim (2^EV)
    let exposure_gain = 2.0f32.powf(push_pull_ev.clamp(-2.0, 2.0));
    let cur_r = (r * exposure_gain).max(0.0);
    let cur_g = (g * exposure_gain).max(0.0);
    let cur_b = (b * exposure_gain).max(0.0);

    match archetype {
        EmulsionArchetype::None => (r, g, b),

        // 1. Rochester Portrait 400 (C-41 Negative, Rochester NY)
        // Technical Reference: Kodak Publication E-4050 / E-4051 ("KODAK PROFESSIONAL PORTRA 400 Film"), Status M Densitometry.
        // Characteristics: Broad linear latitude (>3 decades log exposure), gentle midtone slope (gamma 0.90),
        // parallel CMY highlight shoulder roll-off (0.38), and inter-layer DIR cyan-magenta suppression softening skin tones.
        EmulsionArchetype::RochesterPortrait400 => {
            // DIR Chemical Coupling: red inhibits green (cyan-magenta separation), softening skin tones
            let (r_dir, g_dir, b_dir) = apply_dir_chemical_coupling(
                cur_r, cur_g, cur_b,
                -0.04, 0.02,
                -0.08, 0.02,
                0.02, -0.05,
            );

            // Sensitometric C-41 Portrait D-log H curve: gentle gamma 0.90, soft highlight shoulder roll-off (0.38)
            let r_curv = apply_film_characteristic_curve(r_dir, 0.90, 0.38, 0.08);
            let g_curv = apply_film_characteristic_curve(g_dir, 0.90, 0.38, 0.08);
            let b_curv = apply_film_characteristic_curve(b_dir, 0.90, 0.38, 0.08);

            (r_curv, g_curv, b_curv)
        }

        // 2. Ashigara Chrome 50 (E-6 Reversal, Minamiashigara, Japan)
        // Technical Reference: Fujifilm Technical Data Sheet AF3-960E ("FUJICHROME Velvia 50 Professional [RVP 50]"), Status A Densitometry.
        // Characteristics: High micro-contrast, steep midtone slope (gamma 1.10-1.12), deep D-max (>3.6),
        // crisp highlight threshold (shoulder 0.12), and strong inter-layer DIR edge effects isolating emeralds and deep sky blues.
        EmulsionArchetype::AshigaraChrome50 => {
            // DIR Couplers: high inter-layer edge effect, vivid emeralds and deep sky blues
            let (r_dir, g_dir, b_dir) = apply_dir_chemical_coupling(
                cur_r, cur_g, cur_b,
                -0.15, -0.08,
                -0.18, -0.08,
                -0.06, -0.12,
            );

            // E-6 Reversal D-log H: punchy gamma 1.10, crisp shoulder, restrained toe
            let r_curv = apply_film_characteristic_curve(r_dir, 1.10, 0.12, 0.14);
            let g_curv = apply_film_characteristic_curve(g_dir, 1.12, 0.10, 0.14);
            let b_curv = apply_film_characteristic_curve(b_dir, 1.11, 0.12, 0.16);

            (r_curv, g_curv, b_curv)
        }

        // 3. Rochester K-14 Chrome 64 (K-14 Non-Substantive Reversal, Rochester NY)
        // Technical Reference: Kodak Publication E-88 / CIS-57 ("KODAK EKTACHROME and KODACHROME Professional Films") & ResearchGate Sensitometric Layer Analysis.
        // Characteristics: Non-substantive reversal, asymmetric red layer sensitivity (gamma 1.05 / toe 0.08 vs blue 1.08 / toe 0.14),
        // creating the iconic warm golden highlight glow, punchy midtones, and deep ink-black shadows (D-max > 3.2).
        EmulsionArchetype::RochesterK14Chrome64 => {
            let (r_dir, g_dir, b_dir) = apply_dir_chemical_coupling(
                cur_r, cur_g, cur_b,
                -0.08, 0.06,
                -0.10, 0.04,
                0.08, -0.04,
            );

            // Non-substantive reversal warm golden highlight roll-off and rich shadow depth
            let r_curv = apply_film_characteristic_curve(r_dir, 1.05, 0.18, 0.08);
            let g_curv = apply_film_characteristic_curve(g_dir, 1.06, 0.16, 0.10);
            let b_curv = apply_film_characteristic_curve(b_dir, 1.08, 0.14, 0.14);

            (r_curv, g_curv, b_curv)
        }

        // 4. Rochester Cinema 500 (ECN-2 Tungsten, Rochester NY)
        // Technical Reference: Kodak Technical Data Sheet H-1-5219 / TI2647 ("KODAK VISION3 500T Color Negative Film 5219/7219"), Status M Densitometry.
        // Characteristics: 3200K tungsten motion picture negative, extreme 14+ stop latitude (-8 to +8 camera stops),
        // flat linear slope (gamma 0.82) designed for telecine transfer and digital color grading, with 5+ stops of highlight headroom (shoulder 0.45).
        EmulsionArchetype::RochesterCinema500 => {
            let (r_dir, g_dir, b_dir) = apply_dir_chemical_coupling(
                cur_r, cur_g, cur_b,
                -0.02, 0.01,
                -0.03, 0.01,
                0.01, -0.02,
            );

            // Low-contrast ECN-2 gamma 0.82 with broad highlight dynamic range
            let r_curv = apply_film_characteristic_curve(r_dir, 0.82, 0.45, 0.08);
            let g_curv = apply_film_characteristic_curve(g_dir, 0.82, 0.45, 0.08);
            let b_curv = apply_film_characteristic_curve(b_dir, 0.82, 0.45, 0.08);

            (r_curv, g_curv, b_curv)
        }

        // 5. Rochester Daylight 200 (C-41, Rochester NY)
        // Technical Reference: Kodak Publication E-7022 / CIS-124 ("KODAK GOLD 200 Film"), Status M Densitometry.
        // Characteristics: Consumer daylight negative with punchier midtone slope (gamma 1.12 R / 1.08 G / 0.98 B),
        // yellow dye suppression of blue, and warm amber saturation in sunlit highlights and skin tones.
        // 5. Rochester Daylight 200 (Merged into Rochester Portrait 400 per Story 3.11 spec)
        EmulsionArchetype::RochesterDaylight200 => {
            let (r_dir, g_dir, b_dir) = apply_dir_chemical_coupling(
                cur_r, cur_g, cur_b,
                -0.04, 0.02,
                -0.08, 0.02,
                0.02, -0.05,
            );

            let r_curv = apply_film_characteristic_curve(r_dir, 0.90, 0.38, 0.08);
            let g_curv = apply_film_characteristic_curve(g_dir, 0.90, 0.38, 0.08);
            let b_curv = apply_film_characteristic_curve(b_dir, 0.90, 0.38, 0.08);

            (r_curv, g_curv, b_curv)
        }

        // 6. Mobberley Silver 400 (Panchromatic B&W, Mobberley, Cheshire, UK)
        // Technical Reference: Ilford Technical Information Sheet ("ILFORD HP5 PLUS Professional Black and White Film"), ID-11/D-76 Developer (1+1).
        // Characteristics: Spectral sensitivity modulated by optical enlarger filtration (or neutral Rec.2020 luma),
        // classic G-bar 0.62 / gamma 1.08 across Zone III to Zone VIII, with clean collapse to monochromatic metallic silver.
        EmulsionArchetype::MobberleySilver400 => {
            let (wr, wg, wb) = filter_mode.spectral_weights();
            let pan_lum = wr * cur_r + wg * cur_g + wb * cur_b;
            let mono = apply_film_characteristic_curve(pan_lum, 1.08, 0.30, 0.25);

            (mono, mono, mono)
        }

        // 7. Cambridge Diffusion 100 (Instant Dye Diffusion, Kendall Square, Cambridge MA)
        // Technical Reference: Polaroid Corporation Technical Data Sheet (Polacolor ER / Type 669) & Edwin H. Land PNAS Publication.
        // Characteristics: Reversal dye-developer diffusion transfer, ~5-6 stops latitude, reflective TiO2 highlight base tone (0.025),
        // and pastel greens from phthalocyanine cyan dye couplers.
        EmulsionArchetype::CambridgeDiffusion100 => {
            let (r_dir, g_dir, b_dir) = apply_dir_chemical_coupling(
                cur_r, cur_g, cur_b,
                -0.03, 0.02,
                -0.04, 0.03,
                0.01, -0.03,
            );

            let r_curv = apply_film_characteristic_curve(r_dir, 0.98, 0.35, 0.12);
            let g_curv = apply_film_characteristic_curve(g_dir, 0.98, 0.35, 0.12);
            let b_curv = apply_film_characteristic_curve(b_dir, 0.98, 0.35, 0.12);

            // Reflective TiO2 base tone: gentle cream/pastel reflection in high densities (Linear Rec.2020 luma)
            let lum = 0.2627 * r_curv + 0.6780 * g_curv + 0.0593 * b_curv;
            let base_refl = 0.025 * (1.0 - lum).max(0.0);
            let r_tio2 = (r_curv + base_refl * 1.02).min(2.0);
            let g_tio2 = (g_curv + base_refl * 1.00).min(2.0);
            let b_tio2 = (b_curv + base_refl * 0.94).min(2.0);

            (r_tio2, g_tio2, b_tio2)
        }

        // 8. Leverkusen Portrait 160 (Agfa Agfacolor / Vista, Leverkusen, Germany)
        // Substantive European dye couplers, earthy ochre midtones, soft pastel greens.
        EmulsionArchetype::LeverkusenPortrait160 => {
            let (r_dir, g_dir, b_dir) = apply_dir_chemical_coupling(
                cur_r, cur_g, cur_b,
                -0.05, 0.01,
                -0.06, 0.02,
                0.02, -0.04,
            );
            let r_curv = apply_emulsion_s_curve(r_dir, 0.06, -0.02);
            let g_curv = apply_emulsion_s_curve(g_dir, 0.06, -0.02);
            let b_curv = apply_emulsion_s_curve(b_dir, 0.06, -0.02);

            (r_curv, g_curv, b_curv)
        }

        // 9. Ashigara Portrait 400 (Fujicolor Pro 400H, Minamiashigara, Japan)
        // Patented 4th color layer (cyan-sensitive inter-layer) suppressing fluorescent cast; airy skin tones.
        EmulsionArchetype::AshigaraPortrait400 => {
            let (r_dir, g_dir, b_dir) = apply_dir_chemical_coupling(
                cur_r, cur_g, cur_b,
                -0.06, 0.02,
                -0.12, 0.04,
                0.03, -0.08,
            );
            let r_curv = apply_emulsion_s_curve(r_dir, 0.08, 0.06);
            let g_curv = apply_emulsion_s_curve(g_dir, 0.08, 0.06);
            let b_curv = apply_emulsion_s_curve(b_dir, 0.08, 0.06);

            (r_curv, g_curv, b_curv)
        }

        // 10. Liguria Cinema 80 (Ferrania P30, Cairo Montenotte, Italy)
        // 1960s Italian Neorealist cinema stock; pure silver monochrome, punchy microcontrast.
        EmulsionArchetype::LiguriaCinema80 => {
            let (wr, wg, wb) = filter_mode.spectral_weights();
            let pan_lum = wr * cur_r + wg * cur_g + wb * cur_b;
            let mono = apply_emulsion_s_curve(pan_lum, 0.32, -0.08);

            (mono, mono, mono)
        }

        // 11. Baoding Silver 100 (Lucky SHD 100, Baoding, China)
        // Domestic cubic silver halide; absence of anti-halation undercoat produces specular halation bloom.
        EmulsionArchetype::BaodingSilver100 => {
            let (wr, wg, wb) = filter_mode.spectral_weights();
            let pan_lum = wr * cur_r + wg * cur_g + wb * cur_b;
            let base_mono = apply_emulsion_s_curve(pan_lum, 0.18, 0.10);
            let bloom = if pan_lum > 0.85 {
                let u = (pan_lum - 0.85) / 0.15;
                0.08 * u * u
            } else {
                0.0
            };
            let mono = (base_mono + bloom).min(1.0);

            (mono, mono, mono)
        }
    }
}
