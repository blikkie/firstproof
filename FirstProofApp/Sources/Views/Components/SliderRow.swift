import SwiftUI
import AppKit

public struct TactileSlider: View {
    @Binding public var value: Float
    public let range: ClosedRange<Float>
    public var onReset: (() -> Void)? = nil
    public var onEditingChanged: ((Bool) -> Void)? = nil
    
    @State private var isDragging: Bool = false
    @State private var dragStartValue: Float = 0
    @State private var dragStartLocationX: CGFloat = 0
    
    public init(
        value: Binding<Float>,
        range: ClosedRange<Float>,
        onReset: (() -> Void)? = nil,
        onEditingChanged: ((Bool) -> Void)? = nil
    ) {
        self._value = value
        self.range = range
        self.onReset = onReset
        self.onEditingChanged = onEditingChanged
    }
    
    private func normalizedPosition(for val: Float) -> CGFloat {
        let span = range.upperBound - range.lowerBound
        guard span > 0 else { return 0.5 }
        let clamped = min(max(val, range.lowerBound), range.upperBound)
        return CGFloat((clamped - range.lowerBound) / span)
    }
    
    private func valueForNormalized(_ norm: CGFloat) -> Float {
        let span = range.upperBound - range.lowerBound
        let clamped = min(max(norm, 0.0), 1.0)
        return min(max(range.lowerBound + Float(clamped) * span, range.lowerBound), range.upperBound)
    }
    
    private var isBipolar: Bool {
        range.lowerBound < 0 && range.upperBound > 0
    }
    
    public var body: some View {
        GeometryReader { geo in
            let totalWidth = geo.size.width
            let knobDiameter: CGFloat = 16
            let usableTrackWidth = max(totalWidth - knobDiameter, 1)
            let norm = normalizedPosition(for: value)
            let knobCenterX = knobDiameter / 2 + norm * usableTrackWidth
            let zeroNorm = isBipolar ? normalizedPosition(for: 0.0) : 0.0
            let zeroCenterX = knobDiameter / 2 + zeroNorm * usableTrackWidth
            
            ZStack(alignment: .leading) {
                // 1. Recessed High-Contrast Groove Track
                RoundedRectangle(cornerRadius: 3.5)
                    .fill(Color(red: 0.04, green: 0.04, blue: 0.06))
                    .frame(height: 7)
                    .overlay(
                        RoundedRectangle(cornerRadius: 3.5)
                            .stroke(Color.white.opacity(0.20), lineWidth: 1)
                    )
                    .shadow(color: Color.black.opacity(0.8), radius: 1, x: 0, y: 1)
                    .padding(.horizontal, knobDiameter / 2)
                
                // 2. Active Fill (Machined Silver)
                if isBipolar {
                    let fillLeft = min(zeroCenterX, knobCenterX)
                    let fillWidth = abs(knobCenterX - zeroCenterX)
                    RoundedRectangle(cornerRadius: 2)
                        .fill(Color.white.opacity(0.40))
                        .frame(width: fillWidth, height: 3.5)
                        .offset(x: fillLeft)
                } else if norm > 0.001 {
                    let fillWidth = norm * usableTrackWidth
                    RoundedRectangle(cornerRadius: 2)
                        .fill(Color.white.opacity(0.40))
                        .frame(width: fillWidth, height: 3.5)
                        .offset(x: knobDiameter / 2)
                }
                
                // 3. Center Detent Tick (for Bipolar Sliders)
                if isBipolar {
                    Rectangle()
                        .fill(Color.white.opacity(0.50))
                        .frame(width: 1.5, height: 7)
                        .offset(x: zeroCenterX - 0.75)
                }
                
                // 4. Solid Opaque Bead-Blasted Aluminum Knob (Zero Translucency / Zero Glass)
                ZStack {
                    Circle()
                        .fill(Color(white: 0.94))
                        .frame(width: knobDiameter, height: knobDiameter)
                        .overlay(
                            Circle()
                                .stroke(Color(red: 0.08, green: 0.08, blue: 0.10), lineWidth: 1.5)
                        )
                        .overlay(
                            Circle()
                                .stroke(Color.white.opacity(0.8), lineWidth: 0.5)
                                .padding(1)
                        )
                        .shadow(color: Color.black.opacity(0.7), radius: 2.5, x: 0, y: 1.5)
                    
                    // Tactile center pip
                    Circle()
                        .fill(Color(white: 0.35))
                        .frame(width: 3, height: 3)
                }
                .scaleEffect(isDragging ? 1.12 : 1.0)
                .offset(x: knobCenterX - knobDiameter / 2)
            }
            .frame(height: 20)
            .contentShape(Rectangle())
            .gesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { gesture in
                        if !isDragging {
                            isDragging = true
                            dragStartValue = value
                            dragStartLocationX = gesture.startLocation.x
                            onEditingChanged?(true)
                        }
                        
                        let isShift = NSEvent.modifierFlags.contains(.shift)
                        if isShift {
                            let deltaX = gesture.location.x - dragStartLocationX
                            let fineRatio: CGFloat = 0.10
                            let deltaNorm = (deltaX * fineRatio) / usableTrackWidth
                            let baseNorm = normalizedPosition(for: dragStartValue)
                            let newNorm = min(max(baseNorm + deltaNorm, 0.0), 1.0)
                            value = self.valueForNormalized(newNorm)
                        } else {
                            let clickLocationX = gesture.location.x
                            let newNorm = min(max((clickLocationX - knobDiameter / 2) / usableTrackWidth, 0.0), 1.0)
                            value = self.valueForNormalized(newNorm)
                        }
                    }
                    .onEnded { _ in
                        isDragging = false
                        onEditingChanged?(false)
                    }
            )
            .onTapGesture(count: 2) {
                onReset?()
            }
        }
        .frame(height: 20)
    }
}

