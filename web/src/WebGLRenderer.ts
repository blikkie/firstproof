import { EditSettingsPayload, EvaluatedLut } from './WasmBridge';

const VERTEX_SHADER_SRC = `#version 300 es
in vec2 a_position;
in vec2 a_texCoord;

uniform vec2 u_fitScale;
uniform float u_zoomScale;
uniform vec2 u_panOffset;

out vec2 v_texCoord;

void main() {
    vec2 pos = a_position * u_fitScale * u_zoomScale + u_panOffset;
    gl_Position = vec4(pos, 0.0, 1.0);
    v_texCoord = a_texCoord;
}
`;

const FRAGMENT_SHADER_SRC = `#version 300 es
precision highp float;

in vec2 v_texCoord;
out vec4 fragColor;

uniform sampler2D u_texture;
uniform sampler2D u_curveMasterLut;
uniform sampler2D u_curveRgbLut;

// Edit Settings Uniforms
uniform float u_exposure;
uniform float u_contrast;
uniform float u_highlights;
uniform float u_shadows;
uniform float u_whites;
uniform float u_blacks;
uniform float u_vibrance;
uniform float u_saturation;
uniform float u_deepBlue;
uniform float u_foliage;
uniform float u_temperature;
uniform float u_tint;
uniform int u_toneProfileMode; // 0: AgxVivid, 1: AgxFilmic, 2: CamStandard, 3: CamPortrait, 4: CamLandscape, 5: CamVivid
uniform int u_emulsionArchetype; // 0: None, 1: Portra 400, 2: Velvia 50, 3: Kodachrome 64, 4: Vision3 500T, 5: Gold 200, 6: HP5, 7: Polacolor
uniform float u_emulsionExposure;
uniform int u_printMediaMode; // 0: None, 1: RA4, 2: SilverDye, 3: CottonRag, 4: SilverGelatin, 5: Selenium, 6: Platinum, 7: Cyanotype
uniform float u_printPaperGrade;
uniform float u_printBaseWarmth;
uniform float u_printProcessDepth;
uniform int u_enlargerFilterMode;
uniform float u_splitShadowHue;
uniform float u_splitShadowSat;
uniform float u_splitHighlightHue;
uniform float u_splitHighlightSat;
uniform float u_splitBalance;
uniform float u_cropLeft;
uniform float u_cropTop;
uniform float u_cropRight;
uniform float u_cropBottom;
uniform int u_cropModeActive;

uniform bool u_isOriginal;
uniform bool u_isLinearRec2020;
uniform bool u_hasMasterCurve;
uniform bool u_hasRgbCurves;

// Matrices (Column-major in GLSL)
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
     1.4096986,   0.13136263,  0.16151477,
    -0.4004796,   0.77151406, -0.06034984,
    -0.009218898, 0.09712328,  0.89883506
);

const mat3 AGX_OUTSET_MATRIX = mat3(
     1.1271005, -0.1413298, -0.1413298,
    -0.1106066,  1.1578237, -0.1106066,
    -0.0164939, -0.0164939,  1.2519364
);

const float AGX_MIN_EV = -15.2;
const float AGX_MAX_EV = 5.0;
const float AGX_EV_RANGE = 20.2;
const float AGX_GAMMA = 2.4;
const float AGX_MID_GRAY_NORM = 0.75247525;

// Gamma Transfer Functions
float srgbGammaToLinear(float c) {
    c = clamp(c, 0.0, 1.0);
    return (c <= 0.04045) ? (c / 12.92) : pow((c + 0.055) / 1.055, 2.4);
}

vec3 srgbGammaToLinear3(vec3 c) {
    return vec3(srgbGammaToLinear(c.r), srgbGammaToLinear(c.g), srgbGammaToLinear(c.b));
}

float linearToSrgbGamma(float c) {
    c = clamp(c, 0.0, 1.0);
    return (c <= 0.0031308) ? (c * 12.92) : (1.055 * pow(c, 1.0 / 2.4) - 0.055);
}

vec3 linearToSrgbGamma3(vec3 c) {
    return vec3(linearToSrgbGamma(c.r), linearToSrgbGamma(c.g), linearToSrgbGamma(c.b));
}

// Color Space Conversion HSV
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)) * 360.0, d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    float h = c.x / 60.0;
    float s = c.y;
    float v = c.z;
    float C = v * s;
    float X = C * (1.0 - abs(mod(h, 2.0) - 1.0));
    float m = v - C;
    vec3 rgb;
    if (h < 1.0) rgb = vec3(C, X, 0.0);
    else if (h < 2.0) rgb = vec3(X, C, 0.0);
    else if (h < 3.0) rgb = vec3(0.0, C, X);
    else if (h < 4.0) rgb = vec3(0.0, X, C);
    else if (h < 5.0) rgb = vec3(X, 0.0, C);
    else rgb = vec3(C, 0.0, X);
    return rgb + vec3(m);
}

// AgX Filmic Sigmoids
float agxSigmoid(float x, float power) {
    return x / pow(1.0 + pow(x, power), 1.0 / power);
}

float agxScaledSigmoid(float x, float scale, float slope, float power, float tx, float ty) {
    return scale * agxSigmoid(slope * (x - tx) / scale, power) + ty;
}

float agxApplyCurveChannel(float x) {
    if (x < 0.6060606) {
        return clamp(agxScaledSigmoid(x, -1.0359, 2.3843, 1.5, 0.6060606, 0.43446), 0.0, 1.0);
    } else {
        return clamp(agxScaledSigmoid(x, 1.3475, 2.3843, 1.5, 0.6060606, 0.43446), 0.0, 1.0);
    }
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
    vec3 base = agxToneMap(rec2020Color, contrastGain * 1.15);
    vec3 hsv = rgb2hsv(base);
    
    float skinWeight = clamp(1.0 - abs(hsv.x - 25.0) / 20.0, 0.0, 1.0);
    float skyWeight = clamp(1.0 - abs(hsv.x - 235.0) / 35.0, 0.0, 1.0);
    
    hsv.x -= 18.0 * skyWeight;
    hsv.x -= 3.0 * skinWeight;
    if (hsv.x < 0.0) { hsv.x += 360.0; }
    if (hsv.x >= 360.0) { hsv.x -= 360.0; }
    
    hsv.y = clamp(hsv.y * (1.18 - 0.20 * skinWeight - 0.15 * skyWeight), 0.0, 1.0);
    return clamp(hsv2rgb(hsv), 0.0, 1.0);
}

// Tone Recovery
vec3 applyToneRecovery(vec3 color, float highlights, float shadows, float whites, float blacks) {
    if (highlights == 0.0 && shadows == 0.0 && whites == 0.0 && blacks == 0.0) {
        return color;
    }
    float y_c = dot(color, vec3(0.2126, 0.7152, 0.0722));
    if (y_c <= 1e-6) return color;
    
    float ev_base = log2(max(y_c, 1e-5) / 0.18);
    float delta_ev = 0.0;
    
    if (blacks != 0.0) {
        float u = clamp((-ev_base - 1.0) / 3.0, 0.0, 1.0);
        float w_blacks = u * u * (3.0 - 2.0 * u);
        delta_ev += (blacks / 100.0) * 0.90 * w_blacks;
    }
    if (shadows != 0.0) {
        float u = clamp((ev_base + 4.0) / 4.0, 0.0, 1.0);
        float w_shadows = sin(3.14159265 * u);
        delta_ev += (shadows / 100.0) * 1.20 * w_shadows;
    }
    if (highlights != 0.0) {
        float u = clamp(ev_base / 4.0, 0.0, 1.0);
        float w_highlights = sin(3.14159265 * u);
        delta_ev += (highlights / 100.0) * 1.30 * w_highlights;
    }
    if (whites != 0.0) {
        float u = clamp((ev_base - 1.0) / 3.0, 0.0, 1.0);
        float w_whites = u * u * (3.0 - 2.0 * u);
        delta_ev += (whites / 100.0) * 0.90 * w_whites;
    }
    
    return color * exp2(delta_ev);
}

// Color Adjustments
vec3 applyColorAdjustments(vec3 rgb, float vibrance, float saturation, float deepBlue, float foliage) {
    if (vibrance == 0.0 && saturation == 0.0 && deepBlue == 0.0 && foliage == 0.0) {
        return rgb;
    }
    vec3 c = clamp(rgb, 0.0, 1.0);
    float lum = dot(c, vec3(0.2126, 0.7152, 0.0722));

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

        c += (c - vec3(lum)) * vibGain;
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

// Split Toning
vec3 applySplitToning(vec3 rgb, float shadowHue, float shadowSat, float highlightHue, float highlightSat, float balance) {
    if (shadowSat <= 0.0 && highlightSat <= 0.0) return rgb;

    float lum = dot(rgb, vec3(0.2126, 0.7152, 0.0722));
    float yClamp = clamp(lum, 0.0, 1.0);
    float t = clamp(0.5 - (balance / 100.0) * 0.25, 0.1, 0.9);
    vec3 delta = vec3(0.0);

    if (shadowSat > 0.0 && yClamp < t) {
        float u_s = (t - yClamp) / t;
        float w_s = u_s * u_s * (3.0 - 2.0 * u_s) * (shadowSat / 100.0);
        vec3 tintRgb = hsv2rgb(vec3(shadowHue, 1.0, 1.0));
        float tLum = dot(tintRgb, vec3(0.2126, 0.7152, 0.0722));
        delta += w_s * (tintRgb - vec3(tLum));
    }

    if (highlightSat > 0.0 && yClamp > t) {
        float u_h = (yClamp - t) / (1.0 - t);
        float w_h = u_h * u_h * (3.0 - 2.0 * u_h) * (highlightSat / 100.0);
        vec3 tintRgb = hsv2rgb(vec3(highlightHue, 1.0, 1.0));
        float tLum = dot(tintRgb, vec3(0.2126, 0.7152, 0.0722));
        delta += w_h * (tintRgb - vec3(tLum));
    }

    return clamp(rgb + delta, 0.0, 1.0);
}

// Sensitometric Characteristic Curve
float applyFilmCharacteristicCurve(float val, float gamma, float shoulder, float toe) {
    float v = clamp(val, 0.0, 2.0);
    float base = v;
    if (abs(gamma - 1.0) > 0.001) {
        float u = clamp(v / (v + 0.35), 0.0, 1.0);
        float g_mod = (gamma - 1.0) * 0.40;
        base = max(v + g_mod * u * (1.0 - u) * (2.0 * u - 1.0), 0.0);
    }
    float after_shoulder = base;
    if (shoulder > 0.0 && base > 0.65) {
        float t = clamp((base - 0.65) / 0.35, 0.0, 1.0);
        float comp = shoulder * 0.12 * t * t * (3.0 - 2.0 * t);
        after_shoulder = base - comp;
    }
    if (toe > 0.0 && after_shoulder < 0.35) {
        float t = clamp(after_shoulder / 0.35, 0.0, 1.0);
        float comp = toe * 0.06 * (1.0 - t) * (1.0 - t);
        return max(after_shoulder - comp, 0.0);
    }
    return after_shoulder;
}

// Active DIR Chemical Crosstalk
vec3 applyDirChemicalCoupling(vec3 rgb, float c_rg, float c_rb, float c_gr, float c_gb, float c_br, float c_bg) {
    float eps = 1e-4;
    float d_r = -log2(max(rgb.r, eps)) / 3.321928;
    float d_g = -log2(max(rgb.g, eps)) / 3.321928;
    float d_b = -log2(max(rgb.b, eps)) / 3.321928;

    float c_rr = 1.0 - c_rg - c_rb;
    float c_gg = 1.0 - c_gr - c_gb;
    float c_bb = 1.0 - c_br - c_bg;

    float d_r_out = c_rr * d_r + c_rg * d_g + c_rb * d_b;
    float d_g_out = c_gr * d_r + c_gg * d_g + c_gb * d_b;
    float d_b_out = c_br * d_r + c_bg * d_g + c_bb * d_b;

    return vec3(
        clamp(pow(10.0, -d_r_out), 0.0, 2.0),
        clamp(pow(10.0, -d_g_out), 0.0, 2.0),
        clamp(pow(10.0, -d_b_out), 0.0, 2.0)
    );
}

// Photochemical Recording Emulsion
vec3 applyRecordingEmulsion(vec3 rgb, int archetype, float pushPullEv) {
    if (archetype == 0) return rgb;

    float expGain = pow(2.0, clamp(pushPullEv, -2.0, 2.0));
    vec3 cur = max(rgb * expGain, vec3(0.0));

    if (archetype == 1) { // Rochester Portrait 400
        vec3 dir = applyDirChemicalCoupling(cur, -0.04, 0.02, -0.08, 0.02, 0.02, -0.05);
        return vec3(
            applyFilmCharacteristicCurve(dir.r, 0.90, 0.38, 0.08),
            applyFilmCharacteristicCurve(dir.g, 0.90, 0.38, 0.08),
            applyFilmCharacteristicCurve(dir.b, 0.90, 0.38, 0.08)
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
    } else if (archetype == 5) { // Rochester Daylight 200
        vec3 dir = applyDirChemicalCoupling(cur, -0.06, 0.03, -0.07, 0.02, 0.08, -0.10);
        return vec3(
            applyFilmCharacteristicCurve(dir.r, 1.12, 0.20, 0.12),
            applyFilmCharacteristicCurve(dir.g, 1.08, 0.22, 0.12),
            applyFilmCharacteristicCurve(dir.b, 0.98, 0.26, 0.16)
        );
    } else if (archetype == 6) { // Mobberley Silver 400 (B&W)
        float panLum = 0.28 * cur.r + 0.62 * cur.g + 0.10 * cur.b;
        float mono = applyFilmCharacteristicCurve(panLum, 1.08, 0.30, 0.25);
        return vec3(mono);
    } else if (archetype == 7) { // Cambridge Instant Diffusion 100
        vec3 dir = applyDirChemicalCoupling(cur, -0.03, 0.02, -0.04, 0.03, 0.01, -0.03);
        vec3 curv = vec3(
            applyFilmCharacteristicCurve(dir.r, 0.98, 0.35, 0.12),
            applyFilmCharacteristicCurve(dir.g, 0.98, 0.35, 0.12),
            applyFilmCharacteristicCurve(dir.b, 0.98, 0.35, 0.12)
        );
        float lum = dot(curv, vec3(0.2126, 0.7152, 0.0722));
        float baseRefl = 0.025 * max(1.0 - lum, 0.0);
        return vec3(
            min(curv.r + baseRefl * 1.02, 2.0),
            min(curv.g + baseRefl * 1.00, 2.0),
            min(curv.b + baseRefl * 0.94, 2.0)
        );
    }
    return cur;
}

// Paper Grade S-Curve
float applyPaperGrade(float val, float grade) {
    if (abs(grade) < 0.001) return val;
    float alpha = 0.40 * (grade / 2.0);
    return clamp(val + alpha * val * (1.0 - val) * (2.0 * val - 1.0), 0.0, 1.0);
}

// Paper Base Warmth
vec3 applyBaseWarmth(vec3 rgb, float warmth) {
    if (abs(warmth) < 0.001) return rgb;
    float lum = dot(rgb, vec3(0.2126, 0.7152, 0.0722));
    float factor = (warmth / 100.0) * clamp(pow(lum, 0.6), 0.0, 1.0);
    vec3 mult = (factor > 0.0) ? vec3(1.0 + factor * 0.060, 1.0 + factor * 0.025, 1.0 - factor * 0.045)
                               : vec3(1.0 + factor * 0.025, 1.0 + factor * 0.008, 1.0 - factor * 0.045);
    return clamp(rgb * mult, 0.0, 1.0);
}

// Camera manufacturer S-curve matching ColorScience.metal & export pipeline
float applyCameraToneCurve(float x, int mode) {
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

// Camera HueSatMap Memory Color Adjustments matching ColorScience.metal
vec3 applyCameraHueSat(vec3 hsv, int mode) {
    float h = hsv.x;
    float s = hsv.y;
    float v = hsv.z;
    
    float skyWeight = clamp(1.0 - abs(h - 235.0) / 35.0, 0.0, 1.0);
    float greenWeight = clamp(1.0 - abs(h - 120.0) / 35.0, 0.0, 1.0);
    float skinWeight = clamp(1.0 - abs(h - 30.0) / 20.0, 0.0, 1.0);
    
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
    return vec3(h, clamp(s, 0.0, 1.0), clamp(v, 0.0, 1.0));
}

// Enlarger Optical Filtration
vec3 getEnlargerFilterWeights(int filterMode) {
    if (filterMode == 1) return vec3(0.35, 0.60, 0.05); // YellowK2
    if (filterMode == 2) return vec3(0.55, 0.42, 0.03); // Orange16
    if (filterMode == 3) return vec3(0.85, 0.15, 0.00); // Red25A
    if (filterMode == 4) return vec3(0.15, 0.80, 0.05); // GreenX1
    if (filterMode == 5) return vec3(0.00, 0.60, 0.40); // Orthochromatic
    return vec3(0.2627, 0.6780, 0.0593); // NeutralPanchromatic
}

// Fine-Art Print Media
vec3 applyPrintMedia(vec3 rgb, int mediaMode, int filterMode, float paperGrade, float baseWarmth, float processDepth) {
    if (mediaMode == 0) return rgb;
    float depth = clamp(processDepth / 100.0, 0.0, 1.0);

    if (mediaMode == 1) { // ChromogenicTypeC (RA-4)
        float rG = applyPaperGrade(rgb.r, paperGrade);
        float gG = applyPaperGrade(rgb.g, paperGrade);
        float bG = applyPaperGrade(rgb.b, paperGrade);

        float rP = rG;
        if (rG > 0.70) {
            float u = clamp((rG - 0.70) / 0.30, 0.0, 1.0);
            rP = rG - 0.045 * depth * u * u * (3.0 - 2.0 * u);
        }
        float gP = gG;
        if (gG > 0.70) {
            float u = clamp((gG - 0.70) / 0.30, 0.0, 1.0);
            gP = gG - 0.045 * depth * u * u * (3.0 - 2.0 * u);
        }
        float bP = bG;
        if (bG > 0.70) {
            float u = clamp((bG - 0.70) / 0.30, 0.0, 1.0);
            bP = bG - 0.045 * depth * u * u * (3.0 - 2.0 * u);
        }

        float lum = 0.2126 * rP + 0.7152 * gP + 0.0722 * bP;
        float satScale = 0.70 + 0.35 * depth;
        float rOut = clamp(lum + (rP - lum) * satScale, 0.0, 1.0);
        float gOut = clamp(lum + (gP - lum) * satScale, 0.0, 1.0);
        float bOut = clamp(lum + (bP - lum) * satScale, 0.0, 1.0);

        float dyeDensity = 0.04 * max(1.0 - lum, 0.0) * depth;
        rOut = clamp(rOut * (1.0 - dyeDensity * 0.4), 0.0, 1.0);
        gOut = clamp(gOut * (1.0 - dyeDensity * 0.6), 0.0, 1.0);
        bOut = clamp(bOut * (1.0 - dyeDensity * 0.9), 0.0, 1.0);

        return applyBaseWarmth(vec3(rOut, gOut, bOut), baseWarmth);
    } else if (mediaMode == 2) { // SilverDyeBleach (Azo Reversal)
        float rG = applyPaperGrade(rgb.r, paperGrade);
        float gG = applyPaperGrade(rgb.g, paperGrade);
        float bG = applyPaperGrade(rgb.b, paperGrade);

        float shadowPow = 1.0 + 0.12 * depth;
        float rP = pow(max(rG, 0.0), shadowPow);
        float gP = pow(max(gG, 0.0), shadowPow);
        float bP = pow(max(bG, 0.0), shadowPow);

        float lum = 0.2126 * rP + 0.7152 * gP + 0.0722 * bP;
        float azoSat = 0.80 + 0.30 * depth;
        float rOut = clamp(lum + (rP - lum) * azoSat, 0.0, 1.0);
        float gOut = clamp(lum + (gP - lum) * azoSat, 0.0, 1.0);
        float bOut = clamp(lum + (bP - lum) * azoSat, 0.0, 1.0);

        return applyBaseWarmth(vec3(rOut, gOut, bOut), baseWarmth);
    } else if (mediaMode == 3) { // FineArtCottonRag
        float rG = applyPaperGrade(rgb.r, paperGrade);
        float gG = applyPaperGrade(rgb.g, paperGrade);
        float bG = applyPaperGrade(rgb.b, paperGrade);

        float matteFloor = 0.055 - 0.015 * depth;
        float rP = rG + matteFloor * (1.0 - rG) * (1.0 - rG);
        float gP = gG + matteFloor * (1.0 - gG) * (1.0 - gG);
        float bP = bG + matteFloor * (1.0 - bG) * (1.0 - bG);

        float lum = 0.2126 * rP + 0.7152 * gP + 0.0722 * bP;
        float pigmentSat = 0.75 + 0.25 * depth;
        float rOut = clamp(lum + (rP - lum) * pigmentSat, 0.0, 1.0);
        float gOut = clamp(lum + (gP - lum) * pigmentSat, 0.0, 1.0);
        float bOut = clamp(lum + (bP - lum) * pigmentSat, 0.0, 1.0);

        return applyBaseWarmth(vec3(rOut, gOut, bOut), baseWarmth + 15.0);
    } else if (mediaMode == 4) { // BarytaSilverGelatin
        vec3 w = getEnlargerFilterWeights(filterMode);
        float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
        float silver = applyPaperGrade(rawLum, paperGrade);

        float dmaxGamma = 1.0 + 0.08 * depth;
        float silverToned = pow(max(silver, 0.0), dmaxGamma);

        return applyBaseWarmth(vec3(silverToned), baseWarmth);
    } else if (mediaMode == 5) { // WarmtoneChlorobromide
        vec3 w = getEnlargerFilterWeights(filterMode);
        float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
        float silver = applyPaperGrade(rawLum, paperGrade);

        float shadowPow = 1.0 + 0.10 * depth;
        float silverDeep = pow(max(silver, 0.0), shadowPow);

        float midMask = 4.0 * silver * (1.0 - silver);
        float shadowDensity = max(1.0 - silver, 0.0);

        float rTone = silverDeep + depth * (midMask * 0.030 + shadowDensity * 0.020);
        float gTone = silverDeep + depth * (midMask * 0.012 - shadowDensity * 0.015);
        float bTone = silverDeep + depth * (-midMask * 0.018 + shadowDensity * 0.030);

        vec3 toned = clamp(vec3(rTone, gTone, bTone), 0.0, 1.0);
        return applyBaseWarmth(toned, baseWarmth);
    } else if (mediaMode == 6) { // PlatinumPalladium
        vec3 w = getEnlargerFilterWeights(filterMode);
        float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
        float silver = applyPaperGrade(rawLum, paperGrade);

        float matteFloor = 0.035 * depth;
        float silverBase = matteFloor + (1.0 - matteFloor) * silver;

        float warmLift = max(1.0 - pow(max(silverBase, 0.0), 0.6), 0.0) * depth;
        float rTone = silverBase + warmLift * 0.045;
        float gTone = silverBase + warmLift * 0.028;
        float bTone = silverBase - warmLift * 0.020;

        vec3 toned = clamp(vec3(rTone, gTone, bTone), 0.0, 1.0);
        return applyBaseWarmth(toned, baseWarmth + 10.0 * depth);
    } else if (mediaMode == 7) { // Cyanotype
        vec3 w = getEnlargerFilterWeights(filterMode);
        float rawLum = clamp(dot(rgb, w), 0.0, 1.0);
        float silver = applyPaperGrade(rawLum, paperGrade);

        float density = 1.0 - silver;
        float rP = silver * (1.0 - 0.75 * depth * density);
        float gP = silver * (1.0 - 0.35 * depth * density) + 0.02 * depth * density;
        float bP = silver * (1.0 - 0.05 * depth * density) + 0.07 * depth * density;

        vec3 toned = clamp(vec3(rP, gP, bP), 0.0, 1.0);
        return applyBaseWarmth(toned, baseWarmth + 8.0 * depth);
    }
    return rgb;
}

// Tone Curve Coordinate Mapping (Half-texel offset for exact 256-sample 1D LUT)
vec2 sampleCurveCoord(float val) {
    float u = (clamp(val, 0.0, 1.0) * 255.0 + 0.5) / 256.0;
    return vec2(u, 0.5);
}

void main() {
    vec4 sampled = texture(u_texture, v_texCoord);
    
    // Quick peek original as-shot frame
    if (u_isOriginal) {
        if (u_isLinearRec2020) {
            vec3 srgbLinearOut = agxToneMap(sampled.rgb, 1.0);
            fragColor = vec4(linearToSrgbGamma3(clamp(srgbLinearOut, 0.0, 1.0)), 1.0);
        } else {
            fragColor = sampled;
        }
        return;
    }
    
    // Neutral JPEG Passthrough: Camera JPEGs are already tone-mapped and sRGB encoded.
    // When tone profile is default AgX Vivid and sliders are unadjusted, bypass AgX double-compression to maintain authentic camera punch.
    bool isNeutralEdits = (u_toneProfileMode == 0) &&
                          (u_exposure == 0.0) &&
                          (u_contrast == 0.0) &&
                          (u_highlights == 0.0) &&
                          (u_shadows == 0.0) &&
                          (u_whites == 0.0) &&
                          (u_blacks == 0.0) &&
                          (abs(u_temperature - 5500.0) < 1.0) &&
                          (u_tint == 0.0) &&
                          (u_vibrance == 0.0) &&
                          (u_saturation == 0.0) &&
                          (u_deepBlue == 0.0) &&
                          (u_foliage == 0.0) &&
                          (u_splitShadowSat == 0.0) &&
                          (u_splitHighlightSat == 0.0) &&
                          (!u_hasMasterCurve) &&
                          (!u_hasRgbCurves) &&
                          (u_emulsionArchetype == 0) &&
                          (u_printMediaMode == 0) &&
                          (u_cropLeft <= 0.001) &&
                          (u_cropTop <= 0.001) &&
                          (u_cropRight >= 0.999) &&
                          (u_cropBottom >= 0.999);

    if (!u_isLinearRec2020 && isNeutralEdits) {
        fragColor = sampled;
        return;
    }

    // 1. Scene-Referred Working Space
    vec3 rec2020Color;
    if (u_isLinearRec2020) {
        rec2020Color = sampled.rgb;
    } else {
        vec3 srgbLinear = srgbGammaToLinear3(sampled.rgb);
        rec2020Color = LINEAR_SRGB_TO_REC2020 * srgbLinear;
    }
    
    // 2. White Balance Bradford CAT02 scaling
    if (abs(u_temperature - 5500.0) >= 1.0 || u_tint != 0.0) {
        float tempNorm = clamp(u_temperature / 5500.0, 0.25, 4.0);
        float rScale = pow(tempNorm, 0.75);
        float bScale = pow(1.0 / tempNorm, 0.75);
        float tintFactor = u_tint * 0.002;
        float gScale = max(1.0 - tintFactor, 0.01);
        rScale = max(rScale + tintFactor * 0.5, 0.01);
        bScale = max(bScale + tintFactor * 0.5, 0.01);
        rec2020Color *= vec3(rScale, gScale, bScale);
    }
    
    // 3. Linear Exposure Multiplication
    if (u_exposure != 0.0) {
        rec2020Color *= pow(2.0, u_exposure);
    }
    
    // 4. Photochemical Recording Emulsion
    if (u_emulsionArchetype != 0) {
        rec2020Color = applyRecordingEmulsion(rec2020Color, u_emulsionArchetype, u_emulsionExposure);
    }

    // 5. Tone Recovery
    rec2020Color = applyToneRecovery(rec2020Color, u_highlights, u_shadows, u_whites, u_blacks);
    
    // 6. AgX Filmic Tone Mapping Engine OR Camera-Matching Profiles
    float contrastGain = (u_contrast >= 0.0) ? (1.0 + u_contrast / 100.0) : (1.0 + (u_contrast / 100.0) * 0.60);
    vec3 srgbLinearOut;
    if (u_toneProfileMode == 0) {
        // Mode 0: AgX Vivid (Filmic Punchy) - DEFAULT
        srgbLinearOut = agxToneMapVivid(rec2020Color, contrastGain);
    } else if (u_toneProfileMode == 1) {
        // Mode 1: AgX Neutral (Filmic Base)
        srgbLinearOut = agxToneMap(rec2020Color, contrastGain);
    } else {
        // Modes 2..5: Camera Matching (2: Standard, 3: Portrait, 4: Landscape, 5: Vivid)
        int camMode = u_toneProfileMode - 1;
        vec3 c_srgb = max(REC2020_TO_LINEAR_SRGB * rec2020Color, vec3(0.0));
        vec3 hsv = rgb2hsv(c_srgb);
        vec3 hsvAdjusted = applyCameraHueSat(hsv, camMode);
        vec3 rgbAdjusted = hsv2rgb(hsvAdjusted);
        
        float lum = dot(rgbAdjusted, vec3(0.2126, 0.7152, 0.0722));
        float lumCurved = applyCameraToneCurve(lum, camMode);
        float lumAdj = clamp((lumCurved - 0.18) * contrastGain + 0.18, 0.0, 1.0);
        float gain = lumAdj / max(lum, 1e-6);
        
        srgbLinearOut = clamp(rgbAdjusted * gain, 0.0, 1.0);
    }
    
    vec3 finalColor = srgbLinearOut;
    
    // 7. Color & Vibrance
    finalColor = applyColorAdjustments(finalColor, u_vibrance, u_saturation, u_deepBlue, u_foliage);
    
    // 8. Split Toning
    finalColor = applySplitToning(finalColor, u_splitShadowHue, u_splitShadowSat, u_splitHighlightHue, u_splitHighlightSat, u_splitBalance);
    
    // 9. RGB Tone Curves (Master + RGB 1D Texture LUT Sampler)
    if (u_hasMasterCurve) {
        finalColor.r = texture(u_curveMasterLut, sampleCurveCoord(finalColor.r)).r;
        finalColor.g = texture(u_curveMasterLut, sampleCurveCoord(finalColor.g)).r;
        finalColor.b = texture(u_curveMasterLut, sampleCurveCoord(finalColor.b)).r;
    }
    if (u_hasRgbCurves) {
        finalColor.r = texture(u_curveRgbLut, sampleCurveCoord(finalColor.r)).r;
        finalColor.g = texture(u_curveRgbLut, sampleCurveCoord(finalColor.g)).g;
        finalColor.b = texture(u_curveRgbLut, sampleCurveCoord(finalColor.b)).b;
    }

    // 10. Fine-Art Print Media Chemistry
    if (u_printMediaMode != 0) {
        finalColor = applyPrintMedia(finalColor, u_printMediaMode, u_enlargerFilterMode, u_printPaperGrade, u_printBaseWarmth, u_printProcessDepth);
    }
    
    // Match Shaders.metal lines 360-366: AgX (modes 0 & 1) is already display-referred.
    // Camera Matching profiles (modes 2..5) output linear sRGB and require display gamma.
    if (u_toneProfileMode <= 1) {
        fragColor = vec4(clamp(finalColor, 0.0, 1.0), 1.0);
    } else {
        fragColor = vec4(linearToSrgbGamma3(clamp(finalColor, 0.0, 1.0)), 1.0);
    }
}
`;

