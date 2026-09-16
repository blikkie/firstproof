import SwiftUI
import FirstProofBridge

public struct ColorWorkspaceView: View {
    @Environment(AppViewModel.self) private var viewModel
    @State private var isSplitToningExpanded: Bool = false
    
    public init() {}
    
    public var body: some View {
        @Bindable var viewModel = viewModel
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                Text("WHITE BALANCE")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                Spacer()
                
                Menu {
                    ForEach(WhiteBalancePreset.allCases) { preset in
                        if preset != .custom {
                            Button(action: {
                                viewModel.applyWhiteBalancePreset(preset)
                            }) {
                                HStack {
                                    Text(preset.displayName)
                                    if viewModel.activeWhiteBalancePreset == preset {
                                        Image(systemName: "checkmark")
                                    }
                                }
                            }
                        }
                    }
                    if viewModel.activeWhiteBalancePreset == .custom {
                        Divider()
                        Text("Custom")
                    }
                } label: {
                    HStack(spacing: 4) {
                        Text(viewModel.activeWhiteBalancePreset.displayName)
                            .font(.caption)
                            .foregroundColor(.primary)
                        Image(systemName: "chevron.up.chevron.down")
                            .font(.system(size: 9))
                            .foregroundColor(.secondary)
                    }
                    .padding(.horizontal, 6)
                    .padding(.vertical, 3)
                    .background(
                        RoundedRectangle(cornerRadius: 4)
                            .fill(Color(nsColor: .controlBackgroundColor))
                    )
                }
                .menuStyle(.borderlessButton)
                .fixedSize()
                
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
                range: -200.0...200.0,
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
            
            DisclosureGroup(isExpanded: $isSplitToningExpanded) {
                VStack(spacing: 12) {
                    HStack {
                        Text("Chemical Preset")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Spacer()
                        
                        Menu {
                            Button("Custom / Off") {
                                viewModel.applySplitToningPreset(nil)
                            }
                            Divider()
                            ForEach(SplitToningPreset.builtInPresets) { preset in
                                Button(action: {
                                    viewModel.applySplitToningPreset(preset)
                                }) {
                                    HStack {
                                        Text(preset.name)
                                        if viewModel.selectedSplitToningPresetId == preset.id {
                                            Image(systemName: "checkmark")
                                        }
                                    }
                                }
                            }
                        } label: {
                            HStack(spacing: 4) {
                                let activeName = SplitToningPreset.builtInPresets.first(where: { $0.id == viewModel.selectedSplitToningPresetId })?.name ?? (viewModel.splitToningShadowSat > 0 || viewModel.splitToningHighlightSat > 0 ? "Custom" : "Off")
                                Text(activeName)
                                    .font(.caption)
                                    .foregroundColor(.primary)
                                Image(systemName: "chevron.up.chevron.down")
                                    .font(.system(size: 9))
                                    .foregroundColor(.secondary)
                            }
                            .padding(.horizontal, 6)
                            .padding(.vertical, 3)
                            .background(
                                RoundedRectangle(cornerRadius: 4)
                                    .fill(Color(nsColor: .controlBackgroundColor))
                            )
                        }
                        .menuStyle(.borderlessButton)
                        .fixedSize()
                    }
                    
                    // Dual 2D Color Wheels: Shadows and Highlights side-by-side
                    HStack(spacing: 16) {
                        ColorWheelView(
                            title: "Shadows",
                            hue: $viewModel.splitToningShadowHue,
                            saturation: $viewModel.splitToningShadowSat,
                            onReset: { viewModel.resetSlider(.splitToningShadowSat) },
                            onEditingChanged: { viewModel.handleSliderEditing($0, slider: .splitToningShadowSat) }
                        )
                        
                        ColorWheelView(
                            title: "Highlights",
                            hue: $viewModel.splitToningHighlightHue,
                            saturation: $viewModel.splitToningHighlightSat,
                            onReset: { viewModel.resetSlider(.splitToningHighlightSat) },
                            onEditingChanged: { viewModel.handleSliderEditing($0, slider: .splitToningHighlightSat) }
                        )
                    }
                    .padding(.vertical, 2)
                    
                    SliderRow(
                        label: "Balance",
                        value: $viewModel.splitToningBalance,
                        range: -100.0...100.0,
                        format: "%+.0f",
                        onReset: { viewModel.resetSlider(.splitToningBalance) },
                        onEditingChanged: { viewModel.handleSliderEditing($0, slider: .splitToningBalance) }
                    )
                }
                .padding(.top, 6)
            } label: {
                HStack {
                    Text("SPLIT TONING")
                        .font(.caption)
                        .fontWeight(.bold)
                        .foregroundColor(.secondary)
                    Spacer()
                    if viewModel.splitToningShadowSat > 0 || viewModel.splitToningHighlightSat > 0 {
                        Text("Active")
                            .font(.caption2)
                            .foregroundColor(.accentColor)
                            .padding(.horizontal, 4)
                            .padding(.vertical, 1)
                            .background(Color.accentColor.opacity(0.15))
                            .clipShape(RoundedRectangle(cornerRadius: 3))
                    }
                }
            }
            
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
