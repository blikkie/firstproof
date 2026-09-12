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
        u.cropRight = 1.0
        u.cropBottom = 1.0
        u.cropConstrain = 1
        u.edrHeadroom = 1.0
        u.edrEnabled = 1
        return u
    }
}

public final class MetalRenderer: NSObject, MTKViewDelegate {
    public let device: MTLDevice
    private let commandQueue: MTLCommandQueue
    private var pipelineState: MTLRenderPipelineState?
    private var thumbnailPipelineState: MTLRenderPipelineState?
    private var samplerState: MTLSamplerState?
    private var lutSamplerState: MTLSamplerState?
    private var curveSamplerState: MTLSamplerState?
    private var vertexBuffer: MTLBuffer?
    private var uniformsBuffer: MTLBuffer?
    
    private var currentTexture: MTLTexture?
    private var lutTexture: MTLTexture?
    private var defaultLutTexture: MTLTexture?
    private var curveMasterTexture: MTLTexture?
    private var curveRgbTexture: MTLTexture?
    private var defaultCurveMasterTexture: MTLTexture?
    private var defaultCurveRgbTexture: MTLTexture?
    private var gainMapTexture: MTLTexture?
    private var defaultGainMapTexture: MTLTexture?
    private var gainMapSamplerState: MTLSamplerState?
    private var lastCurves: ToneCurvesRecord?
    private var textureLoader: MTKTextureLoader
    
    public var uniforms = ShaderUniforms()
    
    // MARK: - Centralized Uniforms & Geometry Update
    public func updateUniforms(
        from edits: EditSettings,
        imageAspect: Float,
        isLinearRaw: Bool,
        isOriginal: Bool = false,
        isCropToolActive: Bool = false,
        lensCorrectionEnabled: Bool = true,
        lensCoefficients: LensCorrectionCoefficientsRecord? = nil,
        edrHeadroom: Float = 1.0,
        edrEnabled: Bool = true
    ) {
        uniforms.exposure = edits.exposure
        uniforms.temperature = edits.temperature
        uniforms.tint = edits.tint
        uniforms.contrast = edits.contrast
        uniforms.edrHeadroom = edrHeadroom
        uniforms.edrEnabled = edrEnabled ? 1 : 0
        uniforms.highlights = edits.highlights
        uniforms.shadows = edits.shadows
        uniforms.whites = edits.whites
        uniforms.blacks = edits.blacks
        uniforms.vibrance = edits.vibrance
        uniforms.saturation = edits.saturation
        uniforms.deepBlue = edits.deepBlue
        uniforms.foliage = edits.foliage
        uniforms.straightenAngle = edits.cropAngle * (.pi / 180.0)
        uniforms.aspectRatio = imageAspect
        uniforms.lensCorrectionEnabled = lensCorrectionEnabled ? 1 : 0
        uniforms.isOriginal = isOriginal ? 1.0 : 0.0
        uniforms.isLinearRaw = isLinearRaw ? 1 : 0
        uniforms.lutEnabled = (edits.lutName != nil) ? 1 : 0
        uniforms.lutIntensity = edits.lutIntensity
        uniforms.sharpeningAmount = edits.sharpeningAmount
        uniforms.sharpeningRadius = edits.sharpeningRadius
        uniforms.sharpeningMasking = edits.sharpeningMasking
        uniforms.noiseReduction = edits.noiseReduction
        uniforms.grainAmount = edits.grainAmount
        uniforms.grainSize = edits.grainSize
        uniforms.grainRoughness = edits.grainRoughness
        uniforms.grainChroma = edits.grainChroma
        uniforms.cropLeft = edits.cropLeft
        uniforms.cropTop = edits.cropTop
        uniforms.cropRight = edits.cropRight
        uniforms.cropBottom = edits.cropBottom
        uniforms.perspectiveVertical = edits.perspectiveVertical
        uniforms.perspectiveHorizontal = edits.perspectiveHorizontal
        uniforms.cropConstrain = edits.cropConstrain ? 1 : 0
        uniforms.cropModeActive = isCropToolActive ? 1 : 0
        
        switch edits.toneProfile {
        case .agxVivid:
            uniforms.toneProfileMode = 0
        case .agxFilmic:
            uniforms.toneProfileMode = 1
        case .cameraStandard:
            uniforms.toneProfileMode = 2
        case .cameraPortrait:
            uniforms.toneProfileMode = 3
        case .cameraLandscape:
            uniforms.toneProfileMode = 4
        case .cameraVivid:
            uniforms.toneProfileMode = 5
        }
        
        if let coeff = lensCoefficients, coeff.hasDistortion {
            uniforms.distortionModel = (coeff.distortionModel == "ptlens") ? 1 : 0
            uniforms.k1 = coeff.distortion.indices.contains(0) ? coeff.distortion[0] : 0.0
            uniforms.k2 = coeff.distortion.indices.contains(1) ? coeff.distortion[1] : 0.0
            uniforms.k3 = coeff.distortion.indices.contains(2) ? coeff.distortion[2] : 0.0
            uniforms.k4 = coeff.distortion.indices.contains(3) ? coeff.distortion[3] : 0.0
        } else {
            uniforms.distortionModel = 0
            uniforms.k1 = 0.0
            uniforms.k2 = 0.0
            uniforms.k3 = 0.0
            uniforms.k4 = 0.0
        }
        
        self.rotationDegrees = Int(edits.rotation)
        self.flipHorizontal = edits.flipHorizontal
        self.flipVertical = edits.flipVertical
        
        let isMasterActive = edits.curves.master.count > 2 || (edits.curves.master.count == 2 && (edits.curves.master[0].y != 0.0 || edits.curves.master[1].y != 1.0))
        let isRedActive = edits.curves.red.count > 2 || (edits.curves.red.count == 2 && (edits.curves.red[0].y != 0.0 || edits.curves.red[1].y != 1.0))
        let isGreenActive = edits.curves.green.count > 2 || (edits.curves.green.count == 2 && (edits.curves.green[0].y != 0.0 || edits.curves.green[1].y != 1.0))
        let isBlueActive = edits.curves.blue.count > 2 || (edits.curves.blue.count == 2 && (edits.curves.blue[0].y != 0.0 || edits.curves.blue[1].y != 1.0))
        let hasRgb = isRedActive || isGreenActive || isBlueActive
        let hasCurves = isMasterActive || hasRgb
        uniforms.hasMasterCurve = isMasterActive ? 1 : 0
        uniforms.hasRgbCurves = hasRgb ? 1 : 0
        uniforms.hasCurves = hasCurves ? 1 : 0
        
        if lastCurves != edits.curves {
            lastCurves = edits.curves
            updateCurves(curves: edits.curves)
        }
    }
    
