#include <metal_stdlib>
using namespace metal;

struct VertexInput {
    float2 position [[attribute(0)]];
    float2 texCoord [[attribute(1)]];
};

struct VertexOutput {
    float4 position [[position]];
    float2 texCoord;
};

struct ShaderUniforms {
    float exposure;          // -5.0 .. +5.0 EV
    float temperature;       // 2000K .. 50000K (default 5500K)
    float tint;              // -150 .. +150 (default 0.0)
    float contrast;          // -100 .. +100 (default 0.0)
    float highlights;        // -100 .. +100 (default 0.0)
    float shadows;           // -100 .. +100 (default 0.0)
    float whites;            // -100 .. +100 (default 0.0)
    float blacks;            // -100 .. +100 (default 0.0)
    float vibrance;          // -100 .. +100 (default 0.0)
    float saturation;        // -100 .. +100 (default 0.0)
    float deepBlue;          // -100 .. +100 (default 0.0)
    float foliage;           // -100 .. +100 (default 0.0)
    float straightenAngle;   // Radians
    float aspectRatio;       // width / height
    int lensCorrectionEnabled;
    int distortionModel;     // 0 = poly3, 1 = ptlens
    float k1;
    float k2;
    float k3;
    float k4;
    float isOriginal;        // 1.0 = Bypass edits
    int isLinearRaw;         // 1 = Texture contains Wide-Gamut Linear Rec.2020 half-floats
    int toneProfileMode;     // 0 = AgX Vivid, 1 = AgX Neutral, 2 = Standard, 3 = Portrait, 4 = Landscape, 5 = Vivid
    int lutEnabled;          // 1 = 3D LUT active, 0 = disabled
    float lutIntensity;      // 0.0 .. 1.0
    float lutSize;           // Dimension N (e.g. 17, 33, 65)
    float sharpeningAmount;  // 0.0 .. 100.0 (default 0.0)
    float sharpeningRadius;  // 0.5 .. 3.0 (default 1.0)
    float sharpeningMasking; // 0.0 .. 100.0 (default 0.0)
    float noiseReduction;    // 0.0 .. 100.0 (default 0.0)
    int _padding;
    int _padding2;
};

// MARK: - Scene-Referred Rec.2020 Color Transfer Functions & Matrices (File Scope)

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
    
    // Blue skies from silicon Bayer filters peak around 235 deg (violet-blue).
    // Camera profiles rotate blue towards cyan/azure (215-220 deg)
    // and gently compress saturation to eliminate harsh purple casts.
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

// MARK: - AgX Filmic Tone Mapping Engine Constants & Operators
// AgX Inset Matrix (Linear Rec.2020 -> AgX Inset) - columns in Metal MSL
// Pre-multiplied as AGX_INSET_SRGB * REC2020_TO_LINEAR_SRGB so wide-gamut scene photons
// map accurately into AgX log working space in a single GPU cycle without gamut compression.
constant float3x3 AGX_INSET_MATRIX = float3x3(
    float3( 1.4096986,   0.13136263,  0.16151477),
    float3(-0.4004796,   0.77151406, -0.06034984),
    float3(-0.009218898, 0.09712328,  0.89883506)
);

// AgX Outset Matrix (AgX Curved -> Display sRGB) - columns in Metal MSL
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
    // 1. Gamut compression (prevents negative values)
    float minC = min(rec2020Color.r, min(rec2020Color.g, rec2020Color.b));
    float3 cComp = (minC < 0.0) ? (rec2020Color - minC) : rec2020Color;

    // 2. Inset Gamut Transform (Rec.2020 -> AgX Inset)
    float3 inset = AGX_INSET_MATRIX * cComp;
    
    // 3. Middle-gray (0.18) relative log2 normalization & Sigmoidal Transfer
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
    
    // Skin tone protection (15..45 deg, centered around 25 deg)
    float skinWeight = clamp(1.0 - abs(hsv.x - 25.0) / 20.0, 0.0, 1.0);
    // Sky blue covers 200..270 deg, centered around 235 deg
    float skyWeight = clamp(1.0 - abs(hsv.x - 235.0) / 35.0, 0.0, 1.0);
    
    // Azure sky shift
    hsv.x -= 18.0 * skyWeight;
    // Gentle peach alignment for skin (-3 deg away from yellow towards healthy rose)
    hsv.x -= 3.0 * skinWeight;
    if (hsv.x < 0.0) { hsv.x += 360.0; }
    if (hsv.x >= 360.0) { hsv.x -= 360.0; }
    
    // Vibrant midtone chroma with sky saturation normalized and delicate skin tones protected
    hsv.y = clamp(hsv.y * (1.18 - 0.20 * skinWeight - 0.15 * skyWeight), 0.0, 1.0);
    return clamp(hsv2rgb(hsv), 0.0, 1.0);
}

