import SwiftUI
import AppKit
import MetalKit

public struct MetalCanvasView: NSViewRepresentable {
    @Bindable var viewModel: AppViewModel
    let isShowingOriginal: Bool
    
    public init(viewModel: AppViewModel, isShowingOriginal: Bool) {
        self.viewModel = viewModel
        self.isShowingOriginal = isShowingOriginal
    }
    
    public func makeCoordinator() -> Coordinator {
        Coordinator()
    }
    
    public func makeNSView(context: Context) -> MTKView {
        guard let device = MTLCreateSystemDefaultDevice() else {
            return MTKView()
        }
        
        let mtkView = MTKView(frame: .zero, device: device)
        mtkView.colorPixelFormat = .bgra8Unorm
        mtkView.clearColor = MTLClearColor(red: 0.08, green: 0.08, blue: 0.08, alpha: 1.0)
        mtkView.enableSetNeedsDisplay = true
        mtkView.isPaused = true // Event-driven rendering: only redraws when sliders or image change!
        
        if let metalLayer = mtkView.layer as? CAMetalLayer {
            metalLayer.colorspace = CGColorSpace(name: CGColorSpace.sRGB)
        }
        
        if let renderer = MetalRenderer(device: device) {
            context.coordinator.renderer = renderer
            mtkView.delegate = renderer
            
            if let rawRecord = viewModel.rawPreviewRecord {
                renderer.updateLinearRawTexture(
                    width: Int(rawRecord.width),
                    height: Int(rawRecord.height),
                    data: rawRecord.data
                )
                context.coordinator.lastRawKey = "\(rawRecord.width)x\(rawRecord.height):\(rawRecord.data.count)"
            } else if let image = viewModel.previewImage {
                renderer.updateTexture(with: image)
            }
            
            if let lutName = viewModel.selectedLutName {
                let lutRecord = viewModel.resolveLutRecord(for: lutName)
                renderer.updateLutTexture(with: lutRecord)
                context.coordinator.lastLutName = lutName
            }
        }
        
        return mtkView
    }
    
    public func updateNSView(_ nsView: MTKView, context: Context) {
        if let metalLayer = nsView.layer as? CAMetalLayer, metalLayer.colorspace == nil {
            metalLayer.colorspace = CGColorSpace(name: CGColorSpace.sRGB)
        }
        guard let renderer = context.coordinator.renderer else { return }
        
        // 1. Update Texture if photo selection, raw record, or preview image changed
        let photoChanged = context.coordinator.lastPhotoPath != viewModel.selectedPhoto?.filePath
        let currentRawKey = viewModel.rawPreviewRecord.map { "\($0.width)x\($0.height):\($0.data.count)" }
        let rawChanged = context.coordinator.lastRawKey != currentRawKey
        let imageChanged = context.coordinator.lastPreviewImage !== viewModel.previewImage
        
        if photoChanged || rawChanged || imageChanged {
            if let rawRecord = viewModel.rawPreviewRecord {
                renderer.updateLinearRawTexture(
                    width: Int(rawRecord.width),
                    height: Int(rawRecord.height),
                    data: rawRecord.data
                )
                context.coordinator.lastRawKey = currentRawKey
            } else if let image = viewModel.previewImage {
                renderer.updateTexture(with: image)
                context.coordinator.lastRawKey = nil
            }
            context.coordinator.lastPhotoPath = viewModel.selectedPhoto?.filePath
            context.coordinator.lastPreviewImage = viewModel.previewImage
        }
        
        // 2. Transmit Slider & Lensfun Uniforms to GPU
        renderer.uniforms.exposure = viewModel.exposure
        renderer.uniforms.temperature = viewModel.temperature
        renderer.uniforms.tint = viewModel.tint
        renderer.uniforms.contrast = viewModel.contrast
        renderer.uniforms.straightenAngle = viewModel.straightenAngle * (.pi / 180.0)
        renderer.uniforms.lensCorrectionEnabled = viewModel.lensCorrectionEnabled ? 1 : 0
        renderer.uniforms.isOriginal = isShowingOriginal ? 1.0 : 0.0
        
        switch viewModel.toneProfile {
        case .agxVivid:
            renderer.uniforms.toneProfileMode = 0
        case .agxFilmic:
            renderer.uniforms.toneProfileMode = 1
        case .cameraStandard:
            renderer.uniforms.toneProfileMode = 2
        case .cameraPortrait:
            renderer.uniforms.toneProfileMode = 3
        case .cameraLandscape:
            renderer.uniforms.toneProfileMode = 4
        case .cameraVivid:
            renderer.uniforms.toneProfileMode = 5
        }
        
        if let coeff = viewModel.lensCoefficients, coeff.hasDistortion {
            renderer.uniforms.distortionModel = (coeff.distortionModel == "ptlens") ? 1 : 0
            renderer.uniforms.k1 = coeff.distortion.indices.contains(0) ? coeff.distortion[0] : 0.0
            renderer.uniforms.k2 = coeff.distortion.indices.contains(1) ? coeff.distortion[1] : 0.0
            renderer.uniforms.k3 = coeff.distortion.indices.contains(2) ? coeff.distortion[2] : 0.0
            renderer.uniforms.k4 = coeff.distortion.indices.contains(3) ? coeff.distortion[3] : 0.0
        } else {
            renderer.uniforms.k1 = 0.0
            renderer.uniforms.k2 = 0.0
            renderer.uniforms.k3 = 0.0
            renderer.uniforms.k4 = 0.0
        }
        
        // 3. Update 3D LUT texture if LUT selection changed
        let lutChanged = context.coordinator.lastLutName != viewModel.selectedLutName
        if lutChanged {
            context.coordinator.lastLutName = viewModel.selectedLutName
            if let lutName = viewModel.selectedLutName {
                let lutRecord = viewModel.resolveLutRecord(for: lutName)
                renderer.updateLutTexture(with: lutRecord)
            } else {
                renderer.updateLutTexture(with: nil)
            }
        }
        
        renderer.uniforms.lutEnabled = (viewModel.selectedLutName != nil) ? 1 : 0
        renderer.uniforms.lutIntensity = viewModel.lutIntensity
        
        // Trigger live redraw on MTKView
        nsView.setNeedsDisplay(nsView.bounds)
    }
    
    public class Coordinator {
        var renderer: MetalRenderer?
        var lastPhotoPath: String?
        var lastPreviewImage: NSImage?
        var lastRawKey: String?
        var lastLutName: String?
    }
}
