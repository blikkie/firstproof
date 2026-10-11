#ifndef Detail_metal
#define Detail_metal

#include <metal_stdlib>
using namespace metal;

// MARK: - Detail, Sharpening & Noise Reduction (8-Tap Isotropic Discrete Laplacian)
inline float3 applyDetailAndSharpening(
    float3 color,
    float y_c,
    float y_l, float y_r, float y_t, float y_b,
    float y_tl, float y_tr, float y_bl, float y_br,
    float amount,
    float masking,
    float noiseReduction
) {
    if (amount <= 0.0 && noiseReduction <= 0.0) {
        return color;
    }

    // Isotropic 9-point discrete Laplacian:
    // Orthogonal taps (dist 1) weighted at 1/6 each, diagonal taps (dist sqrt(2)) weighted at 1/12 each.
    float y_neighbors = (1.0 / 6.0) * (y_l + y_r + y_t + y_b)
        + (1.0 / 12.0) * (y_tl + y_tr + y_bl + y_br);
    float delta_y = y_c - y_neighbors;

    // Sobel-Feldman 8-neighbor gradient magnitude (isotropic edge acutance)
    float gx = ((y_tr + 2.0 * y_r + y_br) - (y_tl + 2.0 * y_l + y_bl)) * 0.125;
    float gy = ((y_bl + 2.0 * y_b + y_br) - (y_tl + 2.0 * y_t + y_tr)) * 0.125;
    float grad_mag = sqrt(gx * gx + gy * gy);

    // Edge masking weight:
    // When masking <= 0, w_edge = 1.0 (uniform sharpening across entire image).
    // When masking > 0, flat regions (grad_mag < threshold) are smoothly suppressed towards 0.
    // Derivation: Normalized 8-tap Sobel gradient magnitude has a sensor noise floor < 0.01,
    // microtexture at 0.01-0.05, and distinct subject edges at >= 0.08.
    // The ceiling 0.08 isolates subject edges from noise; the 1.5 power taper provides fine control at low slider values.
    float w_edge = 1.0;
    if (masking > 0.0) {
        float m = clamp(masking / 100.0, 0.0, 1.0);
        float threshold = pow(m, 1.5) * 0.08;
        w_edge = smoothstep(0.0, max(threshold, 1e-4), grad_mag);
    }

    // Photographically calibrated gain: up to 1.5x unsharp mask detail delta
    float sharp_gain = (amount / 100.0) * 1.5;
    float sharp_term = sharp_gain * w_edge * delta_y;
    float nr_term = (noiseReduction / 100.0) * (1.0 - w_edge) * delta_y;

    float delta_adj = sharp_term - nr_term;
    return clamp(color + delta_adj, 0.0, 1.0);
}

// MARK: - Stochastic Emulsion Resynthesis & Physical Optical Grain Engine
inline float proceduralGrainNoise(float u, float v) {
    float2 p = 50.0f * fract(float2(u, v) * 0.3183099f + float2(0.71f, 0.113f));
    return -1.0f + 2.0f * fract(p.x * p.y * (p.x + p.y));
}

