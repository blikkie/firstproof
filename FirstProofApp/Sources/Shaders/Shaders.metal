// AUTO-GENERATED from canonical WGSL by firstproof-engine (Story 4.8)
// DO NOT EDIT DIRECTLY. Edit firstproof-engine/shaders/*.wgsl instead.

// language: metal1.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct ShaderUniforms {
    float exposure;
    float temperature;
    float tint;
    float contrast;
    float highlights;
    float shadows;
    float whites;
    float blacks;
    float vibrance;
    float saturation;
    float deepBlue;
    float foliage;
    float straightenAngle;
    float aspectRatio;
    int lensCorrectionEnabled;
    int distortionModel;
    float k1_;
    float k2_;
    float k3_;
    float k4_;
    float isOriginal;
    int isLinearRaw;
    int toneProfileMode;
    int lutEnabled;
    float lutIntensity;
    float lutSize;
    float sharpeningAmount;
    float sharpeningRadius;
    float sharpeningMasking;
    float noiseReduction;
    float baselineSharpness;
    float cropLeft;
    float cropTop;
    float cropRight;
    float cropBottom;
    float perspectiveVertical;
    float perspectiveHorizontal;
    int cropConstrain;
    int cropModeActive;
    int hasCurves;
    int hasMasterCurve;
    int hasRgbCurves;
    float edrHeadroom;
    int edrEnabled;
    int hasGainMap;
    float gainMapMin;
    float gainMapMax;
    float gainMapGamma;
    float gainMapCapacityMax;
    int emulsionArchetype;
    int emulsionFormat;
    float emulsionExposure;
    int emulsionGrainMode;
    float grainScale;
    float grainSize;
    float grainAmount;
    int isIntermediate;
    float splitToningShadowHue;
    float splitToningShadowSat;
    float splitToningHighlightHue;
    float splitToningHighlightSat;
    float splitToningBalance;
    int printMediaMode;
    int enlargerFilterMode;
    float printPaperGrade;
    float printBaseWarmth;
    float printProcessDepth;
    int halationMode;
    int rotation;
    float asShotTemperature;
    float asShotTint;
    int hasTca;
    float tcaVr;
    float tcaVb;
    float tcaBr;
    float tcaBb;
    float tcaCr;
    float tcaCb;
    float defringeAmount;
    int tcaCorrectionEnabled;
    int clippingOverlayActive;
    float pad0_;
    float pad1_;
    float pad2_;
};
struct VertexOutput {
    metal::float4 position;
    metal::float2 texCoord;
    char _pad2[8];
};
struct VertexInput {
    metal::float2 position;
    metal::float2 texCoord;
};
struct type_4 {
    VertexInput inner[4];
};
struct VertexQuad {
    type_4 vertices;
};
struct type_7 {
    metal::float2 inner[7];
};
struct type_8 {
    float inner[7];
};
struct type_9 {
    metal::float3 inner[24];
};
struct type_10 {
    metal::float2 inner[16];
};
struct type_11 {
    float inner[16];
};
struct GeometryCoordinates {
    metal::float2 uv;
    metal::float2 uvRed;
    metal::float2 uvBlue;
};
struct type_13 {
    metal::float2 inner[12];
};
struct type_19 {
    metal::float4 inner[16];
};
constant metal::float3x3 LINEAR_SRGB_TO_REC2020_ = metal::float3x3(metal::float3(0.6274041, 0.0690972, 0.0163914), metal::float3(0.3292821, 0.9195411, 0.0880133), metal::float3(0.0433138, 0.0113617, 0.8955953));
constant metal::float3x3 REC2020_TO_LINEAR_SRGB = metal::float3x3(metal::float3(1.660491, -0.1245505, -0.0181508), metal::float3(-0.5876411, 1.1328999, -0.1005789), metal::float3(-0.0728499, -0.0083494, 1.1187297));
constant metal::float3x3 LINEAR_SRGB_TO_DISPLAY_P3_ = metal::float3x3(metal::float3(0.8224621, 0.0331941, 0.0170827), metal::float3(0.177538, 0.9668058, 0.0723974), metal::float3(0.0, 0.0, 0.9105199));
constant metal::float3x3 AGX_INSET_MATRIX = metal::float3x3(metal::float3(1.4096986, 0.13136263, 0.16151477), metal::float3(-0.4004796, 0.77151406, -0.06034984), metal::float3(-0.009218898, 0.09712328, 0.89883506));
constant metal::float3x3 AGX_OUTSET_MATRIX = metal::float3x3(metal::float3(1.1271005, -0.1413298, -0.1413298), metal::float3(-0.1106066, 1.1578237, -0.1106066), metal::float3(-0.0164939, -0.0164939, 1.2519364));
constant float AGX_MIN_EV = -10.0;
constant float AGX_MAX_EV = 6.5;
constant float AGX_EV_RANGE = 16.5;
constant float AGX_GAMMA = 2.2;
constant float AGX_MID_GRAY_NORM = 0.6060606;
constant type_7 DCP_KNOTS_STANDARD = type_7 {{metal::float2(0.0, 0.0), metal::float2(0.02, 0.008), metal::float2(0.05, 0.028), metal::float2(0.18, 0.19), metal::float2(0.5, 0.54), metal::float2(0.8, 0.84), metal::float2(1.0, 1.0)}};
constant type_8 DCP_TANGENTS_STANDARD = type_8 {{0.4, 0.5333, 0.9564, 1.17, 1.0469, 0.9, 0.8}};
constant type_7 DCP_KNOTS_PORTRAIT = type_7 {{metal::float2(0.0, 0.0), metal::float2(0.02, 0.01), metal::float2(0.05, 0.035), metal::float2(0.18, 0.185), metal::float2(0.5, 0.51), metal::float2(0.8, 0.82), metal::float2(1.0, 1.0)}};
constant type_8 DCP_TANGENTS_PORTRAIT = type_8 {{0.5, 0.6667, 0.9936, 1.0847, 1.0245, 0.9667, 0.9}};
constant type_7 DCP_KNOTS_LANDSCAPE = type_7 {{metal::float2(0.0, 0.0), metal::float2(0.02, 0.006), metal::float2(0.05, 0.022), metal::float2(0.18, 0.2), metal::float2(0.5, 0.56), metal::float2(0.8, 0.86), metal::float2(1.0, 1.0)}};
constant type_8 DCP_TANGENTS_LANDSCAPE = type_8 {{0.3, 0.4167, 0.9513, 1.2471, 1.0625, 0.85, 0.7}};
constant type_7 DCP_KNOTS_VIVID = type_7 {{metal::float2(0.0, 0.0), metal::float2(0.02, 0.004), metal::float2(0.05, 0.018), metal::float2(0.18, 0.21), metal::float2(0.5, 0.58), metal::float2(0.8, 0.88), metal::float2(1.0, 1.0)}};
constant type_8 DCP_TANGENTS_VIVID = type_8 {{0.2, 0.3333, 0.9718, 1.3166, 1.0781, 0.8, 0.6}};
constant type_9 DCP_HSM_STANDARD = type_9 {{metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.03, 1.02), metal::float3(0.0, 1.05, 1.03), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(1.5, 1.06, 0.99), metal::float3(2.0, 1.1, 0.98), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0)}};
constant type_9 DCP_HSM_PORTRAIT = type_9 {{metal::float3(1.5, 0.96, 1.02), metal::float3(2.0, 0.94, 1.03), metal::float3(0.0, 0.96, 1.03), metal::float3(0.0, 0.94, 1.04), metal::float3(-1.5, 0.96, 1.02), metal::float3(-2.0, 0.94, 1.03), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0)}};
constant type_9 DCP_HSM_LANDSCAPE = type_9 {{metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(2.5, 1.15, 0.97), metal::float3(3.5, 1.22, 0.95), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.05, 1.0), metal::float3(0.0, 1.05, 1.0), metal::float3(0.0, 1.05, 1.0), metal::float3(0.0, 1.05, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0), metal::float3(0.0, 1.0, 1.0)}};
constant type_9 DCP_HSM_VIVID = type_9 {{metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(2.0, 1.25, 0.96), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02), metal::float3(0.0, 1.15, 1.02)}};
constant type_10 FERMAT_DISK = type_10 {{metal::float2(0.1768, 0.0), metal::float2(-0.2258, 0.2068), metal::float2(0.0346, -0.3938), metal::float2(0.2846, 0.3712), metal::float2(-0.5222, -0.0924), metal::float2(0.4947, -0.3147), metal::float2(-0.1655, 0.6155), metal::float2(-0.3156, -0.6076), metal::float2(0.6846, 0.25), metal::float2(-0.7123, 0.294), metal::float2(0.3434, -0.7337), metal::float2(0.2537, 0.8089), metal::float2(-0.7647, -0.4432), metal::float2(0.8971, -0.1972), metal::float2(-0.5475, 0.7788), metal::float2(-0.1265, -0.9761)}};
constant type_11 FERMAT_WEIGHTS = type_11 {{0.6428, 0.4651, 0.3722, 0.3106, 0.2656, 0.2309, 0.2032, 0.1806, 0.1617, 0.1457, 0.132, 0.1201, 0.1097, 0.1006, 0.0926, 0.0854}};

GeometryCoordinates transformGeometryChannels(
    metal::float2 uv,
    ShaderUniforms uniforms
) {
    GeometryCoordinates coords = {};
    bool local_65 = {};
    metal::float2 uvc = {};
    float autoCropScale = 1.0;
    bool local_66 = {};
    int i_2 = 0;
    metal::float2 g = {};
    float r_corr = {};
    metal::float2 centered = {};
    metal::float2 geom = {};
    float r_corr_1 = {};
    metal::float2 geom_r = {};
    metal::float2 geom_b = {};
    bool local_67 = {};
    if (!((uniforms.isOriginal >= 0.5))) {
        local_65 = uniforms.isIntermediate == 1;
    } else {
        local_65 = true;
    }
    bool _e13 = local_65;
    if (_e13) {
        coords.uv = uv;
        coords.uvRed = uv;
        coords.uvBlue = uv;
        GeometryCoordinates _e17 = coords;
        return _e17;
    }
    float cropL = metal::clamp(uniforms.cropLeft, 0.0, 0.99);
    float cropT = metal::clamp(uniforms.cropTop, 0.0, 0.99);
    float cropR = metal::clamp(uniforms.cropRight, cropL + 0.01, 1.0);
    float cropB = metal::clamp(uniforms.cropBottom, cropT + 0.01, 1.0);
    uvc = uv;
    if (uniforms.cropModeActive == 0) {
        uvc.x = cropL + (uv.x * (cropR - cropL));
        uvc.y = cropT + (uv.y * (cropB - cropT));
    }
    float A = metal::max(uniforms.aspectRatio, 0.01);
    float angle = uniforms.straightenAngle;
    float cosA = metal::cos(angle);
    float sinA = metal::sin(angle);
    float cV = uniforms.perspectiveVertical * 0.005;
    float cH = uniforms.perspectiveHorizontal * 0.005;
    if (!((uniforms.cropConstrain != 0))) {
        local_66 = uniforms.lensCorrectionEnabled != 0;
    } else {
        local_66 = true;
    }
    bool _e74 = local_66;
    if (_e74) {
        type_13 testPoints = type_13 {{metal::float2(A, 1.0), metal::float2(-(A), 1.0), metal::float2(A, -1.0), metal::float2(-(A), -1.0), metal::float2(A, 0.0), metal::float2(-(A), 0.0), metal::float2(0.0, 1.0), metal::float2(0.0, -1.0), metal::float2(A, 0.5), metal::float2(-(A), 0.5), metal::float2(A, -0.5), metal::float2(-(A), -0.5)}};
        uint2 loop_bound = uint2(4294967295u);
        bool loop_init = true;
        while(true) {
            if (metal::all(loop_bound == uint2(0u))) { break; }
            loop_bound -= uint2(loop_bound.y == 0u, 1u);
            if (!loop_init) {
                int _e192 = i_2;
                i_2 = as_type<int>(as_type<uint>(_e192) + as_type<uint>(1));
            }
            loop_init = false;
            int _e109 = i_2;
            if (_e109 < 12) {
            } else {
                break;
            }
            {
                int _e112 = i_2;
                metal::float2 p_3 = testPoints.inner[_e112];
                metal::float2 r = metal::float2((testPoints.inner[_e112].x * cosA) - (testPoints.inner[_e112].y * sinA), (testPoints.inner[_e112].x * sinA) + (testPoints.inner[_e112].y * cosA));
                float denom = metal::max((1.0 + (cV * r.y)) + (cH * r.x), 0.1);
                g = r / metal::float2(denom);
                if (uniforms.lensCorrectionEnabled != 0) {
                    metal::float2 _e140 = g;
                    metal::float2 _e141 = g;
                    float r2_ = metal::dot(_e140, _e141);
                    float rad = metal::sqrt(r2_);
                    if (rad > 0.0001) {
                        r_corr = rad;
                        if (uniforms.distortionModel == 0) {
                            r_corr = rad * ((1.0 + (uniforms.k1_ * r2_)) + (uniforms.k2_ * (r2_ * r2_)));
                        } else {
                            if (uniforms.distortionModel == 1) {
                                float a = uniforms.k1_;
                                float b = uniforms.k2_;
                                float c_9 = uniforms.k3_;
                                float d = 1.0 - ((a + b) + c_9);
                                r_corr = (((a * (r2_ * r2_)) + (b * (r2_ * rad))) + (c_9 * r2_)) + (d * rad);
                            }
                        }
                        metal::float2 _e178 = g;
                        float _e179 = r_corr;
                        g = _e178 * (_e179 / rad);
                    }
                }
                float _e183 = g.x;
                float _e187 = g.y;
                float req = metal::max(metal::abs(_e183) / A, metal::abs(_e187));
                float _e190 = autoCropScale;
                if (req > _e190) {
                    autoCropScale = req;
                }
            }
        }
    }
    metal::float2 _e195 = uvc;
    centered = (_e195 - metal::float2(0.5, 0.5)) * 2.0;
    float _e205 = centered.x;
    centered.x = _e205 * A;
    metal::float2 _e207 = centered;
    float _e208 = autoCropScale;
    centered = _e207 / metal::float2(_e208);
    float _e212 = centered.x;
    float _e215 = centered.y;
    float _e219 = centered.x;
    float _e222 = centered.y;
    metal::float2 rot = metal::float2((_e212 * cosA) - (_e215 * sinA), (_e219 * sinA) + (_e222 * cosA));
    float denom_1 = metal::max((1.0 + (cV * rot.y)) + (cH * rot.x), 0.1);
    geom = rot / metal::float2(denom_1);
    if (uniforms.lensCorrectionEnabled != 0) {
        metal::float2 _e241 = geom;
        metal::float2 _e242 = geom;
        float r2_1 = metal::dot(_e241, _e242);
        float r_1 = metal::sqrt(r2_1);
        if (r_1 > 0.0001) {
            r_corr_1 = r_1;
            if (uniforms.distortionModel == 0) {
                r_corr_1 = r_1 * ((1.0 + (uniforms.k1_ * r2_1)) + (uniforms.k2_ * (r2_1 * r2_1)));
            } else {
                if (uniforms.distortionModel == 1) {
                    float a_1 = uniforms.k1_;
                    float b_1 = uniforms.k2_;
                    float c_10 = uniforms.k3_;
                    float d_1 = 1.0 - ((a_1 + b_1) + c_10);
                    r_corr_1 = (((a_1 * (r2_1 * r2_1)) + (b_1 * (r2_1 * r_1))) + (c_10 * r2_1)) + (d_1 * r_1);
                }
            }
            metal::float2 _e279 = geom;
            float _e280 = r_corr_1;
            geom = _e279 * (_e280 / r_1);
        }
    }
    metal::float2 _e283 = geom;
    geom_r = _e283;
    metal::float2 _e285 = geom;
    geom_b = _e285;
    if (uniforms.tcaCorrectionEnabled != 0) {
        local_67 = uniforms.hasTca != 0;
    } else {
        local_67 = false;
    }
    bool _e296 = local_67;
    if (_e296) {
        metal::float2 _e297 = geom;
        metal::float2 _e298 = geom;
        float r2_2 = metal::dot(_e297, _e298);
        float r4_ = r2_2 * r2_2;
        float scale_r = (uniforms.tcaVr + (uniforms.tcaBr * r2_2)) + (uniforms.tcaCr * r4_);
        float scale_b = (uniforms.tcaVb + (uniforms.tcaBb * r2_2)) + (uniforms.tcaCb * r4_);
        metal::float2 _e315 = geom;
        geom_r = _e315 * scale_r;
        metal::float2 _e317 = geom;
        geom_b = _e317 * scale_b;
    }
    float denomA = 2.0 * A;
    float _e323 = geom.x;
    float _e328 = geom.y;
    coords.uv = metal::float2((_e323 / denomA) + 0.5, (_e328 * 0.5) + 0.5);
    float _e336 = geom_r.x;
    float _e341 = geom_r.y;
    coords.uvRed = metal::float2((_e336 / denomA) + 0.5, (_e341 * 0.5) + 0.5);
    float _e349 = geom_b.x;
    float _e354 = geom_b.y;
    coords.uvBlue = metal::float2((_e349 / denomA) + 0.5, (_e354 * 0.5) + 0.5);
    GeometryCoordinates _e360 = coords;
    return _e360;
}

