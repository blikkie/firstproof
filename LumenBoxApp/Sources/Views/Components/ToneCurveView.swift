import SwiftUI
import LumenBoxBridge

public enum ToneCurveChannel: String, CaseIterable, Identifiable {
    case master = "RGB"
    case red = "Red"
    case green = "Green"
    case blue = "Blue"
    
    public var id: String { rawValue }
    
    public var color: Color {
        switch self {
        case .master: return Color.white
        case .red: return Color(red: 1.0, green: 0.28, blue: 0.28)
        case .green: return Color(red: 0.25, green: 0.85, blue: 0.45)
        case .blue: return Color(red: 0.25, green: 0.60, blue: 1.0)
        }
    }
}

public struct ToneCurveView: View {
    @Bindable public var viewModel: AppViewModel
    @State private var selectedChannel: ToneCurveChannel = .master
    @State private var draggingPointIndex: Int? = nil
    @State private var hoveredPointIndex: Int? = nil
    @State private var activeCoordinates: (input: Int, output: Int)? = nil
    
    public init(viewModel: AppViewModel) {
        self.viewModel = viewModel
    }
    
    // MARK: - Current Channel Points Helper
    private var currentPoints: [CurvePointRecord] {
        get {
            switch selectedChannel {
            case .master: return viewModel.curves.master
            case .red: return viewModel.curves.red
            case .green: return viewModel.curves.green
            case .blue: return viewModel.curves.blue
            }
        }
    }
    
    private func setPoints(_ points: [CurvePointRecord]) {
        var updated = viewModel.curves
        // Keep points sorted by x
        let sorted = points.sorted { $0.x < $1.x }
        switch selectedChannel {
        case .master: updated.master = sorted
        case .red: updated.red = sorted
        case .green: updated.green = sorted
        case .blue: updated.blue = sorted
        }
        viewModel.curves = updated
    }
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            // Header & Channel Selector
            HStack {
                Text("TONE CURVE")
                    .font(.system(size: 10, weight: .bold))
                    .foregroundColor(.secondary)
                
                Spacer()
                
                if let coords = activeCoordinates {
                    Text("In: \(coords.input)  Out: \(coords.output)")
                        .font(.system(size: 10, weight: .regular, design: .monospaced))
                        .foregroundColor(.secondary)
                        .padding(.trailing, 4)
                }
                
                Button(action: resetAllCurves) {
                    Image(systemName: "arrow.counterclockwise")
                        .font(.system(size: 10))
                }
                .buttonStyle(.plain)
                .foregroundColor(.secondary)
                .help("Reset all tone curves to linear")
            }
            
            // Channel Pills
            HStack(spacing: 6) {
                ForEach(ToneCurveChannel.allCases) { channel in
                    Button(action: {
                        selectedChannel = channel
                    }) {
                        HStack(spacing: 4) {
                            Circle()
                                .fill(channel.color)
                                .frame(width: 7, height: 7)
                            Text(channel.rawValue)
                                .font(.system(size: 11, weight: selectedChannel == channel ? .semibold : .regular))
                        }
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(
                            RoundedRectangle(cornerRadius: 6)
                                .fill(selectedChannel == channel ? Color.white.opacity(0.15) : Color.white.opacity(0.05))
                        )
                        .overlay(
                            RoundedRectangle(cornerRadius: 6)
                                .stroke(selectedChannel == channel ? channel.color.opacity(0.6) : Color.clear, lineWidth: 1)
                        )
                    }
                    .buttonStyle(.plain)
                }
            }
            
