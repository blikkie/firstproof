import SwiftUI

public struct DetailWorkspaceView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    public init() {}
    
    public var body: some View {
        @Bindable var viewModel = viewModel
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                Text("DETAIL & SHARPENING")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                Spacer()
                Button(action: {
                    if viewModel.isZoomed {
                        viewModel.zoomToFit()
                    } else {
                        viewModel.zoomTo100()
                    }
                }) {
                    Label(viewModel.isZoomed ? "Fit" : "1:1 Loupe", systemImage: viewModel.isZoomed ? "arrow.down.right.and.arrow.up.left" : "viewfinder")
                        .font(.caption)
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
                .help("Toggle 100% (1:1) Pixel Loupe to inspect sharpness and grain (Z)")
            }
            
            // Section 1: Sharpening
            VStack(alignment: .leading, spacing: 10) {
                Text("SHARPENING")
                    .font(.system(size: 10, weight: .bold))
                    .foregroundColor(.secondary)
                
                SliderRow(
                    label: "Amount",
                    value: $viewModel.sharpeningAmount,
                    range: 0.0...100.0,
                    format: "%.0f",
                    onReset: { viewModel.resetSlider(.sharpeningAmount) },
                    onEditingChanged: { viewModel.handleSliderEditing($0, slider: .sharpeningAmount) }
                )
                
                SliderRow(
                    label: "Radius",
                    value: $viewModel.sharpeningRadius,
                    range: 0.5...3.0,
                    format: "%.1f px",
                    onReset: { viewModel.resetSlider(.sharpeningRadius) },
                    onEditingChanged: { viewModel.handleSliderEditing($0, slider: .sharpeningRadius) }
                )
                
                SliderRow(
                    label: "Masking",
                    value: $viewModel.sharpeningMasking,
                    range: 0.0...100.0,
                    format: "%.0f",
                    onReset: { viewModel.resetSlider(.sharpeningMasking) },
                    onEditingChanged: { viewModel.handleSliderEditing($0, slider: .sharpeningMasking) }
                )
            }
            
            Divider()
            
            // Section 2: Noise Reduction
            VStack(alignment: .leading, spacing: 10) {
                Text("NOISE REDUCTION")
                    .font(.system(size: 10, weight: .bold))
                    .foregroundColor(.secondary)
                
                SliderRow(
                    label: "Luminance",
                    value: $viewModel.noiseReduction,
                    range: 0.0...100.0,
                    format: "%.0f",
                    onReset: { viewModel.resetSlider(.noiseReduction) },
                    onEditingChanged: { viewModel.handleSliderEditing($0, slider: .noiseReduction) }
                )
            }
        }
        .padding(.top, 4)
    }
}