metal::float2 transformGeometry(
    metal::float2 uv_1,
    ShaderUniforms uniforms_1
) {
    GeometryCoordinates _e2 = transformGeometryChannels(uv_1, uniforms_1);
    return _e2.uv;
}

metal::float3 applyDetailAndSharpening(
    metal::float3 color,
    float y_c,
    float y_l_1,
    float y_r_1,
    float y_t_1,
    float y_b_1,
    float y_tl_1,
    float y_tr_1,
    float y_bl_1,
    float y_br_1,
    float amount,
    float masking,
    float noiseReduction
) {
    bool local_68 = {};
    float w_edge = 1.0;
    if (amount <= 0.0) {
        local_68 = noiseReduction <= 0.0;
    } else {
        local_68 = false;
    }
    bool _e20 = local_68;
    if (_e20) {
        return color;
    }
    float y_neighbors_2 = (0.16666667 * (((y_l_1 + y_r_1) + y_t_1) + y_b_1)) + (0.083333336 * (((y_tl_1 + y_tr_1) + y_bl_1) + y_br_1));
    float delta_y = y_c - y_neighbors_2;
    float gx = (((y_tr_1 + (2.0 * y_r_1)) + y_br_1) - ((y_tl_1 + (2.0 * y_l_1)) + y_bl_1)) * 0.125;
    float gy = (((y_bl_1 + (2.0 * y_b_1)) + y_br_1) - ((y_tl_1 + (2.0 * y_t_1)) + y_tr_1)) * 0.125;
    float grad_mag = metal::sqrt((gx * gx) + (gy * gy));
    if (masking > 0.0) {
        float m_1 = metal::clamp(masking / 100.0, 0.0, 1.0);
        float threshold = metal::pow(m_1, 1.5) * 0.08;
        w_edge = metal::smoothstep(0.0, metal::max(threshold, 0.0001), grad_mag);
    }
    float sharp_gain = (amount / 100.0) * 1.5;
    float _e80 = w_edge;
    float sharp_term = (sharp_gain * _e80) * delta_y;
    float _e85 = w_edge;
    float nr_term = ((noiseReduction / 100.0) * (1.0 - _e85)) * delta_y;
    float delta_adj = metal::max(sharp_term - nr_term, -0.75 * y_c);
    return metal::max(color + metal::float3(delta_adj), metal::float3(0.0));
}

metal::float3 applyDefringe(
    metal::float3 color_1,
    float y_c_1,
    float y_max_neighbor,
    float defringeAmount
) {
    bool local_69 = {};
    float hue = {};
    if (defringeAmount <= 0.0) {
        return color_1;
    }
    float excess_b = color_1.z - color_1.y;
    float excess_r = color_1.x - color_1.y;
    if (!((excess_b <= 0.0))) {
        local_69 = excess_r <= 0.0;
    } else {
        local_69 = true;
    }
    bool _e20 = local_69;
    if (_e20) {
        return color_1;
    }
    float u_1 = ((2.0 * color_1.x) - color_1.y) - color_1.z;
    float v_1 = 1.7320508 * (color_1.y - color_1.z);
    hue = metal::atan2(v_1, u_1) * 57.29578;
    float _e37 = hue;
    if (_e37 < 0.0) {
        float _e40 = hue;
        hue = _e40 + 360.0;
    }
    float _e43 = hue;
    float _e47 = hue;
    float w_hue = metal::smoothstep(250.0, 270.0, _e43) * (1.0 - metal::smoothstep(330.0, 350.0, _e47));
    if (w_hue <= 0.001) {
        return color_1;
    }
    float edge_contrast = y_max_neighbor - y_c_1;
    float w_edge_1 = metal::smoothstep(0.04, 0.25, edge_contrast);
    float w_highlight = metal::smoothstep(0.04, 0.25, y_max_neighbor);
    float w_fringe = ((w_hue * w_edge_1) * w_highlight) * (defringeAmount / 100.0);
    if (w_fringe <= 0.001) {
        return color_1;
    }
    float r_adj = color_1.x - (w_fringe * excess_r);
    float b_adj = color_1.z - (w_fringe * excess_b);
    return metal::float3(r_adj, color_1.y, b_adj);
}

float proceduralGrainNoise(
    float u,
    float v
) {
    metal::float2 p_4 = 50.0 * metal::fract((metal::float2(u, v) * 0.3183099) + metal::float2(0.71, 0.113));
    return -1.0 + (2.0 * metal::fract((p_4.x * p_4.y) * (p_4.x + p_4.y)));
}

float clusteredGrainNoise(
    metal::float2 p
) {
    metal::float2 i_7 = metal::floor(p);
    metal::float2 f = metal::fract(p);
    metal::float2 u_2 = (f * f) * (metal::float2(3.0) - (2.0 * f));
    float _e12 = proceduralGrainNoise(i_7.x, i_7.y);
    float _e17 = proceduralGrainNoise(i_7.x + 1.0, i_7.y);
    float _e22 = proceduralGrainNoise(i_7.x, i_7.y + 1.0);
    float _e29 = proceduralGrainNoise(i_7.x + 1.0, i_7.y + 1.0);
    return metal::mix(metal::mix(_e12, _e17, u_2.x), metal::mix(_e22, _e29, u_2.x), u_2.y);
}

float evalGrainProcNoise(
    metal::float2 pos,
    float s,
    bool is_instant
) {
    if (is_instant) {
        float _e5 = clusteredGrainNoise(pos / metal::float2(s));
        float _e16 = clusteredGrainNoise((pos / metal::float2(s * 1.8)) + metal::float2(17.3, 31.7));
        return (0.6 * _e5) + (0.4 * _e16);
    } else {
        if (s > 1.0) {
            float t_cluster = metal::clamp((s - 1.0) / 2.0, 0.0, 1.0);
            float _e31 = clusteredGrainNoise(pos / metal::float2(s));
            float _e34 = proceduralGrainNoise(pos.x, pos.y);
            float clumped = (_e34 * (0.6 + (0.4 * _e31))) + (0.25 * _e31);
            return metal::mix(_e34, clumped, t_cluster);
        } else {
            float _e46 = proceduralGrainNoise(pos.x, pos.y);
            return _e46;
        }
    }
}

float evalGrainEta(
    float proc_val,
    float sensor_seed,
    int grain_mode,
    bool is_instant_1
) {
    if (grain_mode == 1) {
        if (is_instant_1) {
            return metal::clamp((0.55 * proc_val) + (0.2 * sensor_seed), -1.0, 1.0);
        } else {
            return metal::clamp((proc_val * (0.45 + (0.55 * metal::abs(sensor_seed)))) + (0.45 * sensor_seed), -1.0, 1.0);
        }
    } else {
        if (grain_mode == 2) {
            if (is_instant_1) {
                return metal::clamp((0.85 * proc_val) + (0.05 * sensor_seed), -1.0, 1.0);
            } else {
                return metal::clamp((0.9 * proc_val) + (0.1 * sensor_seed), -1.0, 1.0);
            }
        }
    }
    return 0.0;
}

metal::float3 applyStochasticEmulsionGrain(
    metal::float3 color_2,
    float y_c_2,
    float y_l_2,
    float y_r_2,
    float y_t_2,
    float y_b_2,
    float y_tl_2,
    float y_tr_2,
    float y_bl_2,
    float y_br_2,
    metal::float2 p_1,
    int grain_mode_1,
    float cluster_size,
    float grain_amount,
    int archetype
) {
    bool local_70 = {};
    bool local_71 = {};
    if (grain_mode_1 == 0) {
        return color_2;
    }
    float amount_1 = metal::clamp(grain_amount, 0.0, 2.0);
    if (amount_1 < 0.001) {
        return color_2;
    }
    float y_lowpass = (0.16666667 * (((y_l_2 + y_r_2) + y_t_2) + y_b_2)) + (0.083333336 * (((y_tl_2 + y_tr_2) + y_bl_2) + y_br_2));
    float delta_sensor = y_c_2 - y_lowpass;
    float gx_1 = (((y_tr_2 + (2.0 * y_r_2)) + y_br_2) - ((y_tl_2 + (2.0 * y_l_2)) + y_bl_2)) * 0.125;
    float gy_1 = (((y_bl_2 + (2.0 * y_b_2)) + y_br_2) - ((y_tl_2 + (2.0 * y_t_2)) + y_tr_2)) * 0.125;
    float grad_mag_1 = metal::sqrt((gx_1 * gx_1) + (gy_1 * gy_1));
    float edge_suppress = 1.0 / (1.0 + (35.0 * grad_mag_1));
    float delta_noise = delta_sensor * edge_suppress;
    float denom_2 = metal::max(metal::sqrt(metal::max(y_c_2, 0.0)), 0.05);
    float sensor_seed_1 = metal::clamp((delta_noise / denom_2) * 20.0, -1.0, 1.0);
    bool is_instant_2 = archetype == 7;
    float s_3 = metal::max(cluster_size, 0.25);
    float _e82 = evalGrainProcNoise(p_1, s_3, is_instant_2);
    float _e83 = evalGrainEta(_e82, sensor_seed_1, grain_mode_1, is_instant_2);
    float y_clamped = metal::clamp(y_c_2, 0.0, 1.0);
    float nutting = (4.0 * y_clamped) * (1.0 - y_clamped);
    float y_pixel = metal::dot(color_2, metal::float3(0.2627, 0.678, 0.0593));
    if (!((archetype == 6))) {
        local_70 = archetype == 10;
    } else {
        local_70 = true;
    }
    bool _e105 = local_70;
    if (!(_e105)) {
        local_71 = archetype == 11;
    } else {
        local_71 = true;
    }
    bool is_monochrome = local_71;
    if (is_monochrome) {
        float delta_2 = ((_e83 * nutting) * 0.28) * amount_1;
        float y_res = metal::max(y_pixel * (1.0 + delta_2), 0.0);
        return metal::float3(y_res);
    } else {
        float _e127 = evalGrainProcNoise(p_1 + metal::float2(17.3, 31.7), s_3, is_instant_2);
        float _e132 = evalGrainProcNoise(p_1 + metal::float2(73.1, 19.9), s_3, is_instant_2);
        float _e137 = evalGrainProcNoise(p_1 + metal::float2(43.7, 89.3), s_3, is_instant_2);
        float _e138 = evalGrainEta(_e127, sensor_seed_1, grain_mode_1, is_instant_2);
        float _e139 = evalGrainEta(_e132, sensor_seed_1, grain_mode_1, is_instant_2);
        float _e140 = evalGrainEta(_e137, sensor_seed_1, grain_mode_1, is_instant_2);
        float eta_final_r = metal::mix(_e83, _e138, 0.25) * 1.264911;
        float eta_final_g = metal::mix(_e83, _e139, 0.25) * 1.264911;
        float eta_final_b = metal::mix(_e83, _e140, 0.25) * 1.264911;
        float scale_1 = (nutting * 0.28) * amount_1;
        float delta_r = eta_final_r * scale_1;
        float delta_g = eta_final_g * scale_1;
        float delta_b = eta_final_b * scale_1;
        float chroma_damp = 1.0 - (0.1 * nutting);
        float r_c = y_pixel + (chroma_damp * (color_2.x - y_pixel));
        float g_c = y_pixel + (chroma_damp * (color_2.y - y_pixel));
        float b_c = y_pixel + (chroma_damp * (color_2.z - y_pixel));
        float r_res = metal::max(r_c * (1.0 + delta_r), 0.0);
        float g_res = metal::max(g_c * (1.0 + delta_g), 0.0);
        float b_res = metal::max(b_c * (1.0 + delta_b), 0.0);
        return metal::float3(r_res, g_res, b_res);
    }
}

float float_mod(
    float x,
    float y
) {
    return x - (y * metal::floor(x / y));
}

float srgbGammaToLinear(
    float c_2
) {
    float cl = metal::clamp(c_2, 0.0, 1.0);
    if (cl <= 0.04045) {
        return cl / 12.92;
    } else {
        return metal::pow((cl + 0.055) / 1.055, 2.4);
    }
}

metal::float3 srgbGammaToLinear3_(
    metal::float3 c_3
) {
    float _e2 = srgbGammaToLinear(c_3.x);
    float _e4 = srgbGammaToLinear(c_3.y);
    float _e6 = srgbGammaToLinear(c_3.z);
    return metal::float3(_e2, _e4, _e6);
}

float linearToSrgbGamma(
    float c_4
) {
    float cl_1 = metal::clamp(c_4, 0.0, 1.0);
    if (cl_1 <= 0.0031308) {
        return 12.92 * cl_1;
    } else {
        return (1.055 * metal::pow(cl_1, 0.41666666)) - 0.055;
    }
}

metal::float3 linearToSrgbGamma3_(
    metal::float3 c_5
) {
    float _e2 = linearToSrgbGamma(c_5.x);
    float _e4 = linearToSrgbGamma(c_5.y);
    float _e6 = linearToSrgbGamma(c_5.z);
    return metal::float3(_e2, _e4, _e6);
}

metal::float3 rgb2hsv(
    metal::float3 c_6
) {
    float h = 0.0;
    float s_1 = 0.0;
    float maxV = metal::max(c_6.x, metal::max(c_6.y, c_6.z));
    float minV = metal::min(c_6.x, metal::min(c_6.y, c_6.z));
    float delta_3 = maxV - minV;
    if (delta_3 > 0.00001) {
        if (maxV == c_6.x) {
            h = (c_6.y - c_6.z) / delta_3;
            float _e22 = h;
            if (_e22 < 0.0) {
                float _e25 = h;
                h = _e25 + 6.0;
            }
        } else {
            if (maxV == c_6.y) {
                h = ((c_6.z - c_6.x) / delta_3) + 2.0;
            } else {
                h = ((c_6.x - c_6.y) / delta_3) + 4.0;
            }
        }
        float _e42 = h;
        h = _e42 * 60.0;
    }
    if (maxV > 0.00001) {
        s_1 = delta_3 / maxV;
    }
    float _e50 = h;
    float _e51 = s_1;
    return metal::float3(_e50, _e51, maxV);
}

metal::float3 hsv2rgb(
    metal::float3 c_7
) {
    metal::float3 rgb = metal::float3(0.0);
    float h_1 = c_7.x / 60.0;
    float s_4 = c_7.y;
    float v_2 = c_7.z;
    float C = v_2 * s_4;
    float _e8 = float_mod(h_1, 2.0);
    float X = C * (1.0 - metal::abs(_e8 - 1.0));
    float m_2 = v_2 - C;
    if (h_1 < 1.0) {
        rgb = metal::float3(C, X, 0.0);
    } else {
        if (h_1 < 2.0) {
            rgb = metal::float3(X, C, 0.0);
        } else {
            if (h_1 < 3.0) {
                rgb = metal::float3(0.0, C, X);
            } else {
                if (h_1 < 4.0) {
                    rgb = metal::float3(0.0, X, C);
                } else {
                    if (h_1 < 5.0) {
                        rgb = metal::float3(X, 0.0, C);
                    } else {
                        rgb = metal::float3(C, 0.0, X);
                    }
                }
            }
        }
    }
    metal::float3 _e41 = rgb;
    return _e41 + metal::float3(m_2);
}

float evalCameraHermite7_(
    float x_1,
    type_7 knots,
    type_8 m
) {
    int k = 0;
    if (x_1 <= knots.inner[0].x) {
        return knots.inner[0].y;
    }
    if (x_1 >= knots.inner[6].x) {
        return knots.inner[6].y;
    }
    uint2 loop_bound_1 = uint2(4294967295u);
    bool loop_init_1 = true;
    while(true) {
        if (metal::all(loop_bound_1 == uint2(0u))) { break; }
        loop_bound_1 -= uint2(loop_bound_1.y == 0u, 1u);
        if (!loop_init_1) {
            int _e83 = k;
            k = as_type<int>(as_type<uint>(_e83) + as_type<uint>(1));
        }
        loop_init_1 = false;
        int _e15 = k;
        if (_e15 < 6) {
        } else {
            break;
        }
        {
            int _e18 = k;
            if (x_1 <= knots.inner[as_type<int>(as_type<uint>(_e18) + as_type<uint>(1))].x) {
                int _e24 = k;
                int _e29 = k;
                float dx = knots.inner[as_type<int>(as_type<uint>(_e24) + as_type<uint>(1))].x - knots.inner[_e29].x;
                int _e33 = k;
                float t = (x_1 - knots.inner[_e33].x) / dx;
                float t2_ = t * t;
                float t3_ = t2_ * t;
                float h00_ = ((2.0 * t3_) - (3.0 * t2_)) + 1.0;
                float h10_ = (t3_ - (2.0 * t2_)) + t;
                float h01_ = (-2.0 * t3_) + (3.0 * t2_);
                float h11_ = t3_ - t2_;
                int _e57 = k;
                int _e61 = k;
                int _e66 = k;
                int _e73 = k;
                return metal::clamp((((h00_ * knots.inner[_e57].y) + ((h10_ * m.inner[_e61]) * dx)) + (h01_ * knots.inner[as_type<int>(as_type<uint>(_e66) + as_type<uint>(1))].y)) + ((h11_ * m.inner[as_type<int>(as_type<uint>(_e73) + as_type<uint>(1))]) * dx), 0.0, 1.0);
            }
        }
    }
    return x_1;
}