            // Interactive 2D Spline Graph
            GeometryReader { geo in
                let size = geo.size.width
                let height = size
                
                ZStack {
                    // 1. Background Canvas
                    RoundedRectangle(cornerRadius: 8)
                        .fill(Color(nsColor: .controlBackgroundColor).opacity(0.6))
                        .overlay(
                            RoundedRectangle(cornerRadius: 8)
                                .stroke(Color.white.opacity(0.1), lineWidth: 1)
                        )
                    
                    // 2. Zone Grid (4x4 zones: Blacks, Shadows, Midtones, Highlights)
                    Path { path in
                        // Grid lines at 25%, 50%, 75%
                        for frac in [0.25, 0.50, 0.75] {
                            let x = size * CGFloat(frac)
                            let y = height * CGFloat(frac)
                            // Vertical
                            path.move(to: CGPoint(x: x, y: 0))
                            path.addLine(to: CGPoint(x: x, y: height))
                            // Horizontal
                            path.move(to: CGPoint(x: 0, y: y))
                            path.addLine(to: CGPoint(x: size, y: y))
                        }
                    }
                    .stroke(Color.white.opacity(0.07), style: StrokeStyle(lineWidth: 1, dash: [3, 3]))
                    
                    // 3. Diagonal Linear Baseline (Identity)
                    Path { path in
                        path.move(to: CGPoint(x: 0, y: height))
                        path.addLine(to: CGPoint(x: size, y: 0))
                    }
                    .stroke(Color.white.opacity(0.15), style: StrokeStyle(lineWidth: 1, dash: [4, 4]))
                    
                    // 4. Ghost Curves (Master curve if in RGB channel mode, or RGB curves if in Master mode)
                    if selectedChannel != .master && viewModel.curves.master.count > 1 {
                        renderCurvePath(points: viewModel.curves.master, size: size, height: height)
                            .stroke(Color.white.opacity(0.2), lineWidth: 1.5)
                    }
                    
                    // 5. Active Channel Fill Gradient under curve
                    renderCurvePath(points: currentPoints, size: size, height: height, closeToBottom: true)
                        .fill(
                            LinearGradient(
                                gradient: Gradient(colors: [
                                    selectedChannel.color.opacity(0.18),
                                    selectedChannel.color.opacity(0.01)
                                ]),
                                startPoint: .top,
                                endPoint: .bottom
                            )
                        )
                    
                    // 6. Active Channel Spline Line
                    renderCurvePath(points: currentPoints, size: size, height: height)
                        .stroke(selectedChannel.color, style: StrokeStyle(lineWidth: 2.0, lineCap: .round, lineJoin: .round))
                        .shadow(color: selectedChannel.color.opacity(0.4), radius: 3)
                    
                    // 7. Interactive Control Points
                    ForEach(Array(currentPoints.enumerated()), id: \.offset) { index, pt in
                        let px = CGFloat(pt.x) * size
                        let py = (1.0 - CGFloat(pt.y)) * height
                        let isHovered = hoveredPointIndex == index
                        let isDragged = draggingPointIndex == index
                        
                        ZStack {
                            // Outer active aura
                            Circle()
                                .fill(selectedChannel.color.opacity(isDragged ? 0.4 : (isHovered ? 0.25 : 0.0)))
                                .frame(width: 22, height: 22)
                            
                            // Outer ring
                            Circle()
                                .stroke(Color.black.opacity(0.8), lineWidth: 2)
                                .background(Circle().fill(selectedChannel.color))
                                .frame(width: isDragged ? 12 : 10, height: isDragged ? 12 : 10)
                        }
                        .position(x: px, y: py)
                        .onHover { hovering in
                            if hovering {
                                hoveredPointIndex = index
                            } else if hoveredPointIndex == index {
                                hoveredPointIndex = nil
                            }
                        }
                    }
                }
                .frame(width: size, height: height)
                .contentShape(Rectangle())
                .gesture(
                    DragGesture(minimumDistance: 0)
                        .onChanged { value in
                            handleDrag(at: value.location, in: CGSize(width: size, height: height))
                        }
                        .onEnded { _ in
                            draggingPointIndex = nil
                            activeCoordinates = nil
                            viewModel.saveCurrentEdits()
                        }
                )
                .onTapGesture(count: 2) {
                    // Double-tap on graph deletes nearest midtone point or resets endpoint
                    if let idx = hoveredPointIndex {
                        deleteOrResetPoint(at: idx)
                    }
                }
            }
            .frame(height: 200)
            .padding(.vertical, 2)
            
