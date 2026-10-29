import Foundation
import Metal
import MetalKit
import AppKit
import FirstProofBridge
import FirstProofShaderTypes

extension MetalRenderer {
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
        let activeProof = proofLutTexture ?? defaultProofLutTexture
        encoder.setFragmentTexture(activeProof, index: 5)
        encoder.setFragmentSamplerState(proofSamplerState ?? samplerState, index: 4)
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
