#include <metal_stdlib>
#include "ShaderTypes.h"
#include "ColorScience.metal"
#include "Geometry.metal"
#include "Detail.metal"

using namespace metal;

struct VertexInput {
    float2 position [[attribute(0)]];
    float2 texCoord [[attribute(1)]];
};

struct VertexOutput {
    float4 position [[position]];
    float2 texCoord;
};

// MARK: - Vertex Shader
vertex VertexOutput vertexShader(
    uint vertexID [[vertex_id]],
    constant VertexInput *vertices [[buffer(0)]],
    constant ShaderUniforms &uniforms [[buffer(1)]]
) {
    VertexOutput out;
    out.position = float4(vertices[vertexID].position, 0.0, 1.0);
    out.texCoord = vertices[vertexID].texCoord;
    return out;
}

// MARK: - Fragment Shader (Linear RAW Pipeline)
fragment float4 fragmentShader(
    VertexOutput in [[stage_in]],
    texture2d<float> texture [[texture(0)]],
    sampler textureSampler [[sampler(0)]],
    texture3d<float> lutTexture [[texture(1)]],
    sampler lutSampler [[sampler(1)]],
    texture1d<float> curveMasterTexture [[texture(2)]],
    texture1d<float> curveRgbTexture [[texture(3)]],
    sampler curveSampler [[sampler(2)]],
    texture2d<float> gainMapTexture [[texture(4)]],
    sampler gainMapSampler [[sampler(3)]],
    constant ShaderUniforms &uniforms [[buffer(1)]]
) {
    // 1. Geometry, Perspective & Crop Homography
    float2 uv = transformGeometry(in.texCoord, uniforms);
    
    // Discard / letterbox pixels outside valid texture bounds
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return float4(0.08, 0.08, 0.08, 1.0); // Neutral dark canvas background
    }
    
    // Sample texture
    float4 sampledColor = texture.sample(textureSampler, uv);
    
    // Fast-path: When displaying an already-rendered intermediate preview snapshot,
    // return the sampled color directly without re-applying tone mapping, curves, or color transforms.
    if (uniforms.isIntermediate == 1) {
        return sampledColor;
    }
    
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
                          (uniforms.splitToningShadowSat == 0.0) &&
                          (uniforms.splitToningHighlightSat == 0.0) &&
                          (uniforms.lutEnabled == 0) &&
                          (uniforms.sharpeningAmount == 0.0) &&
                          (uniforms.noiseReduction == 0.0);
    
    if (uniforms.isLinearRaw == 0 && isNeutralEdits && uniforms.hasGainMap == 0 && (uniforms.edrEnabled == 0 || uniforms.edrHeadroom <= 1.001)) {
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
    float editSplitShadowSat = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.splitToningShadowSat;
    float editSplitHighlightSat = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.splitToningHighlightSat;
    
    // 2. Decode texture to Linear Rec.2020 working space
    float3 rec2020Color;
    if (uniforms.isLinearRaw == 1) {
        rec2020Color = sampledColor.rgb;
    } else {
        float3 srgbLinear = srgbGammaToLinear3(sampledColor.rgb);
        rec2020Color = LINEAR_SRGB_TO_REC2020 * srgbLinear;
    }
    
    // 3. White Balance & Temperature Adjustment in Wide-Gamut Space
    float rScale = 1.0;
    float gScale = 1.0;
    float bScale = 1.0;
    if (abs(editTemp - 5500.0) >= 1.0 || editTint != 0.0) {
        float tempNorm = clamp(editTemp / 5500.0, 0.25, 4.0);
        rScale = pow(tempNorm, 0.75);
        bScale = pow(1.0 / tempNorm, 0.75);
        
        float tintFactor = editTint * 0.002;
        gScale = max(1.0 - tintFactor, 0.01);
        rScale = max(rScale + tintFactor * 0.5, 0.01);
        bScale = max(bScale + tintFactor * 0.5, 0.01);
        
        rec2020Color.r *= rScale;
        rec2020Color.g *= gScale;
        rec2020Color.b *= bScale;
    }
    
    // 4. Linear Exposure Multiplication (2^EV) in Wide-Gamut Space
    float totalExposure = editExposure;
    float exposureGain = 1.0;
    if (totalExposure != 0.0) {
        exposureGain = pow(2.0, totalExposure);
        rec2020Color *= exposureGain;
    }
    
    // 4b. Advanced Bilateral Base/Detail Tone Recovery (Highlights, Shadows, Whites, Blacks)
    if (editHighlights != 0.0 || editShadows != 0.0 || editWhites != 0.0 || editBlacks != 0.0) {
        float2 texelSize = 1.0 / float2(texture.get_width(), texture.get_height());
        float2 step = texelSize * 2.0; // 2-texel stride captures spatial base illumination
        
        // Sample 8 neighbors: 4 orthogonal + 4 diagonal
        float4 s_l  = texture.sample(textureSampler, uv + float2(-step.x, 0.0));
        float4 s_r  = texture.sample(textureSampler, uv + float2( step.x, 0.0));
        float4 s_t  = texture.sample(textureSampler, uv + float2(0.0, -step.y));
        float4 s_b  = texture.sample(textureSampler, uv + float2(0.0,  step.y));
        float4 s_tl = texture.sample(textureSampler, uv + float2(-step.x, -step.y));
        float4 s_tr = texture.sample(textureSampler, uv + float2( step.x, -step.y));
        float4 s_bl = texture.sample(textureSampler, uv + float2(-step.x,  step.y));
        float4 s_br = texture.sample(textureSampler, uv + float2( step.x,  step.y));
        
        float y_c = dot(rec2020Color, float3(0.2627, 0.6780, 0.0593));
        float3 wbExposureScale = float3(rScale, gScale, bScale) * exposureGain;
        
        float4 s_neighbors[8] = { s_l, s_r, s_t, s_b, s_tl, s_tr, s_bl, s_br };
        float y_neighbors[8];
        for (int i = 0; i < 8; ++i) {
            float3 c;
            if (uniforms.isLinearRaw == 1) {
                c = s_neighbors[i].rgb;
            } else {
                float3 srgbLinear = srgbGammaToLinear3(s_neighbors[i].rgb);
                c = LINEAR_SRGB_TO_REC2020 * srgbLinear;
            }
            c *= wbExposureScale;
            y_neighbors[i] = dot(c, float3(0.2627, 0.6780, 0.0593));
        }
        
        rec2020Color = applyBilateralToneRecovery(
            rec2020Color,
            y_c,
            y_neighbors,
            editHighlights,
            editShadows,
            editWhites,
            editBlacks
        );
    }
    
    // 4b. Upstream Recording Emulsion (DIR Coupling & Sensitometric D-log H Curves)
    if (uniforms.isOriginal < 0.5 && uniforms.emulsionArchetype != 0) {
        rec2020Color = applyRecordingEmulsion(
            rec2020Color,
            uniforms.emulsionArchetype,
            uniforms.emulsionExposure
        );
    }
    
    // 5. RGB Tone Curves (Master + R + G + B) in Linear Rec.2020 Working Space
    if (uniforms.hasCurves != 0 && uniforms.isOriginal < 0.5) {
        if (uniforms.hasMasterCurve != 0) {
            rec2020Color.r = curveMasterTexture.sample(curveSampler, clamp(rec2020Color.r, 0.0, 1.0)).r;
            rec2020Color.g = curveMasterTexture.sample(curveSampler, clamp(rec2020Color.g, 0.0, 1.0)).r;
            rec2020Color.b = curveMasterTexture.sample(curveSampler, clamp(rec2020Color.b, 0.0, 1.0)).r;
        }
        if (uniforms.hasRgbCurves != 0) {
            float3 rgbCurved = float3(
                curveRgbTexture.sample(curveSampler, clamp(rec2020Color.r, 0.0, 1.0)).r,
                curveRgbTexture.sample(curveSampler, clamp(rec2020Color.g, 0.0, 1.0)).g,
                curveRgbTexture.sample(curveSampler, clamp(rec2020Color.b, 0.0, 1.0)).b
            );
            rec2020Color = rgbCurved;
        }
    }

    // 6. Color & Vibrance Adjustments in Linear Rec.2020
    rec2020Color = applyColorAdjustments(rec2020Color, editVibrance, editSaturation, editDeepBlue, editFoliage);

    // 7. Split Toning & Color Grading with Optical Balance Anchoring in Linear Rec.2020
    rec2020Color = applySplitToning(
        rec2020Color,
        uniforms.splitToningShadowHue,
        editSplitShadowSat,
        uniforms.splitToningHighlightHue,
        editSplitHighlightSat,
        uniforms.splitToningBalance
    );

    // 8. Detail, Sharpening & Noise Reduction in Linear Rec.2020
    float editSharpening = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.sharpeningAmount;
    float editNoiseReduction = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.noiseReduction;
    if (editSharpening > 0.0 || editNoiseReduction > 0.0) {
        float2 texelSize = 1.0 / float2(texture.get_width(), texture.get_height());
        float2 offset = texelSize * max(uniforms.sharpeningRadius, 0.5);
        float4 leftSample   = texture.sample(textureSampler, uv - float2(offset.x, 0.0));
        float4 rightSample  = texture.sample(textureSampler, uv + float2(offset.x, 0.0));
        float4 topSample    = texture.sample(textureSampler, uv - float2(0.0, offset.y));
        float4 bottomSample = texture.sample(textureSampler, uv + float2(0.0, offset.y));

        float y_c = dot(rec2020Color, float3(0.2627, 0.6780, 0.0593));
        float y_l, y_r, y_t, y_b;
        if (uniforms.isLinearRaw == 0) {
            float3 cl = LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(leftSample.rgb);
            float3 cr = LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(rightSample.rgb);
            float3 ct = LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(topSample.rgb);
            float3 cb = LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(bottomSample.rgb);
            y_l = dot(cl, float3(0.2627, 0.6780, 0.0593));
            y_r = dot(cr, float3(0.2627, 0.6780, 0.0593));
            y_t = dot(ct, float3(0.2627, 0.6780, 0.0593));
            y_b = dot(cb, float3(0.2627, 0.6780, 0.0593));
        } else {
            float3 wbExposureScale = float3(rScale, gScale, bScale) * exposureGain;
            float3 cl = leftSample.rgb * wbExposureScale;
            float3 cr = rightSample.rgb * wbExposureScale;
            float3 ct = topSample.rgb * wbExposureScale;
            float3 cb = bottomSample.rgb * wbExposureScale;
            y_l = dot(cl, float3(0.2627, 0.6780, 0.0593));
            y_r = dot(cr, float3(0.2627, 0.6780, 0.0593));
            y_t = dot(ct, float3(0.2627, 0.6780, 0.0593));
            y_b = dot(cb, float3(0.2627, 0.6780, 0.0593));
        }

        rec2020Color = applyDetailAndSharpening(
            rec2020Color,
            y_c, y_l, y_r, y_t, y_b,
            editSharpening,
            uniforms.sharpeningMasking,
            editNoiseReduction
        );
    }

    // 8.2 Curated Fine-Art Print Media Archetypes & Optical Enlarger Filtration in Linear Rec.2020
    if (uniforms.isOriginal < 0.5 && uniforms.printMediaMode != 0) {
        rec2020Color = applyPrintMedia(
            rec2020Color,
            uniforms.printMediaMode,
            uniforms.enlargerFilterMode,
            uniforms.printPaperGrade,
            uniforms.printBaseWarmth,
            uniforms.printProcessDepth
        );
    }

    // 9. TERMINAL STAGE: AgX Filmic Tone Mapping Engine OR Camera-Matching Profiles -> Linear sRGB
    // Calibrated contrast: Ceiling 2.0x (+100), Floor 0.40x (-100) preventing flat gray collapse
    float contrastGain = (editContrast >= 0.0)
        ? (1.0 + editContrast / 100.0)
        : (1.0 + (editContrast / 100.0) * 0.60);
    float3 srgbLinearOut;
    if (uniforms.toneProfileMode == 0) {
        // Mode 0: AgX Vivid (Filmic Punchy) - DEFAULT
        srgbLinearOut = agxToneMapVivid(rec2020Color, contrastGain);
    } else if (uniforms.toneProfileMode == 1) {
        // Mode 1: AgX Neutral (Filmic Base)
        if (uniforms.isLinearRaw == 1 || editContrast != 0.0) {
            srgbLinearOut = agxToneMap(rec2020Color, contrastGain);
        } else {
            srgbLinearOut = max(REC2020_TO_LINEAR_SRGB * rec2020Color, float3(0.0));
        }
    } else {
        // Modes 2..5: Camera Matching (Standard, Portrait, Landscape, Vivid)
        int camMode = uniforms.toneProfileMode - 1; // 2->1 (Standard), 3->2 (Portrait), 4->3 (Landscape), 5->4 (Vivid)
        float3 c_srgb = max(REC2020_TO_LINEAR_SRGB * rec2020Color, float3(0.0));
        float3 rgbAdjusted = c_srgb;
        
        // Continuous sensitometric per-channel tone curve (Hurter-Driffield film D-log H model):
        // Evaluates each channel through the smooth C^1 camera spline.
        // Naturally self-limiting to [0.0, 1.0] and desaturates highlights to white along the shoulder without arbitrary thresholds.
        float3 curved = float3(
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

    float3 finalColor = srgbLinearOut;

    // 10. Creative 3D Look-Up Table (Post-Tone-Mapping in Linear sRGB)
    if (uniforms.lutEnabled != 0 && uniforms.lutIntensity > 0.0 && uniforms.isOriginal < 0.5) {
        float size = max(uniforms.lutSize, 2.0);
        float3 lutCoord = (finalColor * (size - 1.0) + 0.5) / size;
        float3 lutSample = lutTexture.sample(lutSampler, lutCoord).rgb;
        finalColor = mix(finalColor, lutSample, clamp(uniforms.lutIntensity, 0.0, 1.0));
    }
    
    // 9. Apple EDR Radiance & Dynamic Range Expansion
    if (uniforms.edrEnabled != 0 && uniforms.edrHeadroom > 1.001) {
        float headroom = clamp(uniforms.edrHeadroom, 1.0, 4.0);
        
        if (uniforms.hasGainMap != 0) {
            // Secondary ISO 21496-1 / Google Ultra HDR Gain Map
            float g = gainMapTexture.sample(gainMapSampler, uv).r;
            float gainLog2 = uniforms.gainMapMin + g * (uniforms.gainMapMax - uniforms.gainMapMin);
            float gamma = max(uniforms.gainMapGamma, 0.01);
            float gainFactor = pow(2.0, gainLog2 / gamma);
            
            float maxCap = max(uniforms.gainMapCapacityMax, 1.01);
            float weight = clamp((headroom - 1.0) / (maxCap - 1.0), 0.0, 1.0);
            float effectiveGain = 1.0 + (gainFactor - 1.0) * weight;
            finalColor *= effectiveGain;
        } else if (uniforms.isLinearRaw == 1) {
            // Scene-referred RAW dynamic range expansion
            // Sensor linear luminance before tone compression:
            float sceneLum = dot(rec2020Color, float3(0.2627, 0.6780, 0.0593));
            // Output tone-mapped luminance:
            float outLum = dot(finalColor, float3(0.2126, 0.7152, 0.0722));
            
            // Uncompress upper highlight shoulder (> 0.85) where AgX tone mapping compressed RAW sensor radiance.
            // Midtones and standard whites stay anchored at 1.0 SDR diffuse white.
            float highlightFactor = smoothstep(0.85, 1.0, max(outLum, clamp(sceneLum - 0.15, 0.0, 1.0)));
            if (highlightFactor > 0.0) {
                float edrBoost = 1.0 + (headroom - 1.0) * pow(highlightFactor, 1.5);
                finalColor *= edrBoost;
            }
        }
        
        // Preserve chromaticity on highlight limiting:
        // Proportional scaling prevents individual channel clipping (which causes ratty yellow highlights).
        // Any extreme specular radiance approaching or exceeding headroom smoothly desaturates towards specular white.
        float peakC = max(finalColor.r, max(finalColor.g, finalColor.b));
        if (peakC > headroom) {
            float excess = (peakC - headroom) / peakC;
            finalColor = mix(finalColor * (headroom / peakC), float3(headroom), clamp(excess * 1.5, 0.0, 1.0));
        } else if (peakC > 1.0) {
            // Soft specular desaturation in upper 20% of headroom
            float desatThreshold = 1.0 + (headroom - 1.0) * 0.8;
            if (peakC > desatThreshold) {
                float t = (peakC - desatThreshold) / max(headroom - desatThreshold, 1e-4);
                finalColor = mix(finalColor, float3(peakC), clamp(t * 0.4, 0.0, 0.4));
            }
        }
        
        // 10. Convert to Display P3 primaries for CAMetalLayer in extended linear Display P3
        finalColor = max(LINEAR_SRGB_TO_DISPLAY_P3 * finalColor, float3(0.0));
        return float4(finalColor, sampledColor.a);
    } else {
        // Standard Dynamic Range (SDR) / Thumbnail mode
        finalColor = clamp(finalColor, 0.0, 1.0);
        
        // Match standard display target transform for sRGB layer and thumbnail capture
        return float4(linearToSrgbGamma3(finalColor), sampledColor.a);
    }
}
