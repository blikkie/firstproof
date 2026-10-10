/**
 * First Proof — WebGL Color Science Pipeline
 * Transcribed 1:1 from FirstProofApp/Sources/Shaders/ColorScience.metal
 *
 * Implements:
 * - Wide-gamut color spaces (Linear Rec.2020 <-> Linear sRGB)
 * - Hue, Saturation, Value (HSV) optical transforms
 * - AgX Filmic and AgX Vivid tone mapping operators
 * - Camera manufacturer matching profiles (Standard, Portrait, Landscape, Vivid)
 * - Sensitometric tone recovery (Highlights, Shadows, Whites, Blacks)
 * - Photochemical recording emulsions (Rochester Portrait, Ashigara Chrome, etc.)
 * - Memory color adjustments (Vibrance, Saturation, Deep Blue, Foliage)
 * - Optical split-toning & color grading
 * - Darkroom fine-art print media chemistry (RA-4, Silver Dye Bleach, Cotton Rag)
 */

export const COLOR_SCIENCE_GLSL = `
// Working Space Matrices (Column-major in GLSL matching Metal float3x3)
const mat3 LINEAR_SRGB_TO_REC2020 = mat3(
    0.6274041, 0.0690972, 0.0163914,
    0.3292821, 0.9195411, 0.0880133,
    0.0433138, 0.0113617, 0.8955953
);

const mat3 REC2020_TO_LINEAR_SRGB = mat3(
    1.6604910, -0.1245505, -0.0181508,
    -0.5876411,  1.1328999, -0.1005789,
    -0.0728499, -0.0083494,  1.1187297
);

const mat3 AGX_INSET_MATRIX = mat3(
    1.4096986, 0.13136263, 0.16151477,
    -0.4004796, 0.77151406, -0.06034984,
    -0.009218898, 0.09712328, 0.89883506
);

const mat3 AGX_OUTSET_MATRIX = mat3(
    1.1271005, -0.1413298, -0.1413298,
    -0.1106066, 1.1578237, -0.1106066,
    -0.0164939, -0.0164939, 1.2519364
);

const float AGX_MIN_EV = -10.0;
const float AGX_MAX_EV = 6.5;
const float AGX_EV_RANGE = 16.5;
const float AGX_MID_GRAY_NORM = 0.6060606; // (0 - (-10)) / 16.5
const float AGX_GAMMA = 2.2;

// Math primitives
float log10(float x) {
    return log2(x) * 0.3010299956639812;
}

// Transfer functions
float srgbGammaToLinear(float c) {
    return (c <= 0.04045) ? (c / 12.92) : pow((c + 0.055) / 1.055, 2.4);
}

vec3 srgbGammaToLinear3(vec3 c) {
    return vec3(srgbGammaToLinear(c.r), srgbGammaToLinear(c.g), srgbGammaToLinear(c.b));
}

float linearToSrgbGamma(float c) {
    return (c <= 0.0031308) ? (c * 12.92) : (1.055 * pow(max(c, 0.0), 1.0 / 2.4) - 0.055);
}

vec3 linearToSrgbGamma3(vec3 c) {
    return vec3(linearToSrgbGamma(c.r), linearToSrgbGamma(c.g), linearToSrgbGamma(c.b));
}

// HSV / RGB Conversions
vec3 rgb2hsv(vec3 c) {
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
    return vec3(h, s, maxV);
}

vec3 hsv2rgb(vec3 hsv) {
    float h = hsv.x / 60.0;
    float s = hsv.y;
    float v = hsv.z;
    
    float c = v * s;
    float x = c * (1.0 - abs(mod(h, 2.0) - 1.0));
    float m = v - c;
    
    vec3 rgb = vec3(0.0);
    if (h < 1.0) rgb = vec3(c, x, 0.0);
    else if (h < 2.0) rgb = vec3(x, c, 0.0);
    else if (h < 3.0) rgb = vec3(0.0, c, x);
    else if (h < 4.0) rgb = vec3(0.0, x, c);
    else if (h < 5.0) rgb = vec3(x, 0.0, c);
    else rgb = vec3(c, 0.0, x);
    return rgb + vec3(m);
}

// Calibrated Camera-Matching Monotonic Cubic Hermite Tone Curves (Fritsch-Carlson C^1 Continuous)
const vec2 DCP_KNOTS_STANDARD[7] = vec2[7](
    vec2(0.00, 0.000), vec2(0.02, 0.008), vec2(0.05, 0.028),
    vec2(0.18, 0.190), vec2(0.50, 0.540), vec2(0.80, 0.840), vec2(1.00, 1.000)
);
const float DCP_TANGENTS_STANDARD[7] = float[7](0.4000, 0.5333, 0.9564, 1.1700, 1.0469, 0.9000, 0.8000);

const vec2 DCP_KNOTS_PORTRAIT[7] = vec2[7](
    vec2(0.00, 0.000), vec2(0.02, 0.010), vec2(0.05, 0.035),
    vec2(0.18, 0.185), vec2(0.50, 0.510), vec2(0.80, 0.820), vec2(1.00, 1.000)
);
const float DCP_TANGENTS_PORTRAIT[7] = float[7](0.5000, 0.6667, 0.9936, 1.0847, 1.0245, 0.9667, 0.9000);

const vec2 DCP_KNOTS_LANDSCAPE[7] = vec2[7](
    vec2(0.00, 0.000), vec2(0.02, 0.006), vec2(0.05, 0.022),
    vec2(0.18, 0.200), vec2(0.50, 0.560), vec2(0.80, 0.860), vec2(1.00, 1.000)
);
const float DCP_TANGENTS_LANDSCAPE[7] = float[7](0.3000, 0.4167, 0.9513, 1.2471, 1.0625, 0.8500, 0.7000);

const vec2 DCP_KNOTS_VIVID[7] = vec2[7](
    vec2(0.00, 0.000), vec2(0.02, 0.004), vec2(0.05, 0.018),
    vec2(0.18, 0.210), vec2(0.50, 0.580), vec2(0.80, 0.880), vec2(1.00, 1.000)
);
const float DCP_TANGENTS_VIVID[7] = float[7](0.2000, 0.3333, 0.9718, 1.3166, 1.0781, 0.8000, 0.6000);

float evalCameraHermite7(float x, vec2 knots[7], float m[7]) {
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
float applyCameraToneCurve(float x, int mode) {
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


// AgX Filmic Tone Curve Polynomial (derived from Troy Sobotka reference AgX OCIO LUT)
float agxApplyCurveChannel(float x) {
    float xc = clamp(x, 0.0, 1.0);
    // Horner evaluation: (((((c6 * x + c5) * x + c4) * x + c3) * x + c2) * x + c1) * x
    float curved = (((((15.45672 * xc - 40.02792) * xc + 31.87076) * xc - 6.84882) * xc + 0.42860) * xc + 0.12066) * xc;
    return clamp(curved, 0.0, 1.0);
}

vec3 agxToneMap(vec3 rec2020Color, float contrastGain) {
    float minC = min(rec2020Color.r, min(rec2020Color.g, rec2020Color.b));
    vec3 cComp = (minC < 0.0) ? (rec2020Color - minC) : rec2020Color;

    vec3 inset = AGX_INSET_MATRIX * cComp;
    vec3 xRel = max(inset / 0.18, vec3(1e-6));
    vec3 logEncoded = (log2(xRel) - vec3(AGX_MIN_EV)) / AGX_EV_RANGE;
    vec3 norm = clamp(logEncoded, 0.0, 1.0);
    
    if (abs(contrastGain - 1.0) > 0.001) {
        norm = clamp(vec3(AGX_MID_GRAY_NORM) + (norm - vec3(AGX_MID_GRAY_NORM)) * contrastGain, 0.0, 1.0);
    }
    
    vec3 curved;
    curved.r = agxApplyCurveChannel(norm.r);
    curved.g = agxApplyCurveChannel(norm.g);
    curved.b = agxApplyCurveChannel(norm.b);
    
    vec3 linearized = pow(max(curved, vec3(0.0)), vec3(AGX_GAMMA));
    return max(AGX_OUTSET_MATRIX * linearized, vec3(0.0));
}

vec3 agxToneMapVivid(vec3 rec2020Color, float contrastGain) {
    float punchyContrast = contrastGain * 1.15;
    vec3 base = agxToneMap(rec2020Color, punchyContrast);

    float lum = 0.2126 * base.r + 0.7152 * base.g + 0.0722 * base.b;
    float sat = 1.20;
    return clamp(vec3(
        lum + sat * (base.r - lum),
        lum + sat * (base.g - lum),
        lum + sat * (base.b - lum)
    ), 0.0, 1.0);
}

// Advanced Bilateral Base/Detail Tone Recovery (Highlights, Shadows, Whites, Blacks)
vec3 applyBilateralToneRecovery(
    vec3 color,
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
    // Range weights: Gaussian in EV stops with sigma_r = 0.85 EV (2 * sigma_r^2 = 1.445)
    const float w_s_ortho = 0.60653066;
    const float w_s_diag  = 0.36787944;
    const float inv_two_sigma_r_sq = 1.0 / 1.445;
    
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
    
    if (blacks != 0.0) {
        float u = clamp((-ev_base - 1.0) / 3.0, 0.0, 1.0);
        float w_blacks = u * u * (3.0 - 2.0 * u);
        delta_ev += (blacks / 100.0) * 0.90 * w_blacks;
    }
    
    if (shadows != 0.0) {
        float u = clamp(-ev_base / 3.0, 0.0, 1.0);
        float w_shadows = u * u * (3.0 - 2.0 * u);
        delta_ev += (shadows / 100.0) * 1.20 * w_shadows;
    }
    
    if (highlights != 0.0) {
        float u = clamp(ev_base / 3.0, 0.0, 1.0);
        float w_highlights = u * u * (3.0 - 2.0 * u);
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
vec3 applyToneRecovery(vec3 color, float highlights, float shadows, float whites, float blacks) {
    if (highlights == 0.0 && shadows == 0.0 && whites == 0.0 && blacks == 0.0) {
        return color;
    }
    float lum = dot(color, vec3(0.2627, 0.6780, 0.0593));
    float neighbors[16] = float[16](
        lum, lum, lum, lum,
        lum, lum, lum, lum,
        lum, lum, lum, lum,
        lum, lum, lum, lum
    );
    return applyBilateralToneRecovery(color, lum, neighbors, highlights, shadows, whites, blacks);
}

// Color Adjustments: Vibrance, Saturation, Deep Blue, Foliage
vec3 applyColorAdjustments(vec3 rgb, float vibrance, float saturation, float deepBlue, float foliage) {
    if (vibrance == 0.0 && saturation == 0.0 && deepBlue == 0.0 && foliage == 0.0) {
        return rgb;
    }
    vec3 c = clamp(rgb, 0.0, 1.0);
    float lum = dot(c, vec3(0.2627, 0.6780, 0.0593));

    if (saturation != 0.0) {
        float satGain = max(1.0 + saturation / 100.0, 0.0);
        c = vec3(lum) + (c - vec3(lum)) * satGain;
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
            float dR = (rN - 0.44) / 0.12;
            float dG = (gN - 0.32) / 0.06;
            float d2 = dR * dR + dG * dG;
            float gauss = exp(-0.5 * d2);

            float uLum = clamp((sum - 0.005) / 0.025, 0.0, 1.0);
            float wLum = uLum * uLum * (3.0 - 2.0 * uLum);

            float uRg = clamp((rN - gN + 0.02) / 0.06, 0.0, 1.0);
            float wRg = uRg * uRg * (3.0 - 2.0 * uRg);

            float bN = 1.0 - rN - gN;
            float uGb = clamp((gN - bN + 0.02) / 0.06, 0.0, 1.0);
            float wGb = uGb * uGb * (3.0 - 2.0 * uGb);

            float skinMetric = gauss * wLum * wRg * wGb;
            skinProtect = 1.0 - 0.6 * skinMetric;
        }

        float vibGain = (vibrance > 0.0)
            ? (vibrance / 100.0) * (1.0 - chroma) * 0.8 * skinProtect
            : (vibrance / 100.0) * 0.6;

        c += (c - vec3(lum)) * vibGain;
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

// Optical Split-Toning
vec3 applySplitToning(vec3 rgb, float shadowHue, float shadowSat, float highlightHue, float highlightSat, float balance) {
    if (shadowSat <= 0.0 && highlightSat <= 0.0) {
        return rgb;
    }
    float lum = dot(clamp(rgb, 0.0, 1.0), vec3(0.2627, 0.6780, 0.0593));
    float pivot = clamp(0.50 + balance * 0.0035, 0.05, 0.95);

    float wShadow = 1.0 - smoothstep(0.0, pivot, lum);
    float wHighlight = smoothstep(pivot, 1.0, lum);

    vec3 c = rgb;
    if (shadowSat > 0.0 && wShadow > 0.0) {
        vec3 sColor = hsv2rgb(vec3(shadowHue, clamp(shadowSat / 100.0, 0.0, 1.0), 1.0));
        c = mix(c, c * sColor, wShadow * 0.35);
    }
    if (highlightSat > 0.0 && wHighlight > 0.0) {
        vec3 hColor = hsv2rgb(vec3(highlightHue, clamp(highlightSat / 100.0, 0.0, 1.0), 1.0));
        c = mix(c, 1.0 - (1.0 - c) * (1.0 - hColor * 0.35), wHighlight);
    }
    return clamp(c, 0.0, 1.0);
}

// Photochemical Recording Emulsion (Film Stocks)
float applyFilmCharacteristicCurve(float val, float gamma, float shoulder, float toe) {
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

vec3 applyDirChemicalCoupling(vec3 rgb, float c_rg, float c_rb, float c_gr, float c_gb, float c_br, float c_bg) {
    float eps = 1e-4;
    // Optical density D = -log10(T) = -log2(T) / log2(10)
    float d_r = -log2(max(rgb.r, eps)) * 0.30102999566;
    float d_g = -log2(max(rgb.g, eps)) * 0.30102999566;
    float d_b = -log2(max(rgb.b, eps)) * 0.30102999566;

    float c_rr = 1.0 - c_rg - c_rb;
    float c_gg = 1.0 - c_gr - c_gb;
    float c_bb = 1.0 - c_br - c_bg;

    float d_r_out = c_rr * d_r + c_rg * d_g + c_rb * d_b;
    float d_g_out = c_gr * d_r + c_gg * d_g + c_gb * d_b;
    float d_b_out = c_br * d_r + c_bg * d_g + c_bb * d_b;

    return clamp(vec3(
        pow(10.0, -d_r_out),
        pow(10.0, -d_g_out),
        pow(10.0, -d_b_out)
    ), 0.0, 2.0);
}

vec3 getEnlargerFilterWeights(int filterMode) {
    if (filterMode == 1) return vec3(0.35, 0.60, 0.05); // YellowK2
    if (filterMode == 2) return vec3(0.55, 0.42, 0.03); // Orange16
    if (filterMode == 3) return vec3(0.85, 0.15, 0.00); // Red25A
    if (filterMode == 4) return vec3(0.15, 0.80, 0.05); // GreenX1
    if (filterMode == 5) return vec3(0.00, 0.60, 0.40); // Orthochromatic
    return vec3(0.2627, 0.6780, 0.0593); // NeutralPanchromatic (Linear Rec.2020 photopic luminance)
}

float applyEmulsionSCurve(float x, float alpha, float beta) {
    float c = clamp(x, 0.0, 1.0);
    return clamp(c + alpha * c * (1.0 - c) * (2.0 * c - 1.0) + beta * c * (1.0 - c), 0.0, 1.0);
}

vec3 applyRecordingEmulsion(vec3 rgb, int archetype, float pushPullEv, int filterMode) {
    if (archetype == 0) return rgb;

    float expGain = pow(2.0, clamp(pushPullEv, -2.0, 2.0));
    vec3 cur = max(rgb * expGain, vec3(0.0));

    if (archetype == 1) { // Rochester Portrait 400 (Kodak Portra 400)
        vec3 dir = applyDirChemicalCoupling(cur, -0.04, 0.02, -0.08, 0.02, 0.02, -0.05);
        return vec3(
            applyFilmCharacteristicCurve(dir.r, 0.92, 0.38, 0.04),
            applyFilmCharacteristicCurve(dir.g, 0.90, 0.38, 0.06),
            applyFilmCharacteristicCurve(dir.b, 0.88, 0.40, 0.12)
        );
    } else if (archetype == 2) { // Ashigara Chrome 50
        vec3 dir = applyDirChemicalCoupling(cur, -0.15, -0.08, -0.18, -0.08, -0.06, -0.12);
        return vec3(
            applyFilmCharacteristicCurve(dir.r, 1.10, 0.12, 0.14),
            applyFilmCharacteristicCurve(dir.g, 1.12, 0.10, 0.14),
            applyFilmCharacteristicCurve(dir.b, 1.11, 0.12, 0.16)
        );
    } else if (archetype == 3) { // Rochester K-14 Chrome 64
        vec3 dir = applyDirChemicalCoupling(cur, -0.08, 0.06, -0.10, 0.04, 0.08, -0.04);
        return vec3(
            applyFilmCharacteristicCurve(dir.r, 1.05, 0.18, 0.08),
            applyFilmCharacteristicCurve(dir.g, 1.06, 0.16, 0.10),
            applyFilmCharacteristicCurve(dir.b, 1.08, 0.14, 0.14)
        );
    } else if (archetype == 4) { // Rochester Cinema 500T
        vec3 dir = applyDirChemicalCoupling(cur, -0.02, 0.01, -0.03, 0.01, 0.01, -0.02);
        return vec3(
            applyFilmCharacteristicCurve(dir.r, 0.82, 0.45, 0.08),
            applyFilmCharacteristicCurve(dir.g, 0.82, 0.45, 0.08),
            applyFilmCharacteristicCurve(dir.b, 0.82, 0.45, 0.08)
        );
    } else if (archetype == 5) { // Rochester Daylight 200 (Kodak Gold 200)
        vec3 dir = applyDirChemicalCoupling(cur, -0.02, 0.04, -0.06, 0.03, 0.04, -0.03);
        return vec3(
            applyFilmCharacteristicCurve(dir.r, 1.12, 0.22, 0.05),
            applyFilmCharacteristicCurve(dir.g, 1.08, 0.22, 0.08),
            applyFilmCharacteristicCurve(dir.b, 0.98, 0.24, 0.16)
        );
    } else if (archetype == 6) { // Mobberley Silver 400 (B&W)
        vec3 w = getEnlargerFilterWeights(filterMode);
        float panLum = dot(cur, w);
        float mono = applyFilmCharacteristicCurve(panLum, 1.08, 0.30, 0.25);
        return vec3(mono);
    } else if (archetype == 7) { // Cambridge Instant Diffusion 100
        vec3 dir = applyDirChemicalCoupling(cur, -0.03, 0.02, -0.04, 0.03, 0.01, -0.03);
        vec3 curv = vec3(
            applyFilmCharacteristicCurve(dir.r, 0.98, 0.35, 0.12),
            applyFilmCharacteristicCurve(dir.g, 0.98, 0.35, 0.12),
            applyFilmCharacteristicCurve(dir.b, 0.98, 0.35, 0.12)
        );
        float lum = dot(curv, vec3(0.2627, 0.6780, 0.0593));
        float baseRefl = 0.025 * max(1.0 - lum, 0.0);
        return vec3(
            min(curv.r + baseRefl * 1.02, 2.0),
            min(curv.g + baseRefl * 1.00, 2.0),
            min(curv.b + baseRefl * 0.94, 2.0)
        );
    } else if (archetype == 8) { // Leverkusen Portrait 160 (Agfa Optima 160)
        vec3 dir = applyDirChemicalCoupling(cur, -0.05, 0.01, -0.07, 0.02, 0.02, -0.04);
        return vec3(
            applyFilmCharacteristicCurve(dir.r, 0.96, 0.32, 0.08),
            applyFilmCharacteristicCurve(dir.g, 0.94, 0.32, 0.03),
            applyFilmCharacteristicCurve(dir.b, 0.92, 0.30, 0.10)
        );
    } else if (archetype == 9) { // Ashigara Portrait 400 (Fujicolor Pro 400H)
        vec3 dir = applyDirChemicalCoupling(cur, -0.06, 0.02, -0.14, 0.05, 0.04, -0.08);
        return vec3(
            applyFilmCharacteristicCurve(dir.r, 0.92, 0.36, 0.14),
            applyFilmCharacteristicCurve(dir.g, 0.92, 0.36, 0.04),
            applyFilmCharacteristicCurve(dir.b, 0.94, 0.34, 0.04)
        );
    } else if (archetype == 10) { // Liguria Cinema 80 (Ferrania P30, Cairo Montenotte, Italy)
        vec3 w = getEnlargerFilterWeights(filterMode);
        float panLum = dot(cur, w);
        float mono = applyEmulsionSCurve(panLum, 0.32, -0.08);
        return vec3(mono);
    } else if (archetype == 11) { // Baoding Silver 100 (Lucky SHD 100, Baoding, China)
        vec3 w = getEnlargerFilterWeights(filterMode);
        float panLum = dot(cur, w);
        float mono = applyEmulsionSCurve(panLum, 0.18, 0.10);
        return vec3(mono);
    }
    return cur;
}

// Darkroom Fine-Art Print Media
float applyPaperGrade(float val, float grade) {
    if (abs(grade) < 0.001) {
        return val;
    }
    // ISO 6846 calibration: Grade 4 (+2.0) -> alpha = +0.60; Grade 0 (-2.0) -> alpha = -0.40
    float alpha = (grade / 2.0) * ((grade >= 0.0) ? 0.60 : 0.40);
    return clamp(val + alpha * val * (1.0 - val) * (2.0 * val - 1.0), 0.0, 1.0);
}

vec3 applyBaseWarmth(vec3 rgb, float warmth) {
    if (abs(warmth) < 0.001) {
        return rgb;
    }
    float lum = 0.2627 * rgb.r + 0.6780 * rgb.g + 0.0593 * rgb.b;
    float highlightWeight = clamp(pow(lum, 0.6), 0.0, 1.0);
    float factor = (warmth / 100.0) * highlightWeight;

    vec3 mult;
    if (factor > 0.0) {
        mult = vec3(
            1.0,
            1.0 - factor * 0.013849,
            1.0 - factor * 0.051111
        );
    } else {
        float absF = -factor;
        mult = vec3(
            1.0 - absF * 0.033699,
            1.0 - absF * 0.031749,
            1.0
        );
    }
    return clamp(rgb * mult, 0.0, 1.0);
}

vec3 applyPrintMedia(vec3 rgb, int mediaMode, int filterMode, float paperGrade, float processDepth) {
    if (mediaMode == 0) {
        return rgb;
    }

    float depth = clamp(processDepth / 100.0, 0.0, 1.0);

    if (mediaMode == 1) { // ChromogenicTypeC (RA-4)
        float rG = applyPaperGrade(rgb.r, paperGrade);
        float gG = applyPaperGrade(rgb.g, paperGrade);
        float bG = applyPaperGrade(rgb.b, paperGrade);

        float uR = clamp((rG - 0.70) / 0.30, 0.0, 1.0);
        float rP = rG - 0.045 * depth * uR * uR * (3.0 - 2.0 * uR);

        float uG = clamp((gG - 0.70) / 0.30, 0.0, 1.0);
        float gP = gG - 0.045 * depth * uG * uG * (3.0 - 2.0 * uG);

        float uB = clamp((bG - 0.70) / 0.30, 0.0, 1.0);
        float bP = bG - 0.045 * depth * uB * uB * (3.0 - 2.0 * uB);

        float lum = 0.2627 * rP + 0.6780 * gP + 0.0593 * bP;
        float satScale = 0.70 + 0.35 * depth;
        float rOut = clamp(lum + (rP - lum) * satScale, 0.0, 1.0);
        float gOut = clamp(lum + (gP - lum) * satScale, 0.0, 1.0);
        float bOut = clamp(lum + (bP - lum) * satScale, 0.0, 1.0);

        float dyeDensity = 0.04 * max(1.0 - lum, 0.0) * depth;
        rOut = clamp(rOut * (1.0 - dyeDensity * 0.4), 0.0, 1.0);
        gOut = clamp(gOut * (1.0 - dyeDensity * 0.6), 0.0, 1.0);
        bOut = clamp(bOut * (1.0 - dyeDensity * 0.9), 0.0, 1.0);

        return vec3(rOut, gOut, bOut);
    } else if (mediaMode == 2) { // SilverDyeBleach (Azo Reversal)
        float rG = applyPaperGrade(rgb.r, paperGrade);
        float gG = applyPaperGrade(rgb.g, paperGrade);
        float bG = applyPaperGrade(rgb.b, paperGrade);

        float shadowPow = 1.0 + 0.12 * depth;
        float rP = pow(max(rG, 0.0), shadowPow);
        float gP = pow(max(gG, 0.0), shadowPow);
        float bP = pow(max(bG, 0.0), shadowPow);

        float lum = 0.2627 * rP + 0.6780 * gP + 0.0593 * bP;
        float azoSat = 0.80 + 0.30 * depth;
        float rOut = clamp(lum + (rP - lum) * azoSat, 0.0, 1.0);
        float gOut = clamp(lum + (gP - lum) * azoSat, 0.0, 1.0);
        float bOut = clamp(lum + (bP - lum) * azoSat, 0.0, 1.0);

        return vec3(rOut, gOut, bOut);
    } else if (mediaMode == 3) { // BarytaSilverGelatin
        vec3 w = getEnlargerFilterWeights(filterMode);
        float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
        float silver = applyPaperGrade(rawLum, paperGrade);

        float dmaxGamma = 1.0 + 0.08 * depth;
        float silverToned = pow(silver, dmaxGamma);

        return vec3(silverToned);
    } else if (mediaMode == 4) { // WarmtoneChlorobromide (Selenium Toned)
        vec3 w = getEnlargerFilterWeights(filterMode);
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

        vec3 toned = clamp(vec3(rTone, gTone, bTone), 0.0, 1.0);
        return applyBaseWarmth(toned, 5.0);
    } else if (mediaMode == 5) { // PlatinumPalladium
        vec3 w = getEnlargerFilterWeights(filterMode);
        float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
        float silver = applyPaperGrade(rawLum, paperGrade);

        // Luminous open shadow scale via cubic polynomial toe modulation (f(0) = 0, f(1) = 1)
        float ptScale = silver + depth * 0.10 * silver * (1.0 - silver) * (1.0 - silver);

        // Harmonic bell curve colloidal palladium warmth
        float midWarmth = 4.0 * silver * (1.0 - silver);
        float rTone = ptScale + depth * midWarmth * 0.040;
        float gTone = ptScale + depth * midWarmth * 0.024;
        float bTone = ptScale - depth * midWarmth * 0.016;

        vec3 toned = clamp(vec3(rTone, gTone, bTone), 0.0, 1.0);
        return applyBaseWarmth(toned, 10.0 * depth);
    } else if (mediaMode == 6) { // Cyanotype
        vec3 w = getEnlargerFilterWeights(filterMode);
        float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
        float silver = applyPaperGrade(rawLum, paperGrade);

        float density = 1.0 - silver;
        float rP = silver * (1.0 - 0.85 * depth * density);
        float gP = silver * (1.0 - 0.45 * depth * density);
        float bP = silver * (1.0 - 0.05 * depth * density);

        vec3 toned = clamp(vec3(rP, gP, bP), 0.0, 1.0);
        return applyBaseWarmth(toned, 8.0 * depth);
    } else if (mediaMode == 7) { // BamakoSilverContact (Seydou Keïta & Malick Sidibé)
        vec3 w = getEnlargerFilterWeights(filterMode);
        float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
        float graded = applyPaperGrade(rawLum, paperGrade);
        float c = clamp(graded, 0.0, 1.0);
        float sLum = clamp(c + 0.22 * c * (1.0 - c) * (2.0 * c - 1.0) + 0.14 * c * (1.0 - c), 0.0, 1.0);
        float ferro = pow(max(sLum, 0.0), 1.0 + 0.15 * depth);
        float silver = clamp(ferro * depth + graded * (1.0 - depth), 0.0, 1.0);
        return vec3(silver);
    }

    return rgb;
}

float proceduralGrainNoise(float u, float v) {
    vec2 p = 50.0 * fract(vec2(u, v) * 0.3183099 + vec2(0.71, 0.113));
    return -1.0 + 2.0 * fract(p.x * p.y * (p.x + p.y));
}

float clusteredGrainNoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);

    float a = proceduralGrainNoise(i.x, i.y);
    float b = proceduralGrainNoise(i.x + 1.0, i.y);
    float c = proceduralGrainNoise(i.x, i.y + 1.0);
    float d = proceduralGrainNoise(i.x + 1.0, i.y + 1.0);

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

float evalGrainProcNoise(vec2 pos, float s, bool isInstant) {
    if (isInstant) {
        return 0.60 * clusteredGrainNoise(pos / s)
            + 0.40 * clusteredGrainNoise(pos / (s * 1.8) + vec2(17.3, 31.7));
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

float evalGrainEta(float procVal, float sensorSeed, int grainMode, bool isInstant) {
    if (grainMode == 1) { // SeedFromSensor
        if (isInstant) {
            return clamp(0.55 * procVal + 0.20 * sensorSeed, -1.0, 1.0);
        } else {
            return clamp(procVal * (0.45 + 0.55 * abs(sensorSeed)) + 0.45 * sensorSeed, -1.0, 1.0);
        }
    } else if (grainMode == 2) { // SynthesizePhysical
        if (isInstant) {
            return clamp(0.85 * procVal + 0.05 * sensorSeed, -1.0, 1.0);
        } else {
            return clamp(0.90 * procVal + 0.10 * sensorSeed, -1.0, 1.0);
        }
    }
    return 0.0;
}

vec3 applyStochasticEmulsionGrain(
    vec3 color,
    float y_c,
    float y_l,
    float y_r,
    float y_t,
    float y_b,
    float y_tl,
    float y_tr,
    float y_bl,
    float y_br,
    vec2 p,
    int grainMode,
    float clusterSize,
    float grainAmount,
    int archetype
) {
    if (grainMode == 0) {
        return color;
    }

    float amount = clamp(grainAmount, 0.0, 2.0);
    if (amount < 0.001) {
        return color;
    }

    // 1. Isotropic 9-point lowpass luminance (evaluated at strictly 1-pixel pitch to isolate sensor read noise)
    float yLowpass = (1.0 / 6.0) * (y_l + y_r + y_t + y_b)
        + (1.0 / 12.0) * (y_tl + y_tr + y_bl + y_br);
    float deltaSensor = y_c - yLowpass;

    // Sobel-Feldman 8-neighbor gradient magnitude
    float gx = ((y_tr + 2.0 * y_r + y_br) - (y_tl + 2.0 * y_l + y_bl)) * 0.125;
    float gy = ((y_bl + 2.0 * y_b + y_br) - (y_tl + 2.0 * y_t + y_tr)) * 0.125;
    float gradMag = sqrt(gx * gx + gy * gy);

    float edgeSuppress = 1.0 / (1.0 + 35.0 * gradMag);
    float deltaNoise = deltaSensor * edgeSuppress;

    float seedGain = 20.0;
    float denom = max(sqrt(max(y_c, 0.0)), 0.05);
    float sensorSeed = clamp((deltaNoise / denom) * seedGain, -1.0, 1.0);

    // 2. Stochastic carrier: Polaroid DDT dye diffusion mottling vs silver crystal lattice
    bool isInstant = (archetype == 7);
    float s = max(clusterSize, 0.25);
    float procCommon = evalGrainProcNoise(p, s, isInstant);
    float etaCommon = evalGrainEta(procCommon, sensorSeed, grainMode, isInstant);

    // 3. Nutting midtone density weighting: G(Y) = 4 * Y * (1 - Y)
    float yClamped = clamp(y_c, 0.0, 1.0);
    float nutting = 4.0 * yClamped * (1.0 - yClamped);

    // 4. Optical transmittance modulation
    float yPixel = dot(color, vec3(0.2627, 0.6780, 0.0593));

    bool isMonochrome = (archetype == 6 || archetype == 10 || archetype == 11);
    if (isMonochrome) {
        // Metallic Silver: Pure achromatic optical density modulation (R = G = B)
        float delta = etaCommon * nutting * 0.28 * amount;
        float yRes = max(yPixel * (1.0 + delta), 0.0);
        return vec3(yRes);
    } else {
        // Subtractive 3-Layer Dye Cloud Decorrelation (Cyan -> Red, Magenta -> Green, Yellow -> Blue)
        float procR = evalGrainProcNoise(p + vec2(17.3, 31.7), s, isInstant);
        float procG = evalGrainProcNoise(p + vec2(73.1, 19.9), s, isInstant);
        float procB = evalGrainProcNoise(p + vec2(43.7, 89.3), s, isInstant);

        float etaR = evalGrainEta(procR, sensorSeed, grainMode, isInstant);
        float etaG = evalGrainEta(procG, sensorSeed, grainMode, isInstant);
        float etaB = evalGrainEta(procB, sensorSeed, grainMode, isInstant);

        // Mix 75% common photopic luminance carrier with 25% independent subtractive layer phase
        // Energy-preserving Euclidean normalization: 1.0 / sqrt(0.75^2 + 0.25^2) ≈ 1.264911
        float beta = 0.25;
        float norm = 1.264911;
        float etaFinalR = mix(etaCommon, etaR, beta) * norm;
        float etaFinalG = mix(etaCommon, etaG, beta) * norm;
        float etaFinalB = mix(etaCommon, etaB, beta) * norm;

        float scale = nutting * 0.28 * amount;
        float deltaR = etaFinalR * scale;
        float deltaG = etaFinalG * scale;
        float deltaB = etaFinalB * scale;

        float chromaDamp = 1.0 - 0.10 * nutting;
        float rC = yPixel + chromaDamp * (color.r - yPixel);
        float gC = yPixel + chromaDamp * (color.g - yPixel);
        float bC = yPixel + chromaDamp * (color.b - yPixel);

        float rRes = max(rC * (1.0 + deltaR), 0.0);
        float gRes = max(gC * (1.0 + deltaG), 0.0);
        float bRes = max(bC * (1.0 + deltaB), 0.0);

        return vec3(rRes, gRes, bRes);
    }
}
`;
