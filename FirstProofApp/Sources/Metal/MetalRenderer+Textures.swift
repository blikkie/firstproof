import Foundation
import Metal
import MetalKit
import AppKit
import FirstProofBridge
import FirstProofShaderTypes

extension MetalRenderer {
    // MARK: - Texture Initializers
    func setupDefaultGainMapTexture() {
        let desc = MTLTextureDescriptor.texture2DDescriptor(pixelFormat: .r8Unorm, width: 1, height: 1, mipmapped: false)
        desc.usage = [.shaderRead]
        if let tex = device.makeTexture(descriptor: desc) {
            var zero: UInt8 = 0
            tex.replace(region: MTLRegionMake2D(0, 0, 1, 1), mipmapLevel: 0, withBytes: &zero, bytesPerRow: 1)
            self.defaultGainMapTexture = tex
        }
    }
    
    func setupDefaultLutTexture() {
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
    
    func setupCurveTexturesAndSampler() {
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

    // MARK: - Texture Updates
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
}
