#ifndef Geometry_metal
#define Geometry_metal

#include <metal_stdlib>
#include "ShaderTypes.h"
using namespace metal;

// MARK: - Viewport Geometry, Perspective & Lens Distortion Transform
inline float2 transformGeometry(float2 uv, constant ShaderUniforms &uniforms) {
    if (uniforms.isOriginal >= 0.5 || uniforms.isIntermediate == 1) {
        return uv;
    }
    
    float cropL = clamp(uniforms.cropLeft, 0.0, 0.99);
    float cropT = clamp(uniforms.cropTop, 0.0, 0.99);
    float cropR = clamp(uniforms.cropRight, cropL + 0.01, 1.0);
    float cropB = clamp(uniforms.cropBottom, cropT + 0.01, 1.0);
    
    float2 uvc = uv;
    if (uniforms.cropModeActive == 0) {
        uvc.x = cropL + uv.x * (cropR - cropL);
        uvc.y = cropT + uv.y * (cropB - cropT);
    }
    
    float A = max(uniforms.aspectRatio, 0.01);
    float angle = uniforms.straightenAngle;
    float cosA = cos(angle);
    float sinA = sin(angle);
    float cV = uniforms.perspectiveVertical * 0.005;
    float cH = uniforms.perspectiveHorizontal * 0.005;
    
    float autoCropScale = 1.0;
    if (uniforms.cropConstrain != 0) {
        float2 corners[4] = {
            float2( A,  1.0),
            float2( A, -1.0),
            float2(-A, -1.0),
            float2(-A,  1.0)
        };
        for (int i = 0; i < 4; i++) {
            float2 p = corners[i];
            float2 r = float2(
                p.x * cosA - p.y * sinA,
                p.x * sinA + p.y * cosA
            );
            float req = max(abs(r.x) / A, abs(r.y)) - (cV * r.y + cH * r.x);
            if (req > autoCropScale) {
                autoCropScale = req;
            }
        }
    }
    
    // Centered coordinates scaled by aspect ratio
    float2 centered = (uvc - float2(0.5, 0.5)) * 2.0;
    centered.x *= A;
    centered /= autoCropScale;
    
    // Straighten rotation
    float2 rot = float2(
        centered.x * cosA - centered.y * sinA,
        centered.x * sinA + centered.y * cosA
    );
    
    // Perspective keystone (backward projective homography)
    float denom = max(1.0 + cV * rot.y + cH * rot.x, 0.1);
    float2 geom = rot / denom;
    
    // Lens distortion correction
    if (uniforms.lensCorrectionEnabled != 0) {
        float r2 = dot(geom, geom);
        float r = sqrt(r2);
        if (r > 0.0001) {
            float r_corr = r;
            if (uniforms.distortionModel == 0) {
                r_corr = r * (1.0 + uniforms.k1 * r2 + uniforms.k2 * (r2 * r2));
            } else if (uniforms.distortionModel == 1) {
                float a = uniforms.k1;
                float b = uniforms.k2;
                float c = uniforms.k3;
                float d = 1.0 - (a + b + c);
                r_corr = a * (r2 * r2) + b * (r2 * r) + c * r2 + d * r;
            }
            geom *= (r_corr / r);
        }
    }
    
    geom.x /= (2.0 * A);
    geom.y /= 2.0;
    return geom + float2(0.5, 0.5);
}

#endif /* Geometry_metal */