    // MARK: - Viewport Zoom, Pan & Hit-Testing
    public var zoomScale: Float = 1.0
    public var panOffset: SIMD2<Float> = SIMD2<Float>(0.0, 0.0)
    public private(set) var zoomScale100: Float = 1.0
    public var onZoomScale100Changed: ((Float) -> Void)?
    public private(set) var lastQuadBounds: (left: Float, right: Float, top: Float, bottom: Float) = (-1.0, 1.0, 1.0, -1.0)
    public private(set) var lastQuadUVs: (tl: SIMD2<Float>, tr: SIMD2<Float>, bl: SIMD2<Float>, br: SIMD2<Float>) = (SIMD2<Float>(0, 0), SIMD2<Float>(1, 0), SIMD2<Float>(0, 1), SIMD2<Float>(1, 1))
    
    public func normalizedTextureCoordinates(atNDC pt: CGPoint) -> CGPoint? {
        let x = Float(pt.x)
        let y = Float(pt.y)
        let (left, right, top, bottom) = lastQuadBounds
        guard x >= left, x <= right, y >= bottom, y <= top else { return nil }
        
        let uQuad = (x - left) / max(right - left, 1e-6)
        let vQuad = (top - y) / max(top - bottom, 1e-6)
        
        let (tl, tr, bl, br) = lastQuadUVs
        let uvTop = (1.0 - uQuad) * tl + uQuad * tr
        let uvBottom = (1.0 - uQuad) * bl + uQuad * br
        let uv = (1.0 - vQuad) * uvTop + vQuad * uvBottom
        
        return CGPoint(x: CGFloat(min(max(uv.x, 0.0), 1.0)), y: CGFloat(min(max(uv.y, 0.0), 1.0)))
    }
    
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
            if let shaderURL = findShaderURL(),
               let rawSource = try? String(contentsOf: shaderURL, encoding: .utf8) {
                let resolvedSource = resolveIncludes(in: rawSource, relativeTo: shaderURL.deletingLastPathComponent())
                do {
                    metalLibrary = try device.makeLibrary(source: resolvedSource, options: nil)
                } catch {
                    print("⚠️ Warning: Failed to compile Shaders.metal: \(error)")
                }
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
        pipelineDescriptor.colorAttachments[0].pixelFormat = .rgba16Float
        
        do {
            self.pipelineState = try device.makeRenderPipelineState(descriptor: pipelineDescriptor)
        } catch {
            print("Failed to create Metal render pipeline state: \(error)")
        }
        
        let thumbPipelineDescriptor = MTLRenderPipelineDescriptor()
        thumbPipelineDescriptor.vertexFunction = vertexFunction
        thumbPipelineDescriptor.fragmentFunction = fragmentFunction
        thumbPipelineDescriptor.colorAttachments[0].pixelFormat = .bgra8Unorm
        
        do {
            self.thumbnailPipelineState = try device.makeRenderPipelineState(descriptor: thumbPipelineDescriptor)
        } catch {
            print("Failed to create Metal thumbnail render pipeline state: \(error)")
        }
    }
    
