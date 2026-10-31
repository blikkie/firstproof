import SwiftUI
import AppKit
import FirstProofBridge

public struct CropOverlayView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    // Interactive drag state tracking
    @State private var activeHandle: DragHandle? = nil
    @State private var dragStartCrop: (left: Float, top: Float, right: Float, bottom: Float) = (0, 0, 1, 1)
    @State private var dragStartStraighten: Float = 0.0
    @State private var isRotating: Bool = false
    
    enum DragHandle: Equatable {
        case topLeft
        case topRight
        case bottomLeft
        case bottomRight
        case top
        case bottom
        case left
        case right
        case inside
        case rotate
    }
    
    public init() {}
    
    public var body: some View {
        @Bindable var viewModel = viewModel
        GeometryReader { geometry in
            let viewSize = geometry.size
            let isOrthogonal = (viewModel.rotationDegrees == 90 || viewModel.rotationDegrees == 270)
            let dims = viewModel.currentImageDimensions
            let imgW = CGFloat(isOrthogonal ? dims.height : dims.width)
            let imgH = CGFloat(isOrthogonal ? dims.width : dims.height)
            let imageAspect = imgW / max(imgH, 1.0)
            let viewAspect = viewSize.width / max(viewSize.height, 1.0)
            
            let (renderW, renderH): (CGFloat, CGFloat) = {
                if viewAspect > imageAspect {
                    let h = viewSize.height
                    return (h * imageAspect, h)
                } else {
                    let w = viewSize.width
                    return (w, w / max(imageAspect, 0.001))
                }
            }()
            
            let imageRect = CGRect(
                x: (viewSize.width - renderW) / 2.0,
                y: (viewSize.height - renderH) / 2.0,
                width: renderW,
                height: renderH
            )
            
            let cropX = imageRect.minX + CGFloat(viewModel.cropLeft) * imageRect.width
            let cropY = imageRect.minY + CGFloat(viewModel.cropTop) * imageRect.height
            let cropW = max(CGFloat(viewModel.cropRight - viewModel.cropLeft) * imageRect.width, 20.0)
            let cropH = max(CGFloat(viewModel.cropBottom - viewModel.cropTop) * imageRect.height, 20.0)
            let cropRect = CGRect(x: cropX, y: cropY, width: cropW, height: cropH)
            
            ZStack {
                // 1. Dimmed Scrim outside active crop box (confined strictly to the image)
                scrimView(imageRect: imageRect, cropRect: cropRect)
                
                // 2. Crop Frame Border & Rule-of-Thirds Grid
                cropFrameAndGrid(cropRect: cropRect)
                
                // 3. Visual Handles (Pure display, no competing gestures)
                visualHandlesView(cropRect: cropRect)
                
                // 4. Single Unified Gesture Surface covering entire view
                Color.clear
                    .contentShape(Rectangle())
                    .gesture(
                        DragGesture(minimumDistance: 1)
                            .onChanged { value in
                                handleDragChanged(value: value, imageRect: imageRect, cropRect: cropRect, imageAspect: imageAspect)
                            }
                            .onEnded { _ in
                                handleDragEnded()
                            }
                    )
                
                // 5. Live HUD Readout when rotating
                if isRotating {
                    VStack {
                        Spacer()
                        HStack(spacing: 6) {
                            Image(systemName: "rotate.right")
                            Text(String(format: "Straighten: %+.1f°", viewModel.straightenAngle))
                                .fontWeight(.semibold)
                        }
                        .font(.system(size: 13, weight: .medium, design: .monospaced))
                        .foregroundColor(.white)
                        .padding(.horizontal, 14)
                        .padding(.vertical, 8)
                        .background(Color(red: 0.11, green: 0.11, blue: 0.12).opacity(0.92))
                        .cornerRadius(20)
                        .padding(.bottom, 80)
                    }
                }
                
                // 6. Floating Bottom Action HUD Bar
                VStack {
                    Spacer()
                    floatingActionBar(imageAspect: imageAspect)
                        .padding(.bottom, 24)
                }
            }
            .onDisappear {
                guard viewModel.isCropToolActive else {
                    activeHandle = nil
                    isRotating = false
                    return
                }
                if activeHandle != nil {
                    handleDragEnded()
                } else if viewModel.undoHistory.scrubStartEdits != nil {
                    viewModel.endSliderScrub(actionName: "Crop Box")
                }
            }
        }
    }
    
    // MARK: - Scrim View
    @ViewBuilder
    private func scrimView(imageRect: CGRect, cropRect: CGRect) -> some View {
        Canvas { context, _ in
            context.fill(Path(imageRect), with: .color(Color.black.opacity(0.55)))
            context.blendMode = .clear
            context.fill(Path(cropRect), with: .color(.white))
        }
        .allowsHitTesting(false)
    }
    
    // MARK: - Crop Frame & Rule-of-Thirds Grid
    private func cropFrameAndGrid(cropRect: CGRect) -> some View {
        ZStack {
            // White outline
            Rectangle()
                .stroke(Color.white, lineWidth: 1.5)
                .frame(width: cropRect.width, height: cropRect.height)
                .position(x: cropRect.midX, y: cropRect.midY)
                .shadow(color: Color.black.opacity(0.5), radius: 1.5, x: 0, y: 1)
            
            // Rule-of-Thirds Guide Lines
            Path { path in
                let thirdW = cropRect.width / 3.0
                let thirdH = cropRect.height / 3.0
                
                // Vertical lines
                path.move(to: CGPoint(x: cropRect.minX + thirdW, y: cropRect.minY))
                path.addLine(to: CGPoint(x: cropRect.minX + thirdW, y: cropRect.maxY))
                path.move(to: CGPoint(x: cropRect.minX + 2.0 * thirdW, y: cropRect.minY))
                path.addLine(to: CGPoint(x: cropRect.minX + 2.0 * thirdW, y: cropRect.maxY))
                
                // Horizontal lines
                path.move(to: CGPoint(x: cropRect.minX, y: cropRect.minY + thirdH))
                path.addLine(to: CGPoint(x: cropRect.maxX, y: cropRect.minY + thirdH))
                path.move(to: CGPoint(x: cropRect.minX, y: cropRect.minY + 2.0 * thirdH))
                path.addLine(to: CGPoint(x: cropRect.maxX, y: cropRect.minY + 2.0 * thirdH))
            }
            .stroke(Color.white.opacity(0.65), style: StrokeStyle(lineWidth: 0.75, dash: [4, 4]))
            .shadow(color: Color.black.opacity(0.4), radius: 1, x: 0, y: 0.5)
        }
        .allowsHitTesting(false)
    }
    
    // MARK: - Visual Handles View
    private func visualHandlesView(cropRect: CGRect) -> some View {
        ZStack {
            // 4 Edge Handles
            edgeHandleView(point: CGPoint(x: cropRect.midX, y: cropRect.minY), isHorizontal: true)
            edgeHandleView(point: CGPoint(x: cropRect.midX, y: cropRect.maxY), isHorizontal: true)
            edgeHandleView(point: CGPoint(x: cropRect.minX, y: cropRect.midY), isHorizontal: false)
            edgeHandleView(point: CGPoint(x: cropRect.maxX, y: cropRect.midY), isHorizontal: false)
            
            // 4 Corner Brackets
            cornerHandleView(type: .topLeft, point: CGPoint(x: cropRect.minX, y: cropRect.minY))
            cornerHandleView(type: .topRight, point: CGPoint(x: cropRect.maxX, y: cropRect.minY))
            cornerHandleView(type: .bottomLeft, point: CGPoint(x: cropRect.minX, y: cropRect.maxY))
            cornerHandleView(type: .bottomRight, point: CGPoint(x: cropRect.maxX, y: cropRect.maxY))
        }
        .allowsHitTesting(false)
    }
    
    private func edgeHandleView(point: CGPoint, isHorizontal: Bool) -> some View {
        Capsule()
            .fill(Color.white)
            .frame(width: isHorizontal ? 26 : 5, height: isHorizontal ? 5 : 26)
            .shadow(color: Color.black.opacity(0.6), radius: 1, x: 0, y: 0.5)
            .position(point)
    }
    
    private func cornerHandleView(type: DragHandle, point: CGPoint) -> some View {
        CornerBracketShape(handle: type)
            .stroke(Color.white, lineWidth: 3.5)
            .frame(width: 18, height: 18)
            .shadow(color: Color.black.opacity(0.7), radius: 1, x: 0, y: 0.5)
            .position(point)
    }
    
    // MARK: - Unified Drag Handling
    private func handleDragChanged(value: DragGesture.Value, imageRect: CGRect, cropRect: CGRect, imageAspect: CGFloat) {
        if activeHandle == nil {
            let start = value.startLocation
            let cornerRadius: CGFloat = 28.0
            
            // 1. Check corner handles first
            if hypot(start.x - cropRect.minX, start.y - cropRect.minY) <= cornerRadius {
                activeHandle = .topLeft
            } else if hypot(start.x - cropRect.maxX, start.y - cropRect.minY) <= cornerRadius {
                activeHandle = .topRight
            } else if hypot(start.x - cropRect.minX, start.y - cropRect.maxY) <= cornerRadius {
                activeHandle = .bottomLeft
            } else if hypot(start.x - cropRect.maxX, start.y - cropRect.maxY) <= cornerRadius {
                activeHandle = .bottomRight
            }
            // 2. Check edge handles
            else if abs(start.y - cropRect.minY) <= 18.0 && start.x >= cropRect.minX && start.x <= cropRect.maxX {
                activeHandle = .top
            } else if abs(start.y - cropRect.maxY) <= 18.0 && start.x >= cropRect.minX && start.x <= cropRect.maxX {
                activeHandle = .bottom
            } else if abs(start.x - cropRect.minX) <= 18.0 && start.y >= cropRect.minY && start.y <= cropRect.maxY {
                activeHandle = .left
            } else if abs(start.x - cropRect.maxX) <= 18.0 && start.y >= cropRect.minY && start.y <= cropRect.maxY {
                activeHandle = .right
            }
            // 3. Check inside crop rect (translate)
            else if cropRect.contains(start) {
                activeHandle = .inside
            }
            // 4. Outside crop rect (rotate / straighten)
            else {
                activeHandle = .rotate
                isRotating = true
            }
            
            viewModel.beginSliderScrub()
            dragStartCrop = (viewModel.cropLeft, viewModel.cropTop, viewModel.cropRight, viewModel.cropBottom)
            dragStartStraighten = viewModel.straightenAngle
        }
        
        guard let handle = activeHandle else { return }
        
        switch handle {
        case .rotate:
            let center = CGPoint(x: imageRect.midX, y: imageRect.midY)
            let startVector = CGPoint(x: value.startLocation.x - center.x, y: value.startLocation.y - center.y)
            let currentVector = CGPoint(x: value.location.x - center.x, y: value.location.y - center.y)
            let startAngle = atan2(startVector.y, startVector.x)
            let currentAngle = atan2(currentVector.y, currentVector.x)
            var deltaDegrees = Float((currentAngle - startAngle) * 180.0 / .pi)
            if deltaDegrees > 180.0 { deltaDegrees -= 360.0 }
            if deltaDegrees < -180.0 { deltaDegrees += 360.0 }
            
            let newStraighten = clamp(dragStartStraighten + deltaDegrees, min: -45.0, max: 45.0)
            viewModel.straightenAngle = newStraighten
            
        case .inside:
            let deltaU = Float(value.translation.width / imageRect.width)
            let deltaV = Float(value.translation.height / imageRect.height)
            let width = dragStartCrop.right - dragStartCrop.left
            let height = dragStartCrop.bottom - dragStartCrop.top
            
            var newLeft = dragStartCrop.left + deltaU
            var newTop = dragStartCrop.top + deltaV
            
            newLeft = max(0.0, min(newLeft, 1.0 - width))
            newTop = max(0.0, min(newTop, 1.0 - height))
            
            viewModel.cropLeft = newLeft
            viewModel.cropRight = newLeft + width
            viewModel.cropTop = newTop
            viewModel.cropBottom = newTop + height
            
        case .top, .bottom, .left, .right:
            updateEdgeDrag(type: handle, translation: value.translation, imageRect: imageRect, imageAspect: imageAspect)
            
        case .topLeft, .topRight, .bottomLeft, .bottomRight:
            updateCornerDrag(type: handle, translation: value.translation, imageRect: imageRect, imageAspect: imageAspect)
        }
    }
    
    private func handleDragEnded() {
        guard let handle = activeHandle else { return }
        let actionTitle: String
        switch handle {
        case .rotate:
            actionTitle = "Straighten (\(String(format: "%.1f°", viewModel.straightenAngle)))"
        case .inside:
            actionTitle = "Move Crop"
        case .topLeft, .topRight, .bottomLeft, .bottomRight, .top, .bottom, .left, .right:
            actionTitle = "Crop Box"
        }
        activeHandle = nil
        isRotating = false
        viewModel.endSliderScrub(actionName: actionTitle)
    }
    
    // MARK: - Aspect Ratio Resolver
    private func getTargetRatio(imageAspect: CGFloat) -> CGFloat? {
        guard viewModel.cropAspectRatioPreset != .freeform else { return nil }
        
        let currentW = CGFloat(dragStartCrop.right - dragStartCrop.left)
        let currentH = CGFloat(dragStartCrop.bottom - dragStartCrop.top)
        let isCurrentlyPortrait = (currentW * imageAspect) < currentH
        let isPortrait = (dragStartCrop.left < 0.01 && dragStartCrop.top < 0.01 && dragStartCrop.right > 0.99 && dragStartCrop.bottom > 0.99) ? (imageAspect < 1.0) : isCurrentlyPortrait
        
        if let ratio = calculateTargetCropRatio(
            preset: viewModel.cropAspectRatioPreset,
            imageAspect: Float(imageAspect),
            isPortrait: isPortrait
        ) {
            return CGFloat(ratio)
        }
        return nil
    }
    
    // MARK: - Edge Handle Updates
    private func updateEdgeDrag(type: DragHandle, translation: CGSize, imageRect: CGRect, imageAspect: CGFloat) {
        let deltaU = Float(translation.width / imageRect.width)
        let deltaV = Float(translation.height / imageRect.height)
        let W0 = dragStartCrop.right - dragStartCrop.left
        let H0 = dragStartCrop.bottom - dragStartCrop.top
        
        if let ratio = getTargetRatio(imageAspect: imageAspect) {
            let normRatio = Float(ratio / imageAspect)
            let Yc = (dragStartCrop.top + dragStartCrop.bottom) / 2.0
            let Xc = (dragStartCrop.left + dragStartCrop.right) / 2.0
            
            switch type {
            case .right:
                let maxAllowedW = max(0.01, min(1.0 - dragStartCrop.left, 2.0 * min(Yc, 1.0 - Yc) * normRatio))
                let minW = min(Float(0.05), maxAllowedW)
                let newW = clamp(W0 + deltaU, min: minW, max: maxAllowedW)
                let newH = newW / normRatio
                viewModel.cropLeft = dragStartCrop.left
                viewModel.cropRight = clamp(dragStartCrop.left + newW, min: 0.0, max: 1.0)
                viewModel.cropTop = clamp(Yc - newH / 2.0, min: 0.0, max: 1.0)
                viewModel.cropBottom = clamp(Yc + newH / 2.0, min: 0.0, max: 1.0)
                
            case .left:
                let maxAllowedW = max(0.01, min(dragStartCrop.right, 2.0 * min(Yc, 1.0 - Yc) * normRatio))
                let minW = min(Float(0.05), maxAllowedW)
                let newW = clamp(W0 - deltaU, min: minW, max: maxAllowedW)
                let newH = newW / normRatio
                viewModel.cropLeft = clamp(dragStartCrop.right - newW, min: 0.0, max: 1.0)
                viewModel.cropRight = dragStartCrop.right
                viewModel.cropTop = clamp(Yc - newH / 2.0, min: 0.0, max: 1.0)
                viewModel.cropBottom = clamp(Yc + newH / 2.0, min: 0.0, max: 1.0)
                
            case .bottom:
                let maxAllowedH = max(0.01, min(1.0 - dragStartCrop.top, 2.0 * min(Xc, 1.0 - Xc) / normRatio))
                let minH = min(Float(0.05), maxAllowedH)
                let newH = clamp(H0 + deltaV, min: minH, max: maxAllowedH)
                let newW = newH * normRatio
                viewModel.cropTop = dragStartCrop.top
                viewModel.cropBottom = clamp(dragStartCrop.top + newH, min: 0.0, max: 1.0)
                viewModel.cropLeft = clamp(Xc - newW / 2.0, min: 0.0, max: 1.0)
                viewModel.cropRight = clamp(Xc + newW / 2.0, min: 0.0, max: 1.0)
                
            case .top:
                let maxAllowedH = max(0.01, min(dragStartCrop.bottom, 2.0 * min(Xc, 1.0 - Xc) / normRatio))
                let minH = min(Float(0.05), maxAllowedH)
                let newH = clamp(H0 - deltaV, min: minH, max: maxAllowedH)
                let newW = newH * normRatio
                viewModel.cropTop = clamp(dragStartCrop.bottom - newH, min: 0.0, max: 1.0)
                viewModel.cropBottom = dragStartCrop.bottom
                viewModel.cropLeft = clamp(Xc - newW / 2.0, min: 0.0, max: 1.0)
                viewModel.cropRight = clamp(Xc + newW / 2.0, min: 0.0, max: 1.0)
                
            default:
                break
            }
        } else {
            // Freeform resizing
            switch type {
            case .top:
                viewModel.cropTop = clamp(dragStartCrop.top + deltaV, min: 0.0, max: dragStartCrop.bottom - 0.05)
            case .bottom:
                viewModel.cropBottom = clamp(dragStartCrop.bottom + deltaV, min: dragStartCrop.top + 0.05, max: 1.0)
            case .left:
                viewModel.cropLeft = clamp(dragStartCrop.left + deltaU, min: 0.0, max: dragStartCrop.right - 0.05)
            case .right:
                viewModel.cropRight = clamp(dragStartCrop.right + deltaU, min: dragStartCrop.left + 0.05, max: 1.0)
            default:
                break
            }
        }
    }
    
    // MARK: - Corner Bracket Updates
    private func updateCornerDrag(type: DragHandle, translation: CGSize, imageRect: CGRect, imageAspect: CGFloat) {
        let deltaU = Float(translation.width / imageRect.width)
        let deltaV = Float(translation.height / imageRect.height)
        let W0 = dragStartCrop.right - dragStartCrop.left
        
        if let ratio = getTargetRatio(imageAspect: imageAspect) {
            let normRatio = Float(ratio / imageAspect)
            
            switch type {
            case .bottomRight:
                let delta = (deltaU + deltaV * normRatio) / 2.0
                let maxAllowedW = max(0.01, min(1.0 - dragStartCrop.left, (1.0 - dragStartCrop.top) * normRatio))
                let minW = min(Float(0.05), maxAllowedW)
                let newW = clamp(W0 + delta, min: minW, max: maxAllowedW)
                let newH = newW / normRatio
                viewModel.cropLeft = dragStartCrop.left
                viewModel.cropRight = clamp(dragStartCrop.left + newW, min: 0.0, max: 1.0)
                viewModel.cropTop = dragStartCrop.top
                viewModel.cropBottom = clamp(dragStartCrop.top + newH, min: 0.0, max: 1.0)
                
            case .topRight:
                let delta = (deltaU - deltaV * normRatio) / 2.0
                let maxAllowedW = max(0.01, min(1.0 - dragStartCrop.left, dragStartCrop.bottom * normRatio))
                let minW = min(Float(0.05), maxAllowedW)
                let newW = clamp(W0 + delta, min: minW, max: maxAllowedW)
                let newH = newW / normRatio
                viewModel.cropLeft = dragStartCrop.left
                viewModel.cropRight = clamp(dragStartCrop.left + newW, min: 0.0, max: 1.0)
                viewModel.cropTop = clamp(dragStartCrop.bottom - newH, min: 0.0, max: 1.0)
                viewModel.cropBottom = dragStartCrop.bottom
                
            case .bottomLeft:
                let delta = (-deltaU + deltaV * normRatio) / 2.0
                let maxAllowedW = max(0.01, min(dragStartCrop.right, (1.0 - dragStartCrop.top) * normRatio))
                let minW = min(Float(0.05), maxAllowedW)
                let newW = clamp(W0 + delta, min: minW, max: maxAllowedW)
                let newH = newW / normRatio
                viewModel.cropLeft = clamp(dragStartCrop.right - newW, min: 0.0, max: 1.0)
                viewModel.cropRight = dragStartCrop.right
                viewModel.cropTop = dragStartCrop.top
                viewModel.cropBottom = clamp(dragStartCrop.top + newH, min: 0.0, max: 1.0)
                
            case .topLeft:
                let delta = (-deltaU - deltaV * normRatio) / 2.0
                let maxAllowedW = max(0.01, min(dragStartCrop.right, dragStartCrop.bottom * normRatio))
                let minW = min(Float(0.05), maxAllowedW)
                let newW = clamp(W0 + delta, min: minW, max: maxAllowedW)
                let newH = newW / normRatio
                viewModel.cropLeft = clamp(dragStartCrop.right - newW, min: 0.0, max: 1.0)
                viewModel.cropRight = dragStartCrop.right
                viewModel.cropTop = clamp(dragStartCrop.bottom - newH, min: 0.0, max: 1.0)
                viewModel.cropBottom = dragStartCrop.bottom
                
            default:
                break
            }
        } else {
            // Freeform resizing
            switch type {
            case .topLeft:
                viewModel.cropLeft = clamp(dragStartCrop.left + deltaU, min: 0.0, max: dragStartCrop.right - 0.05)
                viewModel.cropTop = clamp(dragStartCrop.top + deltaV, min: 0.0, max: dragStartCrop.bottom - 0.05)
            case .topRight:
                viewModel.cropRight = clamp(dragStartCrop.right + deltaU, min: dragStartCrop.left + 0.05, max: 1.0)
                viewModel.cropTop = clamp(dragStartCrop.top + deltaV, min: 0.0, max: dragStartCrop.bottom - 0.05)
            case .bottomLeft:
                viewModel.cropLeft = clamp(dragStartCrop.left + deltaU, min: 0.0, max: dragStartCrop.right - 0.05)
                viewModel.cropBottom = clamp(dragStartCrop.bottom + deltaV, min: dragStartCrop.top + 0.05, max: 1.0)
            case .bottomRight:
                viewModel.cropRight = clamp(dragStartCrop.right + deltaU, min: dragStartCrop.left + 0.05, max: 1.0)
                viewModel.cropBottom = clamp(dragStartCrop.bottom + deltaV, min: dragStartCrop.top + 0.05, max: 1.0)
            default:
                break
            }
        }
    }
    
    // MARK: - Floating Action Bar HUD
    private func floatingActionBar(imageAspect: CGFloat) -> some View {
        @Bindable var viewModel = viewModel
        return HStack(spacing: 12) {
            // Aspect Ratio Picker
            Picker("Aspect", selection: Binding<CropAspectRatioPreset>(
                get: { viewModel.cropAspectRatioPreset },
                set: { viewModel.setCropAspectRatioPreset($0) }
            )) {
                ForEach(CropAspectRatioPreset.allCases) { preset in
                    Text(preset.rawValue).tag(preset)
                }
            }
            .labelsHidden()
            .pickerStyle(.segmented)
            .controlSize(.small)
            .frame(width: 330)
            
            Divider()
                .frame(height: 18)
            
            // Flip Orientation (X)
            Button(action: {
                viewModel.flipCropOrientation()
            }) {
                Label("Flip (X)", systemImage: "arrow.triangle.2.circlepath")
                    .font(.caption)
            }
            .buttonStyle(.plain)
            .help("Flip Crop Orientation between Landscape and Portrait (X)")
            
            Divider()
                .frame(height: 18)
            
            // Reset Crop
            Button("Reset") {
                viewModel.resetCrop()
            }
            .buttonStyle(.plain)
            .font(.caption)
            .foregroundColor(.secondary)
            .help("Reset crop to full frame")
            
            Divider()
                .frame(height: 18)
            
            // Cancel Button
            Button(action: {
                activeHandle = nil
                isRotating = false
                viewModel.cancelCrop()
            }) {
                HStack(spacing: 4) {
                    Text("Cancel")
                }
                .font(.caption)
                .foregroundColor(.secondary)
            }
            .buttonStyle(.plain)
            .help("Cancel crop modifications and revert to previous framing (Esc)")
            
            // Done Button
            Button(action: {
                activeHandle = nil
                isRotating = false
                viewModel.commitCrop()
            }) {
                HStack(spacing: 4) {
                    Text("Done")
                    Image(systemName: "return")
                        .font(.system(size: 10, weight: .bold))
                }
                .font(.caption.weight(.semibold))
                .padding(.horizontal, 10)
                .padding(.vertical, 4)
                .background(Color.accentColor)
                .foregroundColor(Color(red: 0.10, green: 0.10, blue: 0.12))
                .cornerRadius(6)
            }
            .buttonStyle(.plain)
            .help("Commit Crop and exit Crop Tool (Return)")
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .background(Color(red: 0.11, green: 0.11, blue: 0.12).opacity(0.92))
        .cornerRadius(12)
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .stroke(Color.white.opacity(0.18), lineWidth: 0.8)
        )
        .shadow(color: Color.black.opacity(0.4), radius: 10, x: 0, y: 5)
    }
    
    private func clamp<T: Comparable>(_ val: T, min: T, max: T) -> T {
        Swift.min(Swift.max(val, min), max)
    }
}

