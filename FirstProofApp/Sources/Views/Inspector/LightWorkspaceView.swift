import SwiftUI

public struct LightWorkspaceView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    public init() {}
    
    public var body: some View {
        @Bindable var viewModel = viewModel
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                Text("LIGHT & TONE")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                Spacer()
                Button(action: {
                    viewModel.applyAutoAdjustments()
                }) {
                    Label("Auto", systemImage: "wand.and.stars")
                        .font(.caption)
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
                .help("Automatically calculate conservative exposure and contrast (Cmd+U)")
            }
            
            SliderRow(
                label: "Exposure",
                value: $viewModel.exposure,
                range: -3.0...3.0,
                format: "%.2f EV",
                onReset: { viewModel.resetSlider(.exposure) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .exposure) }
            )
            
            SliderRow(
                label: "Contrast",
                value: $viewModel.contrast,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.contrast) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .contrast) }
            )
            
            Divider()
                .padding(.vertical, 2)
            
            Text("TONAL RECOVERY")
                .font(.system(size: 10, weight: .bold))
                .foregroundColor(.secondary)
            
            SliderRow(
                label: "Highlights",
                value: $viewModel.highlights,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.highlights) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .highlights) }
            )
            
            SliderRow(
                label: "Shadows",
                value: $viewModel.shadows,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.shadows) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .shadows) }
            )
            
            SliderRow(
                label: "Whites",
                value: $viewModel.whites,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.whites) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .whites) }
            )
            
            SliderRow(
                label: "Blacks",
                value: $viewModel.blacks,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.blacks) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .blacks) }
            )
            
            Divider()
                .padding(.vertical, 2)
            
            ToneCurveView()
        }
        .padding(.top, 4)
    }
}
