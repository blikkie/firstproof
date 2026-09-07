#ifndef ShaderTypes_h
#define ShaderTypes_h

#ifdef __METAL_VERSION__
#include <metal_stdlib>
typedef metal::float2 simd_float2;
typedef metal::float3 simd_float3;
typedef metal::float4 simd_float4;
#else
#include <simd/simd.h>
#include <stdbool.h>
#include <stdint.h>
#endif

struct Vertex {
    simd_float2 position;
    simd_float2 texCoord;
};

struct ShaderUniforms {
    float exposure;          // -5.0 .. +5.0 EV
    float temperature;       // 2000K .. 50000K (default 5500K)
    float tint;              // -150 .. +150 (default 0.0)
    float contrast;          // -100 .. +100 (default 0.0)
    float highlights;        // -100 .. +100 (default 0.0)
    float shadows;           // -100 .. +100 (default 0.0)
    float whites;            // -100 .. +100 (default 0.0)
    float blacks;            // -100 .. +100 (default 0.0)
    float vibrance;          // -100 .. +100 (default 0.0)
    float saturation;        // -100 .. +100 (default 0.0)
    float deepBlue;          // -100 .. +100 (default 0.0)
    float foliage;           // -100 .. +100 (default 0.0)
    float straightenAngle;   // Radians
    float aspectRatio;       // width / height
    int32_t lensCorrectionEnabled;
    int32_t distortionModel; // 0 = poly3, 1 = ptlens
    float k1;
    float k2;
    float k3;
    float k4;
    float isOriginal;        // 1.0 = Bypass edits
    int32_t isLinearRaw;     // 1 = Texture contains Wide-Gamut Linear Rec.2020 half-floats
    int32_t toneProfileMode; // 0 = AgX Vivid, 1 = AgX Neutral, 2 = Standard, 3 = Portrait, 4 = Landscape, 5 = Vivid
    int32_t lutEnabled;      // 1 = 3D LUT active, 0 = disabled
    float lutIntensity;      // 0.0 .. 1.0
    float lutSize;           // Dimension N (e.g. 17, 33, 65)
    float sharpeningAmount;  // 0.0 .. 100.0 (default 0.0)
    float sharpeningRadius;  // 0.5 .. 3.0 (default 1.0)
    float sharpeningMasking; // 0.0 .. 100.0 (default 0.0)
    float noiseReduction;    // 0.0 .. 100.0 (default 0.0)
    float cropLeft;          // 0.0 .. 1.0 (default 0.0)
    float cropTop;           // 0.0 .. 1.0 (default 0.0)
    float cropRight;         // 0.0 .. 1.0 (default 1.0)
    float cropBottom;        // 0.0 .. 1.0 (default 1.0)
    float perspectiveVertical;   // -100.0 .. +100.0 (default 0.0)
    float perspectiveHorizontal; // -100.0 .. +100.0 (default 0.0)
    int32_t cropConstrain;   // 1 = auto-crop inscribed scale, 0 = free
    int32_t cropModeActive;  // 1 = interactive crop mode (show full image with overlay), 0 = normal framed crop
    int32_t _padding;
    int32_t _padding2;
};

#endif /* ShaderTypes_h */
