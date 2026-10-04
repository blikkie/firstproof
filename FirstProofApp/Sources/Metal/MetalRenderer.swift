import Foundation
import Metal
import MetalKit
import AppKit
import FirstProofBridge
import FirstProofShaderTypes

extension ShaderUniforms {
    public static var defaultUniforms: ShaderUniforms {
        var u = ShaderUniforms()
        u.temperature = 5500.0
        u.aspectRatio = 1.5
        u.lensCorrectionEnabled = 1
        u.lutIntensity = 1.0
        u.lutSize = 33.0
        u.sharpeningRadius = 1.0
        u.baselineSharpness = 1.0
        u.cropRight = 1.0
        u.cropBottom = 1.0
        u.cropConstrain = 1
        u.edrHeadroom = 1.0
        u.edrEnabled = 1
        u.splitToningShadowHue = 215.0
        u.splitToningShadowSat = 0.0
        u.splitToningHighlightHue = 45.0
        u.splitToningHighlightSat = 0.0
        u.splitToningBalance = 0.0
        u.emulsionGrainMode = 0
        u.grainScale = 1.0
        return u
    }
}

public final class MetalRenderer: NSObject, MTKViewDelegate {
    public let device: MTLDevice
    var commandQueue: MTLCommandQueue
    var pipelineState: MTLRenderPipelineState?
    var thumbnailPipelineState: MTLRenderPipelineState?
    var samplerState: MTLSamplerState?
    var lutSamplerState: MTLSamplerState?
    var curveSamplerState: MTLSamplerState?
    var vertexBuffer: MTLBuffer?
    var uniformsBuffer: MTLBuffer?
    
    var currentTexture: MTLTexture?
    var lutTexture: MTLTexture?
    var defaultLutTexture: MTLTexture?
    var curveMasterTexture: MTLTexture?
    var curveRgbTexture: MTLTexture?
    var defaultCurveMasterTexture: MTLTexture?
    var defaultCurveRgbTexture: MTLTexture?
    var gainMapTexture: MTLTexture?
    var defaultGainMapTexture: MTLTexture?
    var gainMapSamplerState: MTLSamplerState?
    var lastCurves: ToneCurvesRecord?
    var textureLoader: MTKTextureLoader
    
    public var uniforms = ShaderUniforms()
    
    // MARK: - Viewport Zoom, Pan & Hit-Testing
    public var zoomScale: Float = 1.0
    public var panOffset: SIMD2<Float> = SIMD2<Float>(0.0, 0.0)
    public private(set) var zoomScale100: Float = 1.0
    public var onZoomScale100Changed: ((Float) -> Void)?
    public internal(set) var lastQuadBounds: (left: Float, right: Float, top: Float, bottom: Float) = (-1.0, 1.0, 1.0, -1.0)
    public internal(set) var lastQuadUVs: (tl: SIMD2<Float>, tr: SIMD2<Float>, bl: SIMD2<Float>, br: SIMD2<Float>) = (SIMD2<Float>(0, 0), SIMD2<Float>(1, 0), SIMD2<Float>(0, 1), SIMD2<Float>(1, 1))
    
    // MARK: - Viewport Geometry (Rotation & Flip)
    public var rotationDegrees: Int = 0 // 0, 90, 180, 270
    public var flipHorizontal: Bool = false
    public var flipVertical: Bool = false
    
    public var textureWidth: Int { currentTexture?.width ?? 0 }
    public var textureHeight: Int { currentTexture?.height ?? 0 }
    
    public init?(device: MTLDevice) {
        self.device = device
        guard let queue = device.makeCommandQueue() else { return nil }
        self.commandQueue = queue
        self.textureLoader = MTKTextureLoader(device: device)
        super.init()
        
        setupPipeline()
        setupQuad()
        setupSampler()
        setupDefaultLutTexture()
        setupCurveTexturesAndSampler()
        setupDefaultGainMapTexture()
    }
    
