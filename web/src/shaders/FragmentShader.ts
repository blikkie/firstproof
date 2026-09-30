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

uniform int u_printMediaMode;
uniform float u_printPaperGrade;
uniform float u_printBaseWarmth;
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

uniform int u_isOriginal;
uniform int u_isCameraJpeg;
uniform int u_isLinearRec2020;
uniform int u_hasMasterCurve;
uniform int u_hasRgbCurves;
uniform sampler2D u_cameraJpegTexture;

${COLOR_SCIENCE_GLSL}
${DETAIL_GLSL}

// Tone Curve Coordinate Mapping (Half-texel offset for exact 256-sample 1D LUT)
vec2 sampleCurveCoord(float val) {
    float u = (clamp(val, 0.0, 1.0) * 255.0 + 0.5) / 256.0;
    return vec2(u, 0.5);
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
        c = applyRecordingEmulsion(c, u_emulsionArchetype, u_emulsionExposure);
    }
    float contrastGain = (u_contrast >= 0.0) ? (1.0 + u_contrast / 100.0) : (1.0 + (u_contrast / 100.0) * 0.60);
    vec3 srgbOut;
    if (u_toneProfileMode == 0) {
        srgbOut = agxToneMapVivid(c, contrastGain);
    } else if (u_toneProfileMode == 1) {
        srgbOut = agxToneMap(c, contrastGain);
    } else {
        int camMode = u_toneProfileMode - 1;
        vec3 c_srgb = max(REC2020_TO_LINEAR_SRGB * c, vec3(0.0));
        vec3 rgbAdjusted = c_srgb;
        float lum = dot(rgbAdjusted, vec3(0.2627, 0.6780, 0.0593));
        float lumCurved = applyCameraToneCurve(lum, camMode);
        return clamp((lumCurved - 0.18) * contrastGain + 0.18, 0.0, 1.0);
    }
    return dot(clamp(srgbOut, 0.0, 1.0), vec3(0.2627, 0.6780, 0.0593));
}

void main() {
    // Quick peek camera manufacturer JPEG (J key)
    if (u_isCameraJpeg != 0) {
        fragColor = texture(u_cameraJpegTexture, v_texCoord);
        return;
    }

    vec4 sampled = texture(u_texture, v_texCoord);
    
    // Quick peek original as-shot frame
    if (u_isOriginal != 0) {
        if (u_isLinearRec2020 != 0) {
            vec3 srgbLinearOut = agxToneMap(sampled.rgb, 1.0);
            fragColor = vec4(linearToSrgbGamma3(clamp(srgbLinearOut, 0.0, 1.0)), 1.0);
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

    // 2. Scene-Referred Exposure
    if (u_exposure != 0.0) {
        rec2020Color *= pow(2.0, u_exposure);
    }

    // 3. Tone Recovery in Linear Rec.2020
    rec2020Color = applyToneRecovery(rec2020Color, u_highlights, u_shadows, u_whites, u_blacks);

    // 4. Photochemical Recording Emulsion in Linear Rec.2020
    if (u_emulsionArchetype != 0) {
        rec2020Color = applyRecordingEmulsion(rec2020Color, u_emulsionArchetype, u_emulsionExposure);
    }

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

    // 8. Detail & Sharpening with Edge Masking in Linear Rec.2020
    if (u_sharpeningAmount > 0.0 || u_noiseReduction > 0.0) {
        vec2 texSize = vec2(textureSize(u_texture, 0));
        vec2 offset = vec2(max(u_sharpeningRadius, 0.5)) / texSize;

        float y_c = sampleDisplayLuma(v_texCoord);
        float y_l = sampleDisplayLuma(v_texCoord - vec2(offset.x, 0.0));
        float y_r = sampleDisplayLuma(v_texCoord + vec2(offset.x, 0.0));
        float y_t = sampleDisplayLuma(v_texCoord - vec2(0.0, offset.y));
        float y_b = sampleDisplayLuma(v_texCoord + vec2(0.0, offset.y));

        rec2020Color = applyDetailAndSharpening(
            rec2020Color,
            y_c, y_l, y_r, y_t, y_b,
            u_sharpeningAmount,
            u_sharpeningMasking,
            u_noiseReduction
        );
    }

    // 9. Fine-Art Print Media Chemistry in Linear Rec.2020
    if (u_printMediaMode != 0) {
        rec2020Color = applyPrintMedia(rec2020Color, u_printMediaMode, u_enlargerFilterMode, u_printPaperGrade, u_printBaseWarmth, u_printProcessDepth);
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

    // 11. Final Display Output: Display Gamma (The VERY LAST step!)
    fragColor = vec4(linearToSrgbGamma3(clamp(srgbLinearOut, 0.0, 1.0)), 1.0);
}
`;
