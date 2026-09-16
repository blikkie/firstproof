import SwiftUI

public struct DetailWorkspaceView: View {
    @Environment(AppViewModel.self) private var viewModel
    @State private var isGrainExpanded: Bool = false
    
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
            
            Divider()
            
            // Section 3: Procedural Organic Film Grain (Dormant by default)
            DisclosureGroup(isExpanded: $isGrainExpanded) {
                VStack(alignment: .leading, spacing: 10) {
                    HStack {
                        Spacer()
                        if viewModel.grainAmount > 0.0 {
                            Button("Reset") {
                                viewModel.applyGrainPreset(nil)
                            }
                            .font(.caption2)
                            .buttonStyle(.borderless)
                            .foregroundColor(.secondary)
                        }
                    }
                    
                    // Film Stock Presets Menu
                    Menu {
                        Button("Custom / Off") {
                            viewModel.selectedGrainPresetId = nil
                        }
                        Divider()
                        ForEach(FilmGrainPreset.builtInPresets) { preset in
                            Button {
                                viewModel.applyGrainPreset(preset)
                            } label: {
                                HStack {
                                    Text(preset.name)
                                    if viewModel.selectedGrainPresetId == preset.id {
                                        Image(systemName: "checkmark")
                                    }
                                }
                            }
                        }
                    } label: {
                        HStack {
                            Image(systemName: "film")
                                .foregroundColor(.accentColor)
                                .font(.caption)
                            if let activeId = viewModel.selectedGrainPresetId,
                               let activePreset = FilmGrainPreset.builtInPresets.first(where: { $0.id == activeId }) {
                                Text(activePreset.name)
                                    .font(.caption)
                                    .fontWeight(.medium)
                            } else if viewModel.grainAmount > 0.0 {
                                Text("Custom Grain")
                                    .font(.caption)
                                    .fontWeight(.medium)
                            } else {
                                Text("No Film Grain")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            Spacer()
                        }
                        .padding(.horizontal, 8)
                        .padding(.vertical, 5)
                        .background(Color(NSColor.controlBackgroundColor))
                        .cornerRadius(5)
                    }
                    .menuStyle(.borderlessButton)
                    
                    if let activeId = viewModel.selectedGrainPresetId,
                       let activePreset = FilmGrainPreset.builtInPresets.first(where: { $0.id == activeId }) {
                        Text(activePreset.subtitle)
                            .font(.system(size: 10))
                            .foregroundColor(.secondary)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                    
                    SliderRow(
                        label: "Amount",
                        value: $viewModel.grainAmount,
                        range: 0.0...100.0,
                        format: "%.0f",
                        onReset: { 
                            viewModel.resetSlider(.grainAmount)
                        },
                        onEditingChanged: { isEditing in
                            if isEditing { viewModel.selectedGrainPresetId = nil }
                            viewModel.handleSliderEditing(isEditing, slider: .grainAmount)
                        }
                    )
                    
                    SliderRow(
                        label: "Size",
                        value: $viewModel.grainSize,
                        range: 0.5...3.0,
                        format: "%.1f",
                        onReset: { 
                            viewModel.resetSlider(.grainSize)
                        },
                        onEditingChanged: { isEditing in
                            if isEditing { viewModel.selectedGrainPresetId = nil }
                            viewModel.handleSliderEditing(isEditing, slider: .grainSize)
                        }
                    )
                    
                    SliderRow(
                        label: "Roughness",
                        value: $viewModel.grainRoughness,
                        range: 0.0...100.0,
                        format: "%.0f",
                        onReset: { 
                            viewModel.resetSlider(.grainRoughness)
                        },
                        onEditingChanged: { isEditing in
                            if isEditing { viewModel.selectedGrainPresetId = nil }
                            viewModel.handleSliderEditing(isEditing, slider: .grainRoughness)
                        }
                    )
                    
                    SliderRow(
                        label: "Color Variation",
                        value: $viewModel.grainChroma,
                        range: 0.0...100.0,
                        format: "%.0f",
                        onReset: { 
                            viewModel.resetSlider(.grainChroma)
                        },
                        onEditingChanged: { isEditing in
                            if isEditing { viewModel.selectedGrainPresetId = nil }
                            viewModel.handleSliderEditing(isEditing, slider: .grainChroma)
                        }
                    )
                    
                    // Helpful hint box
                    VStack(alignment: .leading, spacing: 6) {
                        HStack(spacing: 5) {
                            Image(systemName: "info.circle")
                                .foregroundColor(.secondary)
                                .font(.caption2)
                            Text("Silver Halide Emulsion Physics")
                                .font(.caption2)
                                .fontWeight(.semibold)
                                .foregroundColor(.secondary)
                        }
                        Text("Film grain peaks naturally in Zone V midtones and washes out in specular highlights. Double-click any slider label to reset to default.")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                    .padding(8)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color.secondary.opacity(0.06))
                    .cornerRadius(6)
                }
                .padding(.top, 4)
            } label: {
                HStack {
                    Text("ORGANIC FILM GRAIN")
                        .font(.system(size: 10, weight: .bold))
                        .foregroundColor(.secondary)
                    Spacer()
                    if viewModel.grainAmount > 0.0 {
                        Text("On")
                            .font(.system(size: 10, weight: .bold))
                            .foregroundColor(.accentColor)
                    } else {
                        Text("Off")
                            .font(.system(size: 10))
                            .foregroundColor(.secondary)
                    }
                }
            }
        }
        .padding(.top, 4)
        .onAppear {
            isGrainExpanded = viewModel.grainAmount > 0.0
        }
        .onChange(of: viewModel.selectedPhoto?.filePath) { _, _ in
            isGrainExpanded = viewModel.grainAmount > 0.0
        }
        .onChange(of: viewModel.grainAmount) { _, newAmount in
            if newAmount > 0.0 && !isGrainExpanded {
                withAnimation(.easeInOut(duration: 0.2)) {
                    isGrainExpanded = true
                }
            }
        }
    }
}