    private func setupPipeline() {
        var library: MTLLibrary?
        
        if let defaultLib = device.makeDefaultLibrary() {
            library = defaultLib
        } else if let shaderURL = findShaderURL() {
            do {
                let baseDir = shaderURL.deletingLastPathComponent()
                let sourceWithIncludes = try String(contentsOf: shaderURL, encoding: .utf8)
                let resolvedSource = resolveIncludes(in: sourceWithIncludes, relativeTo: baseDir)
                
                let options = MTLCompileOptions()
                options.fastMathEnabled = true
                library = try device.makeLibrary(source: resolvedSource, options: options)
            } catch {
                print("Failed to compile runtime Metal shader: \(error)")
            }
        }
        
        guard let lib = library else {
            print("Failed to locate or compile any Metal library.")
            return
        }
        
        let pipelineDesc = MTLRenderPipelineDescriptor()
        pipelineDesc.vertexFunction = lib.makeFunction(name: "vertexShader")
        pipelineDesc.fragmentFunction = lib.makeFunction(name: "fragmentShader")
        pipelineDesc.colorAttachments[0].pixelFormat = .rgba16Float
        
        do {
            self.pipelineState = try device.makeRenderPipelineState(descriptor: pipelineDesc)
        } catch {
            print("Failed to create main render pipeline state: \(error)")
        }
        
        let thumbDesc = MTLRenderPipelineDescriptor()
        thumbDesc.vertexFunction = lib.makeFunction(name: "vertexShader")
        thumbDesc.fragmentFunction = lib.makeFunction(name: "fragmentShader")
        thumbDesc.colorAttachments[0].pixelFormat = .bgra8Unorm
        
        do {
            self.thumbnailPipelineState = try device.makeRenderPipelineState(descriptor: thumbDesc)
        } catch {
            print("Failed to create thumbnail pipeline state: \(error)")
        }
    }
    
    private func findShaderURL() -> URL? {
        let bundle = Bundle.main
        if let url = bundle.url(forResource: "Shaders", withExtension: "metal") {
            return url
        }
        
        let localPath = "FirstProofApp/Sources/Shaders/Shaders.metal"
        let currentDir = FileManager.default.currentDirectoryPath
        let candidate = URL(fileURLWithPath: currentDir).appendingPathComponent(localPath)
        if FileManager.default.fileExists(atPath: candidate.path) {
            return candidate
        }
        return nil
    }
    
