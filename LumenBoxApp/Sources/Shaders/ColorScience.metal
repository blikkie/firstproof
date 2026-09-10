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

// Camera manufacturer S-curve:
inline float applyCameraToneCurve(float x, int mode) {
    x = clamp(x, 0.0, 1.0);
    float mid = (mode == 2) ? 0.185 : ((mode == 3) ? 0.200 : ((mode == 4) ? 0.210 : 0.190));
    if (x <= 0.18) {
        float t = x / 0.18;
        return mix(0.0, mid, t * t * (3.0 - 2.0 * t));
    } else {
        float t = (x - 0.18) / 0.82;
        return mix(mid, 1.0, t * (2.0 - t));
    }
}

// Camera HueSatMap Memory Color Adjustments:
inline float3 applyCameraHueSat(float3 hsv, int mode) {
    float h = hsv.x;
    float s = hsv.y;
    float v = hsv.z;
    
    float skyWeight = clamp(1.0 - abs(h - 235.0) / 35.0, 0.0, 1.0);
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

constant float AGX_MIN_EV = -15.2;
constant float AGX_MAX_EV = 5.0;
constant float AGX_EV_RANGE = 20.2;
constant float AGX_GAMMA = 2.4;
constant float AGX_MID_GRAY_NORM = 0.75247525;

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
    
    hsv.x -= 18.0 * skyWeight;
    hsv.x -= 3.0 * skinWeight;
    if (hsv.x < 0.0) { hsv.x += 360.0; }
    if (hsv.x >= 360.0) { hsv.x -= 360.0; }
    
    hsv.y = clamp(hsv.y * (1.18 - 0.20 * skinWeight - 0.15 * skyWeight), 0.0, 1.0);
    return clamp(hsv2rgb(hsv), 0.0, 1.0);
}

// MARK: - Selective Tone Recovery (Highlights, Shadows, Whites, Blacks)
inline float3 applyToneRecovery(float3 color, float highlights, float shadows, float whites, float blacks) {
    if (highlights == 0.0 && shadows == 0.0 && whites == 0.0 && blacks == 0.0) {
        return color;
    }
    
    float lum = 0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b;
    if (lum <= 1e-6) {
        return color;
    }
    
    float ev = log2(max(lum, 1e-5) / 0.18);
    float delta_ev = 0.0;
    
    if (blacks != 0.0) {
        float u = clamp((-ev - 1.0) / 3.0, 0.0, 1.0);
        float w_blacks = u * u * (3.0 - 2.0 * u);
        delta_ev += (blacks / 100.0) * 0.90 * w_blacks;
    }
    
    if (shadows != 0.0) {
        float u = clamp((ev + 4.0) / 4.0, 0.0, 1.0);
        float w_shadows = sin(3.141592653589793 * u);
        delta_ev += (shadows / 100.0) * 1.20 * w_shadows;
    }
    
    if (highlights != 0.0) {
        float u = clamp(ev / 4.0, 0.0, 1.0);
        float w_highlights = sin(3.141592653589793 * u);
        delta_ev += (highlights / 100.0) * 1.30 * w_highlights;
    }
    
    if (whites != 0.0) {
        float u = clamp((ev - 1.0) / 3.0, 0.0, 1.0);
        float w_whites = u * u * (3.0 - 2.0 * u);
        delta_ev += (whites / 100.0) * 0.90 * w_whites;
    }
    
    if (delta_ev == 0.0) {
        return color;
    }
    
    float gain = exp2(delta_ev);
    return color * gain;
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
    float lum = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;

    if (saturation != 0.0) {
        float satGain = max(1.0 + saturation / 100.0, 0.0);
        c = lum + (c - lum) * satGain;
    }

    if (vibrance != 0.0) {
        float maxC = max(max(c.r, c.g), c.b);
        float minC = min(min(c.r, c.g), c.b);
        float chroma = maxC - minC;

        float skinProtect = 1.0;
        if (c.r > c.g && c.g > c.b && lum > 0.05) {
            float rg = (c.r - c.g) / max(lum, 0.1);
            float gb = (c.g - c.b) / max(lum, 0.1);
            float dRg = abs(rg - 0.35);
            float dGb = abs(gb - 0.25);
            float skinMetric = clamp(1.0 - (dRg * 3.0 + dGb * 3.0), 0.0, 1.0);
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

#endif /* ColorScience_metal */