    private func findShaderURL() -> URL? {
        if let bundleURL = Bundle.main.url(forResource: "Shaders", withExtension: "metal") {
            return bundleURL
        }
        let sourceURL = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .appendingPathComponent("Shaders/Shaders.metal")
        if FileManager.default.fileExists(atPath: sourceURL.path) {
            return sourceURL
        }
        return nil
    }
    
    private func resolveIncludes(in source: String, relativeTo baseDir: URL) -> String {
        var result = ""
        for line in source.components(separatedBy: .newlines) {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            if trimmed.hasPrefix("#include \"") && trimmed.hasSuffix("\"") {
                let includeName = String(trimmed.dropFirst(10).dropLast(1))
                let incURL = baseDir.appendingPathComponent(includeName)
                if FileManager.default.fileExists(atPath: incURL.path),
                   let incContent = try? String(contentsOf: incURL, encoding: .utf8) {
                    result += resolveIncludes(in: incContent, relativeTo: incURL.deletingLastPathComponent()) + "\n"
                    continue
                }
            }
            result += line + "\n"
        }
        return result
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
        
        let lutSamplerDescriptor = MTLSamplerDescriptor()
        lutSamplerDescriptor.minFilter = .linear
        lutSamplerDescriptor.magFilter = .linear
        lutSamplerDescriptor.rAddressMode = .clampToEdge
        lutSamplerDescriptor.sAddressMode = .clampToEdge
        lutSamplerDescriptor.tAddressMode = .clampToEdge
        self.lutSamplerState = device.makeSamplerState(descriptor: lutSamplerDescriptor)
        
        let gmSamplerDescriptor = MTLSamplerDescriptor()
        gmSamplerDescriptor.minFilter = .linear
        gmSamplerDescriptor.magFilter = .linear
        gmSamplerDescriptor.sAddressMode = .clampToEdge
        gmSamplerDescriptor.tAddressMode = .clampToEdge
        self.gainMapSamplerState = device.makeSamplerState(descriptor: gmSamplerDescriptor)
        
        setupDefaultLutTexture()
        setupCurveTexturesAndSampler()
        setupDefaultGainMapTexture()
    }
    
    private func setupDefaultGainMapTexture() {
        let desc = MTLTextureDescriptor.texture2DDescriptor(pixelFormat: .r8Unorm, width: 1, height: 1, mipmapped: false)
        desc.usage = [.shaderRead]
        if let tex = device.makeTexture(descriptor: desc) {
            var zero: UInt8 = 0
            tex.replace(region: MTLRegionMake2D(0, 0, 1, 1), mipmapLevel: 0, withBytes: &zero, bytesPerRow: 1)
            self.defaultGainMapTexture = tex
        }
    }
    
