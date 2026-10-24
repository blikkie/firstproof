// Canonical Detail, Sharpening & Stochastic Optical Grain Engine

// MARK: - Detail, Sharpening & Noise Reduction (8-Tap Isotropic Discrete Laplacian)
fn applyDetailAndSharpening(
    color: vec3<f32>,
    y_c: f32,
    y_l: f32, y_r: f32, y_t: f32, y_b: f32,
    y_tl: f32, y_tr: f32, y_bl: f32, y_br: f32,
    amount: f32,
    masking: f32,
    noiseReduction: f32
) -> vec3<f32> {
    if (amount <= 0.0 && noiseReduction <= 0.0) {
        return color;
    }

    // Isotropic 9-point discrete Laplacian:
    // Orthogonal taps (dist 1) weighted at 1/6 each, diagonal taps (dist sqrt(2)) weighted at 1/12 each.
    let y_neighbors = (1.0 / 6.0) * (y_l + y_r + y_t + y_b)
        + (1.0 / 12.0) * (y_tl + y_tr + y_bl + y_br);
    let delta_y = y_c - y_neighbors;

    // Sobel-Feldman 8-neighbor gradient magnitude (isotropic edge acutance)
    let gx = ((y_tr + 2.0 * y_r + y_br) - (y_tl + 2.0 * y_l + y_bl)) * 0.125;
    let gy = ((y_bl + 2.0 * y_b + y_br) - (y_tl + 2.0 * y_t + y_tr)) * 0.125;
    let grad_mag = sqrt(gx * gx + gy * gy);

    // Edge masking weight:
    // When masking <= 0, w_edge = 1.0 (uniform sharpening across entire image).
    // When masking > 0, flat regions (grad_mag < threshold) are smoothly suppressed towards 0.
    var w_edge: f32 = 1.0;
    if (masking > 0.0) {
        let m = clamp(masking / 100.0, 0.0, 1.0);
        let threshold = pow(m, 1.5) * 0.08;
        w_edge = smoothstep(0.0, max(threshold, 1e-4), grad_mag);
    }

    // Photographically calibrated gain: up to 1.5x unsharp mask detail delta
    let sharp_gain = (amount / 100.0) * 1.5;
    let sharp_term = sharp_gain * w_edge * delta_y;
    let nr_term = (noiseReduction / 100.0) * (1.0 - w_edge) * delta_y;

    let delta_adj = sharp_term - nr_term;
    return clamp(color + vec3<f32>(delta_adj), vec3<f32>(0.0), vec3<f32>(1.0));
}

// MARK: - Longitudinal / Axial Purple Defringing
fn applyDefringe(
    color: vec3<f32>,
    y_c: f32,
    y_max_neighbor: f32,
    defringeAmount: f32
) -> vec3<f32> {
    if (defringeAmount <= 0.0) {
        return color;
    }
    
    // Only process if both Red and Blue exceed Green (purple/magenta chromaticity)
    let excess_b = color.b - color.g;
    let excess_r = color.r - color.g;
    if (excess_b <= 0.0 || excess_r <= 0.0) {
        return color;
    }
    
    // 1. Sensitometric Hue Qualification in Linear Rec.2020:
    // Hue angle in degrees: Red = 0, Yellow = 60, Green = 120, Cyan = 180, Blue = 240, Magenta/Purple = 300
    let u = 2.0 * color.r - color.g - color.b;
    let v = 1.7320508 * (color.g - color.b);
    var hue = atan2(v, u) * 57.2957795; // 180 / pi
    if (hue < 0.0) {
        hue = hue + 360.0;
    }
    
    // Smooth C1 Hermite trapezoid anchored to the physical purple fringe band (260 - 340 deg)
    let w_hue = smoothstep(250.0, 270.0, hue) * (1.0 - smoothstep(330.0, 350.0, hue));
    if (w_hue <= 0.001) {
        return color;
    }
    
    // 2. High-contrast specular border / edge qualification:
    // Fringe must be adjacent to a significantly brighter highlight region
    let edge_contrast = y_max_neighbor - y_c;
    let w_edge = smoothstep(0.04, 0.25, edge_contrast);
    let w_highlight = smoothstep(0.04, 0.25, y_max_neighbor);
    
    let w_fringe = w_hue * w_edge * w_highlight * (defringeAmount / 100.0);
    if (w_fringe <= 0.001) {
        return color;
    }
    
    // 3. Attenuate excess purple chrominance smoothly toward local green luminance
    let r_adj = color.r - w_fringe * excess_r;
    let b_adj = color.b - w_fringe * excess_b;
    
    return vec3<f32>(r_adj, color.g, b_adj);
}

