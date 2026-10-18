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
    float pad;
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

smooth in vec2 _vs2fs_location0;
layout(location = 0) out vec4 _fs2p_location0;

vec2 transformGeometry(vec2 uv, ShaderUniforms uniforms) {
    bool local_45 = false;
    vec2 uvc = vec2(0.0);
    float autoCropScale = 1.0;
    bool local_46 = false;
    int i_2 = 0;
    vec2 g = vec2(0.0);
    float r_corr = 0.0;
    vec2 centered = vec2(0.0);
    vec2 geom = vec2(0.0);
    float r_corr_1 = 0.0;
    if (!((uniforms.isOriginal >= 0.5))) {
        local_45 = (uniforms.isIntermediate == 1);
    } else {
        local_45 = true;
    }
    bool _e12 = local_45;
    if (_e12) {
        return uv;
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
        local_46 = (uniforms.lensCorrectionEnabled != 0);
    } else {
        local_46 = true;
    }
    bool _e69 = local_46;
    if (_e69) {
        vec2 testPoints[12] = vec2[12](vec2(A, 1.0), vec2(-(A), 1.0), vec2(A, -1.0), vec2(-(A), -1.0), vec2(A, 0.0), vec2(-(A), 0.0), vec2(0.0, 1.0), vec2(0.0, -1.0), vec2(A, 0.5), vec2(-(A), 0.5), vec2(A, -0.5), vec2(-(A), -0.5));
        bool loop_init = true;
        while(true) {
            if (!loop_init) {
                int _e187 = i_2;
                i_2 = (_e187 + 1);
            }
            loop_init = false;
            int _e104 = i_2;
            if ((_e104 < 12)) {
            } else {
                break;
            }
            {
                int _e107 = i_2;
                vec2 p_3 = testPoints[_e107];
                vec2 r = vec2(((p_3.x * cosA) - (p_3.y * sinA)), ((p_3.x * sinA) + (p_3.y * cosA)));
                float denom = max(((1.0 + (cV * r.y)) + (cH * r.x)), 0.1);
                g = (r / vec2(denom));
                if ((uniforms.lensCorrectionEnabled != 0)) {
                    vec2 _e135 = g;
                    vec2 _e136 = g;
                    float r2_ = dot(_e135, _e136);
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
                        vec2 _e173 = g;
                        float _e174 = r_corr;
                        g = (_e173 * (_e174 / rad));
                    }
                }
                float _e178 = g.x;
                float _e182 = g.y;
                float req = max((abs(_e178) / A), abs(_e182));
                float _e185 = autoCropScale;
                if ((req > _e185)) {
                    autoCropScale = req;
                }
            }
        }
    }
    vec2 _e190 = uvc;
    centered = ((_e190 - vec2(0.5, 0.5)) * 2.0);
    float _e200 = centered.x;
    centered.x = (_e200 * A);
    vec2 _e202 = centered;
    float _e203 = autoCropScale;
    centered = (_e202 / vec2(_e203));
    float _e207 = centered.x;
    float _e210 = centered.y;
    float _e214 = centered.x;
    float _e217 = centered.y;
    vec2 rot = vec2(((_e207 * cosA) - (_e210 * sinA)), ((_e214 * sinA) + (_e217 * cosA)));
    float denom_1 = max(((1.0 + (cV * rot.y)) + (cH * rot.x)), 0.1);
    geom = (rot / vec2(denom_1));
    if ((uniforms.lensCorrectionEnabled != 0)) {
        vec2 _e236 = geom;
        vec2 _e237 = geom;
        float r2_1 = dot(_e236, _e237);
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
            vec2 _e274 = geom;
            float _e275 = r_corr_1;
            geom = (_e274 * (_e275 / r_1));
        }
    }
    float _e280 = geom.x;
    geom.x = (_e280 / (2.0 * A));
    float _e286 = geom.y;
    geom.y = (_e286 / 2.0);
    vec2 _e289 = geom;
    return (_e289 + vec2(0.5, 0.5));
}

vec3 applyDetailAndSharpening(vec3 color, float y_c, float y_l_1, float y_r_1, float y_t_1, float y_b_1, float y_tl_1, float y_tr_1, float y_bl_1, float y_br_1, float amount, float masking, float noiseReduction) {
    bool local_47 = false;
    float w_edge = 1.0;
    if ((amount <= 0.0)) {
        local_47 = (noiseReduction <= 0.0);
    } else {
        local_47 = false;
    }
    bool _e20 = local_47;
    if (_e20) {
        return color;
    }
    float y_neighbors_2 = ((0.16666667 * (((y_l_1 + y_r_1) + y_t_1) + y_b_1)) + (0.083333336 * (((y_tl_1 + y_tr_1) + y_bl_1) + y_br_1)));
    float delta_y = (y_c - y_neighbors_2);
    float gx = ((((y_tr_1 + (2.0 * y_r_1)) + y_br_1) - ((y_tl_1 + (2.0 * y_l_1)) + y_bl_1)) * 0.125);
    float gy = ((((y_bl_1 + (2.0 * y_b_1)) + y_br_1) - ((y_tl_1 + (2.0 * y_t_1)) + y_tr_1)) * 0.125);
    float grad_mag = sqrt(((gx * gx) + (gy * gy)));
    if ((masking > 0.0)) {
        float m_1 = clamp((masking / 100.0), 0.0, 1.0);
        float threshold = (pow(m_1, 1.5) * 0.08);
        w_edge = smoothstep(0.0, max(threshold, 0.0001), grad_mag);
    }
    float sharp_gain = ((amount / 100.0) * 1.5);
    float _e80 = w_edge;
    float sharp_term = ((sharp_gain * _e80) * delta_y);
    float _e85 = w_edge;
    float nr_term = (((noiseReduction / 100.0) * (1.0 - _e85)) * delta_y);
    float delta_adj = (sharp_term - nr_term);
    return clamp((color + vec3(delta_adj)), vec3(0.0), vec3(1.0));
}

float proceduralGrainNoise(float u, float v) {
    vec2 p_4 = (50.0 * fract(((vec2(u, v) * 0.3183099) + vec2(0.71, 0.113))));
    return (-1.0 + (2.0 * fract(((p_4.x * p_4.y) * (p_4.x + p_4.y)))));
}

