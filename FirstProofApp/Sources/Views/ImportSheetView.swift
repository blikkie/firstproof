import SwiftUI
import AppKit
import FirstProofBridge

@MainActor
public struct ImportSheetView: View {
    @Environment(AppViewModel.self) private var viewModel
    @Environment(\.dismiss) private var dismiss
    
    @State private var subfolderName: String = ""
    @State private var moveFiles: Bool = false
    @AppStorage("defaultConsolidateSidecars") private var consolidateSidecars: Bool = true
    @State private var deduplicateRawOverJpeg: Bool = true
    @AppStorage("defaultSkipDuplicates") private var skipDuplicates: Bool = true
    @State private var deleteFromCard: Bool = false
    
    @State private var isImporting: Bool = false
    @State private var report: ImportReportRecord? = nil
    @State private var errorMessage: String? = nil
    
    public init() {}
    
    private var sourcePath: String {
        viewModel.importSourcePath ?? viewModel.currentFolderPath ?? ""
    }
    
    private var sourceName: String {
        URL(fileURLWithPath: sourcePath).lastPathComponent
    }
    
    private var isSourceCard: Bool {
        sourcePath.hasPrefix("/Volumes/") && !sourcePath.hasPrefix("/Volumes/Macintosh HD")
    }
    
    private var isCurrentFolderActive: Bool {
        viewModel.importSourcePath == nil || viewModel.importSourcePath == viewModel.currentFolderPath
    }
    
    private var targetCount: Int {
        if !viewModel.importSelectedPaths.isEmpty {
            return viewModel.importSelectedPaths.count
        }
        if isCurrentFolderActive {
            return deduplicateRawOverJpeg ? viewModel.filteredPhotos.count : viewModel.photos.count
        }
        return viewModel.photos.count
    }
    
    public var body: some View {
        VStack(spacing: 20) {
            // Header
            HStack(spacing: 12) {
                Image(systemName: isSourceCard ? "externaldrive.fill.badge.plus" : "square.and.arrow.down.fill")
                    .foregroundColor(.accentColor)
                    .font(.system(size: 28))
                
                VStack(alignment: .leading, spacing: 2) {
                    Text("Import into Main Library")
                        .font(.headline)
                    Text("Transfer negatives from \"\(sourceName)\" into your primary library with deterministic deduplication and sidecar consolidation.")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
                
                if !isImporting && report == nil {
                    Button("Cancel") {
                        dismiss()
                    }
                    .keyboardShortcut(.cancelAction)
                }
            }
            
            Divider()
            
            if let report = report {
                reportView(report)
            } else if isImporting {
                importingProgressView
            } else {
                configurationForm
            }
        }
        .padding(24)
        .frame(width: 560)
        .onAppear {
            self.deduplicateRawOverJpeg = viewModel.importOriginalsOnly
            if subfolderName.isEmpty {
                let formatter = DateFormatter()
                formatter.dateFormat = "yyyy-MM-dd"
                let dateStr = formatter.string(from: Date())
                if !sourceName.isEmpty && sourceName != "DCIM" && !sourceName.hasPrefix("Volumes") {
                    subfolderName = "\(dateStr) - \(sourceName)"
                } else {
                    subfolderName = dateStr
                }
            }
        }
        .onDisappear {
            viewModel.importSelectedPaths = []
        }
    }
    
    // MARK: - Configuration Form
    private var configurationForm: some View {
        VStack(alignment: .leading, spacing: 16) {
            // Destination
            VStack(alignment: .leading, spacing: 6) {
                Text("DESTINATION")
                    .font(.caption2)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                
                HStack {
                    Image(systemName: "folder.fill")
                        .foregroundColor(.accentColor)
                    Text(viewModel.mainLibraryPath)
                        .font(.system(.caption, design: .monospaced))
                        .lineLimit(1)
                        .truncationMode(.middle)
                    Spacer()
                }
                .padding(8)
                .background(Color.secondary.opacity(0.08))
                .cornerRadius(6)
                
                HStack {
                    Text("Subfolder:")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    TextField("Optional subfolder (e.g. 2026-09-13)", text: $subfolderName)
                        .textFieldStyle(.roundedBorder)
                        .font(.caption)
                }
            }
            
            // Ingestion Options
            VStack(alignment: .leading, spacing: 8) {
                Text("DEDUPLICATION & PRECEDENCE")
                    .font(.caption2)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                
                Toggle(
                    deduplicateRawOverJpeg
                        ? "Originals Only: Import original RAW/negatives only (excludes export copies like _edited.jpg, -1.dng)"
                        : "All Copies: Import all files including export derivatives and duplicate versions",
                    isOn: $deduplicateRawOverJpeg
                )
                .font(.subheadline)
                
                Toggle("Consolidate Sidecars: Merge .xmp and Takeout .json into single First Proof XMP", isOn: $consolidateSidecars)
                    .font(.subheadline)
                
                Toggle("Skip duplicates already present in library", isOn: $skipDuplicates)
                    .font(.subheadline)
            }
            
            // Transfer Method / Safety
            VStack(alignment: .leading, spacing: 8) {
                Text("TRANSFER METHOD")
                    .font(.caption2)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                
                if isSourceCard {
                    // Memory Card Safety Box
                    VStack(alignment: .leading, spacing: 6) {
                        HStack(spacing: 8) {
                            Image(systemName: "checkmark.shield.fill")
                                .foregroundColor(deleteFromCard ? .orange : .green)
                            Text("Safe Ingest (Camera Card)")
                                .font(.subheadline)
                                .fontWeight(.medium)
                        }
                        
                        Toggle("Delete negatives from card after import", isOn: $deleteFromCard)
                            .font(.subheadline)
                            .onChange(of: deleteFromCard) { _, newValue in
                                moveFiles = newValue
                            }
                        
                        if deleteFromCard {
                            Text("⚠️ Camera cards should normally be formatted in-camera. Source files will only be removed after cryptographic SHA-256 bit-level verification confirms exact copy.")
                                .font(.caption2)
                                .foregroundColor(.orange)
                        } else {
                            Text("Keeps original negatives on the memory card as your primary field backup.")
                                .font(.caption2)
                                .foregroundColor(.secondary)
                        }
                    }
                    .padding(10)
                    .background(Color.secondary.opacity(0.06))
                    .cornerRadius(6)
                } else {
                    HStack(spacing: 8) {
                        Image(systemName: "checkmark.shield.fill")
                            .foregroundColor(.green)
                            .font(.title3)
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Safe Non-Destructive Copy")
                                .font(.subheadline)
                                .fontWeight(.medium)
                            Text("Source negatives in \"\(sourceName)\" remain completely untouched as an intact archive.")
                                .font(.caption2)
                                .foregroundColor(.secondary)
                        }
                    }
                    .padding(10)
                    .background(Color.secondary.opacity(0.06))
                    .cornerRadius(6)
                }
            }
            
            if let errorMessage = errorMessage {
                Text(errorMessage)
                    .font(.caption)
                    .foregroundColor(.red)
            }
            
            Divider()
            
            // Action Buttons
            HStack {
                Spacer()
                Button("Cancel") {
                    dismiss()
                }
                
                Button(action: startImport) {
                    Text(targetCount > 0
                        ? (deduplicateRawOverJpeg ? "Import \(targetCount) Originals" : "Import \(targetCount) Photos")
                        : "Import All Photos")
                }
                .buttonStyle(.borderedProminent)
            }
        }
    }
    
