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
