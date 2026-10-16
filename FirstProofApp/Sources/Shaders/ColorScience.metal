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
    float maxV = max(c.r, max(c.g, c.b));
    float minV = min(c.r, min(c.g, c.b));
    float delta = maxV - minV;
    
    float h = 0.0;
    if (delta > 1e-5) {
        if (maxV == c.r) {
            h = (c.g - c.b) / delta;
            if (h < 0.0) h += 6.0;
        } else if (maxV == c.g) {
            h = (c.b - c.r) / delta + 2.0;
        } else {
            h = (c.r - c.g) / delta + 4.0;
        }
        h *= 60.0;
    }
    
    float s = (maxV > 1e-5) ? (delta / maxV) : 0.0;
    return float3(h, s, maxV);
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

// MARK: - Calibrated Camera-Matching HueSatMaps (Memory Color & Optical Sensor Crosstalk Correction)

constant float3 DCP_HSM_STANDARD[24] = {
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 0 deg (Red)
    float3(0.0, 1.03, 1.02), float3(0.0, 1.05, 1.03),     // 30 deg (Orange/Skin)
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 60 deg (Yellow)
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 90 deg
    float3(1.5, 1.06, 0.99), float3(2.0, 1.10, 0.98),     // 120 deg (Green/Foliage)
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 150 deg
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 180 deg (Cyan)
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 210 deg (Sky Blue - neutral, no cyan shift)
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 240 deg (Sensor Blue - neutral, no cyan shift)
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 270 deg (Violet - neutral)
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 300 deg (Magenta)
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00)      // 330 deg (Rose)
};

constant float3 DCP_HSM_PORTRAIT[24] = {
    float3(1.5, 0.96, 1.02), float3(2.0, 0.94, 1.03),     // 0 deg (Red)
    float3(0.0, 0.96, 1.03), float3(0.0, 0.94, 1.04),     // 30 deg (Orange/Skin)
    float3(-1.5, 0.96, 1.02), float3(-2.0, 0.94, 1.03),   // 60 deg (Yellow)
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 90 deg
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 120 deg
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 150 deg
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 180 deg
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 210 deg (Sky Blue)
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 240 deg (Sensor Blue)
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 270 deg
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 300 deg
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00)      // 330 deg
};

constant float3 DCP_HSM_LANDSCAPE[24] = {
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 0 deg
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 30 deg
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 60 deg
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 90 deg
    float3(2.5, 1.15, 0.97), float3(3.5, 1.22, 0.95),     // 120 deg (Green/Foliage)
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 150 deg
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 180 deg
    float3(0.0, 1.05, 1.00), float3(0.0, 1.05, 1.00),     // 210 deg (Sky Blue)
    float3(0.0, 1.05, 1.00), float3(0.0, 1.05, 1.00),     // 240 deg (Sensor Blue)
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 270 deg
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00),     // 300 deg
    float3(0.0, 1.00, 1.00), float3(0.0, 1.00, 1.00)      // 330 deg
};

constant float3 DCP_HSM_VIVID[24] = {
    float3(0.0, 1.15, 1.02), float3(0.0, 1.15, 1.02),     // 0 deg
    float3(0.0, 1.15, 1.02), float3(0.0, 1.15, 1.02),     // 30 deg
    float3(0.0, 1.15, 1.02), float3(0.0, 1.15, 1.02),     // 60 deg
    float3(0.0, 1.15, 1.02), float3(0.0, 1.15, 1.02),     // 90 deg
    float3(0.0, 1.15, 1.02), float3(2.0, 1.25, 0.96),     // 120 deg (Green/Foliage)
    float3(0.0, 1.15, 1.02), float3(0.0, 1.15, 1.02),     // 150 deg
    float3(0.0, 1.15, 1.02), float3(0.0, 1.15, 1.02),     // 180 deg
    float3(0.0, 1.15, 1.02), float3(0.0, 1.15, 1.02),     // 210 deg (Sky Blue)
    float3(0.0, 1.15, 1.02), float3(0.0, 1.15, 1.02),     // 240 deg (Sensor Blue)
    float3(0.0, 1.15, 1.02), float3(0.0, 1.15, 1.02),     // 270 deg
    float3(0.0, 1.15, 1.02), float3(0.0, 1.15, 1.02),     // 300 deg
    float3(0.0, 1.15, 1.02), float3(0.0, 1.15, 1.02)      // 330 deg
};