// MARK: - Corner Bracket Shape
private struct CornerBracketShape: Shape {
    let handle: CropOverlayView.DragHandle
    
    func path(in rect: CGRect) -> Path {
        var path = Path()
        let arm = min(rect.width, rect.height)
        
        switch handle {
        case .topLeft:
            path.move(to: CGPoint(x: rect.minX, y: rect.minY + arm))
            path.addLine(to: CGPoint(x: rect.minX, y: rect.minY))
            path.addLine(to: CGPoint(x: rect.minX + arm, y: rect.minY))
        case .topRight:
            path.move(to: CGPoint(x: rect.maxX - arm, y: rect.minY))
            path.addLine(to: CGPoint(x: rect.maxX, y: rect.minY))
            path.addLine(to: CGPoint(x: rect.maxX, y: rect.minY + arm))
        case .bottomLeft:
            path.move(to: CGPoint(x: rect.minX, y: rect.maxY - arm))
            path.addLine(to: CGPoint(x: rect.minX, y: rect.maxY))
            path.addLine(to: CGPoint(x: rect.minX + arm, y: rect.maxY))
        case .bottomRight:
            path.move(to: CGPoint(x: rect.maxX - arm, y: rect.maxY))
            path.addLine(to: CGPoint(x: rect.maxX, y: rect.maxY))
            path.addLine(to: CGPoint(x: rect.maxX, y: rect.maxY - arm))
        default:
            break
        }
        return path
    }
}
