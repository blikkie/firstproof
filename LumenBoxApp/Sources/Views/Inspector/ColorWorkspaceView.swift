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
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .temperature) }
            )
            
            SliderRow(
                label: "Tint",
                value: $viewModel.tint,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.tint) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .tint) }
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
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .vibrance) }
            )
            
            SliderRow(
                label: "Saturation",
                value: $viewModel.saturation,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.saturation) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .saturation) }
            )
            
            SliderRow(
                label: "Deep Blue",
                value: $viewModel.deepBlue,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.deepBlue) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .deepBlue) }
            )
            
            SliderRow(
                label: "Foliage",
                value: $viewModel.foliage,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.foliage) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .foliage) }
            )
            
            Divider()
            
            VStack(alignment: .leading, spacing: 4) {
                Text("Tone Profile")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Picker("", selection: Binding<ColorProfileMode>(
                    get: { viewModel.toneProfile },
                    set: { viewModel.setToneProfile($0) }
                )) {
                    Text("AgX Vivid (Filmic Punchy)").tag(ColorProfileMode.agxVivid)
                    Text("AgX Neutral (Filmic Base)").tag(ColorProfileMode.agxFilmic)
                    Text("Camera Standard").tag(ColorProfileMode.cameraStandard)
                    Text("Camera Portrait").tag(ColorProfileMode.cameraPortrait)
                    Text("Camera Landscape").tag(ColorProfileMode.cameraLandscape)
                    Text("Camera Vivid").tag(ColorProfileMode.cameraVivid)
                }
                .labelsHidden()
                .pickerStyle(.menu)
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
                            viewModel.setSelectedLutName(newValue.isEmpty ? nil : newValue)
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
                    onEditingChanged: { viewModel.handleSliderEditing($0, slider: .lutIntensity) },
                    multiplier: 100.0
                )
            }
        }
        .padding(.top, 4)
    }
}
