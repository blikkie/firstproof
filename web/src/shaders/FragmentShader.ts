import { COLOR_SCIENCE_GLSL } from './ColorScience';
import { DETAIL_GLSL } from './Detail';

export const FRAGMENT_SHADER_SOURCE = `#version 300 es
precision highp float;

in vec2 v_texCoord;
out vec4 fragColor;

uniform sampler2D u_texture;
uniform sampler2D u_curveMasterLut;
uniform sampler2D u_curveRgbLut;

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
uniform int u_emulsionArchetype;
uniform float u_emulsionExposure;
uniform int u_emulsionGrainMode;
uniform float u_grainScale;
uniform float u_grainSize;
uniform float u_grainAmount;
uniform float u_formatMagnification;
uniform int u_halationMode;

uniform int u_printMediaMode;
uniform float u_printPaperGrade;
uniform float u_printProcessDepth;
uniform int u_enlargerFilterMode;

uniform float u_splitShadowHue;
uniform float u_splitShadowSat;
uniform float u_splitHighlightHue;
uniform float u_splitHighlightSat;
uniform float u_splitBalance;

uniform float u_sharpeningAmount;
uniform float u_sharpeningRadius;
uniform float u_sharpeningMasking;
uniform float u_noiseReduction;
uniform float u_baselineSharpness;

uniform float u_cropLeft;
uniform float u_cropTop;
uniform float u_cropRight;
uniform float u_cropBottom;
uniform int u_cropModeActive;
uniform float u_straightenAngle;
uniform int u_cropConstrain;
uniform float u_aspectRatio;
uniform int u_rotation;
uniform int u_flipH;
uniform int u_flipV;
uniform int u_isIntermediate;
uniform float u_perspectiveVertical;
uniform float u_perspectiveHorizontal;

uniform int u_isOriginal;
uniform int u_isLinearRec2020;
uniform int u_hasMasterCurve;
uniform int u_hasRgbCurves;

${COLOR_SCIENCE_GLSL}
${DETAIL_GLSL}

// Interleaved Gradient Noise (Jorge Jimenez, Next Generation Post Processing)
// Extremely fast, zero texture lookups, optimal blue-noise-like properties for sample jitter and dithering
float ign(vec2 p) {
    vec3 magic = vec3(0.06711056, 0.00583715, 52.9829189);
    return fract(magic.z * fract(dot(p, magic.xy)));
}

// Tone Curve Coordinate Mapping (Half-texel offset for exact 256-sample 1D LUT)
vec2 sampleCurveCoord(float val) {
    float u = (clamp(val, 0.0, 1.0) * 255.0 + 0.5) / 256.0;
    return vec2(u, 0.5);
}

// Computes display-referred luminance for a linear Rec.2020 color through AgX or camera tone profile
float computeDisplayLuma(vec3 c) {
    float contrastGain = (u_contrast >= 0.0) ? (1.0 + u_contrast / 100.0) : (1.0 + (u_contrast / 100.0) * 0.60);
    vec3 srgbOut;
    if (u_toneProfileMode == 0) {
        srgbOut = agxToneMapVivid(c, contrastGain);
    } else if (u_toneProfileMode == 1) {
        srgbOut = agxToneMap(c, contrastGain);
    } else {
        int camMode = u_toneProfileMode - 1;
        vec3 c_srgb = max(REC2020_TO_LINEAR_SRGB * c, vec3(0.0));
        float lum = dot(c_srgb, vec3(0.2627, 0.6780, 0.0593));
        float lumCurved = applyCameraToneCurve(lum, camMode);
        return clamp((lumCurved - 0.18) * contrastGain + 0.18, 0.0, 1.0);
    }
    return dot(clamp(srgbOut, 0.0, 1.0), vec3(0.2627, 0.6780, 0.0593));
}

// Evaluates display-referred luminance for a given texture UV coordinate
// Bypasses curve/LUT but accurately models exposure, WB, tone recovery, and AgX/camera tone profile
// to ensure Laplacian unsharp mask operates on calibrated [0, 1] display luma without shadow noise explosion.
float sampleDisplayLuma(vec2 uv) {
    vec4 s = texture(u_texture, uv);
    vec3 c;
    if (u_isLinearRec2020 != 0) {
        c = s.rgb;
    } else {
        c = LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(s.rgb);
    }
    if (abs(u_temperature - 5500.0) >= 1.0 || u_tint != 0.0) {
        float tempNorm = clamp(u_temperature / 5500.0, 0.25, 4.0);
        float rScale = pow(tempNorm, 0.75);
        float bScale = pow(1.0 / tempNorm, 0.75);
        float tintFactor = u_tint * 0.002;
        float gScale = max(1.0 - tintFactor, 0.01);
        rScale = max(rScale + tintFactor * 0.5, 0.01);
        bScale = max(bScale + tintFactor * 0.5, 0.01);
        c *= vec3(rScale, gScale, bScale);
    }
    if (u_exposure != 0.0) {
        c *= pow(2.0, u_exposure);
    }
    c = applyToneRecovery(c, u_highlights, u_shadows, u_whites, u_blacks);
    if (u_emulsionArchetype != 0) {
        c = applyRecordingEmulsion(c, u_emulsionArchetype, u_emulsionExposure, u_enlargerFilterMode);
    }
    return computeDisplayLuma(c);
}

// Viewport Geometry, Perspective & Lens Distortion Transform
// Ported 1:1 from FirstProofApp/Sources/Shaders/Geometry.metal
vec2 transformGeometry(vec2 uv) {
    if (u_isOriginal != 0 || u_isIntermediate != 0) {
        return uv;
    }

    if (u_rotation == 0 && u_flipH == 0 && u_flipV == 0 &&
        u_cropModeActive == 0 && u_cropLeft <= 0.0001 && u_cropTop <= 0.0001 &&
        u_cropRight >= 0.9999 && u_cropBottom >= 0.9999 &&
        abs(u_straightenAngle) <= 0.0001 &&
        abs(u_perspectiveVertical) <= 0.0001 && abs(u_perspectiveHorizontal) <= 0.0001) {
        return uv;
    }

    // 1. Base Orientation (Flips and 90/180/270 Rotation)
    vec2 inCoord = uv;
    if (u_flipH != 0) {
        inCoord.x = 1.0 - inCoord.x;
    }
    if (u_flipV != 0) {
        inCoord.y = 1.0 - inCoord.y;
    }

    vec2 orientedUv = inCoord;
    if (u_rotation == 90) {
        orientedUv = vec2(inCoord.y, 1.0 - inCoord.x);
    } else if (u_rotation == 180) {
        orientedUv = vec2(1.0 - inCoord.x, 1.0 - inCoord.y);
    } else if (u_rotation == 270) {
        orientedUv = vec2(1.0 - inCoord.y, inCoord.x);
    }

    // 2. Crop Inscription (Normal Viewing vs Interactive Crop Tool)
    float cropL = clamp(u_cropLeft, 0.0, 0.99);
    float cropT = clamp(u_cropTop, 0.0, 0.99);
    float cropR = clamp(u_cropRight, cropL + 0.01, 1.0);
    float cropB = clamp(u_cropBottom, cropT + 0.01, 1.0);

    vec2 uvc = orientedUv;
    if (u_cropModeActive == 0) {
        uvc.x = cropL + orientedUv.x * (cropR - cropL);
        uvc.y = cropT + orientedUv.y * (cropB - cropT);
    }

    // 3. Aspect-Ratio Scaled Straighten & Auto-Crop Scaling
    float A = max(u_aspectRatio, 0.01);
    float angle = u_straightenAngle;
    float cosA = cos(angle);
    float sinA = sin(angle);
    float cV = u_perspectiveVertical * 0.005;
    float cH = u_perspectiveHorizontal * 0.005;

    float autoCropScale = 1.0;
    if (u_cropConstrain != 0) {
        vec2 testPoints[12];
        testPoints[0] = vec2( A,  1.0); testPoints[1] = vec2(-A,  1.0);
        testPoints[2] = vec2( A, -1.0); testPoints[3] = vec2(-A, -1.0);
        testPoints[4] = vec2( A,  0.0); testPoints[5] = vec2(-A,  0.0);
        testPoints[6] = vec2(0.0,  1.0); testPoints[7] = vec2(0.0, -1.0);
        testPoints[8] = vec2( A,  0.5); testPoints[9] = vec2(-A,  0.5);
        testPoints[10] = vec2( A, -0.5); testPoints[11] = vec2(-A, -0.5);
        for (int i = 0; i < 12; i++) {
            vec2 p = testPoints[i];
            vec2 r = vec2(
                p.x * cosA - p.y * sinA,
                p.x * sinA + p.y * cosA
            );
            float denom = max(1.0 + cV * r.y + cH * r.x, 0.1);
            vec2 g = r / denom;
            float req = max(abs(g.x) / A, abs(g.y));
            if (req > autoCropScale) {
                autoCropScale = req;
            }
        }
    }

    // Centered coordinates scaled by aspect ratio
    vec2 centered = (uvc - vec2(0.5, 0.5)) * 2.0;
    centered.x *= A;
    centered /= autoCropScale;

    // Straighten rotation
    vec2 rot = vec2(
        centered.x * cosA - centered.y * sinA,
        centered.x * sinA + centered.y * cosA
    );

    // Perspective keystone (backward projective homography)
    float denom = max(1.0 + cV * rot.y + cH * rot.x, 0.1);
    vec2 geom = rot / denom;

    geom.x /= (2.0 * A);
    geom.y /= 2.0;
    return geom + vec2(0.5, 0.5);
}

void main() {
    // 0. Fast-path intermediate pre-rendered snapshots (bypass all transforms)
    if (u_isIntermediate != 0) {
        fragColor = texture(u_texture, v_texCoord);
        return;
    }

    // 1. Viewport Geometry Transformation
    vec2 uv = transformGeometry(v_texCoord);

    // Discard / letterbox pixels outside valid texture bounds
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        fragColor = vec4(0.08, 0.08, 0.08, 1.0); // Neutral dark canvas background
        return;
    }

    vec4 sampled = texture(u_texture, uv);

    // Quick peek original as-shot frame
    if (u_isOriginal != 0) {
        if (u_isLinearRec2020 != 0) {
            vec3 srgbLinearOut = agxToneMap(sampled.rgb, 1.0);
            float dither = (ign(gl_FragCoord.xy) - 0.5) / 255.0;
            fragColor = vec4(clamp(linearToSrgbGamma3(clamp(srgbLinearOut, 0.0, 1.0)) + vec3(dither), 0.0, 1.0), 1.0);
        } else {
            fragColor = sampled;
        }
        return;
    }

    // Fast-path for intermediate pre-rendered snapshots
    bool isNeutralEdits = (u_toneProfileMode == 0) &&
        (abs(u_exposure) < 0.001) &&
        (abs(u_contrast) < 0.001) &&
        (abs(u_highlights) < 0.001) &&
        (abs(u_shadows) < 0.001) &&
        (abs(u_whites) < 0.001) &&
        (abs(u_blacks) < 0.001) &&
        (abs(u_vibrance) < 0.001) &&
        (abs(u_saturation) < 0.001) &&
        (abs(u_deepBlue) < 0.001) &&
        (abs(u_foliage) < 0.001) &&
        (abs(u_temperature - 5500.0) < 1.0) &&
        (abs(u_tint) < 0.001) &&
        (u_emulsionArchetype == 0) &&
        (u_halationMode == 0) &&
        (u_emulsionGrainMode == 0) &&
        (u_printMediaMode == 0) &&
        (u_splitShadowSat < 0.001) &&
        (u_splitHighlightSat < 0.001) &&
        (u_hasMasterCurve == 0) &&
        (u_hasRgbCurves == 0);

    vec3 rec2020Color;
    if (u_isLinearRec2020 != 0) {
        rec2020Color = sampled.rgb;
    } else {
        rec2020Color = LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(sampled.rgb);
    }

    // 1. White Balance in Linear Rec.2020
    float rScale = 1.0;
    float gScale = 1.0;
    float bScale = 1.0;
    if (abs(u_temperature - 5500.0) >= 1.0 || u_tint != 0.0) {
        float tempNorm = clamp(u_temperature / 5500.0, 0.25, 4.0);
        rScale = pow(tempNorm, 0.75);
        bScale = pow(1.0 / tempNorm, 0.75);
        
        float tintFactor = u_tint * 0.002;
        gScale = max(1.0 - tintFactor, 0.01);
        rScale = max(rScale + tintFactor * 0.5, 0.01);
        bScale = max(bScale + tintFactor * 0.5, 0.01);
        
        rec2020Color *= vec3(rScale, gScale, bScale);
    }

    // 2. Scene-Referred Exposure
    float exposureGain = 1.0;
    if (u_exposure != 0.0) {
        exposureGain = pow(2.0, u_exposure);
        rec2020Color *= exposureGain;
    }

    // 3. Multi-Scale Hierarchical Bilateral Tone Recovery in Linear Rec.2020
    if (u_highlights != 0.0 || u_shadows != 0.0 || u_whites != 0.0 || u_blacks != 0.0) {
        vec2 texSize = vec2(textureSize(u_texture, 0));
        vec2 texelSize = 1.0 / texSize;
        float maxDim = max(texSize.x, texSize.y);
        float rInner = 1.0;
        float rOuter = max(16.0, round(maxDim * 0.016));
        vec2 stepInner = texelSize * rInner;
        vec2 stepOuter = texelSize * rOuter;

        float y_c = dot(rec2020Color, vec3(0.2627, 0.6780, 0.0593));
        vec3 wbExposureScale = vec3(rScale, gScale, bScale) * exposureGain;

        // Inner ring: 4 orthogonal + 4 diagonal (rInner = 1 texel)
        vec4 s_l  = texture(u_texture, uv + vec2(-stepInner.x, 0.0));
        vec4 s_r  = texture(u_texture, uv + vec2( stepInner.x, 0.0));
        vec4 s_t  = texture(u_texture, uv + vec2(0.0, -stepInner.y));
        vec4 s_b  = texture(u_texture, uv + vec2(0.0,  stepInner.y));
        vec4 s_tl = texture(u_texture, uv + vec2(-stepInner.x, -stepInner.y));
        vec4 s_tr = texture(u_texture, uv + vec2( stepInner.x, -stepInner.y));
        vec4 s_bl = texture(u_texture, uv + vec2(-stepInner.x,  stepInner.y));
        vec4 s_br = texture(u_texture, uv + vec2( stepInner.x,  stepInner.y));

        // Outer ring: 4 orthogonal + 4 diagonal (rOuter)
        vec4 s_ol  = texture(u_texture, uv + vec2(-stepOuter.x, 0.0));
        vec4 s_or  = texture(u_texture, uv + vec2( stepOuter.x, 0.0));
        vec4 s_ot  = texture(u_texture, uv + vec2(0.0, -stepOuter.y));
        vec4 s_ob  = texture(u_texture, uv + vec2(0.0,  stepOuter.y));
        vec4 s_otl = texture(u_texture, uv + vec2(-stepOuter.x, -stepOuter.y));
        vec4 s_otr = texture(u_texture, uv + vec2( stepOuter.x, -stepOuter.y));
        vec4 s_obl = texture(u_texture, uv + vec2(-stepOuter.x,  stepOuter.y));
        vec4 s_obr = texture(u_texture, uv + vec2( stepOuter.x,  stepOuter.y));

        vec4 s_neighbors[16] = vec4[16](
            s_l, s_r, s_t, s_b, s_tl, s_tr, s_bl, s_br,
            s_ol, s_or, s_ot, s_ob, s_otl, s_otr, s_obl, s_obr
        );
        float y_neighbors[16];
        for (int i = 0; i < 16; ++i) {
            vec3 c;
            if (u_isLinearRec2020 != 0) {
                c = s_neighbors[i].rgb;
            } else {
                c = LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(s_neighbors[i].rgb);
            }
            c *= wbExposureScale;
            y_neighbors[i] = dot(c, vec3(0.2627, 0.6780, 0.0593));
        }

        rec2020Color = applyBilateralToneRecovery(
            rec2020Color,
            y_c,
            y_neighbors,
            u_highlights,
            u_shadows,
            u_whites,
            u_blacks
        );
    }

    // 4. Photochemical Recording Emulsion in Linear Rec.2020
    if (u_emulsionArchetype != 0) {
        rec2020Color = applyRecordingEmulsion(rec2020Color, u_emulsionArchetype, u_emulsionExposure, u_enlargerFilterMode);
    }

    // 4c. Physical Optical Halation Scatter (Point-Spread Emulsion Diffusion)
    if (u_isOriginal == 0 && u_halationMode > 0 && u_emulsionArchetype != 7) {
        float aspect = max(u_aspectRatio, 0.01);
        vec2 halationRadius;
        if (u_rotation == 90 || u_rotation == 270) {
            halationRadius = vec2(0.012 * aspect * u_formatMagnification, 0.012 * u_formatMagnification);
        } else {
            halationRadius = vec2((0.012 / aspect) * u_formatMagnification, 0.012 * u_formatMagnification);
        }

        // Stochastic rotation per fragment via Interleaved Gradient Noise
        // Smooths out discrete Poisson disk tap rings into a continuous, band-free spatial point-spread function
        float jitter = ign(gl_FragCoord.xy) * 6.2831853;
        float cosJ = cos(jitter);
        float sinJ = sin(jitter);
        mat2 rot = mat2(cosJ, -sinJ, sinJ, cosJ);

        // 16-tap Fermat golden spiral with exponential back-scatter distance weighting:
        // Sampling: r_i = sqrt((i + 0.5) / 16), theta_i = i * phi (golden angle phi = pi * (3 - sqrt(5)) ~ 137.507764 deg)
        // Optical PSF: exponential decay w_i = exp(-2.5 * r_i) modeling diffuse photon transmission through film base.
        // Weight normalization: sum(w_i) = 3.6187 used directly below for energy-conserving diffuse glow.
        const vec2 fermatDisk[16] = vec2[16](
            vec2( 0.1768,  0.0000), vec2(-0.2258,  0.2068), vec2( 0.0346, -0.3938), vec2( 0.2846,  0.3712),
            vec2(-0.5222, -0.0924), vec2( 0.4947, -0.3147), vec2(-0.1655,  0.6155), vec2(-0.3156, -0.6076),
            vec2( 0.6846,  0.2500), vec2(-0.7123,  0.2940), vec2( 0.3434, -0.7337), vec2( 0.2537,  0.8089),
            vec2(-0.7647, -0.4432), vec2( 0.8971, -0.1972), vec2(-0.5475,  0.7788), vec2(-0.1265, -0.9761)
        );
        const float fermatWeights[16] = float[16](
            0.6428, 0.4651, 0.3722, 0.3106,
            0.2656, 0.2309, 0.2032, 0.1806,
            0.1617, 0.1457, 0.1320, 0.1201,
            0.1097, 0.1006, 0.0926, 0.0854
        );

        float tempNorm = clamp(u_temperature / 5500.0, 0.25, 4.0);
        float rScale = pow(tempNorm, 0.75);
        float bScale = pow(1.0 / tempNorm, 0.75);
        float tintFactor = u_tint * 0.002;
        float gScale = max(1.0 - tintFactor, 0.01);
        rScale = max(rScale + tintFactor * 0.5, 0.01);
        bScale = max(bScale + tintFactor * 0.5, 0.01);
        vec3 wbExposure = vec3(rScale, gScale, bScale) * pow(2.0, u_exposure);

        float scatterAcc = 0.0;
        for (int i = 0; i < 16; ++i) {
            vec2 rotatedOffset = (rot * fermatDisk[i]) * halationRadius;
            vec4 s = texture(u_texture, uv + rotatedOffset);
            vec3 c;
            if (u_isLinearRec2020 != 0) {
                c = s.rgb;
            } else {
                c = LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(s.rgb);
            }
            c *= wbExposure;
            float y_tap = dot(c, vec3(0.2627, 0.6780, 0.0593));
            if (y_tap > 0.55) {
                float diff = y_tap - 0.55;
                float excess = (diff * diff) / (0.25 + diff);
                scatterAcc += excess * fermatWeights[i];
            }
        }

        float halo = scatterAcc / 3.6187;
        if (halo > 1e-5) {
            if (u_halationMode == 1) {
                // Baoding Silver 100: Neutral panchromatic silver glow
                rec2020Color += vec3(halo * 0.35);
            } else if (u_halationMode == 2) {
                // Rochester Cinema 500 (Stripped Rem-Jet / CineStill): Red-shifted halation
                rec2020Color += vec3(halo * 0.50, halo * 0.12, halo * 0.02);
            }
        }
    }

    vec3 rec2020PreCurve = rec2020Color;

    // 5. Tone Curves in Linear Rec.2020 Working Space
    if (u_hasMasterCurve != 0) {
        vec3 c = clamp(rec2020Color, 0.0, 1.0);
        rec2020Color = vec3(
            texture(u_curveMasterLut, sampleCurveCoord(c.r)).r,
            texture(u_curveMasterLut, sampleCurveCoord(c.g)).r,
            texture(u_curveMasterLut, sampleCurveCoord(c.b)).r
        );
    }
    if (u_hasRgbCurves != 0) {
        vec3 c = clamp(rec2020Color, 0.0, 1.0);
        rec2020Color = vec3(
            texture(u_curveRgbLut, sampleCurveCoord(c.r)).r,
            texture(u_curveRgbLut, sampleCurveCoord(c.g)).g,
            texture(u_curveRgbLut, sampleCurveCoord(c.b)).b
        );
    }
    
    // 6. Color & Vibrance in Linear Rec.2020
    rec2020Color = applyColorAdjustments(rec2020Color, u_vibrance, u_saturation, u_deepBlue, u_foliage);
    
    // 7. Split Toning in Linear Rec.2020
    rec2020Color = applySplitToning(rec2020Color, u_splitShadowHue, u_splitShadowSat, u_splitHighlightHue, u_splitHighlightSat, u_splitBalance);

    // 8. Detail & Sharpening with Edge Masking & Stochastic Emulsion Resynthesis in Linear Rec.2020
    if (u_sharpeningAmount > 0.0 || u_noiseReduction > 0.0 || u_emulsionGrainMode != 0) {
        vec2 texSize = vec2(textureSize(u_texture, 0));
        vec2 pixelCoord = uv * texSize;
        float y_c = computeDisplayLuma(rec2020PreCurve);

        if (u_sharpeningAmount > 0.0 || u_noiseReduction > 0.0) {
            float effectiveRadius = max(u_sharpeningRadius * u_baselineSharpness, 0.5);
            vec2 sharpenOffset = vec2(effectiveRadius) / texSize;
            float y_l = sampleDisplayLuma(uv - vec2(sharpenOffset.x, 0.0));
            float y_r = sampleDisplayLuma(uv + vec2(sharpenOffset.x, 0.0));
            float y_t = sampleDisplayLuma(uv - vec2(0.0, sharpenOffset.y));
            float y_b = sampleDisplayLuma(uv + vec2(0.0, sharpenOffset.y));
            float y_tl = sampleDisplayLuma(uv - sharpenOffset);
            float y_tr = sampleDisplayLuma(uv + vec2(sharpenOffset.x, -sharpenOffset.y));
            float y_bl = sampleDisplayLuma(uv + vec2(-sharpenOffset.x, sharpenOffset.y));
            float y_br = sampleDisplayLuma(uv + sharpenOffset);

            rec2020Color = applyDetailAndSharpening(
                rec2020Color,
                y_c, y_l, y_r, y_t, y_b, y_tl, y_tr, y_bl, y_br,
                u_sharpeningAmount,
                u_sharpeningMasking,
                u_noiseReduction
            );
        }

        if (u_emulsionGrainMode != 0) {
            vec2 grainOffset = 1.0 / texSize;
            float gy_l = sampleDisplayLuma(uv - vec2(grainOffset.x, 0.0));
            float gy_r = sampleDisplayLuma(uv + vec2(grainOffset.x, 0.0));
            float gy_t = sampleDisplayLuma(uv - vec2(0.0, grainOffset.y));
            float gy_b = sampleDisplayLuma(uv + vec2(0.0, grainOffset.y));
            float gy_tl = sampleDisplayLuma(uv - grainOffset);
            float gy_tr = sampleDisplayLuma(uv + vec2(grainOffset.x, -grainOffset.y));
            float gy_bl = sampleDisplayLuma(uv + vec2(-grainOffset.x, grainOffset.y));
            float gy_br = sampleDisplayLuma(uv + grainOffset);

            float s_format = sqrt(max(u_formatMagnification, 0.1));
            float clusterSize = clamp(u_grainSize * s_format, 0.5, 3.5);
            float effectiveAmount = u_grainAmount * pow(max(u_formatMagnification, 0.1), 0.25);

            rec2020Color = applyStochasticEmulsionGrain(
                rec2020Color,
                y_c,
                gy_l,
                gy_r,
                gy_t,
                gy_b,
                gy_tl,
                gy_tr,
                gy_bl,
                gy_br,
                pixelCoord,
                u_emulsionGrainMode,
                clusterSize,
                effectiveAmount,
                u_emulsionArchetype
            );
        }
    }

    // 10. TERMINAL STAGE: AgX Filmic Tone Mapping Engine OR Camera-Matching Profiles -> Linear sRGB
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
        vec3 rgbAdjusted = c_srgb;
        
        // Continuous sensitometric per-channel tone curve (Hurter-Driffield film D-log H model):
        // Evaluates each channel through the smooth C^1 camera spline.
        // Naturally self-limiting to [0.0, 1.0] and desaturates highlights to white along the shoulder without arbitrary thresholds.
        vec3 curved = vec3(
            applyCameraToneCurve(rgbAdjusted.r, camMode),
            applyCameraToneCurve(rgbAdjusted.g, camMode),
            applyCameraToneCurve(rgbAdjusted.b, camMode)
        );
        if (abs(contrastGain - 1.0) > 0.001) {
            srgbLinearOut = clamp((curved - 0.18) * contrastGain + 0.18, 0.0, 1.0);
        } else {
            srgbLinearOut = clamp(curved, 0.0, 1.0);
        }
    }

    // 11. Curated Fine-Art Print Media Archetypes & Optical Enlarger Filtration in Linear Display Space
    if (u_printMediaMode != 0) {
        srgbLinearOut = applyPrintMedia(srgbLinearOut, u_printMediaMode, u_enlargerFilterMode, u_printPaperGrade, u_printProcessDepth);
    }

    // 12. Final Display Output: Display Gamma with 8-bit TPDF dither (The VERY LAST step!)
    // Completely eliminates 8-bit display quantization banding on smooth gradients
    float dither = (ign(gl_FragCoord.xy) - 0.5) / 255.0;
    fragColor = vec4(linearToSrgbGamma3(clamp(srgbLinearOut, 0.0, 1.0)) + vec3(dither), 1.0);
}
`;
