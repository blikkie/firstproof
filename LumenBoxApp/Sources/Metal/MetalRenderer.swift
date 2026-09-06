import Foundation
import Metal
import MetalKit
import AppKit

public struct Vertex {
    public var position: SIMD2<Float>
    public var texCoord: SIMD2<Float>
}

public struct ShaderUniforms {
    public var exposure: Float = 0.0
    public var temperature: Float = 5500.0
    public var tint: Float = 0.0
    public var contrast: Float = 0.0
    public var straightenAngle: Float = 0.0
    public var aspectRatio: Float = 1.5
    public var lensCorrectionEnabled: Int32 = 1
    public var distortionModel: Int32 = 0
    public var k1: Float = 0.0
    public var k2: Float = 0.0
    public var k3: Float = 0.0
    public var k4: Float = 0.0
    public var isOriginal: Float = 0.0
    public var isLinearRaw: Int32 = 0
}

public final class MetalRenderer: NSObject, MTKViewDelegate {
    public let device: MTLDevice
    private let commandQueue: MTLCommandQueue
    private var pipelineState: MTLRenderPipelineState?
    private var samplerState: MTLSamplerState?
    private var vertexBuffer: MTLBuffer?
    private var uniformsBuffer: MTLBuffer?
    
    private var currentTexture: MTLTexture?
    private var textureLoader: MTKTextureLoader
    
    public var uniforms = ShaderUniforms()
    
    public init?(device: MTLDevice) {
        self.device = device
        guard let queue = device.makeCommandQueue() else { return nil }
        self.commandQueue = queue
        self.textureLoader = MTKTextureLoader(device: device)
        super.init()
        
        setupPipeline()
        setupQuad()
        setupSampler()
    }
    
    private func setupPipeline() {
        var metalLibrary: MTLLibrary? = device.makeDefaultLibrary()
        
        if metalLibrary == nil {
            if let mainLib = try? device.makeDefaultLibrary(bundle: Bundle.main) {
                metalLibrary = mainLib
            }
        }
        
        // Fallback: If running in standalone SwiftPM without precompiled metallib, load shader source directly
        if metalLibrary == nil {
            if let shaderURL = Bundle.main.url(forResource: "Shaders", withExtension: "metal"),
               let shaderSource = try? String(contentsOf: shaderURL) {
                metalLibrary = try? device.makeLibrary(source: shaderSource, options: nil)
            } else {
                metalLibrary = try? device.makeLibrary(source: embeddedShaderSource, options: nil)
            }
        }
        
        guard let library = metalLibrary else {
            print("Failed to initialize Metal library")
            return
        }
        
        let vertexFunction = library.makeFunction(name: "vertexShader")
        let fragmentFunction = library.makeFunction(name: "fragmentShader")
        
        let pipelineDescriptor = MTLRenderPipelineDescriptor()
        pipelineDescriptor.vertexFunction = vertexFunction
        pipelineDescriptor.fragmentFunction = fragmentFunction
        pipelineDescriptor.colorAttachments[0].pixelFormat = .bgra8Unorm
        
        do {
            self.pipelineState = try device.makeRenderPipelineState(descriptor: pipelineDescriptor)
        } catch {
            print("Failed to create Metal render pipeline state: \(error)")
        }
    }
    
    private func setupQuad() {
        // Full screen quad in normalized device coordinates [-1, 1]
        let vertices: [Vertex] = [
            Vertex(position: SIMD2<Float>(-1.0,  1.0), texCoord: SIMD2<Float>(0.0, 0.0)),
            Vertex(position: SIMD2<Float>( 1.0,  1.0), texCoord: SIMD2<Float>(1.0, 0.0)),
            Vertex(position: SIMD2<Float>(-1.0, -1.0), texCoord: SIMD2<Float>(0.0, 1.0)),
            Vertex(position: SIMD2<Float>( 1.0, -1.0), texCoord: SIMD2<Float>(1.0, 1.0))
        ]
        
        self.vertexBuffer = device.makeBuffer(
            bytes: vertices,
            length: MemoryLayout<Vertex>.stride * vertices.count,
            options: .storageModeShared
        )
        
        self.uniformsBuffer = device.makeBuffer(
            length: MemoryLayout<ShaderUniforms>.size,
            options: .storageModeShared
        )
    }
    
