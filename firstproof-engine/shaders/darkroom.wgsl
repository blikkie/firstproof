// Canonical Darkroom Vertex & Fragment Pipeline (Linear Rec.2020 Working Domain)

// MARK: - Resource Bindings
@group(0) @binding(0) var<uniform> uniforms: ShaderUniforms;
@group(0) @binding(1) var u_texture: texture_2d<f32>;
@group(0) @binding(2) var u_sampler: sampler;
@group(0) @binding(3) var u_lut_3d: texture_3d<f32>;
@group(0) @binding(4) var u_lut_sampler: sampler;
@group(0) @binding(5) var u_curve_master_lut: texture_1d<f32>;
@group(0) @binding(6) var u_curve_rgb_lut: texture_1d<f32>;
@group(0) @binding(7) var u_curve_sampler: sampler;
@group(0) @binding(8) var u_gain_map: texture_2d<f32>;
@group(0) @binding(9) var u_gain_map_sampler: sampler;
@group(0) @binding(10) var<uniform> quad: VertexQuad;
@group(0) @binding(11) var u_proof_lut: texture_3d<f32>;
@group(0) @binding(12) var u_proof_sampler: sampler;

@vertex
fn vertexShader(@builtin(vertex_index) vertex_id: u32) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(quad.vertices[vertex_id].position, 0.0, 1.0);
    out.texCoord = quad.vertices[vertex_id].texCoord;
    return out;
}



// Interleaved Gradient Noise (Jorge Jimenez, Next Generation Post Processing 2014)
fn ign(p: vec2<f32>) -> f32 {
    let ignCoeffs = vec3<f32>(0.06711056, 0.00583715, 52.9829189);
    return fract(ignCoeffs.z * fract(dot(p, ignCoeffs.xy)));
}

