import SwiftUI
import AppKit
import MetalKit
import LumenBoxBridge

public struct MetalCanvasView: NSViewRepresentable {
    @Bindable var viewModel: AppViewModel
    let exposure: Float
    let contrast: Float
    let temperature: Float
    let tint: Float
    let straightenAngle: Float
    let lensCorrectionEnabled: Bool
    let selectedLutName: String?
    let lutIntensity: Float
    let toneProfile: ColorProfileMode
    let rotationDegrees: Int
    let flipHorizontal: Bool
    let flipVertical: Bool
    let zoomScale: Float
    let panOffset: CGPoint
    let isShowingOriginal: Bool
    
    public init(
        viewModel: AppViewModel,
        exposure: Float,
        contrast: Float,
        temperature: Float,
        tint: Float,
        straightenAngle: Float,
        lensCorrectionEnabled: Bool,
        selectedLutName: String?,
        lutIntensity: Float,
        toneProfile: ColorProfileMode,
        rotationDegrees: Int,
        flipHorizontal: Bool,
        flipVertical: Bool,
        zoomScale: Float,
        panOffset: CGPoint,
        isShowingOriginal: Bool
    ) {
        self.viewModel = viewModel
        self.exposure = exposure
        self.contrast = contrast
        self.temperature = temperature
        self.tint = tint
        self.straightenAngle = straightenAngle
        self.lensCorrectionEnabled = lensCorrectionEnabled
        self.selectedLutName = selectedLutName
        self.lutIntensity = lutIntensity
        self.toneProfile = toneProfile
        self.rotationDegrees = rotationDegrees
        self.flipHorizontal = flipHorizontal
        self.flipVertical = flipVertical
        self.zoomScale = zoomScale
        self.panOffset = panOffset
        self.isShowingOriginal = isShowingOriginal
    }
    
    public func makeCoordinator() -> Coordinator {
        Coordinator()
    }
    
    public func makeNSView(context: Context) -> CanvasMTKView {
        guard let device = MTLCreateSystemDefaultDevice() else {
            return CanvasMTKView()
        }
        
        let mtkView = CanvasMTKView(frame: .zero, device: device)
        mtkView.colorPixelFormat = .bgra8Unorm
        mtkView.clearColor = MTLClearColor(red: 0.08, green: 0.08, blue: 0.08, alpha: 1.0)
        mtkView.enableSetNeedsDisplay = true
        mtkView.isPaused = true // Event-driven rendering: only redraws when sliders, gestures, or image change!
        mtkView.coordinator = context.coordinator
        context.coordinator.nsView = mtkView
        context.coordinator.viewModel = viewModel
        
        if let metalLayer = mtkView.layer as? CAMetalLayer {
            metalLayer.colorspace = CGColorSpace(name: CGColorSpace.sRGB)
        }
        
        if let renderer = MetalRenderer(device: device) {
            context.coordinator.renderer = renderer
            mtkView.delegate = renderer
            
            renderer.onZoomScale100Changed = { [weak viewModel] newZ100 in
                DispatchQueue.main.async {
                    viewModel?.zoomScale100 = newZ100
                }
            }
            
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
            
            if let lutName = selectedLutName {
                let lutRecord = viewModel.resolveLutRecord(for: lutName)
                renderer.updateLutTexture(with: lutRecord)
                context.coordinator.lastLutName = lutName
            }
        }
        
        return mtkView
    }
    
