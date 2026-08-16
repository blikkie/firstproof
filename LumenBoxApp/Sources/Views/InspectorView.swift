import SwiftUI
import AppKit

struct InspectorView: View {
    @Bindable var viewModel: AppViewModel
    
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                Text("ADJUSTMENTS")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                    .padding(.top, 12)
                
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
                
                // MARK: - EXIF & Metadata Readout
                if let photo = viewModel.selectedPhoto {
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
                        
                        // Google Takeout JSON sidecar status
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
                    }
                }
                
                Spacer()
            }
            .padding(16)
        }
        .frame(minWidth: 240, idealWidth: 260, maxWidth: 300)
        .background(Color(NSColor.controlBackgroundColor))
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
