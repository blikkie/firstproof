import SwiftUI
import AppKit
import LumenBoxBridge

public struct ExportSheetView: View {
    @Bindable var viewModel: AppViewModel
    @Environment(\.dismiss) private var dismiss
    
    @State private var selectedDestination: String = FileManager.default.urls(for: .desktopDirectory, in: .userDomainMask).first?.path ?? "/Users"
    @State private var exportFormat: String = "JPEG"
    @State private var quality: Double = 90.0
    @State private var sizeOption: String = "Original"
    @State private var customDimension: String = "2048"
    @State private var isExporting: Bool = false
    @State private var exportResultMessage: String? = nil
    
    public init(viewModel: AppViewModel) {
        self.viewModel = viewModel
    }
    
    public var body: some View {
        VStack(spacing: 20) {
            // Header
            HStack {
                Image(systemName: "square.and.arrow.up.fill")
                    .foregroundColor(.accentColor)
                    .font(.title2)
                Text("Export Photo")
                    .font(.headline)
                Spacer()
                Button("Cancel") {
                    dismiss()
                }
                .keyboardShortcut(.cancelAction)
            }
            
            Divider()
            
            // Destination Folder
            VStack(alignment: .leading, spacing: 6) {
                Text("Destination Folder:")
                    .font(.subheadline)
                    .fontWeight(.medium)
                
                HStack {
                    Text(selectedDestination)
                        .font(.caption)
                        .lineLimit(1)
                        .truncationMode(.middle)
                        .padding(8)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Color.secondary.opacity(0.1))
                        .cornerRadius(6)
                    
                    Button("Browse...") {
                        selectDestinationFolder()
                    }
                }
            }
            
            // Format & Quality
            VStack(alignment: .leading, spacing: 6) {
                Text("Export Format:")
                    .font(.subheadline)
                    .fontWeight(.medium)
                
                Picker("Format", selection: $exportFormat) {
                    Text("JPEG (.jpg)").tag("JPEG")
                    Text("16-Bit TIFF (.tiff)").tag("TIFF")
                }
                .pickerStyle(.segmented)
                
                if exportFormat == "JPEG" {
                    HStack {
                        Text("Quality: \(Int(quality))%")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Slider(value: $quality, in: 50...100, step: 1)
                    }
                    .padding(.top, 4)
                }
            }
            
            // Sizing
            VStack(alignment: .leading, spacing: 6) {
                Text("Image Sizing:")
                    .font(.subheadline)
                    .fontWeight(.medium)
                
                Picker("Size", selection: $sizeOption) {
                    Text("Original Resolution").tag("Original")
                    Text("Web (Long Edge 2048px)").tag("2048")
                    Text("4K Ultra HD (3840px)").tag("3840")
                    Text("Custom Dimension").tag("Custom")
                }
                .pickerStyle(.menu)
                
                if sizeOption == "Custom" {
                    HStack {
                        Text("Max Dimension (px):")
                            .font(.caption)
                        TextField("2048", text: $customDimension)
                            .textFieldStyle(.roundedBorder)
                            .frame(width: 80)
                    }
                }
            }
            
            if let result = exportResultMessage {
                Text(result)
                    .font(.caption)
                    .foregroundColor(result.contains("Success") ? .green : .red)
            }
            
            Divider()
            
            // Actions
            HStack {
                Spacer()
                
                if isExporting {
                    ProgressView()
                        .controlSize(.small)
                        .padding(.trailing, 8)
                }
                
                Button("Export") {
                    performExport()
                }
                .buttonStyle(.borderedProminent)
                .keyboardShortcut(.defaultAction)
                .disabled(isExporting || viewModel.selectedPhoto == nil)
            }
        }
        .padding(24)
        .frame(width: 440)
    }
    
    private func selectDestinationFolder() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.allowsMultipleSelection = false
        panel.prompt = "Choose Export Folder"
        if panel.runModal() == .OK, let url = panel.url {
            selectedDestination = url.path
        }
    }
    
    private func performExport() {
        guard let photo = viewModel.selectedPhoto else { return }
        isExporting = true
        exportResultMessage = nil
        
        let maxDim: UInt32? = {
            switch sizeOption {
            case "2048": return 2048
            case "3840": return 3840
            case "Custom": return UInt32(customDimension)
            default: return nil
            }
        }()
        
        let ext = (exportFormat == "TIFF") ? "tiff" : "jpg"
        let baseName = URL(fileURLWithPath: photo.filePath).deletingPathExtension().lastPathComponent
        let outputPath = URL(fileURLWithPath: selectedDestination)
            .appendingPathComponent("\(baseName)_LumenBox.\(ext)")
            .path
        
        let edits = EditSettings(
            exposure: viewModel.exposure,
            contrast: viewModel.contrast,
            temperature: viewModel.temperature,
            tint: viewModel.tint,
            cropLeft: 0.0,
            cropTop: 0.0,
            cropRight: 1.0,
            cropBottom: 1.0,
            cropAngle: viewModel.straightenAngle
        )
        
        DispatchQueue.global(qos: .userInitiated).async {
            let success = exportPhoto(
                inputPath: photo.filePath,
                outputPath: outputPath,
                edits: edits,
                quality: UInt32(quality),
                maxDimension: maxDim
            )
            
            DispatchQueue.main.async {
                self.isExporting = false
                if success {
                    self.exportResultMessage = "Successfully exported to \(outputPath)"
                    DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
                        self.dismiss()
                    }
                } else {
                    self.exportResultMessage = "Export failed. Please check folder permissions."
                }
            }
        }
    }
}
