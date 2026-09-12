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

    // Photographically calibrated gain: up to 3.5x boost at Amount=100 for crisp, unmistakable bite
    float sharp_gain = (amount / 100.0) * 3.5;
    float sharp_term = sharp_gain * w_edge * delta_y;
    float nr_term = (noiseReduction / 100.0) * (1.0 - w_edge) * delta_y;
    float delta_adj = sharp_term - nr_term;

    float k = clamp((y_c + delta_adj) / max(y_c, 1e-5), 0.0, 3.0);
    return clamp(color * k, 0.0, 1.0);
}

// MARK: - Procedural Film Grain (Luminance-Dependent Silver Halide Simulation)
inline float grainLuminanceWeight(float lum) {
    float bell = 4.0 * lum * (1.0 - lum);
    return max(0.08 * (1.0 - lum), bell);
}

// 32-bit PCG integer hash (period > 4.29 billion, zero periodic tiling)
inline float pcgHash2D(int2 p) {
    uint2 u = as_type<uint2>(p);
    uint h = u.x * 374761393u + u.y * 668265263u + 0x9E3779B9u;
    h = (h ^ (h >> 13)) * 1274126177u;
    h = h ^ (h >> 16);
    return float(h) * (1.0 / 4294967296.0);
}

// C2-continuous Hermite smooth value noise for organic dye cloud clumping
inline float smoothNoise2D(float2 p) {
    int2 i = int2(floor(p));
    float2 f = p - floor(p);
    // Quintic Hermite interpolation: 6f^5 - 15f^4 + 10f^3
    float2 u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);

    float a = pcgHash2D(i);
    float b = pcgHash2D(i + int2(1, 0));
    float c = pcgHash2D(i + int2(0, 1));
    float d = pcgHash2D(i + int2(1, 1));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

inline float organicGrainNoise(float2 pxCoord, float scale, float roughness) {
    float fine = (pcgHash2D(int2(floor(pxCoord))) - 0.5) * 2.0;
    float clump = (smoothNoise2D(pxCoord / scale) - 0.5) * 2.0;
    float micro = (smoothNoise2D((pxCoord + float2(31.7, 47.3)) / (scale * 0.5)) - 0.5) * 2.0;
    float rFactor = clamp(roughness / 100.0, 0.0, 1.0);
    return clump * 0.45 + micro * (0.25 * rFactor) + fine * (0.35 + 0.25 * rFactor);
}

inline float3 applyProceduralFilmGrain(
    float3 color,
    float2 uv,
    float2 textureSize,
    float amount,
    float size,
    float roughness,
    float chroma
) {
    if (amount <= 0.0) {
        return color;
    }
    
    float lum = dot(color, float3(0.2126, 0.7152, 0.0722));
    float weight = grainLuminanceWeight(clamp(lum, 0.0, 1.0));
    
    float grainScale = max(size, 0.4);
    float2 pixelCoord = uv * textureSize;
    
    float monoNoise = organicGrainNoise(pixelCoord, grainScale, roughness);
    
    float3 grainOffset;
    if (chroma > 0.0) {
        float cT = clamp(chroma / 100.0, 0.0, 1.0);
        // Equal-power crossfade: cos^2(theta) + sin^2(theta) = 1.0 preserves total variance
        float theta = cT * 1.5707963;
        float wMono = cos(theta);
        // 1.25x CSF gain compensates for cross-channel luminance cancellation
        float wColor = sin(theta) * 1.25;

        // Physical tri-pack color emulsion layer modeling:
        // Red (bottom layer): finer grain (scale * 0.9)
        // Green (middle layer): medium grain (scale * 1.0)
        // Blue (top layer): coarser dye clouds (scale * 1.25)
        float rNoise = organicGrainNoise(pixelCoord + float2(131.7, 71.3), grainScale * 0.9, roughness);
        float gNoise = organicGrainNoise(pixelCoord + float2(291.7, 431.1), grainScale * 1.0, roughness);
        float bNoise = organicGrainNoise(pixelCoord + float2(531.3, 191.9), grainScale * 1.25, roughness);
        
        grainOffset = float3(
            monoNoise * wMono + rNoise * wColor,
            monoNoise * wMono + gNoise * wColor,
            monoNoise * wMono + bNoise * wColor
        );
    } else {
        grainOffset = float3(monoNoise);
    }
    
    float strength = (amount / 100.0) * 0.18 * weight;
    return clamp(color + grainOffset * strength, 0.0, 1.0);
}

#endif /* Detail_metal */
