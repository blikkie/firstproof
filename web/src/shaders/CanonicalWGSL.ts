// AUTO-GENERATED from canonical WGSL by firstproof-engine (Story 4.8)
// DO NOT EDIT DIRECTLY. Edit firstproof-engine/shaders/*.wgsl instead.

export const CANONICAL_DARKROOM_WGSL = `// Canonical Darkroom Shader Uniforms & Vertex Structures
// Guaranteed bit-identical alignment across WebGPU, Metal Shading Language, and WebGL 2 GLSL ES 300.

struct ShaderUniforms {
    exposure: f32,
    temperature: f32,
    tint: f32,
    contrast: f32,
    highlights: f32,
    shadows: f32,
    whites: f32,
    blacks: f32,
    vibrance: f32,
    saturation: f32,
    deepBlue: f32,
    foliage: f32,
    straightenAngle: f32,
    aspectRatio: f32,
    lensCorrectionEnabled: i32,
    distortionModel: i32,
    k1: f32,
    k2: f32,
    k3: f32,
    k4: f32,
    isOriginal: f32,
    isLinearRaw: i32,
    toneProfileMode: i32,
    lutEnabled: i32,
    lutIntensity: f32,
    lutSize: f32,
    sharpeningAmount: f32,
    sharpeningRadius: f32,
    sharpeningMasking: f32,
    noiseReduction: f32,
    baselineSharpness: f32,
    cropLeft: f32,
    cropTop: f32,
    cropRight: f32,
    cropBottom: f32,
    perspectiveVertical: f32,
    perspectiveHorizontal: f32,
    cropConstrain: i32,
    cropModeActive: i32,
    hasCurves: i32,
    hasMasterCurve: i32,
    hasRgbCurves: i32,
    edrHeadroom: f32,
    edrEnabled: i32,
    hasGainMap: i32,
    gainMapMin: f32,
    gainMapMax: f32,
    gainMapGamma: f32,
    gainMapCapacityMax: f32,
    emulsionArchetype: i32,
    emulsionFormat: i32,
    emulsionExposure: f32,
    emulsionGrainMode: i32,
    grainScale: f32,
    grainSize: f32,
    grainAmount: f32,
    isIntermediate: i32,
    splitToningShadowHue: f32,
    splitToningShadowSat: f32,
    splitToningHighlightHue: f32,
    splitToningHighlightSat: f32,
    splitToningBalance: f32,
    printMediaMode: i32,
    enlargerFilterMode: i32,
    printPaperGrade: f32,
    printBaseWarmth: f32,
    printProcessDepth: f32,
    halationMode: i32,
    rotation: i32,
    asShotTemperature: f32,
    asShotTint: f32,
    hasTca: i32,
    tcaVr: f32,
    tcaVb: f32,
    tcaBr: f32,
    tcaBb: f32,
    tcaCr: f32,
    tcaCb: f32,
    defringeAmount: f32,
    tcaCorrectionEnabled: i32,
    clippingOverlayActive: i32,
    pad0: f32,
    pad1: f32,
    pad2: f32, // 16-byte std140/WebGPU alignment (84 fields * 4 bytes = 336 bytes = 21 * 16)
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texCoord: vec2<f32>,
};

struct VertexInput {
    position: vec2<f32>,
    texCoord: vec2<f32>,
};

struct VertexQuad {
    vertices: array<VertexInput, 4>,
};



// Canonical Darkroom Color Science & Optical Constants

// MARK: - Color Space Transformation Matrices (Column Vectors)
const LINEAR_SRGB_TO_REC2020 = mat3x3<f32>(
    vec3<f32>(0.6274041, 0.0690972, 0.0163914),
    vec3<f32>(0.3292821, 0.9195411, 0.0880133),
    vec3<f32>(0.0433138, 0.0113617, 0.8955953)
);

const REC2020_TO_LINEAR_SRGB = mat3x3<f32>(
    vec3<f32>( 1.6604910, -0.1245505, -0.0181508),
    vec3<f32>(-0.5876411,  1.1328999, -0.1005789),
    vec3<f32>(-0.0728499, -0.0083494,  1.1187297)
);

const LINEAR_SRGB_TO_DISPLAY_P3 = mat3x3<f32>(
    vec3<f32>(0.8224621, 0.0331941, 0.0170827),
    vec3<f32>(0.1775380, 0.9668058, 0.0723974),
    vec3<f32>(0.0000000, 0.0000000, 0.9105199)
);

// MARK: - AgX Filmic Tone Mapping Engine Constants & Operators
const AGX_INSET_MATRIX = mat3x3<f32>(
    vec3<f32>( 1.4096986,   0.13136263,  0.16151477),
    vec3<f32>(-0.4004796,   0.77151406, -0.06034984),
    vec3<f32>(-0.009218898, 0.09712328,  0.89883506)
);

const AGX_OUTSET_MATRIX = mat3x3<f32>(
    vec3<f32>( 1.1271005, -0.1413298, -0.1413298),
    vec3<f32>(-0.1106066,  1.1578237, -0.1106066),
    vec3<f32>(-0.0164939, -0.0164939,  1.2519364)
);

const AGX_MIN_EV: f32 = -10.0;
const AGX_MAX_EV: f32 = 6.5;
const AGX_EV_RANGE: f32 = 16.5;
const AGX_GAMMA: f32 = 2.2;
const AGX_MID_GRAY_NORM: f32 = 0.6060606;

// MARK: - Calibrated Camera-Matching Monotonic Cubic Hermite Tone Curves (Fritsch-Carlson C^1)
const DCP_KNOTS_STANDARD = array<vec2<f32>, 7>(
    vec2<f32>(0.00, 0.000), vec2<f32>(0.02, 0.008), vec2<f32>(0.05, 0.028),
    vec2<f32>(0.18, 0.190), vec2<f32>(0.50, 0.540), vec2<f32>(0.80, 0.840), vec2<f32>(1.00, 1.000)
);
const DCP_TANGENTS_STANDARD = array<f32, 7>(0.4000, 0.5333, 0.9564, 1.1700, 1.0469, 0.9000, 0.8000);

const DCP_KNOTS_PORTRAIT = array<vec2<f32>, 7>(
    vec2<f32>(0.00, 0.000), vec2<f32>(0.02, 0.010), vec2<f32>(0.05, 0.035),
    vec2<f32>(0.18, 0.185), vec2<f32>(0.50, 0.510), vec2<f32>(0.80, 0.820), vec2<f32>(1.00, 1.000)
);
const DCP_TANGENTS_PORTRAIT = array<f32, 7>(0.5000, 0.6667, 0.9936, 1.0847, 1.0245, 0.9667, 0.9000);

const DCP_KNOTS_LANDSCAPE = array<vec2<f32>, 7>(
    vec2<f32>(0.00, 0.000), vec2<f32>(0.02, 0.006), vec2<f32>(0.05, 0.022),
    vec2<f32>(0.18, 0.200), vec2<f32>(0.50, 0.560), vec2<f32>(0.80, 0.860), vec2<f32>(1.00, 1.000)
);
const DCP_TANGENTS_LANDSCAPE = array<f32, 7>(0.3000, 0.4167, 0.9513, 1.2471, 1.0625, 0.8500, 0.7000);

const DCP_KNOTS_VIVID = array<vec2<f32>, 7>(
    vec2<f32>(0.00, 0.000), vec2<f32>(0.02, 0.004), vec2<f32>(0.05, 0.018),
    vec2<f32>(0.18, 0.210), vec2<f32>(0.50, 0.580), vec2<f32>(0.80, 0.880), vec2<f32>(1.00, 1.000)
);
const DCP_TANGENTS_VIVID = array<f32, 7>(0.2000, 0.3333, 0.9718, 1.3166, 1.0781, 0.8000, 0.6000);

// MARK: - Calibrated Camera-Matching HueSatMaps (Memory Color & Optical Sensor Crosstalk Correction)
const DCP_HSM_STANDARD = array<vec3<f32>, 24>(
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 0 deg (Red)
    vec3<f32>(0.0, 1.03, 1.02), vec3<f32>(0.0, 1.05, 1.03),     // 30 deg (Orange/Skin)
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 60 deg (Yellow)
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 90 deg
    vec3<f32>(1.5, 1.06, 0.99), vec3<f32>(2.0, 1.10, 0.98),     // 120 deg (Green/Foliage)
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 150 deg
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 180 deg (Cyan)
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 210 deg (Sky Blue)
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 240 deg (Sensor Blue)
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 270 deg (Violet)
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 300 deg (Magenta)
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00)      // 330 deg (Rose)
);

const DCP_HSM_PORTRAIT = array<vec3<f32>, 24>(
    vec3<f32>(1.5, 0.96, 1.02), vec3<f32>(2.0, 0.94, 1.03),     // 0 deg (Red)
    vec3<f32>(0.0, 0.96, 1.03), vec3<f32>(0.0, 0.94, 1.04),     // 30 deg (Orange/Skin)
    vec3<f32>(-1.5, 0.96, 1.02), vec3<f32>(-2.0, 0.94, 1.03),   // 60 deg (Yellow)
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 90 deg
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 120 deg
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 150 deg
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 180 deg
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 210 deg (Sky Blue)
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 240 deg (Sensor Blue)
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 270 deg
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 300 deg
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00)      // 330 deg
);

const DCP_HSM_LANDSCAPE = array<vec3<f32>, 24>(
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 0 deg
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 30 deg
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 60 deg
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 90 deg
    vec3<f32>(2.5, 1.15, 0.97), vec3<f32>(3.5, 1.22, 0.95),     // 120 deg (Green/Foliage)
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 150 deg
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 180 deg
    vec3<f32>(0.0, 1.05, 1.00), vec3<f32>(0.0, 1.05, 1.00),     // 210 deg (Sky Blue)
    vec3<f32>(0.0, 1.05, 1.00), vec3<f32>(0.0, 1.05, 1.00),     // 240 deg (Sensor Blue)
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 270 deg
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00),     // 300 deg
    vec3<f32>(0.0, 1.00, 1.00), vec3<f32>(0.0, 1.00, 1.00)      // 330 deg
);

const DCP_HSM_VIVID = array<vec3<f32>, 24>(
    vec3<f32>(0.0, 1.15, 1.02), vec3<f32>(0.0, 1.15, 1.02),     // 0 deg
    vec3<f32>(0.0, 1.15, 1.02), vec3<f32>(0.0, 1.15, 1.02),     // 30 deg
    vec3<f32>(0.0, 1.15, 1.02), vec3<f32>(0.0, 1.15, 1.02),     // 60 deg
    vec3<f32>(0.0, 1.15, 1.02), vec3<f32>(0.0, 1.15, 1.02),     // 90 deg
    vec3<f32>(0.0, 1.15, 1.02), vec3<f32>(2.0, 1.25, 0.96),     // 120 deg (Green/Foliage)
    vec3<f32>(0.0, 1.15, 1.02), vec3<f32>(0.0, 1.15, 1.02),     // 150 deg
    vec3<f32>(0.0, 1.15, 1.02), vec3<f32>(0.0, 1.15, 1.02),     // 180 deg
    vec3<f32>(0.0, 1.15, 1.02), vec3<f32>(0.0, 1.15, 1.02),     // 210 deg (Sky Blue)
    vec3<f32>(0.0, 1.15, 1.02), vec3<f32>(0.0, 1.15, 1.02),     // 240 deg (Sensor Blue)
    vec3<f32>(0.0, 1.15, 1.02), vec3<f32>(0.0, 1.15, 1.02),     // 270 deg
    vec3<f32>(0.0, 1.15, 1.02), vec3<f32>(0.0, 1.15, 1.02),     // 300 deg
    vec3<f32>(0.0, 1.15, 1.02), vec3<f32>(0.0, 1.15, 1.02)      // 330 deg
);

// MARK: - Optical Halation 16-Tap Fermat Golden Spiral Disk & Weights
const FERMAT_DISK = array<vec2<f32>, 16>(
    vec2<f32>( 0.1768,  0.0000), vec2<f32>(-0.2258,  0.2068), vec2<f32>( 0.0346, -0.3938), vec2<f32>( 0.2846,  0.3712),
    vec2<f32>(-0.5222, -0.0924), vec2<f32>( 0.4947, -0.3147), vec2<f32>(-0.1655,  0.6155), vec2<f32>(-0.3156, -0.6076),
    vec2<f32>( 0.6846,  0.2500), vec2<f32>(-0.7123,  0.2940), vec2<f32>( 0.3434, -0.7337), vec2<f32>( 0.2537,  0.8089),
    vec2<f32>(-0.7647, -0.4432), vec2<f32>( 0.8971, -0.1972), vec2<f32>(-0.5475,  0.7788), vec2<f32>(-0.1265, -0.9761)
);

const FERMAT_WEIGHTS = array<f32, 16>(
    0.6428, 0.4651, 0.3722, 0.3106,
    0.2656, 0.2309, 0.2032, 0.1806,
    0.1617, 0.1457, 0.1320, 0.1201,
    0.1097, 0.1006, 0.0926, 0.0854
);

// Canonical Viewport Geometry, Perspective & Lens Distortion Transform

struct GeometryCoordinates {
    uv: vec2<f32>,
    uvRed: vec2<f32>,
    uvBlue: vec2<f32>,
};

fn transformGeometryChannels(uv: vec2<f32>, uniforms: ShaderUniforms) -> GeometryCoordinates {
    var coords: GeometryCoordinates;
    if (uniforms.isOriginal >= 0.5 || uniforms.isIntermediate == 1) {
        coords.uv = uv;
        coords.uvRed = uv;
        coords.uvBlue = uv;
        return coords;
    }
    
    let cropL = clamp(uniforms.cropLeft, 0.0, 0.99);
    let cropT = clamp(uniforms.cropTop, 0.0, 0.99);
    let cropR = clamp(uniforms.cropRight, cropL + 0.01, 1.0);
    let cropB = clamp(uniforms.cropBottom, cropT + 0.01, 1.0);
    
    var uvc = uv;
    if (uniforms.cropModeActive == 0) {
        uvc.x = cropL + uv.x * (cropR - cropL);
        uvc.y = cropT + uv.y * (cropB - cropT);
    }
    
    let A = max(uniforms.aspectRatio, 0.01);
    let angle = uniforms.straightenAngle;
    let cosA = cos(angle);
    let sinA = sin(angle);
    // Perspective keystone coefficients: 0.005 scales [-100, +100] to a max backward homography tilt coefficient of [-0.5, +0.5] (~28.6 deg)
    let cV = uniforms.perspectiveVertical * 0.005;
    let cH = uniforms.perspectiveHorizontal * 0.005;
    
    var autoCropScale: f32 = 1.0;
    if (uniforms.cropConstrain != 0 || uniforms.lensCorrectionEnabled != 0) {
        let testPoints = array<vec2<f32>, 12>(
            vec2<f32>( A,  1.0), vec2<f32>(-A,  1.0),
            vec2<f32>( A, -1.0), vec2<f32>(-A, -1.0),
            vec2<f32>( A,  0.0), vec2<f32>(-A,  0.0),
            vec2<f32>(0.0,  1.0), vec2<f32>(0.0, -1.0),
            vec2<f32>( A,  0.5), vec2<f32>(-A,  0.5),
            vec2<f32>( A, -0.5), vec2<f32>(-A, -0.5)
        );
        for (var i: i32 = 0; i < 12; i = i + 1) {
            let p = testPoints[i];
            let r = vec2<f32>(
                p.x * cosA - p.y * sinA,
                p.x * sinA + p.y * cosA
            );
            let denom = max(1.0 + cV * r.y + cH * r.x, 0.1);
            var g = r / denom;
            
            if (uniforms.lensCorrectionEnabled != 0) {
                let r2 = dot(g, g);
                let rad = sqrt(r2);
                if (rad > 0.0001) {
                    var r_corr: f32 = rad;
                    if (uniforms.distortionModel == 0) {
                        r_corr = rad * (1.0 + uniforms.k1 * r2 + uniforms.k2 * (r2 * r2));
                    } else if (uniforms.distortionModel == 1) {
                        let a = uniforms.k1;
                        let b = uniforms.k2;
                        let c = uniforms.k3;
                        let d = 1.0 - (a + b + c);
                        r_corr = a * (r2 * r2) + b * (r2 * rad) + c * r2 + d * rad;
                    }
                    g = g * (r_corr / rad);
                }
            }
            
            let req = max(abs(g.x) / A, abs(g.y));
            if (req > autoCropScale) {
                autoCropScale = req;
            }
        }
    }
    
    // Centered coordinates scaled by aspect ratio
    var centered = (uvc - vec2<f32>(0.5, 0.5)) * 2.0;
    centered.x = centered.x * A;
    centered = centered / autoCropScale;
    
    // Straighten rotation
    let rot = vec2<f32>(
        centered.x * cosA - centered.y * sinA,
        centered.x * sinA + centered.y * cosA
    );
    
    // Perspective keystone (backward projective homography)
    let denom = max(1.0 + cV * rot.y + cH * rot.x, 0.1);
    var geom = rot / denom;
    
    // Lens distortion correction
    if (uniforms.lensCorrectionEnabled != 0) {
        let r2 = dot(geom, geom);
        let r = sqrt(r2);
        if (r > 0.0001) {
            var r_corr: f32 = r;
            if (uniforms.distortionModel == 0) {
                r_corr = r * (1.0 + uniforms.k1 * r2 + uniforms.k2 * (r2 * r2));
            } else if (uniforms.distortionModel == 1) {
                let a = uniforms.k1;
                let b = uniforms.k2;
                let c = uniforms.k3;
                let d = 1.0 - (a + b + c);
                r_corr = a * (r2 * r2) + b * (r2 * r) + c * r2 + d * r;
            }
            geom = geom * (r_corr / r);
        }
    }
    
    var geom_r = geom;
    var geom_b = geom;
    
    // Transverse Chromatic Aberration (TCA) polynomial radial scaling: r_d = (v + b*r^2 + c*r^4) * r
    if (uniforms.tcaCorrectionEnabled != 0 && uniforms.hasTca != 0) {
        let r2 = dot(geom, geom);
        let r4 = r2 * r2;
        let scale_r = uniforms.tcaVr + uniforms.tcaBr * r2 + uniforms.tcaCr * r4;
        let scale_b = uniforms.tcaVb + uniforms.tcaBb * r2 + uniforms.tcaCb * r4;
        geom_r = geom * scale_r;
        geom_b = geom * scale_b;
    }
    
    let denomA = 2.0 * A;
    coords.uv = vec2<f32>(geom.x / denomA + 0.5, geom.y * 0.5 + 0.5);
    coords.uvRed = vec2<f32>(geom_r.x / denomA + 0.5, geom_r.y * 0.5 + 0.5);
    coords.uvBlue = vec2<f32>(geom_b.x / denomA + 0.5, geom_b.y * 0.5 + 0.5);
    return coords;
}

fn transformGeometry(uv: vec2<f32>, uniforms: ShaderUniforms) -> vec2<f32> {
    return transformGeometryChannels(uv, uniforms).uv;
}

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
                          (uniforms.printMediaMode == 0);
    
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
            let isSensorClip = (uniforms.isLinearRaw == 1 && (sampledColor.r >= 1.0 || sampledColor.g >= 1.0 || sampledColor.b >= 1.0)) || max(finalColor.r, max(finalColor.g, finalColor.b)) >= headroom * 0.999;
            let sceneLum = dot(rec2020Color, vec3<f32>(0.2627, 0.6780, 0.0593));
            let isShadowClip = sceneLum <= 0.0007 || max(finalColor.r, max(finalColor.g, finalColor.b)) <= 0.0005;
            if (isSensorClip) {
                return vec4<f32>(1.0, 0.0, 0.0, sampledColor.a);
            } else if (isShadowClip) {
                return vec4<f32>(0.0, 0.25, 1.0, sampledColor.a);
            }
        }
        return vec4<f32>(finalColor, sampledColor.a);
    } else {
        finalColor = clamp(finalColor, vec3<f32>(0.0), vec3<f32>(1.0));
        if (uniforms.clippingOverlayActive != 0 && uniforms.isOriginal < 0.5) {
            let isSensorClip = (uniforms.isLinearRaw == 1 && (sampledColor.r >= 1.0 || sampledColor.g >= 1.0 || sampledColor.b >= 1.0)) || max(finalColor.r, max(finalColor.g, finalColor.b)) >= 0.999;
            let sceneLum = dot(rec2020Color, vec3<f32>(0.2627, 0.6780, 0.0593));
            let isShadowClip = sceneLum <= 0.0007 || max(finalColor.r, max(finalColor.g, finalColor.b)) <= 0.0005;
            if (isSensorClip) {
                return vec4<f32>(1.0, 0.0, 0.0, sampledColor.a);
            } else if (isShadowClip) {
                return vec4<f32>(0.0, 0.25, 1.0, sampledColor.a);
            }
        }
        let dither = (ign(in.position.xy) - 0.5) / 255.0;
        return vec4<f32>(clamp(linearToSrgbGamma3(finalColor) + vec3<f32>(dither), vec3<f32>(0.0), vec3<f32>(1.0)), sampledColor.a);
    }
}
`;