float applyCameraToneCurve(
    float x_2,
    int mode
) {
    float xc = metal::clamp(x_2, 0.0, 1.0);
    if (mode == 2) {
        float _e9 = evalCameraHermite7_(xc, DCP_KNOTS_PORTRAIT, DCP_TANGENTS_PORTRAIT);
        return _e9;
    } else {
        if (mode == 3) {
            float _e14 = evalCameraHermite7_(xc, DCP_KNOTS_LANDSCAPE, DCP_TANGENTS_LANDSCAPE);
            return _e14;
        } else {
            if (mode == 4) {
                float _e19 = evalCameraHermite7_(xc, DCP_KNOTS_VIVID, DCP_TANGENTS_VIVID);
                return _e19;
            } else {
                float _e22 = evalCameraHermite7_(xc, DCP_KNOTS_STANDARD, DCP_TANGENTS_STANDARD);
                return _e22;
            }
        }
    }
}

int naga_f2i32(float value) {
    return static_cast<int>(metal::clamp(value, -2147483600.0, 2147483500.0));
}

int naga_mod(int lhs, int rhs) {
    int divisor = metal::select(rhs, 1, (lhs == (-2147483647 - 1) & rhs == -1) | (rhs == 0));
    return lhs - (lhs / divisor) * divisor;
}

metal::float3 evalHueSatMapDelta(
    float h_deg,
    float s_2,
    type_9 table
) {
    float h_norm = {};
    h_norm = metal::fract(h_deg / 360.0);
    float _e7 = h_norm;
    if (_e7 < 0.0) {
        float _e10 = h_norm;
        h_norm = _e10 + 1.0;
    }
    float _e13 = h_norm;
    float h_coord = _e13 * 12.0;
    int h0_ = naga_mod(naga_f2i32(metal::floor(h_coord)), 12);
    int h1_ = naga_mod(as_type<int>(as_type<uint>(h0_) + as_type<uint>(1)), 12);
    float h_frac = h_coord - metal::floor(h_coord);
    float s_clamped = metal::clamp(s_2, 0.0, 1.0);
    metal::float3 c00_ = table.inner[as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(h0_) * as_type<uint>(2))) + as_type<uint>(0))];
    metal::float3 c01_ = table.inner[as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(h0_) * as_type<uint>(2))) + as_type<uint>(1))];
    metal::float3 c10_ = table.inner[as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(h1_) * as_type<uint>(2))) + as_type<uint>(0))];
    metal::float3 c11_ = table.inner[as_type<int>(as_type<uint>(as_type<int>(as_type<uint>(h1_) * as_type<uint>(2))) + as_type<uint>(1))];
    metal::float3 c0_ = metal::mix(c00_, c01_, s_clamped);
    metal::float3 c1_ = metal::mix(c10_, c11_, s_clamped);
    return metal::mix(c0_, c1_, h_frac);
}

metal::float3 applyCameraHueSatMap(
    metal::float3 c_srgb,
    int mode_1
) {
    metal::float3 delta = metal::float3(0.0);
    if (metal::max(metal::max(c_srgb.x, c_srgb.y), c_srgb.z) <= 0.000001) {
        return c_srgb;
    }
    metal::float3 _e9 = rgb2hsv(c_srgb);
    if (_e9.y <= 0.000001) {
        return c_srgb;
    }
    if (mode_1 == 2) {
        metal::float3 _e21 = evalHueSatMapDelta(_e9.x, _e9.y, DCP_HSM_PORTRAIT);
        delta = _e21;
    } else {
        if (mode_1 == 3) {
            metal::float3 _e27 = evalHueSatMapDelta(_e9.x, _e9.y, DCP_HSM_LANDSCAPE);
            delta = _e27;
        } else {
            if (mode_1 == 4) {
                metal::float3 _e33 = evalHueSatMapDelta(_e9.x, _e9.y, DCP_HSM_VIVID);
                delta = _e33;
            } else {
                metal::float3 _e37 = evalHueSatMapDelta(_e9.x, _e9.y, DCP_HSM_STANDARD);
                delta = _e37;
            }
        }
    }
    float _e40 = delta.x;
    float _e43 = float_mod(_e9.x + _e40, 360.0);
    float _e47 = float_mod(_e43 + 360.0, 360.0);
    float _e50 = delta.y;
    float s_out = metal::clamp(_e9.y * _e50, 0.0, 1.0);
    float _e57 = delta.z;
    float v_out = metal::max(_e9.z * _e57, 0.0);
    metal::float3 _e62 = hsv2rgb(metal::float3(_e47, s_out, v_out));
    return _e62;
}

float agxApplyCurveChannel(
    float x_3
) {
    float xc_1 = metal::clamp(x_3, 0.0, 1.0);
    float curved = ((((((((((15.45672 * xc_1) - 40.02792) * xc_1) + 31.87076) * xc_1) - 6.84882) * xc_1) + 0.4286) * xc_1) + 0.12066) * xc_1;
    return metal::clamp(curved, 0.0, 1.0);
}

metal::float3 agxToneMap(
    metal::float3 rec2020_color,
    float contrast_gain
) {
    metal::float3 cComp = {};
    metal::float3 norm = {};
    float minC = metal::min(rec2020_color.x, metal::min(rec2020_color.y, rec2020_color.z));
    cComp = rec2020_color;
    if (minC < 0.0) {
        cComp = rec2020_color - metal::float3(minC);
    }
    metal::float3 _e13 = cComp;
    metal::float3 inset = AGX_INSET_MATRIX * _e13;
    metal::float3 xRel = metal::max(inset / metal::float3(0.18), metal::float3(0.000001));
    metal::float3 logEncoded = (metal::log2(xRel) - metal::float3(-10.0)) / metal::float3(16.5);
    norm = metal::clamp(logEncoded, metal::float3(0.0), metal::float3(1.0));
    if (metal::abs(contrast_gain - 1.0) > 0.001) {
        metal::float3 _e41 = norm;
        norm = metal::clamp(metal::float3(0.6060606) + ((_e41 - metal::float3(0.6060606)) * contrast_gain), metal::float3(0.0), metal::float3(1.0));
    }
    float _e53 = norm.x;
    float _e54 = agxApplyCurveChannel(_e53);
    float _e56 = norm.y;
    float _e57 = agxApplyCurveChannel(_e56);
    float _e59 = norm.z;
    float _e60 = agxApplyCurveChannel(_e59);
    metal::float3 curved_1 = metal::float3(_e54, _e57, _e60);
    metal::float3 linearized = metal::pow(metal::max(curved_1, metal::float3(0.0)), metal::float3(2.2));
    return metal::max(AGX_OUTSET_MATRIX * linearized, metal::float3(0.0));
}

metal::float3 agxToneMapVivid(
    metal::float3 rec2020_color_1,
    float contrast_gain_1
) {
    float punchyContrast = contrast_gain_1 * 1.35;
    metal::float3 _e4 = agxToneMap(rec2020_color_1, punchyContrast);
    float lum = ((0.2126 * _e4.x) + (0.7152 * _e4.y)) + (0.0722 * _e4.z);
    float t_1 = metal::clamp((lum - 0.65) / 0.35, 0.0, 1.0);
    float h_2 = (t_1 * t_1) * (3.0 - (2.0 * t_1));
    float sat = 1.25 - (0.25 * h_2);
    return metal::clamp(metal::float3(lum + (sat * (_e4.x - lum)), lum + (sat * (_e4.y - lum)), lum + (sat * (_e4.z - lum))), metal::float3(0.0), metal::float3(1.0));
}

metal::float3 agxToneMapPhotographic(
    metal::float3 rec2020_color_2,
    float contrast_gain_2
) {
    metal::float3 compressed = {};
    float norm_1 = {};
    float gain = 1.0;
    float minC_1 = metal::min(rec2020_color_2.x, metal::min(rec2020_color_2.y, rec2020_color_2.z));
    compressed = rec2020_color_2;
    if (minC_1 < 0.0) {
        compressed = rec2020_color_2 - metal::float3(minC_1);
    }
    float photoContrast = contrast_gain_2 * 1.28;
    float _e15 = compressed.x;
    float _e19 = compressed.y;
    float _e24 = compressed.z;
    float lumRec = ((0.2627 * _e15) + (0.678 * _e19)) + (0.0593 * _e24);
    float xRel_1 = metal::max(lumRec / 0.18, 0.000001);
    float logEncoded_1 = (metal::log2(xRel_1) - AGX_MIN_EV) / AGX_EV_RANGE;
    norm_1 = metal::clamp(logEncoded_1, 0.0, 1.0);
    if (metal::abs(photoContrast - 1.0) > 0.001) {
        float _e47 = norm_1;
        norm_1 = metal::clamp(AGX_MID_GRAY_NORM + ((_e47 - AGX_MID_GRAY_NORM) * photoContrast), 0.0, 1.0);
    }
    float _e55 = norm_1;
    float _e56 = agxApplyCurveChannel(_e55);
    float lumMapped = metal::pow(metal::max(_e56, 0.0), AGX_GAMMA);
    if (lumRec > 0.000001) {
        gain = lumMapped / lumRec;
    }
    metal::float3 _e66 = compressed;
    float _e67 = gain;
    metal::float3 cScaled = _e66 * _e67;
    metal::float3 sRatio = REC2020_TO_LINEAR_SRGB * cScaled;
    metal::float3 _e71 = compressed;
    metal::float3 _e72 = agxToneMap(_e71, photoContrast);
    float peakVal = metal::max(lumMapped, metal::max(metal::max(sRatio.x, 0.0), metal::max(metal::max(sRatio.y, 0.0), metal::max(sRatio.z, 0.0))));
    float t_2 = metal::clamp((peakVal - 0.85) / 0.2, 0.0, 1.0);
    float h_3 = (t_2 * t_2) * (3.0 - (2.0 * t_2));
    return metal::clamp(metal::float3((metal::max(sRatio.x, 0.0) * (1.0 - h_3)) + (_e72.x * h_3), (metal::max(sRatio.y, 0.0) * (1.0 - h_3)) + (_e72.y * h_3), (metal::max(sRatio.z, 0.0) * (1.0 - h_3)) + (_e72.z * h_3)), metal::float3(0.0), metal::float3(1.0));
}

metal::float3 applyBilateralToneRecovery(
    metal::float3 color_3,
    float y_c_3,
    type_11 y_neighbors_1,
    float highlights,
    float shadows,
    float whites,
    float blacks
) {
    bool local_72 = {};
    bool local_73 = {};
    bool local_74 = {};
    float total_weight = 1.0;
    float weighted_ev_sum = {};
    int i_3 = 0;
    int i_4 = 4;
    int i_5 = 8;
    int i_6 = 12;
    float delta_ev = 0.0;
    if (highlights == 0.0) {
        local_72 = shadows == 0.0;
    } else {
        local_72 = false;
    }
    bool _e14 = local_72;
    if (_e14) {
        local_73 = whites == 0.0;
    } else {
        local_73 = false;
    }
    bool _e20 = local_73;
    if (_e20) {
        local_74 = blacks == 0.0;
    } else {
        local_74 = false;
    }
    bool _e26 = local_74;
    if (_e26) {
        return color_3;
    }
    if (y_c_3 <= 0.000001) {
        return color_3;
    }
    float ev_c = metal::log2(metal::max(y_c_3, 0.00001) / 0.18);
    weighted_ev_sum = ev_c;
    uint2 loop_bound_2 = uint2(4294967295u);
    bool loop_init_2 = true;
    while(true) {
        if (metal::all(loop_bound_2 == uint2(0u))) { break; }
        loop_bound_2 -= uint2(loop_bound_2.y == 0u, 1u);
        if (!loop_init_2) {
            int _e63 = i_3;
            i_3 = as_type<int>(as_type<uint>(_e63) + as_type<uint>(1));
        }
        loop_init_2 = false;
        int _e42 = i_3;
        if (_e42 < 4) {
        } else {
            break;
        }
        {
            int _e45 = i_3;
            float ev_i = metal::log2(metal::max(y_neighbors_1.inner[_e45], 0.00001) / 0.18);
            float d_ev = ev_i - ev_c;
            float w_r = metal::exp(-((d_ev * d_ev)) * 0.125);
            float w = 0.60653067 * w_r;
            float _e58 = total_weight;
            total_weight = _e58 + w;
            float _e60 = weighted_ev_sum;
            weighted_ev_sum = _e60 + (w * ev_i);
        }
    }
    uint2 loop_bound_3 = uint2(4294967295u);
    bool loop_init_3 = true;
    while(true) {
        if (metal::all(loop_bound_3 == uint2(0u))) { break; }
        loop_bound_3 -= uint2(loop_bound_3.y == 0u, 1u);
        if (!loop_init_3) {
            int _e89 = i_4;
            i_4 = as_type<int>(as_type<uint>(_e89) + as_type<uint>(1));
        }
        loop_init_3 = false;
        int _e68 = i_4;
        if (_e68 < 8) {
        } else {
            break;
        }
        {
            int _e71 = i_4;
            float ev_i_1 = metal::log2(metal::max(y_neighbors_1.inner[_e71], 0.00001) / 0.18);
            float d_ev_1 = ev_i_1 - ev_c;
            float w_r_1 = metal::exp(-((d_ev_1 * d_ev_1)) * 0.125);
            float w_1 = 0.36787945 * w_r_1;
            float _e84 = total_weight;
            total_weight = _e84 + w_1;
            float _e86 = weighted_ev_sum;
            weighted_ev_sum = _e86 + (w_1 * ev_i_1);
        }
    }
    uint2 loop_bound_4 = uint2(4294967295u);
    bool loop_init_4 = true;
    while(true) {
        if (metal::all(loop_bound_4 == uint2(0u))) { break; }
        loop_bound_4 -= uint2(loop_bound_4.y == 0u, 1u);
        if (!loop_init_4) {
            int _e115 = i_5;
            i_5 = as_type<int>(as_type<uint>(_e115) + as_type<uint>(1));
        }
        loop_init_4 = false;
        int _e94 = i_5;
        if (_e94 < 12) {
        } else {
            break;
        }
        {
            int _e97 = i_5;
            float ev_i_2 = metal::log2(metal::max(y_neighbors_1.inner[_e97], 0.00001) / 0.18);
            float d_ev_2 = ev_i_2 - ev_c;
            float w_r_2 = metal::exp(-((d_ev_2 * d_ev_2)) * 0.125);
            float w_2 = 0.60653067 * w_r_2;
            float _e110 = total_weight;
            total_weight = _e110 + w_2;
            float _e112 = weighted_ev_sum;
            weighted_ev_sum = _e112 + (w_2 * ev_i_2);
        }
    }
    uint2 loop_bound_5 = uint2(4294967295u);
    bool loop_init_5 = true;
    while(true) {
        if (metal::all(loop_bound_5 == uint2(0u))) { break; }
        loop_bound_5 -= uint2(loop_bound_5.y == 0u, 1u);
        if (!loop_init_5) {
            int _e141 = i_6;
            i_6 = as_type<int>(as_type<uint>(_e141) + as_type<uint>(1));
        }
        loop_init_5 = false;
        int _e120 = i_6;
        if (_e120 < 16) {
        } else {
            break;
        }
        {
            int _e123 = i_6;
            float ev_i_3 = metal::log2(metal::max(y_neighbors_1.inner[_e123], 0.00001) / 0.18);
            float d_ev_3 = ev_i_3 - ev_c;
            float w_r_3 = metal::exp(-((d_ev_3 * d_ev_3)) * 0.125);
            float w_3 = 0.36787945 * w_r_3;
            float _e136 = total_weight;
            total_weight = _e136 + w_3;
            float _e138 = weighted_ev_sum;
            weighted_ev_sum = _e138 + (w_3 * ev_i_3);
        }
    }
    float _e144 = weighted_ev_sum;
    float _e145 = total_weight;
    float ev_base = _e144 / _e145;
    if (blacks != 0.0) {
        float u_3 = metal::clamp((-(ev_base) - 1.0) / 3.0, 0.0, 1.0);
        float w_blacks = (u_3 * u_3) * (3.0 - (2.0 * u_3));
        float _e165 = delta_ev;
        delta_ev = _e165 + (((blacks / 100.0) * 1.2) * w_blacks);
    }
    if (shadows != 0.0) {
        float u_4 = metal::clamp(-(ev_base) / 3.0, 0.0, 1.0);
        float w_shadows = (u_4 * u_4) * (3.0 - (2.0 * u_4));
        float _e186 = delta_ev;
        delta_ev = _e186 + (((shadows / 100.0) * 2.25) * w_shadows);
    }
    if (highlights != 0.0) {
        float u_5 = metal::clamp(ev_base / 3.0, 0.0, 1.0);
        float w_highlights = (u_5 * u_5) * (3.0 - (2.0 * u_5));
        float _e206 = delta_ev;
        delta_ev = _e206 + (((highlights / 100.0) * 2.25) * w_highlights);
    }
    if (whites != 0.0) {
        float u_6 = metal::clamp((ev_base - 1.0) / 3.0, 0.0, 1.0);
        float w_whites = (u_6 * u_6) * (3.0 - (2.0 * u_6));
        float _e228 = delta_ev;
        delta_ev = _e228 + (((whites / 100.0) * 1.2) * w_whites);
    }
    float _e235 = delta_ev;
    if (_e235 == 0.0) {
        return color_3;
    }
    float detail = metal::clamp(ev_c - ev_base, -3.0, 3.0);
    float _e244 = delta_ev;
    float detail_boost = (0.35 * detail) * (metal::abs(_e244) / 2.25);
    float _e249 = delta_ev;
    float gain_1 = metal::exp2(metal::clamp(_e249 + detail_boost, -6.0, 6.0));
    return color_3 * gain_1;
}