    private func setupSampler() {
        let samplerDescriptor = MTLSamplerDescriptor()
        samplerDescriptor.minFilter = .linear
        samplerDescriptor.magFilter = .linear
        samplerDescriptor.sAddressMode = .clampToZero
        samplerDescriptor.tAddressMode = .clampToZero
        self.samplerState = device.makeSamplerState(descriptor: samplerDescriptor)
    }
    
    // MARK: - Aspect-Fit Quad Update
    private func updateQuad(viewSize: CGSize) {
        guard let texture = currentTexture, viewSize.width > 0, viewSize.height > 0 else { return }
        
        var imageAspect = Float(texture.width) / Float(texture.height)
        
        // Stabilize minor aspect jitter across thumbnail / preview JPEG / raw demosaic
        // by snapping to standard photographic sensor ratios if within 1.5% tolerance
        let standardRatios: [Float] = [3.0 / 2.0, 2.0 / 3.0, 4.0 / 3.0, 3.0 / 4.0, 16.0 / 9.0, 9.0 / 16.0, 1.0]
        for ratio in standardRatios {
            if abs(imageAspect - ratio) < 0.015 {
                imageAspect = ratio
                break
            }
        }
        
        let viewAspect = Float(viewSize.width) / Float(viewSize.height)
        
        var quadW: Float = 1.0
        var quadH: Float = 1.0
        
        if imageAspect > viewAspect {
            // Image is wider than viewport: fit width, letterbox top/bottom
            quadW = 1.0
            quadH = viewAspect / imageAspect
        } else {
            // Image is taller than viewport: fit height, pillarbox left/right
            quadW = imageAspect / viewAspect
            quadH = 1.0
        }
        
        let vertices: [Vertex] = [
            Vertex(position: SIMD2<Float>(-quadW,  quadH), texCoord: SIMD2<Float>(0.0, 0.0)),
            Vertex(position: SIMD2<Float>( quadW,  quadH), texCoord: SIMD2<Float>(1.0, 0.0)),
            Vertex(position: SIMD2<Float>(-quadW, -quadH), texCoord: SIMD2<Float>(0.0, 1.0)),
            Vertex(position: SIMD2<Float>( quadW, -quadH), texCoord: SIMD2<Float>(1.0, 1.0))
        ]
        
        if let vertexBuffer = vertexBuffer {
            let ptr = vertexBuffer.contents().bindMemory(to: Vertex.self, capacity: vertices.count)
            for (idx, vertex) in vertices.enumerated() {
                ptr[idx] = vertex
            }
        }
    }
    
    // MARK: - Texture Upload
    public func updateTexture(with image: NSImage) {
        guard let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else { return }
        
        let width = cgImage.width
        let height = cgImage.height
        guard width > 0, height > 0 else { return }
        
        // Standardize into guaranteed 32-bit RGBA (Big Endian) bitmap to eliminate channel swapping
        let bytesPerRow = width * 4
        let totalBytes = bytesPerRow * height
        guard let pixelBuffer = malloc(totalBytes) else { return }
        defer { free(pixelBuffer) }
        
        let colorSpace = CGColorSpace(name: CGColorSpace.sRGB)!
        let bitmapInfo = CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedLast.rawValue | CGBitmapInfo.byteOrder32Big.rawValue)
        
        if let ctx = CGContext(
            data: pixelBuffer,
            width: width,
            height: height,
            bitsPerComponent: 8,
            bytesPerRow: bytesPerRow,
            space: colorSpace,
            bitmapInfo: bitmapInfo.rawValue
        ) {
            ctx.draw(cgImage, in: CGRect(x: 0, y: 0, width: width, height: height))
            
            let texDesc = MTLTextureDescriptor.texture2DDescriptor(
                pixelFormat: .rgba8Unorm,
                width: width,
                height: height,
                mipmapped: false
            )
            texDesc.usage = [.shaderRead]
            if let texture = device.makeTexture(descriptor: texDesc) {
                texture.replace(
                    region: MTLRegionMake2D(0, 0, width, height),
                    mipmapLevel: 0,
                    withBytes: pixelBuffer,
                    bytesPerRow: bytesPerRow
                )
                self.currentTexture = texture
                self.uniforms.aspectRatio = Float(width) / Float(height)
                self.uniforms.isLinearRaw = 0
                return
            }
        }
        
