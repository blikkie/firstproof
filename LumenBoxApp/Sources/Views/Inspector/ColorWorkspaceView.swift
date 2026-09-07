import SwiftUI
import LumenBoxBridge

public struct ColorWorkspaceView: View {
    @Bindable public var viewModel: AppViewModel
    
    public init(viewModel: AppViewModel) {
        self.viewModel = viewModel
    }
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                Text("WHITE BALANCE")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                Spacer()
                Button(action: {
                    viewModel.isEyedropperActive.toggle()
                }) {
                    Image(systemName: "eyedropper")
                        .font(.system(size: 12, weight: .medium))
                        .foregroundColor(viewModel.isEyedropperActive ? .accentColor : .secondary)
                        .padding(4)
                        .background(
                            RoundedRectangle(cornerRadius: 4)
                                .fill(viewModel.isEyedropperActive ? Color.accentColor.opacity(0.2) : Color.clear)
                        )
                }
                .buttonStyle(.plain)
                .help("White Balance Eyedropper: Click any neutral patch in the photo (Esc to cancel)")
            }
            
            TemperatureSliderRow(
                label: "Temperature",
                kelvin: $viewModel.temperature,
                onReset: { viewModel.resetSlider(.temperature) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            SliderRow(
                label: "Tint",
                value: $viewModel.tint,
                range: -150.0...150.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.tint) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            Divider()
            
            Text("COLOR & VIBRANCE")
                .font(.caption)
                .fontWeight(.bold)
                .foregroundColor(.secondary)
            
            SliderRow(
                label: "Vibrance",
                value: $viewModel.vibrance,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.vibrance) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            SliderRow(
                label: "Saturation",
                value: $viewModel.saturation,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.saturation) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            SliderRow(
                label: "Deep Blue",
                value: $viewModel.deepBlue,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.deepBlue) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            SliderRow(
                label: "Foliage",
                value: $viewModel.foliage,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.foliage) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            Divider()
            
            VStack(alignment: .leading, spacing: 4) {
                Text("Tone Profile")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Picker("", selection: $viewModel.toneProfile) {
                    Text("AgX Vivid (Filmic Punchy)").tag(ColorProfileMode.agxVivid)
                    Text("AgX Neutral (Filmic Base)").tag(ColorProfileMode.agxFilmic)
                    Text("Camera Standard").tag(ColorProfileMode.cameraStandard)
                    Text("Camera Portrait").tag(ColorProfileMode.cameraPortrait)
                    Text("Camera Landscape").tag(ColorProfileMode.cameraLandscape)
                    Text("Camera Vivid").tag(ColorProfileMode.cameraVivid)
                }
                .labelsHidden()
                .pickerStyle(.menu)
                .onChange(of: viewModel.toneProfile) {
                    viewModel.saveCurrentEdits()
                }
            }
            
            Divider()
            
            VStack(alignment: .leading, spacing: 4) {
                Text("3D LUT Look")
                    .font(.caption)
                    .foregroundColor(.secondary)
                
                HStack(spacing: 8) {
                    Picker("", selection: Binding<String>(
                        get: { viewModel.selectedLutName ?? "" },
                        set: { newValue in
                            viewModel.selectedLutName = newValue.isEmpty ? nil : newValue
                            viewModel.saveCurrentEdits()
                        }
                    )) {
                        Text("None (Bypass)").tag("")
                        Divider()
                        ForEach(viewModel.allLutTitles, id: \.self) { title in
                            Text(title).tag(title)
                        }
                    }
                    .labelsHidden()
                    .pickerStyle(.menu)
                    
                    Button(action: {
                        viewModel.importCustomLut()
                    }) {
                        Image(systemName: "plus.circle")
                            .help("Import .cube LUT file...")
                    }
                    .buttonStyle(.borderless)
                }
            }
            
            if viewModel.selectedLutName != nil {
                SliderRow(
                    label: "Intensity",
                    value: $viewModel.lutIntensity,
                    range: 0.0...1.0,
                    format: "%.0f%%",
                    onReset: { viewModel.resetSlider(.lutIntensity) },
                    onEditingChanged: { isEditing in
                        if !isEditing { viewModel.saveCurrentEdits() }
                    },
                    multiplier: 100.0
                )
            }
        }
        .padding(.top, 4)
    }
}