inline float clusteredGrainNoise(float2 p) {
    float2 i = floor(p);
    float2 f = fract(p);
    float2 u = f * f * (3.0 - 2.0 * f);

    float a = proceduralGrainNoise(i.x, i.y);
    float b = proceduralGrainNoise(i.x + 1.0, i.y);
    float c = proceduralGrainNoise(i.x, i.y + 1.0);
    float d = proceduralGrainNoise(i.x + 1.0, i.y + 1.0);

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

inline float evalGrainProcNoise(float2 pos, float s, bool is_instant) {
    if (is_instant) {
        return 0.60 * clusteredGrainNoise(pos / s)
            + 0.40 * clusteredGrainNoise(pos / (s * 1.8) + float2(17.3, 31.7));
    } else if (s > 1.0) {
        float t_cluster = clamp((s - 1.0) / 2.0, 0.0, 1.0);
        float cluster = clusteredGrainNoise(pos / s);
        float micro = proceduralGrainNoise(pos.x, pos.y);
        float clumped = micro * (0.60 + 0.40 * cluster) + 0.25 * cluster;
        return mix(micro, clumped, t_cluster);
    } else {
        return proceduralGrainNoise(pos.x, pos.y);
    }
}

inline float evalGrainEta(float proc_val, float sensor_seed, int grain_mode, bool is_instant) {
    if (grain_mode == 1) { // SeedFromSensor
        if (is_instant) {
            return clamp(0.55 * proc_val + 0.20 * sensor_seed, -1.0, 1.0);
        } else {
            return clamp(proc_val * (0.45 + 0.55 * abs(sensor_seed)) + 0.45 * sensor_seed, -1.0, 1.0);
        }
    } else if (grain_mode == 2) { // SynthesizePhysical
        if (is_instant) {
            return clamp(0.85 * proc_val + 0.05 * sensor_seed, -1.0, 1.0);
        } else {
            return clamp(0.90 * proc_val + 0.10 * sensor_seed, -1.0, 1.0);
        }
    }
    return 0.0;
}

inline float3 applyStochasticEmulsionGrain(
    float3 color,
    float y_c,
    float y_l,
    float y_r,
    float y_t,
    float y_b,
    float y_tl,
    float y_tr,
    float y_bl,
    float y_br,
    float2 p,
    int grain_mode,
    float cluster_size,
    float grain_amount,
    int archetype
) {
    if (grain_mode == 0) {
        return color;
    }

    float amount = clamp(grain_amount, 0.0, 2.0);
    if (amount < 0.001) {
        return color;
    }

    // 1. Isotropic 9-point lowpass luminance (evaluated at 1-pixel pitch to isolate sensor read noise)
    float y_lowpass = (1.0 / 6.0) * (y_l + y_r + y_t + y_b)
        + (1.0 / 12.0) * (y_tl + y_tr + y_bl + y_br);
    float delta_sensor = y_c - y_lowpass;

    // Sobel-Feldman 8-neighbor gradient magnitude
    float gx = ((y_tr + 2.0 * y_r + y_br) - (y_tl + 2.0 * y_l + y_bl)) * 0.125;
    float gy = ((y_bl + 2.0 * y_b + y_br) - (y_tl + 2.0 * y_t + y_tr)) * 0.125;
    float grad_mag = sqrt(gx * gx + gy * gy);

    float edge_suppress = 1.0 / (1.0 + 35.0 * grad_mag);
    float delta_noise = delta_sensor * edge_suppress;

    float seed_gain = 20.0;
    float denom = max(sqrt(max(y_c, 0.0)), 0.05);
    float sensor_seed = clamp((delta_noise / denom) * seed_gain, -1.0, 1.0);

    // 2. Stochastic carrier: Polaroid DDT dye diffusion mottling vs silver crystal lattice
    bool is_instant = (archetype == 7);
    float s = max(cluster_size, 0.25);
    float proc_common = evalGrainProcNoise(p, s, is_instant);
    float eta_common = evalGrainEta(proc_common, sensor_seed, grain_mode, is_instant);

    // 3. Nutting midtone density weighting: G(Y) = 4 * Y * (1 - Y)
    float y_clamped = clamp(y_c, 0.0, 1.0);
    float nutting = 4.0 * y_clamped * (1.0 - y_clamped);

    // 4. Optical transmittance modulation
    float y_pixel = dot(color, float3(0.2627, 0.6780, 0.0593));

    bool is_monochrome = (archetype == 6 || archetype == 10 || archetype == 11);
    if (is_monochrome) {
        // Metallic Silver: Pure achromatic optical density modulation (R = G = B)
        float delta = eta_common * nutting * 0.28 * amount;
        float y_res = max(y_pixel * (1.0 + delta), 0.0);
        return float3(y_res);
    } else {
        // Subtractive 3-Layer Dye Cloud Decorrelation (Cyan -> Red, Magenta -> Green, Yellow -> Blue)
        float proc_r = evalGrainProcNoise(p + float2(17.3, 31.7), s, is_instant);
        float proc_g = evalGrainProcNoise(p + float2(73.1, 19.9), s, is_instant);
        float proc_b = evalGrainProcNoise(p + float2(43.7, 89.3), s, is_instant);

        float eta_r = evalGrainEta(proc_r, sensor_seed, grain_mode, is_instant);
        float eta_g = evalGrainEta(proc_g, sensor_seed, grain_mode, is_instant);
        float eta_b = evalGrainEta(proc_b, sensor_seed, grain_mode, is_instant);

        // Mix 75% common photopic luminance carrier with 25% independent subtractive layer phase
        // Energy-preserving Euclidean normalization: 1.0 / sqrt(0.75^2 + 0.25^2) ≈ 1.264911
        float beta = 0.25;
        float norm = 1.264911;
        float eta_final_r = mix(eta_common, eta_r, beta) * norm;
        float eta_final_g = mix(eta_common, eta_g, beta) * norm;
        float eta_final_b = mix(eta_common, eta_b, beta) * norm;

        float scale = nutting * 0.28 * amount;
        float delta_r = eta_final_r * scale;
        float delta_g = eta_final_g * scale;
        float delta_b = eta_final_b * scale;

        float chroma_damp = 1.0 - 0.10 * nutting;
        float r_c = y_pixel + chroma_damp * (color.r - y_pixel);
        float g_c = y_pixel + chroma_damp * (color.g - y_pixel);
        float b_c = y_pixel + chroma_damp * (color.b - y_pixel);

        float r_res = max(r_c * (1.0 + delta_r), 0.0);
        float g_res = max(g_c * (1.0 + delta_g), 0.0);
        float b_res = max(b_c * (1.0 + delta_b), 0.0);

        return float3(r_res, g_res, b_res);
    }
}

#endif /* Detail_metal */
