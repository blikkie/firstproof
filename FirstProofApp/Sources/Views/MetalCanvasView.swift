import SwiftUI
import AppKit
import MetalKit
import FirstProofBridge

public struct MetalCanvasView: NSViewRepresentable {
    @Environment(AppViewModel.self) private var envViewModel
    private var explicitViewModel: AppViewModel? = nil
    
    var viewModel: AppViewModel {
        explicitViewModel ?? envViewModel
    }
    let edits: EditSettings
    let zoomScale: Float
    let panOffset: CGPoint
    let isShowingOriginal: Bool
    let isCropToolActive: Bool
    
    public init(
        viewModel: AppViewModel? = nil,
        edits: EditSettings,
        zoomScale: Float,
        panOffset: CGPoint,
        isShowingOriginal: Bool,
        isCropToolActive: Bool = false
    ) {
        self.explicitViewModel = viewModel
        self.edits = edits
        self.zoomScale = zoomScale
        self.panOffset = panOffset
        self.isShowingOriginal = isShowingOriginal
        self.isCropToolActive = isCropToolActive
    }
    
    public func makeCoordinator() -> Coordinator {
        Coordinator()
    }
    
    public func makeNSView(context: Context) -> CanvasMTKView {
        guard let device = MTLCreateSystemDefaultDevice() else {
            return CanvasMTKView()
        }
        
        let mtkView = CanvasMTKView(frame: .zero, device: device)
        mtkView.colorPixelFormat = .rgba16Float
        let neutral = NSColor.underPageBackgroundColor.usingColorSpace(.sRGB) ?? NSColor(srgbRed: 0.14, green: 0.14, blue: 0.14, alpha: 1.0)
        mtkView.clearColor = MTLClearColor(red: Double(neutral.redComponent), green: Double(neutral.greenComponent), blue: Double(neutral.blueComponent), alpha: 1.0)
        mtkView.enableSetNeedsDisplay = true
        mtkView.isPaused = true // Event-driven rendering: only redraws when sliders, gestures, or image change!
        mtkView.coordinator = context.coordinator
        context.coordinator.nsView = mtkView
        context.coordinator.viewModel = viewModel
        
        if let metalLayer = mtkView.layer as? CAMetalLayer {
            metalLayer.wantsExtendedDynamicRangeContent = false
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
            
            if let lutName = edits.lutName {
                let lutRecord = viewModel.resolveLutRecord(for: lutName)
                renderer.updateLutTexture(with: lutRecord)
                context.coordinator.lastLutName = lutName
            }
            
            if let gainMap = viewModel.currentGainMapRecord {
                renderer.updateGainMap(record: gainMap)
                context.coordinator.lastGainMapRecord = gainMap
            }
            
            MetalCanvasView.setupCaptureClosure(for: viewModel, renderer: renderer)
        }
        
        return mtkView
    }
    
    public func updateNSView(_ nsView: CanvasMTKView, context: Context) {
        // Synchronize display capabilities with current window/screen
        viewModel.updateScreenCapabilities(for: nsView.window)
        
        let wantsEdr = viewModel.isScreenHdrCapable && viewModel.isHdrEnabled && viewModel.edrHeadroom > 1.001
        if let metalLayer = nsView.layer as? CAMetalLayer {
            if metalLayer.wantsExtendedDynamicRangeContent != wantsEdr {
                metalLayer.wantsExtendedDynamicRangeContent = wantsEdr
                metalLayer.colorspace = wantsEdr
                    ? CGColorSpace(name: CGColorSpace.extendedLinearDisplayP3)
                    : CGColorSpace(name: CGColorSpace.sRGB)
            }
        }
        
        guard let renderer = context.coordinator.renderer else { return }
        context.coordinator.viewModel = viewModel
        MetalCanvasView.setupCaptureClosure(for: viewModel, renderer: renderer)
        
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
        
        let neutral = NSColor.underPageBackgroundColor.usingColorSpace(.sRGB) ?? NSColor(srgbRed: 0.14, green: 0.14, blue: 0.14, alpha: 1.0)
        nsView.clearColor = MTLClearColor(red: Double(neutral.redComponent), green: Double(neutral.greenComponent), blue: Double(neutral.blueComponent), alpha: 1.0)
        
        // 2. Transmit Slider & Lensfun Uniforms to GPU
        let dims = viewModel.currentImageDimensions
        let imageW = Float(dims.width)
        let imageH = Float(dims.height)
        let imageAspect = imageW / max(imageH, 1.0)
        
        let isLinearRaw = viewModel.rawPreviewRecord != nil
        let editsForUniforms = viewModel.isIntermediatePreviewActive ? EditSettings.defaultIdentity : edits
        
        renderer.updateUniforms(
            from: editsForUniforms,
            imageAspect: imageAspect,
            isLinearRaw: isLinearRaw,
            isOriginal: isShowingOriginal,
            isCropToolActive: isCropToolActive,
            lensCorrectionEnabled: viewModel.isIntermediatePreviewActive ? false : viewModel.lensCorrectionEnabled,
            lensCoefficients: viewModel.isIntermediatePreviewActive ? nil : viewModel.lensCoefficients,
            edrHeadroom: viewModel.isScreenHdrCapable ? viewModel.edrHeadroom : 1.0,
            edrEnabled: viewModel.isScreenHdrCapable && viewModel.isHdrEnabled,
            isIntermediate: viewModel.isIntermediatePreviewActive
        )
        
        // 3. Update 3D LUT texture if LUT selection changed
        let lutChanged = context.coordinator.lastLutName != edits.lutName
        if lutChanged {
            context.coordinator.lastLutName = edits.lutName
            if let lutName = edits.lutName {
                let lutRecord = viewModel.resolveLutRecord(for: lutName)
                renderer.updateLutTexture(with: lutRecord)
            } else {
                renderer.updateLutTexture(with: nil)
            }
        }
        
        // 4. Update Ultra HDR Gain Map if changed
        let gainMapChanged = context.coordinator.lastGainMapRecord != viewModel.currentGainMapRecord
        if gainMapChanged {
            renderer.updateGainMap(record: viewModel.currentGainMapRecord)
            context.coordinator.lastGainMapRecord = viewModel.currentGainMapRecord
        }
        
        // 5. Viewport Zoom & Pan
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
        
        // Real-time intermediate capture for subsequent burst scrubbing
        if rawChanged, let path = viewModel.selectedPhoto?.filePath, viewModel.rawPreviewRecord != nil {
            let currentEdits = viewModel.currentEdits
            if let intermediate = renderer.renderThumbnailSnapshot(maxDimension: 2560) {
                ThumbnailCache.shared.saveIntermediate(intermediate, for: path, edits: currentEdits)
            }
        }
    }
    
