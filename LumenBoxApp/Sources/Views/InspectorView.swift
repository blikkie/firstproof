import SwiftUI
import AppKit
import LumenBoxBridge

struct InspectorView: View {
    @Bindable var viewModel: AppViewModel
    
    private var isBulkSelection: Bool {
        viewModel.selectedPhotoPaths.count > 1
    }
    
    private var selectedCount: Int {
        viewModel.selectedPhotoPaths.count > 0 ? viewModel.selectedPhotoPaths.count : (viewModel.selectedPhoto != nil ? 1 : 0)
    }
    
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                if isBulkSelection {
                    // MARK: - Multi-Selection Bulk Culling Panel
                    bulkCullingPanel
                } else if let photo = viewModel.selectedPhoto {
                    // MARK: - Single Photo (Detail View & Grid Single-Selection)
                    singlePhotoPanel(for: photo)
                } else {
                    // No photo selected
                    emptySelectionPanel
                }
                
                Spacer()
            }
            .padding(14)
        }
        .frame(minWidth: 240, idealWidth: 260, maxWidth: 320)
        .background(Color(NSColor.controlBackgroundColor))
    }
    
    // MARK: - Empty State
    private var emptySelectionPanel: some View {
        VStack(spacing: 12) {
            Image(systemName: "photo.on.rectangle.angled")
                .font(.system(size: 36))
                .foregroundColor(.secondary.opacity(0.6))
                .padding(.top, 40)
            Text("No Photo Selected")
                .font(.subheadline)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity)
    }
    
    // MARK: - Bulk Selection Panel
    @ViewBuilder
    private var bulkCullingPanel: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("BULK SELECTION")
                .font(.caption)
                .fontWeight(.bold)
                .foregroundColor(.secondary)
                .padding(.top, 4)
            
            HStack(spacing: 8) {
                Image(systemName: "square.grid.3x3.fill")
                    .foregroundColor(.accentColor)
                Text("\(selectedCount) Photos Selected")
                    .font(.subheadline)
                    .fontWeight(.semibold)
            }
            .padding(8)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color.secondary.opacity(0.1))
            .cornerRadius(6)
            
            // Bulk Rating Controls
            ratingControls
            
            // Bulk Flag Controls
            flagControls
            
            Divider()
            
            VStack(alignment: .leading, spacing: 8) {
                Label("Bulk Actions", systemImage: "info.circle.fill")
                    .font(.subheadline)
                    .foregroundColor(.accentColor)
                
                Text("Ratings and flags apply to all \(selectedCount) selected photos across SQLite and XMP sidecars. Use the Export button in the toolbar to package.")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
            .padding(10)
            .background(Color.accentColor.opacity(0.08))
            .cornerRadius(8)
        }
    }
    
    // MARK: - Single Photo Panel (Context-Aware: Metadata in Grid, Workspace Tabs in Detail)
    @ViewBuilder
    private func singlePhotoPanel(for photo: PhotoRecord) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            // 1. Permanent Filename & Header Card (Always Visible)
            permanentHeaderCard(for: photo)
            
            if viewModel.canvasMode == .detail {
                // In Detail Mode: Full Development Workspace Tabs & Reset All Button
                HStack(spacing: 8) {
                    Picker("Workspace Tab", selection: $viewModel.selectedInspectorTab) {
                        ForEach(InspectorTab.allCases) { tab in
                            Image(systemName: tab.iconName)
                                .help(tab.rawValue)
                                .tag(tab)
                        }
                    }
                    .pickerStyle(.segmented)
                    
                    Button(action: {
                        viewModel.resetAllSliders()
                    }) {
                        Image(systemName: "arrow.counterclockwise")
                            .font(.system(size: 11, weight: .semibold))
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.small)
                    .disabled(!viewModel.hasCustomEdits)
                    .help("Reset All Edits to As-Shot (Cmd+Shift+R)")
                }
                
                Divider()
                
                // Active Workspace Tab View
                switch viewModel.selectedInspectorTab {
                case .light:
                    lightWorkspaceTab
                case .color:
                    colorWorkspaceTab
                case .optics:
                    opticsWorkspaceTab
                case .detail:
                    detailWorkspaceTab
                case .metadata:
                    metadataWorkspaceTab(for: photo)
                }
            } else {
                // In Grid Mode: Metadata editing only (Rating, Flags, EXIF, Takeout)
                metadataWorkspaceTab(for: photo)
                
                Divider()
                
                Button(action: {
                    viewModel.enterDetailMode()
                }) {
                    Label("Develop in Detail View (Return)", systemImage: "slider.horizontal.3")
                        .font(.caption)
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
                .controlSize(.regular)
                .padding(.top, 2)
            }
        }
    }
    
    // MARK: - Permanent Filename Header Card
    private func permanentHeaderCard(for photo: PhotoRecord) -> some View {
        let filename = URL(fileURLWithPath: photo.filePath).lastPathComponent
        let ext = URL(fileURLWithPath: photo.filePath).pathExtension.uppercased()
        
        return VStack(alignment: .leading, spacing: 6) {
            Text(filename)
                .font(.headline)
                .fontWeight(.bold)
                .lineLimit(1)
                .truncationMode(.middle)
                .allowsTightening(true)
                .help(photo.filePath)
                .contextMenu {
                    Button("Copy Filename") {
                        NSPasteboard.general.clearContents()
                        NSPasteboard.general.setString(filename, forType: .string)
                    }
                    Button("Copy Full Path") {
                        NSPasteboard.general.clearContents()
                        NSPasteboard.general.setString(photo.filePath, forType: .string)
                    }
                    Button("Reveal in Finder") {
                        NSWorkspace.shared.selectFile(photo.filePath, inFileViewerRootedAtPath: "")
                    }
                }
            
            HStack(spacing: 6) {
                Text(ext)
                    .font(.system(size: 9, weight: .bold))
                    .padding(.horizontal, 5)
                    .padding(.vertical, 2)
                    .background(Color.accentColor.opacity(0.18))
                    .foregroundColor(.accentColor)
                    .cornerRadius(3)
                
                let cameraMake = photo.cameraMake ?? ""
                let cameraModel = photo.cameraModel ?? ""
                let fullCamera = "\(cameraMake) \(cameraModel)".trimmingCharacters(in: .whitespaces)
                if !fullCamera.isEmpty {
                    Text(fullCamera)
                        .font(.caption2)
                        .foregroundColor(.secondary)
                        .lineLimit(1)
                }
            }
        }
        .padding(10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.secondary.opacity(0.08))
        .cornerRadius(8)
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color.secondary.opacity(0.15), lineWidth: 1)
        )
    }
    
    // MARK: - Rating & Flag Controls
    private var ratingControls: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Star Rating:")
                .font(.caption)
                .foregroundColor(.secondary)
            
            HStack(spacing: 6) {
                ForEach(1...5, id: \.self) { star in
                    let isSelected: Bool = {
                        if let r = viewModel.selectedPhoto?.rating {
                            return r >= Int32(star)
                        }
                        return false
                    }()
                    Button(action: {
                        viewModel.setRating(Int32(star))
                    }) {
                        Image(systemName: isSelected ? "star.fill" : "star")
                            .font(.subheadline)
                            .foregroundColor(isSelected ? .yellow : .secondary)
                    }
                    .buttonStyle(.bordered)
                    .help("Rate \(star) Star\(star > 1 ? "s" : "") (Key \(star))")
                }
                
                Button(action: {
                    viewModel.setRating(nil)
                }) {
                    Image(systemName: "xmark")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
                .help("Clear Rating (Key 0)")
            }
        }
    }
    
    private var flagControls: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Flag:")
                .font(.caption)
                .foregroundColor(.secondary)
            
            HStack(spacing: 8) {
                Button(action: { viewModel.setStatus("pick") }) {
                    Label("Pick (P)", systemImage: "flag.fill")
                        .font(.caption)
                        .foregroundColor(.green)
                }
                .buttonStyle(.bordered)
                .help("Mark as Pick (P)")
                
                Button(action: { viewModel.setStatus("reject") }) {
                    Label("Reject (X)", systemImage: "xmark.circle.fill")
                        .font(.caption)
                        .foregroundColor(.red)
                }
                .buttonStyle(.bordered)
                .help("Mark as Reject (X)")
                
                Button(action: { viewModel.setStatus("none") }) {
                    Text("Clear (U)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
                .help("Unflag (U)")
            }
        }
    }
    
    // MARK: - Workspace Tab 1: ☀️ Light
    private var lightWorkspaceTab: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                Text("LIGHT & TONE")
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
                label: "Exposure",
                value: $viewModel.exposure,
                range: -5.0...5.0,
                format: "%.2f EV",
                onReset: { viewModel.resetSlider(.exposure) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            SliderRow(
                label: "Contrast",
                value: $viewModel.contrast,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.contrast) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            Divider()
                .padding(.vertical, 2)
            
            Text("TONAL RECOVERY")
                .font(.system(size: 10, weight: .bold))
                .foregroundColor(.secondary)
            
            SliderRow(
                label: "Highlights",
                value: $viewModel.highlights,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.highlights) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            SliderRow(
                label: "Shadows",
                value: $viewModel.shadows,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.shadows) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            SliderRow(
                label: "Whites",
                value: $viewModel.whites,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.whites) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
            
            SliderRow(
                label: "Blacks",
                value: $viewModel.blacks,
                range: -100.0...100.0,
                format: "%.0f",
                onReset: { viewModel.resetSlider(.blacks) },
                onEditingChanged: { isEditing in
                    if !isEditing { viewModel.saveCurrentEdits() }
                }
            )
        }
        .padding(.top, 4)
    }
    
    // MARK: - Workspace Tab 2: 🎨 Color
    private var colorWorkspaceTab: some View {
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
    
    // MARK: - Workspace Tab 3: 📐 Optics & Geometry
    private var opticsWorkspaceTab: some View {
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
    
    // MARK: - Workspace Tab 4: 🔎 Detail
    private var detailWorkspaceTab: some View {
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
    
    // MARK: - Workspace Tab 5: ℹ️ Metadata
    @ViewBuilder
    private func metadataWorkspaceTab(for photo: PhotoRecord) -> some View {
        VStack(alignment: .leading, spacing: 14) {
            ratingControls
            flagControls
            Divider()
            metadataRows(for: photo)
        }
        .padding(.top, 4)
    }
    
    // MARK: - Metadata Rows View
    @ViewBuilder
    private func metadataRows(for photo: PhotoRecord) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            if let make = photo.cameraMake, let model = photo.cameraModel, !make.isEmpty || !model.isEmpty {
                MetadataRow(label: "Camera", value: "\(make) \(model)".trimmingCharacters(in: .whitespaces))
            }
            if let lens = photo.lensModel, !lens.isEmpty {
                MetadataRow(label: "Lens", value: lens)
            }
            if let iso = photo.iso {
                MetadataRow(label: "ISO", value: "\(iso)")
            }
            if let aperture = photo.aperture {
                MetadataRow(label: "Aperture", value: String(format: "f/%.1f", aperture))
            }
            if let shutter = photo.shutterSpeed, !shutter.isEmpty {
                MetadataRow(label: "Shutter", value: shutter)
            }
            if let focal = photo.focalLength {
                MetadataRow(label: "Focal Length", value: String(format: "%.0f mm", focal))
            }
            if let taken = photo.takenTime, taken > 0 {
                MetadataRow(label: "Date", value: formatDate(taken))
            }
            if let lat = photo.latitude, let lon = photo.longitude {
                MetadataRow(label: "GPS", value: String(format: "%.4f, %.4f", lat, lon))
            }
            
            let hasTakeout = viewModel.takeoutMatches.contains(where: { $0.imagePath == photo.filePath })
            HStack {
                Text("Google Takeout:")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                if hasTakeout {
                    Label("JSON Matched", systemImage: "checkmark.circle.fill")
                        .font(.caption)
                        .foregroundColor(.green)
                } else {
                    Text("None")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
            .padding(.top, 2)
        }
    }
    
    private func formatDate(_ timestamp: Int64) -> String {
        let date = Date(timeIntervalSince1970: TimeInterval(timestamp))
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}

struct SliderRow: View {
    let label: String
    @Binding var value: Float
    let range: ClosedRange<Float>
    let format: String
    let onReset: () -> Void
    var onEditingChanged: ((Bool) -> Void)? = nil
    var multiplier: Float = 1.0
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(label)
                    .font(.subheadline)
                    .onTapGesture(count: 2, perform: onReset) // Double click label to reset!
                Spacer()
                Text(String(format: format, value * multiplier))
                    .font(.caption)
                    .monospacedDigit()
                    .foregroundColor(.secondary)
            }
            
            Slider(value: $value, in: range) { isEditing in
                onEditingChanged?(isEditing)
            }
        }
    }
}