public struct SliderRow: View {
    public let label: String
    @Binding public var value: Float
    public let range: ClosedRange<Float>
    public let format: String
    public let onReset: () -> Void
    public var onEditingChanged: ((Bool) -> Void)? = nil
    public var multiplier: Float = 1.0
    
    public init(
        label: String,
        value: Binding<Float>,
        range: ClosedRange<Float>,
        format: String,
        onReset: @escaping () -> Void,
        onEditingChanged: ((Bool) -> Void)? = nil,
        multiplier: Float = 1.0
    ) {
        self.label = label
        self._value = value
        self.range = range
        self.format = format
        self.onReset = onReset
        self.onEditingChanged = onEditingChanged
        self.multiplier = multiplier
    }
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(label)
                    .font(.subheadline)
                    .onTapGesture(count: 2, perform: onReset) // Double click label to reset!
                Spacer()
                Text(String(format: format, value * multiplier))
                    .font(.caption)
                    .monospacedDigit()
                    .foregroundColor(.secondary)
            }
            
            TactileSlider(
                value: $value,
                range: range,
                onReset: onReset,
                onEditingChanged: onEditingChanged
            )
        }
    }
}

public struct TemperatureSliderRow: View {
    public let label: String
    @Binding public var kelvin: Float
    public let onReset: () -> Void
    public var onEditingChanged: ((Bool) -> Void)? = nil
    
    public init(
        label: String,
        kelvin: Binding<Float>,
        onReset: @escaping () -> Void,
        onEditingChanged: ((Bool) -> Void)? = nil
    ) {
        self.label = label
        self._kelvin = kelvin
        self.onReset = onReset
        self.onEditingChanged = onEditingChanged
    }
    
    // Perceptual Mired mapping:
    // Human color perception of blackbody temperature is linear in Mireds (10^6 / K), not linear Kelvin.
    // 2000 K  -> 500 Mireds (pos = 0.0)
    // 5500 K  -> 181.8 Mireds (pos = 0.707)
    // 20000 K -> 50 Mireds (pos = 1.0)
    // This gives generous precision across 2,500K - 8,500K where 99% of photo adjustments occur.
    private var sliderPosition: Binding<Float> {
        Binding<Float>(
            get: {
                let k = min(max(kelvin, 2000.0), 20000.0)
                let mired = 1_000_000.0 / k
                let pos = (500.0 - mired) / 450.0
                return min(max(pos, 0.0), 1.0)
            },
            set: { newPos in
                let pos = min(max(newPos, 0.0), 1.0)
                let mired = 500.0 - pos * 450.0
                let rawK = 1_000_000.0 / mired
                // Stepped in 50K increments below 10,000K, 200K increments above
                let steppedK = (rawK < 10000.0)
                    ? (rawK / 50.0).rounded() * 50.0
                    : (rawK / 200.0).rounded() * 200.0
                kelvin = min(max(steppedK, 2000.0), 20000.0)
            }
        )
    }
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(label)
                    .font(.subheadline)
                    .onTapGesture(count: 2, perform: onReset)
                Spacer()
                Text(String(format: "%.0f K", kelvin))
                    .font(.caption)
                    .monospacedDigit()
                    .foregroundColor(.secondary)
            }
            
            TactileSlider(
                value: sliderPosition,
                range: 0.0...1.0,
                onReset: onReset,
                onEditingChanged: onEditingChanged
            )
        }
    }
}

public struct HueSliderRow: View {
    public let label: String
    @Binding public var hue: Float
    public let onReset: () -> Void
    public var onEditingChanged: ((Bool) -> Void)? = nil
    
    public init(
        label: String,
        hue: Binding<Float>,
        onReset: @escaping () -> Void,
        onEditingChanged: ((Bool) -> Void)? = nil
    ) {
        self.label = label
        self._hue = hue
        self.onReset = onReset
        self.onEditingChanged = onEditingChanged
    }
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                HStack(spacing: 6) {
                    Circle()
                        .fill(Color(hue: Double(hue) / 360.0, saturation: 0.85, brightness: 0.95))
                        .frame(width: 8, height: 8)
                    Text(label)
                        .font(.subheadline)
                        .onTapGesture(count: 2, perform: onReset)
                }
                Spacer()
                Text(String(format: "%.0f°", hue))
                    .font(.caption)
                    .monospacedDigit()
                    .foregroundColor(.secondary)
            }
            
            TactileSlider(
                value: $hue,
                range: 0.0...360.0,
                onReset: onReset,
                onEditingChanged: onEditingChanged
            )
        }
    }
}

public struct MetadataRow: View {
    public let label: String
    public let value: String
    
    public init(label: String, value: String) {
        self.label = label
        self.value = value
    }
    
    public var body: some View {
        HStack {
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
            Spacer()
            Text(value)
                .font(.caption)
                .fontWeight(.medium)
                .lineLimit(1)
        }
    }
}