// MARK: - Vertex Shader
vertex VertexOutput vertexShader(
    uint vertexID [[vertex_id]],
    constant VertexInput *vertices [[buffer(0)]],
    constant ShaderUniforms &uniforms [[buffer(1)]]
) {
    VertexOutput out;
    float2 pos = vertices[vertexID].position;
    float2 uv = vertices[vertexID].texCoord;
    
    out.position = float4(pos, 0.0, 1.0);
    
    // Straighten rotation around center (0.5, 0.5) with aspect ratio correction and auto-crop
    if (uniforms.straightenAngle != 0.0 && uniforms.isOriginal < 0.5) {
        float angle = uniforms.straightenAngle;
        float cosA = cos(angle);
        float sinA = sin(angle);
        
        // Compute auto-crop inscribed zoom factor S(theta)
        float absSin = abs(sinA);
        float absCos = abs(cosA);
        float R = max(uniforms.aspectRatio, 0.01);
        float maxR = max(R, 1.0 / R);
        float zoom = max(absCos + absSin * maxR, 1.0);
        
        float2 centered = uv - float2(0.5, 0.5);
        centered.x *= uniforms.aspectRatio; // compensate aspect ratio
        
        float2 rotated = float2(
            centered.x * cosA - centered.y * sinA,
            centered.x * sinA + centered.y * cosA
        );
        rotated.x /= uniforms.aspectRatio;
        
        // Scale by 1.0 / zoom to eliminate black corner bars
        out.texCoord = (rotated / zoom) + float2(0.5, 0.5);
    } else {
        out.texCoord = uv;
    }
    
    return out;
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
    
    // Log-luminance EV relative to photographic middle gray (0.18)
    float ev = log2(max(lum, 1e-5) / 0.18);
    float delta_ev = 0.0;
    
    // 1. Blacks: Anchor deep shadows (ev <= -1.0)
    if (blacks != 0.0) {
        float u = clamp((-ev - 1.0) / 3.0, 0.0, 1.0);
        float w_blacks = u * u * (3.0 - 2.0 * u);
        delta_ev += (blacks / 100.0) * 1.5 * w_blacks;
    }
    
    // 2. Shadows: Lift lower midtones and shadow textures (ev in [-4.0, 0.0])
    if (shadows != 0.0) {
        float u = clamp((ev + 4.0) / 4.0, 0.0, 1.0);
        float w_shadows = sin(3.141592653589793 * u);
        delta_ev += (shadows / 100.0) * 1.75 * w_shadows;
    }
    
    // 3. Highlights: Recover or boost bright areas and skies (ev in [0.0, 4.0])
    if (highlights != 0.0) {
        float u = clamp(ev / 4.0, 0.0, 1.0);
        float w_highlights = sin(3.141592653589793 * u);
        delta_ev += (highlights / 100.0) * 2.0 * w_highlights;
    }
    
    // 4. Whites: Adjust specular highlight threshold and ceiling (ev >= 1.0)
    if (whites != 0.0) {
        float u = clamp((ev - 1.0) / 3.0, 0.0, 1.0);
        float w_whites = u * u * (3.0 - 2.0 * u);
        delta_ev += (whites / 100.0) * 1.5 * w_whites;
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

    // 1. Global Saturation (-100..+100)
    // Linear chromaticity scaling relative to perceptual luminance
    if (saturation != 0.0) {
        float satGain = max(1.0 + saturation / 100.0, 0.0);
        c = lum + (c - lum) * satGain;
    }

    // 2. Smart Vibrance (-100..+100)
    // Non-linear boost that protects already saturated tones and human skin tones.
    if (vibrance != 0.0) {
        float maxC = max(max(c.r, c.g), c.b);
        float minC = min(min(c.r, c.g), c.b);
        float chroma = maxC - minC;

        // Detect human skin tones in RGB: R > G > B with warm peach chromaticity
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

    // 3. Deep Blue (Sky & Water) (-100..+100)
    // Direct chromatic optical polarizing model:
    // Operates on blue excess over warm channels (B - max(R, G)).
    // 100% linear in RGB space -> ZERO hue-division noise -> ZERO dithering!
    if (deepBlue != 0.0) {
        float maxWarm = max(c.r, c.g);
        float skySignal = max(c.b - maxWarm, 0.0);
        if (skySignal > 0.0) {
            float x = deepBlue / 100.0;
            float factor = sign(x) * pow(abs(x), 1.6);
            if (factor > 0.0) {
                // Polarizing filter effect:
                // Attenuates blue luminance, removes red haze to enrich azure saturation
                c.r -= factor * skySignal * 0.30;
                c.g -= factor * skySignal * 0.15;
                c.b -= factor * skySignal * 0.20;
            } else {
                // Airy / luminous sky:
                // factor is negative -> lifts values gently and evenly without color casting
                c.r -= factor * skySignal * 0.25;
                c.g -= factor * skySignal * 0.20;
                c.b -= factor * skySignal * 0.10;
            }
        }
    }

    // 4. Foliage (Lush Greens) (-100..+100)
    // Operates on green excess (G - max(R, B)).
    // Shifts yellow grass towards rich emerald green (lower red, lift green, deepen blue).
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

// MARK: - Fragment Shader (Linear RAW Pipeline)
fragment float4 fragmentShader(
    VertexOutput in [[stage_in]],
    texture2d<float> texture [[texture(0)]],
    sampler textureSampler [[sampler(0)]],
    texture3d<float> lutTexture [[texture(1)]],
    sampler lutSampler [[sampler(1)]],
    constant ShaderUniforms &uniforms [[buffer(1)]]
) {
    float2 uv = in.texCoord;
    
    // Discard pixels rotated outside image bounds
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return float4(0.08, 0.08, 0.08, 1.0); // Neutral dark canvas background
    }
    
    // 1. Optics & Lens Distortion Correction
    if (uniforms.lensCorrectionEnabled != 0 && uniforms.isOriginal < 0.5) {
        float2 norm = (uv - float2(0.5, 0.5)) * 2.0;
        norm.x *= uniforms.aspectRatio;
        float r2 = dot(norm, norm);
        float r = sqrt(r2);
        
        if (r > 0.0001) {
            float r_corr = r;
            if (uniforms.distortionModel == 0) {
                // Poly3 model: r_d = r * (1 + k1*r^2 + k2*r^4)
                r_corr = r * (1.0 + uniforms.k1 * r2 + uniforms.k2 * (r2 * r2));
            } else if (uniforms.distortionModel == 1) {
                // PTLens model: r_d = a*r^4 + b*r^3 + c*r^2 + d*r
                float a = uniforms.k1;
                float b = uniforms.k2;
                float c = uniforms.k3;
                float d = 1.0 - (a + b + c);
                r_corr = a * (r2 * r2) + b * (r2 * r) + c * r2 + d * r;
            }
            float factor = r_corr / r;
            norm *= factor;
            norm.x /= uniforms.aspectRatio;
            uv = norm * 0.5 + float2(0.5, 0.5);
            
            if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
                return float4(0.08, 0.08, 0.08, 1.0);
            }
        }
    }
    
    // Sample texture
    float4 sampledColor = texture.sample(textureSampler, uv);
    
    // If user is holding '\' for original as-shot comparison:
    // For JPEG thumbnails, return sampledColor directly.
    if (uniforms.isOriginal > 0.5 && uniforms.isLinearRaw == 0) {
        return sampledColor;
    }
    
    // Fast-path: When all edit sliders are at default neutral positions for JPEG thumbnails, return exact as-shot pixels.
    // (Linear RAW must always pass through the baseline exposure, filmic curve, and sRGB color transform).
    bool isNeutralEdits = (uniforms.exposure == 0.0) && 
                          (abs(uniforms.temperature - 5500.0) < 1.0) && 
                          (uniforms.tint == 0.0) && 
                          (uniforms.contrast == 0.0) &&
                          (uniforms.highlights == 0.0) &&
                          (uniforms.shadows == 0.0) &&
                          (uniforms.whites == 0.0) &&
                          (uniforms.blacks == 0.0) &&
                          (uniforms.vibrance == 0.0) &&
                          (uniforms.saturation == 0.0) &&
                          (uniforms.deepBlue == 0.0) &&
                          (uniforms.foliage == 0.0) && 
                          (uniforms.lutEnabled == 0) &&
                          (uniforms.sharpeningAmount == 0.0) &&
                          (uniforms.noiseReduction == 0.0);
    
    if (uniforms.isLinearRaw == 0 && isNeutralEdits) {
        return sampledColor;
    }
    
    float editExposure = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.exposure;
    float editTemp = (uniforms.isOriginal > 0.5) ? 5500.0 : uniforms.temperature;
    float editTint = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.tint;
    float editContrast = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.contrast;
    float editHighlights = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.highlights;
    float editShadows = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.shadows;
    float editWhites = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.whites;
    float editBlacks = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.blacks;
    float editVibrance = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.vibrance;
    float editSaturation = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.saturation;
    float editDeepBlue = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.deepBlue;
    float editFoliage = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.foliage;
    
    // 2. Decode texture to Linear Rec.2020 working space
    float3 rec2020Color;
    if (uniforms.isLinearRaw == 1) {
        rec2020Color = sampledColor.rgb;
    } else {
        float3 srgbLinear = srgbGammaToLinear3(sampledColor.rgb);
        rec2020Color = LINEAR_SRGB_TO_REC2020 * srgbLinear;
    }
    
    // 3. White Balance & Temperature Adjustment in Wide-Gamut Space
    if (abs(editTemp - 5500.0) >= 1.0 || editTint != 0.0) {
        float tempNorm = clamp(editTemp / 5500.0, 0.25, 4.0);
        float rScale = pow(tempNorm, 0.75);
        float bScale = pow(1.0 / tempNorm, 0.75);
        
        float tintFactor = editTint * 0.003;
        float gScale = 1.0 - tintFactor;
        rScale += tintFactor * 0.5;
        bScale += tintFactor * 0.5;
        
        rec2020Color.r *= rScale;
        rec2020Color.g *= gScale;
        rec2020Color.b *= bScale;
    }
    
    // 4. Linear Exposure Multiplication (2^EV) in Wide-Gamut Space
    float totalExposure = editExposure;
    if (totalExposure != 0.0) {
        rec2020Color *= pow(2.0, totalExposure);
    }
    
    // 4b. Selective Tone Recovery (Highlights, Shadows, Whites, Blacks)
    rec2020Color = applyToneRecovery(rec2020Color, editHighlights, editShadows, editWhites, editBlacks);
    
    // 5. AgX Filmic Tone Mapping Engine OR Camera-Matching Profiles
    float3 srgbLinearOut;
    if (uniforms.toneProfileMode == 0) {
        // Mode 0: AgX Vivid (Filmic Punchy) - DEFAULT
        float contrastGain = 1.0 + (editContrast / 100.0);
        srgbLinearOut = agxToneMapVivid(rec2020Color, contrastGain);
    } else if (uniforms.toneProfileMode == 1) {
        // Mode 1: AgX Neutral (Filmic Base)
        if (uniforms.isLinearRaw == 1 || editContrast != 0.0) {
            float contrastGain = 1.0 + (editContrast / 100.0);
            srgbLinearOut = agxToneMap(rec2020Color, contrastGain);
        } else {
            srgbLinearOut = max(REC2020_TO_LINEAR_SRGB * rec2020Color, float3(0.0));
        }
    } else {
        // Modes 2..5: Camera Matching (Standard, Portrait, Landscape, Vivid)
        int camMode = uniforms.toneProfileMode - 1; // 2->1 (Standard), 3->2 (Portrait), 4->3 (Landscape), 5->4 (Vivid)
        float3 c_srgb = max(REC2020_TO_LINEAR_SRGB * rec2020Color, float3(0.0));
        float3 hsv = rgb2hsv(c_srgb);
        float3 hsvAdjusted = applyCameraHueSat(hsv, camMode);
        float3 rgbAdjusted = hsv2rgb(hsvAdjusted);
        
        // Luminance-scaled tone curve: evaluates tone curve on luminance axis Y,
        // preserving chromaticity ratios (R/Y, G/Y, B/Y) without hue twisting.
        float lum = dot(rgbAdjusted, float3(0.2126, 0.7152, 0.0722));
        float lumCurved = applyCameraToneCurve(lum, camMode);
        float contrastGain = 1.0 + (editContrast / 100.0);
        float lumAdj = clamp((lumCurved - 0.18) * contrastGain + 0.18, 0.0, 1.0);
        float gain = lumAdj / max(lum, 1e-6);
        
        srgbLinearOut = clamp(rgbAdjusted * gain, 0.0, 1.0);
    }
    
    // 6. Display Target Transform:
    // AgX Filmic tone mapping (modes 0 and 1) directly produces display-referred sRGB values (with AgX 2.4 gamma applied).
    // Applying linearToSrgbGamma3 to AgX output cancels the 2.4 power curve ((x^2.4)^(1/2.2) ~= x),
    // blowing out midtones and highlights. Camera-matching DCP profiles output linear sRGB and require gamma encoding.
    float3 finalColor;
    if (uniforms.toneProfileMode <= 1) {
        finalColor = clamp(srgbLinearOut, 0.0, 1.0);
    } else {
        finalColor = linearToSrgbGamma3(srgbLinearOut);
    }
    
    // 6b. Color & Vibrance Adjustments
    finalColor = applyColorAdjustments(finalColor, editVibrance, editSaturation, editDeepBlue, editFoliage);
    
    // 7. Creative 3D Look-Up Table (Post-Tone-Mapping)
    if (uniforms.lutEnabled != 0 && uniforms.lutIntensity > 0.0 && uniforms.isOriginal < 0.5) {
        float size = max(uniforms.lutSize, 2.0);
        float3 lutCoord = (finalColor * (size - 1.0) + 0.5) / size;
        float3 lutSample = lutTexture.sample(lutSampler, lutCoord).rgb;
        finalColor = mix(finalColor, lutSample, clamp(uniforms.lutIntensity, 0.0, 1.0));
    }

    // 8. Detail, Sharpening & Noise Reduction
    float editSharpening = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.sharpeningAmount;
    float editNoiseReduction = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.noiseReduction;
    if (editSharpening > 0.0 || editNoiseReduction > 0.0) {
        float2 texelSize = 1.0 / float2(texture.get_width(), texture.get_height());
        float2 offset = texelSize * max(uniforms.sharpeningRadius, 0.5);
        float4 leftSample   = texture.sample(textureSampler, uv - float2(offset.x, 0.0));
        float4 rightSample  = texture.sample(textureSampler, uv + float2(offset.x, 0.0));
        float4 topSample    = texture.sample(textureSampler, uv - float2(0.0, offset.y));
        float4 bottomSample = texture.sample(textureSampler, uv + float2(0.0, offset.y));

        float y_c = dot(sampledColor.rgb, float3(0.2126, 0.7152, 0.0722));
        float y_l = dot(leftSample.rgb, float3(0.2126, 0.7152, 0.0722));
        float y_r = dot(rightSample.rgb, float3(0.2126, 0.7152, 0.0722));
        float y_t = dot(topSample.rgb, float3(0.2126, 0.7152, 0.0722));
        float y_b = dot(bottomSample.rgb, float3(0.2126, 0.7152, 0.0722));

        finalColor = applyDetailAndSharpening(
            finalColor,
            y_c, y_l, y_r, y_t, y_b,
            editSharpening,
            uniforms.sharpeningMasking,
            editNoiseReduction
        );
    }
    
    return float4(finalColor, sampledColor.a);
}