// MARK: - Fragment Shader (Linear RAW Pipeline)
@fragment
fn fragmentShader(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Geometry, Perspective & Crop Homography
    let coords = transformGeometryChannels(in.texCoord, uniforms);
    let uv = coords.uv;
    
    // Discard / letterbox pixels outside valid texture bounds
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return vec4<f32>(0.08, 0.08, 0.08, 1.0); // Neutral dark canvas background
    }
    
    // Sample texture (with per-channel TCA realignment when active)
    var sampledColor: vec4<f32>;
    if (uniforms.tcaCorrectionEnabled != 0 && uniforms.hasTca != 0) {
        let sampleG = textureSample(u_texture, u_sampler, uv);
        let sampleR = textureSample(u_texture, u_sampler, coords.uvRed);
        let sampleB = textureSample(u_texture, u_sampler, coords.uvBlue);
        sampledColor = vec4<f32>(sampleR.r, sampleG.g, sampleB.b, sampleG.a);
    } else {
        sampledColor = textureSample(u_texture, u_sampler, uv);
    }
    
    // Fast-path: When displaying an already-rendered intermediate preview snapshot,
    // return the sampled color directly without re-applying tone mapping, curves, or color transforms.
    if (uniforms.isIntermediate == 1) {
        return sampledColor;
    }
    
    // If user is holding backslash for original as-shot comparison:
    if (uniforms.isOriginal > 0.5 && uniforms.isLinearRaw == 0) {
        return sampledColor;
    }
    
    // Fast-path: When all edit sliders are at default neutral positions for JPEG thumbnails, return exact as-shot pixels.
    let isNeutralEdits = (abs(uniforms.exposure) < 0.001) && 
                          (abs(uniforms.temperature - 5500.0) < 1.0) && 
                          (abs(uniforms.tint) < 0.001) && 
                          (abs(uniforms.contrast) < 0.001) &&
                          (abs(uniforms.highlights) < 0.001) &&
                          (abs(uniforms.shadows) < 0.001) &&
                          (abs(uniforms.whites) < 0.001) &&
                          (abs(uniforms.blacks) < 0.001) &&
                          (abs(uniforms.vibrance) < 0.001) &&
                          (abs(uniforms.saturation) < 0.001) &&
                          (abs(uniforms.deepBlue) < 0.001) &&
                          (abs(uniforms.foliage) < 0.001) && 
                          (abs(uniforms.splitToningShadowSat) < 0.001) &&
                          (abs(uniforms.splitToningHighlightSat) < 0.001) &&
                          (uniforms.lutEnabled == 0) &&
                          (uniforms.sharpeningAmount == 0.0) &&
                          (uniforms.noiseReduction == 0.0) &&
                          (uniforms.defringeAmount == 0.0) &&
                          (uniforms.emulsionArchetype == 0) &&
                          (uniforms.halationMode == 0) &&
                          (uniforms.emulsionGrainMode == 0) &&
                          (uniforms.printMediaMode == 0) &&
                          (uniforms.vignettingCorrectionEnabled == 0 || uniforms.hasVignetting == 0);
    
    if (uniforms.isLinearRaw == 0 && isNeutralEdits && uniforms.hasGainMap == 0 && (uniforms.edrEnabled == 0 || uniforms.edrHeadroom <= 1.001)) {
        return sampledColor;
    }
    
    var editExposure = uniforms.exposure;
    var editTemp = uniforms.temperature;
    var editTint = uniforms.tint;
    var editContrast = uniforms.contrast;
    var editHighlights = uniforms.highlights;
    var editShadows = uniforms.shadows;
    var editWhites = uniforms.whites;
    var editBlacks = uniforms.blacks;
    var editVibrance = uniforms.vibrance;
    var editSaturation = uniforms.saturation;
    var editDeepBlue = uniforms.deepBlue;
    var editFoliage = uniforms.foliage;
    var editSplitShadowSat = uniforms.splitToningShadowSat;
    var editSplitHighlightSat = uniforms.splitToningHighlightSat;

    if (uniforms.isOriginal > 0.5) {
        editExposure = 0.0;
        editTemp = 5500.0;
        editTint = 0.0;
        editContrast = 0.0;
        editHighlights = 0.0;
        editShadows = 0.0;
        editWhites = 0.0;
        editBlacks = 0.0;
        editVibrance = 0.0;
        editSaturation = 0.0;
        editDeepBlue = 0.0;
        editFoliage = 0.0;
        editSplitShadowSat = 0.0;
        editSplitHighlightSat = 0.0;
    }
    
    // 2. Decode texture to Linear Rec.2020 working space
    var rec2020Color: vec3<f32>;
    if (uniforms.isLinearRaw == 1) {
        rec2020Color = sampledColor.rgb;
    } else {
        let srgbLinear = srgbGammaToLinear3(sampledColor.rgb);
        rec2020Color = LINEAR_SRGB_TO_REC2020 * srgbLinear;
    }
    
    // 3. White Balance & Temperature Adjustment in Wide-Gamut Space
    var rScale: f32 = 1.0;
    var gScale: f32 = 1.0;
    var bScale: f32 = 1.0;
    var baseTemp: f32 = 5500.0;
    if (uniforms.isLinearRaw == 1 && uniforms.asShotTemperature > 1000.0) {
        baseTemp = uniforms.asShotTemperature;
    }
    var baseTint: f32 = 0.0;
    if (uniforms.isLinearRaw == 1) {
        baseTint = uniforms.asShotTint;
    }

    if (abs(editTemp - baseTemp) >= 1.0 || abs(editTint - baseTint) >= 0.1) {
        let tempNorm = clamp(editTemp / baseTemp, 0.25, 4.0);
        rScale = pow(tempNorm, 0.75);
        bScale = pow(1.0 / tempNorm, 0.75);
        
        let tintFactor = (editTint - baseTint) * 0.002;
        gScale = max(1.0 - tintFactor, 0.01);
        rScale = max(rScale + tintFactor * 0.5, 0.01);
        bScale = max(bScale + tintFactor * 0.5, 0.01);
        
        rec2020Color.r = rec2020Color.r * rScale;
        rec2020Color.g = rec2020Color.g * gScale;
        rec2020Color.b = rec2020Color.b * bScale;
    }

    // 3b. Lens Vignetting Radial Illumination Gain in Linear Rec.2020 Space
    if (uniforms.vignettingCorrectionEnabled != 0 && uniforms.hasVignetting != 0) {
        let A = max(uniforms.aspectRatio, 0.01);
        let xc = (uv.x - 0.5) * 2.0 * A;
        let yc = (uv.y - 0.5) * 2.0;
        let r2 = (xc * xc + yc * yc) / (A * A + 1.0);
        let r4 = r2 * r2;
        let r6 = r4 * r2;
        let r8 = r4 * r4;
        let poly = 1.0 + uniforms.vignettingK1 * r2 + uniforms.vignettingK2 * r4 + uniforms.vignettingK3 * r6 + uniforms.vignettingK4 * r8;
        var vigGain: f32 = 1.0;
        if (poly < 1.0) {
            vigGain = 1.0 / max(poly, 0.05);
        } else {
            vigGain = poly;
        }
        rec2020Color = rec2020Color * vigGain;
    }
    
    // 4. Linear Exposure Multiplication (2^EV) in Wide-Gamut Space
    let totalExposure = editExposure;
    var exposureGain: f32 = 1.0;
    if (totalExposure != 0.0) {
        exposureGain = pow(2.0, totalExposure);
        rec2020Color = rec2020Color * exposureGain;
    }
    
    // 4b. Multi-Scale Hierarchical Bilateral Tone Recovery (Highlights, Shadows, Whites, Blacks)
    if (editHighlights != 0.0 || editShadows != 0.0 || editWhites != 0.0 || editBlacks != 0.0) {
        let texDim = vec2<f32>(textureDimensions(u_texture));
        let texelSize = vec2<f32>(1.0) / texDim;
        let maxDim = max(texDim.x, texDim.y);
        let rInner = max(3.0, round(maxDim * 0.003));
        let rOuter = max(24.0, round(maxDim * 0.024));
        let stepInner = texelSize * rInner;
        let stepOuter = texelSize * rOuter;
        
        let y_c = dot(rec2020Color, vec3<f32>(0.2627, 0.6780, 0.0593));
        let wbExposureScale = vec3<f32>(rScale, gScale, bScale) * exposureGain;
        
        // Inner ring: 4 orthogonal + 4 diagonal
        let s_l  = textureSample(u_texture, u_sampler, uv + vec2<f32>(-stepInner.x, 0.0));
        let s_r  = textureSample(u_texture, u_sampler, uv + vec2<f32>( stepInner.x, 0.0));
        let s_t  = textureSample(u_texture, u_sampler, uv + vec2<f32>(0.0, -stepInner.y));
        let s_b  = textureSample(u_texture, u_sampler, uv + vec2<f32>(0.0,  stepInner.y));
        let s_tl = textureSample(u_texture, u_sampler, uv + vec2<f32>(-stepInner.x, -stepInner.y));
        let s_tr = textureSample(u_texture, u_sampler, uv + vec2<f32>( stepInner.x, -stepInner.y));
        let s_bl = textureSample(u_texture, u_sampler, uv + vec2<f32>(-stepInner.x,  stepInner.y));
        let s_br = textureSample(u_texture, u_sampler, uv + vec2<f32>( stepInner.x,  stepInner.y));

        // Outer ring: 4 orthogonal + 4 diagonal
        let s_ol  = textureSample(u_texture, u_sampler, uv + vec2<f32>(-stepOuter.x, 0.0));
        let s_or  = textureSample(u_texture, u_sampler, uv + vec2<f32>( stepOuter.x, 0.0));
        let s_ot  = textureSample(u_texture, u_sampler, uv + vec2<f32>(0.0, -stepOuter.y));
        let s_ob  = textureSample(u_texture, u_sampler, uv + vec2<f32>(0.0,  stepOuter.y));
        let s_otl = textureSample(u_texture, u_sampler, uv + vec2<f32>(-stepOuter.x, -stepOuter.y));
        let s_otr = textureSample(u_texture, u_sampler, uv + vec2<f32>( stepOuter.x, -stepOuter.y));
        let s_obl = textureSample(u_texture, u_sampler, uv + vec2<f32>(-stepOuter.x,  stepOuter.y));
        let s_obr = textureSample(u_texture, u_sampler, uv + vec2<f32>( stepOuter.x,  stepOuter.y));
        
        let s_neighbors = array<vec4<f32>, 16>(
            s_l, s_r, s_t, s_b, s_tl, s_tr, s_bl, s_br,
            s_ol, s_or, s_ot, s_ob, s_otl, s_otr, s_obl, s_obr
        );
        var y_neighbors = array<f32, 16>(
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
        );
        for (var i: i32 = 0; i < 16; i = i + 1) {
            var c: vec3<f32>;
            if (uniforms.isLinearRaw == 1) {
                c = s_neighbors[i].rgb;
            } else {
                let srgbLinear = srgbGammaToLinear3(s_neighbors[i].rgb);
                c = LINEAR_SRGB_TO_REC2020 * srgbLinear;
            }
            c = c * wbExposureScale;
            y_neighbors[i] = dot(c, vec3<f32>(0.2627, 0.6780, 0.0593));
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
        var formatMag: f32 = 1.0;
        if (uniforms.emulsionFormat == 1) { formatMag = 0.48; }
        else if (uniforms.emulsionFormat == 2) { formatMag = 0.28; }
        else if (uniforms.emulsionFormat == 3) { formatMag = 7.3; }
        else if (uniforms.emulsionFormat == 4) { formatMag = 3.4; }

        let aspect = max(uniforms.aspectRatio, 0.01);
        var halationRadius: vec2<f32>;
        if (uniforms.rotation == 90 || uniforms.rotation == 270) {
            halationRadius = vec2<f32>(0.012 * aspect * formatMag, 0.012 * formatMag);
        } else {
            halationRadius = vec2<f32>((0.012 / aspect) * formatMag, 0.012 * formatMag);
        }
        let wbExposure = vec3<f32>(rScale, gScale, bScale) * exposureGain;
        
        let jitter = ign(in.position.xy) * 6.2831853;
        let cosJ = cos(jitter);
        let sinJ = sin(jitter);
        let rot = mat2x2<f32>(vec2<f32>(cosJ, sinJ), vec2<f32>(-sinJ, cosJ));

        var scatterAcc: f32 = 0.0;
        for (var i: i32 = 0; i < 16; i = i + 1) {
            let rotatedOffset = (rot * FERMAT_DISK[i]) * halationRadius;
            let s = textureSample(u_texture, u_sampler, uv + rotatedOffset);
            var c: vec3<f32>;
            if (uniforms.isLinearRaw == 1) {
                c = s.rgb;
            } else {
                let srgbLinear = srgbGammaToLinear3(s.rgb);
                c = LINEAR_SRGB_TO_REC2020 * srgbLinear;
            }
            c = c * wbExposure;
            let y_tap = dot(c, vec3<f32>(0.2627, 0.6780, 0.0593));
            if (y_tap > 0.55) {
                let diff = y_tap - 0.55;
                let excess = (diff * diff) / (0.25 + diff);
                scatterAcc = scatterAcc + excess * FERMAT_WEIGHTS[i];
            }
        }
        
        let halo = scatterAcc / 3.6187;
        if (halo > 1e-5) {
            if (uniforms.halationMode == 1) {
                rec2020Color = rec2020Color + vec3<f32>(halo * 0.35);
            } else if (uniforms.halationMode == 2) {
                rec2020Color = rec2020Color + vec3<f32>(halo * 0.50, halo * 0.12, halo * 0.02);
            }
        }
    }
    
    // 5. RGB Tone Curves (Master + R + G + B) in Linear Rec.2020 Working Space
    if (uniforms.hasCurves != 0 && uniforms.isOriginal < 0.5) {
        if (uniforms.hasMasterCurve != 0) {
            rec2020Color.r = textureSample(u_curve_master_lut, u_curve_sampler, clamp(rec2020Color.r, 0.0, 1.0)).r;
            rec2020Color.g = textureSample(u_curve_master_lut, u_curve_sampler, clamp(rec2020Color.g, 0.0, 1.0)).r;
            rec2020Color.b = textureSample(u_curve_master_lut, u_curve_sampler, clamp(rec2020Color.b, 0.0, 1.0)).r;
        }
        if (uniforms.hasRgbCurves != 0) {
            rec2020Color = vec3<f32>(
                textureSample(u_curve_rgb_lut, u_curve_sampler, clamp(rec2020Color.r, 0.0, 1.0)).r,
                textureSample(u_curve_rgb_lut, u_curve_sampler, clamp(rec2020Color.g, 0.0, 1.0)).g,
                textureSample(u_curve_rgb_lut, u_curve_sampler, clamp(rec2020Color.b, 0.0, 1.0)).b
            );
        }
    }

    // 6. Color & Vibrance Adjustments in Linear Rec.2020
    rec2020Color = applyColorAdjustments(rec2020Color, editVibrance, editSaturation, editDeepBlue, editFoliage);

    // 7. Split Toning & Color Grading with Optical Balance Anchoring
    rec2020Color = applySplitToning(
        rec2020Color,
        uniforms.splitToningShadowHue,
        editSplitShadowSat,
        uniforms.splitToningHighlightHue,
        editSplitHighlightSat,
        uniforms.splitToningBalance
    );

    // 8. Detail, Sharpening, Noise Reduction & Stochastic Emulsion Resynthesis
    var editSharpening = uniforms.sharpeningAmount;
    var editNoiseReduction = uniforms.noiseReduction;
    var editGrainMode = uniforms.emulsionGrainMode;
    if (uniforms.isOriginal > 0.5) {
        editSharpening = 0.0;
        editNoiseReduction = 0.0;
        editGrainMode = 0;
    }

    if (editSharpening > 0.0 || editNoiseReduction > 0.0 || editGrainMode != 0 || uniforms.defringeAmount > 0.0) {
        var formatMag: f32 = 1.0;
        if (uniforms.emulsionFormat == 1) { formatMag = 0.48; }
        else if (uniforms.emulsionFormat == 2) { formatMag = 0.28; }
        else if (uniforms.emulsionFormat == 3) { formatMag = 7.3; }
        else if (uniforms.emulsionFormat == 4) { formatMag = 3.4; }

        let texSize = vec2<f32>(textureDimensions(u_texture));
        let texelSize = vec2<f32>(1.0) / texSize;
        let pixelCoord = uv * texSize;
        let y_c = dot(rec2020Color, vec3<f32>(0.2627, 0.6780, 0.0593));
        let wbExposureScale = vec3<f32>(rScale, gScale, bScale) * exposureGain;

        if (editSharpening > 0.0 || editNoiseReduction > 0.0 || uniforms.defringeAmount > 0.0) {
            let effectiveRadius = max(uniforms.sharpeningRadius * uniforms.baselineSharpness, 0.5);
            let sharpenOffset = texelSize * effectiveRadius;
            let leftSample   = textureSample(u_texture, u_sampler, uv - vec2<f32>(sharpenOffset.x, 0.0));
            let rightSample  = textureSample(u_texture, u_sampler, uv + vec2<f32>(sharpenOffset.x, 0.0));
            let topSample    = textureSample(u_texture, u_sampler, uv - vec2<f32>(0.0, sharpenOffset.y));
            let bottomSample = textureSample(u_texture, u_sampler, uv + vec2<f32>(0.0, sharpenOffset.y));
            let tlSample     = textureSample(u_texture, u_sampler, uv - sharpenOffset);
            let trSample     = textureSample(u_texture, u_sampler, uv + vec2<f32>(sharpenOffset.x, -sharpenOffset.y));
            let blSample     = textureSample(u_texture, u_sampler, uv + vec2<f32>(-sharpenOffset.x, sharpenOffset.y));
            let brSample     = textureSample(u_texture, u_sampler, uv + sharpenOffset);

            var y_l: f32; var y_r: f32; var y_t: f32; var y_b: f32;
            var y_tl: f32; var y_tr: f32; var y_bl: f32; var y_br: f32;
            if (uniforms.isLinearRaw == 0) {
                y_l  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(leftSample.rgb),   vec3<f32>(0.2627, 0.6780, 0.0593));
                y_r  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(rightSample.rgb),  vec3<f32>(0.2627, 0.6780, 0.0593));
                y_t  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(topSample.rgb),    vec3<f32>(0.2627, 0.6780, 0.0593));
                y_b  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(bottomSample.rgb), vec3<f32>(0.2627, 0.6780, 0.0593));
                y_tl = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(tlSample.rgb),     vec3<f32>(0.2627, 0.6780, 0.0593));
                y_tr = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(trSample.rgb),     vec3<f32>(0.2627, 0.6780, 0.0593));
                y_bl = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(blSample.rgb),     vec3<f32>(0.2627, 0.6780, 0.0593));
                y_br = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(brSample.rgb),     vec3<f32>(0.2627, 0.6780, 0.0593));
            } else {
                y_l  = dot(leftSample.rgb   * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
                y_r  = dot(rightSample.rgb  * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
                y_t  = dot(topSample.rgb    * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
                y_b  = dot(bottomSample.rgb * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
                y_tl = dot(tlSample.rgb     * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
                y_tr = dot(trSample.rgb     * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
                y_bl = dot(blSample.rgb     * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
                y_br = dot(brSample.rgb     * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
            }

            let y_max_neighbor = max(max(max(y_l, y_r), max(y_t, y_b)), max(max(y_tl, y_tr), max(y_bl, y_br)));
            if (uniforms.defringeAmount > 0.0 && uniforms.isOriginal < 0.5) {
                rec2020Color = applyDefringe(rec2020Color, y_c, y_max_neighbor, uniforms.defringeAmount);
            }

            if (editSharpening > 0.0 || editNoiseReduction > 0.0) {
                rec2020Color = applyDetailAndSharpening(
                    rec2020Color,
                    y_c, y_l, y_r, y_t, y_b, y_tl, y_tr, y_bl, y_br,
                    editSharpening,
                    uniforms.sharpeningMasking,
                    editNoiseReduction
                );
            }
        }

        if (editGrainMode != 0) {
            let grainOffset = texelSize;
            let leftSample   = textureSample(u_texture, u_sampler, uv - vec2<f32>(grainOffset.x, 0.0));
            let rightSample  = textureSample(u_texture, u_sampler, uv + vec2<f32>(grainOffset.x, 0.0));
            let topSample    = textureSample(u_texture, u_sampler, uv - vec2<f32>(0.0, grainOffset.y));
            let bottomSample = textureSample(u_texture, u_sampler, uv + vec2<f32>(0.0, grainOffset.y));
            let tlSample     = textureSample(u_texture, u_sampler, uv - grainOffset);
            let trSample     = textureSample(u_texture, u_sampler, uv + vec2<f32>(grainOffset.x, -grainOffset.y));
            let blSample     = textureSample(u_texture, u_sampler, uv + vec2<f32>(-grainOffset.x, grainOffset.y));
            let brSample     = textureSample(u_texture, u_sampler, uv + grainOffset);

            var gy_l: f32; var gy_r: f32; var gy_t: f32; var gy_b: f32;
            var gy_tl: f32; var gy_tr: f32; var gy_bl: f32; var gy_br: f32;
            if (uniforms.isLinearRaw == 0) {
                gy_l  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(leftSample.rgb),   vec3<f32>(0.2627, 0.6780, 0.0593));
                gy_r  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(rightSample.rgb),  vec3<f32>(0.2627, 0.6780, 0.0593));
                gy_t  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(topSample.rgb),    vec3<f32>(0.2627, 0.6780, 0.0593));
                gy_b  = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(bottomSample.rgb), vec3<f32>(0.2627, 0.6780, 0.0593));
                gy_tl = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(tlSample.rgb),     vec3<f32>(0.2627, 0.6780, 0.0593));
                gy_tr = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(trSample.rgb),     vec3<f32>(0.2627, 0.6780, 0.0593));
                gy_bl = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(blSample.rgb),     vec3<f32>(0.2627, 0.6780, 0.0593));
                gy_br = dot(LINEAR_SRGB_TO_REC2020 * srgbGammaToLinear3(brSample.rgb),     vec3<f32>(0.2627, 0.6780, 0.0593));
            } else {
                gy_l  = dot(leftSample.rgb   * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
                gy_r  = dot(rightSample.rgb  * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
                gy_t  = dot(topSample.rgb    * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
                gy_b  = dot(bottomSample.rgb * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
                gy_tl = dot(tlSample.rgb     * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
                gy_tr = dot(trSample.rgb     * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
                gy_bl = dot(blSample.rgb     * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
                gy_br = dot(brSample.rgb     * wbExposureScale, vec3<f32>(0.2627, 0.6780, 0.0593));
            }

            let s_format = sqrt(max(formatMag, 0.1));
            let clusterSize = clamp(uniforms.grainSize * s_format, 0.5, 3.5);
            let effectiveAmount = uniforms.grainAmount * pow(max(formatMag, 0.1), 0.25);

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
    var contrastGain: f32 = 1.0 + editContrast / 100.0;
    if (editContrast < 0.0) {
        contrastGain = 1.0 + (editContrast / 100.0) * 0.60;
    }

    var srgbLinearOut: vec3<f32>;
    if (uniforms.isLinearRaw == 0) {
        let srgb = max(REC2020_TO_LINEAR_SRGB * rec2020Color, vec3<f32>(0.0));
        if (abs(contrastGain - 1.0) > 0.001) {
            srgbLinearOut = clamp((srgb - vec3<f32>(0.18)) * contrastGain + vec3<f32>(0.18), vec3<f32>(0.0), vec3<f32>(1.0));
        } else {
            srgbLinearOut = clamp(srgb, vec3<f32>(0.0), vec3<f32>(1.0));
        }
    } else if (uniforms.toneProfileMode == 0) {
        // Mode 0: AgX Vivid (Filmic Punchy) - DEFAULT
        srgbLinearOut = agxToneMapVivid(rec2020Color, contrastGain);
    } else if (uniforms.toneProfileMode == 1) {
        // Mode 1: AgX Neutral (Filmic Base)
        srgbLinearOut = agxToneMap(rec2020Color, contrastGain);
    } else if (uniforms.toneProfileMode == 6) {
        // Mode 6: AgX Photographic (Chroma Preserving)
        srgbLinearOut = agxToneMapPhotographic(rec2020Color, contrastGain);
    } else {
        // Modes 2..5: Camera Matching (Standard, Portrait, Landscape, Vivid)
        let camMode = uniforms.toneProfileMode - 1;
        let c_srgb = max(REC2020_TO_LINEAR_SRGB * rec2020Color, vec3<f32>(0.0));
        let rgbAdjusted = applyCameraHueSatMap(c_srgb, camMode);
        
        let curved = vec3<f32>(
            applyCameraToneCurve(rgbAdjusted.r, camMode),
            applyCameraToneCurve(rgbAdjusted.g, camMode),
            applyCameraToneCurve(rgbAdjusted.b, camMode)
        );
        if (abs(contrastGain - 1.0) > 0.001) {
            srgbLinearOut = clamp((curved - vec3<f32>(0.18)) * contrastGain + vec3<f32>(0.18), vec3<f32>(0.0), vec3<f32>(1.0));
        } else {
            srgbLinearOut = clamp(curved, vec3<f32>(0.0), vec3<f32>(1.0));
        }
    }

    // 9.1 Curated Fine-Art Print Media Archetypes
    if (uniforms.isOriginal < 0.5 && uniforms.printMediaMode != 0) {
        srgbLinearOut = applyPrintMedia(
            srgbLinearOut,
            uniforms.printMediaMode,
            uniforms.enlargerFilterMode,
            uniforms.printPaperGrade,
            uniforms.printProcessDepth
        );
    }

    var finalColor = srgbLinearOut;

    // 10. Creative 3D Look-Up Table (Post-Tone-Mapping in Linear sRGB)
    if (uniforms.lutEnabled != 0 && uniforms.lutIntensity > 0.0 && uniforms.isOriginal < 0.5) {
        let size = max(uniforms.lutSize, 2.0);
        let lutCoord = (finalColor * (size - 1.0) + vec3<f32>(0.5)) / size;
        let lutSample = textureSample(u_lut_3d, u_lut_sampler, lutCoord).rgb;
        finalColor = mix(finalColor, lutSample, clamp(uniforms.lutIntensity, 0.0, 1.0));
    }

    // 10.1 ColorSync & LittleCMS2 Soft Proofing & Gamut Alarm (Post-LUT / Pre-Display)
    if (uniforms.softProofEnabled != 0 && uniforms.isOriginal < 0.5) {
        let proofSize = 33.0;
        let proofCoord = (clamp(finalColor, vec3<f32>(0.0), vec3<f32>(1.0)) * (proofSize - 1.0) + vec3<f32>(0.5)) / proofSize;
        let proofSample = textureSample(u_proof_lut, u_proof_sampler, proofCoord);
        if (uniforms.gamutWarningEnabled != 0 && proofSample.a < 0.5) {
            let zebra = fract((in.position.x + in.position.y) / 12.0);
            if (zebra < 0.5) {
                finalColor = vec3<f32>(0.0, 0.90, 1.0);
            } else {
                finalColor = vec3<f32>(0.0, 0.35, 0.50);
            }
        } else {
            finalColor = proofSample.rgb;
        }
    }
    
    // 9. Apple EDR Radiance & Dynamic Range Expansion
    if (uniforms.edrEnabled != 0 && uniforms.edrHeadroom > 1.001) {
        let headroom = clamp(uniforms.edrHeadroom, 1.0, 4.0);
        
        if (uniforms.hasGainMap != 0) {
            let g = textureSample(u_gain_map, u_gain_map_sampler, uv).r;
            let gainLog2 = uniforms.gainMapMin + g * (uniforms.gainMapMax - uniforms.gainMapMin);
            let gamma = max(uniforms.gainMapGamma, 0.01);
            let gainFactor = pow(2.0, gainLog2 / gamma);
            
            let maxCap = max(uniforms.gainMapCapacityMax, 1.01);
            let weight = clamp((headroom - 1.0) / (maxCap - 1.0), 0.0, 1.0);
            let effectiveGain = 1.0 + (gainFactor - 1.0) * weight;
            finalColor = finalColor * effectiveGain;
        } else if (uniforms.isLinearRaw == 1) {
            let sceneLum = dot(rec2020Color, vec3<f32>(0.2627, 0.6780, 0.0593));
            let outLum = dot(finalColor, vec3<f32>(0.2126, 0.7152, 0.0722));
            
            let highlightFactor = smoothstep(0.85, 1.0, max(outLum, clamp(sceneLum - 0.15, 0.0, 1.0)));
            if (highlightFactor > 0.0) {
                let edrBoost = 1.0 + (headroom - 1.0) * pow(highlightFactor, 1.5);
                finalColor = finalColor * edrBoost;
            }
        }
        
        let peakC = max(finalColor.r, max(finalColor.g, finalColor.b));
        if (peakC > headroom) {
            let excess = (peakC - headroom) / peakC;
            finalColor = mix(finalColor * (headroom / peakC), vec3<f32>(headroom), clamp(excess * 1.5, 0.0, 1.0));
        } else if (peakC > 1.0) {
            let desatThreshold = 1.0 + (headroom - 1.0) * 0.8;
            if (peakC > desatThreshold) {
                let t = (peakC - desatThreshold) / max(headroom - desatThreshold, 1e-4);
                finalColor = mix(finalColor, vec3<f32>(peakC), clamp(t * 0.4, 0.0, 0.4));
            }
        }
        
        finalColor = max(LINEAR_SRGB_TO_DISPLAY_P3 * finalColor, vec3<f32>(0.0));
        if (uniforms.clippingOverlayActive != 0 && uniforms.isOriginal < 0.5) {
            let isHighlightClip = (uniforms.isLinearRaw == 1 && (sampledColor.r >= 1.0 || sampledColor.g >= 1.0 || sampledColor.b >= 1.0))
                || max(finalColor.r, max(finalColor.g, finalColor.b)) >= headroom * 0.99
                || max(rec2020Color.r, max(rec2020Color.g, rec2020Color.b)) >= 2.88;
            let sceneLum = dot(rec2020Color, vec3<f32>(0.2627, 0.6780, 0.0593));
            let isShadowClip = sceneLum <= 0.0007 || max(finalColor.r, max(finalColor.g, finalColor.b)) <= 0.0005;
            if (isHighlightClip) {
                let zebraHighlight = fract((in.position.x + in.position.y) / 14.0);
                if (zebraHighlight < 0.5) {
                    return vec4<f32>(1.0, 0.12, 0.12, sampledColor.a);
                }
            } else if (isShadowClip) {
                let zebraShadow = fract((in.position.x - in.position.y) / 14.0);
                if (zebraShadow < 0.5) {
                    return vec4<f32>(0.05, 0.40, 1.0, sampledColor.a);
                }
            }
        }
        return vec4<f32>(finalColor, sampledColor.a);
    } else {
        let displayColor = linearToSrgbGamma3(clamp(finalColor, vec3<f32>(0.0), vec3<f32>(1.0)));
        if (uniforms.clippingOverlayActive != 0 && uniforms.isOriginal < 0.5) {
            let isHighlightClip = (uniforms.isLinearRaw == 1 && (sampledColor.r >= 1.0 || sampledColor.g >= 1.0 || sampledColor.b >= 1.0))
                || max(displayColor.r, max(displayColor.g, displayColor.b)) >= 0.992
                || max(rec2020Color.r, max(rec2020Color.g, rec2020Color.b)) >= 2.88;
            let sceneLum = dot(rec2020Color, vec3<f32>(0.2627, 0.6780, 0.0593));
            let isShadowClip = sceneLum <= 0.0007 || max(displayColor.r, max(displayColor.g, displayColor.b)) <= 0.004;
            if (isHighlightClip) {
                let zebraHighlight = fract((in.position.x + in.position.y) / 14.0);
                if (zebraHighlight < 0.5) {
                    return vec4<f32>(1.0, 0.12, 0.12, sampledColor.a);
                }
            } else if (isShadowClip) {
                let zebraShadow = fract((in.position.x - in.position.y) / 14.0);
                if (zebraShadow < 0.5) {
                    return vec4<f32>(0.05, 0.40, 1.0, sampledColor.a);
                }
            }
        }
        let dither = (ign(in.position.xy) - 0.5) / 255.0;
        return vec4<f32>(clamp(displayColor + vec3<f32>(dither), vec3<f32>(0.0), vec3<f32>(1.0)), sampledColor.a);
    }
}
