import SwiftUI
import FirstProofBridge

public struct DevelopWorkspaceView: View {
    @Environment(AppViewModel.self) private var viewModel
    @State private var isSplitToningExpanded: Bool = false
    
    public init() {}
    
    public var body: some View {
        @Bindable var viewModel = viewModel
        VStack(alignment: .leading, spacing: 14) {
            // MARK: - Look / 3D LUT
            VStack(alignment: .leading, spacing: 6) {
                Text("LOOK / 3D LUT")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                
                HStack(spacing: 6) {
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
                    
                    if viewModel.selectedLutName != nil {
                        Slider(
                            value: $viewModel.lutIntensity,
                            in: 0.0...1.0,
                            onEditingChanged: { editing in
                                viewModel.handleSliderEditing(editing, slider: .lutIntensity)
                            }
                        )
                        .frame(width: 65)
                        
                        Text("\(Int(round(viewModel.lutIntensity * 100)))%")
                            .font(.system(.caption, design: .monospaced))
                            .foregroundColor(.secondary)
                            .frame(width: 34, alignment: .trailing)
                            .help("Double-click to reset strength to 100%")
                            .onTapGesture(count: 2) {
                                viewModel.resetSlider(.lutIntensity)
                            }
                    }
                    
                    Button(action: {
                        viewModel.importCustomLut()
                    }) {
                        Image(systemName: "plus.circle")
                            .help("Import .cube LUT into library...")
                    }
                    .buttonStyle(.borderless)
                }
            }
            
            Divider()
                .padding(.vertical, 2)
            
            // MARK: - White Balance
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
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .temperature) },
                onCancel: { viewModel.cancelSliderScrub() }
            )
            
            SliderRow(
                slider: .tint,
                value: $viewModel.tint,
                onReset: { viewModel.resetSlider(.tint) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .tint) },
                onCancel: { viewModel.cancelSliderScrub() }
            )
            
            Divider()
                .padding(.vertical, 2)
            
            // MARK: - Tone & Exposure
            HStack {
                Text("TONE & EXPOSURE")
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
            
            // MARK: - Tone Curve
            ToneCurveView()
            
            Divider()
                .padding(.vertical, 2)
            
            // MARK: - Color & Vibrance
            Text("COLOR & VIBRANCE")
                .font(.caption)
                .fontWeight(.bold)
                .foregroundColor(.secondary)
            
            SliderRow(
                slider: .vibrance,
                value: $viewModel.vibrance,
                onReset: { viewModel.resetSlider(.vibrance) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .vibrance) },
                onCancel: { viewModel.cancelSliderScrub() }
            )
            
            SliderRow(
                slider: .saturation,
                value: $viewModel.saturation,
                onReset: { viewModel.resetSlider(.saturation) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .saturation) },
                onCancel: { viewModel.cancelSliderScrub() }
            )
            
            SliderRow(
                slider: .deepBlue,
                value: $viewModel.deepBlue,
                onReset: { viewModel.resetSlider(.deepBlue) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .deepBlue) },
                onCancel: { viewModel.cancelSliderScrub() }
            )
            
            SliderRow(
                slider: .foliage,
                value: $viewModel.foliage,
                onReset: { viewModel.resetSlider(.foliage) },
                onEditingChanged: { viewModel.handleSliderEditing($0, slider: .foliage) },
                onCancel: { viewModel.cancelSliderScrub() }
            )
            
            Divider()
                .padding(.vertical, 2)
            
            // MARK: - Split Toning
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
                    
                    HStack(spacing: 16) {
                        ColorWheelView(
                            title: "Shadows",
                            hue: $viewModel.splitToningShadowHue,
                            saturation: $viewModel.splitToningShadowSat,
                            onReset: { viewModel.resetSplitToningShadows() },
                            onEditingChanged: { viewModel.handleSliderEditing($0, slider: .splitToningShadowSat) }
                        )
                        
                        ColorWheelView(
                            title: "Highlights",
                            hue: $viewModel.splitToningHighlightHue,
                            saturation: $viewModel.splitToningHighlightSat,
                            onReset: { viewModel.resetSplitToningHighlights() },
                            onEditingChanged: { viewModel.handleSliderEditing($0, slider: .splitToningHighlightSat) }
                        )
                    }
                    .padding(.vertical, 2)
                    
                    SliderRow(
                        slider: .splitToningBalance,
                        value: $viewModel.splitToningBalance,
                        onReset: { viewModel.resetSlider(.splitToningBalance) },
                        onEditingChanged: { viewModel.handleSliderEditing($0, slider: .splitToningBalance) },
                        onCancel: { viewModel.cancelSliderScrub() },
                        customLabel: "Balance"
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
        }
        .padding(.top, 4)
    }
}