    public func updateNSView(_ nsView: CanvasMTKView, context: Context) {
        if let metalLayer = nsView.layer as? CAMetalLayer, metalLayer.colorspace == nil {
            metalLayer.colorspace = CGColorSpace(name: CGColorSpace.sRGB)
        }
        guard let renderer = context.coordinator.renderer else { return }
        context.coordinator.viewModel = viewModel
        
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
        renderer.uniforms.exposure = exposure
        renderer.uniforms.temperature = temperature
        renderer.uniforms.tint = tint
        renderer.uniforms.contrast = contrast
        renderer.uniforms.straightenAngle = straightenAngle * (.pi / 180.0)
        renderer.uniforms.lensCorrectionEnabled = lensCorrectionEnabled ? 1 : 0
        renderer.uniforms.isOriginal = isShowingOriginal ? 1.0 : 0.0
        
        switch toneProfile {
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
        let lutChanged = context.coordinator.lastLutName != selectedLutName
        if lutChanged {
            context.coordinator.lastLutName = selectedLutName
            if let lutName = selectedLutName {
                let lutRecord = viewModel.resolveLutRecord(for: lutName)
                renderer.updateLutTexture(with: lutRecord)
            } else {
                renderer.updateLutTexture(with: nil)
            }
        }
        
        renderer.uniforms.lutEnabled = (selectedLutName != nil) ? 1 : 0
        renderer.uniforms.lutIntensity = lutIntensity
        
        // 4. Viewport Geometry, Zoom & Pan
        renderer.rotationDegrees = rotationDegrees
        renderer.flipHorizontal = flipHorizontal
        renderer.flipVertical = flipVertical
        renderer.zoomScale = zoomScale
        renderer.panOffset = SIMD2<Float>(Float(panOffset.x), Float(panOffset.y))
        renderer.updateQuad(viewSize: nsView.drawableSize)
        
        if abs(viewModel.zoomScale100 - renderer.zoomScale100) > 0.01 {
            let z100 = renderer.zoomScale100
            DispatchQueue.main.async {
                self.viewModel.zoomScale100 = z100
            }
        }
        
        // Trigger immediate synchronous draw call on MTKView
        nsView.draw()
    }
    
    public class Coordinator {
        var renderer: MetalRenderer?
        weak var nsView: CanvasMTKView?
        var viewModel: AppViewModel?
        var lastPhotoPath: String?
        var lastPreviewImage: NSImage?
        var lastRawKey: String?
        var lastLutName: String?
        
        func handleDoubleTap(at pointInNDC: CGPoint) {
            guard let vm = viewModel, let renderer = renderer, let view = nsView else { return }
            if vm.isZoomed {
                vm.zoomToFit()
                renderer.zoomScale = 1.0
                renderer.panOffset = .zero
            } else {
                let base = renderer.zoomScale100 > 1.05 ? renderer.zoomScale100 : 3.0
                let target200 = base * 2.0 // Double-click targets 200% Retina Loupe
                vm.zoomScale = target200
                renderer.zoomScale = target200
                
                let ratio = CGFloat(target200 - 1.0)
                let targetPan = CGPoint(x: -pointInNDC.x * ratio, y: -pointInNDC.y * ratio)
                vm.panOffset = targetPan
                renderer.panOffset = SIMD2<Float>(Float(targetPan.x), Float(targetPan.y))
            }
            renderer.updateQuad(viewSize: view.drawableSize)
            view.draw()
        }
        
        func handlePanDelta(dx: Float, dy: Float) {
            guard let renderer = renderer, let view = nsView, let vm = viewModel else { return }
            renderer.panOffset.x += dx
            renderer.panOffset.y += dy
            renderer.updateQuad(viewSize: view.drawableSize)
            vm.panOffset = CGPoint(x: CGFloat(renderer.panOffset.x), y: CGFloat(renderer.panOffset.y))
            view.draw()
        }
        
        func handlePinch(scaleMultiplier: Float) {
            guard let renderer = renderer, let view = nsView, let vm = viewModel else { return }
            let newScale = max(1.0, min(16.0, renderer.zoomScale * scaleMultiplier))
            renderer.zoomScale = newScale
            if newScale <= 1.05 {
                renderer.panOffset = .zero
            }
            renderer.updateQuad(viewSize: view.drawableSize)
            vm.zoomScale = newScale
            vm.panOffset = CGPoint(x: CGFloat(renderer.panOffset.x), y: CGFloat(renderer.panOffset.y))
            view.draw()
        }
    }
}

// MARK: - Interactive Canvas MTKView Subclass
public final class CanvasMTKView: MTKView {
    public weak var coordinator: MetalCanvasView.Coordinator?
    private var isDragging = false
    private var isSpacePressed = false
    private var lastDragLocation: NSPoint = .zero
    private var trackingArea: NSTrackingArea?
    
    public override var acceptsFirstResponder: Bool { true }
    
