// Canonical Color Science, Emulsions, Print Media & AgX Filmic Tone Mapping

fn float_mod(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}

// Exact sRGB electro-optical transfer functions (IEC 61966-2-1)
fn srgbGammaToLinear(c: f32) -> f32 {
    let cl = clamp(c, 0.0, 1.0);
    if (cl <= 0.04045) {
        return cl / 12.92;
    } else {
        return pow((cl + 0.055) / 1.055, 2.4);
    }
}

fn srgbGammaToLinear3(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        srgbGammaToLinear(c.r),
        srgbGammaToLinear(c.g),
        srgbGammaToLinear(c.b)
    );
}

fn linearToSrgbGamma(c: f32) -> f32 {
    let cl = clamp(c, 0.0, 1.0);
    if (cl <= 0.0031308) {
        return 12.92 * cl;
    } else {
        return 1.055 * pow(cl, 1.0 / 2.4) - 0.055;
    }
}

fn linearToSrgbGamma3(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        linearToSrgbGamma(c.r),
        linearToSrgbGamma(c.g),
        linearToSrgbGamma(c.b)
    );
}

// MARK: - RGB <-> HSV Color Transform Helpers
fn rgb2hsv(c: vec3<f32>) -> vec3<f32> {
    let maxV = max(c.r, max(c.g, c.b));
    let minV = min(c.r, min(c.g, c.b));
    let delta = maxV - minV;
    
    var h: f32 = 0.0;
    if (delta > 1e-5) {
        if (maxV == c.r) {
            h = (c.g - c.b) / delta;
            if (h < 0.0) { h = h + 6.0; }
        } else if (maxV == c.g) {
            h = (c.b - c.r) / delta + 2.0;
        } else {
            h = (c.r - c.g) / delta + 4.0;
        }
        h = h * 60.0;
    }
    
    var s: f32 = 0.0;
    if (maxV > 1e-5) {
        s = delta / maxV;
    }
    return vec3<f32>(h, s, maxV);
}

fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    let h = c.x / 60.0;
    let s = c.y;
    let v = c.z;
    let C = v * s;
    let X = C * (1.0 - abs(float_mod(h, 2.0) - 1.0));
    let m = v - C;
    var rgb = vec3<f32>(0.0);
    if (h < 1.0) { rgb = vec3<f32>(C, X, 0.0); }
    else if (h < 2.0) { rgb = vec3<f32>(X, C, 0.0); }
    else if (h < 3.0) { rgb = vec3<f32>(0.0, C, X); }
    else if (h < 4.0) { rgb = vec3<f32>(0.0, X, C); }
    else if (h < 5.0) { rgb = vec3<f32>(X, 0.0, C); }
    else { rgb = vec3<f32>(C, 0.0, X); }
    return rgb + vec3<f32>(m);
}

// MARK: - Monotonic Cubic Hermite Tone Curves (Fritsch-Carlson C^1 Continuous)
fn evalCameraHermite7(x: f32, knots: array<vec2<f32>, 7>, m: array<f32, 7>) -> f32 {
    if (x <= knots[0].x) { return knots[0].y; }
    if (x >= knots[6].x) { return knots[6].y; }
    for (var k: i32 = 0; k < 6; k = k + 1) {
        if (x <= knots[k + 1].x) {
            let dx = knots[k + 1].x - knots[k].x;
            let t = (x - knots[k].x) / dx;
            let t2 = t * t;
            let t3 = t2 * t;
            let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
            let h10 = t3 - 2.0 * t2 + t;
            let h01 = -2.0 * t3 + 3.0 * t2;
            let h11 = t3 - t2;
            return clamp(h00 * knots[k].y + h10 * m[k] * dx + h01 * knots[k + 1].y + h11 * m[k + 1] * dx, 0.0, 1.0);
        }
    }
    return x;
}

fn applyCameraToneCurve(x: f32, mode: i32) -> f32 {
    let xc = clamp(x, 0.0, 1.0);
    // mode 1: Standard, mode 2: Portrait, mode 3: Landscape, mode 4: Vivid
    if (mode == 2) {
        return evalCameraHermite7(xc, DCP_KNOTS_PORTRAIT, DCP_TANGENTS_PORTRAIT);
    } else if (mode == 3) {
        return evalCameraHermite7(xc, DCP_KNOTS_LANDSCAPE, DCP_TANGENTS_LANDSCAPE);
    } else if (mode == 4) {
        return evalCameraHermite7(xc, DCP_KNOTS_VIVID, DCP_TANGENTS_VIVID);
    } else {
        return evalCameraHermite7(xc, DCP_KNOTS_STANDARD, DCP_TANGENTS_STANDARD);
    }
}