const EMULSION_MAP: Record<string, number> = {
  None: 0,
  RochesterPortrait400: 1,
  AshigaraChrome50: 2,
  RochesterK14Chrome64: 3,
  RochesterCinema500: 4,
  RochesterDaylight200: 5,
  MobberleySilver400: 6,
  CambridgeDiffusion100: 7,
};

const PRINT_MEDIA_MAP: Record<string, number> = {
  None: 0,
  ChromogenicTypeC: 1,
  SilverDyeBleach: 2,
  FineArtCottonRag: 3,
  BarytaSilverGelatin: 4,
  WarmtoneChlorobromide: 5,
  PlatinumPalladium: 6,
  Cyanotype: 7,
  PrussianCyanotype: 7,
};

export class WebGLRenderer {
  private gl: WebGL2RenderingContext;
  private program: WebGLProgram;
  private texture: WebGLTexture | null = null;
  private masterLutTex: WebGLTexture | null = null;
  private rgbLutTex: WebGLTexture | null = null;
  private isLinearRec2020: boolean = false;
  private vbo: WebGLBuffer | null = null;
  private currentRotation: number = 0;
  private currentFlipH: boolean = false;
  private currentFlipV: boolean = false;
  private currentCrop: { left: number; top: number; right: number; bottom: number } = { left: 0, top: 0, right: 1, bottom: 1 };
  private currentCropModeActive: boolean = false;
  private texWidth: number = 1920;
  private texHeight: number = 1080;