    private func setupDefaultLutTexture() {
        let desc = MTLTextureDescriptor()
        desc.textureType = .type3D
        desc.pixelFormat = .rgba32Float
        desc.width = 2
        desc.height = 2
        desc.depth = 2
        desc.usage = [.shaderRead]
        
        guard let tex = device.makeTexture(descriptor: desc) else { return }
        var identityData: [Float] = []
        for b in 0..<2 {
            let bVal = Float(b)
            for g in 0..<2 {
                let gVal = Float(g)
                for r in 0..<2 {
                    let rVal = Float(r)
                    identityData.append(contentsOf: [rVal, gVal, bVal, 1.0])
                }
            }
        }
        identityData.withUnsafeBytes { raw in
            guard let ptr = raw.baseAddress else { return }
            tex.replace(
                region: MTLRegion(origin: MTLOrigin(x: 0, y: 0, z: 0), size: MTLSize(width: 2, height: 2, depth: 2)),
                mipmapLevel: 0,
                slice: 0,
                withBytes: ptr,
                bytesPerRow: 2 * 4 * MemoryLayout<Float>.size,
                bytesPerImage: 2 * 2 * 4 * MemoryLayout<Float>.size
            )
        }
        self.defaultLutTexture = tex
    }
    
    private func setupCurveTexturesAndSampler() {
        let samplerDesc = MTLSamplerDescriptor()
        samplerDesc.minFilter = .linear
        samplerDesc.magFilter = .linear
        samplerDesc.sAddressMode = .clampToEdge
        self.curveSamplerState = device.makeSamplerState(descriptor: samplerDesc)

        let masterDesc = MTLTextureDescriptor()
        masterDesc.textureType = .type1D
        masterDesc.pixelFormat = .r32Float
        masterDesc.width = 256
        masterDesc.height = 1
        masterDesc.depth = 1
        masterDesc.usage = [.shaderRead]
        self.defaultCurveMasterTexture = device.makeTexture(descriptor: masterDesc)
        self.curveMasterTexture = device.makeTexture(descriptor: masterDesc)

        var linearRamp = [Float](repeating: 0, count: 256)
        for i in 0..<256 {
            linearRamp[i] = Float(i) / 255.0
        }
        defaultCurveMasterTexture?.replace(
            region: MTLRegion(origin: MTLOrigin(x: 0, y: 0, z: 0), size: MTLSize(width: 256, height: 1, depth: 1)),
            mipmapLevel: 0,
            withBytes: linearRamp,
            bytesPerRow: 256 * MemoryLayout<Float>.size
        )
        curveMasterTexture?.replace(
            region: MTLRegion(origin: MTLOrigin(x: 0, y: 0, z: 0), size: MTLSize(width: 256, height: 1, depth: 1)),
            mipmapLevel: 0,
            withBytes: linearRamp,
            bytesPerRow: 256 * MemoryLayout<Float>.size
        )

        let rgbDesc = MTLTextureDescriptor()
        rgbDesc.textureType = .type1D
        rgbDesc.pixelFormat = .rgba32Float
        rgbDesc.width = 256
        rgbDesc.height = 1
        rgbDesc.depth = 1
        rgbDesc.usage = [.shaderRead]
        self.defaultCurveRgbTexture = device.makeTexture(descriptor: rgbDesc)
        self.curveRgbTexture = device.makeTexture(descriptor: rgbDesc)

        var rgbRamp = [SIMD4<Float>](repeating: .zero, count: 256)
        for i in 0..<256 {
            let v = Float(i) / 255.0
            rgbRamp[i] = SIMD4<Float>(v, v, v, 1.0)
        }
        defaultCurveRgbTexture?.replace(
            region: MTLRegion(origin: MTLOrigin(x: 0, y: 0, z: 0), size: MTLSize(width: 256, height: 1, depth: 1)),
            mipmapLevel: 0,
            withBytes: rgbRamp,
            bytesPerRow: 256 * MemoryLayout<SIMD4<Float>>.stride
        )
        curveRgbTexture?.replace(
            region: MTLRegion(origin: MTLOrigin(x: 0, y: 0, z: 0), size: MTLSize(width: 256, height: 1, depth: 1)),
            mipmapLevel: 0,
            withBytes: rgbRamp,
            bytesPerRow: 256 * MemoryLayout<SIMD4<Float>>.stride
        )
    }

