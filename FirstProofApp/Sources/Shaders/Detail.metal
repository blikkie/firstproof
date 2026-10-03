#ifndef Detail_metal
#define Detail_metal

#include <metal_stdlib>
using namespace metal;

// MARK: - Detail, Sharpening & Noise Reduction (Luminance USM with Edge Masking)
inline float3 applyDetailAndSharpening(
    float3 color,
    float y_c,
    float y_l, float y_r, float y_t, float y_b,
    float amount,
    float masking,
    float noiseReduction
) {
    if (amount <= 0.0 && noiseReduction <= 0.0) {
        return color;
    }

    // Unsharp mask: Laplacian high-frequency detail
    float y_neighbors = 0.25 * (y_l + y_r + y_t + y_b);
    float delta_y = y_c - y_neighbors;

    // Sobel gradient magnitude
    float gx = (y_r - y_l) * 0.5;
    float gy = (y_b - y_t) * 0.5;
    float grad_mag = sqrt(gx * gx + gy * gy);

    // Edge masking weight:
    // When masking <= 0, w_edge = 1.0 (uniform sharpening across entire image).
    // When masking > 0, flat regions (grad_mag < threshold) are smoothly suppressed towards 0.
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
    float p0 = u * 12.9898 + v * 78.233;
    float sin_val = fract(sin(p0) * 43758.5453);
    return sin_val * 2.0 - 1.0;
}

inline float3 applyStochasticEmulsionGrain(
    float3 color,
    float y_c,
    float y_l,
    float y_r,
    float y_t,
    float y_b,
    float2 uv,
    int grain_mode,
    float grain_scale,
    int archetype
) {
    if (grain_mode == 0) {
        return color;
    }

    float scale = clamp(grain_scale, 0.2, 3.0);
    float y_lowpass = 0.25 * (y_l + y_r + y_t + y_b);
    float delta_sensor = y_c - y_lowpass;

    float gx = (y_r - y_l) * 0.5;
    float gy = (y_b - y_t) * 0.5;
    float grad_mag = sqrt(gx * gx + gy * gy);

    float edge_suppress = 1.0 / (1.0 + 35.0 * grad_mag);
    float delta_noise = delta_sensor * edge_suppress;

    float seed_gain = 20.0;
    float denom = max(sqrt(max(y_c, 0.0)), 0.05);
    float sensor_seed = clamp((delta_noise / denom) * seed_gain, -1.0, 1.0);

    float proc_noise = proceduralGrainNoise(uv.x, uv.y);

    float eta = 0.0;
    if (grain_mode == 1) { // SeedFromSensor
        eta = clamp(proc_noise * (0.45 + 0.55 * abs(sensor_seed)) + 0.45 * sensor_seed, -1.0, 1.0);
    } else if (grain_mode == 2) { // SynthesizePhysical
        eta = clamp(0.75 * proc_noise + 0.25 * sensor_seed, -1.0, 1.0);
    }

    float y_clamped = clamp(y_c, 0.0, 1.0);
    float nutting = 4.0 * y_clamped * (1.0 - y_clamped);

    // Multiplicative optical transmittance modulation obeying Beer-Lambert law T = 10^(-D)
    float delta = eta * nutting * 0.28 * scale;
    float y_pixel = dot(color, float3(0.2627, 0.6780, 0.0593));

    bool is_monochrome = (archetype == 6 || archetype == 10 || archetype == 11);
    if (is_monochrome) {
        float y_res = max(y_pixel * (1.0 + delta), 0.0);
        return float3(y_res);
    } else {
        float chroma_damp = 1.0 - 0.10 * nutting;
        float r_c = y_pixel + chroma_damp * (color.r - y_pixel);
        float g_c = y_pixel + chroma_damp * (color.g - y_pixel);
        float b_c = y_pixel + chroma_damp * (color.b - y_pixel);

        float r_res = max(r_c * (1.0 + delta), 0.0);
        float g_res = max(g_c * (1.0 + delta), 0.0);
        float b_res = max(b_c * (1.0 + delta), 0.0);

        return float3(r_res, g_res, b_res);
    }
}

#endif /* Detail_metal */