float clusteredGrainNoise(vec2 p) {
    vec2 i_7 = floor(p);
    vec2 f = fract(p);
    vec2 u_1 = ((f * f) * (vec2(3.0) - (2.0 * f)));
    float _e12 = proceduralGrainNoise(i_7.x, i_7.y);
    float _e17 = proceduralGrainNoise((i_7.x + 1.0), i_7.y);
    float _e22 = proceduralGrainNoise(i_7.x, (i_7.y + 1.0));
    float _e29 = proceduralGrainNoise((i_7.x + 1.0), (i_7.y + 1.0));
    return mix(mix(_e12, _e17, u_1.x), mix(_e22, _e29, u_1.x), u_1.y);
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

vec3 applyStochasticEmulsionGrain(vec3 color_1, float y_c_1, float y_l_2, float y_r_2, float y_t_2, float y_b_2, float y_tl_2, float y_tr_2, float y_bl_2, float y_br_2, vec2 p_1, int grain_mode_1, float cluster_size, float grain_amount, int archetype) {
    bool local_48 = false;
    bool local_49 = false;
    if ((grain_mode_1 == 0)) {
        return color_1;
    }
    float amount_1 = clamp(grain_amount, 0.0, 2.0);
    if ((amount_1 < 0.001)) {
        return color_1;
    }
    float y_lowpass = ((0.16666667 * (((y_l_2 + y_r_2) + y_t_2) + y_b_2)) + (0.083333336 * (((y_tl_2 + y_tr_2) + y_bl_2) + y_br_2)));
    float delta_sensor = (y_c_1 - y_lowpass);
    float gx_1 = ((((y_tr_2 + (2.0 * y_r_2)) + y_br_2) - ((y_tl_2 + (2.0 * y_l_2)) + y_bl_2)) * 0.125);
    float gy_1 = ((((y_bl_2 + (2.0 * y_b_2)) + y_br_2) - ((y_tl_2 + (2.0 * y_t_2)) + y_tr_2)) * 0.125);
    float grad_mag_1 = sqrt(((gx_1 * gx_1) + (gy_1 * gy_1)));
    float edge_suppress = (1.0 / (1.0 + (35.0 * grad_mag_1)));
    float delta_noise = (delta_sensor * edge_suppress);
    float denom_2 = max(sqrt(max(y_c_1, 0.0)), 0.05);
    float sensor_seed_1 = clamp(((delta_noise / denom_2) * 20.0), -1.0, 1.0);
    bool is_instant_2 = (archetype == 7);
    float s_3 = max(cluster_size, 0.25);
    float _e82 = evalGrainProcNoise(p_1, s_3, is_instant_2);
    float _e83 = evalGrainEta(_e82, sensor_seed_1, grain_mode_1, is_instant_2);
    float y_clamped = clamp(y_c_1, 0.0, 1.0);
    float nutting = ((4.0 * y_clamped) * (1.0 - y_clamped));
    float y_pixel = dot(color_1, vec3(0.2627, 0.678, 0.0593));
    if (!((archetype == 6))) {
        local_48 = (archetype == 10);
    } else {
        local_48 = true;
    }
    bool _e105 = local_48;
    if (!(_e105)) {
        local_49 = (archetype == 11);
    } else {
        local_49 = true;
    }
    bool is_monochrome = local_49;
    if (is_monochrome) {
        float delta_2 = (((_e83 * nutting) * 0.28) * amount_1);
        float y_res = max((y_pixel * (1.0 + delta_2)), 0.0);
        return vec3(y_res);
    } else {
        float _e127 = evalGrainProcNoise((p_1 + vec2(17.3, 31.7)), s_3, is_instant_2);
        float _e132 = evalGrainProcNoise((p_1 + vec2(73.1, 19.9)), s_3, is_instant_2);
        float _e137 = evalGrainProcNoise((p_1 + vec2(43.7, 89.3)), s_3, is_instant_2);
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
        float r_c = (y_pixel + (chroma_damp * (color_1.x - y_pixel)));
        float g_c = (y_pixel + (chroma_damp * (color_1.y - y_pixel)));
        float b_c = (y_pixel + (chroma_damp * (color_1.z - y_pixel)));
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
    float delta_3 = (maxV - minV);
    if ((delta_3 > 1e-5)) {
        if ((maxV == c_6.x)) {
            h = ((c_6.y - c_6.z) / delta_3);
            float _e22 = h;
            if ((_e22 < 0.0)) {
                float _e25 = h;
                h = (_e25 + 6.0);
            }
        } else {
            if ((maxV == c_6.y)) {
                h = (((c_6.z - c_6.x) / delta_3) + 2.0);
            } else {
                h = (((c_6.x - c_6.y) / delta_3) + 4.0);
            }
        }
        float _e42 = h;
        h = (_e42 * 60.0);
    }
    if ((maxV > 1e-5)) {
        s_1 = (delta_3 / maxV);
    }
    float _e50 = h;
    float _e51 = s_1;
    return vec3(_e50, _e51, maxV);
}

vec3 hsv2rgb(vec3 c_7) {
    vec3 rgb = vec3(0.0);
    float h_1 = (c_7.x / 60.0);
    float s_4 = c_7.y;
    float v_1 = c_7.z;
    float C = (v_1 * s_4);
    float _e8 = float_mod(h_1, 2.0);
    float X = (C * (1.0 - abs((_e8 - 1.0))));
    float m_2 = (v_1 - C);
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
    return (_e41 + vec3(m_2));
}

float evalCameraHermite7_(float x_1, vec2 knots[7], float m[7]) {
    int k = 0;
    if ((x_1 <= knots[0].x)) {
        return knots[0].y;
    }
    if ((x_1 >= knots[6].x)) {
        return knots[6].y;
    }
    bool loop_init_1 = true;
    while(true) {
        if (!loop_init_1) {
            int _e83 = k;
            k = (_e83 + 1);
        }
        loop_init_1 = false;
        int _e15 = k;
        if ((_e15 < 6)) {
        } else {
            break;
        }
        {
            int _e18 = k;
            if ((x_1 <= knots[(_e18 + 1)].x)) {
                int _e24 = k;
                int _e29 = k;
                float dx = (knots[(_e24 + 1)].x - knots[_e29].x);
                int _e33 = k;
                float t = ((x_1 - knots[_e33].x) / dx);
                float t2_ = (t * t);
                float t3_ = (t2_ * t);
                float h00_ = (((2.0 * t3_) - (3.0 * t2_)) + 1.0);
                float h10_ = ((t3_ - (2.0 * t2_)) + t);
                float h01_ = ((-2.0 * t3_) + (3.0 * t2_));
                float h11_ = (t3_ - t2_);
                int _e57 = k;
                int _e61 = k;
                int _e66 = k;
                int _e73 = k;
                return clamp(((((h00_ * knots[_e57].y) + ((h10_ * m[_e61]) * dx)) + (h01_ * knots[(_e66 + 1)].y)) + ((h11_ * m[(_e73 + 1)]) * dx)), 0.0, 1.0);
            }
        }
    }
    return x_1;
}

float applyCameraToneCurve(float x_2, int mode) {
    float xc = clamp(x_2, 0.0, 1.0);
    if ((mode == 2)) {
        float _e9 = evalCameraHermite7_(xc, DCP_KNOTS_PORTRAIT, DCP_TANGENTS_PORTRAIT);
        return _e9;
    } else {
        if ((mode == 3)) {
            float _e14 = evalCameraHermite7_(xc, DCP_KNOTS_LANDSCAPE, DCP_TANGENTS_LANDSCAPE);
            return _e14;
        } else {
            if ((mode == 4)) {
                float _e19 = evalCameraHermite7_(xc, DCP_KNOTS_VIVID, DCP_TANGENTS_VIVID);
                return _e19;
            } else {
                float _e22 = evalCameraHermite7_(xc, DCP_KNOTS_STANDARD, DCP_TANGENTS_STANDARD);
                return _e22;
            }
        }
    }
}

vec3 evalHueSatMapDelta(float h_deg, float s_2, vec3 table[24]) {
    float h_norm = 0.0;
    h_norm = fract((h_deg / 360.0));
    float _e7 = h_norm;
    if ((_e7 < 0.0)) {
        float _e10 = h_norm;
        h_norm = (_e10 + 1.0);
    }
    float _e13 = h_norm;
    float h_coord = (_e13 * 12.0);
    int h0_ = (int(floor(h_coord)) % 12);
    int h1_ = ((h0_ + 1) % 12);
    float h_frac = (h_coord - floor(h_coord));
    float s_clamped = clamp(s_2, 0.0, 1.0);
    vec3 c00_ = table[((h0_ * 2) + 0)];
    vec3 c01_ = table[((h0_ * 2) + 1)];
    vec3 c10_ = table[((h1_ * 2) + 0)];
    vec3 c11_ = table[((h1_ * 2) + 1)];
    vec3 c0_ = mix(c00_, c01_, s_clamped);
    vec3 c1_ = mix(c10_, c11_, s_clamped);
    return mix(c0_, c1_, h_frac);
}

vec3 applyCameraHueSatMap(vec3 c_srgb, int mode_1) {
    vec3 delta = vec3(0.0);
    if ((max(max(c_srgb.x, c_srgb.y), c_srgb.z) <= 1e-6)) {
        return c_srgb;
    }
    vec3 _e9 = rgb2hsv(c_srgb);
    if ((_e9.y <= 1e-6)) {
        return c_srgb;
    }
    if ((mode_1 == 2)) {
        vec3 _e21 = evalHueSatMapDelta(_e9.x, _e9.y, DCP_HSM_PORTRAIT);
        delta = _e21;
    } else {
        if ((mode_1 == 3)) {
            vec3 _e27 = evalHueSatMapDelta(_e9.x, _e9.y, DCP_HSM_LANDSCAPE);
            delta = _e27;
        } else {
            if ((mode_1 == 4)) {
                vec3 _e33 = evalHueSatMapDelta(_e9.x, _e9.y, DCP_HSM_VIVID);
                delta = _e33;
            } else {
                vec3 _e37 = evalHueSatMapDelta(_e9.x, _e9.y, DCP_HSM_STANDARD);
                delta = _e37;
            }
        }
    }
    float _e40 = delta.x;
    float _e43 = float_mod((_e9.x + _e40), 360.0);
    float _e47 = float_mod((_e43 + 360.0), 360.0);
    float _e50 = delta.y;
    float s_out = clamp((_e9.y * _e50), 0.0, 1.0);
    float _e57 = delta.z;
    float v_out = max((_e9.z * _e57), 0.0);
    vec3 _e62 = hsv2rgb(vec3(_e47, s_out, v_out));
    return _e62;
}

float agxApplyCurveChannel(float x_3) {
    float xc_1 = clamp(x_3, 0.0, 1.0);
    float curved = (((((((((((15.45672 * xc_1) - 40.02792) * xc_1) + 31.87076) * xc_1) - 6.84882) * xc_1) + 0.4286) * xc_1) + 0.12066) * xc_1);
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
    float t_1 = clamp(((lum - 0.65) / 0.35), 0.0, 1.0);
    float h_2 = ((t_1 * t_1) * (3.0 - (2.0 * t_1)));
    float sat = (1.25 - (0.25 * h_2));
    return clamp(vec3((lum + (sat * (_e4.x - lum))), (lum + (sat * (_e4.y - lum))), (lum + (sat * (_e4.z - lum)))), vec3(0.0), vec3(1.0));
}

vec3 agxToneMapPhotographic(vec3 rec2020_color_2, float contrast_gain_2) {
    vec3 compressed = vec3(0.0);
    float norm_1 = 0.0;
    float gain = 1.0;
    float minC_1 = min(rec2020_color_2.x, min(rec2020_color_2.y, rec2020_color_2.z));
    compressed = rec2020_color_2;
    if ((minC_1 < 0.0)) {
        compressed = (rec2020_color_2 - vec3(minC_1));
    }
    float photoContrast = (contrast_gain_2 * 1.28);
    float _e15 = compressed.x;
    float _e19 = compressed.y;
    float _e24 = compressed.z;
    float lumRec = (((0.2627 * _e15) + (0.678 * _e19)) + (0.0593 * _e24));
    float xRel_1 = max((lumRec / 0.18), 1e-6);
    float logEncoded_1 = ((log2(xRel_1) - AGX_MIN_EV) / AGX_EV_RANGE);
    norm_1 = clamp(logEncoded_1, 0.0, 1.0);
    if ((abs((photoContrast - 1.0)) > 0.001)) {
        float _e47 = norm_1;
        norm_1 = clamp((AGX_MID_GRAY_NORM + ((_e47 - AGX_MID_GRAY_NORM) * photoContrast)), 0.0, 1.0);
    }
    float _e55 = norm_1;
    float _e56 = agxApplyCurveChannel(_e55);
    float lumMapped = pow(max(_e56, 0.0), AGX_GAMMA);
    if ((lumRec > 1e-6)) {
        gain = (lumMapped / lumRec);
    }
    vec3 _e66 = compressed;
    float _e67 = gain;
    vec3 cScaled = (_e66 * _e67);
    vec3 sRatio = (REC2020_TO_LINEAR_SRGB * cScaled);
    vec3 _e71 = compressed;
    vec3 _e72 = agxToneMap(_e71, photoContrast);
    float peakVal = max(lumMapped, max(max(sRatio.x, 0.0), max(max(sRatio.y, 0.0), max(sRatio.z, 0.0))));
    float t_2 = clamp(((peakVal - 0.85) / 0.2), 0.0, 1.0);
    float h_3 = ((t_2 * t_2) * (3.0 - (2.0 * t_2)));
    return clamp(vec3(((max(sRatio.x, 0.0) * (1.0 - h_3)) + (_e72.x * h_3)), ((max(sRatio.y, 0.0) * (1.0 - h_3)) + (_e72.y * h_3)), ((max(sRatio.z, 0.0) * (1.0 - h_3)) + (_e72.z * h_3))), vec3(0.0), vec3(1.0));
}

vec3 applyBilateralToneRecovery(vec3 color_2, float y_c_2, float y_neighbors_1[16], float highlights, float shadows, float whites, float blacks) {
    bool local_50 = false;
    bool local_51 = false;
    bool local_52 = false;
    float total_weight = 1.0;
    float weighted_ev_sum = 0.0;
    int i_3 = 0;
    int i_4 = 4;
    int i_5 = 8;
    int i_6 = 12;
    float delta_ev = 0.0;
    if ((highlights == 0.0)) {
        local_50 = (shadows == 0.0);
    } else {
        local_50 = false;
    }
    bool _e14 = local_50;
    if (_e14) {
        local_51 = (whites == 0.0);
    } else {
        local_51 = false;
    }
    bool _e20 = local_51;
    if (_e20) {
        local_52 = (blacks == 0.0);
    } else {
        local_52 = false;
    }
    bool _e26 = local_52;
    if (_e26) {
        return color_2;
    }
    if ((y_c_2 <= 1e-6)) {
        return color_2;
    }
    float ev_c = log2((max(y_c_2, 1e-5) / 0.18));
    weighted_ev_sum = ev_c;
    bool loop_init_2 = true;
    while(true) {
        if (!loop_init_2) {
            int _e63 = i_3;
            i_3 = (_e63 + 1);
        }
        loop_init_2 = false;
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
    bool loop_init_3 = true;
    while(true) {
        if (!loop_init_3) {
            int _e89 = i_4;
            i_4 = (_e89 + 1);
        }
        loop_init_3 = false;
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
    bool loop_init_4 = true;
    while(true) {
        if (!loop_init_4) {
            int _e115 = i_5;
            i_5 = (_e115 + 1);
        }
        loop_init_4 = false;
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
    bool loop_init_5 = true;
    while(true) {
        if (!loop_init_5) {
            int _e141 = i_6;
            i_6 = (_e141 + 1);
        }
        loop_init_5 = false;
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
        float u_2 = clamp(((-(ev_base) - 1.0) / 3.0), 0.0, 1.0);
        float w_blacks = ((u_2 * u_2) * (3.0 - (2.0 * u_2)));
        float _e165 = delta_ev;
        delta_ev = (_e165 + (((blacks / 100.0) * 1.2) * w_blacks));
    }
    if ((shadows != 0.0)) {
        float u_3 = clamp((-(ev_base) / 3.0), 0.0, 1.0);
        float w_shadows = ((u_3 * u_3) * (3.0 - (2.0 * u_3)));
        float _e186 = delta_ev;
        delta_ev = (_e186 + (((shadows / 100.0) * 2.25) * w_shadows));
    }
    if ((highlights != 0.0)) {
        float u_4 = clamp((ev_base / 3.0), 0.0, 1.0);
        float w_highlights = ((u_4 * u_4) * (3.0 - (2.0 * u_4)));
        float _e206 = delta_ev;
        delta_ev = (_e206 + (((highlights / 100.0) * 2.25) * w_highlights));
    }
    if ((whites != 0.0)) {
        float u_5 = clamp(((ev_base - 1.0) / 3.0), 0.0, 1.0);
        float w_whites = ((u_5 * u_5) * (3.0 - (2.0 * u_5)));
        float _e228 = delta_ev;
        delta_ev = (_e228 + (((whites / 100.0) * 1.2) * w_whites));
    }
    float _e235 = delta_ev;
    if ((_e235 == 0.0)) {
        return color_2;
    }
    float detail = clamp((ev_c - ev_base), -3.0, 3.0);
    float _e244 = delta_ev;
    float detail_boost = ((0.35 * detail) * (abs(_e244) / 2.25));
    float _e249 = delta_ev;
    float gain_1 = exp2(clamp((_e249 + detail_boost), -6.0, 6.0));
    return (color_2 * gain_1);
}

vec3 applyColorAdjustments(vec3 rgb_1, float vibrance, float saturation, float deepBlue, float foliage) {
    bool local_53 = false;
    bool local_54 = false;
    bool local_55 = false;
    vec3 c_8 = vec3(0.0);
    float skinProtect = 1.0;
    float vibGain = 0.0;
    if ((vibrance == 0.0)) {
        local_53 = (saturation == 0.0);
    } else {
        local_53 = false;
    }
    bool _e12 = local_53;
    if (_e12) {
        local_54 = (deepBlue == 0.0);
    } else {
        local_54 = false;
    }
    bool _e18 = local_54;
    if (_e18) {
        local_55 = (foliage == 0.0);
    } else {
        local_55 = false;
    }
    bool _e24 = local_55;
    if (_e24) {
        return rgb_1;
    }
    c_8 = clamp(rgb_1, vec3(0.0), vec3(1.0));
    float _e32 = c_8.x;
    float _e36 = c_8.y;
    float _e41 = c_8.z;
    float lum_1 = (((0.2627 * _e32) + (0.678 * _e36)) + (0.0593 * _e41));
    if ((saturation != 0.0)) {
        float satGain = max((1.0 + (saturation / 100.0)), 0.0);
        vec3 _e54 = c_8;
        c_8 = (vec3(lum_1) + ((_e54 - vec3(lum_1)) * satGain));
    }
    if ((vibrance != 0.0)) {
        float _e62 = c_8.x;
        float _e64 = c_8.y;
        float _e67 = c_8.z;
        float maxC = max(max(_e62, _e64), _e67);
        float _e70 = c_8.x;
        float _e72 = c_8.y;
        float _e75 = c_8.z;
        float minC_2 = min(min(_e70, _e72), _e75);
        float chroma = (maxC - minC_2);
        float _e81 = c_8.x;
        float _e83 = c_8.y;
        float _e86 = c_8.z;
        float sum = ((_e81 + _e83) + _e86);
        if ((sum > 1e-5)) {
            float _e91 = c_8.x;
            float rN = (_e91 / sum);
            float _e94 = c_8.y;
            float gN = (_e94 / sum);
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
            float _e189 = skinProtect;
            vibGain = ((((vibrance / 100.0) * (1.0 - chroma)) * 0.8) * _e189);
        }
        vec3 _e191 = c_8;
        vec3 _e192 = c_8;
        float _e195 = vibGain;
        c_8 = (_e191 + ((_e192 - vec3(lum_1)) * _e195));
    }
    if ((deepBlue != 0.0)) {
        float _e201 = c_8.x;
        float _e203 = c_8.y;
        float maxWarm = max(_e201, _e203);
        float _e206 = c_8.z;
        float skySignal = max((_e206 - maxWarm), 0.0);
        if ((skySignal > 0.0)) {
            float x_5 = (deepBlue / 100.0);
            float factor = (sign(x_5) * pow(abs(x_5), 1.6));
            float _e221 = c_8.x;
            c_8.x = (_e221 - ((factor * skySignal) * 0.28));
            float _e228 = c_8.y;
            c_8.y = (_e228 - ((factor * skySignal) * 0.18));
            float _e235 = c_8.z;
            c_8.z = (_e235 - ((factor * skySignal) * 0.15));
        }
    }
    if ((foliage != 0.0)) {
        float _e243 = c_8.x;
        float _e245 = c_8.z;
        float maxRb = max(_e243, _e245);
        float _e248 = c_8.y;
        float greenSignal = max((_e248 - maxRb), 0.0);
        if ((greenSignal > 0.0)) {
            float x_6 = (foliage / 100.0);
            float factor_1 = (sign(x_6) * pow(abs(x_6), 1.6));
            float _e263 = c_8.x;
            c_8.x = (_e263 - ((factor_1 * greenSignal) * 0.3));
            float _e270 = c_8.z;
            c_8.z = (_e270 - ((factor_1 * greenSignal) * 0.15));
            float _e277 = c_8.y;
            c_8.y = (_e277 + ((factor_1 * greenSignal) * 0.1));
        }
    }
    vec3 _e282 = c_8;
    return clamp(_e282, vec3(0.0), vec3(1.0));
}

vec3 applySplitToning(vec3 rgb_2, float shadowHue, float shadowSat, float highlightHue, float highlightSat, float balance) {
    bool local_56 = false;
    vec3 delta_1 = vec3(0.0);
    bool local_57 = false;
    bool local_58 = false;
    if ((shadowSat <= 0.0)) {
        local_56 = (highlightSat <= 0.0);
    } else {
        local_56 = false;
    }
    bool _e13 = local_56;
    if (_e13) {
        return rgb_2;
    }
    float lum_2 = (((0.2627 * rgb_2.x) + (0.678 * rgb_2.y)) + (0.0593 * rgb_2.z));
    float yClamp = clamp(lum_2, 0.0, 1.0);
    float t_3 = clamp((0.5 - ((balance / 100.0) * 0.25)), 0.1, 0.9);
    if ((shadowSat > 0.0)) {
        local_57 = (yClamp < t_3);
    } else {
        local_57 = false;
    }
    bool _e46 = local_57;
    if (_e46) {
        float u_s = ((t_3 - yClamp) / t_3);
        float w_s = (((u_s * u_s) * (3.0 - (2.0 * u_s))) * (shadowSat / 100.0));
        vec3 _e61 = hsv2rgb(vec3(shadowHue, 1.0, 1.0));
        float tLum = (((0.2627 * _e61.x) + (0.678 * _e61.y)) + (0.0593 * _e61.z));
        vec3 _e73 = delta_1;
        delta_1 = (_e73 + (vec3(w_s) * (_e61 - vec3(tLum))));
    }
    if ((highlightSat > 0.0)) {
        local_58 = (yClamp > t_3);
    } else {
        local_58 = false;
    }
    bool _e85 = local_58;
    if (_e85) {
        float u_h = ((yClamp - t_3) / (1.0 - t_3));
        float w_h = (((u_h * u_h) * (3.0 - (2.0 * u_h))) * (highlightSat / 100.0));
        vec3 _e102 = hsv2rgb(vec3(highlightHue, 1.0, 1.0));
        float tLum_1 = (((0.2627 * _e102.x) + (0.678 * _e102.y)) + (0.0593 * _e102.z));
        vec3 _e114 = delta_1;
        delta_1 = (_e114 + (vec3(w_h) * (_e102 - vec3(tLum_1))));
    }
    vec3 _e120 = delta_1;
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
    float v_2 = clamp(val_1, 0.0, 2.0);
    base = v_2;
    if ((abs((gamma - 1.0)) > 0.001)) {
        float u_6 = clamp((v_2 / (v_2 + 0.35)), 0.0, 1.0);
        float gMod = ((gamma - 1.0) * 0.4);
        base = max((v_2 + (((gMod * u_6) * (1.0 - u_6)) * ((2.0 * u_6) - 1.0))), 0.0);
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

float applyEmulsionSCurve(float x_4, float alpha, float beta) {
    float c_12 = clamp(x_4, 0.0, 1.0);
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
    bool local_28 = false;
    float baseTint = 0.0;
    bool local_29 = false;
    float exposureGain = 1.0;
    bool local_30 = false;
    bool local_31 = false;
    bool local_32 = false;
    float y_neighbors[16] = float[16](0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    int i = 0;
    vec3 c = vec3(0.0);
    bool local_33 = false;
    bool local_34 = false;
    bool local_35 = false;
    float formatMag = 1.0;
    vec2 halationRadius = vec2(0.0);
    bool local_36 = false;
    float scatterAcc = 0.0;
    int i_1 = 0;
    vec3 c_1 = vec3(0.0);
    bool local_37 = false;
    float editSharpening = 0.0;
    float editNoiseReduction = 0.0;
    int editGrainMode = 0;
    bool local_38 = false;
    bool local_39 = false;
    float formatMag_1 = 1.0;
    bool local_40 = false;
    float y_l = 0.0;
    float y_r = 0.0;
    float y_t = 0.0;
    float y_b = 0.0;
    float y_tl = 0.0;
    float y_tr = 0.0;
    float y_bl = 0.0;
    float y_br = 0.0;
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
    bool local_41 = false;
    vec3 finalColor = vec3(0.0);
    bool local_42 = false;
    bool local_43 = false;
    bool local_44 = false;
    ShaderUniforms _e3 = _group_0_binding_0_fs;
    vec2 _e4 = transformGeometry(in_.texCoord, _e3);
    if (!((_e4.x < 0.0))) {
        local = (_e4.x > 1.0);
    } else {
        local = true;
    }
    bool _e15 = local;
    if (!(_e15)) {
        local_1 = (_e4.y < 0.0);
    } else {
        local_1 = true;
    }
    bool _e23 = local_1;
    if (!(_e23)) {
        local_2 = (_e4.y > 1.0);
    } else {
        local_2 = true;
    }
    bool _e31 = local_2;
    if (_e31) {
        _fs2p_location0 = vec4(0.08, 0.08, 0.08, 1.0);
        return;
    }
    vec4 sampledColor = texture(_group_0_binding_1_fs, vec2(_e4));
    int _e42 = _group_0_binding_0_fs.isIntermediate;
    if ((_e42 == 1)) {
        _fs2p_location0 = sampledColor;
        return;
    }
    float _e47 = _group_0_binding_0_fs.isOriginal;
    if ((_e47 > 0.5)) {
        int _e54 = _group_0_binding_0_fs.isLinearRaw;
        local_3 = (_e54 == 0);
    } else {
        local_3 = false;
    }
    bool _e58 = local_3;
    if (_e58) {
        _fs2p_location0 = sampledColor;
        return;
    }
    float _e61 = _group_0_binding_0_fs.exposure;
    if ((abs(_e61) < 0.001)) {
        float _e69 = _group_0_binding_0_fs.temperature;
        local_4 = (abs((_e69 - 5500.0)) < 1.0);
    } else {
        local_4 = false;
    }
    bool _e76 = local_4;
    if (_e76) {
        float _e81 = _group_0_binding_0_fs.tint;
        local_5 = (abs(_e81) < 0.001);
    } else {
        local_5 = false;
    }
    bool _e86 = local_5;
    if (_e86) {
        float _e91 = _group_0_binding_0_fs.contrast;
        local_6 = (abs(_e91) < 0.001);
    } else {
        local_6 = false;
    }
    bool _e96 = local_6;
    if (_e96) {
        float _e101 = _group_0_binding_0_fs.highlights;
        local_7 = (abs(_e101) < 0.001);
    } else {
        local_7 = false;
    }
    bool _e106 = local_7;
    if (_e106) {
        float _e111 = _group_0_binding_0_fs.shadows;
        local_8 = (abs(_e111) < 0.001);
    } else {
        local_8 = false;
    }
    bool _e116 = local_8;
    if (_e116) {
        float _e121 = _group_0_binding_0_fs.whites;
        local_9 = (abs(_e121) < 0.001);
    } else {
        local_9 = false;
    }
    bool _e126 = local_9;
    if (_e126) {
        float _e131 = _group_0_binding_0_fs.blacks;
        local_10 = (abs(_e131) < 0.001);
    } else {
        local_10 = false;
    }
    bool _e136 = local_10;
    if (_e136) {
        float _e141 = _group_0_binding_0_fs.vibrance;
        local_11 = (abs(_e141) < 0.001);
    } else {
        local_11 = false;
    }
    bool _e146 = local_11;
    if (_e146) {
        float _e151 = _group_0_binding_0_fs.saturation;
        local_12 = (abs(_e151) < 0.001);
    } else {
        local_12 = false;
    }
    bool _e156 = local_12;
    if (_e156) {
        float _e161 = _group_0_binding_0_fs.deepBlue;
        local_13 = (abs(_e161) < 0.001);
    } else {
        local_13 = false;
    }
    bool _e166 = local_13;
    if (_e166) {
        float _e171 = _group_0_binding_0_fs.foliage;
        local_14 = (abs(_e171) < 0.001);
    } else {
        local_14 = false;
    }
    bool _e176 = local_14;
    if (_e176) {
        float _e181 = _group_0_binding_0_fs.splitToningShadowSat;
        local_15 = (abs(_e181) < 0.001);
    } else {
        local_15 = false;
    }
    bool _e186 = local_15;
    if (_e186) {
        float _e191 = _group_0_binding_0_fs.splitToningHighlightSat;
        local_16 = (abs(_e191) < 0.001);
    } else {
        local_16 = false;
    }
    bool _e196 = local_16;
    if (_e196) {
        int _e201 = _group_0_binding_0_fs.lutEnabled;
        local_17 = (_e201 == 0);
    } else {
        local_17 = false;
    }
    bool _e205 = local_17;
    if (_e205) {
        float _e210 = _group_0_binding_0_fs.sharpeningAmount;
        local_18 = (_e210 == 0.0);
    } else {
        local_18 = false;
    }
    bool _e214 = local_18;
    if (_e214) {
        float _e219 = _group_0_binding_0_fs.noiseReduction;
        local_19 = (_e219 == 0.0);
    } else {
        local_19 = false;
    }
    bool _e223 = local_19;
    if (_e223) {
        int _e228 = _group_0_binding_0_fs.emulsionArchetype;
        local_20 = (_e228 == 0);
    } else {
        local_20 = false;
    }
    bool _e232 = local_20;
    if (_e232) {
        int _e237 = _group_0_binding_0_fs.halationMode;
        local_21 = (_e237 == 0);
    } else {
        local_21 = false;
    }
    bool _e241 = local_21;
    if (_e241) {
        int _e246 = _group_0_binding_0_fs.emulsionGrainMode;
        local_22 = (_e246 == 0);
    } else {
        local_22 = false;
    }
    bool _e250 = local_22;
    if (_e250) {
        int _e255 = _group_0_binding_0_fs.printMediaMode;
        local_23 = (_e255 == 0);
    } else {
        local_23 = false;
    }
    bool isNeutralEdits = local_23;
    int _e262 = _group_0_binding_0_fs.isLinearRaw;
    if ((_e262 == 0)) {
        local_24 = isNeutralEdits;
    } else {
        local_24 = false;
    }
    bool _e268 = local_24;
    if (_e268) {
        int _e273 = _group_0_binding_0_fs.hasGainMap;
        local_25 = (_e273 == 0);
    } else {
        local_25 = false;
    }
    bool _e277 = local_25;
    if (_e277) {
        int _e282 = _group_0_binding_0_fs.edrEnabled;
        if (!((_e282 == 0))) {
            float _e290 = _group_0_binding_0_fs.edrHeadroom;
            local_27 = (_e290 <= 1.001);
        } else {
            local_27 = true;
        }
        bool _e294 = local_27;
        local_26 = _e294;
    } else {
        local_26 = false;
    }
    bool _e296 = local_26;
    if (_e296) {
        _fs2p_location0 = sampledColor;
        return;
    }
    float _e299 = _group_0_binding_0_fs.exposure;
    editExposure = _e299;
    float _e303 = _group_0_binding_0_fs.temperature;
    editTemp = _e303;
    float _e307 = _group_0_binding_0_fs.tint;
    editTint = _e307;
    float _e311 = _group_0_binding_0_fs.contrast;
    editContrast = _e311;
    float _e315 = _group_0_binding_0_fs.highlights;
    editHighlights = _e315;
    float _e319 = _group_0_binding_0_fs.shadows;
    editShadows = _e319;
    float _e323 = _group_0_binding_0_fs.whites;
    editWhites = _e323;
    float _e327 = _group_0_binding_0_fs.blacks;
    editBlacks = _e327;
    float _e331 = _group_0_binding_0_fs.vibrance;
    editVibrance = _e331;
    float _e335 = _group_0_binding_0_fs.saturation;
    editSaturation = _e335;
    float _e339 = _group_0_binding_0_fs.deepBlue;
    editDeepBlue = _e339;
    float _e343 = _group_0_binding_0_fs.foliage;
    editFoliage = _e343;
    float _e347 = _group_0_binding_0_fs.splitToningShadowSat;
    editSplitShadowSat = _e347;
    float _e351 = _group_0_binding_0_fs.splitToningHighlightSat;
    editSplitHighlightSat = _e351;
    float _e355 = _group_0_binding_0_fs.isOriginal;
    if ((_e355 > 0.5)) {
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
    int _e375 = _group_0_binding_0_fs.isLinearRaw;
    if ((_e375 == 1)) {
        rec2020Color = sampledColor.xyz;
    } else {
        vec3 _e380 = srgbGammaToLinear3_(sampledColor.xyz);
        rec2020Color = (LINEAR_SRGB_TO_REC2020_ * _e380);
    }
    int _e393 = _group_0_binding_0_fs.isLinearRaw;
    if ((_e393 == 1)) {
        float _e400 = _group_0_binding_0_fs.asShotTemperature;
        local_28 = (_e400 > 1000.0);
    } else {
        local_28 = false;
    }
    bool _e404 = local_28;
    if (_e404) {
        float _e407 = _group_0_binding_0_fs.asShotTemperature;
        baseTemp = _e407;
    }
    int _e412 = _group_0_binding_0_fs.isLinearRaw;
    if ((_e412 == 1)) {
        float _e417 = _group_0_binding_0_fs.asShotTint;
        baseTint = _e417;
    }
    float _e418 = editTemp;
    float _e419 = baseTemp;
    if (!((abs((_e418 - _e419)) >= 1.0))) {
        float _e427 = editTint;
        float _e428 = baseTint;
        local_29 = (abs((_e427 - _e428)) >= 0.1);
    } else {
        local_29 = true;
    }
    bool _e434 = local_29;
    if (_e434) {
        float _e435 = editTemp;
        float _e436 = baseTemp;
        float tempNorm = clamp((_e435 / _e436), 0.25, 4.0);
        rScale = pow(tempNorm, 0.75);
        bScale = pow((1.0 / tempNorm), 0.75);
        float _e447 = editTint;
        float _e448 = baseTint;
        float tintFactor = ((_e447 - _e448) * 0.002);
        gScale = max((1.0 - tintFactor), 0.01);
        float _e456 = rScale;
        rScale = max((_e456 + (tintFactor * 0.5)), 0.01);
        float _e462 = bScale;
        bScale = max((_e462 + (tintFactor * 0.5)), 0.01);
        float _e470 = rec2020Color.x;
        float _e471 = rScale;
        rec2020Color.x = (_e470 * _e471);
        float _e475 = rec2020Color.y;
        float _e476 = gScale;
        rec2020Color.y = (_e475 * _e476);
        float _e480 = rec2020Color.z;
        float _e481 = bScale;
        rec2020Color.z = (_e480 * _e481);
    }
    float totalExposure = editExposure;
    if ((totalExposure != 0.0)) {
        exposureGain = pow(2.0, totalExposure);
        vec3 _e490 = rec2020Color;
        float _e491 = exposureGain;
        rec2020Color = (_e490 * _e491);
    }
    float _e493 = editHighlights;
    if (!((_e493 != 0.0))) {
        float _e499 = editShadows;
        local_30 = (_e499 != 0.0);
    } else {
        local_30 = true;
    }
    bool _e503 = local_30;
    if (!(_e503)) {
        float _e507 = editWhites;
        local_31 = (_e507 != 0.0);
    } else {
        local_31 = true;
    }
    bool _e511 = local_31;
    if (!(_e511)) {
        float _e515 = editBlacks;
        local_32 = (_e515 != 0.0);
    } else {
        local_32 = true;
    }
    bool _e519 = local_32;
    if (_e519) {
        vec2 texDim = vec2(uvec2(textureSize(_group_0_binding_1_fs, 0).xy));
        vec2 texelSize = (vec2(1.0) / texDim);
        float maxDim = max(texDim.x, texDim.y);
        float rInner = max(3.0, roundEven((maxDim * 0.003)));
        float rOuter = max(24.0, roundEven((maxDim * 0.024)));
        vec2 stepInner = (texelSize * rInner);
        vec2 stepOuter = (texelSize * rOuter);
        vec3 _e541 = rec2020Color;
        float y_c_3 = dot(_e541, vec3(0.2627, 0.678, 0.0593));
        float _e547 = rScale;
        float _e548 = gScale;
        float _e549 = bScale;
        float _e551 = exposureGain;
        vec3 wbExposureScale = (vec3(_e547, _e548, _e549) * _e551);
        vec4 s_l = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(-(stepInner.x), 0.0))));
        vec4 s_r = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(stepInner.x, 0.0))));
        vec4 s_t = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(0.0, -(stepInner.y)))));
        vec4 s_b = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(0.0, stepInner.y))));
        vec4 s_tl = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(-(stepInner.x), -(stepInner.y)))));
        vec4 s_tr = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(stepInner.x, -(stepInner.y)))));
        vec4 s_bl = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(-(stepInner.x), stepInner.y))));
        vec4 s_br = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(stepInner.x, stepInner.y))));
        vec4 s_ol = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(-(stepOuter.x), 0.0))));
        vec4 s_or = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(stepOuter.x, 0.0))));
        vec4 s_ot = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(0.0, -(stepOuter.y)))));
        vec4 s_ob = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(0.0, stepOuter.y))));
        vec4 s_otl = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(-(stepOuter.x), -(stepOuter.y)))));
        vec4 s_otr = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(stepOuter.x, -(stepOuter.y)))));
        vec4 s_obl = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(-(stepOuter.x), stepOuter.y))));
        vec4 s_obr = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(stepOuter.x, stepOuter.y))));
        vec4 s_neighbors[16] = vec4[16](s_l, s_r, s_t, s_b, s_tl, s_tr, s_bl, s_br, s_ol, s_or, s_ot, s_ob, s_otl, s_otr, s_obl, s_obr);
        bool loop_init_6 = true;
        while(true) {
            if (!loop_init_6) {
                int _e727 = i;
                i = (_e727 + 1);
            }
            loop_init_6 = false;
            int _e698 = i;
            if ((_e698 < 16)) {
            } else {
                break;
            }
            {
                c = vec3(0.0);
                int _e705 = _group_0_binding_0_fs.isLinearRaw;
                if ((_e705 == 1)) {
                    int _e708 = i;
                    c = s_neighbors[_e708].xyz;
                } else {
                    int _e711 = i;
                    vec3 _e714 = srgbGammaToLinear3_(s_neighbors[_e711].xyz);
                    c = (LINEAR_SRGB_TO_REC2020_ * _e714);
                }
                vec3 _e717 = c;
                c = (_e717 * wbExposureScale);
                int _e719 = i;
                vec3 _e721 = c;
                y_neighbors[_e719] = dot(_e721, vec3(0.2627, 0.678, 0.0593));
            }
        }
        vec3 _e730 = rec2020Color;
        float _e731[16] = y_neighbors;
        float _e732 = editHighlights;
        float _e733 = editShadows;
        float _e734 = editWhites;
        float _e735 = editBlacks;
        vec3 _e736 = applyBilateralToneRecovery(_e730, y_c_3, _e731, _e732, _e733, _e734, _e735);
        rec2020Color = _e736;
    }
    float _e739 = _group_0_binding_0_fs.isOriginal;
    if ((_e739 < 0.5)) {
        int _e746 = _group_0_binding_0_fs.emulsionArchetype;
        local_33 = (_e746 != 0);
    } else {
        local_33 = false;
    }
    bool _e750 = local_33;
    if (_e750) {
        vec3 _e751 = rec2020Color;
        int _e754 = _group_0_binding_0_fs.emulsionArchetype;
        float _e757 = _group_0_binding_0_fs.emulsionExposure;
        int _e760 = _group_0_binding_0_fs.enlargerFilterMode;
        vec3 _e761 = applyRecordingEmulsion(_e751, _e754, _e757, _e760);
        rec2020Color = _e761;
    }
    float _e764 = _group_0_binding_0_fs.isOriginal;
    if ((_e764 < 0.5)) {
        int _e771 = _group_0_binding_0_fs.halationMode;
        local_34 = (_e771 > 0);
    } else {
        local_34 = false;
    }
    bool _e775 = local_34;
    if (_e775) {
        int _e780 = _group_0_binding_0_fs.emulsionArchetype;
        local_35 = (_e780 != 7);
    } else {
        local_35 = false;
    }
    bool _e784 = local_35;
    if (_e784) {
        int _e789 = _group_0_binding_0_fs.emulsionFormat;
        if ((_e789 == 1)) {
            formatMag = 0.48;
        } else {
            int _e795 = _group_0_binding_0_fs.emulsionFormat;
            if ((_e795 == 2)) {
                formatMag = 0.28;
            } else {
                int _e801 = _group_0_binding_0_fs.emulsionFormat;
                if ((_e801 == 3)) {
                    formatMag = 7.3;
                } else {
                    int _e807 = _group_0_binding_0_fs.emulsionFormat;
                    if ((_e807 == 4)) {
                        formatMag = 3.4;
                    }
                }
            }
        }
        float _e813 = _group_0_binding_0_fs.aspectRatio;
        float aspect = max(_e813, 0.01);
        int _e819 = _group_0_binding_0_fs.rotation;
        if (!((_e819 == 90))) {
            int _e827 = _group_0_binding_0_fs.rotation;
            local_36 = (_e827 == 270);
        } else {
            local_36 = true;
        }
        bool _e831 = local_36;
        if (_e831) {
            float _e834 = formatMag;
            float _e836 = formatMag;
            halationRadius = vec2(((0.012 * aspect) * _e834), (0.012 * _e836));
        } else {
            float _e842 = formatMag;
            float _e844 = formatMag;
            halationRadius = vec2(((0.012 / aspect) * _e842), (0.012 * _e844));
        }
        float _e848 = rScale;
        float _e849 = gScale;
        float _e850 = bScale;
        float _e852 = exposureGain;
        vec3 wbExposure = (vec3(_e848, _e849, _e850) * _e852);
        float _e856 = ign(in_.position.xy);
        float jitter = (_e856 * 6.2831855);
        float cosJ = cos(jitter);
        float sinJ = sin(jitter);
        mat2x2 rot_1 = mat2x2(vec2(cosJ, sinJ), vec2(-(sinJ), cosJ));
        bool loop_init_7 = true;
        while(true) {
            if (!loop_init_7) {
                int _e916 = i_1;
                i_1 = (_e916 + 1);
            }
            loop_init_7 = false;
            int _e869 = i_1;
            if ((_e869 < 16)) {
            } else {
                break;
            }
            {
                int _e873 = i_1;
                vec2 _e876 = halationRadius;
                vec2 rotatedOffset = ((rot_1 * FERMAT_DISK[_e873]) * _e876);
                vec4 s_5 = texture(_group_0_binding_1_fs, vec2((_e4 + rotatedOffset)));
                c_1 = vec3(0.0);
                int _e886 = _group_0_binding_0_fs.isLinearRaw;
                if ((_e886 == 1)) {
                    c_1 = s_5.xyz;
                } else {
                    vec3 _e891 = srgbGammaToLinear3_(s_5.xyz);
                    c_1 = (LINEAR_SRGB_TO_REC2020_ * _e891);
                }
                vec3 _e894 = c_1;
                c_1 = (_e894 * wbExposure);
                vec3 _e896 = c_1;
                float y_tap = dot(_e896, vec3(0.2627, 0.678, 0.0593));
                if ((y_tap > 0.55)) {
                    float diff = (y_tap - 0.55);
                    float excess = ((diff * diff) / (0.25 + diff));
                    float _e910 = scatterAcc;
                    int _e912 = i_1;
                    scatterAcc = (_e910 + (excess * FERMAT_WEIGHTS[_e912]));
                }
            }
        }
        float _e919 = scatterAcc;
        float halo = (_e919 / 3.6187);
        if ((halo > 1e-5)) {
            int _e926 = _group_0_binding_0_fs.halationMode;
            if ((_e926 == 1)) {
                vec3 _e929 = rec2020Color;
                rec2020Color = (_e929 + vec3((halo * 0.35)));
            } else {
                int _e936 = _group_0_binding_0_fs.halationMode;
                if ((_e936 == 2)) {
                    vec3 _e939 = rec2020Color;
                    rec2020Color = (_e939 + vec3((halo * 0.5), (halo * 0.12), (halo * 0.02)));
                }
            }
        }
    }
    int _e950 = _group_0_binding_0_fs.hasCurves;
    if ((_e950 != 0)) {
        float _e957 = _group_0_binding_0_fs.isOriginal;
        local_37 = (_e957 < 0.5);
    } else {
        local_37 = false;
    }
    bool _e961 = local_37;
    if (_e961) {
        int _e964 = _group_0_binding_0_fs.hasMasterCurve;
        if ((_e964 != 0)) {
            float _e971 = rec2020Color.x;
            vec4 _e975 = texture(_group_0_binding_5_fs, vec2(clamp(_e971, 0.0, 1.0), 0.0));
            rec2020Color.x = _e975.x;
            float _e981 = rec2020Color.y;
            vec4 _e985 = texture(_group_0_binding_5_fs, vec2(clamp(_e981, 0.0, 1.0), 0.0));
            rec2020Color.y = _e985.x;
            float _e991 = rec2020Color.z;
            vec4 _e995 = texture(_group_0_binding_5_fs, vec2(clamp(_e991, 0.0, 1.0), 0.0));
            rec2020Color.z = _e995.x;
        }
        int _e999 = _group_0_binding_0_fs.hasRgbCurves;
        if ((_e999 != 0)) {
            float _e1005 = rec2020Color.x;
            vec4 _e1009 = texture(_group_0_binding_6_fs, vec2(clamp(_e1005, 0.0, 1.0), 0.0));
            float _e1014 = rec2020Color.y;
            vec4 _e1018 = texture(_group_0_binding_6_fs, vec2(clamp(_e1014, 0.0, 1.0), 0.0));
            float _e1023 = rec2020Color.z;
            vec4 _e1027 = texture(_group_0_binding_6_fs, vec2(clamp(_e1023, 0.0, 1.0), 0.0));
            rec2020Color = vec3(_e1009.x, _e1018.y, _e1027.z);
        }
    }
    vec3 _e1030 = rec2020Color;
    float _e1031 = editVibrance;
    float _e1032 = editSaturation;
    float _e1033 = editDeepBlue;
    float _e1034 = editFoliage;
    vec3 _e1035 = applyColorAdjustments(_e1030, _e1031, _e1032, _e1033, _e1034);
    rec2020Color = _e1035;
    vec3 _e1036 = rec2020Color;
    float _e1039 = _group_0_binding_0_fs.splitToningShadowHue;
    float _e1040 = editSplitShadowSat;
    float _e1043 = _group_0_binding_0_fs.splitToningHighlightHue;
    float _e1044 = editSplitHighlightSat;
    float _e1047 = _group_0_binding_0_fs.splitToningBalance;
    vec3 _e1048 = applySplitToning(_e1036, _e1039, _e1040, _e1043, _e1044, _e1047);
    rec2020Color = _e1048;
    float _e1051 = _group_0_binding_0_fs.sharpeningAmount;
    editSharpening = _e1051;
    float _e1055 = _group_0_binding_0_fs.noiseReduction;
    editNoiseReduction = _e1055;
    int _e1059 = _group_0_binding_0_fs.emulsionGrainMode;
    editGrainMode = _e1059;
    float _e1063 = _group_0_binding_0_fs.isOriginal;
    if ((_e1063 > 0.5)) {
        editSharpening = 0.0;
        editNoiseReduction = 0.0;
        editGrainMode = 0;
    }
    float _e1069 = editSharpening;
    if (!((_e1069 > 0.0))) {
        float _e1075 = editNoiseReduction;
        local_38 = (_e1075 > 0.0);
    } else {
        local_38 = true;
    }
    bool _e1079 = local_38;
    if (!(_e1079)) {
        int _e1083 = editGrainMode;
        local_39 = (_e1083 != 0);
    } else {
        local_39 = true;
    }
    bool _e1087 = local_39;
    if (_e1087) {
        int _e1092 = _group_0_binding_0_fs.emulsionFormat;
        if ((_e1092 == 1)) {
            formatMag_1 = 0.48;
        } else {
            int _e1098 = _group_0_binding_0_fs.emulsionFormat;
            if ((_e1098 == 2)) {
                formatMag_1 = 0.28;
            } else {
                int _e1104 = _group_0_binding_0_fs.emulsionFormat;
                if ((_e1104 == 3)) {
                    formatMag_1 = 7.3;
                } else {
                    int _e1110 = _group_0_binding_0_fs.emulsionFormat;
                    if ((_e1110 == 4)) {
                        formatMag_1 = 3.4;
                    }
                }
            }
        }
        vec2 texSize = vec2(uvec2(textureSize(_group_0_binding_1_fs, 0).xy));
        vec2 grainOffset = (vec2(1.0) / texSize);
        vec2 pixelCoord = (_e4 * texSize);
        vec3 _e1121 = rec2020Color;
        float y_c_4 = dot(_e1121, vec3(0.2627, 0.678, 0.0593));
        float _e1127 = rScale;
        float _e1128 = gScale;
        float _e1129 = bScale;
        float _e1131 = exposureGain;
        vec3 wbExposureScale_1 = (vec3(_e1127, _e1128, _e1129) * _e1131);
        float _e1133 = editSharpening;
        if (!((_e1133 > 0.0))) {
            float _e1139 = editNoiseReduction;
            local_40 = (_e1139 > 0.0);
        } else {
            local_40 = true;
        }
        bool _e1143 = local_40;
        if (_e1143) {
            float _e1146 = _group_0_binding_0_fs.sharpeningRadius;
            float _e1149 = _group_0_binding_0_fs.baselineSharpness;
            float effectiveRadius = max((_e1146 * _e1149), 0.5);
            vec2 sharpenOffset = (grainOffset * effectiveRadius);
            vec4 leftSample = texture(_group_0_binding_1_fs, vec2((_e4 - vec2(sharpenOffset.x, 0.0))));
            vec4 rightSample = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(sharpenOffset.x, 0.0))));
            vec4 topSample = texture(_group_0_binding_1_fs, vec2((_e4 - vec2(0.0, sharpenOffset.y))));
            vec4 bottomSample = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(0.0, sharpenOffset.y))));
            vec4 tlSample = texture(_group_0_binding_1_fs, vec2((_e4 - sharpenOffset)));
            vec4 trSample = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(sharpenOffset.x, -(sharpenOffset.y)))));
            vec4 blSample = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(-(sharpenOffset.x), sharpenOffset.y))));
            vec4 brSample = texture(_group_0_binding_1_fs, vec2((_e4 + sharpenOffset)));
            int _e1216 = _group_0_binding_0_fs.isLinearRaw;
            if ((_e1216 == 0)) {
                vec3 _e1221 = srgbGammaToLinear3_(leftSample.xyz);
                y_l = dot((LINEAR_SRGB_TO_REC2020_ * _e1221), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1230 = srgbGammaToLinear3_(rightSample.xyz);
                y_r = dot((LINEAR_SRGB_TO_REC2020_ * _e1230), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1239 = srgbGammaToLinear3_(topSample.xyz);
                y_t = dot((LINEAR_SRGB_TO_REC2020_ * _e1239), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1248 = srgbGammaToLinear3_(bottomSample.xyz);
                y_b = dot((LINEAR_SRGB_TO_REC2020_ * _e1248), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1257 = srgbGammaToLinear3_(tlSample.xyz);
                y_tl = dot((LINEAR_SRGB_TO_REC2020_ * _e1257), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1266 = srgbGammaToLinear3_(trSample.xyz);
                y_tr = dot((LINEAR_SRGB_TO_REC2020_ * _e1266), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1275 = srgbGammaToLinear3_(blSample.xyz);
                y_bl = dot((LINEAR_SRGB_TO_REC2020_ * _e1275), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1284 = srgbGammaToLinear3_(brSample.xyz);
                y_br = dot((LINEAR_SRGB_TO_REC2020_ * _e1284), vec3(0.2627, 0.678, 0.0593));
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
            vec3 _e1347 = rec2020Color;
            float _e1348 = y_l;
            float _e1349 = y_r;
            float _e1350 = y_t;
            float _e1351 = y_b;
            float _e1352 = y_tl;
            float _e1353 = y_tr;
            float _e1354 = y_bl;
            float _e1355 = y_br;
            float _e1356 = editSharpening;
            float _e1359 = _group_0_binding_0_fs.sharpeningMasking;
            float _e1360 = editNoiseReduction;
            vec3 _e1361 = applyDetailAndSharpening(_e1347, y_c_4, _e1348, _e1349, _e1350, _e1351, _e1352, _e1353, _e1354, _e1355, _e1356, _e1359, _e1360);
            rec2020Color = _e1361;
        }
        int _e1362 = editGrainMode;
        if ((_e1362 != 0)) {
            vec4 leftSample_1 = texture(_group_0_binding_1_fs, vec2((_e4 - vec2(grainOffset.x, 0.0))));
            vec4 rightSample_1 = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(grainOffset.x, 0.0))));
            vec4 topSample_1 = texture(_group_0_binding_1_fs, vec2((_e4 - vec2(0.0, grainOffset.y))));
            vec4 bottomSample_1 = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(0.0, grainOffset.y))));
            vec4 tlSample_1 = texture(_group_0_binding_1_fs, vec2((_e4 - grainOffset)));
            vec4 trSample_1 = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(grainOffset.x, -(grainOffset.y)))));
            vec4 blSample_1 = texture(_group_0_binding_1_fs, vec2((_e4 + vec2(-(grainOffset.x), grainOffset.y))));
            vec4 brSample_1 = texture(_group_0_binding_1_fs, vec2((_e4 + grainOffset)));
            int _e1427 = _group_0_binding_0_fs.isLinearRaw;
            if ((_e1427 == 0)) {
                vec3 _e1432 = srgbGammaToLinear3_(leftSample_1.xyz);
                gy_l = dot((LINEAR_SRGB_TO_REC2020_ * _e1432), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1441 = srgbGammaToLinear3_(rightSample_1.xyz);
                gy_r = dot((LINEAR_SRGB_TO_REC2020_ * _e1441), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1450 = srgbGammaToLinear3_(topSample_1.xyz);
                gy_t = dot((LINEAR_SRGB_TO_REC2020_ * _e1450), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1459 = srgbGammaToLinear3_(bottomSample_1.xyz);
                gy_b = dot((LINEAR_SRGB_TO_REC2020_ * _e1459), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1468 = srgbGammaToLinear3_(tlSample_1.xyz);
                gy_tl = dot((LINEAR_SRGB_TO_REC2020_ * _e1468), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1477 = srgbGammaToLinear3_(trSample_1.xyz);
                gy_tr = dot((LINEAR_SRGB_TO_REC2020_ * _e1477), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1486 = srgbGammaToLinear3_(blSample_1.xyz);
                gy_bl = dot((LINEAR_SRGB_TO_REC2020_ * _e1486), vec3(0.2627, 0.678, 0.0593));
                vec3 _e1495 = srgbGammaToLinear3_(brSample_1.xyz);
                gy_br = dot((LINEAR_SRGB_TO_REC2020_ * _e1495), vec3(0.2627, 0.678, 0.0593));
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
            float _e1558 = formatMag_1;
            float s_format = sqrt(max(_e1558, 0.1));
            float _e1564 = _group_0_binding_0_fs.grainSize;
            float clusterSize = clamp((_e1564 * s_format), 0.5, 3.5);
            float _e1571 = _group_0_binding_0_fs.grainAmount;
            float _e1572 = formatMag_1;
            float effectiveAmount = (_e1571 * pow(max(_e1572, 0.1), 0.25));
            vec3 _e1578 = rec2020Color;
            float _e1579 = gy_l;
            float _e1580 = gy_r;
            float _e1581 = gy_t;
            float _e1582 = gy_b;
            float _e1583 = gy_tl;
            float _e1584 = gy_tr;
            float _e1585 = gy_bl;
            float _e1586 = gy_br;
            int _e1587 = editGrainMode;
            int _e1590 = _group_0_binding_0_fs.emulsionArchetype;
            vec3 _e1591 = applyStochasticEmulsionGrain(_e1578, y_c_4, _e1579, _e1580, _e1581, _e1582, _e1583, _e1584, _e1585, _e1586, pixelCoord, _e1587, clusterSize, effectiveAmount, _e1590);
            rec2020Color = _e1591;
        }
    }
    float _e1592 = editContrast;
    contrastGain = (1.0 + (_e1592 / 100.0));
    float _e1598 = editContrast;
    if ((_e1598 < 0.0)) {
        float _e1601 = editContrast;
        contrastGain = (1.0 + ((_e1601 / 100.0) * 0.6));
    }
    int _e1611 = _group_0_binding_0_fs.isLinearRaw;
    if ((_e1611 == 0)) {
        vec3 _e1615 = rec2020Color;
        vec3 srgb = max((REC2020_TO_LINEAR_SRGB * _e1615), vec3(0.0));
        float _e1620 = contrastGain;
        if ((abs((_e1620 - 1.0)) > 0.001)) {
            float _e1629 = contrastGain;
            srgbLinearOut = clamp((((srgb - vec3(0.18)) * _e1629) + vec3(0.18)), vec3(0.0), vec3(1.0));
        } else {
            srgbLinearOut = clamp(srgb, vec3(0.0), vec3(1.0));
        }
    } else {
        int _e1646 = _group_0_binding_0_fs.toneProfileMode;
        if ((_e1646 == 0)) {
            vec3 _e1649 = rec2020Color;
            float _e1650 = contrastGain;
            vec3 _e1651 = agxToneMapVivid(_e1649, _e1650);
            srgbLinearOut = _e1651;
        } else {
            int _e1654 = _group_0_binding_0_fs.toneProfileMode;
            if ((_e1654 == 1)) {
                vec3 _e1657 = rec2020Color;
                float _e1658 = contrastGain;
                vec3 _e1659 = agxToneMap(_e1657, _e1658);
                srgbLinearOut = _e1659;
            } else {
                int _e1662 = _group_0_binding_0_fs.toneProfileMode;
                if ((_e1662 == 6)) {
                    vec3 _e1665 = rec2020Color;
                    float _e1666 = contrastGain;
                    vec3 _e1667 = agxToneMapPhotographic(_e1665, _e1666);
                    srgbLinearOut = _e1667;
                } else {
                    int _e1670 = _group_0_binding_0_fs.toneProfileMode;
                    int camMode = (_e1670 - 1);
                    vec3 _e1674 = rec2020Color;
                    vec3 c_srgb_1 = max((REC2020_TO_LINEAR_SRGB * _e1674), vec3(0.0));
                    vec3 _e1679 = applyCameraHueSatMap(c_srgb_1, camMode);
                    float _e1681 = applyCameraToneCurve(_e1679.x, camMode);
                    float _e1683 = applyCameraToneCurve(_e1679.y, camMode);
                    float _e1685 = applyCameraToneCurve(_e1679.z, camMode);
                    vec3 curved_2 = vec3(_e1681, _e1683, _e1685);
                    float _e1687 = contrastGain;
                    if ((abs((_e1687 - 1.0)) > 0.001)) {
                        float _e1696 = contrastGain;
                        srgbLinearOut = clamp((((curved_2 - vec3(0.18)) * _e1696) + vec3(0.18)), vec3(0.0), vec3(1.0));
                    } else {
                        srgbLinearOut = clamp(curved_2, vec3(0.0), vec3(1.0));
                    }
                }
            }
        }
    }
    float _e1713 = _group_0_binding_0_fs.isOriginal;
    if ((_e1713 < 0.5)) {
        int _e1720 = _group_0_binding_0_fs.printMediaMode;
        local_41 = (_e1720 != 0);
    } else {
        local_41 = false;
    }
    bool _e1724 = local_41;
    if (_e1724) {
        vec3 _e1725 = srgbLinearOut;
        int _e1728 = _group_0_binding_0_fs.printMediaMode;
        int _e1731 = _group_0_binding_0_fs.enlargerFilterMode;
        float _e1734 = _group_0_binding_0_fs.printPaperGrade;
        float _e1737 = _group_0_binding_0_fs.printProcessDepth;
        vec3 _e1738 = applyPrintMedia(_e1725, _e1728, _e1731, _e1734, _e1737);
        srgbLinearOut = _e1738;
    }
    vec3 _e1739 = srgbLinearOut;
    finalColor = _e1739;
    int _e1743 = _group_0_binding_0_fs.lutEnabled;
    if ((_e1743 != 0)) {
        float _e1750 = _group_0_binding_0_fs.lutIntensity;
        local_42 = (_e1750 > 0.0);
    } else {
        local_42 = false;
    }
    bool _e1754 = local_42;
    if (_e1754) {
        float _e1759 = _group_0_binding_0_fs.isOriginal;
        local_43 = (_e1759 < 0.5);
    } else {
        local_43 = false;
    }
    bool _e1763 = local_43;
    if (_e1763) {
        float _e1766 = _group_0_binding_0_fs.lutSize;
        float size = max(_e1766, 2.0);
        vec3 _e1769 = finalColor;
        vec3 lutCoord = (((_e1769 * (size - 1.0)) + vec3(0.5)) / vec3(size));
        vec4 _e1780 = texture(_group_0_binding_3_fs, vec3(lutCoord));
        vec3 lutSample = _e1780.xyz;
        vec3 _e1782 = finalColor;
        float _e1785 = _group_0_binding_0_fs.lutIntensity;
        finalColor = mix(_e1782, lutSample, clamp(_e1785, 0.0, 1.0));
    }
    int _e1792 = _group_0_binding_0_fs.edrEnabled;
    if ((_e1792 != 0)) {
        float _e1799 = _group_0_binding_0_fs.edrHeadroom;
        local_44 = (_e1799 > 1.001);
    } else {
        local_44 = false;
    }
    bool _e1803 = local_44;
    if (_e1803) {
        float _e1806 = _group_0_binding_0_fs.edrHeadroom;
        float headroom = clamp(_e1806, 1.0, 4.0);
        int _e1812 = _group_0_binding_0_fs.hasGainMap;
        if ((_e1812 != 0)) {
            vec4 _e1817 = texture(_group_0_binding_8_fs, vec2(_e4));
            float g_1 = _e1817.x;
            float _e1821 = _group_0_binding_0_fs.gainMapMin;
            float _e1824 = _group_0_binding_0_fs.gainMapMax;
            float _e1827 = _group_0_binding_0_fs.gainMapMin;
            float gainLog2_ = (_e1821 + (g_1 * (_e1824 - _e1827)));
            float _e1833 = _group_0_binding_0_fs.gainMapGamma;
            float gamma_1 = max(_e1833, 0.01);
            float gainFactor = pow(2.0, (gainLog2_ / gamma_1));
            float _e1841 = _group_0_binding_0_fs.gainMapCapacityMax;
            float maxCap = max(_e1841, 1.01);
            float weight = clamp(((headroom - 1.0) / (maxCap - 1.0)), 0.0, 1.0);
            float effectiveGain = (1.0 + ((gainFactor - 1.0) * weight));
            vec3 _e1857 = finalColor;
            finalColor = (_e1857 * effectiveGain);
        } else {
            int _e1861 = _group_0_binding_0_fs.isLinearRaw;
            if ((_e1861 == 1)) {
                vec3 _e1864 = rec2020Color;
                float sceneLum = dot(_e1864, vec3(0.2627, 0.678, 0.0593));
                vec3 _e1870 = finalColor;
                float outLum = dot(_e1870, vec3(0.2126, 0.7152, 0.0722));
                float highlightFactor = smoothstep(0.85, 1.0, max(outLum, clamp((sceneLum - 0.15), 0.0, 1.0)));
                if ((highlightFactor > 0.0)) {
                    float edrBoost = (1.0 + ((headroom - 1.0) * pow(highlightFactor, 1.5)));
                    vec3 _e1894 = finalColor;
                    finalColor = (_e1894 * edrBoost);
                }
            }
        }
        float _e1897 = finalColor.x;
        float _e1899 = finalColor.y;
        float _e1901 = finalColor.z;
        float peakC = max(_e1897, max(_e1899, _e1901));
        if ((peakC > headroom)) {
            float excess_1 = ((peakC - headroom) / peakC);
            vec3 _e1907 = finalColor;
            finalColor = mix((_e1907 * (headroom / peakC)), vec3(headroom), clamp((excess_1 * 1.5), 0.0, 1.0));
        } else {
            if ((peakC > 1.0)) {
                float desatThreshold = (1.0 + ((headroom - 1.0) * 0.8));
                if ((peakC > desatThreshold)) {
                    float t_4 = ((peakC - desatThreshold) / max((headroom - desatThreshold), 0.0001));
                    vec3 _e1931 = finalColor;
                    finalColor = mix(_e1931, vec3(peakC), clamp((t_4 * 0.4), 0.0, 0.4));
                }
            }
        }
        vec3 _e1940 = finalColor;
        finalColor = max((LINEAR_SRGB_TO_DISPLAY_P3_ * _e1940), vec3(0.0));
        vec3 _e1945 = finalColor;
        _fs2p_location0 = vec4(_e1945, sampledColor.w);
        return;
    } else {
        vec3 _e1948 = finalColor;
        finalColor = clamp(_e1948, vec3(0.0), vec3(1.0));
        float _e1956 = ign(in_.position.xy);
        float dither = ((_e1956 - 0.5) / 255.0);
        vec3 _e1961 = finalColor;
        vec3 _e1962 = linearToSrgbGamma3_(_e1961);
        _fs2p_location0 = vec4(clamp((_e1962 + vec3(dither)), vec3(0.0), vec3(1.0)), sampledColor.w);
        return;
    }
}

`;

export const SHADER_UNIFORM_BLOCK_NAME = 'ShaderUniforms_block_0Fragment';
export const SHADER_UNIFORM_SIZE_BYTES = 288;

export const SHADER_SAMPLER_NAMES = {
  texture: '_group_0_binding_1_fs',
  lut3d: '_group_0_binding_3_fs',
  curveMaster: '_group_0_binding_5_fs',
  curveRgb: '_group_0_binding_6_fs',
  gainMap: '_group_0_binding_8_fs',
} as const;