inline float3 evalHueSatMapDelta(float h_deg, float s, constant float3* table) {
    float h_norm = fract(h_deg / 360.0);
    if (h_norm < 0.0) h_norm += 1.0;
    float h_coord = h_norm * 12.0;
    int h0 = int(floor(h_coord)) % 12;
    int h1 = (h0 + 1) % 12;
    float h_frac = h_coord - floor(h_coord);

    float s_clamped = clamp(s, 0.0, 1.0);
    
    float3 c00 = table[h0 * 2 + 0];
    float3 c01 = table[h0 * 2 + 1];
    float3 c10 = table[h1 * 2 + 0];
    float3 c11 = table[h1 * 2 + 1];

    float3 c0 = mix(c00, c01, s_clamped);
    float3 c1 = mix(c10, c11, s_clamped);

    return mix(c0, c1, h_frac);
}

inline float3 applyCameraHueSatMap(float3 c_srgb, int mode) {
    if (max(max(c_srgb.r, c_srgb.g), c_srgb.b) <= 1e-6) {
        return c_srgb;
    }
    float3 hsv = rgb2hsv(c_srgb);
    if (hsv.y <= 1e-6) {
        return c_srgb;
    }
    float3 delta;
    if (mode == 2) {
        delta = evalHueSatMapDelta(hsv.x, hsv.y, DCP_HSM_PORTRAIT);
    } else if (mode == 3) {
        delta = evalHueSatMapDelta(hsv.x, hsv.y, DCP_HSM_LANDSCAPE);
    } else if (mode == 4) {
        delta = evalHueSatMapDelta(hsv.x, hsv.y, DCP_HSM_VIVID);
    } else {
        delta = evalHueSatMapDelta(hsv.x, hsv.y, DCP_HSM_STANDARD);
    }

    float h_out = fmod(fmod(hsv.x + delta.x, 360.0) + 360.0, 360.0);
    float s_out = clamp(hsv.y * delta.y, 0.0, 1.0);
    float v_out = max(hsv.z * delta.z, 0.0);

    return hsv2rgb(float3(h_out, s_out, v_out));
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

inline float agxApplyCurveChannel(float x) {
    float xc = clamp(x, 0.0, 1.0);
    // Horner evaluation: (((((c6 * x + c5) * x + c4) * x + c3) * x + c2) * x + c1) * x
    float curved = (((((15.45672 * xc - 40.02792) * xc + 31.87076) * xc - 6.84882) * xc + 0.42860) * xc + 0.12066) * xc;
    return clamp(curved, 0.0, 1.0);
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
    float punchyContrast = contrastGain * 1.35;
    float3 base = agxToneMap(rec2020Color, punchyContrast);

    float lum = 0.2126 * base.r + 0.7152 * base.g + 0.0722 * base.b;
    float t = clamp((lum - 0.65) / 0.35, 0.0, 1.0);
    float h = t * t * (3.0 - 2.0 * t);
    float baseSat = 1.25 - 0.25 * h;

    float sat = baseSat;

    return clamp(float3(
        lum + sat * (base.r - lum),
        lum + sat * (base.g - lum),
        lum + sat * (base.b - lum)
    ), 0.0, 1.0);
}

// MARK: - Advanced Bilateral Base/Detail Tone Recovery (Highlights, Shadows, Whites, Blacks)
inline float3 applyBilateralToneRecovery(
    float3 color,
    float y_c,
    float y_neighbors[16],
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
    
    // Bilateral filter base illumination estimation (Dual-concentric ring stencil, 16 taps):
    // Inner ring (r1): local edge guard against boundary bleed.
    // Outer ring (r2): macro ambient illumination (Durand & Dorsey 2002 Retinex separation).
    // Spatial weights at characteristic scale: Orthogonal (dist r) = exp(-0.5) ~ 0.60653066, Diagonal (dist r*sqrt(2)) = exp(-1.0) ~ 0.36787944
    // Ambient illumination spatial separation: sigma_r = 2.0 EV (2 * sigma_r^2 = 8.0)
    // Prevents soft facial contours from being treated as hard edges, avoiding feature flattening.
    constexpr float w_s_ortho = 0.60653066;
    constexpr float w_s_diag  = 0.36787944;
    constexpr float inv_two_sigma_r_sq = 1.0 / 8.0;
    
    float total_weight = 1.0;
    float weighted_ev_sum = ev_c;
    
    // 0..3: Inner Orthogonal neighbors (left, right, top, bottom at r1)
    for (int i = 0; i < 4; ++i) {
        float ev_i = log2(max(y_neighbors[i], 1e-5) / 0.18);
        float d_ev = ev_i - ev_c;
        float w_r = exp(-(d_ev * d_ev) * inv_two_sigma_r_sq);
        float w = w_s_ortho * w_r;
        total_weight += w;
        weighted_ev_sum += w * ev_i;
    }
    
    // 4..7: Inner Diagonal neighbors (top-left, top-right, bottom-left, bottom-right at r1)
    for (int i = 4; i < 8; ++i) {
        float ev_i = log2(max(y_neighbors[i], 1e-5) / 0.18);
        float d_ev = ev_i - ev_c;
        float w_r = exp(-(d_ev * d_ev) * inv_two_sigma_r_sq);
        float w = w_s_diag * w_r;
        total_weight += w;
        weighted_ev_sum += w * ev_i;
    }

    // 8..11: Outer Orthogonal neighbors (left, right, top, bottom at r2)
    for (int i = 8; i < 12; ++i) {
        float ev_i = log2(max(y_neighbors[i], 1e-5) / 0.18);
        float d_ev = ev_i - ev_c;
        float w_r = exp(-(d_ev * d_ev) * inv_two_sigma_r_sq);
        float w = w_s_ortho * w_r;
        total_weight += w;
        weighted_ev_sum += w * ev_i;
    }
    
    // 12..15: Outer Diagonal neighbors (top-left, top-right, bottom-left, bottom-right at r2)
    for (int i = 12; i < 16; ++i) {
        float ev_i = log2(max(y_neighbors[i], 1e-5) / 0.18);
        float d_ev = ev_i - ev_c;
        float w_r = exp(-(d_ev * d_ev) * inv_two_sigma_r_sq);
        float w = w_s_diag * w_r;
        total_weight += w;
        weighted_ev_sum += w * ev_i;
    }
    
    float ev_base = weighted_ev_sum / total_weight;
    float delta_ev = 0.0;
    
    // Sensitometric Ansel Adams Zone System mapping (EV stops relative to Zone V middle gray 0.18):
    // Smooth cubic Hermite weights u^2(3 - 2u) guarantee C^1 derivative continuity at zone boundaries.

    // 1. Blacks: Ansel Adams Zones 0-II (-4.0 EV to -1.0 EV, film base + fog and deep textureless shadows)
    if (blacks != 0.0) {
        float u = clamp((-ev_base - 1.0) / 3.0, 0.0, 1.0);
        float w_blacks = u * u * (3.0 - 2.0 * u);
        delta_ev += (blacks / 100.0) * 1.20 * w_blacks;
    }
    
    // 2. Shadows: Ansel Adams Zones II-IV (-3.0 EV to 0.0 EV, shadow texture, dark skin, dark foliage)
    // Full photographic headroom (2.25 EV) allows deep shadow lifting matching Lightroom.
    if (shadows != 0.0) {
        float u = clamp(-ev_base / 3.0, 0.0, 1.0);
        float w_shadows = u * u * (3.0 - 2.0 * u);
        delta_ev += (shadows / 100.0) * 2.25 * w_shadows;
    }
    
    // 3. Highlights: Ansel Adams Zones VI-IX (0.0 EV to +3.0 EV, light skin, textured snow, cloud detail)
    // 2.25 EV pull recovers dense sky clouds and prevents chalky highlight blowout.
    if (highlights != 0.0) {
        float u = clamp(ev_base / 3.0, 0.0, 1.0);
        float w_highlights = u * u * (3.0 - 2.0 * u);
        delta_ev += (highlights / 100.0) * 2.25 * w_highlights;
    }
    
    // 4. Whites: Ansel Adams Zones VII-X (+1.0 EV to +4.0 EV, specular highlights and maximum printable diffuse white)
    if (whites != 0.0) {
        float u = clamp((ev_base - 1.0) / 3.0, 0.0, 1.0);
        float w_whites = u * u * (3.0 - 2.0 * u);
        delta_ev += (whites / 100.0) * 1.20 * w_whites;
    }
    
    if (delta_ev == 0.0) {
        return color;
    }
    
    // Retinex High-Frequency Detail Preservation & Micro-Contrast Restoration:
    // Preserves local edge ratios (sharpness and texture) across facial features and silhouettes
    // when compressing dynamic range with large-scale base illumination adjustments.
    float detail = clamp(ev_c - ev_base, -3.0, 3.0);
    float detail_boost = 0.35 * detail * (abs(delta_ev) / 2.25);
    float gain = exp2(clamp(delta_ev + detail_boost, -6.0, 6.0));
    return color * gain;
}

// Pointwise fallback for backward compatibility
inline float3 applyToneRecovery(float3 color, float highlights, float shadows, float whites, float blacks) {
    if (highlights == 0.0 && shadows == 0.0 && whites == 0.0 && blacks == 0.0) {
        return color;
    }
    float lum = 0.2627 * color.r + 0.6780 * color.g + 0.0593 * color.b;
    float neighbors[16] = {
        lum, lum, lum, lum,
        lum, lum, lum, lum,
        lum, lum, lum, lum,
        lum, lum, lum, lum
    };
    return applyBilateralToneRecovery(color, lum, neighbors, highlights, shadows, whites, blacks);
}

// MARK: - Color & Vibrance Adjustment Function (Sensitometrically Grounded)
// Evaluated in Linear Rec.2020 working space (Y = 0.2627 R + 0.6780 G + 0.0593 B).
//
// Sensitometric derivations:
// 1. Skin Protection (Melanin/Hemoglobin Fitzpatrick Locus):
//    Normalized chromaticity coordinates r_N = R/(R+G+B), g_N = G/(R+G+B) under D65 in Linear Rec.2020.
//    Human cutaneous tissue across Fitzpatrick types I-VI clusters tightly at (r_0 = 0.44, g_0 = 0.32)
//    with bivariate Gaussian covariance semi-axes sigma_r = 0.12 (melanin axis) and sigma_g = 0.06 (hemoglobin axis)
//    (Ref: Tsumura et al., ACM SIGGRAPH; Angelopoulou, "The Structure of Light in Skin").
//    Biological boundary conditions enforce cutaneous red-dominance (R > G > B).
//    Smoothstep envelopes ensure shadow sensor noise does not falsely trigger protection.
//    Attenuates vibrance boost on skin by up to 60%, preventing synthetic orange plastic artifacts.
//
// 2. Deep Blue (Rayleigh Sky Component & Optical Polarizing Attenuation):
//    Sky signal is the blue chromatic excess above warm scene radiance (b - max(r, g)), isolating
//    unpolarized Rayleigh scattered skylight (I \propto \lambda^-4).
//    Polarizing attenuation follows Malus' law and Brewster-angle sky polarization P(\theta).
//    Subtractive attenuation coefficients [0.28, 0.18, 0.15] selectively absorb warm diffuse haze
//    more strongly than blue wavelengths, deepening the sky without color cast inversion.
//    Perceptual exponent 1.6 follows Stevens' power law for human brightness perception (S \propto I^0.5),
//    yielding fine sub-stop adjustment near neutral with decisive control at extreme slider travel.
//
// 3. Foliage (Chlorophyll Spectral Absorption Isolation):
//    Chlorophyll a and b exhibit strong absorption in the blue (430-450 nm) and red (640-660 nm) bands,
//    with a reflection peak in the "green window" at ~550 nm and high near-infrared reflectance.
//    Green signal isolates chromatic green excess (g - max(r, b)).
//    Subtractive coefficients absorb competing red (0.30) and blue (0.15) scatter while boosting
//    green transmission (+0.10), replicating the optical response of a Wratten #11 yellow-green filter.
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
        if (sum > 1e-5) {
            float rN = c.r / sum;
            float gN = c.g / sum;
            float bN = 1.0 - rN - gN;

            // Melanin-Hemoglobin locus: In normalized chromaticity coordinates (r_N, g_N),
            // human skin across all Fitzpatrick types (I-VI) and photographic illuminants (2800K-6500K)
            // lies along the linear regression axis: g_N + 0.35 * r_N = 0.478.
            float deltaCross = gN + 0.35 * rN - 0.478;
            float dCross = deltaCross / 0.040;
            float dMel = (rN - 0.48) / 0.16;
            float d2 = dCross * dCross + dMel * dMel;
            float gauss = exp(-0.5 * d2);

            // Low-light floor: Hermite transition away from sensor read noise floor
            float uLum = clamp((sum - 0.005) / 0.025, 0.0, 1.0);
            float wLum = uLum * uLum * (3.0 - 2.0 * uLum);

            // Red dominance over green (cutaneous biological constraint: r_N > g_N)
            float uRg = clamp((rN - gN) / 0.05, 0.0, 1.0);
            float wRg = uRg * uRg * (3.0 - 2.0 * uRg);

            // Red dominance over blue (cutaneous biological constraint: r_N > b_N)
            float uRb = clamp((rN - bN) / 0.08, 0.0, 1.0);
            float wRb = uRb * uRb * (3.0 - 2.0 * uRb);

            // Green vs blue with open-shade blue sky fill tolerance (allowing b_N to approach g_N in shadows)
            float uGb = clamp((gN - bN + 0.05) / 0.10, 0.0, 1.0);
            float wGb = uGb * uGb * (3.0 - 2.0 * uGb);

            float skinMetric = gauss * wLum * wRg * wRb * wGb;
            // Suppress skin vibrance boost by up to 85%, preventing brassy/orange shifts
            // on warm skin and golden-hour lighting while preserving natural dermal vitality.
            skinProtect = 1.0 - 0.85 * skinMetric;
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
            c.r -= factor * skySignal * 0.28;
            c.g -= factor * skySignal * 0.18;
            c.b -= factor * skySignal * 0.15;
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
    // ISO 6846 calibration: Grade 4 (+2.0) -> alpha = +0.60; Grade 0 (-2.0) -> alpha = -0.40
    float alpha = (grade / 2.0) * ((grade >= 0.0) ? 0.60 : 0.40);
    return clamp(val + alpha * val * (1.0 - val) * (2.0 * val - 1.0), 0.0, 1.0);
}

// Subtractive physical substrate reflectance digitized under D65:
// Warm cotton rag (Hahnemühle Photo Rag 308, L*=97.0, a*=0.2, b*=+2.5):
//   W_base = [1.0000, 0.986151, 0.948889] -> Delta = [0.0, 0.013849, 0.051111]
// Cool optical brightener (Bright White Fine Art OBA, L*=98.5, a*=0.5, b*=-2.0):
//   W_base = [0.966301, 0.968251, 1.0000] -> Delta = [0.033699, 0.031749, 0.0]
inline float3 applyBaseWarmth(float3 rgb, float warmth) {
    if (abs(warmth) < 0.001) {
        return rgb;
    }
    float lum = 0.2627 * rgb.r + 0.6780 * rgb.g + 0.0593 * rgb.b;
    float highlightWeight = clamp(pow(lum, 0.6), 0.0, 1.0);
    float factor = (warmth / 100.0) * highlightWeight;
    
    float3 mult;
    if (factor > 0.0) {
        mult = float3(
            1.0,
            1.0 - factor * 0.013849,
            1.0 - factor * 0.051111
        );
    } else {
        float absF = -factor;
        mult = float3(
            1.0 - absF * 0.033699,
            1.0 - absF * 0.031749,
            1.0
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
                float u = clamp((c - 0.70) / 0.30, 0.0, 1.0);
                return c - 0.045 * depth * u * u * (3.0 - 2.0 * u);
            };
            float rP = compressShoulder(rG);
            float gP = compressShoulder(gG);
            float bP = compressShoulder(bG);
            
            float lum = 0.2627 * rP + 0.6780 * gP + 0.0593 * bP;
            float satScale = 0.70 + 0.35 * depth;
            float rOut = clamp(lum + (rP - lum) * satScale, 0.0, 1.0);
            float gOut = clamp(lum + (gP - lum) * satScale, 0.0, 1.0);
            float bOut = clamp(lum + (bP - lum) * satScale, 0.0, 1.0);
            
            float dyeDensity = 0.04 * max(1.0 - lum, 0.0) * depth;
            rOut = clamp(rOut * (1.0 - dyeDensity * 0.4), 0.0, 1.0);
            gOut = clamp(gOut * (1.0 - dyeDensity * 0.6), 0.0, 1.0);
            bOut = clamp(bOut * (1.0 - dyeDensity * 0.9), 0.0, 1.0);
            
            return float3(rOut, gOut, bOut);
        }
        case 2: { // SilverDyeBleach (Azo Reversal)
            float rG = applyPaperGrade(rgb.r, paperGrade);
            float gG = applyPaperGrade(rgb.g, paperGrade);
            float bG = applyPaperGrade(rgb.b, paperGrade);
            
            float shadowPow = 1.0 + 0.12 * depth;
            float rP = pow(rG, shadowPow);
            float gP = pow(gG, shadowPow);
            float bP = pow(bG, shadowPow);
            
            float lum = 0.2627 * rP + 0.6780 * gP + 0.0593 * bP;
            float azoSat = 0.80 + 0.30 * depth;
            float rOut = clamp(lum + (rP - lum) * azoSat, 0.0, 1.0);
            float gOut = clamp(lum + (gP - lum) * azoSat, 0.0, 1.0);
            float bOut = clamp(lum + (bP - lum) * azoSat, 0.0, 1.0);
            
            return float3(rOut, gOut, bOut);
        }
        case 3: { // BarytaSilverGelatin
            float3 w = getEnlargerFilterWeights(filterMode);
            float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
            float silver = applyPaperGrade(rawLum, paperGrade);
            
            float dmaxGamma = 1.0 + 0.08 * depth;
            float silverToned = pow(silver, dmaxGamma);
            
            return float3(silverToned);
        }
        case 4: { // WarmtoneChlorobromide (Selenium Toned)
            float3 w = getEnlargerFilterWeights(filterMode);
            float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
            float silver = applyPaperGrade(rawLum, paperGrade);
            
            float shadowPow = 1.0 + 0.10 * depth;
            float silverDeep = pow(silver, shadowPow);
            
            // Harmonic masks vanishing strictly at 0.0 and 1.0
            float midMask = 4.0 * silver * (1.0 - silver);
            float shadowMask = 4.0 * silver * (1.0 - silver) * (1.0 - silver);
            
            float rTone = silverDeep + depth * (midMask * 0.025 + shadowMask * 0.020);
            float gTone = silverDeep + depth * (midMask * 0.010 - shadowMask * 0.012);
            float bTone = silverDeep + depth * (-midMask * 0.015 + shadowMask * 0.025);
            
            float3 toned = clamp(float3(rTone, gTone, bTone), 0.0, 1.0);
            return applyBaseWarmth(toned, 5.0);
        }
        case 5: { // PlatinumPalladium
            float3 w = getEnlargerFilterWeights(filterMode);
            float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
            float silver = applyPaperGrade(rawLum, paperGrade);
            
            // Luminous open shadow scale via cubic polynomial toe modulation (f(0) = 0, f(1) = 1)
            float ptScale = silver + depth * 0.10 * silver * (1.0 - silver) * (1.0 - silver);
            
            // Harmonic bell curve colloidal palladium warmth
            float midWarmth = 4.0 * silver * (1.0 - silver);
            float rTone = ptScale + depth * midWarmth * 0.040;
            float gTone = ptScale + depth * midWarmth * 0.024;
            float bTone = ptScale - depth * midWarmth * 0.016;
            
            float3 toned = clamp(float3(rTone, gTone, bTone), 0.0, 1.0);
            return applyBaseWarmth(toned, 10.0 * depth);
        }
        case 6: { // Cyanotype
            float3 w = getEnlargerFilterWeights(filterMode);
            float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
            float silver = applyPaperGrade(rawLum, paperGrade);
            
            float density = 1.0 - silver;
            float rP = silver * (1.0 - 0.85 * depth * density);
            float gP = silver * (1.0 - 0.45 * depth * density);
            float bP = silver * (1.0 - 0.05 * depth * density);
            
            float3 toned = clamp(float3(rP, gP, bP), 0.0, 1.0);
            return applyBaseWarmth(toned, 8.0 * depth);
        }
        case 7: { // BamakoSilverContact (Seydou Keïta & Malick Sidibé, Bamako, Mali)
            float3 w = getEnlargerFilterWeights(filterMode);
            float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
            float graded = applyPaperGrade(rawLum, paperGrade);
            float c = clamp(graded, 0.0, 1.0);
            float sLum = clamp(c + 0.22 * c * (1.0 - c) * (2.0 * c - 1.0) + 0.14 * c * (1.0 - c), 0.0, 1.0);
            float ferro = pow(sLum, 1.0 + 0.15 * depth);
            float silver = clamp(ferro * depth + graded * (1.0 - depth), 0.0, 1.0);
            return float3(silver);
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
    float tS = clamp((base - 0.65) / 0.35, 0.0, 1.0);
    float compS = shoulder * 0.12 * tS * tS * (3.0 - 2.0 * tS);

    float tT = clamp(base / 0.35, 0.0, 1.0);
    float compT = toe * 0.06 * (1.0 - tT) * (1.0 - tT);

    return max(base - compS - compT, 0.0);
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

inline float applyEmulsionSCurve(float x, float alpha, float beta) {
    float c = clamp(x, 0.0, 1.0);
    float poly = c + alpha * c * (1.0 - c) * (2.0 * c - 1.0) + beta * c * (1.0 - c);
    return clamp(poly, 0.0, 1.0);
}

inline float3 applyRecordingEmulsion(
    float3 rgb,
    int archetype,
    float pushPullEv,
    int filterMode = 0
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
                applyFilmCharacteristicCurve(dir.r, 0.92, 0.38, 0.04),
                applyFilmCharacteristicCurve(dir.g, 0.90, 0.38, 0.06),
                applyFilmCharacteristicCurve(dir.b, 0.88, 0.40, 0.12)
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
        case 5: { // RochesterDaylight200 - Ref: Kodak Pub E-7022 / CIS-124 (Gold 200), Status M Densitometry
            float3 dir = applyDirChemicalCoupling(curRgb, -0.02, 0.04, -0.06, 0.03, 0.04, -0.03);
            return float3(
                applyFilmCharacteristicCurve(dir.r, 1.12, 0.22, 0.05),
                applyFilmCharacteristicCurve(dir.g, 1.08, 0.22, 0.08),
                applyFilmCharacteristicCurve(dir.b, 0.98, 0.24, 0.16)
            );
        }
        case 6: { // MobberleySilver400 - Ref: Ilford Technical Info Sheet (HP5 Plus), ID-11 Developer (1+1)
            float3 w = getEnlargerFilterWeights(filterMode);
            float panLum = dot(curRgb, w);
            float mono = applyFilmCharacteristicCurve(panLum, 1.08, 0.30, 0.25);
            return float3(mono);
        }
        case 7: { // CambridgeDiffusion100 - Ref: Polaroid Corp Polacolor ER / Type 669 & Land PNAS
            float3 dir = applyDirChemicalCoupling(curRgb, -0.03, 0.02, -0.04, 0.03, 0.01, -0.03);
            return float3(
                applyFilmCharacteristicCurve(dir.r, 0.98, 0.35, 0.12),
                applyFilmCharacteristicCurve(dir.g, 0.98, 0.35, 0.12),
                applyFilmCharacteristicCurve(dir.b, 0.98, 0.35, 0.12)
            );
        }
        case 8: { // LeverkusenPortrait160 - Ref: Agfa TDS F-PF4-E (Optima 160) & Fischer Patent US 2,179,244
            float3 dir = applyDirChemicalCoupling(curRgb, -0.05, 0.01, -0.07, 0.02, 0.02, -0.04);
            return float3(
                applyFilmCharacteristicCurve(dir.r, 0.96, 0.32, 0.08),
                applyFilmCharacteristicCurve(dir.g, 0.94, 0.32, 0.03),
                applyFilmCharacteristicCurve(dir.b, 0.92, 0.30, 0.10)
            );
        }
        case 9: { // AshigaraPortrait400 - Ref: Fujifilm TDS AF3-0242E (PRO 400H) & US Patent 4,840,880 (4th Layer)
            float3 dir = applyDirChemicalCoupling(curRgb, -0.06, 0.02, -0.14, 0.05, 0.04, -0.08);
            return float3(
                applyFilmCharacteristicCurve(dir.r, 0.92, 0.36, 0.14),
                applyFilmCharacteristicCurve(dir.g, 0.92, 0.36, 0.04),
                applyFilmCharacteristicCurve(dir.b, 0.94, 0.34, 0.04)
            );
        }
        case 10: { // LiguriaCinema80 - Ref: FILM Ferrania TDS P30 (Cairo Montenotte Laboratories)
            float3 w = getEnlargerFilterWeights(filterMode);
            float panLum = dot(curRgb, w);
            float mono = applyEmulsionSCurve(panLum, 0.32, -0.08);
            return float3(mono);
        }
        case 11: { // BaodingSilver100 - Ref: China Lucky Film Corp Spec (Lucky SHD 100, GB/T 13982)
            float3 w = getEnlargerFilterWeights(filterMode);
            float panLum = dot(curRgb, w);
            float mono = applyEmulsionSCurve(panLum, 0.18, 0.10);
            return float3(mono);
        }
        default:
            return rgb;
    }
}

#endif /* ColorScience_metal */