    public func updateCurves(curves: ToneCurvesRecord) {
        let lut = evaluateAllCurvesLut(curves: curves)
        if let masterTex = curveMasterTexture {
            masterTex.replace(
                region: MTLRegion(origin: MTLOrigin(x: 0, y: 0, z: 0), size: MTLSize(width: 256, height: 1, depth: 1)),
                mipmapLevel: 0,
                withBytes: lut.master,
                bytesPerRow: 256 * MemoryLayout<Float>.size
            )
        }
        if let rgbTex = curveRgbTexture {
            var rgba = [SIMD4<Float>](repeating: .zero, count: 256)
            for i in 0..<256 {
                rgba[i] = SIMD4<Float>(lut.red[i], lut.green[i], lut.blue[i], 1.0)
            }
            rgbTex.replace(
                region: MTLRegion(origin: MTLOrigin(x: 0, y: 0, z: 0), size: MTLSize(width: 256, height: 1, depth: 1)),
                mipmapLevel: 0,
                withBytes: rgba,
                bytesPerRow: 256 * MemoryLayout<SIMD4<Float>>.stride
            )
        }
    }
    
    public func updateLutTexture(with lutRecord: CubeLutRecord?) {
        guard let lut = lutRecord, lut.size >= 2 else {
            self.lutTexture = nil
            self.uniforms.lutEnabled = 0
            return
        }
        
        let size = Int(lut.size)
        let desc = MTLTextureDescriptor()
        desc.textureType = .type3D
        desc.pixelFormat = .rgba32Float
        desc.width = size
        desc.height = size
        desc.depth = size
        desc.usage = [.shaderRead]
        
        guard let tex = device.makeTexture(descriptor: desc) else { return }
        lut.dataRgba32f.withUnsafeBytes { raw in
            guard let ptr = raw.baseAddress else { return }
            tex.replace(
                region: MTLRegion(origin: MTLOrigin(x: 0, y: 0, z: 0), size: MTLSize(width: size, height: size, depth: size)),
                mipmapLevel: 0,
                slice: 0,
                withBytes: ptr,
                bytesPerRow: size * 4 * MemoryLayout<Float>.size,
                bytesPerImage: size * size * 4 * MemoryLayout<Float>.size
            )
        }
        self.lutTexture = tex
        self.uniforms.lutEnabled = 1
        self.uniforms.lutSize = Float(size)
    }
    
    // MARK: - Ultra HDR Gain Map Upload
    public func updateGainMap(record: GainMapRecord?) {
        guard let record = record,
              let imageSource = CGImageSourceCreateWithData(record.jpegBytes as CFData, nil),
              let cgImage = CGImageSourceCreateImageAtIndex(imageSource, 0, nil) else {
            self.gainMapTexture = nil
            self.uniforms.hasGainMap = 0
            return
        }
        
        let width = cgImage.width
        let height = cgImage.height
        guard width > 0, height > 0 else { return }
        
        let bytesPerRow = width
        let totalBytes = bytesPerRow * height
        guard let pixelBuffer = malloc(totalBytes) else { return }
        defer { free(pixelBuffer) }
        
        let colorSpace = CGColorSpaceCreateDeviceGray()
        if let ctx = CGContext(
            data: pixelBuffer,
            width: width,
            height: height,
            bitsPerComponent: 8,
            bytesPerRow: bytesPerRow,
            space: colorSpace,
            bitmapInfo: CGImageAlphaInfo.none.rawValue
        ) {
            ctx.draw(cgImage, in: CGRect(x: 0, y: 0, width: width, height: height))
            
            let texDesc = MTLTextureDescriptor.texture2DDescriptor(
                pixelFormat: .r8Unorm,
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
                self.gainMapTexture = texture
                self.uniforms.hasGainMap = 1
                self.uniforms.gainMapMin = record.metadata.minGain
                self.uniforms.gainMapMax = record.metadata.maxGain
                self.uniforms.gainMapGamma = record.metadata.gamma
                self.uniforms.gainMapCapacityMax = record.metadata.hdrCapacityMax
                return
            }
        }
        
        self.gainMapTexture = nil
        self.uniforms.hasGainMap = 0
    }
    
