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
    float exposure;          // -3.0 .. +3.0 EV
    float temperature;       // 2000K .. 20000K (default 5500K)
    float tint;              // -200 .. +200 (default 0.0)
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
    int32_t hasCurves;       // 1 = tone curves active, 0 = bypass
    int32_t hasMasterCurve;  // 1 = master curve active, 0 = identity
    int32_t hasRgbCurves;    // 1 = per-channel RGB curves active, 0 = identity
    float edrHeadroom;       // 1.0 .. 4.0+ (current display max brightness multiplier)
    int32_t edrEnabled;      // 1 = HDR mode active, 0 = SDR mode (clamp to 1.0)
    int32_t hasGainMap;      // 1 = secondary gain map active, 0 = disabled
    float gainMapMin;        // e.g. 0.0
    float gainMapMax;        // e.g. 3.0 (8x boost)
    float gainMapGamma;      // standard 1.0
    float gainMapCapacityMax;// e.g. 3.0
    int32_t emulsionArchetype; // 0 = None, 1 = RochesterPortrait400, 2 = AshigaraChrome50, 3 = RochesterK14Chrome64, 4 = RochesterCinema500, 5 = RochesterDaylight200, 6 = MobberleySilver400, 7 = CambridgeDiffusion100, 8 = LeverkusenPortrait160, 9 = AshigaraPortrait400, 10 = LiguriaCinema80, 11 = BaodingSilver100
    int32_t emulsionFormat;    // 0 = 35mm, 1 = 120, 2 = 4x5, 3 = Super8, 4 = 16mm
    float emulsionExposure;    // -2.0 .. +2.0 (default 0.0)
    int32_t emulsionGrainMode; // 0 = Off, 1 = SeedFromSensor, 2 = SynthesizePhysical
    float grainScale;          // 0.5 .. 2.0 (default 1.0)
    float grainSize;           // 0.2 .. 3.0 (default 1.0)
    float grainAmount;         // 0.0 .. 2.0 (default 1.0)
    int32_t isIntermediate;  // 1 = pre-rendered intermediate preview (bypass all shader color/tone/geometry)
    float splitToningShadowHue;    // 0.0 .. 360.0 (degrees, default 215.0)
    float splitToningShadowSat;    // 0.0 .. 100.0 (default 0.0)
    float splitToningHighlightHue; // 0.0 .. 360.0 (degrees, default 45.0)
    float splitToningHighlightSat; // 0.0 .. 100.0 (default 0.0)
    float splitToningBalance;      // -100.0 .. +100.0 (default 0.0)
    int32_t printMediaMode;        // 0 = None, 1 = ChromogenicTypeC, 2 = SilverDyeBleach, 3 = FineArtCottonRag, 4 = BarytaSilverGelatin, 5 = WarmtoneChlorobromide, 6 = PlatinumPalladium, 7 = Cyanotype, 8 = BamakoSilverContact
    int32_t enlargerFilterMode;    // 0 = Neutral, 1 = YellowK2, 2 = Orange16, 3 = Red25A, 4 = GreenX1, 5 = Orthochromatic
    float printPaperGrade;         // -2.0 .. +2.0 (default 0.0)
    float printBaseWarmth;         // -100.0 .. +100.0 (default 0.0)
    float printProcessDepth;       // 0.0 .. 100.0 (default 100.0)
    int32_t halationMode;          // 0 = Off, 1 = Monochrome (BaodingSilver100), 2 = Red-shifted (RochesterCinema500 stripped)
};

#endif /* ShaderTypes_h */
