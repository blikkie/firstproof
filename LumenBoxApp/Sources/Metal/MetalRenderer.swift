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
        
        let imageAspect = Float(texture.width) / Float(texture.height)
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
        
        let options: [MTKTextureLoader.Option: Any] = [
            .SRGB: false,
            .generateMipmaps: false
        ]
        
        do {
            self.currentTexture = try textureLoader.newTexture(cgImage: cgImage, options: options)
            let width = Float(cgImage.width)
            let height = Float(cgImage.height)
            if height > 0 {
                self.uniforms.aspectRatio = width / height
            }
        } catch {
            print("Failed to load Metal texture: \(error)")
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
    float exposure;
    float temperature;
    float tint;
    float contrast;
    float straightenAngle;
    float aspectRatio;
    int lensCorrectionEnabled;
    int distortionModel;
    float k1;
    float k2;
    float k3;
    float k4;
    float isOriginal;
};

vertex VertexOutput vertexShader(
    uint vertexID [[vertex_id]],
    constant VertexInput *vertices [[buffer(0)]],
    constant ShaderUniforms &uniforms [[buffer(1)]]
) {
    VertexOutput out;
    float2 pos = vertices[vertexID].position;
    float2 uv = vertices[vertexID].texCoord;
    
    out.position = float4(pos, 0.0, 1.0);
    
    if (uniforms.straightenAngle != 0.0 && uniforms.isOriginal < 0.5) {
        float angle = uniforms.straightenAngle;
        float cosA = cos(angle);
        float sinA = sin(angle);
        
        float absSin = abs(sinA);
        float absCos = abs(cosA);
        float R = max(uniforms.aspectRatio, 0.01);
        float maxR = max(R, 1.0 / R);
        float zoom = max(absCos + absSin * maxR, 1.0);
        
        float2 centered = uv - float2(0.5, 0.5);
        centered.x *= uniforms.aspectRatio;
        
        float2 rotated = float2(
            centered.x * cosA - centered.y * sinA,
            centered.x * sinA + centered.y * cosA
        );
        rotated.x /= uniforms.aspectRatio;
        
        out.texCoord = (rotated / zoom) + float2(0.5, 0.5);
    } else {
        out.texCoord = uv;
    }
    
    return out;
}

fragment float4 fragmentShader(
    VertexOutput in [[stage_in]],
    texture2d<float> texture [[texture(0)]],
    sampler textureSampler [[sampler(0)]],
    constant ShaderUniforms &uniforms [[buffer(1)]]
) {
    float2 uv = in.texCoord;
    
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return float4(0.08, 0.08, 0.08, 1.0);
    }
    
    if (uniforms.lensCorrectionEnabled != 0 && uniforms.isOriginal < 0.5) {
        float2 norm = (uv - float2(0.5, 0.5)) * 2.0;
        norm.x *= uniforms.aspectRatio;
        float r2 = dot(norm, norm);
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
            float factor = r_corr / r;
            norm *= factor;
            norm.x /= uniforms.aspectRatio;
            uv = norm * 0.5 + float2(0.5, 0.5);
            
            if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
                return float4(0.08, 0.08, 0.08, 1.0);
            }
        }
    }
    
    float4 sampledColor = texture.sample(textureSampler, uv);
    
    if (uniforms.isOriginal > 0.5) {
        return sampledColor;
    }
    
    float3 linearColor = pow(max(sampledColor.rgb, float3(0.0)), float3(2.2));
    
    float tempNorm = clamp(uniforms.temperature / 5500.0, 0.25, 4.0);
    float rScale = pow(tempNorm, 0.75);
    float bScale = pow(1.0 / tempNorm, 0.75);
    
    float tintFactor = uniforms.tint * 0.003;
    float gScale = 1.0 - tintFactor;
    rScale += tintFactor * 0.5;
    bScale += tintFactor * 0.5;
    
    linearColor.r *= rScale;
    linearColor.g *= gScale;
    linearColor.b *= bScale;
    
    float exposureGain = pow(2.0, uniforms.exposure);
    linearColor *= exposureGain;
    
    float contrastGain = 1.0 + (uniforms.contrast / 100.0);
    if (contrastGain != 1.0) {
        float3 normalized = max(linearColor / 0.18, float3(0.0001));
        linearColor = 0.18 * pow(normalized, float3(contrastGain));
    }
    
    float3 tonemapped = 1.0 - exp(-linearColor * 1.2);
    float3 finalColor = pow(clamp(tonemapped, float3(0.0), float3(1.0)), float3(1.0 / 2.2));
    
    return float4(finalColor, sampledColor.a);
}
"""
