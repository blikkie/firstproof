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
                slider: .exposure,
                value: $viewModel.exposure,
                onReset: { viewModel.resetSlider(.exposure) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .exposure) },
                onCancel: { viewModel.cancelSliderScrub() }
            )
            
            SliderRow(
                slider: .contrast,
                value: $viewModel.contrast,
                onReset: { viewModel.resetSlider(.contrast) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .contrast) },
                onCancel: { viewModel.cancelSliderScrub() }
            )
            
            Divider()
                .padding(.vertical, 2)
            
            Text("TONAL RECOVERY")
                .font(.system(size: 10, weight: .bold))
                .foregroundColor(.secondary)
            
            SliderRow(
                slider: .highlights,
                value: $viewModel.highlights,
                onReset: { viewModel.resetSlider(.highlights) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .highlights) },
                onCancel: { viewModel.cancelSliderScrub() }
            )
            
            SliderRow(
                slider: .shadows,
                value: $viewModel.shadows,
                onReset: { viewModel.resetSlider(.shadows) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .shadows) },
                onCancel: { viewModel.cancelSliderScrub() }
            )
            
            SliderRow(
                slider: .whites,
                value: $viewModel.whites,
                onReset: { viewModel.resetSlider(.whites) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .whites) },
                onCancel: { viewModel.cancelSliderScrub() }
            )
            
            SliderRow(
                slider: .blacks,
                value: $viewModel.blacks,
                onReset: { viewModel.resetSlider(.blacks) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .blacks) },
                onCancel: { viewModel.cancelSliderScrub() }
            )
            
            Divider()
                .padding(.vertical, 2)
            
            ToneCurveView()
        }
        .padding(.top, 4)
    }
}
