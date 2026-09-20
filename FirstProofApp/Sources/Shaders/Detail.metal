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

#endif /* Detail_metal */