    private func resolveIncludes(in source: String, relativeTo baseDir: URL) -> String {
        var result = ""
        let lines = source.components(separatedBy: .newlines)
        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            if trimmed.hasPrefix("#include \"") && trimmed.hasSuffix("\"") {
                let start = trimmed.index(trimmed.startIndex, offsetBy: 10)
                let end = trimmed.index(before: trimmed.endIndex)
                let includeName = String(trimmed[start..<end])
                let includeURL = baseDir.appendingPathComponent(includeName)
                if let includedSource = try? String(contentsOf: includeURL, encoding: .utf8) {
                    result += resolveIncludes(in: includedSource, relativeTo: baseDir) + "\n"
                    continue
                }
            }
            result += line + "\n"
        }
        return result
    }
    
    private func setupQuad() {
        let vertices: [Vertex] = [
            Vertex(position: SIMD2<Float>(-1.0,  1.0), texCoord: SIMD2<Float>(0.0, 0.0)),
            Vertex(position: SIMD2<Float>( 1.0,  1.0), texCoord: SIMD2<Float>(1.0, 0.0)),
            Vertex(position: SIMD2<Float>(-1.0, -1.0), texCoord: SIMD2<Float>(0.0, 1.0)),
            Vertex(position: SIMD2<Float>( 1.0, -1.0), texCoord: SIMD2<Float>(1.0, 1.0)),
        ]
        
        self.vertexBuffer = device.makeBuffer(
            bytes: vertices,
            length: vertices.count * MemoryLayout<Vertex>.size,
            options: .storageModeShared
        )
        
        self.uniformsBuffer = device.makeBuffer(
            length: MemoryLayout<ShaderUniforms>.size,
            options: .storageModeShared
        )
    }
    
    private func setupSampler() {
        let samplerDesc = MTLSamplerDescriptor()
        samplerDesc.minFilter = .linear
        samplerDesc.magFilter = .linear
        samplerDesc.sAddressMode = .clampToEdge
        samplerDesc.tAddressMode = .clampToEdge
        self.samplerState = device.makeSamplerState(descriptor: samplerDesc)
        
        let lutDesc = MTLSamplerDescriptor()
        lutDesc.minFilter = .linear
        lutDesc.magFilter = .linear
        lutDesc.rAddressMode = .clampToEdge
        lutDesc.sAddressMode = .clampToEdge
        lutDesc.tAddressMode = .clampToEdge
        self.lutSamplerState = device.makeSamplerState(descriptor: lutDesc)

        let gainMapDesc = MTLSamplerDescriptor()
        gainMapDesc.minFilter = .linear
        gainMapDesc.magFilter = .linear
        gainMapDesc.sAddressMode = .clampToEdge
        gainMapDesc.tAddressMode = .clampToEdge
        self.gainMapSamplerState = device.makeSamplerState(descriptor: gainMapDesc)
    }
    
    // MARK: - Aspect-Fit, Zoom & Pan Quad Update
    public func updateQuad(viewSize: CGSize) {
        guard let texture = currentTexture, viewSize.width > 0, viewSize.height > 0 else { return }
        
        let isOrthogonal = (rotationDegrees == 90 || rotationDegrees == 270)
        let origW = CGFloat(texture.width)
        let origH = CGFloat(texture.height)
        guard origW > 0, origH > 0 else { return }
        
        let cropWFrac = max(CGFloat(uniforms.cropRight - uniforms.cropLeft), 0.01)
        let cropHFrac = max(CGFloat(uniforms.cropBottom - uniforms.cropTop), 0.01)
        let effectiveOrigW = origW * cropWFrac
        let effectiveOrigH = origH * cropHFrac
        
        let dispW = isOrthogonal ? effectiveOrigH : effectiveOrigW
        let dispH = isOrthogonal ? effectiveOrigW : effectiveOrigH
        let imgAspect = dispW / dispH
        let viewAspect = viewSize.width / viewSize.height
        
        var baseQuadW: CGFloat
        var baseQuadH: CGFloat
        if viewAspect > imgAspect {
            baseQuadH = viewSize.height
            baseQuadW = baseQuadH * imgAspect
        } else {
            baseQuadW = viewSize.width
            baseQuadH = baseQuadW / imgAspect
        }
        
        let fitScaleX = baseQuadW / viewSize.width
        let fitScaleY = baseQuadH / viewSize.height
        
        let scale100 = Float(max(dispW / baseQuadW, dispH / baseQuadH))
        if abs(self.zoomScale100 - scale100) > 0.001 {
            self.zoomScale100 = scale100
            self.onZoomScale100Changed?(scale100)
        }
        
        let effectiveScaleX = Float(fitScaleX) * zoomScale
        let effectiveScaleY = Float(fitScaleY) * zoomScale
        
        let left   = -effectiveScaleX + panOffset.x
        let right  =  effectiveScaleX + panOffset.x
        let top    =  effectiveScaleY + panOffset.y
        let bottom = -effectiveScaleY + panOffset.y
        
        var tl: SIMD2<Float>, tr: SIMD2<Float>, bl: SIMD2<Float>, br: SIMD2<Float>
        switch rotationDegrees {
        case 90:
            tl = SIMD2<Float>(0.0, 1.0)
            tr = SIMD2<Float>(0.0, 0.0)
            bl = SIMD2<Float>(1.0, 1.0)
            br = SIMD2<Float>(1.0, 0.0)
        case 180:
            tl = SIMD2<Float>(1.0, 1.0)
            tr = SIMD2<Float>(0.0, 1.0)
            bl = SIMD2<Float>(1.0, 0.0)
            br = SIMD2<Float>(0.0, 0.0)
        case 270:
            tl = SIMD2<Float>(1.0, 0.0)
            tr = SIMD2<Float>(1.0, 1.0)
            bl = SIMD2<Float>(0.0, 0.0)
            br = SIMD2<Float>(0.0, 1.0)
        default: // 0
            tl = SIMD2<Float>(0.0, 0.0)
            tr = SIMD2<Float>(1.0, 0.0)
            bl = SIMD2<Float>(0.0, 1.0)
            br = SIMD2<Float>(1.0, 1.0)
        }
        
        if flipHorizontal {
            swap(&tl, &tr)
            swap(&bl, &br)
        }
        if flipVertical {
            swap(&tl, &bl)
            swap(&tr, &br)
        }
        
        let vertices: [Vertex] = [
            Vertex(position: SIMD2<Float>(left,  top),    texCoord: tl),
            Vertex(position: SIMD2<Float>(right, top),    texCoord: tr),
            Vertex(position: SIMD2<Float>(left,  bottom), texCoord: bl),
            Vertex(position: SIMD2<Float>(right, bottom), texCoord: br),
        ]
        
        if let vb = self.vertexBuffer {
            let ptr = vb.contents().bindMemory(to: Vertex.self, capacity: vertices.count)
            for (idx, vertex) in vertices.enumerated() {
                ptr[idx] = vertex
            }
        }
        
        self.lastQuadBounds = (left, right, top, bottom)
        self.lastQuadUVs = (tl, tr, bl, br)
    }
    
    // MARK: - MTKViewDelegate
    public func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {
        updateQuad(viewSize: view.bounds.size)
    }
    
    public func draw(in view: MTKView) {
        guard let drawable = view.currentDrawable,
              let renderPassDesc = view.currentRenderPassDescriptor,
              let pipelineState = pipelineState,
              let texture = currentTexture,
              let vertexBuffer = vertexBuffer,
              let uniformsBuffer = uniformsBuffer else {
            return
        }
        
        let uPtr = uniformsBuffer.contents().bindMemory(to: ShaderUniforms.self, capacity: 1)
        uPtr.pointee = self.uniforms
        
        guard let commandBuffer = commandQueue.makeCommandBuffer(),
              let encoder = commandBuffer.makeRenderCommandEncoder(descriptor: renderPassDesc) else {
            return
        }
        
        encoder.setRenderPipelineState(pipelineState)
        encoder.setVertexBuffer(vertexBuffer, offset: 0, index: 0)
        encoder.setVertexBuffer(uniformsBuffer, offset: 0, index: 1)
        
        encoder.setFragmentTexture(texture, index: 0)
        encoder.setFragmentSamplerState(samplerState, index: 0)
        let activeLut = lutTexture ?? defaultLutTexture
        encoder.setFragmentTexture(activeLut, index: 1)
        encoder.setFragmentSamplerState(lutSamplerState ?? samplerState, index: 1)
        let activeMaster = curveMasterTexture ?? defaultCurveMasterTexture
        let activeRgb = curveRgbTexture ?? defaultCurveRgbTexture
        encoder.setFragmentTexture(activeMaster, index: 2)
        encoder.setFragmentTexture(activeRgb, index: 3)
        encoder.setFragmentSamplerState(curveSamplerState ?? samplerState, index: 2)
        let activeGainMap = gainMapTexture ?? defaultGainMapTexture
        encoder.setFragmentTexture(activeGainMap, index: 4)
        encoder.setFragmentSamplerState(gainMapSamplerState ?? samplerState, index: 3)
        encoder.setFragmentBuffer(uniformsBuffer, offset: 0, index: 1)
        
        encoder.drawPrimitives(type: .triangleStrip, vertexStart: 0, vertexCount: 4)
        encoder.endEncoding()
        
        commandBuffer.present(drawable)
        commandBuffer.commit()
    }
}