  // Uniform locations
  private uLocs: Record<string, WebGLUniformLocation | null> = {};

  constructor(canvas: HTMLCanvasElement) {
    const gl = canvas.getContext('webgl2', { alpha: false, antialias: true, preserveDrawingBuffer: true });
    if (!gl) {
      throw new Error('WebGL 2 is not supported by your browser');
    }
    this.gl = gl;
    this.program = this.initProgram();
    this.initBuffers();
    this.initLutTextures();
  }

  private createShader(type: number, src: string): WebGLShader {
    const shader = this.gl.createShader(type)!;
    this.gl.shaderSource(shader, src);
    this.gl.compileShader(shader);
    if (!this.gl.getShaderParameter(shader, this.gl.COMPILE_STATUS)) {
      const info = this.gl.getShaderInfoLog(shader);
      this.gl.deleteShader(shader);
      throw new Error(`Shader compile failure: ${info}`);
    }
    return shader;
  }

  private initProgram(): WebGLProgram {
    const vert = this.createShader(this.gl.VERTEX_SHADER, VERTEX_SHADER_SRC);
    const frag = this.createShader(this.gl.FRAGMENT_SHADER, FRAGMENT_SHADER_SRC);
    const prog = this.gl.createProgram()!;
    this.gl.attachShader(prog, vert);
    this.gl.attachShader(prog, frag);
    this.gl.linkProgram(prog);
    if (!this.gl.getProgramParameter(prog, this.gl.LINK_STATUS)) {
      throw new Error(`Program link error: ${this.gl.getProgramInfoLog(prog)}`);
    }
    this.gl.useProgram(prog);

    // Cache uniforms
    const uniforms = [
      'u_texture', 'u_curveMasterLut', 'u_curveRgbLut',
      'u_exposure', 'u_contrast', 'u_highlights', 'u_shadows', 'u_whites', 'u_blacks',
      'u_vibrance', 'u_saturation', 'u_deepBlue', 'u_foliage', 'u_temperature', 'u_tint',
      'u_toneProfileMode', 'u_emulsionArchetype', 'u_emulsionExposure',
      'u_printMediaMode', 'u_printPaperGrade', 'u_printBaseWarmth', 'u_printProcessDepth', 'u_enlargerFilterMode',
      'u_splitShadowHue', 'u_splitShadowSat', 'u_splitHighlightHue', 'u_splitHighlightSat', 'u_splitBalance',
      'u_fitScale', 'u_zoomScale', 'u_panOffset', 'u_cropLeft', 'u_cropTop', 'u_cropRight', 'u_cropBottom', 'u_cropModeActive',
      'u_isOriginal', 'u_isLinearRec2020', 'u_hasMasterCurve', 'u_hasRgbCurves'
    ];
    for (const u of uniforms) {
      this.uLocs[u] = this.gl.getUniformLocation(prog, u);
    }

    return prog;
  }

