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
                    slider: .sharpeningAmount,
                    value: $viewModel.sharpeningAmount,
                    onReset: { viewModel.resetSlider(.sharpeningAmount) },
                    onEditingChanged: { viewModel.handleSliderEditing($0, slider: .sharpeningAmount) },
                    onCancel: { viewModel.cancelSliderScrub() },
                    customLabel: "Amount"
                )
                
                SliderRow(
                    slider: .sharpeningRadius,
                    value: $viewModel.sharpeningRadius,
                    onReset: { viewModel.resetSlider(.sharpeningRadius) },
                    onEditingChanged: { viewModel.handleSliderEditing($0, slider: .sharpeningRadius) },
                    onCancel: { viewModel.cancelSliderScrub() },
                    customLabel: "Radius"
                )
                
                SliderRow(
                    slider: .sharpeningMasking,
                    value: $viewModel.sharpeningMasking,
                    onReset: { viewModel.resetSlider(.sharpeningMasking) },
                    onEditingChanged: { viewModel.handleSliderEditing($0, slider: .sharpeningMasking) },
                    onCancel: { viewModel.cancelSliderScrub() },
                    customLabel: "Masking"
                )
            }
            
            Divider()
            
            // Section 2: Noise Reduction
            VStack(alignment: .leading, spacing: 10) {
                Text("NOISE REDUCTION")
                    .font(.system(size: 10, weight: .bold))
                    .foregroundColor(.secondary)
                
                SliderRow(
                    slider: .noiseReduction,
                    value: $viewModel.noiseReduction,
                    onReset: { viewModel.resetSlider(.noiseReduction) },
                    onEditingChanged: { viewModel.handleSliderEditing($0, slider: .noiseReduction) },
                    onCancel: { viewModel.cancelSliderScrub() },
                    customLabel: "Luminance"
                )
            }
        }
        .padding(.top, 4)
    }
}