// MARK: - Stochastic Emulsion Resynthesis & Physical Optical Grain Engine
fn proceduralGrainNoise(u: f32, v: f32) -> f32 {
    let p = 50.0 * fract(vec2<f32>(u, v) * 0.3183099 + vec2<f32>(0.71, 0.113));
    return -1.0 + 2.0 * fract(p.x * p.y * (p.x + p.y));
}

fn clusteredGrainNoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = proceduralGrainNoise(i.x, i.y);
    let b = proceduralGrainNoise(i.x + 1.0, i.y);
    let c = proceduralGrainNoise(i.x, i.y + 1.0);
    let d = proceduralGrainNoise(i.x + 1.0, i.y + 1.0);

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn evalGrainProcNoise(pos: vec2<f32>, s: f32, is_instant: bool) -> f32 {
    if (is_instant) {
        return 0.60 * clusteredGrainNoise(pos / s)
            + 0.40 * clusteredGrainNoise(pos / (s * 1.8) + vec2<f32>(17.3, 31.7));
    } else if (s > 1.0) {
        let t_cluster = clamp((s - 1.0) / 2.0, 0.0, 1.0);
        let cluster = clusteredGrainNoise(pos / s);
        let micro = proceduralGrainNoise(pos.x, pos.y);
        let clumped = micro * (0.60 + 0.40 * cluster) + 0.25 * cluster;
        return mix(micro, clumped, t_cluster);
    } else {
        return proceduralGrainNoise(pos.x, pos.y);
    }
}

fn evalGrainEta(proc_val: f32, sensor_seed: f32, grain_mode: i32, is_instant: bool) -> f32 {
    if (grain_mode == 1) { // SeedFromSensor
        if (is_instant) {
            return clamp(0.55 * proc_val + 0.20 * sensor_seed, -1.0, 1.0);
        } else {
            return clamp(proc_val * (0.45 + 0.55 * abs(sensor_seed)) + 0.45 * sensor_seed, -1.0, 1.0);
        }
    } else if (grain_mode == 2) { // SynthesizePhysical
        if (is_instant) {
            return clamp(0.85 * proc_val + 0.05 * sensor_seed, -1.0, 1.0);
        } else {
            return clamp(0.90 * proc_val + 0.10 * sensor_seed, -1.0, 1.0);
        }
    }
    return 0.0;
}