    // MARK: - Progress View
    private var importingProgressView: some View {
        VStack(spacing: 16) {
            if viewModel.importProgressTotal > 0 {
                VStack(spacing: 12) {
                    ProgressView(
                        value: Double(viewModel.importProgressCurrent),
                        total: Double(viewModel.importProgressTotal)
                    )
                    .progressViewStyle(.linear)
                    .frame(maxWidth: 420)
                    
                    HStack {
                        if viewModel.importProgressCurrent > 0 {
                            Text("\(viewModel.importProgressCurrent) of \(viewModel.importProgressTotal) imported")
                                .font(.system(.subheadline, design: .rounded))
                                .fontWeight(.medium)
                        } else {
                            Text("Preparing \(viewModel.importProgressTotal) files...")
                                .font(.system(.subheadline, design: .rounded))
                                .fontWeight(.medium)
                        }
                        
                        Spacer()
                        
                        if viewModel.importProgressTotal > 0 && viewModel.importProgressCurrent > 0 {
                            let pct = Int((Double(viewModel.importProgressCurrent) / Double(max(1, viewModel.importProgressTotal))) * 100)
                            Text("\(pct)%")
                                .font(.system(.subheadline, design: .monospaced))
                                .foregroundColor(.secondary)
                        }
                    }
                    .frame(maxWidth: 420)
                    
                    if !viewModel.importCurrentFileName.isEmpty {
                        Text(viewModel.importCurrentFileName)
                            .font(.system(.caption2, design: .monospaced))
                            .foregroundColor(.secondary)
                            .lineLimit(1)
                            .truncationMode(.middle)
                            .frame(maxWidth: 420)
                    }
                }
                .padding(.top, 16)
            } else {
                ProgressView()
                    .scaleEffect(1.2)
                    .padding(.top, 12)
                
                Text("Analyzing Library & Resolving Captures...")
                    .font(.headline)
                
                Text("Scanning files, clustering normalized stems, and resolving original precedence...")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, 20)
            }
        }
        .frame(maxWidth: .infinity, minHeight: 180)
    }
    
    // MARK: - Report View
    private func reportView(_ report: ImportReportRecord) -> some View {
        VStack(spacing: 16) {
            Image(systemName: report.errorCount == 0 ? "checkmark.circle.fill" : "exclamationmark.triangle.fill")
                .foregroundColor(report.errorCount == 0 ? .green : .orange)
                .font(.system(size: 44))
            
            Text(report.errorCount == 0 ? "Import Completed Successfully!" : "Import Finished with Warnings")
                .font(.headline)
            
            // Stats Grid
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                statTile(title: "Imported Originals", value: "\(report.importedCount)", icon: "photo.fill", color: .green)
                statTile(title: "Consolidated Sidecars", value: "\(report.consolidatedSidecars)", icon: "doc.text.fill", color: .blue)
                if report.duplicateClonesSuppressed > 0 {
                    statTile(title: "Duplicated Originals Skipped", value: "\(report.duplicateClonesSuppressed)", icon: "doc.on.doc.fill", color: .orange)
                }
                if report.rawOverJpegSuppressed > 0 {
                    statTile(title: "Suppressed JPEGs", value: "\(report.rawOverJpegSuppressed)", icon: "sparkles", color: .purple)
                }
                statTile(title: "Already in Library", value: "\(report.skippedDuplicates)", icon: "arrow.triangle.merge", color: .secondary)
            }
            .padding(.horizontal, 12)
            
            if !report.errorMessages.isEmpty {
                ScrollView {
                    VStack(alignment: .leading, spacing: 4) {
                        ForEach(report.errorMessages, id: \.self) { msg in
                            Text("• \(msg)")
                                .font(.caption2)
                                .foregroundColor(.red)
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .frame(maxHeight: 70)
                .padding(6)
                .background(Color.red.opacity(0.08))
                .cornerRadius(6)
            }
            
            Text(isSourceCard && deleteFromCard
                 ? "Import finished. Source card negatives removed after SHA-256 verification."
                 : "Source negatives in \"\(sourceName)\" remain intact in their original folder.")
                .font(.caption)
                .foregroundColor(.secondary)

            Divider()
            
            HStack {
                Button("Stay in \"\(sourceName)\"") {
                    dismiss()
                }
                
                Spacer()
                
                Button("Show in Main Library") {
                    let firstImportedName = viewModel.importSelectedPaths.first.map { URL(fileURLWithPath: $0).lastPathComponent }
                    dismiss()
                    viewModel.ratingFilter = nil
                    viewModel.statusFilter = nil
                    viewModel.enterGridMode()
                    viewModel.selectScope(.mainLibrary, preferredPhotoPath: firstImportedName)
                }
                .buttonStyle(.borderedProminent)
                .keyboardShortcut(.defaultAction)
            }
        }
    }
    
    private func statTile(title: String, value: String, icon: String, color: Color) -> some View {
        HStack(spacing: 8) {
            Image(systemName: icon)
                .foregroundColor(color)
                .font(.title3)
            VStack(alignment: .leading, spacing: 1) {
                Text(value)
                    .font(.headline)
                Text(title)
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
            Spacer()
        }
        .padding(8)
        .background(Color.secondary.opacity(0.06))
        .cornerRadius(6)
    }
    
    private func startImport() {
        self.isImporting = true
        self.viewModel.isImporting = true
        self.errorMessage = nil
        
        let sources: [String]
        if !viewModel.importSelectedPaths.isEmpty {
            sources = viewModel.importSelectedPaths
        } else if isCurrentFolderActive && !sourcePath.isEmpty {
            sources = deduplicateRawOverJpeg
                ? viewModel.filteredPhotos.map(\.filePath)
                : viewModel.photos.map(\.filePath)
        } else if !sourcePath.isEmpty {
            sources = [sourcePath]
        } else {
            self.errorMessage = "No source files or folder selected."
            self.isImporting = false
            self.viewModel.isImporting = false
            return
        }
        
        let effectiveMove = isSourceCard ? (deleteFromCard && moveFiles) : false
        let options = ImportOptionsRecord(
            moveFiles: effectiveMove,
            consolidateSidecars: consolidateSidecars,
            deduplicateRawOverJpeg: deduplicateRawOverJpeg,
            skipDuplicates: skipDuplicates,
            verifyChecksum: true,
            subfolderName: subfolderName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ? nil : subfolderName.trimmingCharacters(in: .whitespacesAndNewlines)
        )
        
        let dest = viewModel.mainLibraryPath
        
        viewModel.importProgressCurrent = 0
        viewModel.importProgressTotal = 0
        viewModel.importCurrentFileName = ""
        
        let tracker = ImportProgressTracker { [weak viewModel] curr, tot, file in
            DispatchQueue.main.async {
                viewModel?.importProgressCurrent = curr
                viewModel?.importProgressTotal = tot
                viewModel?.importCurrentFileName = file
            }
        }
        
        DispatchQueue.global(qos: .userInitiated).async {
            do {
                let rep = try importToLibrary(
                    sourcePathsOrFolders: sources,
                    destinationFolder: dest,
                    options: options,
                    progress: tracker
                )
                DispatchQueue.main.async {
                    self.report = rep
                    self.isImporting = false
                    self.viewModel.isImporting = false
                }
            } catch {
                DispatchQueue.main.async {
                    self.errorMessage = "Import failed: \(error.localizedDescription)"
                    self.isImporting = false
                    self.viewModel.isImporting = false
                }
            }
        }
    }
}

final class ImportProgressTracker: ImportProgressCallback, @unchecked Sendable {
    let handler: (UInt32, UInt32, String) -> Void
    init(_ handler: @escaping (UInt32, UInt32, String) -> Void) {
        self.handler = handler
    }
    func onProgress(current: UInt32, total: UInt32, currentFile: String) {
        handler(current, total, currentFile)
    }
}
