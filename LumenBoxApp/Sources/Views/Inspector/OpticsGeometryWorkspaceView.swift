import SwiftUI

public struct OpticsGeometryWorkspaceView: View {
    @Bindable public var viewModel: AppViewModel
    
    public init(viewModel: AppViewModel) {
        self.viewModel = viewModel
    }
    
    public var body: some View {
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
                
                HStack(spacing: 8) {
                    Picker("Aspect Ratio", selection: Binding<CropAspectRatioPreset>(
                        get: { viewModel.cropAspectRatioPreset },
                        set: { viewModel.setCropAspectRatioPreset($0) }
                    )) {
                        ForEach(CropAspectRatioPreset.allCases) { preset in
                            Text(preset.rawValue).tag(preset)
                        }
                    }
                    .pickerStyle(.menu)
                    
                    Button(action: {
                        viewModel.flipCropOrientation()
                    }) {
                        Image(systemName: "arrow.left.and.right.righttriangle.left.righttriangle.right.fill")
                            .font(.system(size: 11))
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.small)
                    .help("Flip Aspect Ratio Orientation (X)")
                }
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
                    label: "Straighten",
                    value: $viewModel.straightenAngle,
                    range: -45.0...45.0,
                    format: "%.1f°",
                    onReset: { viewModel.resetSlider(.straighten) },
                    onEditingChanged: { isEditing in
                        if !isEditing { viewModel.saveCurrentEdits() }
                    }
                )
                
                SliderRow(
                    label: "Vertical",
                    value: $viewModel.perspectiveVertical,
                    range: -100.0...100.0,
                    format: "%.0f",
                    onReset: { viewModel.resetSlider(.perspectiveVertical) },
                    onEditingChanged: { isEditing in
                        if !isEditing { viewModel.saveCurrentEdits() }
                    }
                )
                
                SliderRow(
                    label: "Horizontal",
                    value: $viewModel.perspectiveHorizontal,
                    range: -100.0...100.0,
                    format: "%.0f",
                    onReset: { viewModel.resetSlider(.perspectiveHorizontal) },
                    onEditingChanged: { isEditing in
                        if !isEditing { viewModel.saveCurrentEdits() }
                    }
                )
                
                Toggle("Constrain Crop (Auto-Crop)", isOn: $viewModel.cropConstrain)
                    .toggleStyle(.switch)
                    .onChange(of: viewModel.cropConstrain) { _, _ in
                        viewModel.saveCurrentEdits()
                    }
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
