/**
 * Detail, Sharpening & Noise Reduction (8-Tap Isotropic Discrete Laplacian)
 * Transcribed 1:1 from FirstProofApp/Sources/Shaders/Detail.metal
 */
export const DETAIL_GLSL = `
vec3 applyDetailAndSharpening(
    vec3 color,
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
    return clamp(color + vec3(delta_adj), 0.0, 1.0);
}
`;