// MARK: - Camera-Matching HueSatMaps
fn evalHueSatMapDelta(h_deg: f32, s: f32, table: array<vec3<f32>, 24>) -> vec3<f32> {
    var h_norm = fract(h_deg / 360.0);
    if (h_norm < 0.0) { h_norm = h_norm + 1.0; }
    let h_coord = h_norm * 12.0;
    let h0 = i32(floor(h_coord)) % 12;
    let h1 = (h0 + 1) % 12;
    let h_frac = h_coord - floor(h_coord);

    let s_clamped = clamp(s, 0.0, 1.0);
    
    let c00 = table[h0 * 2 + 0];
    let c01 = table[h0 * 2 + 1];
    let c10 = table[h1 * 2 + 0];
    let c11 = table[h1 * 2 + 1];

    let c0 = mix(c00, c01, s_clamped);
    let c1 = mix(c10, c11, s_clamped);

    return mix(c0, c1, h_frac);
}

fn applyCameraHueSatMap(c_srgb: vec3<f32>, mode: i32) -> vec3<f32> {
    if (max(max(c_srgb.r, c_srgb.g), c_srgb.b) <= 1e-6) {
        return c_srgb;
    }
    let hsv = rgb2hsv(c_srgb);
    if (hsv.y <= 1e-6) {
        return c_srgb;
    }
    var delta = vec3<f32>(0.0);
    if (mode == 2) {
        delta = evalHueSatMapDelta(hsv.x, hsv.y, DCP_HSM_PORTRAIT);
    } else if (mode == 3) {
        delta = evalHueSatMapDelta(hsv.x, hsv.y, DCP_HSM_LANDSCAPE);
    } else if (mode == 4) {
        delta = evalHueSatMapDelta(hsv.x, hsv.y, DCP_HSM_VIVID);
    } else {
        delta = evalHueSatMapDelta(hsv.x, hsv.y, DCP_HSM_STANDARD);
    }

    let h_out = float_mod(float_mod(hsv.x + delta.x, 360.0) + 360.0, 360.0);
    let s_out = clamp(hsv.y * delta.y, 0.0, 1.0);
    let v_out = max(hsv.z * delta.z, 0.0);

    return hsv2rgb(vec3<f32>(h_out, s_out, v_out));
}

// MARK: - AgX Filmic Tone Mapping Engine
fn agxApplyCurveChannel(x: f32) -> f32 {
    let xc = clamp(x, 0.0, 1.0);
    // Horner polynomial evaluation
    let curved = (((((15.45672 * xc - 40.02792) * xc + 31.87076) * xc - 6.84882) * xc + 0.42860) * xc + 0.12066) * xc;
    return clamp(curved, 0.0, 1.0);
}

fn agxToneMap(rec2020_color: vec3<f32>, contrast_gain: f32) -> vec3<f32> {
    let minC = min(rec2020_color.r, min(rec2020_color.g, rec2020_color.b));
    var cComp = rec2020_color;
    if (minC < 0.0) {
        cComp = rec2020_color - vec3<f32>(minC);
    }

    let inset = AGX_INSET_MATRIX * cComp;
    let xRel = max(inset / 0.18, vec3<f32>(1e-6));
    let logEncoded = (log2(xRel) - vec3<f32>(AGX_MIN_EV)) / AGX_EV_RANGE;
    var norm = clamp(logEncoded, vec3<f32>(0.0), vec3<f32>(1.0));
    
    if (abs(contrast_gain - 1.0) > 0.001) {
        norm = clamp(vec3<f32>(AGX_MID_GRAY_NORM) + (norm - vec3<f32>(AGX_MID_GRAY_NORM)) * contrast_gain, vec3<f32>(0.0), vec3<f32>(1.0));
    }
    
    let curved = vec3<f32>(
        agxApplyCurveChannel(norm.r),
        agxApplyCurveChannel(norm.g),
        agxApplyCurveChannel(norm.b)
    );
    
    let linearized = pow(max(curved, vec3<f32>(0.0)), vec3<f32>(AGX_GAMMA));
    return max(AGX_OUTSET_MATRIX * linearized, vec3<f32>(0.0));
}