            // Bottom Zone Labels
            HStack {
                Text("Blacks")
                Spacer()
                Text("Shadows")
                Spacer()
                Text("Midtones")
                Spacer()
                Text("Highlights")
            }
            .font(.system(size: 9))
            .foregroundColor(.secondary.opacity(0.7))
            .padding(.horizontal, 2)
        }
    }
    
    // MARK: - Spline Path Evaluation
    private func renderCurvePath(points: [CurvePointRecord], size: CGFloat, height: CGFloat, closeToBottom: Bool = false) -> Path {
        let samples = evaluateCurveLut(points: points)
        guard samples.count > 1 else { return Path() }
        
        var path = Path()
        let step = size / CGFloat(samples.count - 1)
        
        let firstY = (1.0 - CGFloat(samples[0])) * height
        path.move(to: CGPoint(x: 0, y: firstY))
        
        for i in 1..<samples.count {
            let x = CGFloat(i) * step
            let y = (1.0 - CGFloat(samples[i])) * height
            path.addLine(to: CGPoint(x: x, y: y))
        }
        
        if closeToBottom {
            path.addLine(to: CGPoint(x: size, y: height))
            path.addLine(to: CGPoint(x: 0, y: height))
            path.closeSubpath()
        }
        
        return path
    }
    
    // MARK: - Gesture Handling
    private func handleDrag(at location: CGPoint, in size: CGSize) {
        let normX = Float((location.x / size.width).clamped(to: 0.0...1.0))
        let normY = Float((1.0 - (location.y / size.height)).clamped(to: 0.0...1.0))
        
        activeCoordinates = (input: Int((normX * 255.0).rounded()), output: Int((normY * 255.0).rounded()))
        
        var pts = currentPoints
        
        // 1. If dragging an existing point
        if let idx = draggingPointIndex, idx < pts.count {
            updatePointPosition(at: idx, normX: normX, normY: normY, in: &pts)
            setPoints(pts)
            return
        }
        
        // 2. Find closest existing point within touch hit radius (20pt)
        let hitRadius: CGFloat = 20.0
        var closestIndex: Int? = nil
        var minDistance: CGFloat = .infinity
        
        for (idx, pt) in pts.enumerated() {
            let px = CGFloat(pt.x) * size.width
            let py = (1.0 - CGFloat(pt.y)) * size.height
            let d = hypot(location.x - px, location.y - py)
            if d < hitRadius && d < minDistance {
                minDistance = d
                closestIndex = idx
            }
        }
        
        if let idx = closestIndex {
            draggingPointIndex = idx
            updatePointPosition(at: idx, normX: normX, normY: normY, in: &pts)
            setPoints(pts)
        } else {
            // 3. Clicked in empty space on the graph: add a new control point!
            // Do not add point too close to an existing point in X
            let canAdd = !pts.contains { abs($0.x - normX) < 0.04 }
            if canAdd && pts.count < 16 {
                let newPoint = CurvePointRecord(x: normX, y: normY)
                pts.append(newPoint)
                pts.sort { $0.x < $1.x }
                if let newIdx = pts.firstIndex(of: newPoint) {
                    draggingPointIndex = newIdx
                }
                setPoints(pts)
            }
        }
    }
    
    private func updatePointPosition(at idx: Int, normX: Float, normY: Float, in pts: inout [CurvePointRecord]) {
        if idx == 0 {
            // Start endpoint: fixed at x = 0, y can be lifted/dropped
            pts[idx].x = 0.0
            pts[idx].y = normY
        } else if idx == pts.count - 1 {
            // End endpoint: fixed at x = 1, y can be lifted/dropped
            pts[idx].x = 1.0
            pts[idx].y = normY
        } else {
            // Interior midtone point: constrain x between neighbors with a small margin
            let minX = pts[idx - 1].x + 0.02
            let maxX = pts[idx + 1].x - 0.02
            pts[idx].x = normX.clamped(to: minX...maxX)
            pts[idx].y = normY
        }
    }
    
    private func deleteOrResetPoint(at idx: Int) {
        var pts = currentPoints
        guard idx < pts.count else { return }
        
        if idx == 0 {
            pts[0].y = 0.0
        } else if idx == pts.count - 1 {
            pts[pts.count - 1].y = 1.0
        } else {
            pts.remove(at: idx)
        }
        setPoints(pts)
        draggingPointIndex = nil
        hoveredPointIndex = nil
        viewModel.saveCurrentEdits()
    }
    
    private func resetActiveChannel() {
        let linear = [CurvePointRecord(x: 0.0, y: 0.0), CurvePointRecord(x: 1.0, y: 1.0)]
        setPoints(linear)
        viewModel.saveCurrentEdits()
    }
    
    private func resetAllCurves() {
        viewModel.curves = .defaultIdentity
        draggingPointIndex = nil
        hoveredPointIndex = nil
        activeCoordinates = nil
        viewModel.saveCurrentEdits()
    }
}

fileprivate extension Comparable {
    func clamped(to limits: ClosedRange<Self>) -> Self {
        min(max(self, limits.lowerBound), limits.upperBound)
    }
}