metal::float3 applyColorAdjustments(
    metal::float3 rgb_1,
    float vibrance,
    float saturation,
    float deepBlue,
    float foliage
) {
    bool local_75 = {};
    bool local_76 = {};
    bool local_77 = {};
    metal::float3 c_8 = {};
    float skinProtect = 1.0;
    float vibGain = {};
    if (vibrance == 0.0) {
        local_75 = saturation == 0.0;
    } else {
        local_75 = false;
    }
    bool _e12 = local_75;
    if (_e12) {
        local_76 = deepBlue == 0.0;
    } else {
        local_76 = false;
    }
    bool _e18 = local_76;
    if (_e18) {
        local_77 = foliage == 0.0;
    } else {
        local_77 = false;
    }
    bool _e24 = local_77;
    if (_e24) {
        return rgb_1;
    }
    c_8 = metal::max(rgb_1, metal::float3(0.0));
    float _e30 = c_8.x;
    float _e34 = c_8.y;
    float _e39 = c_8.z;
    float lum_1 = ((0.2627 * _e30) + (0.678 * _e34)) + (0.0593 * _e39);
    if (saturation != 0.0) {
        float satGain = metal::max(1.0 + (saturation / 100.0), 0.0);
        metal::float3 _e52 = c_8;
        c_8 = metal::float3(lum_1) + ((_e52 - metal::float3(lum_1)) * satGain);
    }
    if (vibrance != 0.0) {
        float _e60 = c_8.x;
        float _e62 = c_8.y;
        float _e65 = c_8.z;
        float maxC = metal::max(metal::max(_e60, _e62), _e65);
        float _e68 = c_8.x;
        float _e70 = c_8.y;
        float _e73 = c_8.z;
        float minC_2 = metal::min(metal::min(_e68, _e70), _e73);
        float chroma = maxC - minC_2;
        float _e79 = c_8.x;
        float _e81 = c_8.y;
        float _e84 = c_8.z;
        float sum = (_e79 + _e81) + _e84;
        if (sum > 0.00001) {
            float _e89 = c_8.x;
            float rN = _e89 / sum;
            float _e92 = c_8.y;
            float gN = _e92 / sum;
            float bN = (1.0 - rN) - gN;
            float deltaCross = (gN + (0.35 * rN)) - 0.478;
            float dCross = deltaCross / 0.04;
            float dMel = (rN - 0.48) / 0.16;
            float d2_ = (dCross * dCross) + (dMel * dMel);
            float gauss = metal::exp(-0.5 * d2_);
            float uLum = metal::clamp((sum - 0.005) / 0.025, 0.0, 1.0);
            float wLum = (uLum * uLum) * (3.0 - (2.0 * uLum));
            float uRg = metal::clamp((rN - gN) / 0.05, 0.0, 1.0);
            float wRg = (uRg * uRg) * (3.0 - (2.0 * uRg));
            float uRb = metal::clamp((rN - bN) / 0.08, 0.0, 1.0);
            float wRb = (uRb * uRb) * (3.0 - (2.0 * uRb));
            float uGb = metal::clamp(((gN - bN) + 0.05) / 0.1, 0.0, 1.0);
            float wGb = (uGb * uGb) * (3.0 - (2.0 * uGb));
            float skinMetric = (((gauss * wLum) * wRg) * wRb) * wGb;
            skinProtect = 1.0 - (0.85 * skinMetric);
        }
        vibGain = (vibrance / 100.0) * 0.6;
        if (vibrance > 0.0) {
            float normChroma = metal::clamp(chroma / metal::max(lum_1, 0.0001), 0.0, 1.0);
            float _e193 = skinProtect;
            vibGain = (((vibrance / 100.0) * (1.0 - normChroma)) * 0.8) * _e193;
        }
        metal::float3 _e195 = c_8;
        metal::float3 _e196 = c_8;
        float _e199 = vibGain;
        c_8 = _e195 + ((_e196 - metal::float3(lum_1)) * _e199);
    }
    if (deepBlue != 0.0) {
        float _e205 = c_8.x;
        float _e207 = c_8.y;
        float maxWarm = metal::max(_e205, _e207);
        float _e210 = c_8.z;
        float skySignal = metal::max(_e210 - maxWarm, 0.0);
        if (skySignal > 0.0) {
            float x_5 = deepBlue / 100.0;
            float factor = metal::sign(x_5) * metal::pow(metal::abs(x_5), 1.6);
            float _e225 = c_8.x;
            c_8.x = _e225 - ((factor * skySignal) * 0.28);
            float _e232 = c_8.y;
            c_8.y = _e232 - ((factor * skySignal) * 0.18);
            float _e239 = c_8.z;
            c_8.z = _e239 - ((factor * skySignal) * 0.15);
        }
    }
    if (foliage != 0.0) {
        float _e247 = c_8.x;
        float _e249 = c_8.z;
        float maxRb = metal::max(_e247, _e249);
        float _e252 = c_8.y;
        float greenSignal = metal::max(_e252 - maxRb, 0.0);
        if (greenSignal > 0.0) {
            float x_6 = foliage / 100.0;
            float factor_1 = metal::sign(x_6) * metal::pow(metal::abs(x_6), 1.6);
            float _e267 = c_8.x;
            c_8.x = _e267 - ((factor_1 * greenSignal) * 0.3);
            float _e274 = c_8.z;
            c_8.z = _e274 - ((factor_1 * greenSignal) * 0.15);
            float _e281 = c_8.y;
            c_8.y = _e281 + ((factor_1 * greenSignal) * 0.1);
        }
    }
    metal::float3 _e286 = c_8;
    return metal::max(_e286, metal::float3(0.0));
}

metal::float3 applySplitToning(
    metal::float3 rgb_2,
    float shadowHue,
    float shadowSat,
    float highlightHue,
    float highlightSat,
    float balance
) {
    bool local_78 = {};
    metal::float3 delta_1 = metal::float3(0.0);
    bool local_79 = {};
    bool local_80 = {};
    if (shadowSat <= 0.0) {
        local_78 = highlightSat <= 0.0;
    } else {
        local_78 = false;
    }
    bool _e13 = local_78;
    if (_e13) {
        return rgb_2;
    }
    float lum_2 = ((0.2627 * rgb_2.x) + (0.678 * rgb_2.y)) + (0.0593 * rgb_2.z);
    float yClamp = metal::clamp(lum_2, 0.0, 1.0);
    float t_3 = metal::clamp(0.5 - ((balance / 100.0) * 0.25), 0.1, 0.9);
    if (shadowSat > 0.0) {
        local_79 = yClamp < t_3;
    } else {
        local_79 = false;
    }
    bool _e46 = local_79;
    if (_e46) {
        float u_s = (t_3 - yClamp) / t_3;
        float w_s = ((u_s * u_s) * (3.0 - (2.0 * u_s))) * (shadowSat / 100.0);
        metal::float3 _e61 = hsv2rgb(metal::float3(shadowHue, 1.0, 1.0));
        float tLum = ((0.2627 * _e61.x) + (0.678 * _e61.y)) + (0.0593 * _e61.z);
        metal::float3 _e73 = delta_1;
        delta_1 = _e73 + (metal::float3(w_s) * (_e61 - metal::float3(tLum)));
    }
    if (highlightSat > 0.0) {
        local_80 = yClamp > t_3;
    } else {
        local_80 = false;
    }
    bool _e85 = local_80;
    if (_e85) {
        float u_h = (yClamp - t_3) / (1.0 - t_3);
        float w_h = ((u_h * u_h) * (3.0 - (2.0 * u_h))) * (highlightSat / 100.0);
        metal::float3 _e102 = hsv2rgb(metal::float3(highlightHue, 1.0, 1.0));
        float tLum_1 = ((0.2627 * _e102.x) + (0.678 * _e102.y)) + (0.0593 * _e102.z);
        metal::float3 _e114 = delta_1;
        delta_1 = _e114 + (metal::float3(w_h) * (_e102 - metal::float3(tLum_1)));
    }
    metal::float3 _e120 = delta_1;
    return metal::clamp(rgb_2 + _e120, metal::float3(0.0), metal::float3(1.0));
}

float applyPaperGrade(
    float val,
    float grade
) {
    float scale = 0.4;
    if (metal::abs(grade) < 0.001) {
        return val;
    }
    if (grade >= 0.0) {
        scale = 0.6;
    }
    float _e12 = scale;
    float alpha_1 = (grade / 2.0) * _e12;
    return metal::clamp(val + (((alpha_1 * val) * (1.0 - val)) * ((2.0 * val) - 1.0)), 0.0, 1.0);
}

metal::float3 applyBaseWarmth(
    metal::float3 rgb_3,
    float warmth
) {
    metal::float3 mult = metal::float3(1.0);
    if (metal::abs(warmth) < 0.001) {
        return rgb_3;
    }
    float lum_3 = ((0.2627 * rgb_3.x) + (0.678 * rgb_3.y)) + (0.0593 * rgb_3.z);
    float highlightWeight = metal::clamp(metal::pow(lum_3, 0.6), 0.0, 1.0);
    float factor_2 = (warmth / 100.0) * highlightWeight;
    if (factor_2 > 0.0) {
        mult = metal::float3(1.0, 1.0 - (factor_2 * 0.013849), 1.0 - (factor_2 * 0.051111));
    } else {
        float absF = -(factor_2);
        mult = metal::float3(1.0 - (absF * 0.033699), 1.0 - (absF * 0.031749), 1.0);
    }
    metal::float3 _e50 = mult;
    return metal::clamp(rgb_3 * _e50, metal::float3(0.0), metal::float3(1.0));
}

metal::float3 getEnlargerFilterWeights(
    int filterMode
) {
    if (filterMode == 1) {
        return metal::float3(0.35, 0.6, 0.05);
    }
    if (filterMode == 2) {
        return metal::float3(0.55, 0.42, 0.03);
    }
    if (filterMode == 3) {
        return metal::float3(0.85, 0.15, 0.0);
    }
    if (filterMode == 4) {
        return metal::float3(0.15, 0.8, 0.05);
    }
    if (filterMode == 5) {
        return metal::float3(0.0, 0.6, 0.4);
    }
    return metal::float3(0.2627, 0.678, 0.0593);
}

metal::float3 applyPrintMedia(
    metal::float3 rgb_4,
    int mediaMode,
    int filterMode_1,
    float paperGrade,
    float processDepth
) {
    float rOut = {};
    float gOut = {};
    float bOut = {};
    if (mediaMode == 0) {
        return rgb_4;
    }
    float depth = metal::clamp(processDepth / 100.0, 0.0, 1.0);
    if (mediaMode == 1) {
        float _e15 = applyPaperGrade(rgb_4.x, paperGrade);
        float _e17 = applyPaperGrade(rgb_4.y, paperGrade);
        float _e19 = applyPaperGrade(rgb_4.z, paperGrade);
        float uR = metal::clamp((_e15 - 0.7) / 0.3, 0.0, 1.0);
        float rP = _e15 - ((((0.045 * depth) * uR) * uR) * (3.0 - (2.0 * uR)));
        float uG = metal::clamp((_e17 - 0.7) / 0.3, 0.0, 1.0);
        float gP = _e17 - ((((0.045 * depth) * uG) * uG) * (3.0 - (2.0 * uG)));
        float uB = metal::clamp((_e19 - 0.7) / 0.3, 0.0, 1.0);
        float bP = _e19 - ((((0.045 * depth) * uB) * uB) * (3.0 - (2.0 * uB)));
        float lum_4 = ((0.2627 * rP) + (0.678 * gP)) + (0.0593 * bP);
        float satScale = 0.7 + (0.35 * depth);
        rOut = metal::clamp(lum_4 + ((rP - lum_4) * satScale), 0.0, 1.0);
        gOut = metal::clamp(lum_4 + ((gP - lum_4) * satScale), 0.0, 1.0);
        bOut = metal::clamp(lum_4 + ((bP - lum_4) * satScale), 0.0, 1.0);
        float dyeDensity = (0.04 * metal::max(1.0 - lum_4, 0.0)) * depth;
        float _e111 = rOut;
        rOut = metal::clamp(_e111 * (1.0 - (dyeDensity * 0.4)), 0.0, 1.0);
        float _e120 = gOut;
        gOut = metal::clamp(_e120 * (1.0 - (dyeDensity * 0.6)), 0.0, 1.0);
        float _e129 = bOut;
        bOut = metal::clamp(_e129 * (1.0 - (dyeDensity * 0.9)), 0.0, 1.0);
        float _e138 = rOut;
        float _e139 = gOut;
        float _e140 = bOut;
        return metal::float3(_e138, _e139, _e140);
    } else {
        if (mediaMode == 2) {
            float _e145 = applyPaperGrade(rgb_4.x, paperGrade);
            float _e147 = applyPaperGrade(rgb_4.y, paperGrade);
            float _e149 = applyPaperGrade(rgb_4.z, paperGrade);
            float shadowPow = 1.0 + (0.12 * depth);
            float rP_1 = metal::pow(_e145, shadowPow);
            float gP_1 = metal::pow(_e147, shadowPow);
            float bP_1 = metal::pow(_e149, shadowPow);
            float lum_5 = ((0.2627 * rP_1) + (0.678 * gP_1)) + (0.0593 * bP_1);
            float azoSat = 0.8 + (0.3 * depth);
            float rOut_1 = metal::clamp(lum_5 + ((rP_1 - lum_5) * azoSat), 0.0, 1.0);
            float gOut_1 = metal::clamp(lum_5 + ((gP_1 - lum_5) * azoSat), 0.0, 1.0);
            float bOut_1 = metal::clamp(lum_5 + ((bP_1 - lum_5) * azoSat), 0.0, 1.0);
            return metal::float3(rOut_1, gOut_1, bOut_1);
        } else {
            if (mediaMode == 3) {
                metal::float3 _e190 = getEnlargerFilterWeights(filterMode_1);
                float rawLum = metal::clamp(metal::dot(rgb_4, _e190), 0.0, 1.0);
                float _e195 = applyPaperGrade(rawLum, paperGrade);
                float dmaxGamma = 1.0 + (0.08 * depth);
                float silverToned = metal::pow(_e195, dmaxGamma);
                return metal::float3(silverToned);
            } else {
                if (mediaMode == 4) {
                    metal::float3 _e204 = getEnlargerFilterWeights(filterMode_1);
                    float rawLum_1 = metal::clamp(metal::dot(rgb_4, _e204), 0.0, 1.0);
                    float _e209 = applyPaperGrade(rawLum_1, paperGrade);
                    float shadowPow_1 = 1.0 + (0.1 * depth);
                    float silverDeep = metal::pow(_e209, shadowPow_1);
                    float midMask = (4.0 * _e209) * (1.0 - _e209);
                    float shadowMask = ((4.0 * _e209) * (1.0 - _e209)) * (1.0 - _e209);
                    float rTone = silverDeep + (depth * ((midMask * 0.025) + (shadowMask * 0.02)));
                    float gTone = silverDeep + (depth * ((midMask * 0.01) - (shadowMask * 0.012)));
                    float bTone = silverDeep + (depth * ((-(midMask) * 0.015) + (shadowMask * 0.025)));
                    metal::float3 toned = metal::clamp(metal::float3(rTone, gTone, bTone), metal::float3(0.0), metal::float3(1.0));
                    metal::float3 _e257 = applyBaseWarmth(toned, 5.0);
                    return _e257;
                } else {
                    if (mediaMode == 5) {
                        metal::float3 _e260 = getEnlargerFilterWeights(filterMode_1);
                        float rawLum_2 = metal::clamp(metal::dot(rgb_4, _e260), 0.0, 1.0);
                        float _e265 = applyPaperGrade(rawLum_2, paperGrade);
                        float ptScale = _e265 + ((((depth * 0.1) * _e265) * (1.0 - _e265)) * (1.0 - _e265));
                        float midWarmth = (4.0 * _e265) * (1.0 - _e265);
                        float rTone_1 = ptScale + ((depth * midWarmth) * 0.04);
                        float gTone_1 = ptScale + ((depth * midWarmth) * 0.024);
                        float bTone_1 = ptScale - ((depth * midWarmth) * 0.016);
                        metal::float3 toned_1 = metal::clamp(metal::float3(rTone_1, gTone_1, bTone_1), metal::float3(0.0), metal::float3(1.0));
                        metal::float3 _e301 = applyBaseWarmth(toned_1, 10.0 * depth);
                        return _e301;
                    } else {
                        if (mediaMode == 6) {
                            metal::float3 _e304 = getEnlargerFilterWeights(filterMode_1);
                            float rawLum_3 = metal::clamp(metal::dot(rgb_4, _e304), 0.0, 1.0);
                            float _e309 = applyPaperGrade(rawLum_3, paperGrade);
                            float density = 1.0 - _e309;
                            float rP_2 = _e309 * (1.0 - ((0.85 * depth) * density));
                            float gP_2 = _e309 * (1.0 - ((0.45 * depth) * density));
                            float bP_2 = _e309 * (1.0 - ((0.05 * depth) * density));
                            metal::float3 toned_2 = metal::clamp(metal::float3(rP_2, gP_2, bP_2), metal::float3(0.0), metal::float3(1.0));
                            metal::float3 _e338 = applyBaseWarmth(toned_2, 8.0 * depth);
                            return _e338;
                        } else {
                            if (mediaMode == 7) {
                                metal::float3 _e341 = getEnlargerFilterWeights(filterMode_1);
                                float rawLum_4 = metal::clamp(metal::dot(rgb_4, _e341), 0.0, 1.0);
                                float _e346 = applyPaperGrade(rawLum_4, paperGrade);
                                float c_11 = metal::clamp(_e346, 0.0, 1.0);
                                float sLum = metal::clamp((c_11 + (((0.22 * c_11) * (1.0 - c_11)) * ((2.0 * c_11) - 1.0))) + ((0.14 * c_11) * (1.0 - c_11)), 0.0, 1.0);
                                float ferro = metal::pow(sLum, 1.0 + (0.15 * depth));
                                float silver = metal::clamp((ferro * depth) + (_e346 * (1.0 - depth)), 0.0, 1.0);
                                return metal::float3(silver);
                            }
                        }
                    }
                }
            }
        }
    }
    return rgb_4;
}

