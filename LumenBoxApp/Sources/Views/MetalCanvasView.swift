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
            
            if let image = viewModel.previewImage {
                renderer.updateTexture(with: image)
            }
        }
        
        return mtkView
    }
    
    public func updateNSView(_ nsView: MTKView, context: Context) {
        if let metalLayer = nsView.layer as? CAMetalLayer, metalLayer.colorspace == nil {
            metalLayer.colorspace = CGColorSpace(name: CGColorSpace.sRGB)
        }
        guard let renderer = context.coordinator.renderer else { return }
        
        // 1. Update Texture if photo selection or preview image changed
        let photoChanged = context.coordinator.lastPhotoPath != viewModel.selectedPhoto?.filePath
        let imageChanged = context.coordinator.lastPreviewImage !== viewModel.previewImage
        
        if (photoChanged || imageChanged), let image = viewModel.previewImage {
            renderer.updateTexture(with: image)
            context.coordinator.lastPhotoPath = viewModel.selectedPhoto?.filePath
            context.coordinator.lastPreviewImage = image
        }
        
        // 2. Transmit Slider & Lensfun Uniforms to GPU
        renderer.uniforms.exposure = viewModel.exposure
        renderer.uniforms.temperature = viewModel.temperature
        renderer.uniforms.tint = viewModel.tint
        renderer.uniforms.contrast = viewModel.contrast
        renderer.uniforms.straightenAngle = viewModel.straightenAngle * (.pi / 180.0)
        renderer.uniforms.lensCorrectionEnabled = viewModel.lensCorrectionEnabled ? 1 : 0
        renderer.uniforms.isOriginal = isShowingOriginal ? 1.0 : 0.0
        
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
        
        // Trigger live redraw on MTKView
        nsView.setNeedsDisplay(nsView.bounds)
    }
    
    public class Coordinator {
        var renderer: MetalRenderer?
        var lastPhotoPath: String?
        var lastPreviewImage: NSImage?
    }
}
