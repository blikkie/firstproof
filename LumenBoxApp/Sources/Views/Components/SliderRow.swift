import SwiftUI
import AppKit

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
            
            Slider(value: $value, in: range) { isEditing in
                onEditingChanged?(isEditing)
            }
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
    // 5500 K  -> 181.8 Mireds (pos = 0.663)
    // 50000 K -> 20 Mireds (pos = 1.0)
    // This gives generous precision across 2,500K - 8,500K where 99% of photo adjustments occur.
    private var sliderPosition: Binding<Float> {
        Binding<Float>(
            get: {
                let k = min(max(kelvin, 2000.0), 50000.0)
                let mired = 1_000_000.0 / k
                let pos = (500.0 - mired) / 480.0
                return min(max(pos, 0.0), 1.0)
            },
            set: { newPos in
                let pos = min(max(newPos, 0.0), 1.0)
                let mired = 500.0 - pos * 480.0
                let rawK = 1_000_000.0 / mired
                // Stepped in 50K increments below 10,000K, 200K increments above
                let steppedK = (rawK < 10000.0)
                    ? (rawK / 50.0).rounded() * 50.0
                    : (rawK / 200.0).rounded() * 200.0
                kelvin = min(max(steppedK, 2000.0), 50000.0)
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
            
            Slider(value: sliderPosition, in: 0.0...1.0) { isEditing in
                onEditingChanged?(isEditing)
            }
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
