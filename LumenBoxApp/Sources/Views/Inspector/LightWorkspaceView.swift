import SwiftUI

public struct LightWorkspaceView: View {
    @Bindable public var viewModel: AppViewModel
    
    public init(viewModel: AppViewModel) {
        self.viewModel = viewModel
    }
    
    public var body: some View {
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
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            SliderRow(
                label: "Contrast",
                value: $viewModel.contrast,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.contrast) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
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
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            SliderRow(
                label: "Shadows",
                value: $viewModel.shadows,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.shadows) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            SliderRow(
                label: "Whites",
                value: $viewModel.whites,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.whites) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            SliderRow(
                label: "Blacks",
                value: $viewModel.blacks,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.blacks) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            Divider()
                .padding(.vertical, 2)
            
            ToneCurveView(viewModel: viewModel)
        }
        .padding(.top, 4)
    }
}