fn agxToneMapVivid(rec2020_color: vec3<f32>, contrast_gain: f32) -> vec3<f32> {
    let punchyContrast = contrast_gain * 1.35;
    let base = agxToneMap(rec2020_color, punchyContrast);

    let lum = 0.2126 * base.r + 0.7152 * base.g + 0.0722 * base.b;
    let t = clamp((lum - 0.65) / 0.35, 0.0, 1.0);
    let h = t * t * (3.0 - 2.0 * t);
    let baseSat = 1.25 - 0.25 * h;
    let sat = baseSat;

    return clamp(vec3<f32>(
        lum + sat * (base.r - lum),
        lum + sat * (base.g - lum),
        lum + sat * (base.b - lum)
    ), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn agxToneMapPhotographic(rec2020_color: vec3<f32>, contrast_gain: f32) -> vec3<f32> {
    let minC = min(rec2020_color.r, min(rec2020_color.g, rec2020_color.b));
    var compressed = rec2020_color;
    if (minC < 0.0) {
        compressed = rec2020_color - vec3<f32>(minC);
    }

    let photoContrast = contrast_gain * 1.28;

    // 1. Scene luminance in Linear Rec.2020
    let lumRec = 0.2627 * compressed.r + 0.6780 * compressed.g + 0.0593 * compressed.b;

    // 2. Evaluate AgX sigmoidal transfer on scene luminance
    let xRel = max(lumRec / 0.18, 1e-6);
    let logEncoded = (log2(xRel) - AGX_MIN_EV) / AGX_EV_RANGE;
    var norm = clamp(logEncoded, 0.0, 1.0);
    if (abs(photoContrast - 1.0) > 0.001) {
        norm = clamp(AGX_MID_GRAY_NORM + (norm - AGX_MID_GRAY_NORM) * photoContrast, 0.0, 1.0);
    }
    let curved = agxApplyCurveChannel(norm);
    let lumMapped = pow(max(curved, 0.0), AGX_GAMMA);

    // 3. Proportional ratio scaling
    var gain: f32 = 1.0;
    if (lumRec > 1e-6) {
        gain = lumMapped / lumRec;
    }
    let cScaled = compressed * gain;
    let sRatio = REC2020_TO_LINEAR_SRGB * cScaled;

    // 4. Base per-channel AgX with matching photographic contrast
    let sAgx = agxToneMap(compressed, photoContrast);

    // 5. C^1 Hermite highlight desaturation blend
    let peakVal = max(lumMapped, max(max(sRatio.r, 0.0), max(max(sRatio.g, 0.0), max(sRatio.b, 0.0))));
    let t = clamp((peakVal - 0.85) / 0.20, 0.0, 1.0);
    let h = t * t * (3.0 - 2.0 * t);

    return clamp(vec3<f32>(
        max(sRatio.r, 0.0) * (1.0 - h) + sAgx.r * h,
        max(sRatio.g, 0.0) * (1.0 - h) + sAgx.g * h,
        max(sRatio.b, 0.0) * (1.0 - h) + sAgx.b * h
    ), vec3<f32>(0.0), vec3<f32>(1.0));
}

// MARK: - Advanced Bilateral Base/Detail Tone Recovery
fn applyBilateralToneRecovery(
    color: vec3<f32>,
    y_c: f32,
    y_neighbors: array<f32, 16>,
    highlights: f32,
    shadows: f32,
    whites: f32,
    blacks: f32
) -> vec3<f32> {
    if (highlights == 0.0 && shadows == 0.0 && whites == 0.0 && blacks == 0.0) {
        return color;
    }
    if (y_c <= 1e-6) {
        return color;
    }
    
    let ev_c = log2(max(y_c, 1e-5) / 0.18);
    
    let w_s_ortho: f32 = 0.60653066;
    let w_s_diag: f32  = 0.36787944;
    let inv_two_sigma_r_sq: f32 = 1.0 / 8.0;
    
    var total_weight: f32 = 1.0;
    var weighted_ev_sum: f32 = ev_c;
    
    // 0..3: Inner Orthogonal neighbors
    for (var i: i32 = 0; i < 4; i = i + 1) {
        let ev_i = log2(max(y_neighbors[i], 1e-5) / 0.18);
        let d_ev = ev_i - ev_c;
        let w_r = exp(-(d_ev * d_ev) * inv_two_sigma_r_sq);
        let w = w_s_ortho * w_r;
        total_weight = total_weight + w;
        weighted_ev_sum = weighted_ev_sum + w * ev_i;
    }
    
    // 4..7: Inner Diagonal neighbors
    for (var i: i32 = 4; i < 8; i = i + 1) {
        let ev_i = log2(max(y_neighbors[i], 1e-5) / 0.18);
        let d_ev = ev_i - ev_c;
        let w_r = exp(-(d_ev * d_ev) * inv_two_sigma_r_sq);
        let w = w_s_diag * w_r;
        total_weight = total_weight + w;
        weighted_ev_sum = weighted_ev_sum + w * ev_i;
    }

    // 8..11: Outer Orthogonal neighbors
    for (var i: i32 = 8; i < 12; i = i + 1) {
        let ev_i = log2(max(y_neighbors[i], 1e-5) / 0.18);
        let d_ev = ev_i - ev_c;
        let w_r = exp(-(d_ev * d_ev) * inv_two_sigma_r_sq);
        let w = w_s_ortho * w_r;
        total_weight = total_weight + w;
        weighted_ev_sum = weighted_ev_sum + w * ev_i;
    }
    
    // 12..15: Outer Diagonal neighbors
    for (var i: i32 = 12; i < 16; i = i + 1) {
        let ev_i = log2(max(y_neighbors[i], 1e-5) / 0.18);
        let d_ev = ev_i - ev_c;
        let w_r = exp(-(d_ev * d_ev) * inv_two_sigma_r_sq);
        let w = w_s_diag * w_r;
        total_weight = total_weight + w;
        weighted_ev_sum = weighted_ev_sum + w * ev_i;
    }
    
    let ev_base = weighted_ev_sum / total_weight;
    var delta_ev: f32 = 0.0;
    
    // 1. Blacks: Zones 0-II (-4.0 EV to -1.0 EV)
    if (blacks != 0.0) {
        let u = clamp((-ev_base - 1.0) / 3.0, 0.0, 1.0);
        let w_blacks = u * u * (3.0 - 2.0 * u);
        delta_ev = delta_ev + (blacks / 100.0) * 1.20 * w_blacks;
    }
    
    // 2. Shadows: Zones II-IV (-3.0 EV to 0.0 EV)
    if (shadows != 0.0) {
        let u = clamp(-ev_base / 3.0, 0.0, 1.0);
        let w_shadows = u * u * (3.0 - 2.0 * u);
        delta_ev = delta_ev + (shadows / 100.0) * 2.25 * w_shadows;
    }
    
    // 3. Highlights: Zones VI-IX (0.0 EV to +3.0 EV)
    if (highlights != 0.0) {
        let u = clamp(ev_base / 3.0, 0.0, 1.0);
        let w_highlights = u * u * (3.0 - 2.0 * u);
        delta_ev = delta_ev + (highlights / 100.0) * 2.25 * w_highlights;
    }
    
    // 4. Whites: Zones VII-X (+1.0 EV to +4.0 EV)
    if (whites != 0.0) {
        let u = clamp((ev_base - 1.0) / 3.0, 0.0, 1.0);
        let w_whites = u * u * (3.0 - 2.0 * u);
        delta_ev = delta_ev + (whites / 100.0) * 1.20 * w_whites;
    }
    
    if (delta_ev == 0.0) {
        return color;
    }
    
    let detail = clamp(ev_c - ev_base, -3.0, 3.0);
    let detail_boost = 0.35 * detail * (abs(delta_ev) / 2.25);
    let gain = exp2(clamp(delta_ev + detail_boost, -6.0, 6.0));
    return color * gain;
}

// MARK: - Color & Vibrance Adjustment Function
fn applyColorAdjustments(
    rgb: vec3<f32>,
    vibrance: f32,
    saturation: f32,
    deepBlue: f32,
    foliage: f32
) -> vec3<f32> {
    if (vibrance == 0.0 && saturation == 0.0 && deepBlue == 0.0 && foliage == 0.0) {
        return rgb;
    }
    
    var c = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    let lum = 0.2627 * c.r + 0.6780 * c.g + 0.0593 * c.b;

    if (saturation != 0.0) {
        let satGain = max(1.0 + saturation / 100.0, 0.0);
        c = vec3<f32>(lum) + (c - vec3<f32>(lum)) * satGain;
    }

    if (vibrance != 0.0) {
        let maxC = max(max(c.r, c.g), c.b);
        let minC = min(min(c.r, c.g), c.b);
        let chroma = maxC - minC;

        var skinProtect: f32 = 1.0;
        let sum = c.r + c.g + c.b;
        if (sum > 1e-5) {
            let rN = c.r / sum;
            let gN = c.g / sum;
            let bN = 1.0 - rN - gN;

            let deltaCross = gN + 0.35 * rN - 0.478;
            let dCross = deltaCross / 0.040;
            let dMel = (rN - 0.48) / 0.16;
            let d2 = dCross * dCross + dMel * dMel;
            let gauss = exp(-0.5 * d2);

            let uLum = clamp((sum - 0.005) / 0.025, 0.0, 1.0);
            let wLum = uLum * uLum * (3.0 - 2.0 * uLum);

            let uRg = clamp((rN - gN) / 0.05, 0.0, 1.0);
            let wRg = uRg * uRg * (3.0 - 2.0 * uRg);

            let uRb = clamp((rN - bN) / 0.08, 0.0, 1.0);
            let wRb = uRb * uRb * (3.0 - 2.0 * uRb);

            let uGb = clamp((gN - bN + 0.05) / 0.10, 0.0, 1.0);
            let wGb = uGb * uGb * (3.0 - 2.0 * uGb);

            let skinMetric = gauss * wLum * wRg * wRb * wGb;
            skinProtect = 1.0 - 0.85 * skinMetric;
        }

        var vibGain: f32 = (vibrance / 100.0) * 0.6;
        if (vibrance > 0.0) {
            vibGain = (vibrance / 100.0) * (1.0 - chroma) * 0.8 * skinProtect;
        }

        c = c + (c - vec3<f32>(lum)) * vibGain;
    }

    if (deepBlue != 0.0) {
        let maxWarm = max(c.r, c.g);
        let skySignal = max(c.b - maxWarm, 0.0);
        if (skySignal > 0.0) {
            let x = deepBlue / 100.0;
            let factor = sign(x) * pow(abs(x), 1.6);
            c.r = c.r - factor * skySignal * 0.28;
            c.g = c.g - factor * skySignal * 0.18;
            c.b = c.b - factor * skySignal * 0.15;
        }
    }

    if (foliage != 0.0) {
        let maxRb = max(c.r, c.b);
        let greenSignal = max(c.g - maxRb, 0.0);
        if (greenSignal > 0.0) {
            let x = foliage / 100.0;
            let factor = sign(x) * pow(abs(x), 1.6);
            c.r = c.r - factor * greenSignal * 0.30;
            c.b = c.b - factor * greenSignal * 0.15;
            c.g = c.g + factor * greenSignal * 0.10;
        }
    }

    return clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
}

// MARK: - Split Toning & Color Grading
fn applySplitToning(
    rgb: vec3<f32>,
    shadowHue: f32,
    shadowSat: f32,
    highlightHue: f32,
    highlightSat: f32,
    balance: f32
) -> vec3<f32> {
    if (shadowSat <= 0.0 && highlightSat <= 0.0) {
        return rgb;
    }

    let lum = 0.2627 * rgb.r + 0.6780 * rgb.g + 0.0593 * rgb.b;
    let yClamp = clamp(lum, 0.0, 1.0);

    let t = clamp(0.5 - (balance / 100.0) * 0.25, 0.1, 0.9);
    var delta = vec3<f32>(0.0);

    // Shadow Toning
    if (shadowSat > 0.0 && yClamp < t) {
        let u_s = (t - yClamp) / t;
        let w_s = u_s * u_s * (3.0 - 2.0 * u_s) * (shadowSat / 100.0);
        let tintRgb = hsv2rgb(vec3<f32>(shadowHue, 1.0, 1.0));
        let tLum = 0.2627 * tintRgb.r + 0.6780 * tintRgb.g + 0.0593 * tintRgb.b;
        delta = delta + vec3<f32>(w_s) * (tintRgb - vec3<f32>(tLum));
    }

    // Highlight Toning
    if (highlightSat > 0.0 && yClamp > t) {
        let u_h = (yClamp - t) / (1.0 - t);
        let w_h = u_h * u_h * (3.0 - 2.0 * u_h) * (highlightSat / 100.0);
        let tintRgb = hsv2rgb(vec3<f32>(highlightHue, 1.0, 1.0));
        let tLum = 0.2627 * tintRgb.r + 0.6780 * tintRgb.g + 0.0593 * tintRgb.b;
        delta = delta + vec3<f32>(w_h) * (tintRgb - vec3<f32>(tLum));
    }

    return clamp(rgb + delta, vec3<f32>(0.0), vec3<f32>(1.0));
}

// MARK: - Curated Fine-Art Print Media Archetypes & Enlarger Filtration
fn applyPaperGrade(val: f32, grade: f32) -> f32 {
    if (abs(grade) < 0.001) {
        return val;
    }
    var scale: f32 = 0.40;
    if (grade >= 0.0) { scale = 0.60; }
    let alpha = (grade / 2.0) * scale;
    return clamp(val + alpha * val * (1.0 - val) * (2.0 * val - 1.0), 0.0, 1.0);
}

fn applyBaseWarmth(rgb: vec3<f32>, warmth: f32) -> vec3<f32> {
    if (abs(warmth) < 0.001) {
        return rgb;
    }
    let lum = 0.2627 * rgb.r + 0.6780 * rgb.g + 0.0593 * rgb.b;
    let highlightWeight = clamp(pow(lum, 0.6), 0.0, 1.0);
    let factor = (warmth / 100.0) * highlightWeight;
    
    var mult = vec3<f32>(1.0);
    if (factor > 0.0) {
        mult = vec3<f32>(
            1.0,
            1.0 - factor * 0.013849,
            1.0 - factor * 0.051111
        );
    } else {
        let absF = -factor;
        mult = vec3<f32>(
            1.0 - absF * 0.033699,
            1.0 - absF * 0.031749,
            1.0
        );
    }
    return clamp(rgb * mult, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn getEnlargerFilterWeights(filterMode: i32) -> vec3<f32> {
    if (filterMode == 1) { return vec3<f32>(0.35, 0.60, 0.05); } // YellowK2
    if (filterMode == 2) { return vec3<f32>(0.55, 0.42, 0.03); } // Orange16
    if (filterMode == 3) { return vec3<f32>(0.85, 0.15, 0.00); } // Red25A
    if (filterMode == 4) { return vec3<f32>(0.15, 0.80, 0.05); } // GreenX1
    if (filterMode == 5) { return vec3<f32>(0.00, 0.60, 0.40); } // Orthochromatic
    return vec3<f32>(0.2627, 0.6780, 0.0593);                    // NeutralPanchromatic
}

fn applyPrintMedia(
    rgb: vec3<f32>,
    mediaMode: i32,
    filterMode: i32,
    paperGrade: f32,
    processDepth: f32
) -> vec3<f32> {
    if (mediaMode == 0) {
        return rgb;
    }
    
    let depth = clamp(processDepth / 100.0, 0.0, 1.0);
    
    if (mediaMode == 1) { // ChromogenicTypeC (RA-4)
        let rG = applyPaperGrade(rgb.r, paperGrade);
        let gG = applyPaperGrade(rgb.g, paperGrade);
        let bG = applyPaperGrade(rgb.b, paperGrade);
        
        let uR = clamp((rG - 0.70) / 0.30, 0.0, 1.0);
        let rP = rG - 0.045 * depth * uR * uR * (3.0 - 2.0 * uR);
        let uG = clamp((gG - 0.70) / 0.30, 0.0, 1.0);
        let gP = gG - 0.045 * depth * uG * uG * (3.0 - 2.0 * uG);
        let uB = clamp((bG - 0.70) / 0.30, 0.0, 1.0);
        let bP = bG - 0.045 * depth * uB * uB * (3.0 - 2.0 * uB);
        
        let lum = 0.2627 * rP + 0.6780 * gP + 0.0593 * bP;
        let satScale = 0.70 + 0.35 * depth;
        var rOut = clamp(lum + (rP - lum) * satScale, 0.0, 1.0);
        var gOut = clamp(lum + (gP - lum) * satScale, 0.0, 1.0);
        var bOut = clamp(lum + (bP - lum) * satScale, 0.0, 1.0);
        
        let dyeDensity = 0.04 * max(1.0 - lum, 0.0) * depth;
        rOut = clamp(rOut * (1.0 - dyeDensity * 0.4), 0.0, 1.0);
        gOut = clamp(gOut * (1.0 - dyeDensity * 0.6), 0.0, 1.0);
        bOut = clamp(bOut * (1.0 - dyeDensity * 0.9), 0.0, 1.0);
        
        return vec3<f32>(rOut, gOut, bOut);
    } else if (mediaMode == 2) { // SilverDyeBleach (Azo Reversal)
        let rG = applyPaperGrade(rgb.r, paperGrade);
        let gG = applyPaperGrade(rgb.g, paperGrade);
        let bG = applyPaperGrade(rgb.b, paperGrade);
        
        let shadowPow = 1.0 + 0.12 * depth;
        let rP = pow(rG, shadowPow);
        let gP = pow(gG, shadowPow);
        let bP = pow(bG, shadowPow);
        
        let lum = 0.2627 * rP + 0.6780 * gP + 0.0593 * bP;
        let azoSat = 0.80 + 0.30 * depth;
        let rOut = clamp(lum + (rP - lum) * azoSat, 0.0, 1.0);
        let gOut = clamp(lum + (gP - lum) * azoSat, 0.0, 1.0);
        let bOut = clamp(lum + (bP - lum) * azoSat, 0.0, 1.0);
        
        return vec3<f32>(rOut, gOut, bOut);
    } else if (mediaMode == 3) { // BarytaSilverGelatin
        let w = getEnlargerFilterWeights(filterMode);
        let rawLum = clamp(dot(rgb, w), 0.0, 1.0);
        let silver = applyPaperGrade(rawLum, paperGrade);
        
        let dmaxGamma = 1.0 + 0.08 * depth;
        let silverToned = pow(silver, dmaxGamma);
        
        return vec3<f32>(silverToned);
    } else if (mediaMode == 4) { // WarmtoneChlorobromide (Selenium Toned)
        let w = getEnlargerFilterWeights(filterMode);
        let rawLum = clamp(dot(rgb, w), 0.0, 1.0);
        let silver = applyPaperGrade(rawLum, paperGrade);
        
        let shadowPow = 1.0 + 0.10 * depth;
        let silverDeep = pow(silver, shadowPow);
        
        let midMask = 4.0 * silver * (1.0 - silver);
        let shadowMask = 4.0 * silver * (1.0 - silver) * (1.0 - silver);
        
        let rTone = silverDeep + depth * (midMask * 0.025 + shadowMask * 0.020);
        let gTone = silverDeep + depth * (midMask * 0.010 - shadowMask * 0.012);
        let bTone = silverDeep + depth * (-midMask * 0.015 + shadowMask * 0.025);
        
        let toned = clamp(vec3<f32>(rTone, gTone, bTone), vec3<f32>(0.0), vec3<f32>(1.0));
        return applyBaseWarmth(toned, 5.0);
    } else if (mediaMode == 5) { // PlatinumPalladium
        let w = getEnlargerFilterWeights(filterMode);
        let rawLum = clamp(dot(rgb, w), 0.0, 1.0);
        let silver = applyPaperGrade(rawLum, paperGrade);
        
        let ptScale = silver + depth * 0.10 * silver * (1.0 - silver) * (1.0 - silver);
        let midWarmth = 4.0 * silver * (1.0 - silver);
        let rTone = ptScale + depth * midWarmth * 0.040;
        let gTone = ptScale + depth * midWarmth * 0.024;
        let bTone = ptScale - depth * midWarmth * 0.016;
        
        let toned = clamp(vec3<f32>(rTone, gTone, bTone), vec3<f32>(0.0), vec3<f32>(1.0));
        return applyBaseWarmth(toned, 10.0 * depth);
    } else if (mediaMode == 6) { // Cyanotype
        let w = getEnlargerFilterWeights(filterMode);
        let rawLum = clamp(dot(rgb, w), 0.0, 1.0);
        let silver = applyPaperGrade(rawLum, paperGrade);
        
        let density = 1.0 - silver;
        let rP = silver * (1.0 - 0.85 * depth * density);
        let gP = silver * (1.0 - 0.45 * depth * density);
        let bP = silver * (1.0 - 0.05 * depth * density);
        
        let toned = clamp(vec3<f32>(rP, gP, bP), vec3<f32>(0.0), vec3<f32>(1.0));
        return applyBaseWarmth(toned, 8.0 * depth);
    } else if (mediaMode == 7) { // BamakoSilverContact
        let w = getEnlargerFilterWeights(filterMode);
        let rawLum = clamp(dot(rgb, w), 0.0, 1.0);
        let graded = applyPaperGrade(rawLum, paperGrade);
        let c = clamp(graded, 0.0, 1.0);
        let sLum = clamp(c + 0.22 * c * (1.0 - c) * (2.0 * c - 1.0) + 0.14 * c * (1.0 - c), 0.0, 1.0);
        let ferro = pow(sLum, 1.0 + 0.15 * depth);
        let silver = clamp(ferro * depth + graded * (1.0 - depth), 0.0, 1.0);
        return vec3<f32>(silver);
    }
    return rgb;
}

// MARK: - Upstream Recording Emulsions
fn applyFilmCharacteristicCurve(val: f32, gamma: f32, shoulder: f32, toe: f32) -> f32 {
    let v = clamp(val, 0.0, 2.0);
    var base = v;
    if (abs(gamma - 1.0) > 0.001) {
        let u = clamp(v / (v + 0.35), 0.0, 1.0);
        let gMod = (gamma - 1.0) * 0.40;
        base = max(v + gMod * u * (1.0 - u) * (2.0 * u - 1.0), 0.0);
    }
    let tS = clamp((base - 0.65) / 0.35, 0.0, 1.0);
    let compS = shoulder * 0.12 * tS * tS * (3.0 - 2.0 * tS);

    let tT = clamp(base / 0.35, 0.0, 1.0);
    let compT = toe * 0.06 * (1.0 - tT) * (1.0 - tT);

    return max(base - compS - compT, 0.0);
}

fn applyDirChemicalCoupling(
    rgb: vec3<f32>,
    c_rg: f32, c_rb: f32,
    c_gr: f32, c_gb: f32,
    c_br: f32, c_bg: f32
) -> vec3<f32> {
    let eps: f32 = 1e-4;
    let inv_ln10: f32 = 0.43429448190325182;
    let d_r = -(log(max(rgb.r, eps)) * inv_ln10);
    let d_g = -(log(max(rgb.g, eps)) * inv_ln10);
    let d_b = -(log(max(rgb.b, eps)) * inv_ln10);

    let c_rr = 1.0 - c_rg - c_rb;
    let c_gg = 1.0 - c_gr - c_gb;
    let c_bb = 1.0 - c_br - c_bg;

    let d_r_out = c_rr * d_r + c_rg * d_g + c_rb * d_b;
    let d_g_out = c_gr * d_r + c_gg * d_g + c_gb * d_b;
    let d_b_out = c_br * d_r + c_bg * d_g + c_bb * d_b;

    return clamp(vec3<f32>(
        pow(10.0, -d_r_out),
        pow(10.0, -d_g_out),
        pow(10.0, -d_b_out)
    ), vec3<f32>(0.0), vec3<f32>(2.0));
}

fn applyEmulsionSCurve(x: f32, alpha: f32, beta: f32) -> f32 {
    let c = clamp(x, 0.0, 1.0);
    let poly = c + alpha * c * (1.0 - c) * (2.0 * c - 1.0) + beta * c * (1.0 - c);
    return clamp(poly, 0.0, 1.0);
}

fn applyRecordingEmulsion(
    rgb: vec3<f32>,
    archetype: i32,
    pushPullEv: f32,
    filterMode: i32
) -> vec3<f32> {
    if (archetype == 0) {
        return rgb;
    }

    let expGain = pow(2.0, clamp(pushPullEv, -2.0, 2.0));
    let curRgb = max(rgb * expGain, vec3<f32>(0.0));

    if (archetype == 1) { // RochesterPortrait400
        let dir = applyDirChemicalCoupling(curRgb, -0.04, 0.02, -0.08, 0.02, 0.02, -0.05);
        return vec3<f32>(
            applyFilmCharacteristicCurve(dir.r, 0.92, 0.38, 0.04),
            applyFilmCharacteristicCurve(dir.g, 0.90, 0.38, 0.06),
            applyFilmCharacteristicCurve(dir.b, 0.88, 0.40, 0.12)
        );
    } else if (archetype == 2) { // AshigaraChrome50
        let dir = applyDirChemicalCoupling(curRgb, -0.15, -0.08, -0.18, -0.08, -0.06, -0.12);
        return vec3<f32>(
            applyFilmCharacteristicCurve(dir.r, 1.10, 0.12, 0.14),
            applyFilmCharacteristicCurve(dir.g, 1.12, 0.10, 0.14),
            applyFilmCharacteristicCurve(dir.b, 1.11, 0.12, 0.16)
        );
    } else if (archetype == 3) { // RochesterK14Chrome64
        let dir = applyDirChemicalCoupling(curRgb, -0.08, 0.06, -0.10, 0.04, 0.08, -0.04);
        return vec3<f32>(
            applyFilmCharacteristicCurve(dir.r, 1.05, 0.18, 0.08),
            applyFilmCharacteristicCurve(dir.g, 1.06, 0.16, 0.10),
            applyFilmCharacteristicCurve(dir.b, 1.08, 0.14, 0.14)
        );
    } else if (archetype == 4) { // RochesterCinema500
        let dir = applyDirChemicalCoupling(curRgb, -0.02, 0.01, -0.03, 0.01, 0.01, -0.02);
        return vec3<f32>(
            applyFilmCharacteristicCurve(dir.r, 0.82, 0.45, 0.08),
            applyFilmCharacteristicCurve(dir.g, 0.82, 0.45, 0.08),
            applyFilmCharacteristicCurve(dir.b, 0.82, 0.45, 0.08)
        );
    } else if (archetype == 5) { // RochesterDaylight200
        let dir = applyDirChemicalCoupling(curRgb, -0.02, 0.04, -0.06, 0.03, 0.04, -0.03);
        return vec3<f32>(
            applyFilmCharacteristicCurve(dir.r, 1.12, 0.22, 0.05),
            applyFilmCharacteristicCurve(dir.g, 1.08, 0.22, 0.08),
            applyFilmCharacteristicCurve(dir.b, 0.98, 0.24, 0.16)
        );
    } else if (archetype == 6) { // MobberleySilver400
        let w = getEnlargerFilterWeights(filterMode);
        let panLum = dot(curRgb, w);
        let mono = applyFilmCharacteristicCurve(panLum, 1.08, 0.30, 0.25);
        return vec3<f32>(mono);
    } else if (archetype == 7) { // CambridgeDiffusion100
        let dir = applyDirChemicalCoupling(curRgb, -0.03, 0.02, -0.04, 0.03, 0.01, -0.03);
        return vec3<f32>(
            applyFilmCharacteristicCurve(dir.r, 0.98, 0.35, 0.12),
            applyFilmCharacteristicCurve(dir.g, 0.98, 0.35, 0.12),
            applyFilmCharacteristicCurve(dir.b, 0.98, 0.35, 0.12)
        );
    } else if (archetype == 8) { // LeverkusenPortrait160
        let dir = applyDirChemicalCoupling(curRgb, -0.05, 0.01, -0.07, 0.02, 0.02, -0.04);
        return vec3<f32>(
            applyFilmCharacteristicCurve(dir.r, 0.96, 0.32, 0.08),
            applyFilmCharacteristicCurve(dir.g, 0.94, 0.32, 0.03),
            applyFilmCharacteristicCurve(dir.b, 0.92, 0.30, 0.10)
        );
    } else if (archetype == 9) { // AshigaraPortrait400
        let dir = applyDirChemicalCoupling(curRgb, -0.06, 0.02, -0.14, 0.05, 0.04, -0.08);
        return vec3<f32>(
            applyFilmCharacteristicCurve(dir.r, 0.92, 0.36, 0.14),
            applyFilmCharacteristicCurve(dir.g, 0.92, 0.36, 0.04),
            applyFilmCharacteristicCurve(dir.b, 0.94, 0.34, 0.04)
        );
    } else if (archetype == 10) { // LiguriaCinema80
        let w = getEnlargerFilterWeights(filterMode);
        let panLum = dot(curRgb, w);
        let mono = applyEmulsionSCurve(panLum, 0.32, -0.08);
        return vec3<f32>(mono);
    } else if (archetype == 11) { // BaodingSilver100
        let w = getEnlargerFilterWeights(filterMode);
        let panLum = dot(curRgb, w);
        let mono = applyEmulsionSCurve(panLum, 0.18, 0.10);
        return vec3<f32>(mono);
    }
    return rgb;
}
