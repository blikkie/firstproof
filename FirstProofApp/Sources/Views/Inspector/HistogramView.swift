import SwiftUI
import AppKit
import FirstProofBridge

public struct HistogramView: View {
    @Environment(AppViewModel.self) private var viewModel
    @State private var hoverFrac: CGFloat? = nil
    
    public init() {}
    
    public var body: some View {
        VStack(spacing: 6) {
            // MARK: - HUD Telemetry Header
            hudHeader
            
            if viewModel.isHistogramVisible {
                // MARK: - Sensitometric Histogram Graph Canvas
                graphCanvas
                    .frame(height: 64)
                    .background(Color.black.opacity(0.25))
                    .cornerRadius(4)
                    .overlay(
                        RoundedRectangle(cornerRadius: 4)
                            .stroke(Color.secondary.opacity(0.12), lineWidth: 1)
                    )
                
                // MARK: - Ansel Adams Zone & EV Stop Scale Labels
                zoneScaleLabels
            }
        }
        .padding(8)
        .frame(maxWidth: .infinity)
        .background(Color.secondary.opacity(0.06))
        .cornerRadius(8)
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color.secondary.opacity(0.14), lineWidth: 1)
        )
    }
    
    // MARK: - HUD Header Bar (2-Line Layout to prevent truncation & support Dynamic Type)
    private var hudHeader: some View {
        let shadowClip = viewModel.histogramRecord?.shadowClipPercent ?? 0.0
        let sensorClip = viewModel.histogramRecord?.sensorClipPercent ?? 0.0
        let rolloff = viewModel.histogramRecord?.displayRolloffPercent ?? 0.0
        
        return VStack(alignment: .leading, spacing: 5) {
            // Line 1: Section Title & Interactive Hover Zone Readout + Collapse Button
            HStack {
                if let frac = hoverFrac {
                    let info = zoneReadout(for: frac)
                    HStack(spacing: 4) {
                        Text(info.zone)
                            .font(.system(size: 9.5, weight: .bold, design: .monospaced))
                            .foregroundColor(.primary)
                        Text("•")
                            .foregroundColor(.secondary.opacity(0.5))
                        Text(info.ev)
                            .font(.system(size: 9.5, weight: .bold, design: .monospaced))
                            .foregroundColor(info.ev.contains("+") ? Color(red: 0.95, green: 0.55, blue: 0.35) : (info.ev.contains("-") ? Color(red: 0.35, green: 0.65, blue: 0.95) : .accentColor))
                        Text("•")
                            .foregroundColor(.secondary.opacity(0.5))
                        Text(info.description)
                            .font(.system(size: 8.5, weight: .regular))
                            .foregroundColor(.secondary)
                            .lineLimit(1)
                            .truncationMode(.tail)
                    }
                    .help("\(info.zone) (\(info.description)): \(info.ev) relative to 18% middle grey (Zone V)")
                } else {
                    Text("HISTOGRAM (EV)")
                        .font(.system(size: 9.5, weight: .bold, design: .monospaced))
                        .foregroundColor(.secondary.opacity(0.8))
                }
                
                Spacer(minLength: 4)
                
                // Collapse / Expand Toggle
                Button(action: {
                    withAnimation(.easeInOut(duration: 0.15)) {
                        viewModel.toggleHistogram()
                    }
                }) {
                    Image(systemName: viewModel.isHistogramVisible ? "chevron.up" : "chevron.down")
                        .font(.system(size: 9, weight: .semibold))
                        .foregroundColor(.secondary.opacity(0.7))
                        .padding(2)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .help(viewModel.isHistogramVisible ? "Hide Histogram (H)" : "Show Histogram (H)")
            }
            .frame(minHeight: 20)
            
            // Line 2: Telemetry Badges (Shadow Floor, AgX Roll-Off, Sensor Saturation)
            HStack(spacing: 5) {
                // Shadow Floor Clipping Badge
                Button(action: {
                    viewModel.toggleClippingOverlay()
                }) {
                    HStack(spacing: 3) {
                        Image(systemName: "arrowtriangle.left.fill")
                            .font(.system(size: 7.5))
                        Text(String(format: "Floor %.1f%%", shadowClip))
                            .font(.system(size: 8.5, weight: .semibold, design: .monospaced))
                    }
                    .foregroundColor(shadowClip > 0.05 ? Color(red: 0.35, green: 0.60, blue: 0.95) : .secondary.opacity(0.6))
                    .padding(.horizontal, 5)
                    .padding(.vertical, 2.5)
                    .background(shadowClip > 0.05 ? Color.blue.opacity(0.18) : Color.secondary.opacity(0.08))
                    .cornerRadius(4)
                }
                .buttonStyle(.plain)
                .help("Shadow Floor: \(String(format: "%.2f%%", shadowClip)) crushed at noise floor (≤ −8.0 EV). Click or press J to toggle clipping overlay.")
                
                // AgX Shoulder Roll-Off Telemetry Badge
                if rolloff > 0.05 {
                    HStack(spacing: 3) {
                        Text("AgX")
                            .font(.system(size: 7.5, weight: .bold))
                        Text(String(format: "%.1f%%", rolloff))
                            .font(.system(size: 8.5, weight: .semibold, design: .monospaced))
                    }
                    .foregroundColor(Color(red: 0.92, green: 0.70, blue: 0.35))
                    .padding(.horizontal, 5)
                    .padding(.vertical, 2.5)
                    .background(Color.orange.opacity(0.14))
                    .cornerRadius(4)
                    .help("AgX Shoulder: \(String(format: "%.2f%%", rolloff)) in preserved highlight compression (Zone VII+).")
                }
                
                // Hardware Photosite Saturation Badge
                Button(action: {
                    viewModel.toggleClippingOverlay()
                }) {
                    HStack(spacing: 3) {
                        Text(String(format: "Clip %.1f%%", sensorClip))
                            .font(.system(size: 8.5, weight: .semibold, design: .monospaced))
                        Image(systemName: "arrowtriangle.right.fill")
                            .font(.system(size: 7.5))
                    }
                    .foregroundColor(sensorClip > 0.05 ? Color(red: 0.95, green: 0.30, blue: 0.30) : .secondary.opacity(0.6))
                    .padding(.horizontal, 5)
                    .padding(.vertical, 2.5)
                    .background(sensorClip > 0.05 ? Color.red.opacity(0.18) : Color.secondary.opacity(0.08))
                    .cornerRadius(4)
                }
                .buttonStyle(.plain)
                .help("Sensor Saturation: \(String(format: "%.2f%%", sensorClip)) blown raw Bayer photosites (≥ 1.0 capacity). Click or press J to toggle clipping overlay.")
                
                Spacer(minLength: 0)
            }
        }
    }
    
    // MARK: - Graph Canvas
    private var graphCanvas: some View {
        GeometryReader { geo in
            let w = geo.size.width
            let h = geo.size.height
            
            ZStack(alignment: .topLeading) {
                // 1. Ansel Adams Zone Reference Grid Lines
                Canvas { ctx, size in
                    drawZoneGrid(in: ctx, size: size)
                }
                
                // 2. Continuous Histogram Densities (Luminance + RGB traces)
                if let record = viewModel.histogramRecord, record.binsLuminance.count > 1 {
                    Canvas { ctx, size in
                        drawHistogramTraces(in: ctx, size: size, record: record)
                    }
                }
                
                // 3. Hover Hairline Indicator
                if let frac = hoverFrac {
                    let hx = frac * w
                    Path { p in
                        p.move(to: CGPoint(x: hx, y: 0))
                        p.addLine(to: CGPoint(x: hx, y: h))
                    }
                    .stroke(Color.white.opacity(0.5), style: StrokeStyle(lineWidth: 1, dash: [2, 2]))
                }
            }
            .contentShape(Rectangle())
            .onContinuousHover { phase in
                switch phase {
                case .active(let point):
                    hoverFrac = w > 0 ? min(max(point.x / w, 0.0), 1.0) : 0.0
                case .ended:
                    hoverFrac = nil
                }
            }
        }
    }
    
    // MARK: - Grid & Markers Drawing
    private func drawZoneGrid(in ctx: GraphicsContext, size: CGSize) {
        let totalStops: CGFloat = 12.0 // -8 EV to +4 EV
        
        for stop in 0...12 {
            let frac = CGFloat(stop) / totalStops
            let x = frac * size.width
            let ev = -8 + stop
            
            // Zone V (0.0 EV / 18% Middle Grey) - Distinct Accent Marker
            if ev == 0 {
                var p = Path()
                p.move(to: CGPoint(x: x, y: 0))
                p.addLine(to: CGPoint(x: x, y: size.height))
                ctx.stroke(p, with: .color(Color.accentColor.opacity(0.40)), style: StrokeStyle(lineWidth: 1, dash: [3, 2]))
            } else if ev == -5 || ev == 3 {
                // Zone 0 (-5 EV, Dmax) and Zone VIII (+3 EV, Textured White)
                var p = Path()
                p.move(to: CGPoint(x: x, y: size.height - 8))
                p.addLine(to: CGPoint(x: x, y: size.height))
                ctx.stroke(p, with: .color(Color.secondary.opacity(0.28)), lineWidth: 1)
            } else {
                // Standard 1-stop ticks along bottom
                var p = Path()
                p.move(to: CGPoint(x: x, y: size.height - 4))
                p.addLine(to: CGPoint(x: x, y: size.height))
                ctx.stroke(p, with: .color(Color.secondary.opacity(0.18)), lineWidth: 1)
            }
        }
    }
    
    // MARK: - Traces Drawing
    private func drawHistogramTraces(in ctx: GraphicsContext, size: CGSize, record: EvHistogramRecord) {
        let w = size.width
        let h = size.height
        let n = record.binsLuminance.count
        guard n > 1 else { return }
        
        // Helper to construct path for an array of normalized bins
        func buildPath(for bins: [Float]) -> Path {
            var path = Path()
            path.move(to: CGPoint(x: 0, y: h - CGFloat(bins[0]) * (h - 2)))
            for i in 1..<n {
                let x = CGFloat(i) / CGFloat(n - 1) * w
                let y = h - CGFloat(bins[i]) * (h - 2)
                path.addLine(to: CGPoint(x: x, y: y))
            }
            return path
        }
        
        // 1. Luminance Monochrome Filled Mass
        var fillPath = buildPath(for: record.binsLuminance)
        fillPath.addLine(to: CGPoint(x: w, y: h))
        fillPath.addLine(to: CGPoint(x: 0, y: h))
        fillPath.closeSubpath()
        
        let grad = Gradient(colors: [
            Color.secondary.opacity(0.32),
            Color.secondary.opacity(0.04)
        ])
        ctx.fill(fillPath, with: .linearGradient(grad, startPoint: CGPoint(x: 0, y: 0), endPoint: CGPoint(x: 0, y: h)))
        
        // 2. Muted Desaturated Channel Traces (Quiet Darkroom Tones)
        let rPath = buildPath(for: record.binsR)
        ctx.stroke(rPath, with: .color(Color(red: 0.88, green: 0.35, blue: 0.35).opacity(0.40)), lineWidth: 0.9)
        
        let gPath = buildPath(for: record.binsG)
        ctx.stroke(gPath, with: .color(Color(red: 0.35, green: 0.75, blue: 0.40).opacity(0.40)), lineWidth: 0.9)
        
        let bPath = buildPath(for: record.binsB)
        ctx.stroke(bPath, with: .color(Color(red: 0.38, green: 0.58, blue: 0.90).opacity(0.40)), lineWidth: 0.9)
        
        // 3. Luminance Stroke Outline
        let lumStroke = buildPath(for: record.binsLuminance)
        ctx.stroke(lumStroke, with: .color(Color.white.opacity(0.55)), lineWidth: 1.0)
    }
    
    // MARK: - Scale Labels (2-Line Layout for EV Stops & Zone Landmarks)
    private var zoneScaleLabels: some View {
        VStack(spacing: 3) {
            // Line 1: Physical Exposure Stops (-8 EV Noise Floor to +4 EV Sensor Saturation)
            HStack {
                Text("−8 EV (Floor)")
                    .frame(maxWidth: .infinity, alignment: .leading)
                Text("0 EV (18%)")
                    .foregroundColor(.accentColor.opacity(0.90))
                    .fontWeight(.semibold)
                    .frame(maxWidth: .infinity, alignment: .center)
                Text("+4 EV (Clip)")
                    .frame(maxWidth: .infinity, alignment: .trailing)
            }
            
            // Line 2: Ansel Adams Tonal Zone Landmarks
            HStack {
                Text("Zone 0 (Dmax)")
                    .frame(maxWidth: .infinity, alignment: .leading)
                Text("Zone V (Grey)")
                    .foregroundColor(.accentColor.opacity(0.90))
                    .fontWeight(.semibold)
                    .frame(maxWidth: .infinity, alignment: .center)
                Text("Zone VIII+ (White)")
                    .frame(maxWidth: .infinity, alignment: .trailing)
            }
        }
        .font(.system(size: 8.5, weight: .regular, design: .monospaced))
        .foregroundColor(.secondary.opacity(0.70))
        .fixedSize(horizontal: false, vertical: true)
    }
    
    // MARK: - Zone System Readout Helper
    private func zoneReadout(for frac: CGFloat) -> (zone: String, description: String, ev: String) {
        let totalStops: Float = 12.0
        let clampedFrac = Float(min(max(frac, 0.0), 1.0))
        let ev = -8.0 + clampedFrac * totalStops
        let evString = String(format: "%+.1f EV", ev)
        
        let zone: String
        let desc: String
        if ev < -5.5 {
            zone = "Zone 0"
            desc = "Sub-Floor / Noise"
        } else if ev < -4.5 {
            zone = "Zone 0"
            desc = "Dmax Maximum Black"
        } else if ev < -3.5 {
            zone = "Zone I"
            desc = "Deep Shadow"
        } else if ev < -2.5 {
            zone = "Zone II"
            desc = "First Texture"
        } else if ev < -1.5 {
            zone = "Zone III"
            desc = "Shadow Detail"
        } else if ev < -0.5 {
            zone = "Zone IV"
            desc = "Low Midtone"
        } else if ev <= 0.5 {
            zone = "Zone V"
            desc = "18% Middle Grey"
        } else if ev <= 1.5 {
            zone = "Zone VI"
            desc = "Skin Midtone"
        } else if ev <= 2.5 {
            zone = "Zone VII"
            desc = "Textured White"
        } else if ev <= 3.5 {
            zone = "Zone VIII"
            desc = "High Texture"
        } else if ev <= 4.0 {
            zone = "Zone IX"
            desc = "Blank White"
        } else {
            zone = "Zone X"
            desc = "Sensor Saturation"
        }
        
        return (zone, desc, evString)
    }
}
