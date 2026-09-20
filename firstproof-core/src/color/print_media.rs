//! Curated Fine-Art Print Media Archetypes & Optical Enlarger Filtration
//!
//! Models the downstream darkroom printmaking stage where the captured frame
//! (clean digital RAW or chemical emulsion) is exposed onto authentic fine-art paper chemistry.
//!
//! Grounded in physical sensitometric D-log H paper curves, base paper spectral reflectance (W_base),
//! optical enlarger spectral filtration, and archival chemical darkroom toners.
//!
//! Bit-level dual-engine parity with Shaders.metal is strictly enforced.

pub use firstproof_types::modes::{EnlargerFilterMode, PrintMediaMode};


/// Applies paper grade (hardness) curve to a normalized luminance value [0.0, 1.0].
/// Grade 0 = normal (Grade 2). -2.0 = Extra Soft (Grade 0), +2.0 = Ultra Hard (Grade 4/5).
/// Uses a smooth polynomial S-curve preserving 0.0, 0.5, and 1.0 with strictly finite derivatives everywhere.
#[inline]
pub fn apply_paper_grade(val: f32, grade: f32) -> f32 {
    if grade.abs() < 0.001 {
        return val;
    }
    let alpha = 0.40 * (grade / 2.0);
    (val + alpha * val * (1.0 - val) * (2.0 * val - 1.0)).clamp(0.0, 1.0)
}

/// Applies paper base warmth (W_base reflectance trim) to RGB.
/// warmth: -100.0 (cool barium sulfate optical brightener) to +100.0 (warm unbleached cotton rag).
/// Multiplicative paper base reflectance: true blacks stay 0.0, highlights naturally reflect paper substrate.
#[inline]
pub fn apply_base_warmth(r: f32, g: f32, b: f32, warmth: f32) -> (f32, f32, f32) {
    if warmth.abs() < 0.001 {
        return (r, g, b);
    }
    let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let highlight_weight = lum.powf(0.6).clamp(0.0, 1.0);
    let factor = (warmth / 100.0) * highlight_weight;

    let (r_mult, g_mult, b_mult) = if factor > 0.0 {
        // Warm unbleached natural fiber: gentle amber/cream lift
        (
            1.0 + factor * 0.060,
            1.0 + factor * 0.025,
            1.0 - factor * 0.045,
        )
    } else {
        // Cool optical brightener: subtle cool barium sulfate blue lift
        let abs_f = -factor;
        (
            1.0 - abs_f * 0.025,
            1.0 - abs_f * 0.008,
            1.0 + abs_f * 0.045,
        )
    };

    ((r * r_mult).clamp(0.0, 1.0), (g * g_mult).clamp(0.0, 1.0), (b * b_mult).clamp(0.0, 1.0))
}