fn applyStochasticEmulsionGrain(
    color: vec3<f32>,
    y_c: f32,
    y_l: f32,
    y_r: f32,
    y_t: f32,
    y_b: f32,
    y_tl: f32,
    y_tr: f32,
    y_bl: f32,
    y_br: f32,
    p: vec2<f32>,
    grain_mode: i32,
    cluster_size: f32,
    grain_amount: f32,
    archetype: i32
) -> vec3<f32> {
    if (grain_mode == 0) {
        return color;
    }

    let amount = clamp(grain_amount, 0.0, 2.0);
    if (amount < 0.001) {
        return color;
    }

    // 1. Isotropic 9-point lowpass luminance
    let y_lowpass = (1.0 / 6.0) * (y_l + y_r + y_t + y_b)
        + (1.0 / 12.0) * (y_tl + y_tr + y_bl + y_br);
    let delta_sensor = y_c - y_lowpass;

    // Sobel-Feldman 8-neighbor gradient magnitude
    let gx = ((y_tr + 2.0 * y_r + y_br) - (y_tl + 2.0 * y_l + y_bl)) * 0.125;
    let gy = ((y_bl + 2.0 * y_b + y_br) - (y_tl + 2.0 * y_t + y_tr)) * 0.125;
    let grad_mag = sqrt(gx * gx + gy * gy);

    let edge_suppress = 1.0 / (1.0 + 35.0 * grad_mag);
    let delta_noise = delta_sensor * edge_suppress;

    let seed_gain: f32 = 20.0;
    let denom = max(sqrt(max(y_c, 0.0)), 0.05);
    let sensor_seed = clamp((delta_noise / denom) * seed_gain, -1.0, 1.0);

    // 2. Stochastic carrier: Polaroid DDT dye diffusion mottling vs silver crystal lattice
    let is_instant = (archetype == 7);
    let s = max(cluster_size, 0.25);
    let proc_common = evalGrainProcNoise(p, s, is_instant);
    let eta_common = evalGrainEta(proc_common, sensor_seed, grain_mode, is_instant);

    // 3. Nutting midtone density weighting: G(Y) = 4 * Y * (1 - Y)
    let y_clamped = clamp(y_c, 0.0, 1.0);
    let nutting = 4.0 * y_clamped * (1.0 - y_clamped);

    // 4. Optical transmittance modulation
    let y_pixel = dot(color, vec3<f32>(0.2627, 0.6780, 0.0593));

    let is_monochrome = (archetype == 6 || archetype == 10 || archetype == 11);
    if (is_monochrome) {
        // Metallic Silver: Pure achromatic optical density modulation (R = G = B)
        let delta = eta_common * nutting * 0.28 * amount;
        let y_res = max(y_pixel * (1.0 + delta), 0.0);
        return vec3<f32>(y_res);
    } else {
        // Subtractive 3-Layer Dye Cloud Decorrelation (Cyan -> Red, Magenta -> Green, Yellow -> Blue)
        let proc_r = evalGrainProcNoise(p + vec2<f32>(17.3, 31.7), s, is_instant);
        let proc_g = evalGrainProcNoise(p + vec2<f32>(73.1, 19.9), s, is_instant);
        let proc_b = evalGrainProcNoise(p + vec2<f32>(43.7, 89.3), s, is_instant);

        let eta_r = evalGrainEta(proc_r, sensor_seed, grain_mode, is_instant);
        let eta_g = evalGrainEta(proc_g, sensor_seed, grain_mode, is_instant);
        let eta_b = evalGrainEta(proc_b, sensor_seed, grain_mode, is_instant);

        // Mix 75% common photopic luminance carrier with 25% independent subtractive layer phase
        // Energy-preserving Euclidean normalization: 1.0 / sqrt(0.75^2 + 0.25^2) ≈ 1.264911
        let beta: f32 = 0.25;
        let norm: f32 = 1.264911;
        let eta_final_r = mix(eta_common, eta_r, beta) * norm;
        let eta_final_g = mix(eta_common, eta_g, beta) * norm;
        let eta_final_b = mix(eta_common, eta_b, beta) * norm;

        let scale = nutting * 0.28 * amount;
        let delta_r = eta_final_r * scale;
        let delta_g = eta_final_g * scale;
        let delta_b = eta_final_b * scale;

        let chroma_damp = 1.0 - 0.10 * nutting;
        let r_c = y_pixel + chroma_damp * (color.r - y_pixel);
        let g_c = y_pixel + chroma_damp * (color.g - y_pixel);
        let b_c = y_pixel + chroma_damp * (color.b - y_pixel);

        let r_res = max(r_c * (1.0 + delta_r), 0.0);
        let g_res = max(g_c * (1.0 + delta_g), 0.0);
        let b_res = max(b_c * (1.0 + delta_b), 0.0);

        return vec3<f32>(r_res, g_res, b_res);
    }
}
