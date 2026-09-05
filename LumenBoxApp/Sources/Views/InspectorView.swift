import SwiftUI
import AppKit
import LumenBoxBridge

struct InspectorView: View {
    @Bindable var viewModel: AppViewModel
    
    // Persisted app-wide collapse preferences
    @AppStorage("inspector_isMetadataExpanded") private var isMetadataExpanded: Bool = true
    @AppStorage("inspector_isOpticsExpanded") private var isOpticsExpanded: Bool = true
    @AppStorage("inspector_isLightExpanded") private var isLightExpanded: Bool = true
    @AppStorage("inspector_isToneExpanded") private var isToneExpanded: Bool = true
    
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
                
                Text("Ratings and flags apply to all \(selectedCount) selected photos across SQLite and XMP sidecars.")
                    .font(.caption2)
                    .foregroundColor(.secondary)
                
                Button(action: {
                    viewModel.isExportPackageSheetPresented = true
                }) {
                    Label("Export Package (\(selectedCount))...", systemImage: "shippingbox.fill")
                        .font(.caption)
                }
                .buttonStyle(.borderedProminent)
                .padding(.top, 4)
            }
            .padding(10)
            .background(Color.accentColor.opacity(0.08))
            .cornerRadius(8)
        }
    }
    
    // MARK: - Single Photo Panel (Always shows Filename, Quick Controls, Collapsible EXIF, and Adjustments)
    @ViewBuilder
    private func singlePhotoPanel(for photo: PhotoRecord) -> some View {
        VStack(alignment: .leading, spacing: 14) {
            // 1. Permanent Filename & Header Card (Always Visible)
            permanentHeaderCard(for: photo)
            
            // 2. Quick Rating & Flag
            ratingControls
            flagControls
            
            Divider()
            
            // 3. Collapsible EXIF Metadata Section
            collapsibleMetadataSection(for: photo)
            
            // 4. Detail View RAW Adjustments (Only when in Detail Mode)
            if viewModel.canvasMode == .detail {
                Divider()
                
                Text("DEVELOPMENT")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                
                // Optics & Geometry
                collapsibleOpticsSection
                
                Divider()
                
                // Basic Light & Color
                collapsibleLightSection
                
                Divider()
                
                // Tone & Contrast
                collapsibleToneSection
            } else {
                // In Grid view: affordance to enter detail mode
                Button(action: {
                    viewModel.enterDetailMode()
                }) {
                    Label("Open in Detail View (Return)", systemImage: "photo.fill")
                        .font(.caption)
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
                .padding(.top, 6)
            }
        }
    }
    
    // MARK: - Permanent Filename Header Card
    private func permanentHeaderCard(for photo: PhotoRecord) -> some View {
        let filename = URL(fileURLWithPath: photo.filePath).lastPathComponent
        let ext = URL(fileURLWithPath: photo.filePath).pathExtension.uppercased()
        
        return VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .top, spacing: 8) {
                Image(systemName: viewModel.selectedPhotoIsRaw ? "camera.metering.matrix" : "photo.fill")
                    .foregroundColor(.accentColor)
                    .font(.title3)
                    .padding(.top, 2)
                
                VStack(alignment: .leading, spacing: 3) {
                    Text(filename)
                        .font(.headline)
                        .fontWeight(.bold)
                        .lineLimit(2)
                        .truncationMode(.middle)
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
                
                Spacer()
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
    
    // MARK: - Collapsible EXIF Metadata Section
    private func collapsibleMetadataSection(for photo: PhotoRecord) -> some View {
        DisclosureGroup(isExpanded: $isMetadataExpanded) {
            VStack(alignment: .leading, spacing: 6) {
                let cameraMake = photo.cameraMake ?? ""
                let cameraModel = photo.cameraModel ?? ""
                let fullCamera = "\(cameraMake) \(cameraModel)".trimmingCharacters(in: .whitespaces)
                if !fullCamera.isEmpty {
                    MetadataRow(label: "Camera", value: fullCamera)
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
            .padding(.top, 6)
        } label: {
            Label("EXIF Metadata", systemImage: "info.circle.fill")
                .font(.headline)
        }
    }
    
    // MARK: - Collapsible Optics & Geometry
    private var collapsibleOpticsSection: some View {
        DisclosureGroup(isExpanded: $isOpticsExpanded) {
            VStack(alignment: .leading, spacing: 12) {
                Toggle("Lens Correction (Lensfun)", isOn: $viewModel.lensCorrectionEnabled)
                    .toggleStyle(.switch)
                
                SliderRow(
                    label: "Straighten",
                    value: $viewModel.straightenAngle,
                    range: -45.0...45.0,
                    format: "%.1f°",
                    onReset: { viewModel.resetSlider(.straighten) }
                )
            }
            .padding(.top, 8)
        } label: {
            Label("Optics & Geometry", systemImage: "crop.rotate")
                .font(.headline)
        }
    }
    
    // MARK: - Collapsible Basic Light & Color
    private var collapsibleLightSection: some View {
        DisclosureGroup(isExpanded: $isLightExpanded) {
            VStack(alignment: .leading, spacing: 12) {
                SliderRow(
                    label: "Exposure",
                    value: $viewModel.exposure,
                    range: -5.0...5.0,
                    format: "%.2f EV",
                    onReset: { viewModel.resetSlider(.exposure) }
                )
                
                SliderRow(
                    label: "Temperature",
                    value: $viewModel.temperature,
                    range: 2000.0...50000.0,
                    format: "%.0f K",
                    onReset: { viewModel.resetSlider(.temperature) }
                )
                
                SliderRow(
                    label: "Tint",
                    value: $viewModel.tint,
                    range: -150.0...150.0,
                    format: "%.0f",
                    onReset: { viewModel.resetSlider(.tint) }
                )
            }
            .padding(.top, 8)
        } label: {
            Label("Basic Light & Color", systemImage: "sun.max.fill")
                .font(.headline)
        }
    }
    
    // MARK: - Collapsible Tone & Contrast
    private var collapsibleToneSection: some View {
        DisclosureGroup(isExpanded: $isToneExpanded) {
            VStack(alignment: .leading, spacing: 12) {
                SliderRow(
                    label: "Contrast",
                    value: $viewModel.contrast,
                    range: -100.0...100.0,
                    format: "%.0f",
                    onReset: { viewModel.resetSlider(.contrast) }
                )
            }
            .padding(.top, 8)
        } label: {
            Label("Tone & Contrast", systemImage: "slider.horizontal.3")
                .font(.headline)
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
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(label)
                    .font(.subheadline)
                    .onTapGesture(count: 2, perform: onReset) // Double click label to reset!
                Spacer()
                Text(String(format: format, value))
                    .font(.caption)
                    .monospacedDigit()
                    .foregroundColor(.secondary)
            }
            
            Slider(value: $value, in: range)
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
