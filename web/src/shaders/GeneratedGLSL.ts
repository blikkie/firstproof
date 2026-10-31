// AUTO-GENERATED from canonical WGSL by firstproof-engine (Story 4.8)
// DO NOT EDIT DIRECTLY. Edit firstproof-engine/shaders/*.wgsl instead.

export const FRAGMENT_SHADER_SOURCE = `#version 300 es

precision highp float;
precision highp int;

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
    int vignettingCorrectionEnabled;
    int hasVignetting;
    float vignettingK1_;
    float vignettingK2_;
    float vignettingK3_;
    float vignettingK4_;
    int softProofEnabled;
    int gamutWarningEnabled;
    int proofIntent;
    float pad0_;
    float pad1_;
    float pad2_;
};
struct VertexOutput {
    vec4 position;
    vec2 texCoord;
};
struct VertexInput {
    vec2 position;
    vec2 texCoord;
};
struct VertexQuad {
    VertexInput vertices[4];
};
struct GeometryCoordinates {
    vec2 uv;
    vec2 uvRed;
    vec2 uvBlue;
};
const mat3x3 LINEAR_SRGB_TO_REC2020_ = mat3x3(vec3(0.6274041, 0.0690972, 0.0163914), vec3(0.3292821, 0.9195411, 0.0880133), vec3(0.0433138, 0.0113617, 0.8955953));
const mat3x3 REC2020_TO_LINEAR_SRGB = mat3x3(vec3(1.660491, -0.1245505, -0.0181508), vec3(-0.5876411, 1.1328999, -0.1005789), vec3(-0.0728499, -0.0083494, 1.1187297));
const mat3x3 LINEAR_SRGB_TO_DISPLAY_P3_ = mat3x3(vec3(0.8224621, 0.0331941, 0.0170827), vec3(0.177538, 0.9668058, 0.0723974), vec3(0.0, 0.0, 0.9105199));
const mat3x3 AGX_INSET_MATRIX = mat3x3(vec3(1.4096986, 0.13136263, 0.16151477), vec3(-0.4004796, 0.77151406, -0.06034984), vec3(-0.009218898, 0.09712328, 0.89883506));
const mat3x3 AGX_OUTSET_MATRIX = mat3x3(vec3(1.1271005, -0.1413298, -0.1413298), vec3(-0.1106066, 1.1578237, -0.1106066), vec3(-0.0164939, -0.0164939, 1.2519364));
const float AGX_MIN_EV = -10.0;
const float AGX_MAX_EV = 6.5;
const float AGX_EV_RANGE = 16.5;
const float AGX_GAMMA = 2.2;
const float AGX_MID_GRAY_NORM = 0.6060606;
const vec2 DCP_KNOTS_STANDARD[7] = vec2[7](vec2(0.0, 0.0), vec2(0.02, 0.008), vec2(0.05, 0.028), vec2(0.18, 0.19), vec2(0.5, 0.54), vec2(0.8, 0.84), vec2(1.0, 1.0));
const float DCP_TANGENTS_STANDARD[7] = float[7](0.4, 0.5333, 0.9564, 1.17, 1.0469, 0.9, 0.8);
const vec2 DCP_KNOTS_PORTRAIT[7] = vec2[7](vec2(0.0, 0.0), vec2(0.02, 0.01), vec2(0.05, 0.035), vec2(0.18, 0.185), vec2(0.5, 0.51), vec2(0.8, 0.82), vec2(1.0, 1.0));
const float DCP_TANGENTS_PORTRAIT[7] = float[7](0.5, 0.6667, 0.9936, 1.0847, 1.0245, 0.9667, 0.9);
const vec2 DCP_KNOTS_LANDSCAPE[7] = vec2[7](vec2(0.0, 0.0), vec2(0.02, 0.006), vec2(0.05, 0.022), vec2(0.18, 0.2), vec2(0.5, 0.56), vec2(0.8, 0.86), vec2(1.0, 1.0));
const float DCP_TANGENTS_LANDSCAPE[7] = float[7](0.3, 0.4167, 0.9513, 1.2471, 1.0625, 0.85, 0.7);
const vec2 DCP_KNOTS_VIVID[7] = vec2[7](vec2(0.0, 0.0), vec2(0.02, 0.004), vec2(0.05, 0.018), vec2(0.18, 0.21), vec2(0.5, 0.58), vec2(0.8, 0.88), vec2(1.0, 1.0));
const float DCP_TANGENTS_VIVID[7] = float[7](0.2, 0.3333, 0.9718, 1.3166, 1.0781, 0.8, 0.6);
const vec3 DCP_HSM_STANDARD[24] = vec3[24](vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.03, 1.02), vec3(0.0, 1.05, 1.03), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(1.5, 1.06, 0.99), vec3(2.0, 1.1, 0.98), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0));
const vec3 DCP_HSM_PORTRAIT[24] = vec3[24](vec3(1.5, 0.96, 1.02), vec3(2.0, 0.94, 1.03), vec3(0.0, 0.96, 1.03), vec3(0.0, 0.94, 1.04), vec3(-1.5, 0.96, 1.02), vec3(-2.0, 0.94, 1.03), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0));
const vec3 DCP_HSM_LANDSCAPE[24] = vec3[24](vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(2.5, 1.15, 0.97), vec3(3.5, 1.22, 0.95), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.05, 1.0), vec3(0.0, 1.05, 1.0), vec3(0.0, 1.05, 1.0), vec3(0.0, 1.05, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0));
const vec3 DCP_HSM_VIVID[24] = vec3[24](vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(2.0, 1.25, 0.96), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02), vec3(0.0, 1.15, 1.02));
const vec2 FERMAT_DISK[16] = vec2[16](vec2(0.1768, 0.0), vec2(-0.2258, 0.2068), vec2(0.0346, -0.3938), vec2(0.2846, 0.3712), vec2(-0.5222, -0.0924), vec2(0.4947, -0.3147), vec2(-0.1655, 0.6155), vec2(-0.3156, -0.6076), vec2(0.6846, 0.25), vec2(-0.7123, 0.294), vec2(0.3434, -0.7337), vec2(0.2537, 0.8089), vec2(-0.7647, -0.4432), vec2(0.8971, -0.1972), vec2(-0.5475, 0.7788), vec2(-0.1265, -0.9761));
const float FERMAT_WEIGHTS[16] = float[16](0.6428, 0.4651, 0.3722, 0.3106, 0.2656, 0.2309, 0.2032, 0.1806, 0.1617, 0.1457, 0.132, 0.1201, 0.1097, 0.1006, 0.0926, 0.0854);

layout(std140) uniform ShaderUniforms_block_0Fragment { ShaderUniforms _group_0_binding_0_fs; };

uniform highp sampler2D _group_0_binding_1_fs;

uniform highp sampler3D _group_0_binding_3_fs;

uniform highp sampler2D _group_0_binding_5_fs;

uniform highp sampler2D _group_0_binding_6_fs;

uniform highp sampler2D _group_0_binding_8_fs;

uniform highp sampler3D _group_0_binding_11_fs;

smooth in vec2 _vs2fs_location0;
layout(location = 0) out vec4 _fs2p_location0;

GeometryCoordinates transformGeometryChannels(vec2 uv, ShaderUniforms uniforms) {
    GeometryCoordinates coords = GeometryCoordinates(vec2(0.0), vec2(0.0), vec2(0.0));
    bool local_70 = false;
    vec2 uvc = vec2(0.0);
    float autoCropScale = 1.0;
    bool local_71 = false;
    int i_2 = 0;
    vec2 g = vec2(0.0);
    float r_corr = 0.0;
    vec2 centered = vec2(0.0);
    vec2 geom = vec2(0.0);
    float r_corr_1 = 0.0;
    vec2 geom_r = vec2(0.0);
    vec2 geom_b = vec2(0.0);
    bool local_72 = false;
    if (!((uniforms.isOriginal >= 0.5))) {
        local_70 = (uniforms.isIntermediate == 1);
    } else {
        local_70 = true;
    }
    bool _e13 = local_70;
    if (_e13) {
        coords.uv = uv;
        coords.uvRed = uv;
        coords.uvBlue = uv;
        GeometryCoordinates _e17 = coords;
        return _e17;
    }
    float cropL = clamp(uniforms.cropLeft, 0.0, 0.99);
    float cropT = clamp(uniforms.cropTop, 0.0, 0.99);
    float cropR = clamp(uniforms.cropRight, (cropL + 0.01), 1.0);
    float cropB = clamp(uniforms.cropBottom, (cropT + 0.01), 1.0);
    uvc = uv;
    if ((uniforms.cropModeActive == 0)) {
        uvc.x = (cropL + (uv.x * (cropR - cropL)));
        uvc.y = (cropT + (uv.y * (cropB - cropT)));
    }
    float A = max(uniforms.aspectRatio, 0.01);
    float angle = uniforms.straightenAngle;
    float cosA = cos(angle);
    float sinA = sin(angle);
    float cV = (uniforms.perspectiveVertical * 0.005);
    float cH = (uniforms.perspectiveHorizontal * 0.005);
    if (!((uniforms.cropConstrain != 0))) {
        local_71 = (uniforms.lensCorrectionEnabled != 0);
    } else {
        local_71 = true;
    }
    bool _e74 = local_71;
    if (_e74) {
        vec2 testPoints[12] = vec2[12](vec2(A, 1.0), vec2(-(A), 1.0), vec2(A, -1.0), vec2(-(A), -1.0), vec2(A, 0.0), vec2(-(A), 0.0), vec2(0.0, 1.0), vec2(0.0, -1.0), vec2(A, 0.5), vec2(-(A), 0.5), vec2(A, -0.5), vec2(-(A), -0.5));
        bool loop_init = true;
        while(true) {
            if (!loop_init) {
                int _e192 = i_2;
                i_2 = (_e192 + 1);
            }
            loop_init = false;
            int _e109 = i_2;
            if ((_e109 < 12)) {
            } else {
                break;
            }
            {
                int _e112 = i_2;
                vec2 p_3 = testPoints[_e112];
                vec2 r = vec2(((p_3.x * cosA) - (p_3.y * sinA)), ((p_3.x * sinA) + (p_3.y * cosA)));
                float denom = max(((1.0 + (cV * r.y)) + (cH * r.x)), 0.1);
                g = (r / vec2(denom));
                if ((uniforms.lensCorrectionEnabled != 0)) {
                    vec2 _e140 = g;
                    vec2 _e141 = g;
                    float r2_ = dot(_e140, _e141);
                    float rad = sqrt(r2_);
                    if ((rad > 0.0001)) {
                        r_corr = rad;
                        if ((uniforms.distortionModel == 0)) {
                            r_corr = (rad * ((1.0 + (uniforms.k1_ * r2_)) + (uniforms.k2_ * (r2_ * r2_))));
                        } else {
                            if ((uniforms.distortionModel == 1)) {
                                float a = uniforms.k1_;
                                float b = uniforms.k2_;
                                float c_9 = uniforms.k3_;
                                float d = (1.0 - ((a + b) + c_9));
                                r_corr = ((((a * (r2_ * r2_)) + (b * (r2_ * rad))) + (c_9 * r2_)) + (d * rad));
                            }
                        }
                        vec2 _e178 = g;
                        float _e179 = r_corr;
                        g = (_e178 * (_e179 / rad));
                    }
                }
                float _e183 = g.x;
                float _e187 = g.y;
                float req = max((abs(_e183) / A), abs(_e187));
                float _e190 = autoCropScale;
                if ((req > _e190)) {
                    autoCropScale = req;
                }
            }
        }
    }
    vec2 _e195 = uvc;
    centered = ((_e195 - vec2(0.5, 0.5)) * 2.0);
    float _e205 = centered.x;
    centered.x = (_e205 * A);
    vec2 _e207 = centered;
    float _e208 = autoCropScale;
    centered = (_e207 / vec2(_e208));
    float _e212 = centered.x;
    float _e215 = centered.y;
    float _e219 = centered.x;
    float _e222 = centered.y;
    vec2 rot = vec2(((_e212 * cosA) - (_e215 * sinA)), ((_e219 * sinA) + (_e222 * cosA)));
    float denom_1 = max(((1.0 + (cV * rot.y)) + (cH * rot.x)), 0.1);
    geom = (rot / vec2(denom_1));
    if ((uniforms.lensCorrectionEnabled != 0)) {
        vec2 _e241 = geom;
        vec2 _e242 = geom;
        float r2_1 = dot(_e241, _e242);
        float r_1 = sqrt(r2_1);
        if ((r_1 > 0.0001)) {
            r_corr_1 = r_1;
            if ((uniforms.distortionModel == 0)) {
                r_corr_1 = (r_1 * ((1.0 + (uniforms.k1_ * r2_1)) + (uniforms.k2_ * (r2_1 * r2_1))));
            } else {
                if ((uniforms.distortionModel == 1)) {
                    float a_1 = uniforms.k1_;
                    float b_1 = uniforms.k2_;
                    float c_10 = uniforms.k3_;
                    float d_1 = (1.0 - ((a_1 + b_1) + c_10));
                    r_corr_1 = ((((a_1 * (r2_1 * r2_1)) + (b_1 * (r2_1 * r_1))) + (c_10 * r2_1)) + (d_1 * r_1));
                }
            }
            vec2 _e279 = geom;
            float _e280 = r_corr_1;
            geom = (_e279 * (_e280 / r_1));
        }
    }
    vec2 _e283 = geom;
    geom_r = _e283;
    vec2 _e285 = geom;
    geom_b = _e285;
    if ((uniforms.tcaCorrectionEnabled != 0)) {
        local_72 = (uniforms.hasTca != 0);
    } else {
        local_72 = false;
    }
    bool _e296 = local_72;
    if (_e296) {
        vec2 _e297 = geom;
        vec2 _e298 = geom;
        float r2_2 = dot(_e297, _e298);
        float r4_ = (r2_2 * r2_2);
        float scale_r = ((uniforms.tcaVr + (uniforms.tcaBr * r2_2)) + (uniforms.tcaCr * r4_));
        float scale_b = ((uniforms.tcaVb + (uniforms.tcaBb * r2_2)) + (uniforms.tcaCb * r4_));
        vec2 _e315 = geom;
        geom_r = (_e315 * scale_r);
        vec2 _e317 = geom;
        geom_b = (_e317 * scale_b);
    }
    float denomA = (2.0 * A);
    float _e323 = geom.x;
    float _e328 = geom.y;
    coords.uv = vec2(((_e323 / denomA) + 0.5), ((_e328 * 0.5) + 0.5));
    float _e336 = geom_r.x;
    float _e341 = geom_r.y;
    coords.uvRed = vec2(((_e336 / denomA) + 0.5), ((_e341 * 0.5) + 0.5));
    float _e349 = geom_b.x;
    float _e354 = geom_b.y;
    coords.uvBlue = vec2(((_e349 / denomA) + 0.5), ((_e354 * 0.5) + 0.5));
    GeometryCoordinates _e360 = coords;
    return _e360;
}

vec2 transformGeometry(vec2 uv_1, ShaderUniforms uniforms_1) {
    GeometryCoordinates _e2 = transformGeometryChannels(uv_1, uniforms_1);
    return _e2.uv;
}

vec3 applyDetailAndSharpening(vec3 color, float y_c, float y_l_1, float y_r_1, float y_t_1, float y_b_1, float y_tl_1, float y_tr_1, float y_bl_1, float y_br_1, float amount, float masking, float noiseReduction) {
    bool local_73 = false;
    float w_edge = 1.0;
    if ((amount <= 0.0)) {
        local_73 = (noiseReduction <= 0.0);
    } else {
        local_73 = false;
    }
    bool _e20 = local_73;
    if (_e20) {
        return color;
    }
    float y_neighbors_2 = ((0.16666667 * (((y_l_1 + y_r_1) + y_t_1) + y_b_1)) + (0.083333336 * (((y_tl_1 + y_tr_1) + y_bl_1) + y_br_1)));
    float delta_y = (y_c - y_neighbors_2);
    float gx = ((((y_tr_1 + (2.0 * y_r_1)) + y_br_1) - ((y_tl_1 + (2.0 * y_l_1)) + y_bl_1)) * 0.125);
    float gy = ((((y_bl_1 + (2.0 * y_b_1)) + y_br_1) - ((y_tl_1 + (2.0 * y_t_1)) + y_tr_1)) * 0.125);
    float grad_mag = sqrt(((gx * gx) + (gy * gy)));
    if ((masking > 0.0)) {
        float m = clamp((masking / 100.0), 0.0, 1.0);
        float threshold = (pow(m, 1.5) * 0.08);
        w_edge = smoothstep(0.0, max(threshold, 0.0001), grad_mag);
    }
    float sharp_gain = ((amount / 100.0) * 1.5);
    float _e80 = w_edge;
    float sharp_term = ((sharp_gain * _e80) * delta_y);
    float _e85 = w_edge;
    float nr_term = (((noiseReduction / 100.0) * (1.0 - _e85)) * delta_y);
    float delta_adj = max((sharp_term - nr_term), (-0.75 * y_c));
    return max((color + vec3(delta_adj)), vec3(0.0));
}

vec3 applyDefringe(vec3 color_1, float y_c_1, float y_max_neighbor, float defringeAmount) {
    bool local_74 = false;
    float hue = 0.0;
    if ((defringeAmount <= 0.0)) {
        return color_1;
    }
    float excess_b = (color_1.z - color_1.y);
    float excess_r = (color_1.x - color_1.y);
    if (!((excess_b <= 0.0))) {
        local_74 = (excess_r <= 0.0);
    } else {
        local_74 = true;
    }
    bool _e20 = local_74;
    if (_e20) {
        return color_1;
    }
    float u_1 = (((2.0 * color_1.x) - color_1.y) - color_1.z);
    float v_1 = (1.7320508 * (color_1.y - color_1.z));
    hue = (atan(v_1, u_1) * 57.29578);
    float _e37 = hue;
    if ((_e37 < 0.0)) {
        float _e40 = hue;
        hue = (_e40 + 360.0);
    }
    float _e43 = hue;
    float _e47 = hue;
    float w_hue = (smoothstep(250.0, 270.0, _e43) * (1.0 - smoothstep(330.0, 350.0, _e47)));
    if ((w_hue <= 0.001)) {
        return color_1;
    }
    float edge_contrast = (y_max_neighbor - y_c_1);
    float w_edge_1 = smoothstep(0.04, 0.25, edge_contrast);
    float w_highlight = smoothstep(0.04, 0.25, y_max_neighbor);
    float w_fringe = (((w_hue * w_edge_1) * w_highlight) * (defringeAmount / 100.0));
    if ((w_fringe <= 0.001)) {
        return color_1;
    }
    float r_adj = (color_1.x - (w_fringe * excess_r));
    float b_adj = (color_1.z - (w_fringe * excess_b));
    return vec3(r_adj, color_1.y, b_adj);
}

float proceduralGrainNoise(float u, float v) {
    vec2 p_4 = (50.0 * fract(((vec2(u, v) * 0.3183099) + vec2(0.71, 0.113))));
    return (-1.0 + (2.0 * fract(((p_4.x * p_4.y) * (p_4.x + p_4.y)))));
}

float clusteredGrainNoise(vec2 p) {
    vec2 i_7 = floor(p);
    vec2 f = fract(p);
    vec2 u_2 = ((f * f) * (vec2(3.0) - (2.0 * f)));
    float _e12 = proceduralGrainNoise(i_7.x, i_7.y);
    float _e17 = proceduralGrainNoise((i_7.x + 1.0), i_7.y);
    float _e22 = proceduralGrainNoise(i_7.x, (i_7.y + 1.0));
    float _e29 = proceduralGrainNoise((i_7.x + 1.0), (i_7.y + 1.0));
    return mix(mix(_e12, _e17, u_2.x), mix(_e22, _e29, u_2.x), u_2.y);
}

float evalGrainProcNoise(vec2 pos, float s, bool is_instant) {
    if (is_instant) {
        float _e5 = clusteredGrainNoise((pos / vec2(s)));
        float _e16 = clusteredGrainNoise(((pos / vec2((s * 1.8))) + vec2(17.3, 31.7)));
        return ((0.6 * _e5) + (0.4 * _e16));
    } else {
        if ((s > 1.0)) {
            float t_cluster = clamp(((s - 1.0) / 2.0), 0.0, 1.0);
            float _e31 = clusteredGrainNoise((pos / vec2(s)));
            float _e34 = proceduralGrainNoise(pos.x, pos.y);
            float clumped = ((_e34 * (0.6 + (0.4 * _e31))) + (0.25 * _e31));
            return mix(_e34, clumped, t_cluster);
        } else {
            float _e46 = proceduralGrainNoise(pos.x, pos.y);
            return _e46;
        }
    }
}

float evalGrainEta(float proc_val, float sensor_seed, int grain_mode, bool is_instant_1) {
    if ((grain_mode == 1)) {
        if (is_instant_1) {
            return clamp(((0.55 * proc_val) + (0.2 * sensor_seed)), -1.0, 1.0);
        } else {
            return clamp(((proc_val * (0.45 + (0.55 * abs(sensor_seed)))) + (0.45 * sensor_seed)), -1.0, 1.0);
        }
    } else {
        if ((grain_mode == 2)) {
            if (is_instant_1) {
                return clamp(((0.85 * proc_val) + (0.05 * sensor_seed)), -1.0, 1.0);
            } else {
                return clamp(((0.9 * proc_val) + (0.1 * sensor_seed)), -1.0, 1.0);
            }
        }
    }
    return 0.0;
}

vec3 applyStochasticEmulsionGrain(vec3 color_2, float y_c_2, float y_l_2, float y_r_2, float y_t_2, float y_b_2, float y_tl_2, float y_tr_2, float y_bl_2, float y_br_2, vec2 p_1, int grain_mode_1, float cluster_size, float grain_amount, int archetype) {
    bool local_75 = false;
    bool local_76 = false;
    if ((grain_mode_1 == 0)) {
        return color_2;
    }
    float amount_1 = clamp(grain_amount, 0.0, 2.0);
    if ((amount_1 < 0.001)) {
        return color_2;
    }
    float y_lowpass = ((0.16666667 * (((y_l_2 + y_r_2) + y_t_2) + y_b_2)) + (0.083333336 * (((y_tl_2 + y_tr_2) + y_bl_2) + y_br_2)));
    float delta_sensor = (y_c_2 - y_lowpass);
    float gx_1 = ((((y_tr_2 + (2.0 * y_r_2)) + y_br_2) - ((y_tl_2 + (2.0 * y_l_2)) + y_bl_2)) * 0.125);
    float gy_1 = ((((y_bl_2 + (2.0 * y_b_2)) + y_br_2) - ((y_tl_2 + (2.0 * y_t_2)) + y_tr_2)) * 0.125);
    float grad_mag_1 = sqrt(((gx_1 * gx_1) + (gy_1 * gy_1)));
    float edge_suppress = (1.0 / (1.0 + (35.0 * grad_mag_1)));
    float delta_noise = (delta_sensor * edge_suppress);
    float denom_2 = max(sqrt(max(y_c_2, 0.0)), 0.05);
    float sensor_seed_1 = clamp(((delta_noise / denom_2) * 20.0), -1.0, 1.0);
    bool is_instant_2 = (archetype == 7);
    float s_2 = max(cluster_size, 0.25);
    float _e82 = evalGrainProcNoise(p_1, s_2, is_instant_2);
    float _e83 = evalGrainEta(_e82, sensor_seed_1, grain_mode_1, is_instant_2);
    float y_clamped = clamp(y_c_2, 0.0, 1.0);
    float nutting = ((4.0 * y_clamped) * (1.0 - y_clamped));
    float y_pixel = dot(color_2, vec3(0.2627, 0.678, 0.0593));
    if (!((archetype == 6))) {
        local_75 = (archetype == 10);
    } else {
        local_75 = true;
    }
    bool _e105 = local_75;
    if (!(_e105)) {
        local_76 = (archetype == 11);
    } else {
        local_76 = true;
    }
    bool is_monochrome = local_76;
    if (is_monochrome) {
        float delta_1 = (((_e83 * nutting) * 0.28) * amount_1);
        float y_res = max((y_pixel * (1.0 + delta_1)), 0.0);
        return vec3(y_res);
    } else {
        float _e127 = evalGrainProcNoise((p_1 + vec2(17.3, 31.7)), s_2, is_instant_2);
        float _e132 = evalGrainProcNoise((p_1 + vec2(73.1, 19.9)), s_2, is_instant_2);
        float _e137 = evalGrainProcNoise((p_1 + vec2(43.7, 89.3)), s_2, is_instant_2);
        float _e138 = evalGrainEta(_e127, sensor_seed_1, grain_mode_1, is_instant_2);
        float _e139 = evalGrainEta(_e132, sensor_seed_1, grain_mode_1, is_instant_2);
        float _e140 = evalGrainEta(_e137, sensor_seed_1, grain_mode_1, is_instant_2);
        float eta_final_r = (mix(_e83, _e138, 0.25) * 1.264911);
        float eta_final_g = (mix(_e83, _e139, 0.25) * 1.264911);
        float eta_final_b = (mix(_e83, _e140, 0.25) * 1.264911);
        float scale_1 = ((nutting * 0.28) * amount_1);
        float delta_r = (eta_final_r * scale_1);
        float delta_g = (eta_final_g * scale_1);
        float delta_b = (eta_final_b * scale_1);
        float chroma_damp = (1.0 - (0.1 * nutting));
        float r_c = (y_pixel + (chroma_damp * (color_2.x - y_pixel)));
        float g_c = (y_pixel + (chroma_damp * (color_2.y - y_pixel)));
        float b_c = (y_pixel + (chroma_damp * (color_2.z - y_pixel)));
        float r_res = max((r_c * (1.0 + delta_r)), 0.0);
        float g_res = max((g_c * (1.0 + delta_g)), 0.0);
        float b_res = max((b_c * (1.0 + delta_b)), 0.0);
        return vec3(r_res, g_res, b_res);
    }
}

float float_mod(float x, float y) {
    return (x - (y * floor((x / y))));
}

float srgbGammaToLinear(float c_2) {
    float cl = clamp(c_2, 0.0, 1.0);
    if ((cl <= 0.04045)) {
        return (cl / 12.92);
    } else {
        return pow(((cl + 0.055) / 1.055), 2.4);
    }
}

vec3 srgbGammaToLinear3_(vec3 c_3) {
    float _e2 = srgbGammaToLinear(c_3.x);
    float _e4 = srgbGammaToLinear(c_3.y);
    float _e6 = srgbGammaToLinear(c_3.z);
    return vec3(_e2, _e4, _e6);
}

float linearToSrgbGamma(float c_4) {
    float cl_1 = clamp(c_4, 0.0, 1.0);
    if ((cl_1 <= 0.0031308)) {
        return (12.92 * cl_1);
    } else {
        return ((1.055 * pow(cl_1, 0.41666666)) - 0.055);
    }
}

vec3 linearToSrgbGamma3_(vec3 c_5) {
    float _e2 = linearToSrgbGamma(c_5.x);
    float _e4 = linearToSrgbGamma(c_5.y);
    float _e6 = linearToSrgbGamma(c_5.z);
    return vec3(_e2, _e4, _e6);
}

vec3 rgb2hsv(vec3 c_6) {
    float h = 0.0;
    float s_1 = 0.0;
    float maxV = max(c_6.x, max(c_6.y, c_6.z));
    float minV = min(c_6.x, min(c_6.y, c_6.z));
    float delta_2 = (maxV - minV);
    if ((delta_2 > 1e-5)) {
        if ((maxV == c_6.x)) {
            h = ((c_6.y - c_6.z) / delta_2);
            float _e22 = h;
            if ((_e22 < 0.0)) {
                float _e25 = h;
                h = (_e25 + 6.0);
            }
        } else {
            if ((maxV == c_6.y)) {
                h = (((c_6.z - c_6.x) / delta_2) + 2.0);
            } else {
                h = (((c_6.x - c_6.y) / delta_2) + 4.0);
            }
        }
        float _e42 = h;
        h = (_e42 * 60.0);
    }
    if ((maxV > 1e-5)) {
        s_1 = (delta_2 / maxV);
    }
    float _e50 = h;
    float _e51 = s_1;
    return vec3(_e50, _e51, maxV);
}

vec3 hsv2rgb(vec3 c_7) {
    vec3 rgb = vec3(0.0);
    float h_1 = (c_7.x / 60.0);
    float s_3 = c_7.y;
    float v_2 = c_7.z;
    float C = (v_2 * s_3);
    float _e8 = float_mod(h_1, 2.0);
    float X = (C * (1.0 - abs((_e8 - 1.0))));
    float m_1 = (v_2 - C);
    if ((h_1 < 1.0)) {
        rgb = vec3(C, X, 0.0);
    } else {
        if ((h_1 < 2.0)) {
            rgb = vec3(X, C, 0.0);
        } else {
            if ((h_1 < 3.0)) {
                rgb = vec3(0.0, C, X);
            } else {
                if ((h_1 < 4.0)) {
                    rgb = vec3(0.0, X, C);
                } else {
                    if ((h_1 < 5.0)) {
                        rgb = vec3(X, 0.0, C);
                    } else {
                        rgb = vec3(C, 0.0, X);
                    }
                }
            }
        }
    }
    vec3 _e41 = rgb;
    return (_e41 + vec3(m_1));
}

float agxApplyCurveChannel(float x_1) {
    float xc = clamp(x_1, 0.0, 1.0);
    float curved = (((((((((((15.45672 * xc) - 40.02792) * xc) + 31.87076) * xc) - 6.84882) * xc) + 0.4286) * xc) + 0.12066) * xc);
    return clamp(curved, 0.0, 1.0);
}

vec3 agxToneMap(vec3 rec2020_color, float contrast_gain) {
    vec3 cComp = vec3(0.0);
    vec3 norm = vec3(0.0);
    float minC = min(rec2020_color.x, min(rec2020_color.y, rec2020_color.z));
    cComp = rec2020_color;
    if ((minC < 0.0)) {
        cComp = (rec2020_color - vec3(minC));
    }
    vec3 _e13 = cComp;
    vec3 inset = (AGX_INSET_MATRIX * _e13);
    vec3 xRel = max((inset / vec3(0.18)), vec3(1e-6));
    vec3 logEncoded = ((log2(xRel) - vec3(-10.0)) / vec3(16.5));
    norm = clamp(logEncoded, vec3(0.0), vec3(1.0));
    if ((abs((contrast_gain - 1.0)) > 0.001)) {
        vec3 _e41 = norm;
        norm = clamp((vec3(0.6060606) + ((_e41 - vec3(0.6060606)) * contrast_gain)), vec3(0.0), vec3(1.0));
    }
    float _e53 = norm.x;
    float _e54 = agxApplyCurveChannel(_e53);
    float _e56 = norm.y;
    float _e57 = agxApplyCurveChannel(_e56);
    float _e59 = norm.z;
    float _e60 = agxApplyCurveChannel(_e59);
    vec3 curved_1 = vec3(_e54, _e57, _e60);
    vec3 linearized = pow(max(curved_1, vec3(0.0)), vec3(2.2));
    return max((AGX_OUTSET_MATRIX * linearized), vec3(0.0));
}

vec3 agxToneMapVivid(vec3 rec2020_color_1, float contrast_gain_1) {
    float punchyContrast = (contrast_gain_1 * 1.35);
    vec3 _e4 = agxToneMap(rec2020_color_1, punchyContrast);
    float lum = (((0.2126 * _e4.x) + (0.7152 * _e4.y)) + (0.0722 * _e4.z));
    float t = clamp(((lum - 0.65) / 0.35), 0.0, 1.0);
    float h_2 = ((t * t) * (3.0 - (2.0 * t)));
    float sat = (1.25 - (0.25 * h_2));
    return clamp(vec3((lum + (sat * (_e4.x - lum))), (lum + (sat * (_e4.y - lum))), (lum + (sat * (_e4.z - lum)))), vec3(0.0), vec3(1.0));
}

vec3 applyBilateralToneRecovery(vec3 color_3, float y_c_3, float y_neighbors_1[16], float highlights, float shadows, float whites, float blacks) {
    bool local_77 = false;
    bool local_78 = false;
    bool local_79 = false;
    float total_weight = 1.0;
    float weighted_ev_sum = 0.0;
    int i_3 = 0;
    int i_4 = 4;
    int i_5 = 8;
    int i_6 = 12;
    float delta_ev = 0.0;
    if ((highlights == 0.0)) {
        local_77 = (shadows == 0.0);
    } else {
        local_77 = false;
    }
    bool _e14 = local_77;
    if (_e14) {
        local_78 = (whites == 0.0);
    } else {
        local_78 = false;
    }
    bool _e20 = local_78;
    if (_e20) {
        local_79 = (blacks == 0.0);
    } else {
        local_79 = false;
    }
    bool _e26 = local_79;
    if (_e26) {
        return color_3;
    }
    if ((y_c_3 <= 1e-6)) {
        return color_3;
    }
    float ev_c = log2((max(y_c_3, 1e-5) / 0.18));
    weighted_ev_sum = ev_c;
    bool loop_init_1 = true;
    while(true) {
        if (!loop_init_1) {
            int _e63 = i_3;
            i_3 = (_e63 + 1);
        }
        loop_init_1 = false;
        int _e42 = i_3;
        if ((_e42 < 4)) {
        } else {
            break;
        }
        {
            int _e45 = i_3;
            float ev_i = log2((max(y_neighbors_1[_e45], 1e-5) / 0.18));
            float d_ev = (ev_i - ev_c);
            float w_r = exp((-((d_ev * d_ev)) * 0.125));
            float w = (0.60653067 * w_r);
            float _e58 = total_weight;
            total_weight = (_e58 + w);
            float _e60 = weighted_ev_sum;
            weighted_ev_sum = (_e60 + (w * ev_i));
        }
    }
    bool loop_init_2 = true;
    while(true) {
        if (!loop_init_2) {
            int _e89 = i_4;
            i_4 = (_e89 + 1);
        }
        loop_init_2 = false;
        int _e68 = i_4;
        if ((_e68 < 8)) {
        } else {
            break;
        }
        {
            int _e71 = i_4;
            float ev_i_1 = log2((max(y_neighbors_1[_e71], 1e-5) / 0.18));
            float d_ev_1 = (ev_i_1 - ev_c);
            float w_r_1 = exp((-((d_ev_1 * d_ev_1)) * 0.125));
            float w_1 = (0.36787945 * w_r_1);
            float _e84 = total_weight;
            total_weight = (_e84 + w_1);
            float _e86 = weighted_ev_sum;
            weighted_ev_sum = (_e86 + (w_1 * ev_i_1));
        }
    }
    bool loop_init_3 = true;
    while(true) {
        if (!loop_init_3) {
            int _e115 = i_5;
            i_5 = (_e115 + 1);
        }
        loop_init_3 = false;
        int _e94 = i_5;
        if ((_e94 < 12)) {
        } else {
            break;
        }
        {
            int _e97 = i_5;
            float ev_i_2 = log2((max(y_neighbors_1[_e97], 1e-5) / 0.18));
            float d_ev_2 = (ev_i_2 - ev_c);
            float w_r_2 = exp((-((d_ev_2 * d_ev_2)) * 0.125));
            float w_2 = (0.60653067 * w_r_2);
            float _e110 = total_weight;
            total_weight = (_e110 + w_2);
            float _e112 = weighted_ev_sum;
            weighted_ev_sum = (_e112 + (w_2 * ev_i_2));
        }
    }
    bool loop_init_4 = true;
    while(true) {
        if (!loop_init_4) {
            int _e141 = i_6;
            i_6 = (_e141 + 1);
        }
        loop_init_4 = false;
        int _e120 = i_6;
        if ((_e120 < 16)) {
        } else {
            break;
        }
        {
            int _e123 = i_6;
            float ev_i_3 = log2((max(y_neighbors_1[_e123], 1e-5) / 0.18));
            float d_ev_3 = (ev_i_3 - ev_c);
            float w_r_3 = exp((-((d_ev_3 * d_ev_3)) * 0.125));
            float w_3 = (0.36787945 * w_r_3);
            float _e136 = total_weight;
            total_weight = (_e136 + w_3);
            float _e138 = weighted_ev_sum;
            weighted_ev_sum = (_e138 + (w_3 * ev_i_3));
        }
    }
    float _e144 = weighted_ev_sum;
    float _e145 = total_weight;
    float ev_base = (_e144 / _e145);
    if ((blacks != 0.0)) {
        float u_3 = clamp(((-(ev_base) - 1.0) / 3.0), 0.0, 1.0);
        float w_blacks = ((u_3 * u_3) * (3.0 - (2.0 * u_3)));
        float _e165 = delta_ev;
        delta_ev = (_e165 + (((blacks / 100.0) * 1.2) * w_blacks));
    }
    if ((shadows != 0.0)) {
        float u_4 = clamp((-(ev_base) / 3.0), 0.0, 1.0);
        float w_shadows = ((u_4 * u_4) * (3.0 - (2.0 * u_4)));
        float _e186 = delta_ev;
        delta_ev = (_e186 + (((shadows / 100.0) * 2.25) * w_shadows));
    }
    if ((highlights != 0.0)) {
        float u_5 = clamp((ev_base / 3.0), 0.0, 1.0);
        float w_highlights = ((u_5 * u_5) * (3.0 - (2.0 * u_5)));
        float _e206 = delta_ev;
        delta_ev = (_e206 + (((highlights / 100.0) * 2.25) * w_highlights));
    }
    if ((whites != 0.0)) {
        float u_6 = clamp(((ev_base - 1.0) / 3.0), 0.0, 1.0);
        float w_whites = ((u_6 * u_6) * (3.0 - (2.0 * u_6)));
        float _e228 = delta_ev;
        delta_ev = (_e228 + (((whites / 100.0) * 1.2) * w_whites));
    }
    float _e235 = delta_ev;
    if ((_e235 == 0.0)) {
        return color_3;
    }
    float detail = clamp((ev_c - ev_base), -3.0, 3.0);
    float _e244 = delta_ev;
    float detail_boost = ((0.35 * detail) * (abs(_e244) / 2.25));
    float _e249 = delta_ev;
    float gain = exp2(clamp((_e249 + detail_boost), -6.0, 6.0));
    return (color_3 * gain);
}

vec3 applyColorAdjustments(vec3 rgb_1, float vibrance, float saturation, float deepBlue, float foliage) {
    bool local_80 = false;
    bool local_81 = false;
    bool local_82 = false;
    vec3 c_8 = vec3(0.0);
    float skinProtect = 1.0;
    float vibGain = 0.0;
    if ((vibrance == 0.0)) {
        local_80 = (saturation == 0.0);
    } else {
        local_80 = false;
    }
    bool _e12 = local_80;
    if (_e12) {
        local_81 = (deepBlue == 0.0);
    } else {
        local_81 = false;
    }
    bool _e18 = local_81;
    if (_e18) {
        local_82 = (foliage == 0.0);
    } else {
        local_82 = false;
    }
    bool _e24 = local_82;
    if (_e24) {
        return rgb_1;
    }
    c_8 = max(rgb_1, vec3(0.0));
    float _e30 = c_8.x;
    float _e34 = c_8.y;
    float _e39 = c_8.z;
    float lum_1 = (((0.2627 * _e30) + (0.678 * _e34)) + (0.0593 * _e39));
    if ((saturation != 0.0)) {
        float satGain = max((1.0 + (saturation / 100.0)), 0.0);
        vec3 _e52 = c_8;
        c_8 = (vec3(lum_1) + ((_e52 - vec3(lum_1)) * satGain));
    }
    if ((vibrance != 0.0)) {
        float _e60 = c_8.x;
        float _e62 = c_8.y;
        float _e65 = c_8.z;
        float maxC = max(max(_e60, _e62), _e65);
        float _e68 = c_8.x;
        float _e70 = c_8.y;
        float _e73 = c_8.z;
        float minC_1 = min(min(_e68, _e70), _e73);
        float chroma = (maxC - minC_1);
        float _e79 = c_8.x;
        float _e81 = c_8.y;
        float _e84 = c_8.z;
        float sum = ((_e79 + _e81) + _e84);
        if ((sum > 1e-5)) {
            float _e89 = c_8.x;
            float rN = (_e89 / sum);
            float _e92 = c_8.y;
            float gN = (_e92 / sum);
            float bN = ((1.0 - rN) - gN);
            float deltaCross = ((gN + (0.35 * rN)) - 0.478);
            float dCross = (deltaCross / 0.04);
            float dMel = ((rN - 0.48) / 0.16);
            float d2_ = ((dCross * dCross) + (dMel * dMel));
            float gauss = exp((-0.5 * d2_));
            float uLum = clamp(((sum - 0.005) / 0.025), 0.0, 1.0);
            float wLum = ((uLum * uLum) * (3.0 - (2.0 * uLum)));
            float uRg = clamp(((rN - gN) / 0.05), 0.0, 1.0);
            float wRg = ((uRg * uRg) * (3.0 - (2.0 * uRg)));
            float uRb = clamp(((rN - bN) / 0.08), 0.0, 1.0);
            float wRb = ((uRb * uRb) * (3.0 - (2.0 * uRb)));
            float uGb = clamp((((gN - bN) + 0.05) / 0.1), 0.0, 1.0);
            float wGb = ((uGb * uGb) * (3.0 - (2.0 * uGb)));
            float skinMetric = ((((gauss * wLum) * wRg) * wRb) * wGb);
            skinProtect = (1.0 - (0.85 * skinMetric));
        }
        vibGain = ((vibrance / 100.0) * 0.6);
        if ((vibrance > 0.0)) {
            float normChroma = clamp((chroma / max(lum_1, 0.0001)), 0.0, 1.0);
            float _e193 = skinProtect;
            vibGain = ((((vibrance / 100.0) * (1.0 - normChroma)) * 0.8) * _e193);
        }
        vec3 _e195 = c_8;
        vec3 _e196 = c_8;
        float _e199 = vibGain;
        c_8 = (_e195 + ((_e196 - vec3(lum_1)) * _e199));
    }
    if ((deepBlue != 0.0)) {
        float _e205 = c_8.x;
        float _e207 = c_8.y;
        float maxWarm = max(_e205, _e207);
        float _e210 = c_8.z;
        float skySignal = max((_e210 - maxWarm), 0.0);
        if ((skySignal > 0.0)) {
            float x_3 = (deepBlue / 100.0);
            float factor = (sign(x_3) * pow(abs(x_3), 1.6));
            float _e225 = c_8.x;
            c_8.x = (_e225 - ((factor * skySignal) * 0.28));
            float _e232 = c_8.y;
            c_8.y = (_e232 - ((factor * skySignal) * 0.18));
            float _e239 = c_8.z;
            c_8.z = (_e239 - ((factor * skySignal) * 0.15));
        }
    }
    if ((foliage != 0.0)) {
        float _e247 = c_8.x;
        float _e249 = c_8.z;
        float maxRb = max(_e247, _e249);
        float _e252 = c_8.y;
        float greenSignal = max((_e252 - maxRb), 0.0);
        if ((greenSignal > 0.0)) {
            float x_4 = (foliage / 100.0);
            float factor_1 = (sign(x_4) * pow(abs(x_4), 1.6));
            float _e267 = c_8.x;
            c_8.x = (_e267 - ((factor_1 * greenSignal) * 0.3));
            float _e274 = c_8.z;
            c_8.z = (_e274 - ((factor_1 * greenSignal) * 0.15));
            float _e281 = c_8.y;
            c_8.y = (_e281 + ((factor_1 * greenSignal) * 0.1));
        }
    }
    vec3 _e286 = c_8;
    return max(_e286, vec3(0.0));
}

vec3 applySplitToning(vec3 rgb_2, float shadowHue, float shadowSat, float highlightHue, float highlightSat, float balance) {
    bool local_83 = false;
    vec3 delta = vec3(0.0);
    bool local_84 = false;
    bool local_85 = false;
    if ((shadowSat <= 0.0)) {
        local_83 = (highlightSat <= 0.0);
    } else {
        local_83 = false;
    }
    bool _e13 = local_83;
    if (_e13) {
        return rgb_2;
    }
    float lum_2 = (((0.2627 * rgb_2.x) + (0.678 * rgb_2.y)) + (0.0593 * rgb_2.z));
    float yClamp = clamp(lum_2, 0.0, 1.0);
    float t_1 = clamp((0.5 - ((balance / 100.0) * 0.25)), 0.1, 0.9);
    if ((shadowSat > 0.0)) {
        local_84 = (yClamp < t_1);
    } else {
        local_84 = false;
    }
    bool _e46 = local_84;
    if (_e46) {
        float u_s = ((t_1 - yClamp) / t_1);
        float w_s = (((u_s * u_s) * (3.0 - (2.0 * u_s))) * (shadowSat / 100.0));
        vec3 _e61 = hsv2rgb(vec3(shadowHue, 1.0, 1.0));
        float tLum = (((0.2627 * _e61.x) + (0.678 * _e61.y)) + (0.0593 * _e61.z));
        vec3 _e73 = delta;
        delta = (_e73 + (vec3(w_s) * (_e61 - vec3(tLum))));
    }
    if ((highlightSat > 0.0)) {
        local_85 = (yClamp > t_1);
    } else {
        local_85 = false;
    }
    bool _e85 = local_85;
    if (_e85) {
        float u_h = ((yClamp - t_1) / (1.0 - t_1));
        float w_h = (((u_h * u_h) * (3.0 - (2.0 * u_h))) * (highlightSat / 100.0));
        vec3 _e102 = hsv2rgb(vec3(highlightHue, 1.0, 1.0));
        float tLum_1 = (((0.2627 * _e102.x) + (0.678 * _e102.y)) + (0.0593 * _e102.z));
        vec3 _e114 = delta;
        delta = (_e114 + (vec3(w_h) * (_e102 - vec3(tLum_1))));
    }
    vec3 _e120 = delta;
    return clamp((rgb_2 + _e120), vec3(0.0), vec3(1.0));
}

float applyPaperGrade(float val, float grade) {
    float scale = 0.4;
    if ((abs(grade) < 0.001)) {
        return val;
    }
    if ((grade >= 0.0)) {
        scale = 0.6;
    }
    float _e12 = scale;
    float alpha_1 = ((grade / 2.0) * _e12);
    return clamp((val + (((alpha_1 * val) * (1.0 - val)) * ((2.0 * val) - 1.0))), 0.0, 1.0);
}

vec3 applyBaseWarmth(vec3 rgb_3, float warmth) {
    vec3 mult = vec3(1.0);
    if ((abs(warmth) < 0.001)) {
        return rgb_3;
    }
    float lum_3 = (((0.2627 * rgb_3.x) + (0.678 * rgb_3.y)) + (0.0593 * rgb_3.z));
    float highlightWeight = clamp(pow(lum_3, 0.6), 0.0, 1.0);
    float factor_2 = ((warmth / 100.0) * highlightWeight);
    if ((factor_2 > 0.0)) {
        mult = vec3(1.0, (1.0 - (factor_2 * 0.013849)), (1.0 - (factor_2 * 0.051111)));
    } else {
        float absF = -(factor_2);
        mult = vec3((1.0 - (absF * 0.033699)), (1.0 - (absF * 0.031749)), 1.0);
    }
    vec3 _e50 = mult;
    return clamp((rgb_3 * _e50), vec3(0.0), vec3(1.0));
}

vec3 getEnlargerFilterWeights(int filterMode) {
    if ((filterMode == 1)) {
        return vec3(0.35, 0.6, 0.05);
    }
    if ((filterMode == 2)) {
        return vec3(0.55, 0.42, 0.03);
    }
    if ((filterMode == 3)) {
        return vec3(0.85, 0.15, 0.0);
    }
    if ((filterMode == 4)) {
        return vec3(0.15, 0.8, 0.05);
    }
    if ((filterMode == 5)) {
        return vec3(0.0, 0.6, 0.4);
    }
    return vec3(0.2627, 0.678, 0.0593);
}

vec3 applyPrintMedia(vec3 rgb_4, int mediaMode, int filterMode_1, float paperGrade, float processDepth) {
    float rOut = 0.0;
    float gOut = 0.0;
    float bOut = 0.0;
    if ((mediaMode == 0)) {
        return rgb_4;
    }
    float depth = clamp((processDepth / 100.0), 0.0, 1.0);
    if ((mediaMode == 1)) {
        float _e15 = applyPaperGrade(rgb_4.x, paperGrade);
        float _e17 = applyPaperGrade(rgb_4.y, paperGrade);
        float _e19 = applyPaperGrade(rgb_4.z, paperGrade);
        float uR = clamp(((_e15 - 0.7) / 0.3), 0.0, 1.0);
        float rP = (_e15 - ((((0.045 * depth) * uR) * uR) * (3.0 - (2.0 * uR))));
        float uG = clamp(((_e17 - 0.7) / 0.3), 0.0, 1.0);
        float gP = (_e17 - ((((0.045 * depth) * uG) * uG) * (3.0 - (2.0 * uG))));
        float uB = clamp(((_e19 - 0.7) / 0.3), 0.0, 1.0);
        float bP = (_e19 - ((((0.045 * depth) * uB) * uB) * (3.0 - (2.0 * uB))));
        float lum_4 = (((0.2627 * rP) + (0.678 * gP)) + (0.0593 * bP));
        float satScale = (0.7 + (0.35 * depth));
        rOut = clamp((lum_4 + ((rP - lum_4) * satScale)), 0.0, 1.0);
        gOut = clamp((lum_4 + ((gP - lum_4) * satScale)), 0.0, 1.0);
        bOut = clamp((lum_4 + ((bP - lum_4) * satScale)), 0.0, 1.0);
        float dyeDensity = ((0.04 * max((1.0 - lum_4), 0.0)) * depth);
        float _e111 = rOut;
        rOut = clamp((_e111 * (1.0 - (dyeDensity * 0.4))), 0.0, 1.0);
        float _e120 = gOut;
        gOut = clamp((_e120 * (1.0 - (dyeDensity * 0.6))), 0.0, 1.0);
        float _e129 = bOut;
        bOut = clamp((_e129 * (1.0 - (dyeDensity * 0.9))), 0.0, 1.0);
        float _e138 = rOut;
        float _e139 = gOut;
        float _e140 = bOut;
        return vec3(_e138, _e139, _e140);
    } else {
        if ((mediaMode == 2)) {
            float _e145 = applyPaperGrade(rgb_4.x, paperGrade);
            float _e147 = applyPaperGrade(rgb_4.y, paperGrade);
            float _e149 = applyPaperGrade(rgb_4.z, paperGrade);
            float shadowPow = (1.0 + (0.12 * depth));
            float rP_1 = pow(_e145, shadowPow);
            float gP_1 = pow(_e147, shadowPow);
            float bP_1 = pow(_e149, shadowPow);
            float lum_5 = (((0.2627 * rP_1) + (0.678 * gP_1)) + (0.0593 * bP_1));
            float azoSat = (0.8 + (0.3 * depth));
            float rOut_1 = clamp((lum_5 + ((rP_1 - lum_5) * azoSat)), 0.0, 1.0);
            float gOut_1 = clamp((lum_5 + ((gP_1 - lum_5) * azoSat)), 0.0, 1.0);
            float bOut_1 = clamp((lum_5 + ((bP_1 - lum_5) * azoSat)), 0.0, 1.0);
            return vec3(rOut_1, gOut_1, bOut_1);
        } else {
            if ((mediaMode == 3)) {
                vec3 _e190 = getEnlargerFilterWeights(filterMode_1);
                float rawLum = clamp(dot(rgb_4, _e190), 0.0, 1.0);
                float _e195 = applyPaperGrade(rawLum, paperGrade);
                float dmaxGamma = (1.0 + (0.08 * depth));
                float silverToned = pow(_e195, dmaxGamma);
                return vec3(silverToned);
            } else {
                if ((mediaMode == 4)) {
                    vec3 _e204 = getEnlargerFilterWeights(filterMode_1);
                    float rawLum_1 = clamp(dot(rgb_4, _e204), 0.0, 1.0);
                    float _e209 = applyPaperGrade(rawLum_1, paperGrade);
                    float shadowPow_1 = (1.0 + (0.1 * depth));
                    float silverDeep = pow(_e209, shadowPow_1);
                    float midMask = ((4.0 * _e209) * (1.0 - _e209));
                    float shadowMask = (((4.0 * _e209) * (1.0 - _e209)) * (1.0 - _e209));
                    float rTone = (silverDeep + (depth * ((midMask * 0.025) + (shadowMask * 0.02))));
                    float gTone = (silverDeep + (depth * ((midMask * 0.01) - (shadowMask * 0.012))));
                    float bTone = (silverDeep + (depth * ((-(midMask) * 0.015) + (shadowMask * 0.025))));
                    vec3 toned = clamp(vec3(rTone, gTone, bTone), vec3(0.0), vec3(1.0));
                    vec3 _e257 = applyBaseWarmth(toned, 5.0);
                    return _e257;
                } else {
                    if ((mediaMode == 5)) {
                        vec3 _e260 = getEnlargerFilterWeights(filterMode_1);
                        float rawLum_2 = clamp(dot(rgb_4, _e260), 0.0, 1.0);
                        float _e265 = applyPaperGrade(rawLum_2, paperGrade);
                        float ptScale = (_e265 + ((((depth * 0.1) * _e265) * (1.0 - _e265)) * (1.0 - _e265)));
                        float midWarmth = ((4.0 * _e265) * (1.0 - _e265));
                        float rTone_1 = (ptScale + ((depth * midWarmth) * 0.04));
                        float gTone_1 = (ptScale + ((depth * midWarmth) * 0.024));
                        float bTone_1 = (ptScale - ((depth * midWarmth) * 0.016));
                        vec3 toned_1 = clamp(vec3(rTone_1, gTone_1, bTone_1), vec3(0.0), vec3(1.0));
                        vec3 _e301 = applyBaseWarmth(toned_1, (10.0 * depth));
                        return _e301;
                    } else {
                        if ((mediaMode == 6)) {
                            vec3 _e304 = getEnlargerFilterWeights(filterMode_1);
                            float rawLum_3 = clamp(dot(rgb_4, _e304), 0.0, 1.0);
                            float _e309 = applyPaperGrade(rawLum_3, paperGrade);
                            float density = (1.0 - _e309);
                            float rP_2 = (_e309 * (1.0 - ((0.85 * depth) * density)));
                            float gP_2 = (_e309 * (1.0 - ((0.45 * depth) * density)));
                            float bP_2 = (_e309 * (1.0 - ((0.05 * depth) * density)));
                            vec3 toned_2 = clamp(vec3(rP_2, gP_2, bP_2), vec3(0.0), vec3(1.0));
                            vec3 _e338 = applyBaseWarmth(toned_2, (8.0 * depth));
                            return _e338;
                        } else {
                            if ((mediaMode == 7)) {
                                vec3 _e341 = getEnlargerFilterWeights(filterMode_1);
                                float rawLum_4 = clamp(dot(rgb_4, _e341), 0.0, 1.0);
                                float _e346 = applyPaperGrade(rawLum_4, paperGrade);
                                float c_11 = clamp(_e346, 0.0, 1.0);
                                float sLum = clamp(((c_11 + (((0.22 * c_11) * (1.0 - c_11)) * ((2.0 * c_11) - 1.0))) + ((0.14 * c_11) * (1.0 - c_11))), 0.0, 1.0);
                                float ferro = pow(sLum, (1.0 + (0.15 * depth)));
                                float silver = clamp(((ferro * depth) + (_e346 * (1.0 - depth))), 0.0, 1.0);
                                return vec3(silver);
                            }
                        }
                    }
                }
            }
        }
    }
    return rgb_4;
}

float applyFilmCharacteristicCurve(float val_1, float gamma, float shoulder, float toe) {
    float base = 0.0;
    float v_3 = clamp(val_1, 0.0, 2.0);
    base = v_3;
    if ((abs((gamma - 1.0)) > 0.001)) {
        float u_7 = clamp((v_3 / (v_3 + 0.35)), 0.0, 1.0);
        float gMod = ((gamma - 1.0) * 0.4);
        base = max((v_3 + (((gMod * u_7) * (1.0 - u_7)) * ((2.0 * u_7) - 1.0))), 0.0);
    }
    float _e35 = base;
    float tS = clamp(((_e35 - 0.65) / 0.35), 0.0, 1.0);
    float compS = ((((shoulder * 0.12) * tS) * tS) * (3.0 - (2.0 * tS)));
    float _e52 = base;
    float tT = clamp((_e52 / 0.35), 0.0, 1.0);
    float compT = (((toe * 0.06) * (1.0 - tT)) * (1.0 - tT));
    float _e66 = base;
    return max(((_e66 - compS) - compT), 0.0);
}

vec3 applyDirChemicalCoupling(vec3 rgb_5, float c_rg, float c_rb, float c_gr, float c_gb, float c_br, float c_bg) {
    float d_r = -((log(max(rgb_5.x, 0.0001)) * 0.4342945));
    float d_g = -((log(max(rgb_5.y, 0.0001)) * 0.4342945));
    float d_b = -((log(max(rgb_5.z, 0.0001)) * 0.4342945));
    float c_rr = ((1.0 - c_rg) - c_rb);
    float c_gg = ((1.0 - c_gr) - c_gb);
    float c_bb = ((1.0 - c_br) - c_bg);
    float d_r_out = (((c_rr * d_r) + (c_rg * d_g)) + (c_rb * d_b));
    float d_g_out = (((c_gr * d_r) + (c_gg * d_g)) + (c_gb * d_b));
    float d_b_out = (((c_br * d_r) + (c_bg * d_g)) + (c_bb * d_b));
    return clamp(vec3(pow(10.0, -(d_r_out)), pow(10.0, -(d_g_out)), pow(10.0, -(d_b_out))), vec3(0.0), vec3(2.0));
}

float applyEmulsionSCurve(float x_2, float alpha, float beta) {
    float c_12 = clamp(x_2, 0.0, 1.0);
    float poly = ((c_12 + (((alpha * c_12) * (1.0 - c_12)) * ((2.0 * c_12) - 1.0))) + ((beta * c_12) * (1.0 - c_12)));
    return clamp(poly, 0.0, 1.0);
}

vec3 applyRecordingEmulsion(vec3 rgb_6, int archetype_1, float pushPullEv, int filterMode_2) {
    if ((archetype_1 == 0)) {
        return rgb_6;
    }
    float expGain = pow(2.0, clamp(pushPullEv, -2.0, 2.0));
    vec3 curRgb = max((rgb_6 * expGain), vec3(0.0));
    if ((archetype_1 == 1)) {
        vec3 _e23 = applyDirChemicalCoupling(curRgb, -0.04, 0.02, -0.08, 0.02, 0.02, -0.05);
        float _e28 = applyFilmCharacteristicCurve(_e23.x, 0.92, 0.38, 0.04);
        float _e33 = applyFilmCharacteristicCurve(_e23.y, 0.9, 0.38, 0.06);
        float _e38 = applyFilmCharacteristicCurve(_e23.z, 0.88, 0.4, 0.12);
        return vec3(_e28, _e33, _e38);
    } else {
        if ((archetype_1 == 2)) {
            vec3 _e48 = applyDirChemicalCoupling(curRgb, -0.15, -0.08, -0.18, -0.08, -0.06, -0.12);
            float _e53 = applyFilmCharacteristicCurve(_e48.x, 1.1, 0.12, 0.14);
            float _e58 = applyFilmCharacteristicCurve(_e48.y, 1.12, 0.1, 0.14);
            float _e63 = applyFilmCharacteristicCurve(_e48.z, 1.11, 0.12, 0.16);
            return vec3(_e53, _e58, _e63);
        } else {
            if ((archetype_1 == 3)) {
                vec3 _e73 = applyDirChemicalCoupling(curRgb, -0.08, 0.06, -0.1, 0.04, 0.08, -0.04);
                float _e78 = applyFilmCharacteristicCurve(_e73.x, 1.05, 0.18, 0.08);
                float _e83 = applyFilmCharacteristicCurve(_e73.y, 1.06, 0.16, 0.1);
                float _e88 = applyFilmCharacteristicCurve(_e73.z, 1.08, 0.14, 0.14);
                return vec3(_e78, _e83, _e88);
            } else {
                if ((archetype_1 == 4)) {
                    vec3 _e98 = applyDirChemicalCoupling(curRgb, -0.02, 0.01, -0.03, 0.01, 0.01, -0.02);
                    float _e103 = applyFilmCharacteristicCurve(_e98.x, 0.82, 0.45, 0.08);
                    float _e108 = applyFilmCharacteristicCurve(_e98.y, 0.82, 0.45, 0.08);
                    float _e113 = applyFilmCharacteristicCurve(_e98.z, 0.82, 0.45, 0.08);
                    return vec3(_e103, _e108, _e113);
                } else {
                    if ((archetype_1 == 5)) {
                        vec3 _e123 = applyDirChemicalCoupling(curRgb, -0.02, 0.04, -0.06, 0.03, 0.04, -0.03);
                        float _e128 = applyFilmCharacteristicCurve(_e123.x, 1.12, 0.22, 0.05);
                        float _e133 = applyFilmCharacteristicCurve(_e123.y, 1.08, 0.22, 0.08);
                        float _e138 = applyFilmCharacteristicCurve(_e123.z, 0.98, 0.24, 0.16);
                        return vec3(_e128, _e133, _e138);
                    } else {
                        if ((archetype_1 == 6)) {
                            vec3 _e142 = getEnlargerFilterWeights(filterMode_2);
                            float panLum = dot(curRgb, _e142);
                            float _e147 = applyFilmCharacteristicCurve(panLum, 1.08, 0.3, 0.25);
                            return vec3(_e147);
                        } else {
                            if ((archetype_1 == 7)) {
                                vec3 _e157 = applyDirChemicalCoupling(curRgb, -0.03, 0.02, -0.04, 0.03, 0.01, -0.03);
                                float _e162 = applyFilmCharacteristicCurve(_e157.x, 0.98, 0.35, 0.12);
                                float _e167 = applyFilmCharacteristicCurve(_e157.y, 0.98, 0.35, 0.12);
                                float _e172 = applyFilmCharacteristicCurve(_e157.z, 0.98, 0.35, 0.12);
                                return vec3(_e162, _e167, _e172);
                            } else {
                                if ((archetype_1 == 8)) {
                                    vec3 _e182 = applyDirChemicalCoupling(curRgb, -0.05, 0.01, -0.07, 0.02, 0.02, -0.04);
                                    float _e187 = applyFilmCharacteristicCurve(_e182.x, 0.96, 0.32, 0.08);
                                    float _e192 = applyFilmCharacteristicCurve(_e182.y, 0.94, 0.32, 0.03);
                                    float _e197 = applyFilmCharacteristicCurve(_e182.z, 0.92, 0.3, 0.1);
                                    return vec3(_e187, _e192, _e197);
                                } else {
                                    if ((archetype_1 == 9)) {
                                        vec3 _e207 = applyDirChemicalCoupling(curRgb, -0.06, 0.02, -0.14, 0.05, 0.04, -0.08);
                                        float _e212 = applyFilmCharacteristicCurve(_e207.x, 0.92, 0.36, 0.14);
                                        float _e217 = applyFilmCharacteristicCurve(_e207.y, 0.92, 0.36, 0.04);
                                        float _e222 = applyFilmCharacteristicCurve(_e207.z, 0.94, 0.34, 0.04);
                                        return vec3(_e212, _e217, _e222);
                                    } else {
                                        if ((archetype_1 == 10)) {
                                            vec3 _e226 = getEnlargerFilterWeights(filterMode_2);
                                            float panLum_1 = dot(curRgb, _e226);
                                            float _e230 = applyEmulsionSCurve(panLum_1, 0.32, -0.08);
                                            return vec3(_e230);
                                        } else {
                                            if ((archetype_1 == 11)) {
                                                vec3 _e234 = getEnlargerFilterWeights(filterMode_2);
                                                float panLum_2 = dot(curRgb, _e234);
                                                float _e238 = applyEmulsionSCurve(panLum_2, 0.18, 0.1);
                                                return vec3(_e238);
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

float ign(vec2 p_2) {
    vec3 ignCoeffs = vec3(0.06711056, 0.00583715, 52.982918);
    return fract((ignCoeffs.z * fract(dot(p_2, ignCoeffs.xy))));
}

void main() {
    VertexOutput in_ = VertexOutput(gl_FragCoord, _vs2fs_location0);
    bool local = false;
    bool local_1 = false;
    bool local_2 = false;
    vec4 sampledColor = vec4(0.0);
    bool local_3 = false;
    bool local_4 = false;
    bool local_5 = false;
    bool local_6 = false;
    bool local_7 = false;
    bool local_8 = false;
    bool local_9 = false;
    bool local_10 = false;
    bool local_11 = false;
    bool local_12 = false;
    bool local_13 = false;
    bool local_14 = false;
    bool local_15 = false;
    bool local_16 = false;
    bool local_17 = false;
    bool local_18 = false;
    bool local_19 = false;
    bool local_20 = false;
    bool local_21 = false;
    bool local_22 = false;
    bool local_23 = false;
    bool local_24 = false;
    bool local_25 = false;
    bool local_26 = false;
    bool local_27 = false;
    bool local_28 = false;
    bool local_29 = false;
    bool local_30 = false;
    bool local_31 = false;
    float editExposure = 0.0;
    float editTemp = 0.0;
    float editTint = 0.0;
    float editContrast = 0.0;
    float editHighlights = 0.0;
    float editShadows = 0.0;
    float editWhites = 0.0;
    float editBlacks = 0.0;
    float editVibrance = 0.0;
    float editSaturation = 0.0;
    float editDeepBlue = 0.0;
    float editFoliage = 0.0;
    float editSplitShadowSat = 0.0;
    float editSplitHighlightSat = 0.0;
    vec3 rec2020Color = vec3(0.0);
    float rScale = 1.0;
    float gScale = 1.0;
    float bScale = 1.0;
    float baseTemp = 5500.0;
    bool local_32 = false;
    float baseTint = 0.0;
    bool local_33 = false;
    bool local_34 = false;
    float vigGain = 1.0;
    float exposureGain = 1.0;
    bool local_35 = false;
    bool local_36 = false;
    bool local_37 = false;
    float y_neighbors[16] = float[16](0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    int i = 0;
    vec3 c = vec3(0.0);
    bool local_38 = false;
    bool local_39 = false;
    bool local_40 = false;
    float formatMag = 1.0;
    vec2 halationRadius = vec2(0.0);
    bool local_41 = false;
    float scatterAcc = 0.0;
    int i_1 = 0;
    vec3 c_1 = vec3(0.0);
    bool local_42 = false;
    float editSharpening = 0.0;
    float editNoiseReduction = 0.0;
    int editGrainMode = 0;
    bool local_43 = false;
    bool local_44 = false;
    bool local_45 = false;
    float formatMag_1 = 1.0;
    bool local_46 = false;
    bool local_47 = false;
    float y_l = 0.0;
    float y_r = 0.0;
    float y_t = 0.0;
    float y_b = 0.0;
    float y_tl = 0.0;
    float y_tr = 0.0;
    float y_bl = 0.0;
    float y_br = 0.0;
    bool local_48 = false;
    bool local_49 = false;
    float gy_l = 0.0;
    float gy_r = 0.0;
    float gy_t = 0.0;
    float gy_b = 0.0;
    float gy_tl = 0.0;
    float gy_tr = 0.0;
    float gy_bl = 0.0;
    float gy_br = 0.0;
    float contrastGain = 0.0;
    vec3 srgbLinearOut = vec3(0.0);
    bool local_50 = false;
    vec3 finalColor = vec3(0.0);
    bool local_51 = false;
    bool local_52 = false;
    bool local_53 = false;
    bool local_54 = false;
    bool local_55 = false;
    bool local_56 = false;
    bool local_57 = false;
    bool local_58 = false;
    bool local_59 = false;
    bool local_60 = false;
    bool local_61 = false;
    bool local_62 = false;
    bool local_63 = false;
    bool local_64 = false;
    bool local_65 = false;
    bool local_66 = false;
    bool local_67 = false;
    bool local_68 = false;
    bool local_69 = false;
    ShaderUniforms _e3 = _group_0_binding_0_fs;
    GeometryCoordinates _e4 = transformGeometryChannels(in_.texCoord, _e3);
    vec2 uv_2 = _e4.uv;
    if (!((uv_2.x < 0.0))) {
        local = (uv_2.x > 1.0);
    } else {
        local = true;
    }
    bool _e16 = local;
    if (!(_e16)) {
        local_1 = (uv_2.y < 0.0);
    } else {
        local_1 = true;
    }
    bool _e24 = local_1;
    if (!(_e24)) {
        local_2 = (uv_2.y > 1.0);
    } else {
        local_2 = true;
    }
    bool _e32 = local_2;
    if (_e32) {
        _fs2p_location0 = vec4(0.08, 0.08, 0.08, 1.0);
        return;
    }
    int _e41 = _group_0_binding_0_fs.tcaCorrectionEnabled;
    if ((_e41 != 0)) {
        int _e48 = _group_0_binding_0_fs.hasTca;
        local_3 = (_e48 != 0);
    } else {
        local_3 = false;
    }
    bool _e52 = local_3;
    if (_e52) {
        vec4 sampleG = texture(_group_0_binding_1_fs, vec2(uv_2));
        vec4 sampleR = texture(_group_0_binding_1_fs, vec2(_e4.uvRed));
        vec4 sampleB = texture(_group_0_binding_1_fs, vec2(_e4.uvBlue));
        sampledColor = vec4(sampleR.x, sampleG.y, sampleB.z, sampleG.w);
    } else {
        vec4 _e71 = texture(_group_0_binding_1_fs, vec2(uv_2));
        sampledColor = _e71;
    }
    int _e74 = _group_0_binding_0_fs.isIntermediate;
    if ((_e74 == 1)) {
        vec4 _e77 = sampledColor;
        _fs2p_location0 = _e77;
        return;
    }
    float _e80 = _group_0_binding_0_fs.isOriginal;
    if ((_e80 > 0.5)) {
        int _e87 = _group_0_binding_0_fs.isLinearRaw;
        local_4 = (_e87 == 0);
    } else {
        local_4 = false;
    }
    bool _e91 = local_4;
    if (_e91) {
        vec4 _e92 = sampledColor;
        _fs2p_location0 = _e92;
        return;
    }
    float _e95 = _group_0_binding_0_fs.exposure;
    if ((abs(_e95) < 0.001)) {
        float _e103 = _group_0_binding_0_fs.temperature;
        local_5 = (abs((_e103 - 5500.0)) < 1.0);
    } else {
        local_5 = false;
    }
    bool _e110 = local_5;
    if (_e110) {
        float _e115 = _group_0_binding_0_fs.tint;
        local_6 = (abs(_e115) < 0.001);
    } else {
        local_6 = false;
    }
    bool _e120 = local_6;
    if (_e120) {
        float _e125 = _group_0_binding_0_fs.contrast;
        local_7 = (abs(_e125) < 0.001);
    } else {
        local_7 = false;
    }
    bool _e130 = local_7;
    if (_e130) {
        float _e135 = _group_0_binding_0_fs.highlights;
        local_8 = (abs(_e135) < 0.001);
    } else {
        local_8 = false;
    }
    bool _e140 = local_8;
    if (_e140) {
        float _e145 = _group_0_binding_0_fs.shadows;
        local_9 = (abs(_e145) < 0.001);
    } else {
        local_9 = false;
    }
    bool _e150 = local_9;
    if (_e150) {
        float _e155 = _group_0_binding_0_fs.whites;
        local_10 = (abs(_e155) < 0.001);
    } else {
        local_10 = false;
    }
    bool _e160 = local_10;
    if (_e160) {
        float _e165 = _group_0_binding_0_fs.blacks;
        local_11 = (abs(_e165) < 0.001);
    } else {
        local_11 = false;
    }
    bool _e170 = local_11;
    if (_e170) {
        float _e175 = _group_0_binding_0_fs.vibrance;
        local_12 = (abs(_e175) < 0.001);
    } else {
        local_12 = false;
    }
    bool _e180 = local_12;
    if (_e180) {
        float _e185 = _group_0_binding_0_fs.saturation;
        local_13 = (abs(_e185) < 0.001);
    } else {
        local_13 = false;
    }
    bool _e190 = local_13;
    if (_e190) {
        float _e195 = _group_0_binding_0_fs.deepBlue;
        local_14 = (abs(_e195) < 0.001);
    } else {
        local_14 = false;
    }
    bool _e200 = local_14;
    if (_e200) {
        float _e205 = _group_0_binding_0_fs.foliage;
        local_15 = (abs(_e205) < 0.001);
    } else {
        local_15 = false;
    }
    bool _e210 = local_15;
    if (_e210) {
        float _e215 = _group_0_binding_0_fs.splitToningShadowSat;
        local_16 = (abs(_e215) < 0.001);
    } else {
        local_16 = false;
    }
    bool _e220 = local_16;
    if (_e220) {
        float _e225 = _group_0_binding_0_fs.splitToningHighlightSat;
        local_17 = (abs(_e225) < 0.001);
    } else {
        local_17 = false;
    }
    bool _e230 = local_17;
    if (_e230) {
        int _e235 = _group_0_binding_0_fs.lutEnabled;
        local_18 = (_e235 == 0);
    } else {
        local_18 = false;
    }
    bool _e239 = local_18;
    if (_e239) {
        float _e244 = _group_0_binding_0_fs.sharpeningAmount;
        local_19 = (_e244 == 0.0);
    } else {
        local_19 = false;
    }
    bool _e248 = local_19;
    if (_e248) {
        float _e253 = _group_0_binding_0_fs.noiseReduction;
        local_20 = (_e253 == 0.0);
    } else {
        local_20 = false;
    }
    bool _e257 = local_20;
    if (_e257) {
        float _e262 = _group_0_binding_0_fs.defringeAmount;
        local_21 = (_e262 == 0.0);
    } else {
        local_21 = false;
    }
    bool _e266 = local_21;
    if (_e266) {
        int _e271 = _group_0_binding_0_fs.emulsionArchetype;
        local_22 = (_e271 == 0);
    } else {
        local_22 = false;
    }
    bool _e275 = local_22;
    if (_e275) {
        int _e280 = _group_0_binding_0_fs.halationMode;
        local_23 = (_e280 == 0);
    } else {
        local_23 = false;
    }
    bool _e284 = local_23;
    if (_e284) {
        int _e289 = _group_0_binding_0_fs.emulsionGrainMode;
        local_24 = (_e289 == 0);
    } else {
        local_24 = false;
    }
    bool _e293 = local_24;
    if (_e293) {
        int _e298 = _group_0_binding_0_fs.printMediaMode;
        local_25 = (_e298 == 0);
    } else {
        local_25 = false;
    }
    bool _e302 = local_25;
    if (_e302) {
        int _e307 = _group_0_binding_0_fs.vignettingCorrectionEnabled;
        if (!((_e307 == 0))) {
            int _e315 = _group_0_binding_0_fs.hasVignetting;
            local_27 = (_e315 == 0);
        } else {
            local_27 = true;
        }
        bool _e319 = local_27;
        local_26 = _e319;
    } else {
        local_26 = false;
    }
    bool isNeutralEdits = local_26;
    int _e324 = _group_0_binding_0_fs.isLinearRaw;
    if ((_e324 == 0)) {
        local_28 = isNeutralEdits;
    } else {
        local_28 = false;
    }
    bool _e330 = local_28;
    if (_e330) {
        int _e335 = _group_0_binding_0_fs.hasGainMap;
        local_29 = (_e335 == 0);
    } else {
        local_29 = false;
    }
    bool _e339 = local_29;
    if (_e339) {
        int _e344 = _group_0_binding_0_fs.edrEnabled;
        if (!((_e344 == 0))) {
            float _e352 = _group_0_binding_0_fs.edrHeadroom;
            local_31 = (_e352 <= 1.001);
        } else {
            local_31 = true;
        }
        bool _e356 = local_31;
        local_30 = _e356;
    } else {
        local_30 = false;
    }
    bool _e358 = local_30;
    if (_e358) {
        vec4 _e359 = sampledColor;
        _fs2p_location0 = _e359;
        return;
    }
    float _e362 = _group_0_binding_0_fs.exposure;
    editExposure = _e362;
    float _e366 = _group_0_binding_0_fs.temperature;
    editTemp = _e366;
    float _e370 = _group_0_binding_0_fs.tint;
    editTint = _e370;
    float _e374 = _group_0_binding_0_fs.contrast;
    editContrast = _e374;
    float _e378 = _group_0_binding_0_fs.highlights;
    editHighlights = _e378;
    float _e382 = _group_0_binding_0_fs.shadows;
    editShadows = _e382;
    float _e386 = _group_0_binding_0_fs.whites;
    editWhites = _e386;
    float _e390 = _group_0_binding_0_fs.blacks;
    editBlacks = _e390;
    float _e394 = _group_0_binding_0_fs.vibrance;
    editVibrance = _e394;
    float _e398 = _group_0_binding_0_fs.saturation;
    editSaturation = _e398;
    float _e402 = _group_0_binding_0_fs.deepBlue;
    editDeepBlue = _e402;
    float _e406 = _group_0_binding_0_fs.foliage;
    editFoliage = _e406;
    float _e410 = _group_0_binding_0_fs.splitToningShadowSat;
    editSplitShadowSat = _e410;
    float _e414 = _group_0_binding_0_fs.splitToningHighlightSat;
    editSplitHighlightSat = _e414;
    float _e418 = _group_0_binding_0_fs.isOriginal;
    if ((_e418 > 0.5)) {
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
    int _e438 = _group_0_binding_0_fs.isLinearRaw;
    if ((_e438 == 1)) {
        vec4 _e441 = sampledColor;
        rec2020Color = _e441.xyz;
    } else {
        vec4 _e443 = sampledColor;
        vec3 _e445 = srgbGammaToLinear3_(_e443.xyz);
        rec2020Color = (LINEAR_SRGB_TO_REC2020_ * _e445);
    }
    int _e458 = _group_0_binding_0_fs.isLinearRaw;
    if ((_e458 == 1)) {
        float _e465 = _group_0_binding_0_fs.asShotTemperature;
        local_32 = (_e465 > 1000.0);
    } else {
        local_32 = false;
    }
    bool _e469 = local_32;
    if (_e469) {
        float _e472 = _group_0_binding_0_fs.asShotTemperature;
        baseTemp = _e472;
    }
    int _e477 = _group_0_binding_0_fs.isLinearRaw;
    if ((_e477 == 1)) {
        float _e482 = _group_0_binding_0_fs.asShotTint;
        baseTint = _e482;
    }
    float _e483 = editTemp;
    float _e484 = baseTemp;
    if (!((abs((_e483 - _e484)) >= 1.0))) {
        float _e492 = editTint;
        float _e493 = baseTint;
        local_33 = (abs((_e492 - _e493)) >= 0.1);
    } else {
        local_33 = true;
    }
    bool _e499 = local_33;
    if (_e499) {
        float _e500 = editTemp;
        float _e501 = baseTemp;
        float tempNorm = clamp((_e500 / _e501), 0.25, 4.0);
        rScale = pow(tempNorm, 0.75);
        bScale = pow((1.0 / tempNorm), 0.75);
        float _e512 = editTint;
        float _e513 = baseTint;
        float tintFactor = ((_e512 - _e513) * 0.002);
        gScale = max((1.0 - tintFactor), 0.01);
        float _e521 = rScale;
        rScale = max((_e521 + (tintFactor * 0.5)), 0.01);
        float _e527 = bScale;
        bScale = max((_e527 + (tintFactor * 0.5)), 0.01);
        float _e535 = rec2020Color.x;
        float _e536 = rScale;
        rec2020Color.x = (_e535 * _e536);
        float _e540 = rec2020Color.y;
        float _e541 = gScale;
        rec2020Color.y = (_e540 * _e541);
        float _e545 = rec2020Color.z;
        float _e546 = bScale;
        rec2020Color.z = (_e545 * _e546);
    }
    int _e550 = _group_0_binding_0_fs.vignettingCorrectionEnabled;
    if ((_e550 != 0)) {
        int _e557 = _group_0_binding_0_fs.hasVignetting;
        local_34 = (_e557 != 0);
    } else {
        local_34 = false;
    }
    bool _e561 = local_34;
    if (_e561) {
        float _e564 = _group_0_binding_0_fs.aspectRatio;
        float A_1 = max(_e564, 0.01);
        float xc_1 = (((uv_2.x - 0.5) * 2.0) * A_1);
        float yc = ((uv_2.y - 0.5) * 2.0);
        float r2_3 = (((xc_1 * xc_1) + (yc * yc)) / ((A_1 * A_1) + 1.0));
        float r4_1 = (r2_3 * r2_3);
        float r6_ = (r4_1 * r2_3);
        float r8_ = (r4_1 * r4_1);
        float _e590 = _group_0_binding_0_fs.vignettingK1_;
        float _e596 = _group_0_binding_0_fs.vignettingK2_;
        float _e601 = _group_0_binding_0_fs.vignettingK3_;
        float _e606 = _group_0_binding_0_fs.vignettingK4_;
        float poly_1 = ((((1.0 + (_e590 * r2_3)) + (_e596 * r4_1)) + (_e601 * r6_)) + (_e606 * r8_));
        if ((poly_1 < 1.0)) {
            vigGain = (1.0 / max(poly_1, 0.05));
        } else {
            vigGain = poly_1;
        }
        vec3 _e617 = rec2020Color;
        float _e618 = vigGain;
        rec2020Color = (_e617 * _e618);
    }
    float totalExposure = editExposure;
    if ((totalExposure != 0.0)) {
        exposureGain = pow(2.0, totalExposure);
        vec3 _e627 = rec2020Color;
        float _e628 = exposureGain;
        rec2020Color = (_e627 * _e628);
    }
    float _e630 = editHighlights;
    if (!((_e630 != 0.0))) {
        float _e636 = editShadows;
        local_35 = (_e636 != 0.0);
    } else {
        local_35 = true;
    }
    bool _e640 = local_35;
    if (!(_e640)) {
        float _e644 = editWhites;
        local_36 = (_e644 != 0.0);
    } else {
        local_36 = true;
    }
    bool _e648 = local_36;
    if (!(_e648)) {
        float _e652 = editBlacks;
        local_37 = (_e652 != 0.0);
    } else {
        local_37 = true;
    }
    bool _e656 = local_37;
    if (_e656) {
        vec2 texDim = vec2(uvec2(textureSize(_group_0_binding_1_fs, 0).xy));
        vec2 texelSize = (vec2(1.0) / texDim);
        float maxDim = max(texDim.x, texDim.y);
        float rInner = max(3.0, roundEven((maxDim * 0.003)));
        float rOuter = max(24.0, roundEven((maxDim * 0.024)));
        vec2 stepInner = (texelSize * rInner);
        vec2 stepOuter = (texelSize * rOuter);
        vec3 _e678 = rec2020Color;
        float y_c_4 = dot(_e678, vec3(0.2627, 0.678, 0.0593));
        float _e684 = rScale;
        float _e685 = gScale;
        float _e686 = bScale;
        float _e688 = exposureGain;
        vec3 wbExposureScale = (vec3(_e684, _e685, _e686) * _e688);
        vec4 s_l = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(-(stepInner.x), 0.0))));
        vec4 s_r = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(stepInner.x, 0.0))));
        vec4 s_t = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(0.0, -(stepInner.y)))));
        vec4 s_b = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(0.0, stepInner.y))));
        vec4 s_tl = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(-(stepInner.x), -(stepInner.y)))));
        vec4 s_tr = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(stepInner.x, -(stepInner.y)))));
        vec4 s_bl = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(-(stepInner.x), stepInner.y))));
        vec4 s_br = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(stepInner.x, stepInner.y))));
        vec4 s_ol = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(-(stepOuter.x), 0.0))));
        vec4 s_or = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(stepOuter.x, 0.0))));
        vec4 s_ot = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(0.0, -(stepOuter.y)))));
        vec4 s_ob = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(0.0, stepOuter.y))));
        vec4 s_otl = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(-(stepOuter.x), -(stepOuter.y)))));
        vec4 s_otr = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(stepOuter.x, -(stepOuter.y)))));
        vec4 s_obl = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(-(stepOuter.x), stepOuter.y))));
        vec4 s_obr = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(stepOuter.x, stepOuter.y))));
        vec4 s_neighbors[16] = vec4[16](s_l, s_r, s_t, s_b, s_tl, s_tr, s_bl, s_br, s_ol, s_or, s_ot, s_ob, s_otl, s_otr, s_obl, s_obr);
        bool loop_init_5 = true;
        while(true) {
            if (!loop_init_5) {
                int _e864 = i;
                i = (_e864 + 1);
            }
            loop_init_5 = false;
            int _e835 = i;
            if ((_e835 < 16)) {
            } else {
                break;
            }
            {
                c = vec3(0.0);
                int _e842 = _group_0_binding_0_fs.isLinearRaw;
                if ((_e842 == 1)) {
                    int _e845 = i;
                    c = s_neighbors[_e845].xyz;
                } else {
                    int _e848 = i;
                    vec3 _e851 = srgbGammaToLinear3_(s_neighbors[_e848].xyz);
                    c = (LINEAR_SRGB_TO_REC2020_ * _e851);
                }
                vec3 _e854 = c;
                c = (_e854 * wbExposureScale);
                int _e856 = i;
                vec3 _e858 = c;
                y_neighbors[_e856] = dot(_e858, vec3(0.2627, 0.678, 0.0593));
            }
        }
        vec3 _e867 = rec2020Color;
        float _e868[16] = y_neighbors;
        float _e869 = editHighlights;
        float _e870 = editShadows;
        float _e871 = editWhites;
        float _e872 = editBlacks;
        vec3 _e873 = applyBilateralToneRecovery(_e867, y_c_4, _e868, _e869, _e870, _e871, _e872);
        rec2020Color = _e873;
    }
    float _e876 = _group_0_binding_0_fs.isOriginal;
    if ((_e876 < 0.5)) {
        int _e883 = _group_0_binding_0_fs.emulsionArchetype;
        local_38 = (_e883 != 0);
    } else {
        local_38 = false;
    }
    bool _e887 = local_38;
    if (_e887) {
        vec3 _e888 = rec2020Color;
        int _e891 = _group_0_binding_0_fs.emulsionArchetype;
        float _e894 = _group_0_binding_0_fs.emulsionExposure;
        int _e897 = _group_0_binding_0_fs.enlargerFilterMode;
        vec3 _e898 = applyRecordingEmulsion(_e888, _e891, _e894, _e897);
        rec2020Color = _e898;
    }
    float _e901 = _group_0_binding_0_fs.isOriginal;
    if ((_e901 < 0.5)) {
        int _e908 = _group_0_binding_0_fs.halationMode;
        local_39 = (_e908 > 0);
    } else {
        local_39 = false;
    }
    bool _e912 = local_39;
    if (_e912) {
        int _e917 = _group_0_binding_0_fs.emulsionArchetype;
        local_40 = (_e917 != 7);
    } else {
        local_40 = false;
    }
    bool _e921 = local_40;
    if (_e921) {
        int _e926 = _group_0_binding_0_fs.emulsionFormat;
        if ((_e926 == 1)) {
            formatMag = 0.48;
        } else {
            int _e932 = _group_0_binding_0_fs.emulsionFormat;
            if ((_e932 == 2)) {
                formatMag = 0.28;
            } else {
                int _e938 = _group_0_binding_0_fs.emulsionFormat;
                if ((_e938 == 3)) {
                    formatMag = 7.3;
                } else {
                    int _e944 = _group_0_binding_0_fs.emulsionFormat;
                    if ((_e944 == 4)) {
                        formatMag = 3.4;
                    }
                }
            }
        }
        float _e950 = _group_0_binding_0_fs.aspectRatio;
        float aspect = max(_e950, 0.01);
        int _e956 = _group_0_binding_0_fs.rotation;
        if (!((_e956 == 90))) {
            int _e964 = _group_0_binding_0_fs.rotation;
            local_41 = (_e964 == 270);
        } else {
            local_41 = true;
        }
        bool _e968 = local_41;
        if (_e968) {
            float _e971 = formatMag;
            float _e973 = formatMag;
            halationRadius = vec2(((0.012 * aspect) * _e971), (0.012 * _e973));
        } else {
            float _e979 = formatMag;
            float _e981 = formatMag;
            halationRadius = vec2(((0.012 / aspect) * _e979), (0.012 * _e981));
        }
        float _e985 = rScale;
        float _e986 = gScale;
        float _e987 = bScale;
        float _e989 = exposureGain;
        vec3 wbExposure = (vec3(_e985, _e986, _e987) * _e989);
        float _e993 = ign(in_.position.xy);
        float jitter = (_e993 * 6.2831855);
        float cosJ = cos(jitter);
        float sinJ = sin(jitter);
        mat2x2 rot_1 = mat2x2(vec2(cosJ, sinJ), vec2(-(sinJ), cosJ));
        bool loop_init_6 = true;
        while(true) {
            if (!loop_init_6) {
                int _e1053 = i_1;
                i_1 = (_e1053 + 1);
            }
            loop_init_6 = false;
            int _e1006 = i_1;
            if ((_e1006 < 16)) {
            } else {
                break;
            }
            {
                int _e1010 = i_1;
                vec2 _e1013 = halationRadius;
                vec2 rotatedOffset = ((rot_1 * FERMAT_DISK[_e1010]) * _e1013);
                vec4 s_4 = texture(_group_0_binding_1_fs, vec2((uv_2 + rotatedOffset)));
                c_1 = vec3(0.0);
                int _e1023 = _group_0_binding_0_fs.isLinearRaw;
                if ((_e1023 == 1)) {
                    c_1 = s_4.xyz;
                } else {
                    vec3 _e1028 = srgbGammaToLinear3_(s_4.xyz);
                    c_1 = (LINEAR_SRGB_TO_REC2020_ * _e1028);
                }
                vec3 _e1031 = c_1;
                c_1 = (_e1031 * wbExposure);
                vec3 _e1033 = c_1;
                float y_tap = dot(_e1033, vec3(0.2627, 0.678, 0.0593));
                if ((y_tap > 0.55)) {
                    float diff = (y_tap - 0.55);
                    float excess = ((diff * diff) / (0.25 + diff));
                    float _e1047 = scatterAcc;
                    int _e1049 = i_1;
                    scatterAcc = (_e1047 + (excess * FERMAT_WEIGHTS[_e1049]));
                }
            }
        }
        float _e1056 = scatterAcc;
        float halo = (_e1056 / 3.6187);
        if ((halo > 1e-5)) {
            int _e1063 = _group_0_binding_0_fs.halationMode;
            if ((_e1063 == 1)) {
                vec3 _e1066 = rec2020Color;
                rec2020Color = (_e1066 + vec3((halo * 0.35)));
            } else {
                int _e1073 = _group_0_binding_0_fs.halationMode;
                if ((_e1073 == 2)) {
                    vec3 _e1076 = rec2020Color;
                    rec2020Color = (_e1076 + vec3((halo * 0.5), (halo * 0.12), (halo * 0.02)));
                }
            }
        }
    }
    int _e1087 = _group_0_binding_0_fs.hasCurves;
    if ((_e1087 != 0)) {
        float _e1094 = _group_0_binding_0_fs.isOriginal;
        local_42 = (_e1094 < 0.5);
    } else {
        local_42 = false;
    }
    bool _e1098 = local_42;
    if (_e1098) {
        int _e1101 = _group_0_binding_0_fs.hasMasterCurve;
        if ((_e1101 != 0)) {
            float _e1108 = rec2020Color.x;
            vec4 _e1112 = texture(_group_0_binding_5_fs, vec2(clamp(_e1108, 0.0, 1.0), 0.0));
            rec2020Color.x = _e1112.x;
            float _e1118 = rec2020Color.y;
            vec4 _e1122 = texture(_group_0_binding_5_fs, vec2(clamp(_e1118, 0.0, 1.0), 0.0));
            rec2020Color.y = _e1122.x;
            float _e1128 = rec2020Color.z;
            vec4 _e1132 = texture(_group_0_binding_5_fs, vec2(clamp(_e1128, 0.0, 1.0), 0.0));
            rec2020Color.z = _e1132.x;
        }
        int _e1136 = _group_0_binding_0_fs.hasRgbCurves;
        if ((_e1136 != 0)) {
            float _e1142 = rec2020Color.x;
            vec4 _e1146 = texture(_group_0_binding_6_fs, vec2(clamp(_e1142, 0.0, 1.0), 0.0));
            float _e1151 = rec2020Color.y;
            vec4 _e1155 = texture(_group_0_binding_6_fs, vec2(clamp(_e1151, 0.0, 1.0), 0.0));
            float _e1160 = rec2020Color.z;
            vec4 _e1164 = texture(_group_0_binding_6_fs, vec2(clamp(_e1160, 0.0, 1.0), 0.0));
            rec2020Color = vec3(_e1146.x, _e1155.y, _e1164.z);
        }
    }
    vec3 _e1167 = rec2020Color;
    float _e1168 = editVibrance;
    float _e1169 = editSaturation;
    float _e1170 = editDeepBlue;
    float _e1171 = editFoliage;
    vec3 _e1172 = applyColorAdjustments(_e1167, _e1168, _e1169, _e1170, _e1171);
    rec2020Color = _e1172;
    vec3 _e1173 = rec2020Color;
    float _e1176 = _group_0_binding_0_fs.splitToningShadowHue;
    float _e1177 = editSplitShadowSat;
    float _e1180 = _group_0_binding_0_fs.splitToningHighlightHue;
    float _e1181 = editSplitHighlightSat;
    float _e1184 = _group_0_binding_0_fs.splitToningBalance;
    vec3 _e1185 = applySplitToning(_e1173, _e1176, _e1177, _e1180, _e1181, _e1184);
    rec2020Color = _e1185;
    float _e1188 = _group_0_binding_0_fs.sharpeningAmount;
    editSharpening = _e1188;
    float _e1192 = _group_0_binding_0_fs.noiseReduction;
    editNoiseReduction = _e1192;
    int _e1196 = _group_0_binding_0_fs.emulsionGrainMode;
    editGrainMode = _e1196;
    float _e1200 = _group_0_binding_0_fs.isOriginal;
    if ((_e1200 > 0.5)) {
        editSharpening = 0.0;
        editNoiseReduction = 0.0;
        editGrainMode = 0;
    }
    float _e1206 = editSharpening;
    if (!((_e1206 > 0.0))) {
        float _e1212 = editNoiseReduction;
        local_43 = (_e1212 > 0.0);
    } else {
        local_43 = true;
    }
    bool _e1216 = local_43;
    if (!(_e1216)) {
        int _e1220 = editGrainMode;
        local_44 = (_e1220 != 0);
    } else {
        local_44 = true;
    }
    bool _e1224 = local_44;
    if (!(_e1224)) {
        float _e1230 = _group_0_binding_0_fs.defringeAmount;
        local_45 = (_e1230 > 0.0);
    } else {
        local_45 = true;
    }
    bool _e1234 = local_45;
    if (_e1234) {
        int _e1239 = _group_0_binding_0_fs.emulsionFormat;
        if ((_e1239 == 1)) {
            formatMag_1 = 0.48;
        } else {
            int _e1245 = _group_0_binding_0_fs.emulsionFormat;
            if ((_e1245 == 2)) {
                formatMag_1 = 0.28;
            } else {
                int _e1251 = _group_0_binding_0_fs.emulsionFormat;
                if ((_e1251 == 3)) {
                    formatMag_1 = 7.3;
                } else {
                    int _e1257 = _group_0_binding_0_fs.emulsionFormat;
                    if ((_e1257 == 4)) {
                        formatMag_1 = 3.4;
                    }
                }
            }
        }
        vec2 texSize = vec2(uvec2(textureSize(_group_0_binding_1_fs, 0).xy));
        vec2 grainOffset = (vec2(1.0) / texSize);
        vec2 pixelCoord = (uv_2 * texSize);
        vec3 _e1268 = rec2020Color;
        float y_c_5 = dot(_e1268, vec3(0.2627, 0.678, 0.0593));
        float _e1274 = rScale;
        float _e1275 = gScale;
        float _e1276 = bScale;
        float _e1278 = exposureGain;
        vec3 wbExposureScale_1 = (vec3(_e1274, _e1275, _e1276) * _e1278);
        float _e1280 = editSharpening;
        if (!((_e1280 > 0.0))) {
            float _e1286 = editNoiseReduction;
            local_46 = (_e1286 > 0.0);
        } else {
            local_46 = true;
        }
        bool _e1290 = local_46;
        if (!(_e1290)) {
            float _e1296 = _group_0_binding_0_fs.defringeAmount;
            local_47 = (_e1296 > 0.0);
        } else {
            local_47 = true;
        }
        bool _e1300 = local_47;
        if (_e1300) {
            float _e1303 = _group_0_binding_0_fs.sharpeningRadius;
            float _e1306 = _group_0_binding_0_fs.baselineSharpness;
            float effectiveRadius = max((_e1303 * _e1306), 0.5);
            vec2 sharpenOffset = (grainOffset * effectiveRadius);
            vec4 leftSample = texture(_group_0_binding_1_fs, vec2((uv_2 - vec2(sharpenOffset.x, 0.0))));
            vec4 rightSample = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(sharpenOffset.x, 0.0))));
            vec4 topSample = texture(_group_0_binding_1_fs, vec2((uv_2 - vec2(0.0, sharpenOffset.y))));
            vec4 bottomSample = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(0.0, sharpenOffset.y))));
            vec4 tlSample = texture(_group_0_binding_1_fs, vec2((uv_2 - sharpenOffset)));
            vec4 trSample = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(sharpenOffset.x, -(sharpenOffset.y)))));
            vec4 blSample = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(-(sharpenOffset.x), sharpenOffset.y))));
            vec4 brSample = texture(_group_0_binding_1_fs, vec2((uv_2 + sharpenOffset)));
            int _e1373 = _group_0_binding_0_fs.isLinearRaw;
            if ((_e1373 == 0)) {
                vec3 _e1378 = srgbGammaToLinear3_(leftSample.xyz);
                y_l = dot((LINEAR_SRGB_TO_REC2020_ * _e1378), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1387 = srgbGammaToLinear3_(rightSample.xyz);
                y_r = dot((LINEAR_SRGB_TO_REC2020_ * _e1387), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1396 = srgbGammaToLinear3_(topSample.xyz);
                y_t = dot((LINEAR_SRGB_TO_REC2020_ * _e1396), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1405 = srgbGammaToLinear3_(bottomSample.xyz);
                y_b = dot((LINEAR_SRGB_TO_REC2020_ * _e1405), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1414 = srgbGammaToLinear3_(tlSample.xyz);
                y_tl = dot((LINEAR_SRGB_TO_REC2020_ * _e1414), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1423 = srgbGammaToLinear3_(trSample.xyz);
                y_tr = dot((LINEAR_SRGB_TO_REC2020_ * _e1423), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1432 = srgbGammaToLinear3_(blSample.xyz);
                y_bl = dot((LINEAR_SRGB_TO_REC2020_ * _e1432), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1441 = srgbGammaToLinear3_(brSample.xyz);
                y_br = dot((LINEAR_SRGB_TO_REC2020_ * _e1441), vec3(0.2627, 0.678, 0.0593));
            } else {
                y_l = dot((leftSample.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
                y_r = dot((rightSample.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
                y_t = dot((topSample.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
                y_b = dot((bottomSample.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
                y_tl = dot((tlSample.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
                y_tr = dot((trSample.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
                y_bl = dot((blSample.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
                y_br = dot((brSample.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
            }
            float _e1504 = y_l;
            float _e1505 = y_r;
            float _e1507 = y_t;
            float _e1508 = y_b;
            float _e1511 = y_tl;
            float _e1512 = y_tr;
            float _e1514 = y_bl;
            float _e1515 = y_br;
            float y_max_neighbor_1 = max(max(max(_e1504, _e1505), max(_e1507, _e1508)), max(max(_e1511, _e1512), max(_e1514, _e1515)));
            float _e1521 = _group_0_binding_0_fs.defringeAmount;
            if ((_e1521 > 0.0)) {
                float _e1528 = _group_0_binding_0_fs.isOriginal;
                local_48 = (_e1528 < 0.5);
            } else {
                local_48 = false;
            }
            bool _e1532 = local_48;
            if (_e1532) {
                vec3 _e1533 = rec2020Color;
                float _e1536 = _group_0_binding_0_fs.defringeAmount;
                vec3 _e1537 = applyDefringe(_e1533, y_c_5, y_max_neighbor_1, _e1536);
                rec2020Color = _e1537;
            }
            float _e1538 = editSharpening;
            if (!((_e1538 > 0.0))) {
                float _e1544 = editNoiseReduction;
                local_49 = (_e1544 > 0.0);
            } else {
                local_49 = true;
            }
            bool _e1548 = local_49;
            if (_e1548) {
                vec3 _e1549 = rec2020Color;
                float _e1550 = y_l;
                float _e1551 = y_r;
                float _e1552 = y_t;
                float _e1553 = y_b;
                float _e1554 = y_tl;
                float _e1555 = y_tr;
                float _e1556 = y_bl;
                float _e1557 = y_br;
                float _e1558 = editSharpening;
                float _e1561 = _group_0_binding_0_fs.sharpeningMasking;
                float _e1562 = editNoiseReduction;
                vec3 _e1563 = applyDetailAndSharpening(_e1549, y_c_5, _e1550, _e1551, _e1552, _e1553, _e1554, _e1555, _e1556, _e1557, _e1558, _e1561, _e1562);
                rec2020Color = _e1563;
            }
        }
        int _e1564 = editGrainMode;
        if ((_e1564 != 0)) {
            vec4 leftSample_1 = texture(_group_0_binding_1_fs, vec2((uv_2 - vec2(grainOffset.x, 0.0))));
            vec4 rightSample_1 = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(grainOffset.x, 0.0))));
            vec4 topSample_1 = texture(_group_0_binding_1_fs, vec2((uv_2 - vec2(0.0, grainOffset.y))));
            vec4 bottomSample_1 = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(0.0, grainOffset.y))));
            vec4 tlSample_1 = texture(_group_0_binding_1_fs, vec2((uv_2 - grainOffset)));
            vec4 trSample_1 = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(grainOffset.x, -(grainOffset.y)))));
            vec4 blSample_1 = texture(_group_0_binding_1_fs, vec2((uv_2 + vec2(-(grainOffset.x), grainOffset.y))));
            vec4 brSample_1 = texture(_group_0_binding_1_fs, vec2((uv_2 + grainOffset)));
            int _e1629 = _group_0_binding_0_fs.isLinearRaw;
            if ((_e1629 == 0)) {
                vec3 _e1634 = srgbGammaToLinear3_(leftSample_1.xyz);
                gy_l = dot((LINEAR_SRGB_TO_REC2020_ * _e1634), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1643 = srgbGammaToLinear3_(rightSample_1.xyz);
                gy_r = dot((LINEAR_SRGB_TO_REC2020_ * _e1643), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1652 = srgbGammaToLinear3_(topSample_1.xyz);
                gy_t = dot((LINEAR_SRGB_TO_REC2020_ * _e1652), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1661 = srgbGammaToLinear3_(bottomSample_1.xyz);
                gy_b = dot((LINEAR_SRGB_TO_REC2020_ * _e1661), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1670 = srgbGammaToLinear3_(tlSample_1.xyz);
                gy_tl = dot((LINEAR_SRGB_TO_REC2020_ * _e1670), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1679 = srgbGammaToLinear3_(trSample_1.xyz);
                gy_tr = dot((LINEAR_SRGB_TO_REC2020_ * _e1679), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1688 = srgbGammaToLinear3_(blSample_1.xyz);
                gy_bl = dot((LINEAR_SRGB_TO_REC2020_ * _e1688), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1697 = srgbGammaToLinear3_(brSample_1.xyz);
                gy_br = dot((LINEAR_SRGB_TO_REC2020_ * _e1697), vec3(0.2627, 0.678, 0.0593));
            } else {
                gy_l = dot((leftSample_1.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
                gy_r = dot((rightSample_1.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
                gy_t = dot((topSample_1.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
                gy_b = dot((bottomSample_1.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
                gy_tl = dot((tlSample_1.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
                gy_tr = dot((trSample_1.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
                gy_bl = dot((blSample_1.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
                gy_br = dot((brSample_1.xyz * wbExposureScale_1), vec3(0.2627, 0.678, 0.0593));
            }
            float _e1760 = formatMag_1;
            float s_format = sqrt(max(_e1760, 0.1));
            float _e1766 = _group_0_binding_0_fs.grainSize;
            float clusterSize = clamp((_e1766 * s_format), 0.5, 3.5);
            float _e1773 = _group_0_binding_0_fs.grainAmount;
            float _e1774 = formatMag_1;
            float effectiveAmount = (_e1773 * pow(max(_e1774, 0.1), 0.25));
            vec3 _e1780 = rec2020Color;
            float _e1781 = gy_l;
            float _e1782 = gy_r;
            float _e1783 = gy_t;
            float _e1784 = gy_b;
            float _e1785 = gy_tl;
            float _e1786 = gy_tr;
            float _e1787 = gy_bl;
            float _e1788 = gy_br;
            int _e1789 = editGrainMode;
            int _e1792 = _group_0_binding_0_fs.emulsionArchetype;
            vec3 _e1793 = applyStochasticEmulsionGrain(_e1780, y_c_5, _e1781, _e1782, _e1783, _e1784, _e1785, _e1786, _e1787, _e1788, pixelCoord, _e1789, clusterSize, effectiveAmount, _e1792);
            rec2020Color = _e1793;
        }
    }
    float _e1794 = editContrast;
    contrastGain = (1.0 + (_e1794 / 100.0));
    float _e1800 = editContrast;
    if ((_e1800 < 0.0)) {
        float _e1803 = editContrast;
        contrastGain = (1.0 + ((_e1803 / 100.0) * 0.6));
    }
    int _e1813 = _group_0_binding_0_fs.isLinearRaw;
    if ((_e1813 == 0)) {
        vec3 _e1817 = rec2020Color;
        vec3 srgb = max((REC2020_TO_LINEAR_SRGB * _e1817), vec3(0.0));
        float _e1822 = contrastGain;
        if ((abs((_e1822 - 1.0)) > 0.001)) {
            float _e1831 = contrastGain;
            srgbLinearOut = clamp((((srgb - vec3(0.18)) * _e1831) + vec3(0.18)), vec3(0.0), vec3(1.0));
        } else {
            srgbLinearOut = clamp(srgb, vec3(0.0), vec3(1.0));
        }
    } else {
        vec3 _e1846 = rec2020Color;
        float _e1847 = contrastGain;
        vec3 _e1848 = agxToneMapVivid(_e1846, _e1847);
        srgbLinearOut = _e1848;
    }
    float _e1851 = _group_0_binding_0_fs.isOriginal;
    if ((_e1851 < 0.5)) {
        int _e1858 = _group_0_binding_0_fs.printMediaMode;
        local_50 = (_e1858 != 0);
    } else {
        local_50 = false;
    }
    bool _e1862 = local_50;
    if (_e1862) {
        vec3 _e1863 = srgbLinearOut;
        int _e1866 = _group_0_binding_0_fs.printMediaMode;
        int _e1869 = _group_0_binding_0_fs.enlargerFilterMode;
        float _e1872 = _group_0_binding_0_fs.printPaperGrade;
        float _e1875 = _group_0_binding_0_fs.printProcessDepth;
        vec3 _e1876 = applyPrintMedia(_e1863, _e1866, _e1869, _e1872, _e1875);
        srgbLinearOut = _e1876;
    }
    vec3 _e1877 = srgbLinearOut;
    finalColor = _e1877;
    int _e1881 = _group_0_binding_0_fs.lutEnabled;
    if ((_e1881 != 0)) {
        float _e1888 = _group_0_binding_0_fs.lutIntensity;
        local_51 = (_e1888 > 0.0);
    } else {
        local_51 = false;
    }
    bool _e1892 = local_51;
    if (_e1892) {
        float _e1897 = _group_0_binding_0_fs.isOriginal;
        local_52 = (_e1897 < 0.5);
    } else {
        local_52 = false;
    }
    bool _e1901 = local_52;
    if (_e1901) {
        float _e1904 = _group_0_binding_0_fs.lutSize;
        float size = max(_e1904, 2.0);
        vec3 _e1907 = finalColor;
        vec3 lutCoord = (((_e1907 * (size - 1.0)) + vec3(0.5)) / vec3(size));
        vec4 _e1918 = texture(_group_0_binding_3_fs, vec3(lutCoord));
        vec3 lutSample = _e1918.xyz;
        vec3 _e1920 = finalColor;
        float _e1923 = _group_0_binding_0_fs.lutIntensity;
        finalColor = mix(_e1920, lutSample, clamp(_e1923, 0.0, 1.0));
    }
    int _e1930 = _group_0_binding_0_fs.softProofEnabled;
    if ((_e1930 != 0)) {
        float _e1937 = _group_0_binding_0_fs.isOriginal;
        local_53 = (_e1937 < 0.5);
    } else {
        local_53 = false;
    }
    bool _e1941 = local_53;
    if (_e1941) {
        vec3 _e1943 = finalColor;
        vec3 proofCoord = (((clamp(_e1943, vec3(0.0), vec3(1.0)) * (33.0 - 1.0)) + vec3(0.5)) / vec3(33.0));
        vec4 proofSample = texture(_group_0_binding_11_fs, vec3(proofCoord));
        int _e1962 = _group_0_binding_0_fs.gamutWarningEnabled;
        if ((_e1962 != 0)) {
            local_54 = (proofSample.w < 0.5);
        } else {
            local_54 = false;
        }
        bool _e1971 = local_54;
        if (_e1971) {
            float zebra = fract(((in_.position.x + in_.position.y) / 12.0));
            if ((zebra < 0.5)) {
                finalColor = vec3(0.0, 0.9, 1.0);
            } else {
                finalColor = vec3(0.0, 0.35, 0.5);
            }
        } else {
            finalColor = proofSample.xyz;
        }
    }
    int _e1993 = _group_0_binding_0_fs.edrEnabled;
    if ((_e1993 != 0)) {
        float _e2000 = _group_0_binding_0_fs.edrHeadroom;
        local_55 = (_e2000 > 1.001);
    } else {
        local_55 = false;
    }
    bool _e2004 = local_55;
    if (_e2004) {
        float _e2007 = _group_0_binding_0_fs.edrHeadroom;
        float headroom = clamp(_e2007, 1.0, 4.0);
        int _e2013 = _group_0_binding_0_fs.hasGainMap;
        if ((_e2013 != 0)) {
            vec4 _e2018 = texture(_group_0_binding_8_fs, vec2(uv_2));
            float g_1 = _e2018.x;
            float _e2022 = _group_0_binding_0_fs.gainMapMin;
            float _e2025 = _group_0_binding_0_fs.gainMapMax;
            float _e2028 = _group_0_binding_0_fs.gainMapMin;
            float gainLog2_ = (_e2022 + (g_1 * (_e2025 - _e2028)));
            float _e2034 = _group_0_binding_0_fs.gainMapGamma;
            float gamma_1 = max(_e2034, 0.01);
            float gainFactor = pow(2.0, (gainLog2_ / gamma_1));
            float _e2042 = _group_0_binding_0_fs.gainMapCapacityMax;
            float maxCap = max(_e2042, 1.01);
            float weight = clamp(((headroom - 1.0) / (maxCap - 1.0)), 0.0, 1.0);
            float effectiveGain = (1.0 + ((gainFactor - 1.0) * weight));
            vec3 _e2058 = finalColor;
            finalColor = (_e2058 * effectiveGain);
        } else {
            int _e2062 = _group_0_binding_0_fs.isLinearRaw;
            if ((_e2062 == 1)) {
                vec3 _e2065 = rec2020Color;
                float sceneLum = dot(_e2065, vec3(0.2627, 0.678, 0.0593));
                vec3 _e2071 = finalColor;
                float outLum = dot(_e2071, vec3(0.2126, 0.7152, 0.0722));
                float highlightFactor = smoothstep(0.85, 1.0, max(outLum, clamp((sceneLum - 0.15), 0.0, 1.0)));
                if ((highlightFactor > 0.0)) {
                    float edrBoost = (1.0 + ((headroom - 1.0) * pow(highlightFactor, 1.5)));
                    vec3 _e2095 = finalColor;
                    finalColor = (_e2095 * edrBoost);
                }
            }
        }
        float _e2098 = finalColor.x;
        float _e2100 = finalColor.y;
        float _e2102 = finalColor.z;
        float peakC = max(_e2098, max(_e2100, _e2102));
        if ((peakC > headroom)) {
            float excess_1 = ((peakC - headroom) / peakC);
            vec3 _e2108 = finalColor;
            finalColor = mix((_e2108 * (headroom / peakC)), vec3(headroom), clamp((excess_1 * 1.5), 0.0, 1.0));
        } else {
            if ((peakC > 1.0)) {
                float desatThreshold = (1.0 + ((headroom - 1.0) * 0.8));
                if ((peakC > desatThreshold)) {
                    float t_2 = ((peakC - desatThreshold) / max((headroom - desatThreshold), 0.0001));
                    vec3 _e2132 = finalColor;
                    finalColor = mix(_e2132, vec3(peakC), clamp((t_2 * 0.4), 0.0, 0.4));
                }
            }
        }
        vec3 _e2141 = finalColor;
        finalColor = max((LINEAR_SRGB_TO_DISPLAY_P3_ * _e2141), vec3(0.0));
        int _e2148 = _group_0_binding_0_fs.clippingOverlayActive;
        if ((_e2148 != 0)) {
            float _e2155 = _group_0_binding_0_fs.isOriginal;
            local_56 = (_e2155 < 0.5);
        } else {
            local_56 = false;
        }
        bool _e2159 = local_56;
        if (_e2159) {
            int _e2162 = _group_0_binding_0_fs.isLinearRaw;
            if ((_e2162 == 1)) {
                float _e2168 = sampledColor.x;
                if (!((_e2168 >= 1.0))) {
                    float _e2175 = sampledColor.y;
                    local_58 = (_e2175 >= 1.0);
                } else {
                    local_58 = true;
                }
                bool _e2179 = local_58;
                if (!(_e2179)) {
                    float _e2184 = sampledColor.z;
                    local_59 = (_e2184 >= 1.0);
                } else {
                    local_59 = true;
                }
                bool _e2188 = local_59;
                local_57 = _e2188;
            } else {
                local_57 = false;
            }
            bool _e2190 = local_57;
            if (!(_e2190)) {
                float _e2195 = finalColor.x;
                float _e2197 = finalColor.y;
                float _e2199 = finalColor.z;
                local_60 = (max(_e2195, max(_e2197, _e2199)) >= (headroom * 0.99));
            } else {
                local_60 = true;
            }
            bool _e2206 = local_60;
            if (!(_e2206)) {
                float _e2211 = rec2020Color.x;
                float _e2213 = rec2020Color.y;
                float _e2215 = rec2020Color.z;
                local_61 = (max(_e2211, max(_e2213, _e2215)) >= 2.88);
            } else {
                local_61 = true;
            }
            bool isHighlightClip = local_61;
            vec3 _e2222 = rec2020Color;
            float sceneLum_1 = dot(_e2222, vec3(0.2627, 0.678, 0.0593));
            if (!((sceneLum_1 <= 0.0007))) {
                float _e2234 = finalColor.x;
                float _e2236 = finalColor.y;
                float _e2238 = finalColor.z;
                local_62 = (max(_e2234, max(_e2236, _e2238)) <= 0.0005);
            } else {
                local_62 = true;
            }
            bool isShadowClip = local_62;
            if (isHighlightClip) {
                float zebraHighlight = fract(((in_.position.x + in_.position.y) / 14.0));
                if ((zebraHighlight < 0.5)) {
                    float _e2256 = sampledColor.w;
                    _fs2p_location0 = vec4(1.0, 0.12, 0.12, _e2256);
                    return;
                }
            } else {
                if (isShadowClip) {
                    float zebraShadow = fract(((in_.position.x - in_.position.y) / 14.0));
                    if ((zebraShadow < 0.5)) {
                        float _e2272 = sampledColor.w;
                        _fs2p_location0 = vec4(0.05, 0.4, 1.0, _e2272);
                        return;
                    }
                }
            }
        }
        vec3 _e2277 = finalColor;
        float _e2279 = sampledColor.w;
        _fs2p_location0 = vec4(_e2277, _e2279);
        return;
    } else {
        vec3 _e2281 = finalColor;
        vec3 _e2287 = linearToSrgbGamma3_(clamp(_e2281, vec3(0.0), vec3(1.0)));
        int _e2290 = _group_0_binding_0_fs.clippingOverlayActive;
        if ((_e2290 != 0)) {
            float _e2297 = _group_0_binding_0_fs.isOriginal;
            local_63 = (_e2297 < 0.5);
        } else {
            local_63 = false;
        }
        bool _e2301 = local_63;
        if (_e2301) {
            int _e2304 = _group_0_binding_0_fs.isLinearRaw;
            if ((_e2304 == 1)) {
                float _e2310 = sampledColor.x;
                if (!((_e2310 >= 1.0))) {
                    float _e2317 = sampledColor.y;
                    local_65 = (_e2317 >= 1.0);
                } else {
                    local_65 = true;
                }
                bool _e2321 = local_65;
                if (!(_e2321)) {
                    float _e2326 = sampledColor.z;
                    local_66 = (_e2326 >= 1.0);
                } else {
                    local_66 = true;
                }
                bool _e2330 = local_66;
                local_64 = _e2330;
            } else {
                local_64 = false;
            }
            bool _e2332 = local_64;
            if (!(_e2332)) {
                local_67 = (max(_e2287.x, max(_e2287.y, _e2287.z)) >= 0.992);
            } else {
                local_67 = true;
            }
            bool _e2344 = local_67;
            if (!(_e2344)) {
                float _e2349 = rec2020Color.x;
                float _e2351 = rec2020Color.y;
                float _e2353 = rec2020Color.z;
                local_68 = (max(_e2349, max(_e2351, _e2353)) >= 2.88);
            } else {
                local_68 = true;
            }
            bool isHighlightClip_1 = local_68;
            vec3 _e2360 = rec2020Color;
            float sceneLum_2 = dot(_e2360, vec3(0.2627, 0.678, 0.0593));
            if (!((sceneLum_2 <= 0.0007))) {
                local_69 = (max(_e2287.x, max(_e2287.y, _e2287.z)) <= 0.004);
            } else {
                local_69 = true;
            }
            bool isShadowClip_1 = local_69;
            if (isHighlightClip_1) {
                float zebraHighlight_1 = fract(((in_.position.x + in_.position.y) / 14.0));
                if ((zebraHighlight_1 < 0.5)) {
                    float _e2391 = sampledColor.w;
                    _fs2p_location0 = vec4(1.0, 0.12, 0.12, _e2391);
                    return;
                }
            } else {
                if (isShadowClip_1) {
                    float zebraShadow_1 = fract(((in_.position.x - in_.position.y) / 14.0));
                    if ((zebraShadow_1 < 0.5)) {
                        float _e2407 = sampledColor.w;
                        _fs2p_location0 = vec4(0.05, 0.4, 1.0, _e2407);
                        return;
                    }
                }
            }
        }
        float _e2414 = ign(in_.position.xy);
        float dither = ((_e2414 - 0.5) / 255.0);
        float _e2427 = sampledColor.w;
        _fs2p_location0 = vec4(clamp((_e2287 + vec3(dither)), vec3(0.0), vec3(1.0)), _e2427);
        return;
    }
}

`;

export const SHADER_UNIFORM_BLOCK_NAME = 'ShaderUniforms_block_0Fragment';
export const SHADER_UNIFORM_SIZE_BYTES = 368;

export const SHADER_SAMPLER_NAMES = {
  texture: '_group_0_binding_1_fs',
  lut3d: '_group_0_binding_3_fs',
  curveMaster: '_group_0_binding_5_fs',
  curveRgb: '_group_0_binding_6_fs',
  gainMap: '_group_0_binding_8_fs',
  proofLut: '_group_0_binding_11_fs',
} as const;