    // MARK: - Aspect-Fit, Zoom & Pan Quad Update
    public func updateQuad(viewSize: CGSize) {
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
        
        let isOrthogonal = (rotationDegrees == 90 || rotationDegrees == 270)
        let displayAspect = isOrthogonal ? (1.0 / imageAspect) : imageAspect
        
        let viewAspect = Float(viewSize.width) / Float(viewSize.height)
        
        var quadW: Float = 1.0
        var quadH: Float = 1.0
        
        if displayAspect > viewAspect {
            // Image is wider than viewport: fit width, letterbox top/bottom
            quadW = 1.0
            quadH = viewAspect / displayAspect
        } else {
            // Image is taller than viewport: fit height, pillarbox left/right
            quadW = displayAspect / viewAspect
            quadH = 1.0
        }
        
        // Compute 1:1 Pixel Mapping Scale (where 1 image pixel = 1 screen pixel)
        let displayPixelsW = isOrthogonal ? Float(texture.height) : Float(texture.width)
        let fitWidthPixels = Float(viewSize.width) * quadW
        let fitScale = displayPixelsW / max(fitWidthPixels, 1.0)
        let newZ100 = max(1.0, fitScale)
        if abs(self.zoomScale100 - newZ100) > 0.01 {
            self.zoomScale100 = newZ100
            onZoomScale100Changed?(newZ100)
        }
        
        // Apply zoom scale
        let currentScale = max(1.0, zoomScale)
        let scaledW = quadW * currentScale
        let scaledH = quadH * currentScale
        
        // Clamp pan offsets so the image edge cannot be dragged past the viewport bounds
        let maxPanX = max(0.0, scaledW - 1.0)
        let maxPanY = max(0.0, scaledH - 1.0)
        panOffset.x = max(-maxPanX, min(maxPanX, panOffset.x))
        panOffset.y = max(-maxPanY, min(maxPanY, panOffset.y))
        
        let left   = -scaledW + panOffset.x
        let right  =  scaledW + panOffset.x
        let top    =  scaledH + panOffset.y
        let bottom = -scaledH + panOffset.y
        
        // Derive vertex texture coordinates according to 90° rotation and horizontal/vertical flips
        var (tl, tr, bl, br): (SIMD2<Float>, SIMD2<Float>, SIMD2<Float>, SIMD2<Float>)
        switch (rotationDegrees % 360 + 360) % 360 {
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
            Vertex(position: SIMD2<Float>(left,   top), texCoord: tl),
            Vertex(position: SIMD2<Float>(right,  top), texCoord: tr),
            Vertex(position: SIMD2<Float>(left,  bottom), texCoord: bl),
            Vertex(position: SIMD2<Float>(right, bottom), texCoord: br)
        ]
        
        if let vertexBuffer = vertexBuffer {
            let ptr = vertexBuffer.contents().bindMemory(to: Vertex.self, capacity: vertices.count)
            for (idx, vertex) in vertices.enumerated() {
                ptr[idx] = vertex
            }
        }
        
        self.lastQuadBounds = (left, right, top, bottom)
        self.lastQuadUVs = (tl, tr, bl, br)
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
        if let metalLayer = view.layer as? CAMetalLayer {
            let wantsEdr = uniforms.edrEnabled != 0 && uniforms.edrHeadroom > 1.001
            if metalLayer.wantsExtendedDynamicRangeContent != wantsEdr {
                metalLayer.wantsExtendedDynamicRangeContent = wantsEdr
                metalLayer.colorspace = wantsEdr
                    ? CGColorSpace(name: CGColorSpace.extendedLinearDisplayP3)
                    : CGColorSpace(name: CGColorSpace.sRGB)
            }
        }
        
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
    
    // MARK: - Offscreen Real-Time Thumbnail Snapshot
    public func renderThumbnailSnapshot(maxDimension: CGFloat = 320) -> NSImage? {
        guard let texture = currentTexture,
              let pipelineState = pipelineState else {
            return nil
        }
        
        let origW = texture.width
        let origH = texture.height
        guard origW > 0, origH > 0 else { return nil }
        
        let isOrthogonal = (rotationDegrees == 90 || rotationDegrees == 270)
        let cropWFrac = max(CGFloat(uniforms.cropRight - uniforms.cropLeft), 0.01)
        let cropHFrac = max(CGFloat(uniforms.cropBottom - uniforms.cropTop), 0.01)
        let effectiveOrigW = CGFloat(origW) * cropWFrac
        let effectiveOrigH = CGFloat(origH) * cropHFrac
        let dispW = isOrthogonal ? effectiveOrigH : effectiveOrigW
        let dispH = isOrthogonal ? effectiveOrigW : effectiveOrigH
        
        let targetW: Int
        let targetH: Int
        if dispW >= dispH {
            targetW = Int(maxDimension)
            targetH = max(Int((CGFloat(dispH) * maxDimension / CGFloat(dispW)).rounded()), 1)
        } else {
            targetH = Int(maxDimension)
            targetW = max(Int((CGFloat(dispW) * maxDimension / CGFloat(dispH)).rounded()), 1)
        }
        
        let desc = MTLTextureDescriptor.texture2DDescriptor(
            pixelFormat: .bgra8Unorm,
            width: targetW,
            height: targetH,
            mipmapped: false
        )
        desc.usage = [.renderTarget, .shaderRead]
        desc.storageMode = .shared
        
        guard let offscreenTexture = device.makeTexture(descriptor: desc) else { return nil }
        
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
            Vertex(position: SIMD2<Float>(-1.0,  1.0), texCoord: tl),
            Vertex(position: SIMD2<Float>( 1.0,  1.0), texCoord: tr),
            Vertex(position: SIMD2<Float>(-1.0, -1.0), texCoord: bl),
            Vertex(position: SIMD2<Float>( 1.0, -1.0), texCoord: br),
        ]
        
        guard let quadBuffer = device.makeBuffer(
            bytes: vertices,
            length: vertices.count * MemoryLayout<Vertex>.size,
            options: .storageModeShared
        ) else { return nil }
        
        var thumbUniforms = self.uniforms
        thumbUniforms.isOriginal = 0.0
        thumbUniforms.cropModeActive = 0
        thumbUniforms.aspectRatio = Float(origW) / Float(origH)
        thumbUniforms.edrEnabled = 0
        thumbUniforms.edrHeadroom = 1.0
        
        guard let thumbUniformsBuffer = device.makeBuffer(
            bytes: &thumbUniforms,
            length: MemoryLayout<ShaderUniforms>.size,
            options: .storageModeShared
        ) else { return nil }
        
        let passDesc = MTLRenderPassDescriptor()
        passDesc.colorAttachments[0].texture = offscreenTexture
        passDesc.colorAttachments[0].loadAction = .clear
        passDesc.colorAttachments[0].clearColor = MTLClearColor(red: 0, green: 0, blue: 0, alpha: 1)
        passDesc.colorAttachments[0].storeAction = .store
        
        guard let commandBuffer = commandQueue.makeCommandBuffer(),
              let encoder = commandBuffer.makeRenderCommandEncoder(descriptor: passDesc) else {
            return nil
        }
        
        let pState = thumbnailPipelineState ?? pipelineState
        encoder.setRenderPipelineState(pState)
        encoder.setVertexBuffer(quadBuffer, offset: 0, index: 0)
        encoder.setVertexBuffer(thumbUniformsBuffer, offset: 0, index: 1)
        
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
        encoder.setFragmentBuffer(thumbUniformsBuffer, offset: 0, index: 1)
        
        encoder.drawPrimitives(type: .triangleStrip, vertexStart: 0, vertexCount: 4)
        encoder.endEncoding()
        
        commandBuffer.commit()
        commandBuffer.waitUntilCompleted()
        
        let bytesPerRow = targetW * 4
        var rawBytes = [UInt8](repeating: 0, count: bytesPerRow * targetH)
        rawBytes.withUnsafeMutableBytes { ptr in
            offscreenTexture.getBytes(
                ptr.baseAddress!,
                bytesPerRow: bytesPerRow,
                from: MTLRegionMake2D(0, 0, targetW, targetH),
                mipmapLevel: 0
            )
        }
        
        guard let provider = CGDataProvider(data: Data(rawBytes) as CFData) else { return nil }
        guard let cgImg = CGImage(
            width: targetW,
            height: targetH,
            bitsPerComponent: 8,
            bitsPerPixel: 32,
            bytesPerRow: bytesPerRow,
            space: CGColorSpace(name: CGColorSpace.sRGB)!,
            bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedFirst.rawValue | CGBitmapInfo.byteOrder32Little.rawValue),
            provider: provider,
            decode: nil,
            shouldInterpolate: true,
            intent: .defaultIntent
        ) else { return nil }
        
        return NSImage(cgImage: cgImg, size: NSSize(width: targetW, height: targetH))
    }
}
