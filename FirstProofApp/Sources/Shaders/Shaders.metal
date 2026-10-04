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

// Interleaved Gradient Noise (Jorge Jimenez, Next Generation Post Processing)
// Extremely fast, zero texture lookups, optimal blue-noise-like properties for sample jitter and dithering
inline float ign(float2 p) {
    float3 magic = float3(0.06711056f, 0.00583715f, 52.9829189f);
    return fract(magic.z * fract(dot(p, magic.xy)));
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
                          (uniforms.noiseReduction == 0.0) &&
                          (uniforms.emulsionArchetype == 0) &&
                          (uniforms.halationMode == 0) &&
                          (uniforms.emulsionGrainMode == 0) &&
                          (uniforms.printMediaMode == 0);
    
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
    
    // 4b. Multi-Scale Hierarchical Bilateral Tone Recovery (Highlights, Shadows, Whites, Blacks)
    if (editHighlights != 0.0 || editShadows != 0.0 || editWhites != 0.0 || editBlacks != 0.0) {
        float2 texDim = float2(texture.get_width(), texture.get_height());
        float2 texelSize = 1.0 / texDim;
        float maxDim = max(texDim.x, texDim.y);
        float rInner = 1.0;
        float rOuter = max(16.0, round(maxDim * 0.016));
        float2 stepInner = texelSize * rInner;
        float2 stepOuter = texelSize * rOuter;
        
        float y_c = dot(rec2020Color, float3(0.2627, 0.6780, 0.0593));
        float3 wbExposureScale = float3(rScale, gScale, bScale) * exposureGain;
        
        // Inner ring: 4 orthogonal + 4 diagonal (rInner = 1 texel)
        float4 s_l  = texture.sample(textureSampler, uv + float2(-stepInner.x, 0.0));
        float4 s_r  = texture.sample(textureSampler, uv + float2( stepInner.x, 0.0));
        float4 s_t  = texture.sample(textureSampler, uv + float2(0.0, -stepInner.y));
        float4 s_b  = texture.sample(textureSampler, uv + float2(0.0,  stepInner.y));
        float4 s_tl = texture.sample(textureSampler, uv + float2(-stepInner.x, -stepInner.y));
        float4 s_tr = texture.sample(textureSampler, uv + float2( stepInner.x, -stepInner.y));
        float4 s_bl = texture.sample(textureSampler, uv + float2(-stepInner.x,  stepInner.y));
        float4 s_br = texture.sample(textureSampler, uv + float2( stepInner.x,  stepInner.y));

        // Outer ring: 4 orthogonal + 4 diagonal (rOuter)
        float4 s_ol  = texture.sample(textureSampler, uv + float2(-stepOuter.x, 0.0));
        float4 s_or  = texture.sample(textureSampler, uv + float2( stepOuter.x, 0.0));
        float4 s_ot  = texture.sample(textureSampler, uv + float2(0.0, -stepOuter.y));
        float4 s_ob  = texture.sample(textureSampler, uv + float2(0.0,  stepOuter.y));
        float4 s_otl = texture.sample(textureSampler, uv + float2(-stepOuter.x, -stepOuter.y));
        float4 s_otr = texture.sample(textureSampler, uv + float2( stepOuter.x, -stepOuter.y));
        float4 s_obl = texture.sample(textureSampler, uv + float2(-stepOuter.x,  stepOuter.y));
        float4 s_obr = texture.sample(textureSampler, uv + float2( stepOuter.x,  stepOuter.y));
        
        float4 s_neighbors[16] = {
            s_l, s_r, s_t, s_b, s_tl, s_tr, s_bl, s_br,
            s_ol, s_or, s_ot, s_ob, s_otl, s_otr, s_obl, s_obr
        };
        float y_neighbors[16];
        for (int i = 0; i < 16; ++i) {
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
            uniforms.emulsionExposure,
            uniforms.enlargerFilterMode
        );
    }
    
    // 4c. Physical Optical Halation Scatter (Point-Spread Emulsion Diffusion)
    if (uniforms.isOriginal < 0.5 && uniforms.halationMode > 0 && uniforms.emulsionArchetype != 7) {
        float formatMag = 1.0f;
        if (uniforms.emulsionFormat == 1) formatMag = 0.48f;
        else if (uniforms.emulsionFormat == 2) formatMag = 0.28f;
        else if (uniforms.emulsionFormat == 3) formatMag = 7.3f;
        else if (uniforms.emulsionFormat == 4) formatMag = 3.4f;

        float aspect = max(uniforms.aspectRatio, 0.01f);
        float2 halationRadius;
        if (uniforms.rotation == 90 || uniforms.rotation == 270) {
            halationRadius = float2(0.012f * aspect * formatMag, 0.012f * formatMag);
        } else {
            halationRadius = float2((0.012f / aspect) * formatMag, 0.012f * formatMag);
        }
        float3 wbExposure = float3(rScale, gScale, bScale) * exposureGain;
        
        // Stochastic rotation per fragment via Interleaved Gradient Noise
        // Smooths out discrete Poisson disk tap rings into a continuous, band-free spatial point-spread function
        float jitter = ign(in.position.xy) * 6.2831853f;
        float cosJ = cos(jitter);
        float sinJ = sin(jitter);
        float2x2 rot = float2x2(float2(cosJ, sinJ), float2(-sinJ, cosJ));

        // 16-tap Fermat golden spiral with exponential distance weighting
        constexpr float2 fermatDisk[16] = {
            float2( 0.1768,  0.0000), float2(-0.2258,  0.2068), float2( 0.0346, -0.3938), float2( 0.2846,  0.3712),
            float2(-0.5222, -0.0924), float2( 0.4947, -0.3147), float2(-0.1655,  0.6155), float2(-0.3156, -0.6076),
            float2( 0.6846,  0.2500), float2(-0.7123,  0.2940), float2( 0.3434, -0.7337), float2( 0.2537,  0.8089),
            float2(-0.7647, -0.4432), float2( 0.8971, -0.1972), float2(-0.5475,  0.7788), float2(-0.1265, -0.9761)
        };
        constexpr float fermatWeights[16] = {
            0.6428, 0.4651, 0.3722, 0.3106,
            0.2656, 0.2309, 0.2032, 0.1806,
            0.1617, 0.1457, 0.1320, 0.1201,
            0.1097, 0.1006, 0.0926, 0.0854
        };
        
        float scatterAcc = 0.0;
        for (int i = 0; i < 16; ++i) {
            float2 rotatedOffset = (rot * fermatDisk[i]) * halationRadius;
            float4 s = texture.sample(textureSampler, uv + rotatedOffset);
            float3 c;
            if (uniforms.isLinearRaw == 1) {
                c = s.rgb;
            } else {
                float3 srgbLinear = srgbGammaToLinear3(s.rgb);
                c = LINEAR_SRGB_TO_REC2020 * srgbLinear;
            }
            c *= wbExposure;
            float y_tap = dot(c, float3(0.2627, 0.6780, 0.0593));
            if (y_tap > 0.55f) {
                float diff = y_tap - 0.55f;
                float excess = (diff * diff) / (0.25f + diff);
                scatterAcc += excess * fermatWeights[i];
            }
        }
        
        float halo = scatterAcc / 3.6187f;
        if (halo > 1e-5f) {
            if (uniforms.halationMode == 1) {
                // Baoding Silver 100: Neutral panchromatic silver glow
                rec2020Color += float3(halo * 0.35f);
            } else if (uniforms.halationMode == 2) {
                // Rochester Cinema 500 (Stripped Rem-Jet / CineStill): Red-shifted halation
                rec2020Color += float3(halo * 0.50f, halo * 0.12f, halo * 0.02f);
            }
        }
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

    // 8. Detail, Sharpening, Noise Reduction & Stochastic Emulsion Resynthesis in Linear Rec.2020
    float editSharpening = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.sharpeningAmount;
    float editNoiseReduction = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.noiseReduction;
    int editGrainMode = (uniforms.isOriginal > 0.5) ? 0 : uniforms.emulsionGrainMode;
    if (editSharpening > 0.0 || editNoiseReduction > 0.0 || editGrainMode != 0) {
        float formatMag = 1.0f;
        if (uniforms.emulsionFormat == 1) formatMag = 0.48f;
        else if (uniforms.emulsionFormat == 2) formatMag = 0.28f;
        else if (uniforms.emulsionFormat == 3) formatMag = 7.3f;
        else if (uniforms.emulsionFormat == 4) formatMag = 3.4f;

        float2 texSize = float2(texture.get_width(), texture.get_height());
        float2 texelSize = 1.0 / texSize;
        float2 pixelCoord = uv * texSize;
        float y_c = dot(rec2020Color, float3(0.2627, 0.6780, 0.0593));
        float3 wbExposureScale = float3(rScale, gScale, bScale) * exposureGain;

        if (editSharpening > 0.0 || editNoiseReduction > 0.0) {
            float effectiveRadius = max(uniforms.sharpeningRadius * uniforms.baselineSharpness, 0.5f);
            float2 sharpenOffset = texelSize * effectiveRadius;
            float4 leftSample   = texture.sample(textureSampler, uv - float2(sharpenOffset.x, 0.0));
            float4 rightSample  = texture.sample(textureSampler, uv + float2(sharpenOffset.x, 0.0));
            float4 topSample    = texture.sample(textureSampler, uv - float2(0.0, sharpenOffset.y));
            float4 bottomSample = texture.sample(textureSampler, uv + float2(0.0, sharpenOffset.y));
            float4 tlSample     = texture.sample(textureSampler, uv - sharpenOffset);
            float4 trSample     = texture.sample(textureSampler, uv + float2(sharpenOffset.x, -sharpenOffset.y));
            float4 blSample     = texture.sample(textureSampler, uv + float2(-sharpenOffset.x, sharpenOffset.y));
            float4 brSample     = texture.sample(textureSampler, uv + sharpenOffset);

            float y_l, y_r, y_t, y_b, y_tl, y_tr, y_bl, y_br;
            if (uniforms.isLinearRaw == 0) {
                y_l  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(leftSample.rgb),   float3(0.2627, 0.6780, 0.0593));
                y_r  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(rightSample.rgb),  float3(0.2627, 0.6780, 0.0593));
                y_t  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(topSample.rgb),    float3(0.2627, 0.6780, 0.0593));
                y_b  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(bottomSample.rgb), float3(0.2627, 0.6780, 0.0593));
                y_tl = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(tlSample.rgb),     float3(0.2627, 0.6780, 0.0593));
                y_tr = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(trSample.rgb),     float3(0.2627, 0.6780, 0.0593));
                y_bl = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(blSample.rgb),     float3(0.2627, 0.6780, 0.0593));
                y_br = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(brSample.rgb),     float3(0.2627, 0.6780, 0.0593));
            } else {
                y_l  = dot(leftSample.rgb   * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
                y_r  = dot(rightSample.rgb  * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
                y_t  = dot(topSample.rgb    * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
                y_b  = dot(bottomSample.rgb * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
                y_tl = dot(tlSample.rgb     * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
                y_tr = dot(trSample.rgb     * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
                y_bl = dot(blSample.rgb     * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
                y_br = dot(brSample.rgb     * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
            }

            rec2020Color = applyDetailAndSharpening(
                rec2020Color,
                y_c, y_l, y_r, y_t, y_b, y_tl, y_tr, y_bl, y_br,
                editSharpening,
                uniforms.sharpeningMasking,
                editNoiseReduction
            );
        }

        if (editGrainMode != 0) {
            float2 grainOffset = texelSize;
            float4 leftSample   = texture.sample(textureSampler, uv - float2(grainOffset.x, 0.0));
            float4 rightSample  = texture.sample(textureSampler, uv + float2(grainOffset.x, 0.0));
            float4 topSample    = texture.sample(textureSampler, uv - float2(0.0, grainOffset.y));
            float4 bottomSample = texture.sample(textureSampler, uv + float2(0.0, grainOffset.y));
            float4 tlSample     = texture.sample(textureSampler, uv - grainOffset);
            float4 trSample     = texture.sample(textureSampler, uv + float2(grainOffset.x, -grainOffset.y));
            float4 blSample     = texture.sample(textureSampler, uv + float2(-grainOffset.x, grainOffset.y));
            float4 brSample     = texture.sample(textureSampler, uv + grainOffset);

            float gy_l, gy_r, gy_t, gy_b, gy_tl, gy_tr, gy_bl, gy_br;
            if (uniforms.isLinearRaw == 0) {
                gy_l  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(leftSample.rgb),   float3(0.2627, 0.6780, 0.0593));
                gy_r  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(rightSample.rgb),  float3(0.2627, 0.6780, 0.0593));
                gy_t  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(topSample.rgb),    float3(0.2627, 0.6780, 0.0593));
                gy_b  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(bottomSample.rgb), float3(0.2627, 0.6780, 0.0593));
                gy_tl = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(tlSample.rgb),     float3(0.2627, 0.6780, 0.0593));
                gy_tr = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(trSample.rgb),     float3(0.2627, 0.6780, 0.0593));
                gy_bl = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(blSample.rgb),     float3(0.2627, 0.6780, 0.0593));
                gy_br = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(brSample.rgb),     float3(0.2627, 0.6780, 0.0593));
            } else {
                gy_l  = dot(leftSample.rgb   * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
                gy_r  = dot(rightSample.rgb  * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
                gy_t  = dot(topSample.rgb    * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
                gy_b  = dot(bottomSample.rgb * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
                gy_tl = dot(tlSample.rgb     * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
                gy_tr = dot(trSample.rgb     * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
                gy_bl = dot(blSample.rgb     * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
                gy_br = dot(brSample.rgb     * wbExposureScale, float3(0.2627, 0.6780, 0.0593));
            }

            float s_format = sqrt(max(formatMag, 0.1f));
            float clusterSize = clamp(uniforms.grainSize * s_format, 0.5f, 3.5f);
            float effectiveAmount = uniforms.grainAmount * pow(max(formatMag, 0.1f), 0.25f);

            rec2020Color = applyStochasticEmulsionGrain(
                rec2020Color,
                y_c, gy_l, gy_r, gy_t, gy_b, gy_tl, gy_tr, gy_bl, gy_br,
                pixelCoord,
                editGrainMode,
                clusterSize,
                effectiveAmount,
                uniforms.emulsionArchetype
            );
        }
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

    // 9.1 Curated Fine-Art Print Media Archetypes & Optical Enlarger Filtration in Linear Display Space
    if (uniforms.isOriginal < 0.5 && uniforms.printMediaMode != 0) {
        srgbLinearOut = applyPrintMedia(
            srgbLinearOut,
            uniforms.printMediaMode,
            uniforms.enlargerFilterMode,
            uniforms.printPaperGrade,
            uniforms.printBaseWarmth,
            uniforms.printProcessDepth
        );
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
        
        // Match standard display target transform for sRGB layer and thumbnail capture with 8-bit TPDF dither
        float dither = (ign(in.position.xy) - 0.5f) / 255.0f;
        return float4(clamp(linearToSrgbGamma3(finalColor) + float3(dither), 0.0f, 1.0f), sampledColor.a);
    }
}