float applyFilmCharacteristicCurve(
    float val_1,
    float gamma,
    float shoulder,
    float toe
) {
    float base = {};
    float v_3 = metal::clamp(val_1, 0.0, 2.0);
    base = v_3;
    if (metal::abs(gamma - 1.0) > 0.001) {
        float u_7 = metal::clamp(v_3 / (v_3 + 0.35), 0.0, 1.0);
        float gMod = (gamma - 1.0) * 0.4;
        base = metal::max(v_3 + (((gMod * u_7) * (1.0 - u_7)) * ((2.0 * u_7) - 1.0)), 0.0);
    }
    float _e35 = base;
    float tS = metal::clamp((_e35 - 0.65) / 0.35, 0.0, 1.0);
    float compS = (((shoulder * 0.12) * tS) * tS) * (3.0 - (2.0 * tS));
    float _e52 = base;
    float tT = metal::clamp(_e52 / 0.35, 0.0, 1.0);
    float compT = ((toe * 0.06) * (1.0 - tT)) * (1.0 - tT);
    float _e66 = base;
    return metal::max((_e66 - compS) - compT, 0.0);
}

metal::float3 applyDirChemicalCoupling(
    metal::float3 rgb_5,
    float c_rg,
    float c_rb,
    float c_gr,
    float c_gb,
    float c_br,
    float c_bg
) {
    float d_r = -((metal::log(metal::max(rgb_5.x, 0.0001)) * 0.4342945));
    float d_g = -((metal::log(metal::max(rgb_5.y, 0.0001)) * 0.4342945));
    float d_b = -((metal::log(metal::max(rgb_5.z, 0.0001)) * 0.4342945));
    float c_rr = (1.0 - c_rg) - c_rb;
    float c_gg = (1.0 - c_gr) - c_gb;
    float c_bb = (1.0 - c_br) - c_bg;
    float d_r_out = ((c_rr * d_r) + (c_rg * d_g)) + (c_rb * d_b);
    float d_g_out = ((c_gr * d_r) + (c_gg * d_g)) + (c_gb * d_b);
    float d_b_out = ((c_br * d_r) + (c_bg * d_g)) + (c_bb * d_b);
    return metal::clamp(metal::float3(metal::pow(10.0, -(d_r_out)), metal::pow(10.0, -(d_g_out)), metal::pow(10.0, -(d_b_out))), metal::float3(0.0), metal::float3(2.0));
}

float applyEmulsionSCurve(
    float x_4,
    float alpha,
    float beta
) {
    float c_12 = metal::clamp(x_4, 0.0, 1.0);
    float poly = (c_12 + (((alpha * c_12) * (1.0 - c_12)) * ((2.0 * c_12) - 1.0))) + ((beta * c_12) * (1.0 - c_12));
    return metal::clamp(poly, 0.0, 1.0);
}

