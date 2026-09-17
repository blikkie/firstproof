import SwiftUI
import AppKit

public struct ColorWheelView: View {
    public let title: String
    @Binding public var hue: Float          // 0 .. 360 degrees
    @Binding public var saturation: Float   // 0 .. 100 percent
    public let onReset: () -> Void
    public var onEditingChanged: ((Bool) -> Void)? = nil
    
    public init(
        title: String,
        hue: Binding<Float>,
        saturation: Binding<Float>,
        onReset: @escaping () -> Void,
        onEditingChanged: ((Bool) -> Void)? = nil
    ) {
        self.title = title
        self._hue = hue
        self._saturation = saturation
        self.onReset = onReset
        self.onEditingChanged = onEditingChanged
    }
    
    public var body: some View {
        VStack(alignment: .center, spacing: 6) {
            HStack {
                Text(title)
                    .font(.caption)
                    .fontWeight(.medium)
                    .foregroundColor(.primary)
                    .onTapGesture(count: 2, perform: onReset)
                Spacer()
                Text("\(Int(saturation))%")
                    .font(.caption2)
                    .monospacedDigit()
                    .foregroundColor(saturation > 0 ? .primary : .secondary)
            }
            
            // Strict 1:1 Aspect Ratio via native SquareAspectLayout
            SquareAspectLayout {
                GeometryReader { geo in
                    let size = min(geo.size.width, geo.size.height)
                    let radius = size / 2.0
                    let center = CGPoint(x: geo.size.width / 2.0, y: geo.size.height / 2.0)
                    
                    let angleRad = Double(hue) * .pi / 180.0
                    let maxR = Double(radius - 6.0)
                    let r = Double(saturation) / 100.0 * maxR
                    let puckX = center.x + CGFloat(cos(angleRad) * r)
                    let puckY = center.y + CGFloat(sin(angleRad) * r)
                    
                    ZStack {
                        // 1. Angular chromatic hue gradient
                        Circle()
                            .fill(
                                AngularGradient(
                                    gradient: Gradient(colors: [
                                        Color(hue: 0.0 / 6.0, saturation: 1.0, brightness: 0.85),
                                        Color(hue: 1.0 / 6.0, saturation: 1.0, brightness: 0.85),
                                        Color(hue: 2.0 / 6.0, saturation: 1.0, brightness: 0.85),
                                        Color(hue: 3.0 / 6.0, saturation: 1.0, brightness: 0.85),
                                        Color(hue: 4.0 / 6.0, saturation: 1.0, brightness: 0.85),
                                        Color(hue: 5.0 / 6.0, saturation: 1.0, brightness: 0.85),
                                        Color(hue: 6.0 / 6.0, saturation: 1.0, brightness: 0.85),
                                    ]),
                                    center: .center
                                )
                            )
                            .opacity(0.85)
                        
                        // 2. Radial desaturation mask (neutral in center -> transparent at perimeter)
                        Circle()
                            .fill(
                                RadialGradient(
                                    gradient: Gradient(colors: [
                                        Color(nsColor: .windowBackgroundColor).opacity(0.95),
                                        Color(nsColor: .windowBackgroundColor).opacity(0.5),
                                        Color.clear
                                    ]),
                                    center: .center,
                                    startRadius: 0,
                                    endRadius: radius
                                )
                            )
                        
                        // 3. Center neutral crosshairs & border ring
                        Circle()
                            .strokeBorder(Color.white.opacity(0.18), lineWidth: 1)
                        
                        Path { path in
                            path.move(to: CGPoint(x: center.x - 4, y: center.y))
                            path.addLine(to: CGPoint(x: center.x + 4, y: center.y))
                            path.move(to: CGPoint(x: center.x, y: center.y - 4))
                            path.addLine(to: CGPoint(x: center.x, y: center.y + 4))
                        }
                        .stroke(Color.secondary.opacity(0.45), lineWidth: 1)
                        .allowsHitTesting(false)
                        
                        // 4. Declarative render puck (hit-testing disabled to prevent gesture cancellation)
                        Circle()
                            .fill(Color(hue: Double(hue) / 360.0, saturation: Double(saturation) / 100.0, brightness: 0.95))
                            .frame(width: 12, height: 12)
                            .overlay(Circle().stroke(Color.white, lineWidth: 1.5))
                            .shadow(color: .black.opacity(0.4), radius: 2)
                            .position(x: puckX, y: puckY)
                            .allowsHitTesting(false)
                        
                        // 5. Stationary full-canvas gesture container
                        Color.clear
                            .contentShape(Circle())
                            .gesture(
                                DragGesture(minimumDistance: 0)
                                    .onChanged { value in
                                        onEditingChanged?(true)
                                        let dx = value.location.x - center.x
                                        let dy = value.location.y - center.y
                                        let dist = sqrt(dx * dx + dy * dy)
                                        let sat = min(max(Float(dist / CGFloat(maxR)) * 100.0, 0.0), 100.0)
                                        
                                        var deg = Float(atan2(dy, dx) * 180.0 / .pi)
                                        if deg < 0 { deg += 360.0 }
                                        
                                        if sat > 1.0 {
                                            self.hue = deg
                                        }
                                        self.saturation = sat
                                    }
                                    .onEnded { _ in
                                        onEditingChanged?(false)
                                    }
                            )
                            .onTapGesture(count: 2) {
                                onReset()
                            }
                    }
                }
            }
            .clipped()
        }
    }
}
