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
    
    if (uniforms.isLinearRaw == 0 && isNeutralEdits && uniforms.hasGainMap == 0) {
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
        
        float lum = dot(rgbAdjusted, float3(0.2126, 0.7152, 0.0722));
        float lumCurved = applyCameraToneCurve(lum, camMode);
        float contrastGain = 1.0 + (editContrast / 100.0);
        float lumAdj = clamp((lumCurved - 0.18) * contrastGain + 0.18, 0.0, 1.0);
        float gain = lumAdj / max(lum, 1e-6);
        
        srgbLinearOut = clamp(rgbAdjusted * gain, 0.0, 1.0);
    }
    
    // 6. Display Target Transform (Linear wide-gamut output for extended linear display)
    float3 finalColor = srgbLinearOut;
    
    // 6b. Color & Vibrance Adjustments
    finalColor = applyColorAdjustments(finalColor, editVibrance, editSaturation, editDeepBlue, editFoliage);
    
    // 6c. RGB Tone Curves (Master + R + G + B)
    if (uniforms.hasCurves != 0 && uniforms.isOriginal < 0.5) {
        if (uniforms.hasMasterCurve != 0) {
            finalColor.r = curveMasterTexture.sample(curveSampler, clamp(finalColor.r, 0.0, 1.0)).r;
            finalColor.g = curveMasterTexture.sample(curveSampler, clamp(finalColor.g, 0.0, 1.0)).r;
            finalColor.b = curveMasterTexture.sample(curveSampler, clamp(finalColor.b, 0.0, 1.0)).r;
        }
        if (uniforms.hasRgbCurves != 0) {
            float3 rgbCurved = float3(
                curveRgbTexture.sample(curveSampler, clamp(finalColor.r, 0.0, 1.0)).r,
                curveRgbTexture.sample(curveSampler, clamp(finalColor.g, 0.0, 1.0)).g,
                curveRgbTexture.sample(curveSampler, clamp(finalColor.b, 0.0, 1.0)).b
            );
            finalColor = rgbCurved;
        }
    }
    
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
    
    // 9. Apple EDR Radiance & Dynamic Range Expansion
    if (uniforms.edrEnabled != 0 && uniforms.edrHeadroom > 1.0) {
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
            finalColor = min(finalColor * effectiveGain, float3(headroom));
        } else {
            // Scene-referred RAW dynamic range expansion
            // Sensor linear luminance before tone compression:
            float sceneLum = dot(rec2020Color, float3(0.2627, 0.6780, 0.0593));
            // Output tone-mapped luminance:
            float outLum = dot(finalColor, float3(0.2126, 0.7152, 0.0722));
            
            // Smoothly transition highlights from SDR diffuse white into available EDR headroom.
            // Midtones (<= 0.55) remain completely untouched (SDR standard reference white),
            // while upper highlights (0.55..1.0) gracefully uncompress up to available display headroom.
            float highlightFactor = smoothstep(0.55, 1.0, max(outLum, sceneLum));
            if (highlightFactor > 0.0) {
                float edrBoost = 1.0 + (headroom - 1.0) * pow(highlightFactor, 1.5);
                finalColor = min(finalColor * edrBoost, float3(headroom));
            }
        }
        // 10. Convert to Display P3 primaries for CAMetalLayer in extended linear Display P3
        finalColor = max(LINEAR_SRGB_TO_DISPLAY_P3 * finalColor, float3(0.0));
        return float4(finalColor, sampledColor.a);
    } else {
        // Standard Dynamic Range (SDR) / Thumbnail mode
        finalColor = clamp(finalColor, 0.0, 1.0);
        
        // Match standard display target transform for sRGB layer and thumbnail capture
        if (uniforms.toneProfileMode <= 1) {
            return float4(finalColor, sampledColor.a);
        } else {
            return float4(linearToSrgbGamma3(finalColor), sampledColor.a);
        }
    }
}
