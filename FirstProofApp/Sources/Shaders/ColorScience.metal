#ifndef ColorScience_metal
#define ColorScience_metal

#include <metal_stdlib>
using namespace metal;

// MARK: - Scene-Referred Rec.2020 Color Transfer Functions & Matrices

// Exact sRGB electro-optical transfer functions (IEC 61966-2-1)
inline float srgbGammaToLinear(float c) {
    c = clamp(c, 0.0, 1.0);
    return (c <= 0.04045) ? (c / 12.92) : pow((c + 0.055) / 1.055, 2.4);
}

inline float3 srgbGammaToLinear3(float3 c) {
    return float3(srgbGammaToLinear(c.r), srgbGammaToLinear(c.g), srgbGammaToLinear(c.b));
}

inline float linearToSrgbGamma(float c) {
    c = clamp(c, 0.0, 1.0);
    return (c <= 0.0031308) ? (12.92 * c) : (1.055 * pow(c, 1.0 / 2.4) - 0.055);
}

inline float3 linearToSrgbGamma3(float3 c) {
    return float3(linearToSrgbGamma(c.r), linearToSrgbGamma(c.g), linearToSrgbGamma(c.b));
}

// MARK: - RGB <-> HSV Color Transform Helpers

inline float3 rgb2hsv(float3 c) {
    float4 K = float4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    float4 p = mix(float4(c.bg, K.wz), float4(c.gb, K.xy), step(c.b, c.g));
    float4 q = mix(float4(p.xyw, c.r), float4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return float3(abs(q.z + (q.w - q.y) / (6.0 * d + e)) * 360.0, d / (q.x + e), q.x);
}

inline float3 hsv2rgb(float3 c) {
    float h = c.x / 60.0;
    float s = c.y;
    float v = c.z;
    float C = v * s;
    float X = C * (1.0 - abs(fmod(h, 2.0) - 1.0));
    float m = v - C;
    float3 rgb;
    if (h < 1.0) rgb = float3(C, X, 0.0);
    else if (h < 2.0) rgb = float3(X, C, 0.0);
    else if (h < 3.0) rgb = float3(0.0, C, X);
    else if (h < 4.0) rgb = float3(0.0, X, C);
    else if (h < 5.0) rgb = float3(X, 0.0, C);
    else rgb = float3(C, 0.0, X);
    return rgb + float3(m);
}

// MARK: - Calibrated Camera-Matching Monotonic Cubic Hermite Tone Curves (Fritsch-Carlson C^1 Continuous)

constant float2 DCP_KNOTS_STANDARD[7] = {
    float2(0.00, 0.000), float2(0.02, 0.008), float2(0.05, 0.028),
    float2(0.18, 0.190), float2(0.50, 0.540), float2(0.80, 0.840), float2(1.00, 1.000)
};
constant float DCP_TANGENTS_STANDARD[7] = { 0.4000, 0.5333, 0.9564, 1.1700, 1.0469, 0.9000, 0.8000 };

constant float2 DCP_KNOTS_PORTRAIT[7] = {
    float2(0.00, 0.000), float2(0.02, 0.010), float2(0.05, 0.035),
    float2(0.18, 0.185), float2(0.50, 0.510), float2(0.80, 0.820), float2(1.00, 1.000)
};
constant float DCP_TANGENTS_PORTRAIT[7] = { 0.5000, 0.6667, 0.9936, 1.0847, 1.0245, 0.9667, 0.9000 };

constant float2 DCP_KNOTS_LANDSCAPE[7] = {
    float2(0.00, 0.000), float2(0.02, 0.006), float2(0.05, 0.022),
    float2(0.18, 0.200), float2(0.50, 0.560), float2(0.80, 0.860), float2(1.00, 1.000)
};
constant float DCP_TANGENTS_LANDSCAPE[7] = { 0.3000, 0.4167, 0.9513, 1.2471, 1.0625, 0.8500, 0.7000 };

constant float2 DCP_KNOTS_VIVID[7] = {
    float2(0.00, 0.000), float2(0.02, 0.004), float2(0.05, 0.018),
    float2(0.18, 0.210), float2(0.50, 0.580), float2(0.80, 0.880), float2(1.00, 1.000)
};
constant float DCP_TANGENTS_VIVID[7] = { 0.2000, 0.3333, 0.9718, 1.3166, 1.0781, 0.8000, 0.6000 };

inline float evalCameraHermite7(float x, constant float2* knots, constant float* m) {
    if (x <= knots[0].x) return knots[0].y;
    if (x >= knots[6].x) return knots[6].y;
    for (int k = 0; k < 6; ++k) {
        if (x <= knots[k + 1].x) {
            float dx = knots[k + 1].x - knots[k].x;
            float t = (x - knots[k].x) / dx;
            float t2 = t * t;
            float t3 = t2 * t;
            float h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
            float h10 = t3 - 2.0 * t2 + t;
            float h01 = -2.0 * t3 + 3.0 * t2;
            float h11 = t3 - t2;
            return clamp(h00 * knots[k].y + h10 * m[k] * dx + h01 * knots[k + 1].y + h11 * m[k + 1] * dx, 0.0, 1.0);
        }
    }
    return x;
}

// Camera manufacturer S-curve with verified C^1 derivative continuity:
inline float applyCameraToneCurve(float x, int mode) {
    x = clamp(x, 0.0, 1.0);
    // mode 1: Standard, mode 2: Portrait, mode 3: Landscape, mode 4: Vivid
    if (mode == 2) {
        return evalCameraHermite7(x, DCP_KNOTS_PORTRAIT, DCP_TANGENTS_PORTRAIT);
    } else if (mode == 3) {
        return evalCameraHermite7(x, DCP_KNOTS_LANDSCAPE, DCP_TANGENTS_LANDSCAPE);
    } else if (mode == 4) {
        return evalCameraHermite7(x, DCP_KNOTS_VIVID, DCP_TANGENTS_VIVID);
    } else {
        return evalCameraHermite7(x, DCP_KNOTS_STANDARD, DCP_TANGENTS_STANDARD);
    }
}

// Camera HueSatMap Memory Color Adjustments:
inline float3 applyCameraHueSat(float3 hsv, int mode) {
    float h = hsv.x;
    float s = hsv.y;
    float v = hsv.z;
    
    float skyWeight = clamp(1.0 - abs(h - 235.0) / 35.0, 0.0, 1.0);
    float vSky = smoothstep(0.20, 0.40, v);
    float sSky = smoothstep(0.08, 0.25, s);
    skyWeight *= (vSky * sSky);

    float greenWeight = clamp(1.0 - abs(h - 120.0) / 35.0, 0.0, 1.0);
    float skinWeight = clamp(1.0 - abs(h - 30.0) / 20.0, 0.0, 1.0);
    
    // Shift raw sensor blue towards natural sky azure
    h -= 18.0 * skyWeight;
    if (h < 0.0) { h += 360.0; }
    
    if (mode == 1) {
        // Camera Standard: natural azure sky, vivid foliage, healthy skin
        s *= (1.0 - 0.20 * skyWeight + 0.10 * greenWeight + 0.05 * skinWeight);
        v *= (1.0 - 0.03 * skyWeight + 0.03 * skinWeight);
    } else if (mode == 2) {
        // Camera Portrait: gentle skin smoothing, calm skies
        s *= (1.0 - 0.05 * skinWeight - 0.22 * skyWeight);
        v *= (1.0 + 0.04 * skinWeight);
    } else if (mode == 3) {
        // Camera Landscape: punchy azure sky & greens
        s *= (1.0 - 0.12 * skyWeight + 0.20 * greenWeight);
        v *= (1.0 - 0.05 * skyWeight - 0.04 * greenWeight);
    } else if (mode == 4) {
        // Camera Vivid
        s *= (1.20 - 0.15 * skyWeight);
    }
    // Optical Scotopic / Mesopic desaturation: As luminance approaches sensor read noise floor,
    // rod-dominant vision has zero chromaticity. Suppresses thermal noise spikes without edge clipping.
    float scotopicWeight = smoothstep(0.01, 0.08, v);
    s *= scotopicWeight;
    return float3(h, clamp(s, 0.0, 1.0), clamp(v, 0.0, 1.0));
}

// Linear sRGB -> Linear Rec.2020 Matrix (columns in Metal MSL)
constant float3x3 LINEAR_SRGB_TO_REC2020 = float3x3(
    float3(0.6274041, 0.0690972, 0.0163914),
    float3(0.3292821, 0.9195411, 0.0880133),
    float3(0.0433138, 0.0113617, 0.8955953)
);

// Linear Rec.2020 -> Linear sRGB Matrix (columns in Metal MSL)
constant float3x3 REC2020_TO_LINEAR_SRGB = float3x3(
    float3( 1.6604910, -0.1245505, -0.0181508),
    float3(-0.5876411,  1.1328999, -0.1005789),
    float3(-0.0728499, -0.0083494,  1.1187297)
);

// Linear sRGB -> Linear Display P3 Matrix (columns in Metal MSL)
constant float3x3 LINEAR_SRGB_TO_DISPLAY_P3 = float3x3(
    float3(0.8224621, 0.0331941, 0.0170827),
    float3(0.1775380, 0.9668058, 0.0723974),
    float3(0.0000000, 0.0000000, 0.9105199)
);

// MARK: - AgX Filmic Tone Mapping Engine Constants & Operators
constant float3x3 AGX_INSET_MATRIX = float3x3(
    float3( 1.4096986,   0.13136263,  0.16151477),
    float3(-0.4004796,   0.77151406, -0.06034984),
    float3(-0.009218898, 0.09712328,  0.89883506)
);

constant float3x3 AGX_OUTSET_MATRIX = float3x3(
    float3( 1.1271005, -0.1413298, -0.1413298),
    float3(-0.1106066,  1.1578237, -0.1106066),
    float3(-0.0164939, -0.0164939,  1.2519364)
);

constant float AGX_MIN_EV = -10.0;
constant float AGX_MAX_EV = 6.5;
constant float AGX_EV_RANGE = 16.5;
constant float AGX_GAMMA = 2.2;
constant float AGX_MID_GRAY_NORM = 0.6060606;

inline float agxSigmoid(float x, float power) {
    return x / pow(1.0 + pow(x, power), 1.0 / power);
}

inline float agxScaledSigmoid(float x, float scale, float slope, float power, float tx, float ty) {
    return scale * agxSigmoid(slope * (x - tx) / scale, power) + ty;
}

inline float agxApplyCurveChannel(float x) {
    if (x < 0.6060606) {
        return clamp(agxScaledSigmoid(x, -1.0359, 2.3843, 1.5, 0.6060606, 0.43446), 0.0, 1.0);
    } else {
        return clamp(agxScaledSigmoid(x, 1.3475, 2.3843, 1.5, 0.6060606, 0.43446), 0.0, 1.0);
    }
}

inline float3 agxToneMap(float3 rec2020Color, float contrastGain) {
    float minC = min(rec2020Color.r, min(rec2020Color.g, rec2020Color.b));
    float3 cComp = (minC < 0.0) ? (rec2020Color - minC) : rec2020Color;

    float3 inset = AGX_INSET_MATRIX * cComp;
    
    float3 xRel = max(inset / 0.18, float3(1e-6));
    float3 logEncoded = (log2(xRel) - float3(AGX_MIN_EV)) / AGX_EV_RANGE;
    float3 norm = clamp(logEncoded, 0.0, 1.0);
    
    if (abs(contrastGain - 1.0) > 0.001) {
        norm = clamp(float3(AGX_MID_GRAY_NORM) + (norm - float3(AGX_MID_GRAY_NORM)) * contrastGain, 0.0, 1.0);
    }
    
    float3 curved;
    curved.r = agxApplyCurveChannel(norm.r);
    curved.g = agxApplyCurveChannel(norm.g);
    curved.b = agxApplyCurveChannel(norm.b);
    
    float3 linearized = pow(max(curved, float3(0.0)), float3(AGX_GAMMA));
    return max(AGX_OUTSET_MATRIX * linearized, float3(0.0));
}

inline float3 agxToneMapVivid(float3 rec2020Color, float contrastGain) {
    float3 base = agxToneMap(rec2020Color, contrastGain * 1.15);
    float3 hsv = rgb2hsv(base);
    
    float skinWeight = clamp(1.0 - abs(hsv.x - 25.0) / 20.0, 0.0, 1.0);
    float skyWeight = clamp(1.0 - abs(hsv.x - 235.0) / 35.0, 0.0, 1.0);
    float vSky = smoothstep(0.20, 0.40, hsv.z);
    float sSky = smoothstep(0.08, 0.25, hsv.y);
    skyWeight *= (vSky * sSky);
    
    hsv.x -= 18.0 * skyWeight;
    hsv.x -= 3.0 * skinWeight;
    if (hsv.x < 0.0) { hsv.x += 360.0; }
    if (hsv.x >= 360.0) { hsv.x -= 360.0; }
    
    // Vibrant midtone chroma with sky saturation normalized and delicate skin tones protected.
    // Shadow noise protection: scale chroma boost with luminance floor so read noise in deep shadows (V < 0.05) is not amplified.
    float vShadow = clamp((hsv.z - 0.03) / 0.12, 0.0, 1.0);
    float shadowWeight = vShadow * vShadow * (3.0 - 2.0 * vShadow);
    float satBoost = (0.18 - 0.20 * skinWeight - 0.15 * skyWeight) * shadowWeight;
    hsv.y = clamp(hsv.y * (1.0 + satBoost), 0.0, 1.0);

    // Optical Scotopic / Mesopic desaturation: As luminance approaches sensor read noise floor,
    // rod-dominant vision has zero chromaticity. Suppresses thermal noise spikes without edge clipping.
    float scotopicWeight = smoothstep(0.01, 0.08, hsv.z);
    hsv.y *= scotopicWeight;
    return clamp(hsv2rgb(hsv), 0.0, 1.0);
}

// MARK: - Advanced Bilateral Base/Detail Tone Recovery (Highlights, Shadows, Whites, Blacks)
inline float3 applyBilateralToneRecovery(
    float3 color,
    float y_c,
    float y_neighbors[8],
    float highlights,
    float shadows,
    float whites,
    float blacks
) {
    if (highlights == 0.0 && shadows == 0.0 && whites == 0.0 && blacks == 0.0) {
        return color;
    }
    
    if (y_c <= 1e-6) {
        return color;
    }
    
    // Log-luminance EV relative to photographic middle gray (0.18)
    float ev_c = log2(max(y_c, 1e-5) / 0.18);
    
    // Bilateral filter base illumination estimation:
    // Spatial weights: Center = 1.0, Orthogonal (dist 1) = exp(-0.5) ~ 0.60653066, Diagonal (dist sqrt(2)) = exp(-1.0) ~ 0.36787944
    // Range weights: Gaussian in EV stops with sigma_r = 0.85 EV (2 * sigma_r^2 = 1.445)
    constexpr float w_s_ortho = 0.60653066;
    constexpr float w_s_diag  = 0.36787944;
    constexpr float inv_two_sigma_r_sq = 1.0 / 1.445;
    
    float total_weight = 1.0;
    float weighted_ev_sum = ev_c;
    
    // 0..3: Orthogonal neighbors (left, right, top, bottom)
    for (int i = 0; i < 4; ++i) {
        float ev_i = log2(max(y_neighbors[i], 1e-5) / 0.18);
        float d_ev = ev_i - ev_c;
        float w_r = exp(-(d_ev * d_ev) * inv_two_sigma_r_sq);
        float w = w_s_ortho * w_r;
        total_weight += w;
        weighted_ev_sum += w * ev_i;
    }
    
    // 4..7: Diagonal neighbors (top-left, top-right, bottom-left, bottom-right)
    for (int i = 4; i < 8; ++i) {
        float ev_i = log2(max(y_neighbors[i], 1e-5) / 0.18);
        float d_ev = ev_i - ev_c;
        float w_r = exp(-(d_ev * d_ev) * inv_two_sigma_r_sq);
        float w = w_s_diag * w_r;
        total_weight += w;
        weighted_ev_sum += w * ev_i;
    }
    
    float ev_base = weighted_ev_sum / total_weight;
    float delta_ev = 0.0;
    
    if (blacks != 0.0) {
        float u = clamp((-ev_base - 1.0) / 3.0, 0.0, 1.0);
        float w_blacks = u * u * (3.0 - 2.0 * u);
        delta_ev += (blacks / 100.0) * 0.90 * w_blacks;
    }
    
    if (shadows != 0.0) {
        float u = clamp((ev_base + 4.0) / 4.0, 0.0, 1.0);
        float w_shadows = sin(3.141592653589793 * u);
        delta_ev += (shadows / 100.0) * 1.20 * w_shadows;
    }
    
    if (highlights != 0.0) {
        float u = clamp(ev_base / 4.0, 0.0, 1.0);
        float w_highlights = sin(3.141592653589793 * u);
        delta_ev += (highlights / 100.0) * 1.30 * w_highlights;
    }
    
    if (whites != 0.0) {
        float u = clamp((ev_base - 1.0) / 3.0, 0.0, 1.0);
        float w_whites = u * u * (3.0 - 2.0 * u);
        delta_ev += (whites / 100.0) * 0.90 * w_whites;
    }
    
    if (delta_ev == 0.0) {
        return color;
    }
    
    float gain = exp2(delta_ev);
    return color * gain;
}

// Pointwise fallback for backward compatibility
inline float3 applyToneRecovery(float3 color, float highlights, float shadows, float whites, float blacks) {
    if (highlights == 0.0 && shadows == 0.0 && whites == 0.0 && blacks == 0.0) {
        return color;
    }
    float lum = 0.2627 * color.r + 0.6780 * color.g + 0.0593 * color.b;
    float neighbors[8] = { lum, lum, lum, lum, lum, lum, lum, lum };
    return applyBilateralToneRecovery(color, lum, neighbors, highlights, shadows, whites, blacks);
}

// MARK: - Color & Vibrance Adjustment Function
inline float3 applyColorAdjustments(
    float3 rgb,
    float vibrance,
    float saturation,
    float deepBlue,
    float foliage
) {
    if (vibrance == 0.0 && saturation == 0.0 && deepBlue == 0.0 && foliage == 0.0) {
        return rgb;
    }
    
    float3 c = clamp(rgb, 0.0, 1.0);
    float lum = 0.2627 * c.r + 0.6780 * c.g + 0.0593 * c.b;

    if (saturation != 0.0) {
        float satGain = max(1.0 + saturation / 100.0, 0.0);
        c = lum + (c - lum) * satGain;
    }

    if (vibrance != 0.0) {
        float maxC = max(max(c.r, c.g), c.b);
        float minC = min(min(c.r, c.g), c.b);
        float chroma = maxC - minC;

        float skinProtect = 1.0;
        float sum = c.r + c.g + c.b;
        if (c.r > c.g && c.g > c.b && sum > 0.015) {
            float rN = c.r / sum;
            float gN = c.g / sum;
            float dR = abs(rN - 0.44);
            float dG = abs(gN - 0.32);
            float skinMetric = clamp(1.0 - (dR * 4.5 + dG * 9.0), 0.0, 1.0);
            skinProtect = 1.0 - 0.6 * skinMetric;
        }

        float vibGain = (vibrance > 0.0)
            ? (vibrance / 100.0) * (1.0 - chroma) * 0.8 * skinProtect
            : (vibrance / 100.0) * 0.6;

        c += (c - lum) * vibGain;
    }

    if (deepBlue != 0.0) {
        float maxWarm = max(c.r, c.g);
        float skySignal = max(c.b - maxWarm, 0.0);
        if (skySignal > 0.0) {
            float x = deepBlue / 100.0;
            float factor = sign(x) * pow(abs(x), 1.6);
            if (factor > 0.0) {
                c.r -= factor * skySignal * 0.30;
                c.g -= factor * skySignal * 0.15;
                c.b -= factor * skySignal * 0.20;
            } else {
                c.r -= factor * skySignal * 0.25;
                c.g -= factor * skySignal * 0.20;
                c.b -= factor * skySignal * 0.10;
            }
        }
    }

    if (foliage != 0.0) {
        float maxRb = max(c.r, c.b);
        float greenSignal = max(c.g - maxRb, 0.0);
        if (greenSignal > 0.0) {
            float x = foliage / 100.0;
            float factor = sign(x) * pow(abs(x), 1.6);
            c.r -= factor * greenSignal * 0.30;
            c.b -= factor * greenSignal * 0.15;
            c.g += factor * greenSignal * 0.10;
        }
    }

    return clamp(c, 0.0, 1.0);
}

// MARK: - Split Toning & Color Grading with Optical Balance Anchoring
inline float3 applySplitToning(
    float3 rgb,
    float shadowHue,
    float shadowSat,
    float highlightHue,
    float highlightSat,
    float balance
) {
    if (shadowSat <= 0.0 && highlightSat <= 0.0) {
        return rgb;
    }

    float lum = 0.2627 * rgb.r + 0.6780 * rgb.g + 0.0593 * rgb.b;
    float yClamp = clamp(lum, 0.0, 1.0);

    // Midpoint threshold shifted by balance (-100..+100)
    float t = clamp(0.5 - (balance / 100.0) * 0.25, 0.1, 0.9);

    float3 delta = float3(0.0);

    // Shadow Toning
    if (shadowSat > 0.0 && yClamp < t) {
        float u_s = (t - yClamp) / t;
        float w_s = u_s * u_s * (3.0 - 2.0 * u_s) * (shadowSat / 100.0);
        float3 tintRgb = hsv2rgb(float3(shadowHue, 1.0, 1.0));
        float tLum = 0.2627 * tintRgb.r + 0.6780 * tintRgb.g + 0.0593 * tintRgb.b;
        delta += w_s * (tintRgb - float3(tLum));
    }

    // Highlight Toning
    if (highlightSat > 0.0 && yClamp > t) {
        float u_h = (yClamp - t) / (1.0 - t);
        float w_h = u_h * u_h * (3.0 - 2.0 * u_h) * (highlightSat / 100.0);
        float3 tintRgb = hsv2rgb(float3(highlightHue, 1.0, 1.0));
        float tLum = 0.2627 * tintRgb.r + 0.6780 * tintRgb.g + 0.0593 * tintRgb.b;
        delta += w_h * (tintRgb - float3(tLum));
    }

    return clamp(rgb + delta, 0.0, 1.0);
}

// MARK: - Curated Fine-Art Print Media Archetypes & Optical Enlarger Filtration
inline float applyPaperGrade(float val, float grade) {
    if (abs(grade) < 0.001) {
        return val;
    }
    float alpha = 0.40 * (grade / 2.0);
    return clamp(val + alpha * val * (1.0 - val) * (2.0 * val - 1.0), 0.0, 1.0);
}

inline float3 applyBaseWarmth(float3 rgb, float warmth) {
    if (abs(warmth) < 0.001) {
        return rgb;
    }
    float lum = 0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b;
    float highlightWeight = clamp(pow(lum, 0.6), 0.0, 1.0);
    float factor = (warmth / 100.0) * highlightWeight;
    
    float3 mult;
    if (factor > 0.0) {
        mult = float3(
            1.0 + factor * 0.060,
            1.0 + factor * 0.025,
            1.0 - factor * 0.045
        );
    } else {
        float absF = -factor;
        mult = float3(
            1.0 - absF * 0.025,
            1.0 - absF * 0.008,
            1.0 + absF * 0.045
        );
    }
    return clamp(rgb * mult, 0.0, 1.0);
}

inline float3 getEnlargerFilterWeights(int filterMode) {
    switch (filterMode) {
        case 1: return float3(0.35, 0.60, 0.05); // YellowK2
        case 2: return float3(0.55, 0.42, 0.03); // Orange16
        case 3: return float3(0.85, 0.15, 0.00); // Red25A
        case 4: return float3(0.15, 0.80, 0.05); // GreenX1
        case 5: return float3(0.00, 0.60, 0.40); // Orthochromatic
        default: return float3(0.2627, 0.6780, 0.0593); // NeutralPanchromatic
    }
}

inline float3 applyPrintMedia(
    float3 rgb,
    int mediaMode,
    int filterMode,
    float paperGrade,
    float baseWarmth,
    float processDepth
) {
    if (mediaMode == 0) {
        return rgb;
    }
    
    float depth = clamp(processDepth / 100.0, 0.0, 1.0);
    
    switch (mediaMode) {
        case 1: { // ChromogenicTypeC (RA-4)
            float rG = applyPaperGrade(rgb.r, paperGrade);
            float gG = applyPaperGrade(rgb.g, paperGrade);
            float bG = applyPaperGrade(rgb.b, paperGrade);
            
            // Sensitometric highlight compression: smooth cubic Hermite soft shoulder
            auto compressShoulder = [depth](float c) -> float {
                if (c > 0.70) {
                    float u = clamp((c - 0.70) / 0.30, 0.0, 1.0);
                    return c - 0.045 * depth * u * u * (3.0 - 2.0 * u);
                }
                return c;
            };
            float rP = compressShoulder(rG);
            float gP = compressShoulder(gG);
            float bP = compressShoulder(bG);
            
            float lum = 0.2126 * rP + 0.7152 * gP + 0.0722 * bP;
            float satScale = 0.70 + 0.35 * depth;
            float rOut = clamp(lum + (rP - lum) * satScale, 0.0, 1.0);
            float gOut = clamp(lum + (gP - lum) * satScale, 0.0, 1.0);
            float bOut = clamp(lum + (bP - lum) * satScale, 0.0, 1.0);
            
            float dyeDensity = 0.04 * max(1.0 - lum, 0.0) * depth;
            rOut = clamp(rOut * (1.0 - dyeDensity * 0.4), 0.0, 1.0);
            gOut = clamp(gOut * (1.0 - dyeDensity * 0.6), 0.0, 1.0);
            bOut = clamp(bOut * (1.0 - dyeDensity * 0.9), 0.0, 1.0);
            
            return applyBaseWarmth(float3(rOut, gOut, bOut), baseWarmth);
        }
        case 2: { // SilverDyeBleach (Azo Reversal)
            float rG = applyPaperGrade(rgb.r, paperGrade);
            float gG = applyPaperGrade(rgb.g, paperGrade);
            float bG = applyPaperGrade(rgb.b, paperGrade);
            
            float shadowPow = 1.0 + 0.12 * depth;
            float rP = pow(rG, shadowPow);
            float gP = pow(gG, shadowPow);
            float bP = pow(bG, shadowPow);
            
            float lum = 0.2126 * rP + 0.7152 * gP + 0.0722 * bP;
            float azoSat = 0.80 + 0.30 * depth;
            float rOut = clamp(lum + (rP - lum) * azoSat, 0.0, 1.0);
            float gOut = clamp(lum + (gP - lum) * azoSat, 0.0, 1.0);
            float bOut = clamp(lum + (bP - lum) * azoSat, 0.0, 1.0);
            
            return applyBaseWarmth(float3(rOut, gOut, bOut), baseWarmth);
        }
        case 3: { // FineArtCottonRag (Mineral Pigment)
            float rG = applyPaperGrade(rgb.r, paperGrade);
            float gG = applyPaperGrade(rgb.g, paperGrade);
            float bG = applyPaperGrade(rgb.b, paperGrade);
            
            // Fiber surface scatter: lifts shadows without compressing specular highlights
            float matteFloor = 0.055 - 0.015 * depth;
            float rP = rG + matteFloor * (1.0 - rG) * (1.0 - rG);
            float gP = gG + matteFloor * (1.0 - gG) * (1.0 - gG);
            float bP = bG + matteFloor * (1.0 - bG) * (1.0 - bG);
            
            float lum = 0.2126 * rP + 0.7152 * gP + 0.0722 * bP;
            float pigmentSat = 0.75 + 0.25 * depth;
            float rOut = clamp(lum + (rP - lum) * pigmentSat, 0.0, 1.0);
            float gOut = clamp(lum + (gP - lum) * pigmentSat, 0.0, 1.0);
            float bOut = clamp(lum + (bP - lum) * pigmentSat, 0.0, 1.0);
            
            return applyBaseWarmth(float3(rOut, gOut, bOut), baseWarmth + 15.0);
        }
        case 4: { // BarytaSilverGelatin
            float3 w = getEnlargerFilterWeights(filterMode);
            float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
            float silver = applyPaperGrade(rawLum, paperGrade);
            
            float dmaxGamma = 1.0 + 0.08 * depth;
            float silverToned = pow(silver, dmaxGamma);
            
            return applyBaseWarmth(float3(silverToned), baseWarmth);
        }
        case 5: { // WarmtoneChlorobromide (Selenium Toned)
            float3 w = getEnlargerFilterWeights(filterMode);
            float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
            float silver = applyPaperGrade(rawLum, paperGrade);
            
            float shadowPow = 1.0 + 0.10 * depth;
            float silverDeep = pow(silver, shadowPow);
            
            // Smooth parabolic harmonic bell curve for midtone dye deposition
            float midMask = 4.0 * silver * (1.0 - silver);
            float shadowDensity = max(1.0 - silver, 0.0);
            
            float rTone = silverDeep + depth * (midMask * 0.030 + shadowDensity * 0.020);
            float gTone = silverDeep + depth * (midMask * 0.012 - shadowDensity * 0.015);
            float bTone = silverDeep + depth * (-midMask * 0.018 + shadowDensity * 0.030);
            
            float3 toned = clamp(float3(rTone, gTone, bTone), 0.0, 1.0);
            return applyBaseWarmth(toned, baseWarmth);
        }
        case 6: { // PlatinumPalladium
            float3 w = getEnlargerFilterWeights(filterMode);
            float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
            float silver = applyPaperGrade(rawLum, paperGrade);
            
            float matteFloor = 0.035 * depth;
            float silverBase = matteFloor + (1.0 - matteFloor) * silver;
            
            float warmLift = max(1.0 - pow(silverBase, 0.6), 0.0) * depth;
            float rTone = silverBase + warmLift * 0.045;
            float gTone = silverBase + warmLift * 0.028;
            float bTone = silverBase - warmLift * 0.020;
            
            float3 toned = clamp(float3(rTone, gTone, bTone), 0.0, 1.0);
            return applyBaseWarmth(toned, baseWarmth + 10.0 * depth);
        }
        case 7: { // Cyanotype
            float3 w = getEnlargerFilterWeights(filterMode);
            float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
            float silver = applyPaperGrade(rawLum, paperGrade);
            
            float density = 1.0 - silver;
            float rP = silver * (1.0 - 0.75 * depth * density);
            float gP = silver * (1.0 - 0.35 * depth * density) + 0.02 * depth * density;
            float bP = silver * (1.0 - 0.05 * depth * density) + 0.07 * depth * density;
            
            float3 toned = clamp(float3(rP, gP, bP), 0.0, 1.0);
            return applyBaseWarmth(toned, baseWarmth + 8.0 * depth);
        }
        default:
            return rgb;
    }
}

// MARK: - Upstream Recording Emulsions & Photochemical Grain Mechanics

inline float applyFilmCharacteristicCurve(float val, float gamma, float shoulder, float toe) {
    float v = clamp(val, 0.0, 2.0);
    float base = v;
    if (abs(gamma - 1.0) > 0.001) {
        float u = clamp(v / (v + 0.35), 0.0, 1.0);
        float gMod = (gamma - 1.0) * 0.40;
        base = max(v + gMod * u * (1.0 - u) * (2.0 * u - 1.0), 0.0);
    }
    float afterShoulder = base;
    if (shoulder > 0.0 && base > 0.65) {
        float t = clamp((base - 0.65) / 0.35, 0.0, 1.0);
        float comp = shoulder * 0.12 * t * t * (3.0 - 2.0 * t);
        afterShoulder = base - comp;
    }
    if (toe > 0.0 && afterShoulder < 0.35) {
        float t = clamp(afterShoulder / 0.35, 0.0, 1.0);
        float comp = toe * 0.06 * (1.0 - t) * (1.0 - t);
        return max(afterShoulder - comp, 0.0);
    }
    return afterShoulder;
}

inline float3 applyDirChemicalCoupling(
    float3 rgb,
    float c_rg, float c_rb,
    float c_gr, float c_gb,
    float c_br, float c_bg
) {
    float eps = 1e-4;
    float d_r = -log10(max(rgb.r, eps));
    float d_g = -log10(max(rgb.g, eps));
    float d_b = -log10(max(rgb.b, eps));

    float c_rr = 1.0 - c_rg - c_rb;
    float c_gg = 1.0 - c_gr - c_gb;
    float c_bb = 1.0 - c_br - c_bg;

    float d_r_out = c_rr * d_r + c_rg * d_g + c_rb * d_b;
    float d_g_out = c_gr * d_r + c_gg * d_g + c_gb * d_b;
    float d_b_out = c_br * d_r + c_bg * d_g + c_bb * d_b;

    return clamp(float3(pow(10.0, -d_r_out), pow(10.0, -d_g_out), pow(10.0, -d_b_out)), 0.0, 2.0);
}

inline float3 applyRecordingEmulsion(
    float3 rgb,
    int archetype,
    float pushPullEv
) {
    if (archetype == 0) {
        return rgb;
    }

    float expGain = pow(2.0, clamp(pushPullEv, -2.0, 2.0));
    float3 curRgb = max(rgb * expGain, 0.0);

    switch (archetype) {
        case 1: { // RochesterPortrait400 - Ref: Kodak Pub E-4050 (Portra 400), Status M Densitometry
            float3 dir = applyDirChemicalCoupling(curRgb, -0.04, 0.02, -0.08, 0.02, 0.02, -0.05);
            return float3(
                applyFilmCharacteristicCurve(dir.r, 0.90, 0.38, 0.08),
                applyFilmCharacteristicCurve(dir.g, 0.90, 0.38, 0.08),
                applyFilmCharacteristicCurve(dir.b, 0.90, 0.38, 0.08)
            );
        }
        case 2: { // AshigaraChrome50 - Ref: Fujifilm Data Sheet AF3-960E (Velvia 50), Status A Densitometry
            float3 dir = applyDirChemicalCoupling(curRgb, -0.15, -0.08, -0.18, -0.08, -0.06, -0.12);
            return float3(
                applyFilmCharacteristicCurve(dir.r, 1.10, 0.12, 0.14),
                applyFilmCharacteristicCurve(dir.g, 1.12, 0.10, 0.14),
                applyFilmCharacteristicCurve(dir.b, 1.11, 0.12, 0.16)
            );
        }
        case 3: { // RochesterK14Chrome64 - Ref: Kodak Pub E-88 (Kodachrome 64) & ResearchGate Layer Analysis
            float3 dir = applyDirChemicalCoupling(curRgb, -0.08, 0.06, -0.10, 0.04, 0.08, -0.04);
            return float3(
                applyFilmCharacteristicCurve(dir.r, 1.05, 0.18, 0.08),
                applyFilmCharacteristicCurve(dir.g, 1.06, 0.16, 0.10),
                applyFilmCharacteristicCurve(dir.b, 1.08, 0.14, 0.14)
            );
        }
        case 4: { // RochesterCinema500 - Ref: Kodak Data Sheet H-1-5219 (Vision3 500T 5219), Status M Densitometry
            float3 dir = applyDirChemicalCoupling(curRgb, -0.02, 0.01, -0.03, 0.01, 0.01, -0.02);
            return float3(
                applyFilmCharacteristicCurve(dir.r, 0.82, 0.45, 0.08),
                applyFilmCharacteristicCurve(dir.g, 0.82, 0.45, 0.08),
                applyFilmCharacteristicCurve(dir.b, 0.82, 0.45, 0.08)
            );
        }
        case 5: { // RochesterDaylight200 - Ref: Kodak Pub E-7022 (Gold 200), Status M Densitometry
            float3 dir = applyDirChemicalCoupling(curRgb, -0.06, 0.03, -0.07, 0.02, 0.08, -0.10);
            return float3(
                applyFilmCharacteristicCurve(dir.r, 1.12, 0.20, 0.12),
                applyFilmCharacteristicCurve(dir.g, 1.08, 0.22, 0.12),
                applyFilmCharacteristicCurve(dir.b, 0.98, 0.26, 0.16)
            );
        }
        case 6: { // MobberleySilver400 - Ref: Ilford Technical Info Sheet (HP5 Plus), ID-11 Developer (1+1)
            float panLum = 0.28 * curRgb.r + 0.62 * curRgb.g + 0.10 * curRgb.b;
            float mono = applyFilmCharacteristicCurve(panLum, 1.08, 0.30, 0.25);
            return float3(mono);
        }
        case 7: { // CambridgeDiffusion100 - Ref: Polaroid Corp Polacolor ER / Type 669 & Land PNAS
            float3 dir = applyDirChemicalCoupling(curRgb, -0.03, 0.02, -0.04, 0.03, 0.01, -0.03);
            float3 curv = float3(
                applyFilmCharacteristicCurve(dir.r, 0.98, 0.35, 0.12),
                applyFilmCharacteristicCurve(dir.g, 0.98, 0.35, 0.12),
                applyFilmCharacteristicCurve(dir.b, 0.98, 0.35, 0.12)
            );
            float lum = 0.2126 * curv.r + 0.7152 * curv.g + 0.0722 * curv.b;
            float baseRefl = 0.025 * max(1.0 - lum, 0.0);
            return min(curv + baseRefl * float3(1.02, 1.00, 0.94), 2.0);
        }
        default:
            return rgb;
    }
}

#endif /* ColorScience_metal */