metal::float3 applyRecordingEmulsion(
    metal::float3 rgb_6,
    int archetype_1,
    float pushPullEv,
    int filterMode_2
) {
    if (archetype_1 == 0) {
        return rgb_6;
    }
    float expGain = metal::pow(2.0, metal::clamp(pushPullEv, -2.0, 2.0));
    metal::float3 curRgb = metal::max(rgb_6 * expGain, metal::float3(0.0));
    if (archetype_1 == 1) {
        metal::float3 _e23 = applyDirChemicalCoupling(curRgb, -0.04, 0.02, -0.08, 0.02, 0.02, -0.05);
        float _e28 = applyFilmCharacteristicCurve(_e23.x, 0.92, 0.38, 0.04);
        float _e33 = applyFilmCharacteristicCurve(_e23.y, 0.9, 0.38, 0.06);
        float _e38 = applyFilmCharacteristicCurve(_e23.z, 0.88, 0.4, 0.12);
        return metal::float3(_e28, _e33, _e38);
    } else {
        if (archetype_1 == 2) {
            metal::float3 _e48 = applyDirChemicalCoupling(curRgb, -0.15, -0.08, -0.18, -0.08, -0.06, -0.12);
            float _e53 = applyFilmCharacteristicCurve(_e48.x, 1.1, 0.12, 0.14);
            float _e58 = applyFilmCharacteristicCurve(_e48.y, 1.12, 0.1, 0.14);
            float _e63 = applyFilmCharacteristicCurve(_e48.z, 1.11, 0.12, 0.16);
            return metal::float3(_e53, _e58, _e63);
        } else {
            if (archetype_1 == 3) {
                metal::float3 _e73 = applyDirChemicalCoupling(curRgb, -0.08, 0.06, -0.1, 0.04, 0.08, -0.04);
                float _e78 = applyFilmCharacteristicCurve(_e73.x, 1.05, 0.18, 0.08);
                float _e83 = applyFilmCharacteristicCurve(_e73.y, 1.06, 0.16, 0.1);
                float _e88 = applyFilmCharacteristicCurve(_e73.z, 1.08, 0.14, 0.14);
                return metal::float3(_e78, _e83, _e88);
            } else {
                if (archetype_1 == 4) {
                    metal::float3 _e98 = applyDirChemicalCoupling(curRgb, -0.02, 0.01, -0.03, 0.01, 0.01, -0.02);
                    float _e103 = applyFilmCharacteristicCurve(_e98.x, 0.82, 0.45, 0.08);
                    float _e108 = applyFilmCharacteristicCurve(_e98.y, 0.82, 0.45, 0.08);
                    float _e113 = applyFilmCharacteristicCurve(_e98.z, 0.82, 0.45, 0.08);
                    return metal::float3(_e103, _e108, _e113);
                } else {
                    if (archetype_1 == 5) {
                        metal::float3 _e123 = applyDirChemicalCoupling(curRgb, -0.02, 0.04, -0.06, 0.03, 0.04, -0.03);
                        float _e128 = applyFilmCharacteristicCurve(_e123.x, 1.12, 0.22, 0.05);
                        float _e133 = applyFilmCharacteristicCurve(_e123.y, 1.08, 0.22, 0.08);
                        float _e138 = applyFilmCharacteristicCurve(_e123.z, 0.98, 0.24, 0.16);
                        return metal::float3(_e128, _e133, _e138);
                    } else {
                        if (archetype_1 == 6) {
                            metal::float3 _e142 = getEnlargerFilterWeights(filterMode_2);
                            float panLum = metal::dot(curRgb, _e142);
                            float _e147 = applyFilmCharacteristicCurve(panLum, 1.08, 0.3, 0.25);
                            return metal::float3(_e147);
                        } else {
                            if (archetype_1 == 7) {
                                metal::float3 _e157 = applyDirChemicalCoupling(curRgb, -0.03, 0.02, -0.04, 0.03, 0.01, -0.03);
                                float _e162 = applyFilmCharacteristicCurve(_e157.x, 0.98, 0.35, 0.12);
                                float _e167 = applyFilmCharacteristicCurve(_e157.y, 0.98, 0.35, 0.12);
                                float _e172 = applyFilmCharacteristicCurve(_e157.z, 0.98, 0.35, 0.12);
                                return metal::float3(_e162, _e167, _e172);
                            } else {
                                if (archetype_1 == 8) {
                                    metal::float3 _e182 = applyDirChemicalCoupling(curRgb, -0.05, 0.01, -0.07, 0.02, 0.02, -0.04);
                                    float _e187 = applyFilmCharacteristicCurve(_e182.x, 0.96, 0.32, 0.08);
                                    float _e192 = applyFilmCharacteristicCurve(_e182.y, 0.94, 0.32, 0.03);
                                    float _e197 = applyFilmCharacteristicCurve(_e182.z, 0.92, 0.3, 0.1);
                                    return metal::float3(_e187, _e192, _e197);
                                } else {
                                    if (archetype_1 == 9) {
                                        metal::float3 _e207 = applyDirChemicalCoupling(curRgb, -0.06, 0.02, -0.14, 0.05, 0.04, -0.08);
                                        float _e212 = applyFilmCharacteristicCurve(_e207.x, 0.92, 0.36, 0.14);
                                        float _e217 = applyFilmCharacteristicCurve(_e207.y, 0.92, 0.36, 0.04);
                                        float _e222 = applyFilmCharacteristicCurve(_e207.z, 0.94, 0.34, 0.04);
                                        return metal::float3(_e212, _e217, _e222);
                                    } else {
                                        if (archetype_1 == 10) {
                                            metal::float3 _e226 = getEnlargerFilterWeights(filterMode_2);
                                            float panLum_1 = metal::dot(curRgb, _e226);
                                            float _e230 = applyEmulsionSCurve(panLum_1, 0.32, -0.08);
                                            return metal::float3(_e230);
                                        } else {
                                            if (archetype_1 == 11) {
                                                metal::float3 _e234 = getEnlargerFilterWeights(filterMode_2);
                                                float panLum_2 = metal::dot(curRgb, _e234);
                                                float _e238 = applyEmulsionSCurve(panLum_2, 0.18, 0.1);
                                                return metal::float3(_e238);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return rgb_6;
}

float ign(
    metal::float2 p_2
) {
    metal::float3 ignCoeffs = metal::float3(0.06711056, 0.00583715, 52.982918);
    return metal::fract(ignCoeffs.z * metal::fract(metal::dot(p_2, ignCoeffs.xy)));
}

struct vertexShaderInput {
};
struct vertexShaderOutput {
    metal::float4 position [[position]];
    metal::float2 texCoord [[user(loc0), center_perspective]];
};
vertex vertexShaderOutput vertexShader(
  uint vertex_id [[vertex_id]]
, constant VertexQuad& quad_ [[buffer(0)]]
) {
    VertexOutput out = {};
    metal::float2 _e7 = quad_.vertices.inner[vertex_id].position;
    out.position = metal::float4(_e7, 0.0, 1.0);
    metal::float2 _e16 = quad_.vertices.inner[vertex_id].texCoord;
    out.texCoord = _e16;
    VertexOutput _e17 = out;
    const auto _tmp = _e17;
    return vertexShaderOutput { _tmp.position, _tmp.texCoord };
}


struct fragmentShaderInput {
    metal::float2 texCoord [[user(loc0), center_perspective]];
};
struct fragmentShaderOutput {
    metal::float4 member_1 [[color(0)]];
};
fragment fragmentShaderOutput fragmentShader(
  fragmentShaderInput varyings_1 [[stage_in]]
, metal::float4 position [[position]]
, constant ShaderUniforms& uniforms_2 [[buffer(1)]]
, metal::texture2d<float, metal::access::sample> u_texture [[texture(0)]]
, metal::sampler u_sampler [[sampler(0)]]
, metal::texture3d<float, metal::access::sample> u_lut_3d [[texture(1)]]
, metal::sampler u_lut_sampler [[sampler(1)]]
, metal::texture1d<float, metal::access::sample> u_curve_master_lut [[texture(2)]]
, metal::texture1d<float, metal::access::sample> u_curve_rgb_lut [[texture(3)]]
, metal::sampler u_curve_sampler [[sampler(2)]]
, metal::texture2d<float, metal::access::sample> u_gain_map [[texture(4)]]
, metal::sampler u_gain_map_sampler [[sampler(3)]]
) {
    const VertexOutput in = { position, varyings_1.texCoord };
    bool local = {};
    bool local_1 = {};
    bool local_2 = {};
    metal::float4 sampledColor = {};
    bool local_3 = {};
    bool local_4 = {};
    bool local_5 = {};
    bool local_6 = {};
    bool local_7 = {};
    bool local_8 = {};
    bool local_9 = {};
    bool local_10 = {};
    bool local_11 = {};
    bool local_12 = {};
    bool local_13 = {};
    bool local_14 = {};
    bool local_15 = {};
    bool local_16 = {};
    bool local_17 = {};
    bool local_18 = {};
    bool local_19 = {};
    bool local_20 = {};
    bool local_21 = {};
    bool local_22 = {};
    bool local_23 = {};
    bool local_24 = {};
    bool local_25 = {};
    bool local_26 = {};
    bool local_27 = {};
    bool local_28 = {};
    bool local_29 = {};
    float editExposure = {};
    float editTemp = {};
    float editTint = {};
    float editContrast = {};
    float editHighlights = {};
    float editShadows = {};
    float editWhites = {};
    float editBlacks = {};
    float editVibrance = {};
    float editSaturation = {};
    float editDeepBlue = {};
    float editFoliage = {};
    float editSplitShadowSat = {};
    float editSplitHighlightSat = {};
    metal::float3 rec2020Color = {};
    float rScale = 1.0;
    float gScale = 1.0;
    float bScale = 1.0;
    float baseTemp = 5500.0;
    bool local_30 = {};
    float baseTint = 0.0;
    bool local_31 = {};
    float exposureGain = 1.0;
    bool local_32 = {};
    bool local_33 = {};
    bool local_34 = {};
    type_11 y_neighbors = type_11 {{0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0}};
    int i = 0;
    metal::float3 c = {};
    bool local_35 = {};
    bool local_36 = {};
    bool local_37 = {};
    float formatMag = 1.0;
    metal::float2 halationRadius = {};
    bool local_38 = {};
    float scatterAcc = 0.0;
    int i_1 = 0;
    metal::float3 c_1 = {};
    bool local_39 = {};
    float editSharpening = {};
    float editNoiseReduction = {};
    int editGrainMode = {};
    bool local_40 = {};
    bool local_41 = {};
    bool local_42 = {};
    float formatMag_1 = 1.0;
    bool local_43 = {};
    bool local_44 = {};
    float y_l = {};
    float y_r = {};
    float y_t = {};
    float y_b = {};
    float y_tl = {};
    float y_tr = {};
    float y_bl = {};
    float y_br = {};
    bool local_45 = {};
    bool local_46 = {};
    float gy_l = {};
    float gy_r = {};
    float gy_t = {};
    float gy_b = {};
    float gy_tl = {};
    float gy_tr = {};
    float gy_bl = {};
    float gy_br = {};
    float contrastGain = {};
    metal::float3 srgbLinearOut = {};
    bool local_47 = {};
    metal::float3 finalColor = {};
    bool local_48 = {};
    bool local_49 = {};
    bool local_50 = {};
    bool local_51 = {};
    bool local_52 = {};
    bool local_53 = {};
    bool local_54 = {};
    bool local_55 = {};
    bool local_56 = {};
    bool local_57 = {};
    bool local_58 = {};
    bool local_59 = {};
    bool local_60 = {};
    bool local_61 = {};
    bool local_62 = {};
    bool local_63 = {};
    bool local_64 = {};
    ShaderUniforms _e3 = uniforms_2;
    GeometryCoordinates _e4 = transformGeometryChannels(in.texCoord, _e3);
    metal::float2 uv_2 = _e4.uv;
    if (!((_e4.uv.x < 0.0))) {
        local = _e4.uv.x > 1.0;
    } else {
        local = true;
    }
    bool _e16 = local;
    if (!(_e16)) {
        local_1 = _e4.uv.y < 0.0;
    } else {
        local_1 = true;
    }
    bool _e24 = local_1;
    if (!(_e24)) {
        local_2 = _e4.uv.y > 1.0;
    } else {
        local_2 = true;
    }
    bool _e32 = local_2;
    if (_e32) {
        return fragmentShaderOutput { metal::float4(0.08, 0.08, 0.08, 1.0) };
    }
    int _e41 = uniforms_2.tcaCorrectionEnabled;
    if (_e41 != 0) {
        int _e48 = uniforms_2.hasTca;
        local_3 = _e48 != 0;
    } else {
        local_3 = false;
    }
    bool _e52 = local_3;
    if (_e52) {
        metal::float4 sampleG = u_texture.sample(u_sampler, uv_2);
        metal::float4 sampleR = u_texture.sample(u_sampler, _e4.uvRed);
        metal::float4 sampleB = u_texture.sample(u_sampler, _e4.uvBlue);
        sampledColor = metal::float4(sampleR.x, sampleG.y, sampleB.z, sampleG.w);
    } else {
        metal::float4 _e71 = u_texture.sample(u_sampler, uv_2);
        sampledColor = _e71;
    }
    int _e74 = uniforms_2.isIntermediate;
    if (_e74 == 1) {
        metal::float4 _e77 = sampledColor;
        return fragmentShaderOutput { _e77 };
    }
    float _e80 = uniforms_2.isOriginal;
    if (_e80 > 0.5) {
        int _e87 = uniforms_2.isLinearRaw;
        local_4 = _e87 == 0;
    } else {
        local_4 = false;
    }
    bool _e91 = local_4;
    if (_e91) {
        metal::float4 _e92 = sampledColor;
        return fragmentShaderOutput { _e92 };
    }
    float _e95 = uniforms_2.exposure;
    if (metal::abs(_e95) < 0.001) {
        float _e103 = uniforms_2.temperature;
        local_5 = metal::abs(_e103 - 5500.0) < 1.0;
    } else {
        local_5 = false;
    }
    bool _e110 = local_5;
    if (_e110) {
        float _e115 = uniforms_2.tint;
        local_6 = metal::abs(_e115) < 0.001;
    } else {
        local_6 = false;
    }
    bool _e120 = local_6;
    if (_e120) {
        float _e125 = uniforms_2.contrast;
        local_7 = metal::abs(_e125) < 0.001;
    } else {
        local_7 = false;
    }
    bool _e130 = local_7;
    if (_e130) {
        float _e135 = uniforms_2.highlights;
        local_8 = metal::abs(_e135) < 0.001;
    } else {
        local_8 = false;
    }
    bool _e140 = local_8;
    if (_e140) {
        float _e145 = uniforms_2.shadows;
        local_9 = metal::abs(_e145) < 0.001;
    } else {
        local_9 = false;
    }
    bool _e150 = local_9;
    if (_e150) {
        float _e155 = uniforms_2.whites;
        local_10 = metal::abs(_e155) < 0.001;
    } else {
        local_10 = false;
    }
    bool _e160 = local_10;
    if (_e160) {
        float _e165 = uniforms_2.blacks;
        local_11 = metal::abs(_e165) < 0.001;
    } else {
        local_11 = false;
    }
    bool _e170 = local_11;
    if (_e170) {
        float _e175 = uniforms_2.vibrance;
        local_12 = metal::abs(_e175) < 0.001;
    } else {
        local_12 = false;
    }
    bool _e180 = local_12;
    if (_e180) {
        float _e185 = uniforms_2.saturation;
        local_13 = metal::abs(_e185) < 0.001;
    } else {
        local_13 = false;
    }
    bool _e190 = local_13;
    if (_e190) {
        float _e195 = uniforms_2.deepBlue;
        local_14 = metal::abs(_e195) < 0.001;
    } else {
        local_14 = false;
    }
    bool _e200 = local_14;
    if (_e200) {
        float _e205 = uniforms_2.foliage;
        local_15 = metal::abs(_e205) < 0.001;
    } else {
        local_15 = false;
    }
    bool _e210 = local_15;
    if (_e210) {
        float _e215 = uniforms_2.splitToningShadowSat;
        local_16 = metal::abs(_e215) < 0.001;
    } else {
        local_16 = false;
    }
    bool _e220 = local_16;
    if (_e220) {
        float _e225 = uniforms_2.splitToningHighlightSat;
        local_17 = metal::abs(_e225) < 0.001;
    } else {
        local_17 = false;
    }
    bool _e230 = local_17;
    if (_e230) {
        int _e235 = uniforms_2.lutEnabled;
        local_18 = _e235 == 0;
    } else {
        local_18 = false;
    }
    bool _e239 = local_18;
    if (_e239) {
        float _e244 = uniforms_2.sharpeningAmount;
        local_19 = _e244 == 0.0;
    } else {
        local_19 = false;
    }
    bool _e248 = local_19;
    if (_e248) {
        float _e253 = uniforms_2.noiseReduction;
        local_20 = _e253 == 0.0;
    } else {
        local_20 = false;
    }
    bool _e257 = local_20;
    if (_e257) {
        float _e262 = uniforms_2.defringeAmount;
        local_21 = _e262 == 0.0;
    } else {
        local_21 = false;
    }
    bool _e266 = local_21;
    if (_e266) {
        int _e271 = uniforms_2.emulsionArchetype;
        local_22 = _e271 == 0;
    } else {
        local_22 = false;
    }
    bool _e275 = local_22;
    if (_e275) {
        int _e280 = uniforms_2.halationMode;
        local_23 = _e280 == 0;
    } else {
        local_23 = false;
    }
    bool _e284 = local_23;
    if (_e284) {
        int _e289 = uniforms_2.emulsionGrainMode;
        local_24 = _e289 == 0;
    } else {
        local_24 = false;
    }
    bool _e293 = local_24;
    if (_e293) {
        int _e298 = uniforms_2.printMediaMode;
        local_25 = _e298 == 0;
    } else {
        local_25 = false;
    }
    bool isNeutralEdits = local_25;
    int _e305 = uniforms_2.isLinearRaw;
    if (_e305 == 0) {
        local_26 = isNeutralEdits;
    } else {
        local_26 = false;
    }
    bool _e311 = local_26;
    if (_e311) {
        int _e316 = uniforms_2.hasGainMap;
        local_27 = _e316 == 0;
    } else {
        local_27 = false;
    }
    bool _e320 = local_27;
    if (_e320) {
        int _e325 = uniforms_2.edrEnabled;
        if (!((_e325 == 0))) {
            float _e333 = uniforms_2.edrHeadroom;
            local_29 = _e333 <= 1.001;
        } else {
            local_29 = true;
        }
        bool _e337 = local_29;
        local_28 = _e337;
    } else {
        local_28 = false;
    }
    bool _e339 = local_28;
    if (_e339) {
        metal::float4 _e340 = sampledColor;
        return fragmentShaderOutput { _e340 };
    }
    float _e343 = uniforms_2.exposure;
    editExposure = _e343;
    float _e347 = uniforms_2.temperature;
    editTemp = _e347;
    float _e351 = uniforms_2.tint;
    editTint = _e351;
    float _e355 = uniforms_2.contrast;
    editContrast = _e355;
    float _e359 = uniforms_2.highlights;
    editHighlights = _e359;
    float _e363 = uniforms_2.shadows;
    editShadows = _e363;
    float _e367 = uniforms_2.whites;
    editWhites = _e367;
    float _e371 = uniforms_2.blacks;
    editBlacks = _e371;
    float _e375 = uniforms_2.vibrance;
    editVibrance = _e375;
    float _e379 = uniforms_2.saturation;
    editSaturation = _e379;
    float _e383 = uniforms_2.deepBlue;
    editDeepBlue = _e383;
    float _e387 = uniforms_2.foliage;
    editFoliage = _e387;
    float _e391 = uniforms_2.splitToningShadowSat;
    editSplitShadowSat = _e391;
    float _e395 = uniforms_2.splitToningHighlightSat;
    editSplitHighlightSat = _e395;
    float _e399 = uniforms_2.isOriginal;
    if (_e399 > 0.5) {
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
    int _e419 = uniforms_2.isLinearRaw;
    if (_e419 == 1) {
        metal::float4 _e422 = sampledColor;
        rec2020Color = _e422.xyz;
    } else {
        metal::float4 _e424 = sampledColor;
        metal::float3 _e426 = srgbGammaToLinear3_(_e424.xyz);
        rec2020Color = LINEAR_SRGB_TO_REC2020_ * _e426;
    }
    int _e439 = uniforms_2.isLinearRaw;
    if (_e439 == 1) {
        float _e446 = uniforms_2.asShotTemperature;
        local_30 = _e446 > 1000.0;
    } else {
        local_30 = false;
    }
    bool _e450 = local_30;
    if (_e450) {
        float _e453 = uniforms_2.asShotTemperature;
        baseTemp = _e453;
    }
    int _e458 = uniforms_2.isLinearRaw;
    if (_e458 == 1) {
        float _e463 = uniforms_2.asShotTint;
        baseTint = _e463;
    }
    float _e464 = editTemp;
    float _e465 = baseTemp;
    if (!((metal::abs(_e464 - _e465) >= 1.0))) {
        float _e473 = editTint;
        float _e474 = baseTint;
        local_31 = metal::abs(_e473 - _e474) >= 0.1;
    } else {
        local_31 = true;
    }
    bool _e480 = local_31;
    if (_e480) {
        float _e481 = editTemp;
        float _e482 = baseTemp;
        float tempNorm = metal::clamp(_e481 / _e482, 0.25, 4.0);
        rScale = metal::pow(tempNorm, 0.75);
        bScale = metal::pow(1.0 / tempNorm, 0.75);
        float _e493 = editTint;
        float _e494 = baseTint;
        float tintFactor = (_e493 - _e494) * 0.002;
        gScale = metal::max(1.0 - tintFactor, 0.01);
        float _e502 = rScale;
        rScale = metal::max(_e502 + (tintFactor * 0.5), 0.01);
        float _e508 = bScale;
        bScale = metal::max(_e508 + (tintFactor * 0.5), 0.01);
        float _e516 = rec2020Color.x;
        float _e517 = rScale;
        rec2020Color.x = _e516 * _e517;
        float _e521 = rec2020Color.y;
        float _e522 = gScale;
        rec2020Color.y = _e521 * _e522;
        float _e526 = rec2020Color.z;
        float _e527 = bScale;
        rec2020Color.z = _e526 * _e527;
    }
    float totalExposure = editExposure;
    if (totalExposure != 0.0) {
        exposureGain = metal::pow(2.0, totalExposure);
        metal::float3 _e536 = rec2020Color;
        float _e537 = exposureGain;
        rec2020Color = _e536 * _e537;
    }
    float _e539 = editHighlights;
    if (!((_e539 != 0.0))) {
        float _e545 = editShadows;
        local_32 = _e545 != 0.0;
    } else {
        local_32 = true;
    }
    bool _e549 = local_32;
    if (!(_e549)) {
        float _e553 = editWhites;
        local_33 = _e553 != 0.0;
    } else {
        local_33 = true;
    }
    bool _e557 = local_33;
    if (!(_e557)) {
        float _e561 = editBlacks;
        local_34 = _e561 != 0.0;
    } else {
        local_34 = true;
    }
    bool _e565 = local_34;
    if (_e565) {
        metal::float2 texDim = static_cast<metal::float2>(metal::uint2(u_texture.get_width(), u_texture.get_height()));
        metal::float2 texelSize = metal::float2(1.0) / texDim;
        float maxDim = metal::max(texDim.x, texDim.y);
        float rInner = metal::max(3.0, metal::rint(maxDim * 0.003));
        float rOuter = metal::max(24.0, metal::rint(maxDim * 0.024));
        metal::float2 stepInner = texelSize * rInner;
        metal::float2 stepOuter = texelSize * rOuter;
        metal::float3 _e587 = rec2020Color;
        float y_c_4 = metal::dot(_e587, metal::float3(0.2627, 0.678, 0.0593));
        float _e593 = rScale;
        float _e594 = gScale;
        float _e595 = bScale;
        float _e597 = exposureGain;
        metal::float3 wbExposureScale = metal::float3(_e593, _e594, _e595) * _e597;
        metal::float4 s_l = u_texture.sample(u_sampler, uv_2 + metal::float2(-(stepInner.x), 0.0));
        metal::float4 s_r = u_texture.sample(u_sampler, uv_2 + metal::float2(stepInner.x, 0.0));
        metal::float4 s_t = u_texture.sample(u_sampler, uv_2 + metal::float2(0.0, -(stepInner.y)));
        metal::float4 s_b = u_texture.sample(u_sampler, uv_2 + metal::float2(0.0, stepInner.y));
        metal::float4 s_tl = u_texture.sample(u_sampler, uv_2 + metal::float2(-(stepInner.x), -(stepInner.y)));
        metal::float4 s_tr = u_texture.sample(u_sampler, uv_2 + metal::float2(stepInner.x, -(stepInner.y)));
        metal::float4 s_bl = u_texture.sample(u_sampler, uv_2 + metal::float2(-(stepInner.x), stepInner.y));
        metal::float4 s_br = u_texture.sample(u_sampler, uv_2 + metal::float2(stepInner.x, stepInner.y));
        metal::float4 s_ol = u_texture.sample(u_sampler, uv_2 + metal::float2(-(stepOuter.x), 0.0));
        metal::float4 s_or = u_texture.sample(u_sampler, uv_2 + metal::float2(stepOuter.x, 0.0));
        metal::float4 s_ot = u_texture.sample(u_sampler, uv_2 + metal::float2(0.0, -(stepOuter.y)));
        metal::float4 s_ob = u_texture.sample(u_sampler, uv_2 + metal::float2(0.0, stepOuter.y));
        metal::float4 s_otl = u_texture.sample(u_sampler, uv_2 + metal::float2(-(stepOuter.x), -(stepOuter.y)));
        metal::float4 s_otr = u_texture.sample(u_sampler, uv_2 + metal::float2(stepOuter.x, -(stepOuter.y)));
        metal::float4 s_obl = u_texture.sample(u_sampler, uv_2 + metal::float2(-(stepOuter.x), stepOuter.y));
        metal::float4 s_obr = u_texture.sample(u_sampler, uv_2 + metal::float2(stepOuter.x, stepOuter.y));
        type_19 s_neighbors = type_19 {{s_l, s_r, s_t, s_b, s_tl, s_tr, s_bl, s_br, s_ol, s_or, s_ot, s_ob, s_otl, s_otr, s_obl, s_obr}};
        uint2 loop_bound_6 = uint2(4294967295u);
        bool loop_init_6 = true;
        while(true) {
            if (metal::all(loop_bound_6 == uint2(0u))) { break; }
            loop_bound_6 -= uint2(loop_bound_6.y == 0u, 1u);
            if (!loop_init_6) {
                int _e773 = i;
                i = as_type<int>(as_type<uint>(_e773) + as_type<uint>(1));
            }
            loop_init_6 = false;
            int _e744 = i;
            if (_e744 < 16) {
            } else {
                break;
            }
            {
                c = metal::float3 {};
                int _e751 = uniforms_2.isLinearRaw;
                if (_e751 == 1) {
                    int _e754 = i;
                    c = s_neighbors.inner[_e754].xyz;
                } else {
                    int _e757 = i;
                    metal::float3 _e760 = srgbGammaToLinear3_(s_neighbors.inner[_e757].xyz);
                    c = LINEAR_SRGB_TO_REC2020_ * _e760;
                }
                metal::float3 _e763 = c;
                c = _e763 * wbExposureScale;
                int _e765 = i;
                metal::float3 _e767 = c;
                y_neighbors.inner[_e765] = metal::dot(_e767, metal::float3(0.2627, 0.678, 0.0593));
            }
        }
        metal::float3 _e776 = rec2020Color;
        type_11 _e777 = y_neighbors;
        float _e778 = editHighlights;
        float _e779 = editShadows;
        float _e780 = editWhites;
        float _e781 = editBlacks;
        metal::float3 _e782 = applyBilateralToneRecovery(_e776, y_c_4, _e777, _e778, _e779, _e780, _e781);
        rec2020Color = _e782;
    }
    float _e785 = uniforms_2.isOriginal;
    if (_e785 < 0.5) {
        int _e792 = uniforms_2.emulsionArchetype;
        local_35 = _e792 != 0;
    } else {
        local_35 = false;
    }
    bool _e796 = local_35;
    if (_e796) {
        metal::float3 _e797 = rec2020Color;
        int _e800 = uniforms_2.emulsionArchetype;
        float _e803 = uniforms_2.emulsionExposure;
        int _e806 = uniforms_2.enlargerFilterMode;
        metal::float3 _e807 = applyRecordingEmulsion(_e797, _e800, _e803, _e806);
        rec2020Color = _e807;
    }
    float _e810 = uniforms_2.isOriginal;
    if (_e810 < 0.5) {
        int _e817 = uniforms_2.halationMode;
        local_36 = _e817 > 0;
    } else {
        local_36 = false;
    }
    bool _e821 = local_36;
    if (_e821) {
        int _e826 = uniforms_2.emulsionArchetype;
        local_37 = _e826 != 7;
    } else {
        local_37 = false;
    }
    bool _e830 = local_37;
    if (_e830) {
        int _e835 = uniforms_2.emulsionFormat;
        if (_e835 == 1) {
            formatMag = 0.48;
        } else {
            int _e841 = uniforms_2.emulsionFormat;
            if (_e841 == 2) {
                formatMag = 0.28;
            } else {
                int _e847 = uniforms_2.emulsionFormat;
                if (_e847 == 3) {
                    formatMag = 7.3;
                } else {
                    int _e853 = uniforms_2.emulsionFormat;
                    if (_e853 == 4) {
                        formatMag = 3.4;
                    }
                }
            }
        }
        float _e859 = uniforms_2.aspectRatio;
        float aspect = metal::max(_e859, 0.01);
        int _e865 = uniforms_2.rotation;
        if (!((_e865 == 90))) {
            int _e873 = uniforms_2.rotation;
            local_38 = _e873 == 270;
        } else {
            local_38 = true;
        }
        bool _e877 = local_38;
        if (_e877) {
            float _e880 = formatMag;
            float _e882 = formatMag;
            halationRadius = metal::float2((0.012 * aspect) * _e880, 0.012 * _e882);
        } else {
            float _e888 = formatMag;
            float _e890 = formatMag;
            halationRadius = metal::float2((0.012 / aspect) * _e888, 0.012 * _e890);
        }
        float _e894 = rScale;
        float _e895 = gScale;
        float _e896 = bScale;
        float _e898 = exposureGain;
        metal::float3 wbExposure = metal::float3(_e894, _e895, _e896) * _e898;
        float _e902 = ign(in.position.xy);
        float jitter = _e902 * 6.2831855;
        float cosJ = metal::cos(jitter);
        float sinJ = metal::sin(jitter);
        metal::float2x2 rot_1 = metal::float2x2(metal::float2(cosJ, sinJ), metal::float2(-(sinJ), cosJ));
        uint2 loop_bound_7 = uint2(4294967295u);
        bool loop_init_7 = true;
        while(true) {
            if (metal::all(loop_bound_7 == uint2(0u))) { break; }
            loop_bound_7 -= uint2(loop_bound_7.y == 0u, 1u);
            if (!loop_init_7) {
                int _e962 = i_1;
                i_1 = as_type<int>(as_type<uint>(_e962) + as_type<uint>(1));
            }
            loop_init_7 = false;
            int _e915 = i_1;
            if (_e915 < 16) {
            } else {
                break;
            }
            {
                int _e919 = i_1;
                metal::float2 _e922 = halationRadius;
                metal::float2 rotatedOffset = (rot_1 * FERMAT_DISK.inner[_e919]) * _e922;
                metal::float4 s_5 = u_texture.sample(u_sampler, uv_2 + rotatedOffset);
                c_1 = metal::float3 {};
                int _e932 = uniforms_2.isLinearRaw;
                if (_e932 == 1) {
                    c_1 = s_5.xyz;
                } else {
                    metal::float3 _e937 = srgbGammaToLinear3_(s_5.xyz);
                    c_1 = LINEAR_SRGB_TO_REC2020_ * _e937;
                }
                metal::float3 _e940 = c_1;
                c_1 = _e940 * wbExposure;
                metal::float3 _e942 = c_1;
                float y_tap = metal::dot(_e942, metal::float3(0.2627, 0.678, 0.0593));
                if (y_tap > 0.55) {
                    float diff = y_tap - 0.55;
                    float excess = (diff * diff) / (0.25 + diff);
                    float _e956 = scatterAcc;
                    int _e958 = i_1;
                    scatterAcc = _e956 + (excess * FERMAT_WEIGHTS.inner[_e958]);
                }
            }
        }
        float _e965 = scatterAcc;
        float halo = _e965 / 3.6187;
        if (halo > 0.00001) {
            int _e972 = uniforms_2.halationMode;
            if (_e972 == 1) {
                metal::float3 _e975 = rec2020Color;
                rec2020Color = _e975 + metal::float3(halo * 0.35);
            } else {
                int _e982 = uniforms_2.halationMode;
                if (_e982 == 2) {
                    metal::float3 _e985 = rec2020Color;
                    rec2020Color = _e985 + metal::float3(halo * 0.5, halo * 0.12, halo * 0.02);
                }
            }
        }
    }
    int _e996 = uniforms_2.hasCurves;
    if (_e996 != 0) {
        float _e1003 = uniforms_2.isOriginal;
        local_39 = _e1003 < 0.5;
    } else {
        local_39 = false;
    }
    bool _e1007 = local_39;
    if (_e1007) {
        int _e1010 = uniforms_2.hasMasterCurve;
        if (_e1010 != 0) {
            float _e1017 = rec2020Color.x;
            metal::float4 _e1021 = u_curve_master_lut.sample(u_curve_sampler, metal::clamp(_e1017, 0.0, 1.0));
            rec2020Color.x = _e1021.x;
            float _e1027 = rec2020Color.y;
            metal::float4 _e1031 = u_curve_master_lut.sample(u_curve_sampler, metal::clamp(_e1027, 0.0, 1.0));
            rec2020Color.y = _e1031.x;
            float _e1037 = rec2020Color.z;
            metal::float4 _e1041 = u_curve_master_lut.sample(u_curve_sampler, metal::clamp(_e1037, 0.0, 1.0));
            rec2020Color.z = _e1041.x;
        }
        int _e1045 = uniforms_2.hasRgbCurves;
        if (_e1045 != 0) {
            float _e1051 = rec2020Color.x;
            metal::float4 _e1055 = u_curve_rgb_lut.sample(u_curve_sampler, metal::clamp(_e1051, 0.0, 1.0));
            float _e1060 = rec2020Color.y;
            metal::float4 _e1064 = u_curve_rgb_lut.sample(u_curve_sampler, metal::clamp(_e1060, 0.0, 1.0));
            float _e1069 = rec2020Color.z;
            metal::float4 _e1073 = u_curve_rgb_lut.sample(u_curve_sampler, metal::clamp(_e1069, 0.0, 1.0));
            rec2020Color = metal::float3(_e1055.x, _e1064.y, _e1073.z);
        }
    }
    metal::float3 _e1076 = rec2020Color;
    float _e1077 = editVibrance;
    float _e1078 = editSaturation;
    float _e1079 = editDeepBlue;
    float _e1080 = editFoliage;
    metal::float3 _e1081 = applyColorAdjustments(_e1076, _e1077, _e1078, _e1079, _e1080);
    rec2020Color = _e1081;
    metal::float3 _e1082 = rec2020Color;
    float _e1085 = uniforms_2.splitToningShadowHue;
    float _e1086 = editSplitShadowSat;
    float _e1089 = uniforms_2.splitToningHighlightHue;
    float _e1090 = editSplitHighlightSat;
    float _e1093 = uniforms_2.splitToningBalance;
    metal::float3 _e1094 = applySplitToning(_e1082, _e1085, _e1086, _e1089, _e1090, _e1093);
    rec2020Color = _e1094;
    float _e1097 = uniforms_2.sharpeningAmount;
    editSharpening = _e1097;
    float _e1101 = uniforms_2.noiseReduction;
    editNoiseReduction = _e1101;
    int _e1105 = uniforms_2.emulsionGrainMode;
    editGrainMode = _e1105;
    float _e1109 = uniforms_2.isOriginal;
    if (_e1109 > 0.5) {
        editSharpening = 0.0;
        editNoiseReduction = 0.0;
        editGrainMode = 0;
    }
    float _e1115 = editSharpening;
    if (!((_e1115 > 0.0))) {
        float _e1121 = editNoiseReduction;
        local_40 = _e1121 > 0.0;
    } else {
        local_40 = true;
    }
    bool _e1125 = local_40;
    if (!(_e1125)) {
        int _e1129 = editGrainMode;
        local_41 = _e1129 != 0;
    } else {
        local_41 = true;
    }
    bool _e1133 = local_41;
    if (!(_e1133)) {
        float _e1139 = uniforms_2.defringeAmount;
        local_42 = _e1139 > 0.0;
    } else {
        local_42 = true;
    }
    bool _e1143 = local_42;
    if (_e1143) {
        int _e1148 = uniforms_2.emulsionFormat;
        if (_e1148 == 1) {
            formatMag_1 = 0.48;
        } else {
            int _e1154 = uniforms_2.emulsionFormat;
            if (_e1154 == 2) {
                formatMag_1 = 0.28;
            } else {
                int _e1160 = uniforms_2.emulsionFormat;
                if (_e1160 == 3) {
                    formatMag_1 = 7.3;
                } else {
                    int _e1166 = uniforms_2.emulsionFormat;
                    if (_e1166 == 4) {
                        formatMag_1 = 3.4;
                    }
                }
            }
        }
        metal::float2 texSize = static_cast<metal::float2>(metal::uint2(u_texture.get_width(), u_texture.get_height()));
        metal::float2 grainOffset = metal::float2(1.0) / texSize;
        metal::float2 pixelCoord = uv_2 * texSize;
        metal::float3 _e1177 = rec2020Color;
        float y_c_5 = metal::dot(_e1177, metal::float3(0.2627, 0.678, 0.0593));
        float _e1183 = rScale;
        float _e1184 = gScale;
        float _e1185 = bScale;
        float _e1187 = exposureGain;
        metal::float3 wbExposureScale_1 = metal::float3(_e1183, _e1184, _e1185) * _e1187;
        float _e1189 = editSharpening;
        if (!((_e1189 > 0.0))) {
            float _e1195 = editNoiseReduction;
            local_43 = _e1195 > 0.0;
        } else {
            local_43 = true;
        }
        bool _e1199 = local_43;
        if (!(_e1199)) {
            float _e1205 = uniforms_2.defringeAmount;
            local_44 = _e1205 > 0.0;
        } else {
            local_44 = true;
        }
        bool _e1209 = local_44;
        if (_e1209) {
            float _e1212 = uniforms_2.sharpeningRadius;
            float _e1215 = uniforms_2.baselineSharpness;
            float effectiveRadius = metal::max(_e1212 * _e1215, 0.5);
            metal::float2 sharpenOffset = grainOffset * effectiveRadius;
            metal::float4 leftSample = u_texture.sample(u_sampler, uv_2 - metal::float2(sharpenOffset.x, 0.0));
            metal::float4 rightSample = u_texture.sample(u_sampler, uv_2 + metal::float2(sharpenOffset.x, 0.0));
            metal::float4 topSample = u_texture.sample(u_sampler, uv_2 - metal::float2(0.0, sharpenOffset.y));
            metal::float4 bottomSample = u_texture.sample(u_sampler, uv_2 + metal::float2(0.0, sharpenOffset.y));
            metal::float4 tlSample = u_texture.sample(u_sampler, uv_2 - sharpenOffset);
            metal::float4 trSample = u_texture.sample(u_sampler, uv_2 + metal::float2(sharpenOffset.x, -(sharpenOffset.y)));
            metal::float4 blSample = u_texture.sample(u_sampler, uv_2 + metal::float2(-(sharpenOffset.x), sharpenOffset.y));
            metal::float4 brSample = u_texture.sample(u_sampler, uv_2 + sharpenOffset);
            int _e1282 = uniforms_2.isLinearRaw;
            if (_e1282 == 0) {
                metal::float3 _e1287 = srgbGammaToLinear3_(leftSample.xyz);
                y_l = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1287, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1296 = srgbGammaToLinear3_(rightSample.xyz);
                y_r = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1296, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1305 = srgbGammaToLinear3_(topSample.xyz);
                y_t = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1305, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1314 = srgbGammaToLinear3_(bottomSample.xyz);
                y_b = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1314, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1323 = srgbGammaToLinear3_(tlSample.xyz);
                y_tl = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1323, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1332 = srgbGammaToLinear3_(trSample.xyz);
                y_tr = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1332, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1341 = srgbGammaToLinear3_(blSample.xyz);
                y_bl = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1341, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1350 = srgbGammaToLinear3_(brSample.xyz);
                y_br = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1350, metal::float3(0.2627, 0.678, 0.0593));
            } else {
                y_l = metal::dot(leftSample.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
                y_r = metal::dot(rightSample.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
                y_t = metal::dot(topSample.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
                y_b = metal::dot(bottomSample.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
                y_tl = metal::dot(tlSample.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
                y_tr = metal::dot(trSample.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
                y_bl = metal::dot(blSample.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
                y_br = metal::dot(brSample.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
            }
            float _e1413 = y_l;
            float _e1414 = y_r;
            float _e1416 = y_t;
            float _e1417 = y_b;
            float _e1420 = y_tl;
            float _e1421 = y_tr;
            float _e1423 = y_bl;
            float _e1424 = y_br;
            float y_max_neighbor_1 = metal::max(metal::max(metal::max(_e1413, _e1414), metal::max(_e1416, _e1417)), metal::max(metal::max(_e1420, _e1421), metal::max(_e1423, _e1424)));
            float _e1430 = uniforms_2.defringeAmount;
            if (_e1430 > 0.0) {
                float _e1437 = uniforms_2.isOriginal;
                local_45 = _e1437 < 0.5;
            } else {
                local_45 = false;
            }
            bool _e1441 = local_45;
            if (_e1441) {
                metal::float3 _e1442 = rec2020Color;
                float _e1445 = uniforms_2.defringeAmount;
                metal::float3 _e1446 = applyDefringe(_e1442, y_c_5, y_max_neighbor_1, _e1445);
                rec2020Color = _e1446;
            }
            float _e1447 = editSharpening;
            if (!((_e1447 > 0.0))) {
                float _e1453 = editNoiseReduction;
                local_46 = _e1453 > 0.0;
            } else {
                local_46 = true;
            }
            bool _e1457 = local_46;
            if (_e1457) {
                metal::float3 _e1458 = rec2020Color;
                float _e1459 = y_l;
                float _e1460 = y_r;
                float _e1461 = y_t;
                float _e1462 = y_b;
                float _e1463 = y_tl;
                float _e1464 = y_tr;
                float _e1465 = y_bl;
                float _e1466 = y_br;
                float _e1467 = editSharpening;
                float _e1470 = uniforms_2.sharpeningMasking;
                float _e1471 = editNoiseReduction;
                metal::float3 _e1472 = applyDetailAndSharpening(_e1458, y_c_5, _e1459, _e1460, _e1461, _e1462, _e1463, _e1464, _e1465, _e1466, _e1467, _e1470, _e1471);
                rec2020Color = _e1472;
            }
        }
        int _e1473 = editGrainMode;
        if (_e1473 != 0) {
            metal::float4 leftSample_1 = u_texture.sample(u_sampler, uv_2 - metal::float2(grainOffset.x, 0.0));
            metal::float4 rightSample_1 = u_texture.sample(u_sampler, uv_2 + metal::float2(grainOffset.x, 0.0));
            metal::float4 topSample_1 = u_texture.sample(u_sampler, uv_2 - metal::float2(0.0, grainOffset.y));
            metal::float4 bottomSample_1 = u_texture.sample(u_sampler, uv_2 + metal::float2(0.0, grainOffset.y));
            metal::float4 tlSample_1 = u_texture.sample(u_sampler, uv_2 - grainOffset);
            metal::float4 trSample_1 = u_texture.sample(u_sampler, uv_2 + metal::float2(grainOffset.x, -(grainOffset.y)));
            metal::float4 blSample_1 = u_texture.sample(u_sampler, uv_2 + metal::float2(-(grainOffset.x), grainOffset.y));
            metal::float4 brSample_1 = u_texture.sample(u_sampler, uv_2 + grainOffset);
            int _e1538 = uniforms_2.isLinearRaw;
            if (_e1538 == 0) {
                metal::float3 _e1543 = srgbGammaToLinear3_(leftSample_1.xyz);
                gy_l = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1543, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1552 = srgbGammaToLinear3_(rightSample_1.xyz);
                gy_r = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1552, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1561 = srgbGammaToLinear3_(topSample_1.xyz);
                gy_t = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1561, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1570 = srgbGammaToLinear3_(bottomSample_1.xyz);
                gy_b = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1570, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1579 = srgbGammaToLinear3_(tlSample_1.xyz);
                gy_tl = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1579, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1588 = srgbGammaToLinear3_(trSample_1.xyz);
                gy_tr = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1588, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1597 = srgbGammaToLinear3_(blSample_1.xyz);
                gy_bl = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1597, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1606 = srgbGammaToLinear3_(brSample_1.xyz);
                gy_br = metal::dot(LINEAR_SRGB_TO_REC2020_ * _e1606, metal::float3(0.2627, 0.678, 0.0593));
            } else {
                gy_l = metal::dot(leftSample_1.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
                gy_r = metal::dot(rightSample_1.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
                gy_t = metal::dot(topSample_1.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
                gy_b = metal::dot(bottomSample_1.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
                gy_tl = metal::dot(tlSample_1.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
                gy_tr = metal::dot(trSample_1.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
                gy_bl = metal::dot(blSample_1.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
                gy_br = metal::dot(brSample_1.xyz * wbExposureScale_1, metal::float3(0.2627, 0.678, 0.0593));
            }
            float _e1669 = formatMag_1;
            float s_format = metal::sqrt(metal::max(_e1669, 0.1));
            float _e1675 = uniforms_2.grainSize;
            float clusterSize = metal::clamp(_e1675 * s_format, 0.5, 3.5);
            float _e1682 = uniforms_2.grainAmount;
            float _e1683 = formatMag_1;
            float effectiveAmount = _e1682 * metal::pow(metal::max(_e1683, 0.1), 0.25);
            metal::float3 _e1689 = rec2020Color;
            float _e1690 = gy_l;
            float _e1691 = gy_r;
            float _e1692 = gy_t;
            float _e1693 = gy_b;
            float _e1694 = gy_tl;
            float _e1695 = gy_tr;
            float _e1696 = gy_bl;
            float _e1697 = gy_br;
            int _e1698 = editGrainMode;
            int _e1701 = uniforms_2.emulsionArchetype;
            metal::float3 _e1702 = applyStochasticEmulsionGrain(_e1689, y_c_5, _e1690, _e1691, _e1692, _e1693, _e1694, _e1695, _e1696, _e1697, pixelCoord, _e1698, clusterSize, effectiveAmount, _e1701);
            rec2020Color = _e1702;
        }
    }
    float _e1703 = editContrast;
    contrastGain = 1.0 + (_e1703 / 100.0);
    float _e1709 = editContrast;
    if (_e1709 < 0.0) {
        float _e1712 = editContrast;
        contrastGain = 1.0 + ((_e1712 / 100.0) * 0.6);
    }
    int _e1722 = uniforms_2.isLinearRaw;
    if (_e1722 == 0) {
        metal::float3 _e1726 = rec2020Color;
        metal::float3 srgb = metal::max(REC2020_TO_LINEAR_SRGB * _e1726, metal::float3(0.0));
        float _e1731 = contrastGain;
        if (metal::abs(_e1731 - 1.0) > 0.001) {
            float _e1740 = contrastGain;
            srgbLinearOut = metal::clamp(((srgb - metal::float3(0.18)) * _e1740) + metal::float3(0.18), metal::float3(0.0), metal::float3(1.0));
        } else {
            srgbLinearOut = metal::clamp(srgb, metal::float3(0.0), metal::float3(1.0));
        }
    } else {
        int _e1757 = uniforms_2.toneProfileMode;
        if (_e1757 == 0) {
            metal::float3 _e1760 = rec2020Color;
            float _e1761 = contrastGain;
            metal::float3 _e1762 = agxToneMapVivid(_e1760, _e1761);
            srgbLinearOut = _e1762;
        } else {
            int _e1765 = uniforms_2.toneProfileMode;
            if (_e1765 == 1) {
                metal::float3 _e1768 = rec2020Color;
                float _e1769 = contrastGain;
                metal::float3 _e1770 = agxToneMap(_e1768, _e1769);
                srgbLinearOut = _e1770;
            } else {
                int _e1773 = uniforms_2.toneProfileMode;
                if (_e1773 == 6) {
                    metal::float3 _e1776 = rec2020Color;
                    float _e1777 = contrastGain;
                    metal::float3 _e1778 = agxToneMapPhotographic(_e1776, _e1777);
                    srgbLinearOut = _e1778;
                } else {
                    int _e1781 = uniforms_2.toneProfileMode;
                    int camMode = as_type<int>(as_type<uint>(_e1781) - as_type<uint>(1));
                    metal::float3 _e1785 = rec2020Color;
                    metal::float3 c_srgb_1 = metal::max(REC2020_TO_LINEAR_SRGB * _e1785, metal::float3(0.0));
                    metal::float3 _e1790 = applyCameraHueSatMap(c_srgb_1, camMode);
                    float _e1792 = applyCameraToneCurve(_e1790.x, camMode);
                    float _e1794 = applyCameraToneCurve(_e1790.y, camMode);
                    float _e1796 = applyCameraToneCurve(_e1790.z, camMode);
                    metal::float3 curved_2 = metal::float3(_e1792, _e1794, _e1796);
                    float _e1798 = contrastGain;
                    if (metal::abs(_e1798 - 1.0) > 0.001) {
                        float _e1807 = contrastGain;
                        srgbLinearOut = metal::clamp(((curved_2 - metal::float3(0.18)) * _e1807) + metal::float3(0.18), metal::float3(0.0), metal::float3(1.0));
                    } else {
                        srgbLinearOut = metal::clamp(curved_2, metal::float3(0.0), metal::float3(1.0));
                    }
                }
            }
        }
    }
    float _e1824 = uniforms_2.isOriginal;
    if (_e1824 < 0.5) {
        int _e1831 = uniforms_2.printMediaMode;
        local_47 = _e1831 != 0;
    } else {
        local_47 = false;
    }
    bool _e1835 = local_47;
    if (_e1835) {
        metal::float3 _e1836 = srgbLinearOut;
        int _e1839 = uniforms_2.printMediaMode;
        int _e1842 = uniforms_2.enlargerFilterMode;
        float _e1845 = uniforms_2.printPaperGrade;
        float _e1848 = uniforms_2.printProcessDepth;
        metal::float3 _e1849 = applyPrintMedia(_e1836, _e1839, _e1842, _e1845, _e1848);
        srgbLinearOut = _e1849;
    }
    metal::float3 _e1850 = srgbLinearOut;
    finalColor = _e1850;
    int _e1854 = uniforms_2.lutEnabled;
    if (_e1854 != 0) {
        float _e1861 = uniforms_2.lutIntensity;
        local_48 = _e1861 > 0.0;
    } else {
        local_48 = false;
    }
    bool _e1865 = local_48;
    if (_e1865) {
        float _e1870 = uniforms_2.isOriginal;
        local_49 = _e1870 < 0.5;
    } else {
        local_49 = false;
    }
    bool _e1874 = local_49;
    if (_e1874) {
        float _e1877 = uniforms_2.lutSize;
        float size = metal::max(_e1877, 2.0);
        metal::float3 _e1880 = finalColor;
        metal::float3 lutCoord = ((_e1880 * (size - 1.0)) + metal::float3(0.5)) / metal::float3(size);
        metal::float4 _e1891 = u_lut_3d.sample(u_lut_sampler, lutCoord);
        metal::float3 lutSample = _e1891.xyz;
        metal::float3 _e1893 = finalColor;
        float _e1896 = uniforms_2.lutIntensity;
        finalColor = metal::mix(_e1893, lutSample, metal::clamp(_e1896, 0.0, 1.0));
    }
    int _e1903 = uniforms_2.edrEnabled;
    if (_e1903 != 0) {
        float _e1910 = uniforms_2.edrHeadroom;
        local_50 = _e1910 > 1.001;
    } else {
        local_50 = false;
    }
    bool _e1914 = local_50;
    if (_e1914) {
        float _e1917 = uniforms_2.edrHeadroom;
        float headroom = metal::clamp(_e1917, 1.0, 4.0);
        int _e1923 = uniforms_2.hasGainMap;
        if (_e1923 != 0) {
            metal::float4 _e1928 = u_gain_map.sample(u_gain_map_sampler, uv_2);
            float g_1 = _e1928.x;
            float _e1932 = uniforms_2.gainMapMin;
            float _e1935 = uniforms_2.gainMapMax;
            float _e1938 = uniforms_2.gainMapMin;
            float gainLog2_ = _e1932 + (g_1 * (_e1935 - _e1938));
            float _e1944 = uniforms_2.gainMapGamma;
            float gamma_1 = metal::max(_e1944, 0.01);
            float gainFactor = metal::pow(2.0, gainLog2_ / gamma_1);
            float _e1952 = uniforms_2.gainMapCapacityMax;
            float maxCap = metal::max(_e1952, 1.01);
            float weight = metal::clamp((headroom - 1.0) / (maxCap - 1.0), 0.0, 1.0);
            float effectiveGain = 1.0 + ((gainFactor - 1.0) * weight);
            metal::float3 _e1968 = finalColor;
            finalColor = _e1968 * effectiveGain;
        } else {
            int _e1972 = uniforms_2.isLinearRaw;
            if (_e1972 == 1) {
                metal::float3 _e1975 = rec2020Color;
                float sceneLum = metal::dot(_e1975, metal::float3(0.2627, 0.678, 0.0593));
                metal::float3 _e1981 = finalColor;
                float outLum = metal::dot(_e1981, metal::float3(0.2126, 0.7152, 0.0722));
                float highlightFactor = metal::smoothstep(0.85, 1.0, metal::max(outLum, metal::clamp(sceneLum - 0.15, 0.0, 1.0)));
                if (highlightFactor > 0.0) {
                    float edrBoost = 1.0 + ((headroom - 1.0) * metal::pow(highlightFactor, 1.5));
                    metal::float3 _e2005 = finalColor;
                    finalColor = _e2005 * edrBoost;
                }
            }
        }
        float _e2008 = finalColor.x;
        float _e2010 = finalColor.y;
        float _e2012 = finalColor.z;
        float peakC = metal::max(_e2008, metal::max(_e2010, _e2012));
        if (peakC > headroom) {
            float excess_1 = (peakC - headroom) / peakC;
            metal::float3 _e2018 = finalColor;
            finalColor = metal::mix(_e2018 * (headroom / peakC), metal::float3(headroom), metal::clamp(excess_1 * 1.5, 0.0, 1.0));
        } else {
            if (peakC > 1.0) {
                float desatThreshold = 1.0 + ((headroom - 1.0) * 0.8);
                if (peakC > desatThreshold) {
                    float t_4 = (peakC - desatThreshold) / metal::max(headroom - desatThreshold, 0.0001);
                    metal::float3 _e2042 = finalColor;
                    finalColor = metal::mix(_e2042, metal::float3(peakC), metal::clamp(t_4 * 0.4, 0.0, 0.4));
                }
            }
        }
        metal::float3 _e2051 = finalColor;
        finalColor = metal::max(LINEAR_SRGB_TO_DISPLAY_P3_ * _e2051, metal::float3(0.0));
        int _e2058 = uniforms_2.clippingOverlayActive;
        if (_e2058 != 0) {
            float _e2065 = uniforms_2.isOriginal;
            local_51 = _e2065 < 0.5;
        } else {
            local_51 = false;
        }
        bool _e2069 = local_51;
        if (_e2069) {
            int _e2072 = uniforms_2.isLinearRaw;
            if (_e2072 == 1) {
                float _e2078 = sampledColor.x;
                if (!((_e2078 >= 1.0))) {
                    float _e2085 = sampledColor.y;
                    local_53 = _e2085 >= 1.0;
                } else {
                    local_53 = true;
                }
                bool _e2089 = local_53;
                if (!(_e2089)) {
                    float _e2094 = sampledColor.z;
                    local_54 = _e2094 >= 1.0;
                } else {
                    local_54 = true;
                }
                bool _e2098 = local_54;
                local_52 = _e2098;
            } else {
                local_52 = false;
            }
            bool _e2100 = local_52;
            if (!(_e2100)) {
                float _e2105 = finalColor.x;
                float _e2107 = finalColor.y;
                float _e2109 = finalColor.z;
                local_55 = metal::max(_e2105, metal::max(_e2107, _e2109)) >= (headroom * 0.99);
            } else {
                local_55 = true;
            }
            bool _e2116 = local_55;
            if (!(_e2116)) {
                float _e2121 = rec2020Color.x;
                float _e2123 = rec2020Color.y;
                float _e2125 = rec2020Color.z;
                local_56 = metal::max(_e2121, metal::max(_e2123, _e2125)) >= 2.88;
            } else {
                local_56 = true;
            }
            bool isHighlightClip = local_56;
            metal::float3 _e2132 = rec2020Color;
            float sceneLum_1 = metal::dot(_e2132, metal::float3(0.2627, 0.678, 0.0593));
            if (!((sceneLum_1 <= 0.0007))) {
                float _e2144 = finalColor.x;
                float _e2146 = finalColor.y;
                float _e2148 = finalColor.z;
                local_57 = metal::max(_e2144, metal::max(_e2146, _e2148)) <= 0.0005;
            } else {
                local_57 = true;
            }
            bool isShadowClip = local_57;
            if (isHighlightClip) {
                float zebraHighlight = metal::fract((in.position.x + in.position.y) / 14.0);
                if (zebraHighlight < 0.5) {
                    float _e2166 = sampledColor.w;
                    return fragmentShaderOutput { metal::float4(1.0, 0.12, 0.12, _e2166) };
                }
            } else {
                if (isShadowClip) {
                    float zebraShadow = metal::fract((in.position.x - in.position.y) / 14.0);
                    if (zebraShadow < 0.5) {
                        float _e2182 = sampledColor.w;
                        return fragmentShaderOutput { metal::float4(0.05, 0.4, 1.0, _e2182) };
                    }
                }
            }
        }
        metal::float3 _e2187 = finalColor;
        float _e2189 = sampledColor.w;
        return fragmentShaderOutput { metal::float4(_e2187, _e2189) };
    } else {
        metal::float3 _e2191 = finalColor;
        metal::float3 _e2197 = linearToSrgbGamma3_(metal::clamp(_e2191, metal::float3(0.0), metal::float3(1.0)));
        int _e2200 = uniforms_2.clippingOverlayActive;
        if (_e2200 != 0) {
            float _e2207 = uniforms_2.isOriginal;
            local_58 = _e2207 < 0.5;
        } else {
            local_58 = false;
        }
        bool _e2211 = local_58;
        if (_e2211) {
            int _e2214 = uniforms_2.isLinearRaw;
            if (_e2214 == 1) {
                float _e2220 = sampledColor.x;
                if (!((_e2220 >= 1.0))) {
                    float _e2227 = sampledColor.y;
                    local_60 = _e2227 >= 1.0;
                } else {
                    local_60 = true;
                }
                bool _e2231 = local_60;
                if (!(_e2231)) {
                    float _e2236 = sampledColor.z;
                    local_61 = _e2236 >= 1.0;
                } else {
                    local_61 = true;
                }
                bool _e2240 = local_61;
                local_59 = _e2240;
            } else {
                local_59 = false;
            }
            bool _e2242 = local_59;
            if (!(_e2242)) {
                local_62 = metal::max(_e2197.x, metal::max(_e2197.y, _e2197.z)) >= 0.992;
            } else {
                local_62 = true;
            }
            bool _e2254 = local_62;
            if (!(_e2254)) {
                float _e2259 = rec2020Color.x;
                float _e2261 = rec2020Color.y;
                float _e2263 = rec2020Color.z;
                local_63 = metal::max(_e2259, metal::max(_e2261, _e2263)) >= 2.88;
            } else {
                local_63 = true;
            }
            bool isHighlightClip_1 = local_63;
            metal::float3 _e2270 = rec2020Color;
            float sceneLum_2 = metal::dot(_e2270, metal::float3(0.2627, 0.678, 0.0593));
            if (!((sceneLum_2 <= 0.0007))) {
                local_64 = metal::max(_e2197.x, metal::max(_e2197.y, _e2197.z)) <= 0.004;
            } else {
                local_64 = true;
            }
            bool isShadowClip_1 = local_64;
            if (isHighlightClip_1) {
                float zebraHighlight_1 = metal::fract((in.position.x + in.position.y) / 14.0);
                if (zebraHighlight_1 < 0.5) {
                    float _e2301 = sampledColor.w;
                    return fragmentShaderOutput { metal::float4(1.0, 0.12, 0.12, _e2301) };
                }
            } else {
                if (isShadowClip_1) {
                    float zebraShadow_1 = metal::fract((in.position.x - in.position.y) / 14.0);
                    if (zebraShadow_1 < 0.5) {
                        float _e2317 = sampledColor.w;
                        return fragmentShaderOutput { metal::float4(0.05, 0.4, 1.0, _e2317) };
                    }
                }
            }
        }
        float _e2324 = ign(in.position.xy);
        float dither = (_e2324 - 0.5) / 255.0;
        float _e2337 = sampledColor.w;
        return fragmentShaderOutput { metal::float4(metal::clamp(_e2197 + metal::float3(dither), metal::float3(0.0), metal::float3(1.0)), _e2337) };
    }
}