/// Core sensitometric print media transform.
/// Bit-level parity with applyPrintMedia in Shaders.metal.
pub fn apply_print_media(
    r: f32,
    g: f32,
    b: f32,
    media: PrintMediaMode,
    filter: EnlargerFilterMode,
    paper_grade: f32,
    base_warmth: f32,
    process_depth: f32,
) -> (f32, f32, f32) {
    if media == PrintMediaMode::None {
        return (r, g, b);
    }

    let depth = (process_depth / 100.0).clamp(0.0, 1.0);

    match media {
        PrintMediaMode::None => (r, g, b),

        // 1. Gallery Chromogenic Type-C (RA-4)
        // Creamy highlight roll-off, natural skin tone density, organic CMY dye coupling.
        PrintMediaMode::ChromogenicTypeC => {
            let r_graded = apply_paper_grade(r, paper_grade);
            let g_graded = apply_paper_grade(g, paper_grade);
            let b_graded = apply_paper_grade(b, paper_grade);

            // Sensitometric highlight compression: smooth cubic Hermite soft-shoulder roll-off
            let compress = |c: f32| -> f32 {
                if c > 0.70 {
                    let u = ((c - 0.70) / 0.30).clamp(0.0, 1.0);
                    c - 0.045 * depth * u * u * (3.0 - 2.0 * u)
                } else {
                    c
                }
            };
            let r_p = compress(r_graded);
            let g_p = compress(g_graded);
            let b_p = compress(b_graded);

            // Natural coupled dye saturation: 0% = gentle pastel 0.70x -> 100% = rich archival 1.05x
            let lum = 0.2126 * r_p + 0.7152 * g_p + 0.0722 * b_p;
            let sat_scale = 0.70 + 0.35 * depth;
            let mut r_out = (lum + (r_p - lum) * sat_scale).clamp(0.0, 1.0);
            let mut g_out = (lum + (g_p - lum) * sat_scale).clamp(0.0, 1.0);
            let mut b_out = (lum + (b_p - lum) * sat_scale).clamp(0.0, 1.0);

            // Organic dye coupling: subtle density retention in midtones/shadows
            let dye_density = 0.04 * (1.0 - lum).max(0.0) * depth;
            r_out = (r_out * (1.0 - dye_density * 0.4)).clamp(0.0, 1.0);
            g_out = (g_out * (1.0 - dye_density * 0.6)).clamp(0.0, 1.0);
            b_out = (b_out * (1.0 - dye_density * 0.9)).clamp(0.0, 1.0);

            apply_base_warmth(r_out, g_out, b_out, base_warmth)
        }

        // 2. Silver Dye-Bleach Classic (Azo Reversal)
        // Deep shadow density (Dmax > 3.0), punchy micro-contrast, vibrant pure azo primary separation.
        PrintMediaMode::SilverDyeBleach => {
            let r_graded = apply_paper_grade(r, paper_grade);
            let g_graded = apply_paper_grade(g, paper_grade);
            let b_graded = apply_paper_grade(b, paper_grade);

            // Smooth continuous shadow deepening across all luminance
            let shadow_pow = 1.0 + 0.12 * depth;
            let r_p = r_graded.powf(shadow_pow);
            let g_p = g_graded.powf(shadow_pow);
            let b_p = b_graded.powf(shadow_pow);

            // Pure azo dye saturation: 0% = 0.80x, 100% = 1.10x
            let lum = 0.2126 * r_p + 0.7152 * g_p + 0.0722 * b_p;
            let azo_sat = 0.80 + 0.30 * depth;
            let r_out = (lum + (r_p - lum) * azo_sat).clamp(0.0, 1.0);
            let g_out = (lum + (g_p - lum) * azo_sat).clamp(0.0, 1.0);
            let b_out = (lum + (b_p - lum) * azo_sat).clamp(0.0, 1.0);

            apply_base_warmth(r_out, g_out, b_out, base_warmth)
        }

        // 3. Archival Fine-Art Cotton Rag (Mineral Pigment)
        // Velvety matte shadow floor (Dmax ~ 1.65, black floor ~0.045), unbleached cotton base warmth.
        // Fiber surface scatter: lifts shadows without compressing specular highlights.
        PrintMediaMode::FineArtCottonRag => {
            let r_graded = apply_paper_grade(r, paper_grade);
            let g_graded = apply_paper_grade(g, paper_grade);
            let b_graded = apply_paper_grade(b, paper_grade);

            let matte_floor = 0.055 - 0.015 * depth;
            let r_p = r_graded + matte_floor * (1.0 - r_graded) * (1.0 - r_graded);
            let g_p = g_graded + matte_floor * (1.0 - g_graded) * (1.0 - g_graded);
            let b_p = b_graded + matte_floor * (1.0 - b_graded) * (1.0 - b_graded);

            let lum = 0.2126 * r_p + 0.7152 * g_p + 0.0722 * b_p;
            let pigment_sat = 0.75 + 0.25 * depth;
            let r_out = (lum + (r_p - lum) * pigment_sat).clamp(0.0, 1.0);
            let g_out = (lum + (g_p - lum) * pigment_sat).clamp(0.0, 1.0);
            let b_out = (lum + (b_p - lum) * pigment_sat).clamp(0.0, 1.0);

            apply_base_warmth(r_out, g_out, b_out, base_warmth + 15.0)
        }

        // 4. Baryta Silver Gelatin FB
        // Neutral barium sulfate base. Smooth continuous Dmax boost without any threshold knees or banding.
        PrintMediaMode::BarytaSilverGelatin => {
            let (wr, wg, wb) = filter.spectral_weights();
            let raw_lum = (wr * r + wg * g + wb * b).clamp(0.0, 1.0);
            let silver = apply_paper_grade(raw_lum, paper_grade);

            let dmax_gamma = 1.0 + 0.08 * depth;
            let silver_toned = silver.powf(dmax_gamma);

            apply_base_warmth(silver_toned, silver_toned, silver_toned, base_warmth)
        }

        // 5. Warmtone Chlorobromide (Selenium Toned)
        // Deep velvety Dmax with subtle cool plum-black depths and warm charcoal midtones.
        // Uses smooth parabolic harmonic bell curve for midtone dye deposition.
        PrintMediaMode::WarmtoneChlorobromide => {
            let (wr, wg, wb) = filter.spectral_weights();
            let raw_lum = (wr * r + wg * g + wb * b).clamp(0.0, 1.0);
            let silver = apply_paper_grade(raw_lum, paper_grade);

            let shadow_pow = 1.0 + 0.10 * depth;
            let silver_deep = silver.powf(shadow_pow);

            let mid_mask = 4.0 * silver * (1.0 - silver);
            let shadow_density = (1.0 - silver).max(0.0);

            let r_tone = silver_deep + depth * (mid_mask * 0.030 + shadow_density * 0.020);
            let g_tone = silver_deep + depth * (mid_mask * 0.012 - shadow_density * 0.015);
            let b_tone = silver_deep + depth * (-mid_mask * 0.018 + shadow_density * 0.030);

            let r_toned = r_tone.clamp(0.0, 1.0);
            let g_toned = g_tone.clamp(0.0, 1.0);
            let b_toned = b_tone.clamp(0.0, 1.0);

            apply_base_warmth(r_toned, g_toned, b_toned, base_warmth)
        }

        // 6. Platinum / Palladium (Noble Metal Cotton Rag)
        // Gentle warm noble-metal midtone luminescence with soft matte fiber shadow floor.
        PrintMediaMode::PlatinumPalladium => {
            let (wr, wg, wb) = filter.spectral_weights();
            let raw_lum = (wr * r + wg * g + wb * b).clamp(0.0, 1.0);
            let silver = apply_paper_grade(raw_lum, paper_grade);

            let matte_floor = 0.035 * depth;
            let silver_base = matte_floor + (1.0 - matte_floor) * silver;

            let warm_lift = (1.0 - silver_base.powf(0.6)).max(0.0) * depth;
            let r_tone = silver_base + warm_lift * 0.045;
            let g_tone = silver_base + warm_lift * 0.028;
            let b_tone = silver_base - warm_lift * 0.020;

            let r_toned = r_tone.clamp(0.0, 1.0);
            let g_toned = g_tone.clamp(0.0, 1.0);
            let b_toned = b_tone.clamp(0.0, 1.0);

            apply_base_warmth(r_toned, g_toned, b_toned, base_warmth + 10.0 * depth)
        }

        // 7. Historic Cyanotype (1842 Prussian Blue Iron Salts)
        // Authentic subtractive ferric ferrocyanide pigment chemistry.
        PrintMediaMode::Cyanotype => {
            let (wr, wg, wb) = filter.spectral_weights();
            let raw_lum = (wr * r + wg * g + wb * b).clamp(0.0, 1.0);
            let silver = apply_paper_grade(raw_lum, paper_grade);

            let density = 1.0 - silver;

            // Subtractive Prussian blue: Red & Green absorbed by pigment, Blue transmits
            let r_p = silver * (1.0 - 0.75 * depth * density);
            let g_p = silver * (1.0 - 0.35 * depth * density) + 0.02 * depth * density;
            let b_p = silver * (1.0 - 0.05 * depth * density) + 0.07 * depth * density;

            let r_toned = r_p.clamp(0.0, 1.0);
            let g_toned = g_p.clamp(0.0, 1.0);
            let b_toned = b_p.clamp(0.0, 1.0);

            apply_base_warmth(r_toned, g_toned, b_toned, base_warmth + 8.0 * depth)
        }
    }
}
