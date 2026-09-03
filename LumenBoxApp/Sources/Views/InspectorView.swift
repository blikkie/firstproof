import SwiftUI
import AppKit
import LumenBoxBridge

struct InspectorView: View {
    @Bindable var viewModel: AppViewModel
    
    private var isBulkOrGrid: Bool {
        viewModel.canvasMode == .grid || viewModel.selectedPhotoPaths.count > 1
    }
    
    private var selectedCount: Int {
        viewModel.selectedPhotoPaths.count > 0 ? viewModel.selectedPhotoPaths.count : (viewModel.selectedPhoto != nil ? 1 : 0)
    }
    
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if isBulkOrGrid {
                    // MARK: - Grid / Multi-Selection Metadata & Culling Panel
                    cullingMetadataPanel
                } else {
                    // MARK: - Detail View RAW Adjustments Panel
                    adjustmentsPanel
                }
                
                Spacer()
            }
            .padding(16)
        }
        .frame(minWidth: 240, idealWidth: 260, maxWidth: 300)
        .background(Color(NSColor.controlBackgroundColor))
    }
    
    // MARK: - Culling & Metadata Panel (Grid / Bulk Selection)
    @ViewBuilder
    private var cullingMetadataPanel: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("CULLING & METADATA")
                .font(.caption)
                .fontWeight(.bold)
                .foregroundColor(.secondary)
                .padding(.top, 4)
            
            // Selection Badge
            HStack(spacing: 8) {
                Image(systemName: selectedCount > 1 ? "square.grid.3x3.fill" : "photo.fill")
                    .foregroundColor(.accentColor)
                Text(selectedCount > 1 ? "\(selectedCount) Photos Selected" : (viewModel.selectedPhoto != nil ? URL(fileURLWithPath: viewModel.selectedPhoto!.filePath).lastPathComponent : "No Photo Selected"))
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }
            .padding(8)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color.secondary.opacity(0.1))
            .cornerRadius(6)
            
            // 1. Bulk / Single Rating Controls
            VStack(alignment: .leading, spacing: 6) {
                Text("Star Rating:")
                    .font(.caption)
                    .foregroundColor(.secondary)
                
                HStack(spacing: 6) {
                    ForEach(1...5, id: \.self) { star in
                        Button(action: {
                            viewModel.setRating(Int32(star))
                        }) {
                            Image(systemName: "star.fill")
                                .font(.subheadline)
                                .foregroundColor(.yellow)
                        }
                        .buttonStyle(.bordered)
                        .help("Rate \(star) Star\(star > 1 ? "s" : "")")
                    }
                    
                    Button(action: {
                        viewModel.setRating(nil)
                    }) {
                        Image(systemName: "xmark")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .buttonStyle(.plain)
                    .help("Clear Rating")
                }
            }
            
            // 2. Bulk / Single Pick & Reject Controls
            VStack(alignment: .leading, spacing: 6) {
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
                    
                    Button(action: { viewModel.setStatus("reject") }) {
                        Label("Reject (X)", systemImage: "xmark.circle.fill")
                            .font(.caption)
                            .foregroundColor(.red)
                    }
                    .buttonStyle(.bordered)
                    
                    Button(action: { viewModel.setStatus("none") }) {
                        Text("Clear")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .buttonStyle(.plain)
                }
            }
            
            Divider()
            
            // 3. Informational Banner or EXIF Details
            if selectedCount > 1 {
                VStack(alignment: .leading, spacing: 8) {
                    Label("Bulk Operations", systemImage: "info.circle.fill")
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
            } else if let photo = viewModel.selectedPhoto {
                VStack(alignment: .leading, spacing: 8) {
                    Label("EXIF Metadata", systemImage: "info.circle.fill")
                        .font(.headline)
                    
                    MetadataRow(label: "File", value: URL(fileURLWithPath: photo.filePath).lastPathComponent)
                    MetadataRow(label: "Camera", value: "\(photo.cameraMake ?? "Unknown") \(photo.cameraModel ?? "")")
                    MetadataRow(label: "Lens", value: photo.lensModel ?? "Unknown")
                    MetadataRow(label: "ISO", value: photo.iso != nil ? "\(photo.iso!)" : "N/A")
                    MetadataRow(label: "Aperture", value: photo.aperture != nil ? "f/\(photo.aperture!)" : "N/A")
                    MetadataRow(label: "Shutter", value: photo.shutterSpeed ?? "N/A")
                    MetadataRow(label: "Focal Length", value: photo.focalLength != nil ? "\(photo.focalLength!)mm" : "N/A")
                    
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
                    .padding(.top, 4)
                    
                    Button(action: {
                        viewModel.canvasMode = .detail
                    }) {
                        Label("Open in Detail View", systemImage: "photo.fill")
                            .font(.caption)
                    }
                    .buttonStyle(.bordered)
                    .padding(.top, 4)
                }
            }
        }
    }
    
    // MARK: - Detail View RAW Adjustments Panel
    @ViewBuilder
    private var adjustmentsPanel: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("ADJUSTMENTS")
                .font(.caption)
                .fontWeight(.bold)
                .foregroundColor(.secondary)
                .padding(.top, 4)
            
            // MARK: - Optics & Geometry
            DisclosureGroup(isExpanded: .constant(true)) {
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
            
            Divider()
            
            // MARK: - Basic Tone & WB
            DisclosureGroup(isExpanded: .constant(true)) {
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
            
            Divider()
            
            // MARK: - Contrast
            DisclosureGroup(isExpanded: .constant(true)) {
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
            
            Divider()
            
            // Quick rating / flag in Detail view
            if let photo = viewModel.selectedPhoto {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Quick Rating & Flag")
                        .font(.subheadline)
                        .fontWeight(.medium)
                    
                    HStack(spacing: 8) {
                        Button(action: { viewModel.setStatus("pick") }) {
                            Image(systemName: "flag.fill")
                                .foregroundColor(.green)
                        }
                        .buttonStyle(.bordered)
                        
                        Button(action: { viewModel.setStatus("reject") }) {
                            Image(systemName: "xmark.circle.fill")
                                .foregroundColor(.red)
                        }
                        .buttonStyle(.bordered)
                        
                        Spacer()
                        
                        ForEach(1...5, id: \.self) { star in
                            Button(action: { viewModel.setRating(Int32(star)) }) {
                                Image(systemName: (photo.rating ?? 0) >= star ? "star.fill" : "star")
                                    .foregroundColor(.yellow)
                                    .font(.caption)
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
            }
        }
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