  private initBuffers() {
    const gl = this.gl;
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);

    // Quad geometry: pos(2), uv(2)
    const vertices = new Float32Array([
      -1, -1,  0, 1,
       1, -1,  1, 1,
      -1,  1,  0, 0,
       1,  1,  1, 0,
    ]);

    const vbo = gl.createBuffer();
    this.vbo = vbo;
    gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.DYNAMIC_DRAW);

    const aPos = gl.getAttribLocation(this.program, 'a_position');
    const aTex = gl.getAttribLocation(this.program, 'a_texCoord');

    gl.enableVertexAttribArray(aPos);
    gl.vertexAttribPointer(aPos, 2, gl.FLOAT, false, 16, 0);

    gl.enableVertexAttribArray(aTex);
    gl.vertexAttribPointer(aTex, 2, gl.FLOAT, false, 16, 8);
  }

  public updateQuadGeometry(
    rotation: number = 0,
    flipH: boolean = false,
    flipV: boolean = false,
    crop: { left: number; top: number; right: number; bottom: number } = { left: 0, top: 0, right: 1, bottom: 1 },
    cropModeActive: boolean = false
  ) {
    if (!this.vbo) return;
    const effectiveCrop = cropModeActive ? { left: 0, top: 0, right: 1, bottom: 1 } : crop;
    if (
      this.currentRotation === rotation &&
      this.currentFlipH === flipH &&
      this.currentFlipV === flipV &&
      this.currentCropModeActive === cropModeActive &&
      this.currentCrop.left === effectiveCrop.left &&
      this.currentCrop.top === effectiveCrop.top &&
      this.currentCrop.right === effectiveCrop.right &&
      this.currentCrop.bottom === effectiveCrop.bottom
    ) {
      return;
    }
    this.currentRotation = rotation;
    this.currentFlipH = flipH;
    this.currentFlipV = flipV;
    this.currentCropModeActive = cropModeActive;
    this.currentCrop = { ...effectiveCrop };

    const rot = ((rotation % 360) + 360) % 360;
    const mapScreenToTex = (xs: number, ys: number): [number, number] => {
      switch (rot) {
        case 90: return [ys, 1.0 - xs];
        case 180: return [1.0 - xs, 1.0 - ys];
        case 270: return [1.0 - ys, xs];
        default: return [xs, ys];
      }
    };

    let tl = mapScreenToTex(effectiveCrop.left, effectiveCrop.top);
    let tr = mapScreenToTex(effectiveCrop.right, effectiveCrop.top);
    let bl = mapScreenToTex(effectiveCrop.left, effectiveCrop.bottom);
    let br = mapScreenToTex(effectiveCrop.right, effectiveCrop.bottom);

    if (flipH) {
      const tmpT = tl; tl = tr; tr = tmpT;
      const tmpB = bl; bl = br; br = tmpB;
    }
    if (flipV) {
      const tmpL = tl; tl = bl; bl = tmpL;
      const tmpR = tr; tr = br; br = tmpR;
    }

    // Quad geometry matching TRIANGLE_STRIP: (-1, -1), (1, -1), (-1, 1), (1, 1)
    const vertices = new Float32Array([
      -1, -1,  bl[0], bl[1],
       1, -1,  br[0], br[1],
      -1,  1,  tl[0], tl[1],
       1,  1,  tr[0], tr[1],
    ]);

    const gl = this.gl;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.vbo);
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, vertices);
  }

  private initLutTextures() {
    const gl = this.gl;
    const identityBytes = new Uint8Array(256 * 4);
    for (let i = 0; i < 256; i++) {
      identityBytes[i * 4] = i;
      identityBytes[i * 4 + 1] = i;
      identityBytes[i * 4 + 2] = i;
      identityBytes[i * 4 + 3] = 255;
    }

    this.masterLutTex = gl.createTexture();
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.masterLutTex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 256, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, identityBytes);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);

    this.rgbLutTex = gl.createTexture();
    gl.activeTexture(gl.TEXTURE2);
    gl.bindTexture(gl.TEXTURE_2D, this.rgbLutTex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 256, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, identityBytes);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  }

  public uploadImage(image: HTMLImageElement | HTMLCanvasElement | ImageBitmap | ImageData) {
    this.isLinearRec2020 = false;
    this.texWidth = (image as any).width || 1920;
    this.texHeight = (image as any).height || 1080;
    const gl = this.gl;
    if (!this.texture) {
      this.texture = gl.createTexture();
    }
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);

    if (image instanceof ImageData) {
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, image.width, image.height, 0, gl.RGBA, gl.UNSIGNED_BYTE, image.data);
    } else {
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
    }
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  }

  public uploadLinearRec2020Texture(width: number, height: number, data: Uint16Array) {
    this.isLinearRec2020 = true;
    this.texWidth = width;
    this.texHeight = height;
    const gl = this.gl;
    gl.getExtension('EXT_color_buffer_float');

    if (!this.texture) {
      this.texture = gl.createTexture();
    }
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.texture);
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);

    // Upload 16-bit IEEE 754 Half-Float RGBA16F linear Rec.2020 texture (un-clipped 0.0 .. 4.0+)
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA16F,
      width,
      height,
      0,
      gl.RGBA,
      gl.HALF_FLOAT,
      data
    );
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  }

  public updateToneCurvesLut(lut: EvaluatedLut) {
    this.uploadCurveLuts(lut);
  }

  public uploadCurveLuts(lut: EvaluatedLut) {
    const gl = this.gl;

    // Master LUT
    const masterBytes = new Uint8Array(256 * 4);
    for (let i = 0; i < 256; i++) {
      const val = Math.round(Math.min(Math.max(lut.master[i], 0), 1) * 255);
      masterBytes[i * 4] = val;
      masterBytes[i * 4 + 1] = val;
      masterBytes[i * 4 + 2] = val;
      masterBytes[i * 4 + 3] = 255;
    }
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.masterLutTex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 256, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, masterBytes);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    // RGB LUT
    const rgbBytes = new Uint8Array(256 * 4);
    for (let i = 0; i < 256; i++) {
      rgbBytes[i * 4] = Math.round(Math.min(Math.max(lut.red[i], 0), 1) * 255);
      rgbBytes[i * 4 + 1] = Math.round(Math.min(Math.max(lut.green[i], 0), 1) * 255);
      rgbBytes[i * 4 + 2] = Math.round(Math.min(Math.max(lut.blue[i], 0), 1) * 255);
      rgbBytes[i * 4 + 3] = 255;
    }
    gl.activeTexture(gl.TEXTURE2);
    gl.bindTexture(gl.TEXTURE_2D, this.rgbLutTex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 256, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, rgbBytes);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  }

  public computeFitScale(
    rotation: number = 0,
    crop: { left: number; top: number; right: number; bottom: number } = { left: 0, top: 0, right: 1, bottom: 1 },
    cropModeActive: boolean = false
  ): { fitScaleX: number; fitScaleY: number } {
    const isOrthogonal = (rotation === 90 || rotation === 270);
    const origW = isOrthogonal ? this.texHeight : this.texWidth;
    const origH = isOrthogonal ? this.texWidth : this.texHeight;
    const cropW = cropModeActive ? 1.0 : Math.max(0.01, (crop.right ?? 1.0) - (crop.left ?? 0.0));
    const cropH = cropModeActive ? 1.0 : Math.max(0.01, (crop.bottom ?? 1.0) - (crop.top ?? 0.0));
    const dispW = Math.max(1, origW * cropW);
    const dispH = Math.max(1, origH * cropH);
    const imgAspect = dispW / dispH;

    const viewW = this.gl.canvas.width;
    const viewH = this.gl.canvas.height;
    const viewAspect = (viewW > 0 && viewH > 0) ? (viewW / viewH) : imgAspect;

    let fitScaleX = 1.0;
    let fitScaleY = 1.0;
    if (viewAspect > imgAspect) {
      fitScaleY = 1.0;
      fitScaleX = (viewH * imgAspect) / Math.max(viewW, 1);
    } else {
      fitScaleX = 1.0;
      fitScaleY = (viewW / imgAspect) / Math.max(viewH, 1);
    }
    return { fitScaleX, fitScaleY };
  }

  public render(
    edits: EditSettingsPayload,
    isOriginal: boolean = false,
    zoomScale: number = 1.0,
    panOffset: { x: number; y: number } = { x: 0, y: 0 },
    cropModeActive: boolean = false
  ) {
    const gl = this.gl;
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
    gl.clearColor(0.05, 0.06, 0.08, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.useProgram(this.program);

    this.updateQuadGeometry(
      edits.rotation ?? 0,
      edits.flip_horizontal ?? false,
      edits.flip_vertical ?? false,
      {
        left: edits.crop_left ?? 0.0,
        top: edits.crop_top ?? 0.0,
        right: edits.crop_right ?? 1.0,
        bottom: edits.crop_bottom ?? 1.0,
      },
      cropModeActive
    );

    const { fitScaleX, fitScaleY } = this.computeFitScale(
      edits.rotation ?? 0,
      {
        left: edits.crop_left ?? 0.0,
        top: edits.crop_top ?? 0.0,
        right: edits.crop_right ?? 1.0,
        bottom: edits.crop_bottom ?? 1.0,
      },
      cropModeActive
    );

    // Zoom & Pan & Fit Scale
    gl.uniform2f(this.uLocs['u_fitScale'], fitScaleX, fitScaleY);
    gl.uniform1f(this.uLocs['u_zoomScale'], zoomScale);
    gl.uniform2f(this.uLocs['u_panOffset'], panOffset.x, panOffset.y);

    // Crop Coordinates
    gl.uniform1f(this.uLocs['u_cropLeft'], edits.crop_left ?? 0.0);
    gl.uniform1f(this.uLocs['u_cropTop'], edits.crop_top ?? 0.0);
    gl.uniform1f(this.uLocs['u_cropRight'], edits.crop_right ?? 1.0);
    gl.uniform1f(this.uLocs['u_cropBottom'], edits.crop_bottom ?? 1.0);
    gl.uniform1i(this.uLocs['u_cropModeActive'], cropModeActive ? 1 : 0);

    if (this.texture) {
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, this.texture);
    }
    if (this.masterLutTex) {
      gl.activeTexture(gl.TEXTURE1);
      gl.bindTexture(gl.TEXTURE_2D, this.masterLutTex);
    }
    if (this.rgbLutTex) {
      gl.activeTexture(gl.TEXTURE2);
      gl.bindTexture(gl.TEXTURE_2D, this.rgbLutTex);
    }

    gl.uniform1i(this.uLocs['u_texture'], 0);
    gl.uniform1i(this.uLocs['u_curveMasterLut'], 1);
    gl.uniform1i(this.uLocs['u_curveRgbLut'], 2);

    gl.uniform1f(this.uLocs['u_exposure'], edits.exposure);
    gl.uniform1f(this.uLocs['u_contrast'], edits.contrast);
    gl.uniform1f(this.uLocs['u_highlights'], edits.highlights);
    gl.uniform1f(this.uLocs['u_shadows'], edits.shadows);
    gl.uniform1f(this.uLocs['u_whites'], edits.whites);
    gl.uniform1f(this.uLocs['u_blacks'], edits.blacks);

    gl.uniform1f(this.uLocs['u_vibrance'], edits.vibrance);
    gl.uniform1f(this.uLocs['u_saturation'], edits.saturation);
    gl.uniform1f(this.uLocs['u_deepBlue'], edits.deep_blue);
    gl.uniform1f(this.uLocs['u_foliage'], edits.foliage);
    gl.uniform1f(this.uLocs['u_temperature'], edits.temperature);
    gl.uniform1f(this.uLocs['u_tint'], edits.tint);

    const modeMap: Record<string, number> = {
      AgxVivid: 0, AgxFilmic: 1, CameraStandard: 2, CameraPortrait: 3, CameraLandscape: 4, CameraVivid: 5
    };
    gl.uniform1i(this.uLocs['u_toneProfileMode'], modeMap[edits.tone_profile] ?? 0);
    gl.uniform1i(this.uLocs['u_emulsionArchetype'], EMULSION_MAP[edits.emulsion_archetype] ?? 0);
    gl.uniform1f(this.uLocs['u_emulsionExposure'], edits.emulsion_exposure);

    gl.uniform1i(this.uLocs['u_printMediaMode'], PRINT_MEDIA_MAP[edits.print_media] ?? 0);
    gl.uniform1f(this.uLocs['u_printPaperGrade'], edits.print_paper_grade);
    gl.uniform1f(this.uLocs['u_printBaseWarmth'], edits.print_base_warmth);
    gl.uniform1f(this.uLocs['u_printProcessDepth'], (edits as any).print_process_depth ?? 100.0);
    const filterMap: Record<string, number> = {
      NeutralPanchromatic: 0, FilterGrade2: 0, YellowK2: 1, Orange16: 2, Red25A: 3, GreenX1: 4, Orthochromatic: 5
    };
    gl.uniform1i(this.uLocs['u_enlargerFilterMode'], filterMap[(edits as any).enlarger_filter] ?? 0);

    gl.uniform1f(this.uLocs['u_splitShadowHue'], edits.split_toning_shadow_hue);
    gl.uniform1f(this.uLocs['u_splitShadowSat'], edits.split_toning_shadow_sat);
    gl.uniform1f(this.uLocs['u_splitHighlightHue'], edits.split_toning_highlight_hue);
    gl.uniform1f(this.uLocs['u_splitHighlightSat'], edits.split_toning_highlight_sat);
    gl.uniform1f(this.uLocs['u_splitBalance'], edits.split_toning_balance);

    gl.uniform1i(this.uLocs['u_isOriginal'], isOriginal ? 1 : 0);
    gl.uniform1i(this.uLocs['u_isLinearRec2020'], this.isLinearRec2020 ? 1 : 0);
    gl.uniform1i(this.uLocs['u_hasMasterCurve'], edits.curves.master.length > 2 ? 1 : 0);
    gl.uniform1i(this.uLocs['u_hasRgbCurves'], (edits.curves.red.length > 2 || edits.curves.green.length > 2 || edits.curves.blue.length > 2) ? 1 : 0);

    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
  }

  public getCanvas(): HTMLCanvasElement {
    return this.gl.canvas as HTMLCanvasElement;
  }

  public readPixels(): Uint8Array {
    const gl = this.gl;
    const pixels = new Uint8Array(gl.canvas.width * gl.canvas.height * 4);
    gl.readPixels(0, 0, gl.canvas.width, gl.canvas.height, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    return pixels;
  }

  public static createOffscreen(width: number, height: number): WebGLRenderer {
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    return new WebGLRenderer(canvas);
  }
}