        // Fallback to MTKTextureLoader if context fails
        do {
            self.currentTexture = try textureLoader.newTexture(cgImage: cgImage, options: [.SRGB: false])
            self.uniforms.aspectRatio = Float(width) / Float(height)
            self.uniforms.isLinearRaw = 0
        } catch {
            print("Failed to load Metal texture: \(error)")
        }
    }
    
    // MARK: - Linear Raw Half-Float Texture Upload
    public func updateLinearRawTexture(width: Int, height: Int, data: Data) {
        guard width > 0, height > 0, data.count >= width * height * 8 else { return }
        
        let texDesc = MTLTextureDescriptor.texture2DDescriptor(
            pixelFormat: .rgba16Float,
            width: width,
            height: height,
            mipmapped: false
        )
        texDesc.usage = [.shaderRead]
        if let texture = device.makeTexture(descriptor: texDesc) {
            data.withUnsafeBytes { rawBuffer in
                if let baseAddress = rawBuffer.baseAddress {
                    texture.replace(
                        region: MTLRegionMake2D(0, 0, width, height),
                        mipmapLevel: 0,
                        withBytes: baseAddress,
                        bytesPerRow: width * 8
                    )
                }
            }
            self.currentTexture = texture
            self.uniforms.aspectRatio = Float(width) / Float(height)
            self.uniforms.isLinearRaw = 1
        }
    }
    
    // MARK: - MTKViewDelegate
    public func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {
        updateQuad(viewSize: size)
    }
    
    public func draw(in view: MTKView) {
        guard let drawable = view.currentDrawable,
              let renderPassDescriptor = view.currentRenderPassDescriptor,
              let pipelineState = pipelineState,
              let texture = currentTexture,
              let vertexBuffer = vertexBuffer,
              let uniformsBuffer = uniformsBuffer else {
            return
        }
        
        // Dynamically adjust quad to preserve exact aspect ratio inside current viewport
        updateQuad(viewSize: view.drawableSize)
        
        // Update uniforms buffer in RAM
        let contents = uniformsBuffer.contents().bindMemory(to: ShaderUniforms.self, capacity: 1)
        contents.pointee = uniforms
        
        guard let commandBuffer = commandQueue.makeCommandBuffer(),
              let encoder = commandBuffer.makeRenderCommandEncoder(descriptor: renderPassDescriptor) else {
            return
        }
        
        encoder.setRenderPipelineState(pipelineState)
        encoder.setVertexBuffer(vertexBuffer, offset: 0, index: 0)
        encoder.setVertexBuffer(uniformsBuffer, offset: 0, index: 1)
        
        encoder.setFragmentTexture(texture, index: 0)
        encoder.setFragmentSamplerState(samplerState, index: 0)
        encoder.setFragmentBuffer(uniformsBuffer, offset: 0, index: 1)
        
        encoder.drawPrimitives(type: .triangleStrip, vertexStart: 0, vertexCount: 4)
        encoder.endEncoding()
        
        commandBuffer.present(drawable)
        commandBuffer.commit()
    }
}