    public override func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize)
        if let coordinator = coordinator, let renderer = coordinator.renderer {
            renderer.updateQuad(viewSize: self.drawableSize)
            let z100 = renderer.zoomScale100
            DispatchQueue.main.async {
                coordinator.viewModel?.zoomScale100 = z100
            }
        }
    }
    
    public override func updateTrackingAreas() {
        super.updateTrackingAreas()
        if let trackingArea = trackingArea {
            removeTrackingArea(trackingArea)
        }
        let options: NSTrackingArea.Options = [.activeInKeyWindow, .mouseMoved, .cursorUpdate]
        let area = NSTrackingArea(rect: bounds, options: options, owner: self, userInfo: nil)
        addTrackingArea(area)
        self.trackingArea = area
    }
    
    public override func cursorUpdate(with event: NSEvent) {
        updateCursor()
    }
    
    public override func mouseMoved(with event: NSEvent) {
        updateCursor()
    }
    
    public override func keyDown(with event: NSEvent) {
        if event.keyCode == 49 { // Spacebar
            isSpacePressed = true
            updateCursor()
        } else {
            super.keyDown(with: event)
        }
    }
    
    public override func keyUp(with event: NSEvent) {
        if event.keyCode == 49 { // Spacebar
            isSpacePressed = false
            updateCursor()
        } else {
            super.keyUp(with: event)
        }
    }
    
    private func updateCursor() {
        guard let coordinator = coordinator, let renderer = coordinator.renderer else {
            NSCursor.arrow.set()
            return
        }
        if isDragging {
            NSCursor.closedHand.set()
        } else if renderer.zoomScale > 1.05 || isSpacePressed {
            NSCursor.openHand.set()
        } else {
            NSCursor.arrow.set()
        }
    }
    
    public override func mouseDown(with event: NSEvent) {
        guard let coordinator = coordinator, let renderer = coordinator.renderer else {
            super.mouseDown(with: event)
            return
        }
        
        if event.clickCount == 2 {
            let loc = convert(event.locationInWindow, from: nil)
            let ndcX = Float((loc.x / max(bounds.width, 1.0)) * 2.0 - 1.0)
            let ndcY = Float((loc.y / max(bounds.height, 1.0)) * 2.0 - 1.0)
            coordinator.handleDoubleTap(at: CGPoint(x: CGFloat(ndcX), y: CGFloat(ndcY)))
            updateCursor()
            return
        }
        
        let canPan = renderer.zoomScale > 1.05 || isSpacePressed
        if canPan {
            isDragging = true
            lastDragLocation = convert(event.locationInWindow, from: nil)
            NSCursor.closedHand.set()
        } else {
            super.mouseDown(with: event)
        }
    }
    
    public override func mouseDragged(with event: NSEvent) {
        guard isDragging, let coordinator = coordinator else {
            super.mouseDragged(with: event)
            return
        }
        
        let currentLoc = convert(event.locationInWindow, from: nil)
        let deltaX = currentLoc.x - lastDragLocation.x
        let deltaY = currentLoc.y - lastDragLocation.y
        lastDragLocation = currentLoc
        
        let ndcDeltaX = Float(deltaX / max(bounds.width, 1.0) * 2.0)
        let ndcDeltaY = Float(deltaY / max(bounds.height, 1.0) * 2.0)
        
        coordinator.handlePanDelta(dx: ndcDeltaX, dy: ndcDeltaY)
    }
    
    public override func mouseUp(with event: NSEvent) {
        if isDragging {
            isDragging = false
            updateCursor()
        } else {
            super.mouseUp(with: event)
        }
    }
    
    public override func scrollWheel(with event: NSEvent) {
        guard let coordinator = coordinator, let renderer = coordinator.renderer else {
            super.scrollWheel(with: event)
            return
        }
        
        if renderer.zoomScale > 1.05 {
            let dx = Float(event.scrollingDeltaX / max(bounds.width, 1.0) * 2.0)
            let dy = Float(-event.scrollingDeltaY / max(bounds.height, 1.0) * 2.0)
            coordinator.handlePanDelta(dx: dx, dy: dy)
        } else {
            super.scrollWheel(with: event)
        }
    }
    
    public override func magnify(with event: NSEvent) {
        guard let coordinator = coordinator else {
            super.magnify(with: event)
            return
        }
        let factor = Float(1.0 + event.magnification)
        coordinator.handlePinch(scaleMultiplier: factor)
        updateCursor()
    }
}
