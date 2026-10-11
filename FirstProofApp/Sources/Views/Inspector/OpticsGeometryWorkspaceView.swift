import SwiftUI

public struct OpticsGeometryWorkspaceView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    public init() {}
    
    public var body: some View {
        @Bindable var viewModel = viewModel
        VStack(alignment: .leading, spacing: 14) {
            // MARK: 1. Crop & Aspect Ratio
            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Text("CROP & COMPOSITION")
                        .font(.caption)
                        .fontWeight(.bold)
                        .foregroundColor(.secondary)
                    Spacer()
                    if viewModel.isCropToolActive || viewModel.cropLeft > 0.001 || viewModel.cropTop > 0.001 || viewModel.cropRight < 0.999 || viewModel.cropBottom < 0.999 {
                        Button("Reset Crop") {
                            viewModel.resetCrop()
                        }
                        .buttonStyle(.plain)
                        .font(.caption2)
                        .foregroundColor(.accentColor)
                    }
                }
                
                Button(action: {
                    viewModel.toggleCropTool()
                }) {
                    HStack {
                        Image(systemName: viewModel.isCropToolActive ? "checkmark" : "crop")
                        Text(viewModel.isCropToolActive ? "Done Crop (Return)" : "Interactive Crop (C)")
                            .fontWeight(.medium)
                    }
                    .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
                .tint(viewModel.isCropToolActive ? .green : .accentColor)
                .controlSize(.regular)
            }
            
            Divider()
            
            // MARK: 2. Perspective & Straighten
            VStack(alignment: .leading, spacing: 10) {
                HStack {
                    Text("PERSPECTIVE & KEYSTONE")
                        .font(.caption)
                        .fontWeight(.bold)
                        .foregroundColor(.secondary)
                    Spacer()
                    if viewModel.straightenAngle != 0 || viewModel.perspectiveVertical != 0 || viewModel.perspectiveHorizontal != 0 {
                        Button("Reset Geometry") {
                            viewModel.resetGeometry()
                        }
                        .buttonStyle(.plain)
                        .font(.caption2)
                        .foregroundColor(.accentColor)
                    }
                }
                
                SliderRow(
                    slider: .straighten,
                    value: $viewModel.straightenAngle,
                    onReset: { viewModel.resetSlider(.straighten) },
                    onEditingChanged: { viewModel.handleSliderEditing($0, slider: .straighten) },
                    onCancel: { viewModel.cancelSliderScrub() }
                )
                
                SliderRow(
                    slider: .perspectiveVertical,
                    value: $viewModel.perspectiveVertical,
                    onReset: { viewModel.resetSlider(.perspectiveVertical) },
                    onEditingChanged: { viewModel.handleSliderEditing($0, slider: .perspectiveVertical) },
                    onCancel: { viewModel.cancelSliderScrub() },
                    customLabel: "Vertical"
                )
                
                SliderRow(
                    slider: .perspectiveHorizontal,
                    value: $viewModel.perspectiveHorizontal,
                    onReset: { viewModel.resetSlider(.perspectiveHorizontal) },
                    onEditingChanged: { viewModel.handleSliderEditing($0, slider: .perspectiveHorizontal) },
                    onCancel: { viewModel.cancelSliderScrub() },
                    customLabel: "Horizontal"
                )
                
                Toggle("Constrain Crop (Auto-Crop)", isOn: Binding<Bool>(
                    get: { viewModel.cropConstrain },
                    set: { _ in viewModel.toggleCropConstrain() }
                ))
                .toggleStyle(.switch)
            }
            
            Divider()
            
            // MARK: 3. Orientation (Rotate & Flip)
            VStack(alignment: .leading, spacing: 8) {
                Text("ORIENTATION")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                
                HStack(spacing: 8) {
                    Button(action: { viewModel.rotateCounterClockwise() }) {
                        Label("Rotate Left", systemImage: "rotate.left")
                            .font(.caption)
                            .frame(maxWidth: .infinity)
                    }
                    .help("Rotate 90° Counter-Clockwise (Cmd+[)")
                    
                    Button(action: { viewModel.rotateClockwise() }) {
                        Label("Rotate Right", systemImage: "rotate.right")
                            .font(.caption)
                            .frame(maxWidth: .infinity)
                    }
                    .help("Rotate 90° Clockwise (Cmd+])")
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
                
                HStack(spacing: 8) {
                    Button(action: { viewModel.toggleFlipHorizontal() }) {
                        Label("Flip H", systemImage: "flip.horizontal")
                            .font(.caption)
                            .frame(maxWidth: .infinity)
                    }
                    .tint(viewModel.flipHorizontal ? .accentColor : nil)
                    .help("Flip Horizontal")
                    
                    Button(action: { viewModel.toggleFlipVertical() }) {
                        Label("Flip V", systemImage: "arrow.up.and.down.righttriangle.up.righttriangle.down")
                            .font(.caption)
                            .frame(maxWidth: .infinity)
                    }
                    .tint(viewModel.flipVertical ? .accentColor : nil)
                    .help("Flip Vertical")
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
            }
            
            Divider()
            
            // MARK: 4. Lens Corrections
            VStack(alignment: .leading, spacing: 8) {
                Text("LENS CORRECTIONS")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                
                Toggle("Lens Correction (Lensfun)", isOn: $viewModel.lensCorrectionEnabled)
                    .toggleStyle(.switch)
                
                if let coeff = viewModel.lensCoefficients {
                    VStack(alignment: .leading, spacing: 2) {
                        if let lens = viewModel.selectedPhoto?.lensModel, !lens.isEmpty {
                            Text(lens)
                                .font(.caption2)
                                .foregroundColor(.secondary)
                                .lineLimit(1)
                        }
                        Text("Model: \(coeff.distortionModel.uppercased())")
                            .font(.system(size: 9))
                            .foregroundColor(.secondary.opacity(0.8))
                    }
                    .padding(6)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color.secondary.opacity(0.08))
                    .cornerRadius(4)
                }
            }
        }
        .padding(.top, 4)
    }
}