    private static func setupCaptureClosure(for viewModel: AppViewModel, renderer: MetalRenderer) {
        viewModel.captureThumbnailSnapshot = { [weak renderer, weak viewModel] in
            guard let renderer = renderer, let vm = viewModel else { return nil }
            let imageW: Float
            let imageH: Float
            if let raw = vm.rawPreviewRecord {
                imageW = Float(raw.width)
                imageH = Float(raw.height)
            } else if let img = vm.previewImage {
                imageW = Float(img.size.width)
                imageH = Float(img.size.height)
            } else {
                imageW = 4000.0
                imageH = 3000.0
            }
            let imageAspect = imageW / max(imageH, 1.0)
            
            renderer.updateUniforms(
                from: vm.currentEdits,
                imageAspect: imageAspect,
                isLinearRaw: vm.rawPreviewRecord != nil,
                isOriginal: false,
                isCropToolActive: false,
                lensCorrectionEnabled: vm.lensCorrectionEnabled,
                lensCoefficients: vm.lensCoefficients
            )
            return renderer.renderThumbnailSnapshot(maxDimension: 320)
        }
        
        viewModel.captureIntermediateSnapshot = { [weak renderer, weak viewModel] in
            guard let renderer = renderer, let vm = viewModel else { return nil }
            guard vm.rawPreviewRecord != nil else { return nil }
            let imageW: Float
            let imageH: Float
            if let raw = vm.rawPreviewRecord {
                imageW = Float(raw.width)
                imageH = Float(raw.height)
            } else if let img = vm.previewImage {
                imageW = Float(img.size.width)
                imageH = Float(img.size.height)
            } else {
                imageW = 4000.0
                imageH = 3000.0
            }
            let imageAspect = imageW / max(imageH, 1.0)
            
            renderer.updateUniforms(
                from: vm.currentEdits,
                imageAspect: imageAspect,
                isLinearRaw: true,
                isOriginal: false,
                isCropToolActive: false,
                lensCorrectionEnabled: vm.lensCorrectionEnabled,
                lensCoefficients: vm.lensCoefficients
            )
            return renderer.renderThumbnailSnapshot(maxDimension: 2560)
        }
    }
    
    @MainActor
    public class Coordinator {
        var renderer: MetalRenderer?
        weak var nsView: CanvasMTKView?
        var viewModel: AppViewModel?
        var lastPhotoPath: String?
        var lastPreviewImage: NSImage?
        var lastRawKey: String?
        var lastLutName: String?
        var lastGainMapRecord: GainMapRecord?
        
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
    
    public override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        if let metalLayer = self.layer as? CAMetalLayer {
            metalLayer.colorspace = CGColorSpace(name: CGColorSpace.sRGB)
        }
    }
    
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
        if event.keyCode == 53 { // Escape: cancel eyedropper
            if coordinator?.viewModel?.isEyedropperActive == true {
                coordinator?.viewModel?.isEyedropperActive = false
                updateCursor()
                return
            }
        }
        if event.keyCode == 49 { // Spacebar
            isSpacePressed = true
            updateCursor()
        } else if event.keyCode == 42 || event.characters == "\\" { // Backslash: Hold for Original
            coordinator?.viewModel?.isShowingOriginal = true
        } else {
            super.keyDown(with: event)
        }
    }
    
    public override func keyUp(with event: NSEvent) {
        if event.keyCode == 49 { // Spacebar
            isSpacePressed = false
            updateCursor()
        } else if event.keyCode == 42 || event.characters == "\\" { // Backslash: Release Original
            coordinator?.viewModel?.isShowingOriginal = false
        } else {
            super.keyUp(with: event)
        }
    }
    
    private func updateCursor() {
        guard let coordinator = coordinator, let renderer = coordinator.renderer else {
            NSCursor.arrow.set()
            return
        }
        if coordinator.viewModel?.isEyedropperActive == true {
            NSCursor.crosshair.set()
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
        
        if let vm = coordinator.viewModel, vm.isEyedropperActive {
            let loc = convert(event.locationInWindow, from: nil)
            let ndcX = Float((loc.x / max(bounds.width, 1.0)) * 2.0 - 1.0)
            let ndcY = Float((loc.y / max(bounds.height, 1.0)) * 2.0 - 1.0)
            if let uv = renderer.normalizedTextureCoordinates(atNDC: CGPoint(x: CGFloat(ndcX), y: CGFloat(ndcY))) {
                vm.applyEyedropperSample(atNormalizedUV: uv)
                vm.isEyedropperActive = false
                updateCursor()
                return
            }
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
