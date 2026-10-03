//! Stochastic Emulsion Resynthesis & Physical Optical Grain Engine
//!
//! Grounded in sensitometric principles:
//! 1. Sensitometric Domain Placement: Grain executes in Wide-Gamut Linear Rec.2020
//!    prior to terminal display tone mapping, preventing shadow noise amplification.
//! 2. Noise Transmutation (Zero Plastic Denoising): High-ISO photon arrival variations
//!    (delta_sensor = Y - Y_lowpass) serve as the physical nucleation seed for silver
//!    crystals or subtractive dye clouds.
//! 3. Nutting's Law of Granularity: Grain density peaks at midtone D ~ 1.0 via G(Y) = 4Y(1-Y)
//!    and smoothly extinguishes at Dmax (shadows) and Dmin (highlights).
//! 4. Chrominance Phase Consolidation: High-frequency chrominance confetti is transferred
//!    to luminance optical density, preserving 100% of spatial acutance without spatial blurring.

use crate::modes::{EmulsionArchetype, EmulsionGrainMode};

/// Deterministic 2D continuous hash for procedural physical grain seeding.
#[inline(always)]
pub fn procedural_grain_noise(u: f32, v: f32) -> f32 {
    let p0 = u * 12.9898 + v * 78.233;
    let sin_val = (p0.sin() * 43_758.547_f32).fract();
    sin_val * 2.0 - 1.0
}

/// Applies stochastic emulsion resynthesis directly within the 4-tap detail stencil.
///
/// Parameters:
/// - `r, g, b`: Center pixel in Linear Rec.2020
/// - `y_c`: Center photopic luminance (0.2627 R + 0.6780 G + 0.0593 B)
/// - `y_lowpass`: Mean neighborhood photopic luminance ((y_l + y_r + y_t + y_b) / 4.0)
/// - `u, v`: Normalized texture coordinates (for procedural spatial variation)
/// - `grain_mode`: Off, SeedFromSensor, or SynthesizePhysical
/// - `grain_scale`: Crystal cluster radius multiplier (0.5 to 2.0, default 1.0)
/// - `archetype`: Active emulsion archetype (metallic silver vs organic dye clouds)
#[inline]
pub fn apply_stochastic_emulsion_grain(
    r: f32,
    g: f32,
    b: f32,
    y_c: f32,
    y_l: f32,
    y_r: f32,
    y_t: f32,
    y_b: f32,
    u: f32,
    v: f32,
    grain_mode: EmulsionGrainMode,
    grain_scale: f32,
    archetype: EmulsionArchetype,
) -> (f32, f32, f32) {
    if grain_mode == EmulsionGrainMode::Off {
        return (r, g, b);
    }

    let scale = grain_scale.clamp(0.2, 3.0);

    // 1. High-frequency sensor residual: delta_sensor = Y - Y_lowpass
    let y_lowpass = 0.25 * (y_l + y_r + y_t + y_b);
    let delta_sensor = y_c - y_lowpass;

    // Sobel gradient magnitude to decouple macroscopic scene edges from stochastic noise
    let gx = (y_r - y_l) * 0.5;
    let gy = (y_b - y_t) * 0.5;
    let grad_mag = (gx * gx + gy * gy).sqrt();

    // Edge gradient decoupling: suppresses unsharp mask overshooting on optical edges
    let edge_suppress = 1.0 / (1.0 + 35.0 * grad_mag);
    let delta_noise = delta_sensor * edge_suppress;

    // Sensor amplification gain: maps true sensor photon noise into [-1.0, 1.0] nucleation field.
    // Decoupled from scale to eliminate quadratic compounding across the slider range.
    let seed_gain = 20.0;
    let denom = y_c.max(0.0).sqrt().max(0.05);
    let sensor_seed = ((delta_noise / denom) * seed_gain).clamp(-1.0, 1.0);

    // 2. Stochastic silver crystal lattice carrier
    let proc_noise = procedural_grain_noise(u, v);

    // 3. Emulsion grain nucleation
    let eta = match grain_mode {
        EmulsionGrainMode::Off => 0.0,
        EmulsionGrainMode::SeedFromSensor => {
            // Sensor photon noise modulates crystal lattice without edge oversharpening
            (proc_noise * (0.45 + 0.55 * sensor_seed.abs()) + 0.45 * sensor_seed).clamp(-1.0, 1.0)
        }
        EmulsionGrainMode::SynthesizePhysical => {
            // Pure physical crystal acutance with subtle sensor foundation
            (0.75 * proc_noise + 0.25 * sensor_seed).clamp(-1.0, 1.0)
        }
    };

    // 4. Nutting midtone density weighting: G(Y) = 4 * Y * (1 - Y)
    // Strictly anchored at G(0.0) == 0.0 and G(1.0) == 0.0
    let y_clamped = y_c.clamp(0.0, 1.0);
    let nutting = 4.0 * y_clamped * (1.0 - y_clamped);

    // Multiplicative optical transmittance modulation obeying Beer-Lambert law T = 10^(-D)
    let delta = eta * nutting * 0.28 * scale;
    let y_pixel = 0.2627 * r + 0.6780 * g + 0.0593 * b;

    // 5. Emulsion-specific remapping
    if archetype.is_monochrome() {
        // Metallic Silver Crystals: Pure achromatic optical density modulation
        let y_res = (y_pixel * (1.0 + delta)).max(0.0);
        (y_res, y_res, y_res)
    } else {
        // Organic Dye Clouds: Multiplicative optical transmittance with subtle chrominance phase consolidation
        let chroma_damp = 1.0 - 0.10 * nutting;
        let r_c = y_pixel + chroma_damp * (r - y_pixel);
        let g_c = y_pixel + chroma_damp * (g - y_pixel);
        let b_c = y_pixel + chroma_damp * (b - y_pixel);

        let r_res = (r_c * (1.0 + delta)).max(0.0);
        let g_res = (g_c * (1.0 + delta)).max(0.0);
        let b_res = (b_c * (1.0 + delta)).max(0.0);

        (r_res, g_res, b_res)
    }
}
