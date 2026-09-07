import SwiftUI

public struct DetailWorkspaceView: View {
    @Bindable public var viewModel: AppViewModel
    
    public init(viewModel: AppViewModel) {
        self.viewModel = viewModel
    }
    
    public var body: some View {
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
                    onEditingChanged: { isEditing in
                        if !isEditing { viewModel.saveCurrentEdits() }
                    }
                )
                
                SliderRow(
                    label: "Radius",
                    value: $viewModel.sharpeningRadius,
                    range: 0.5...3.0,
                    format: "%.1f px",
                    onReset: { viewModel.resetSlider(.sharpeningRadius) },
                    onEditingChanged: { isEditing in
                        if !isEditing { viewModel.saveCurrentEdits() }
                    }
                )
                
                SliderRow(
                    label: "Masking",
                    value: $viewModel.sharpeningMasking,
                    range: 0.0...100.0,
                    format: "%.0f",
                    onReset: { viewModel.resetSlider(.sharpeningMasking) },
                    onEditingChanged: { isEditing in
                        if !isEditing { viewModel.saveCurrentEdits() }
                    }
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
                    onEditingChanged: { isEditing in
                        if !isEditing { viewModel.saveCurrentEdits() }
                    }
                )
            }
            
            // Helpful hint box
            VStack(alignment: .leading, spacing: 6) {
                HStack(spacing: 5) {
                    Image(systemName: "info.circle")
                        .foregroundColor(.secondary)
                        .font(.caption2)
                    Text("Capture De-Blur & Masking")
                        .font(.caption2)
                        .fontWeight(.semibold)
                        .foregroundColor(.secondary)
                }
                Text("Masking suppresses sharpening in flat areas like skies and skin. Double-click any label to reset to camera baseline.")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
            .padding(8)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color.secondary.opacity(0.06))
            .cornerRadius(6)
        }
        .padding(.top, 4)
    }
}
