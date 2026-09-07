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
    @State private var colorSpaceChoice: String = "sRGB" // "sRGB" or "DisplayP3"
    @State private var renderingIntentChoice: String = "RelativeColorimetric" // "RelativeColorimetric" or "Perceptual"
    @State private var isExporting: Bool = false
    @State private var exportResultMessage: String? = nil
    
    public init(viewModel: AppViewModel) {
        self.viewModel = viewModel
    }
    
    private var targetPhotos: [PhotoRecord] {
        viewModel.selectedPhotosList
    }
    
    public var body: some View {
        VStack(spacing: 18) {
            // Header
            HStack {
                Image(systemName: "square.and.arrow.up.fill")
                    .foregroundColor(.accentColor)
                    .font(.title2)
                VStack(alignment: .leading, spacing: 2) {
                    Text(targetPhotos.count > 1 ? "Batch Export (\(targetPhotos.count) Photos)" : "Export Photo")
                        .font(.headline)
                    if targetPhotos.count > 1 {
                        Text("Multi-core parallel processing via Rayon engine")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                }
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
            
            // Color Space & Rendering Intent (LittleCMS2 Engine)
            VStack(alignment: .leading, spacing: 8) {
                Text("Color Space & ICC Profile:")
                    .font(.subheadline)
                    .fontWeight(.medium)
                
                HStack(spacing: 20) {
                    Picker("", selection: $colorSpaceChoice) {
                        Text("sRGB").tag("sRGB")
                        Text("Display P3").tag("DisplayP3")
                    }
                    .pickerStyle(.radioGroup)
                    .horizontalRadioGroupLayout()
                    .labelsHidden()
                    
                    if colorSpaceChoice == "DisplayP3" {
                        HStack(spacing: 6) {
                            Text("Intent:")
                                .font(.caption)
                                .foregroundColor(.secondary)
                            Picker("", selection: $renderingIntentChoice) {
                                Text("Relative Colorimetric").tag("RelativeColorimetric")
                                Text("Perceptual").tag("Perceptual")
                            }
                            .pickerStyle(.menu)
                            .labelsHidden()
                            .controlSize(.small)
                        }
                    }
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
                
                Button(targetPhotos.count > 1 ? "Export \(targetPhotos.count) Photos" : "Export") {
                    performExport()
                }
                .buttonStyle(.borderedProminent)
                .keyboardShortcut(.defaultAction)
                .disabled(isExporting || targetPhotos.isEmpty)
            }
        }
        .padding(24)
        .frame(width: 480)
        .onAppear {
            NotificationService.shared.requestAuthorization()
        }
        .onExitCommand {
            dismiss()
        }
    }
    
    private func selectDestinationFolder() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.canCreateDirectories = true
        panel.allowsMultipleSelection = false
        panel.prompt = "Choose Export Folder"
        if panel.runModal() == .OK, let url = panel.url {
            selectedDestination = url.path
        }
    }
    
    private func performExport() {
        let photos = targetPhotos
        guard !photos.isEmpty else { return }
        
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
        let destination = selectedDestination
        let cs: ExportColorSpace = (colorSpaceChoice == "DisplayP3") ? .displayP3 : .srgb
        let ri: ExportRenderingIntent = (renderingIntentChoice == "Perceptual") ? .perceptual : .relativeColorimetric
        
        // Single photo export: can perform and wait or dock
        if photos.count == 1, let photo = photos.first {
            let baseName = URL(fileURLWithPath: photo.filePath).deletingPathExtension().lastPathComponent
            let outputPath = URL(fileURLWithPath: destination)
                .appendingPathComponent("\(baseName)_LumenBox.\(ext)")
                .path
            
            let edits = (photo.filePath == viewModel.selectedPhoto?.filePath) ? viewModel.currentEdits : (readXmp(path: photo.filePath)?.edits ?? .defaultIdentity)
            let lensCoeffs = (photo.filePath == viewModel.selectedPhoto?.filePath && viewModel.lensCorrectionEnabled) ? viewModel.lensCoefficients : nil
            
            DispatchQueue.global(qos: .userInitiated).async {
                let success = exportPhoto(
                    inputPath: photo.filePath,
                    outputPath: outputPath,
                    edits: edits,
                    lensCoeffs: lensCoeffs,
                    quality: UInt32(quality),
                    maxDimension: maxDim,
                    colorSpace: cs,
                    renderingIntent: ri
                )
                
                DispatchQueue.main.async {
                    self.isExporting = false
                    if success {
                        self.exportResultMessage = "Successfully exported to \(outputPath)"
                        NotificationService.shared.postBatchExportCompletedNotification(count: 1, destination: destination)
                        DispatchQueue.main.asyncAfter(deadline: .now() + 1.2) {
                            self.dismiss()
                        }
                    } else {
                        self.exportResultMessage = "Export failed. Please check folder permissions."
                    }
                }
            }
        } else {
            // Batch export (Story 17.2 & 17.3): Dismiss sheet immediately and run in background queue with docked progress bar!
            var items: [BatchExportItemRecord] = []
            for photo in photos {
                let baseName = URL(fileURLWithPath: photo.filePath).deletingPathExtension().lastPathComponent
                let outputPath = URL(fileURLWithPath: destination)
                    .appendingPathComponent("\(baseName)_LumenBox.\(ext)")
                    .path
                let edits = (photo.filePath == viewModel.selectedPhoto?.filePath) ? viewModel.currentEdits : (readXmp(path: photo.filePath)?.edits ?? .defaultIdentity)
                let lensCoeffs = (photo.filePath == viewModel.selectedPhoto?.filePath && viewModel.lensCorrectionEnabled) ? viewModel.lensCoefficients : nil
                
                items.append(BatchExportItemRecord(
                    inputPath: photo.filePath,
                    outputPath: outputPath,
                    edits: edits,
                    lensCoeffs: lensCoeffs,
                    quality: UInt32(quality),
                    maxDimension: maxDim,
                    colorSpace: cs,
                    renderingIntent: ri
                ))
            }
            
            // Set up background queue state
            viewModel.isBatchExporting = true
            viewModel.isBatchExportCancelled = false
            viewModel.batchExportTotalCount = items.count
            viewModel.batchExportCompletedCount = 0
            viewModel.batchExportProgress = 0.0
            viewModel.batchExportStatusText = "Exporting \(items.count) photos in parallel..."
            
            dismiss()
            
            DispatchQueue.global(qos: .userInitiated).async { [weak viewModel] in
                guard let vm = viewModel else { return }
                
                // Chunk items into slices of 4 so we can provide real-time animated UI progress
                let chunkSize = 4
                var allResults: [BatchExportResultRecord] = []
                let total = items.count
                var completed = 0
                
                for i in stride(from: 0, to: items.count, by: chunkSize) {
                    if vm.isBatchExportCancelled { break }
                    let chunk = Array(items[i..<min(i + chunkSize, items.count)])
                    let chunkResults = exportBatchPhotos(items: chunk)
                    allResults.append(contentsOf: chunkResults)
                    completed += chunk.count
                    
                    let prog = Double(completed) / Double(total)
                    DispatchQueue.main.async {
                        vm.batchExportCompletedCount = completed
                        vm.batchExportProgress = prog
                        vm.batchExportStatusText = "Exporting \(completed) of \(total)..."
                    }
                }
                
                let successCount = allResults.filter { $0.success }.count
                DispatchQueue.main.async {
                    vm.lastBatchExportResults = allResults
                    vm.batchExportStatusText = "Finished exporting \(successCount) of \(total) photos"
                    
                    if !vm.isBatchExportCancelled {
                        NotificationService.shared.postBatchExportCompletedNotification(count: successCount, destination: destination)
                    }
                    
                    DispatchQueue.main.asyncAfter(deadline: .now() + 3.0) {
                        if !vm.isBatchExportCancelled {
                            vm.isBatchExporting = false
                        }
                    }
                }
            }
        }
    }
}
