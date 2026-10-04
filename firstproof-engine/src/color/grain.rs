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
//! 5. Isotropic 8-Tap Stencil: Circularly symmetric discrete Laplacian (4 orthogonal + 4 diagonal)
//!    eliminates directional cross-hatching artifacts.
//! 6. Continuous Cluster Synthesis: Grain cluster envelope scales with format magnification ($M$)
//!    using continuous value noise with cubic Hermite blending, decoupling spatial cluster diameter
//!    from high-frequency sensor noise extraction.

use crate::modes::{EmulsionArchetype, EmulsionGrainMode};

/// Deterministic 2D non-linear permutation polynomial hash for procedural physical grain seeding.
/// Eliminates directional Moiré lattice artifacts and trigonometric precision breakdown on full-res frames.
#[inline(always)]
pub fn procedural_grain_noise(u: f32, v: f32) -> f32 {
    let fract = |x: f32| x - x.floor();
    let px = 50.0 * fract(u * 0.3183099 + 0.71);
    let py = 50.0 * fract(v * 0.3183099 + 0.113);
    let val = fract(px * py * (px + py));
    -1.0 + 2.0 * val
}

/// Continuous 2D value noise with cubic Hermite interpolation for organic silver crystal clumping.
#[inline]
pub fn clustered_grain_noise(px: f32, py: f32) -> f32 {
    let ix = px.floor();
    let iy = py.floor();
    let fx = px - ix;
    let fy = py - iy;
    let hx = fx * fx * (3.0 - 2.0 * fx);
    let hy = fy * fy * (3.0 - 2.0 * fy);

    let n00 = procedural_grain_noise(ix, iy);
    let n10 = procedural_grain_noise(ix + 1.0, iy);
    let n01 = procedural_grain_noise(ix, iy + 1.0);
    let n11 = procedural_grain_noise(ix + 1.0, iy + 1.0);

    let top = n00 * (1.0 - hx) + n10 * hx;
    let bot = n01 * (1.0 - hx) + n11 * hx;
    top * (1.0 - hy) + bot * hy
}

/// Applies stochastic emulsion resynthesis directly within the 8-tap isotropic detail stencil.
///
/// Parameters:
/// - `r, g, b`: Center pixel in Linear Rec.2020
/// - `y_c`: Center photopic luminance (0.2627 R + 0.6780 G + 0.0593 B)
/// - `y_l, y_r, y_t, y_b`: Orthogonal neighborhood photopic luminances (1-texel pitch)
/// - `y_tl, y_tr, y_bl, y_br`: Diagonal neighborhood photopic luminances (1-texel pitch)
/// - `px, py`: Pixel coordinates for spatial grain cluster evaluation
/// - `grain_mode`: Off, SeedFromSensor, or SynthesizePhysical
/// - `cluster_size`: Physical grain cluster diameter in pixels (grain_size * format_mag)
/// - `grain_amount`: RMS density contrast / modulation depth (0.0 to 2.0, default 1.0)
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
    y_tl: f32,
    y_tr: f32,
    y_bl: f32,
    y_br: f32,
    px: f32,
    py: f32,
    grain_mode: EmulsionGrainMode,
    cluster_size: f32,
    grain_amount: f32,
    archetype: EmulsionArchetype,
) -> (f32, f32, f32) {
    if grain_mode == EmulsionGrainMode::Off {
        return (r, g, b);
    }

    // 1. Isotropic 9-point lowpass luminance (evaluated at strictly 1-pixel pitch to isolate sensor read noise):
    // Orthogonal taps (dist 1) weighted at 1/6 each, diagonal taps (dist sqrt(2)) weighted at 1/12 each.
    let y_lowpass = (1.0 / 6.0) * (y_l + y_r + y_t + y_b)
        + (1.0 / 12.0) * (y_tl + y_tr + y_bl + y_br);
    let delta_sensor = y_c - y_lowpass;

    // Sobel-Feldman 8-neighbor gradient magnitude to decouple macroscopic scene edges from stochastic noise
    let gx = ((y_tr + 2.0 * y_r + y_br) - (y_tl + 2.0 * y_l + y_bl)) * 0.125;
    let gy = ((y_bl + 2.0 * y_b + y_br) - (y_tl + 2.0 * y_t + y_tr)) * 0.125;
    let grad_mag = (gx * gx + gy * gy).sqrt();

    // Edge gradient decoupling: suppresses unsharp mask overshooting on optical edges
    let edge_suppress = 1.0 / (1.0 + 35.0 * grad_mag);
    let delta_noise = delta_sensor * edge_suppress;

    // Sensor amplification gain: maps true sensor photon noise into [-1.0, 1.0] nucleation field.
    let seed_gain = 20.0;
    let denom = y_c.max(0.0).sqrt().max(0.05);
    let sensor_seed = ((delta_noise / denom) * seed_gain).clamp(-1.0, 1.0);

    // 2. Stochastic silver crystal lattice carrier / dye diffusion
    let is_instant = archetype == EmulsionArchetype::CambridgeDiffusion100;
    let s = cluster_size.max(0.25);
    let proc_noise = if is_instant {
        // Instant dye diffusion transfer (DDT) through reagent pods:
        // Softer, fine-grained micro-diffusion mottling rather than crystalline silver halide
        0.60 * clustered_grain_noise(px / s, py / s)
            + 0.40 * clustered_grain_noise(px / (s * 1.8) + 17.3, py / (s * 1.8) + 31.7)
    } else if s > 1.0 {
        let t_cluster = ((s - 1.0) / 2.0).clamp(0.0, 1.0);
        let cluster = clustered_grain_noise(px / s, py / s);
        let micro = procedural_grain_noise(px, py);
        // Crystal-dense clump modulation: micro-crystal acutance is modulated by the cluster envelope,
        // preserving crisp high-frequency grain bite across every pixel while clumping organically.
        let clumped = micro * (0.60 + 0.40 * cluster) + 0.25 * cluster;
        (1.0 - t_cluster) * micro + t_cluster * clumped
    } else {
        procedural_grain_noise(px, py)
    };

    // 3. Emulsion grain nucleation
    let eta = match grain_mode {
        EmulsionGrainMode::Off => 0.0,
        EmulsionGrainMode::SeedFromSensor => {
            if is_instant {
                (0.55 * proc_noise + 0.20 * sensor_seed).clamp(-1.0, 1.0)
            } else {
                (proc_noise * (0.45 + 0.55 * sensor_seed.abs()) + 0.45 * sensor_seed).clamp(-1.0, 1.0)
            }
        }
        EmulsionGrainMode::SynthesizePhysical => {
            if is_instant {
                (0.85 * proc_noise + 0.05 * sensor_seed).clamp(-1.0, 1.0)
            } else {
                (0.90 * proc_noise + 0.10 * sensor_seed).clamp(-1.0, 1.0)
            }
        }
    };

    // 4. Nutting midtone density weighting: G(Y) = 4 * Y * (1 - Y)
    // Strictly anchored at G(0.0) == 0.0 and G(1.0) == 0.0
    let y_clamped = y_c.clamp(0.0, 1.0);
    let nutting = 4.0 * y_clamped * (1.0 - y_clamped);

    // Multiplicative optical transmittance modulation obeying Beer-Lambert law T = 10^(-D)
    let amount = grain_amount.clamp(0.0, 2.0);
    let delta = eta * nutting * 0.28 * amount;
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