struct TemperatureSliderRow: View {
    let label: String
    @Binding var kelvin: Float
    let onReset: () -> Void
    var onEditingChanged: ((Bool) -> Void)? = nil
    
    // Perceptual Mired mapping:
    // Human color perception of blackbody temperature is linear in Mireds (10^6 / K), not linear Kelvin.
    // 2000 K  -> 500 Mireds (pos = 0.0)
    // 5500 K  -> 181.8 Mireds (pos = 0.663)
    // 50000 K -> 20 Mireds (pos = 1.0)
    // This gives generous precision across 2,500K - 8,500K where 99% of photo adjustments occur.
    private var sliderPosition: Binding<Float> {
        Binding<Float>(
            get: {
                let k = min(max(kelvin, 2000.0), 50000.0)
                let mired = 1_000_000.0 / k
                let pos = (500.0 - mired) / 480.0
                return min(max(pos, 0.0), 1.0)
            },
            set: { newPos in
                let pos = min(max(newPos, 0.0), 1.0)
                let mired = 500.0 - pos * 480.0
                let rawK = 1_000_000.0 / mired
                // Stepped in 50K increments below 10,000K, 200K increments above
                let steppedK = (rawK < 10000.0)
                    ? (rawK / 50.0).rounded() * 50.0
                    : (rawK / 200.0).rounded() * 200.0
                kelvin = min(max(steppedK, 2000.0), 50000.0)
            }
        )
    }
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(label)
                    .font(.subheadline)
                    .onTapGesture(count: 2, perform: onReset)
                Spacer()
                Text(String(format: "%.0f K", kelvin))
                    .font(.caption)
                    .monospacedDigit()
                    .foregroundColor(.secondary)
            }
            
            Slider(value: sliderPosition, in: 0.0...1.0) { isEditing in
                onEditingChanged?(isEditing)
            }
        }
    }
}

struct MetadataRow: View {
    let label: String
    let value: String
    
    var body: some View {
        HStack {
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
            Spacer()
            Text(value)
                .font(.caption)
                .fontWeight(.medium)
                .lineLimit(1)
        }
    }
}