private let embeddedShaderSource = """
#include <metal_stdlib>
using namespace metal;

struct VertexInput {
    float2 position [[attribute(0)]];
    float2 texCoord [[attribute(1)]];
};

struct VertexOutput {
    float4 position [[position]];
    float2 texCoord;
};

struct ShaderUniforms {
    float exposure;          // -5.0 .. +5.0 EV
    float temperature;       // 2000K .. 50000K (default 5500K)
    float tint;              // -150 .. +150 (default 0.0)
    float contrast;          // -100 .. +100 (default 0.0)
    float straightenAngle;   // Radians
    float aspectRatio;       // width / height
    int lensCorrectionEnabled;
    int distortionModel;     // 0 = poly3, 1 = ptlens
    float k1;
    float k2;
    float k3;
    float k4;
    float isOriginal;        // 1.0 = Bypass edits
    int isLinearRaw;         // 1 = Texture contains Wide-Gamut Linear Rec.2020 half-floats
};

// MARK: - Vertex Shader
vertex VertexOutput vertexShader(
    uint vertexID [[vertex_id]],
    constant VertexInput *vertices [[buffer(0)]],
    constant ShaderUniforms &uniforms [[buffer(1)]]
) {
    VertexOutput out;
    float2 pos = vertices[vertexID].position;
    float2 uv = vertices[vertexID].texCoord;
    
    out.position = float4(pos, 0.0, 1.0);
    
    // Straighten rotation around center (0.5, 0.5) with aspect ratio correction and auto-crop
    if (uniforms.straightenAngle != 0.0 && uniforms.isOriginal < 0.5) {
        float angle = uniforms.straightenAngle;
        float cosA = cos(angle);
        float sinA = sin(angle);
        
        // Compute auto-crop inscribed zoom factor S(theta)
        float absSin = abs(sinA);
        float absCos = abs(cosA);
        float R = max(uniforms.aspectRatio, 0.01);
        float maxR = max(R, 1.0 / R);
        float zoom = max(absCos + absSin * maxR, 1.0);
        
        float2 centered = uv - float2(0.5, 0.5);
        centered.x *= uniforms.aspectRatio; // compensate aspect ratio
        
        float2 rotated = float2(
            centered.x * cosA - centered.y * sinA,
            centered.x * sinA + centered.y * cosA
        );
        rotated.x /= uniforms.aspectRatio;
        
        // Scale by 1.0 / zoom to eliminate black corner bars
        out.texCoord = (rotated / zoom) + float2(0.5, 0.5);
    } else {
        out.texCoord = uv;
    }
    
    return out;
}

// MARK: - Fragment Shader (Linear RAW Pipeline)
fragment float4 fragmentShader(
    VertexOutput in [[stage_in]],
    texture2d<float> texture [[texture(0)]],
    sampler textureSampler [[sampler(0)]],
    constant ShaderUniforms &uniforms [[buffer(1)]]
) {
    float2 uv = in.texCoord;
    
    // Discard pixels rotated outside image bounds
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return float4(0.08, 0.08, 0.08, 1.0); // Neutral dark canvas background
    }
    
    // 1. Optics & Lens Distortion Correction
    if (uniforms.lensCorrectionEnabled != 0 && uniforms.isOriginal < 0.5) {
        float2 norm = (uv - float2(0.5, 0.5)) * 2.0;
        norm.x *= uniforms.aspectRatio;
        float r2 = dot(norm, norm);
        float r = sqrt(r2);
        
        if (r > 0.0001) {
            float r_corr = r;
            if (uniforms.distortionModel == 0) {
                // Poly3 model: r_d = r * (1 + k1*r^2 + k2*r^4)
                r_corr = r * (1.0 + uniforms.k1 * r2 + uniforms.k2 * (r2 * r2));
            } else if (uniforms.distortionModel == 1) {
                // PTLens model: r_d = a*r^4 + b*r^3 + c*r^2 + d*r
                float a = uniforms.k1;
                float b = uniforms.k2;
                float c = uniforms.k3;
                float d = 1.0 - (a + b + c);
                r_corr = a * (r2 * r2) + b * (r2 * r) + c * r2 + d * r;
            }
            float factor = r_corr / r;
            norm *= factor;
            norm.x /= uniforms.aspectRatio;
            uv = norm * 0.5 + float2(0.5, 0.5);
            
            if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
                return float4(0.08, 0.08, 0.08, 1.0);
            }
        }
    }
    
    // Sample texture
    float4 sampledColor = texture.sample(textureSampler, uv);
    
    // If user is holding '\\' for original as-shot comparison:
    // For JPEG thumbnails, return sampledColor directly.
    if (uniforms.isOriginal > 0.5 && uniforms.isLinearRaw == 0) {
        return sampledColor;
    }
    
    // Fast-path: When all edit sliders are at default neutral positions for JPEG thumbnails, return exact as-shot pixels.
    // (Linear RAW must always pass through the baseline exposure, filmic curve, and sRGB color transform).
    bool isNeutralEdits = (uniforms.exposure == 0.0) && 
                          (abs(uniforms.temperature - 5500.0) < 1.0) && 
                          (uniforms.tint == 0.0) && 
                          (uniforms.contrast == 0.0);
    
    if (uniforms.isLinearRaw == 0 && isNeutralEdits) {
        return sampledColor;
    }
    
    float editExposure = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.exposure;
    float editTemp = (uniforms.isOriginal > 0.5) ? 5500.0 : uniforms.temperature;
    float editTint = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.tint;
    float editContrast = (uniforms.isOriginal > 0.5) ? 0.0 : uniforms.contrast;
    
// MARK: - Scene-Referred Rec.2020 Color Pipeline

// Exact sRGB electro-optical transfer functions (IEC 61966-2-1)
#define srgbGammaToLinear(c) ((c <= 0.04045) ? (c / 12.92) : pow((c + 0.055) / 1.055, 2.4))
#define linearToSrgbGamma(c) ((c <= 0.0031308) ? (12.92 * c) : (1.055 * pow(c, 1.0 / 2.4) - 0.055))

// Linear sRGB -> Linear Rec.2020 Matrix (columns in Metal MSL)
constant float3x3 LINEAR_SRGB_TO_REC2020 = float3x3(
    float3(0.6274041, 0.0690972, 0.0163914),
    float3(0.3292821, 0.9195411, 0.0880133),
    float3(0.0433138, 0.0113617, 0.8955953)
);

// Linear Rec.2020 -> Linear sRGB Matrix (columns in Metal MSL)
constant float3x3 REC2020_TO_LINEAR_SRGB = float3x3(
    float3( 1.6604910, -0.1245505, -0.0181508),
    float3(-0.5876411,  1.1328999, -0.1005789),
    float3(-0.0728499, -0.0083494,  1.1187297)
);

    // 2. Decode texture to Linear Rec.2020 working space
    float3 rec2020Color;
    if (uniforms.isLinearRaw == 1) {
        // Texture is already native Wide-Gamut Linear Rec.2020 scene photons!
        rec2020Color = sampledColor.rgb;
    } else {
        // Embedded JPEG or standard sRGB image: decode gamma to linear then map to Rec.2020
        float3 linearSrgb = float3(
            srgbGammaToLinear(clamp(sampledColor.r, 0.0, 1.0)),
            srgbGammaToLinear(clamp(sampledColor.g, 0.0, 1.0)),
            srgbGammaToLinear(clamp(sampledColor.b, 0.0, 1.0))
        );
        rec2020Color = LINEAR_SRGB_TO_REC2020 * linearSrgb;
    }
    
    // 3. White Balance & Temperature Adjustment in Wide-Gamut Space
    if (abs(editTemp - 5500.0) >= 1.0 || editTint != 0.0) {
        float tempNorm = clamp(editTemp / 5500.0, 0.25, 4.0);
        float rScale = pow(tempNorm, 0.75);
        float bScale = pow(1.0 / tempNorm, 0.75);
        
        float tintFactor = editTint * 0.003;
        float gScale = 1.0 - tintFactor;
        rScale += tintFactor * 0.5;
        bScale += tintFactor * 0.5;
        
        rec2020Color.r *= rScale;
        rec2020Color.g *= gScale;
        rec2020Color.b *= bScale;
    }
    
    // 4. Linear Exposure Multiplication (2^EV) in Wide-Gamut Space
    float totalExposure = editExposure;
    if (uniforms.isLinearRaw == 1) {
        totalExposure += 0.65; // Baseline photographic exposure gain for RAW sensor midtone anchoring
    }
    if (totalExposure != 0.0) {
        rec2020Color *= pow(2.0, totalExposure);
    }
    
    // 5. Tone & Contrast S-Curve in Wide-Gamut Space
    if (uniforms.isLinearRaw == 1 || editContrast != 0.0) {
        float contrastGain = 1.0 + (editContrast / 100.0);
        float3 norm = max(rec2020Color / 0.18, float3(0.0001));
        float3 curved = 0.18 * pow(norm, float3(1.15 * contrastGain));
        rec2020Color = curved / (float3(1.0) + curved * 0.35);
    }
    
    // 6. Display Target Transform: Linear Rec.2020 -> Linear sRGB -> Standard Display Gamma
    float3 srgbLinearOut = max(REC2020_TO_LINEAR_SRGB * rec2020Color, float3(0.0));
    float3 finalColor = float3(
        linearToSrgbGamma(clamp(srgbLinearOut.r, 0.0, 1.0)),
        linearToSrgbGamma(clamp(srgbLinearOut.g, 0.0, 1.0)),
        linearToSrgbGamma(clamp(srgbLinearOut.b, 0.0, 1.0))
    );
    
    return float4(finalColor, sampledColor.a);
}
"""

