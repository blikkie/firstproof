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
            Text("WHITE BALANCE & COLOR")
                .font(.caption)
                .fontWeight(.bold)
                .foregroundColor(.secondary)
            
            SliderRow(
                label: "Temperature",
                value: $viewModel.temperature,
                range: 2000.0...50000.0,
                format: "%.0f K",
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
            Text("GEOMETRY & OPTICS")
                .font(.caption)
                .fontWeight(.bold)
                .foregroundColor(.secondary)
            
            // 90-degree rotate and flip controls
            VStack(spacing: 6) {
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
            }
            .buttonStyle(.bordered)
            .controlSize(.small)
            
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
            
            Divider()
            
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
        .padding(.top, 4)
    }
    
    // MARK: - Workspace Tab 4: 🔎 Detail
    private var detailWorkspaceTab: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("DETAIL & SHARPENING")
                .font(.caption)
                .fontWeight(.bold)
                .foregroundColor(.secondary)
            
            VStack(alignment: .leading, spacing: 8) {
                HStack(spacing: 6) {
                    Image(systemName: "viewfinder")
                        .foregroundColor(.accentColor)
                    Text("1:1 Pixel Inspection")
                        .font(.subheadline)
                        .fontWeight(.semibold)
                }
                Text("Evaluating critical focus, optical sharpness, and sensor noise requires inspecting pixels at 100% (1:1) scale.")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
            .padding(10)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color.accentColor.opacity(0.08))
            .cornerRadius(8)
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
