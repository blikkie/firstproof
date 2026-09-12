import SwiftUI
import AppKit
import FirstProofBridge

public struct ExportPackageSheetView: View {
    @Bindable var viewModel: AppViewModel
    @Environment(\.dismiss) private var dismiss
    
    @State private var selectedDestination: String = (FileManager.default.urls(for: .picturesDirectory, in: .userDomainMask).first?.appendingPathComponent("First Proof Export").path) ?? (NSHomeDirectory() + "/Pictures/First Proof Export")
    @State private var moveFiles: Bool = false
    @State private var openAfterExport: Bool = true
    @State private var isExporting: Bool = false
    @State private var report: ExportPackageReport? = nil
    @State private var errorMessage: String? = nil
    
    public init(viewModel: AppViewModel) {
        self.viewModel = viewModel
    }
    
    private var targetCount: Int {
        let count = viewModel.selectedPhotoPaths.count
        return count > 0 ? count : (viewModel.selectedPhoto != nil ? 1 : 0)
    }
    
    public var body: some View {
        VStack(spacing: 20) {
            // Header
            HStack(spacing: 12) {
                Image(systemName: "shippingbox.fill")
                    .foregroundColor(.accentColor)
                    .font(.system(size: 28))
                
                VStack(alignment: .leading, spacing: 2) {
                    Text("Export as Portable Library")
                        .font(.headline)
                    Text("Package \(targetCount) selected \(targetCount == 1 ? "photo" : "photos") into a standalone project folder with an isolated catalog and sidecars.")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
                
                Button("Close") {
                    dismiss()
                }
                .keyboardShortcut(.cancelAction)
            }
            
            Divider()
            
            if let report = report {
                // Success Result View
                VStack(spacing: 16) {
                    Image(systemName: report.errorCount == 0 ? "checkmark.circle.fill" : "exclamationmark.triangle.fill")
                        .foregroundColor(report.errorCount == 0 ? .green : .orange)
                        .font(.system(size: 44))
                    
                    VStack(spacing: 4) {
                        Text(report.errorCount == 0 ? "Export Completed Successfully!" : "Export Completed with Warnings")
                            .font(.headline)
                        Text("Exported \(report.exportedCount) of \(report.totalRequested) items to:")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                        Text(selectedDestination)
                            .font(.caption)
                            .fontWeight(.medium)
                            .padding(.horizontal, 12)
                            .padding(.vertical, 4)
                            .background(Color.secondary.opacity(0.1))
                            .cornerRadius(4)
                    }
                    
                    if !report.errorMessages.isEmpty {
                        VStack(alignment: .leading, spacing: 2) {
                            ForEach(report.errorMessages, id: \.self) { msg in
                                Text("• \(msg)")
                                    .font(.caption2)
                                    .foregroundColor(.red)
                            }
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(8)
                        .background(Color.red.opacity(0.1))
                        .cornerRadius(6)
                    }
                    
                    HStack(spacing: 12) {
                        Button("Reveal in Finder") {
                            NSWorkspace.shared.selectFile(nil, inFileViewerRootedAtPath: selectedDestination)
                        }
                        
                        Button("Done") {
                            dismiss()
                        }
                        .buttonStyle(.borderedProminent)
                    }
                    .padding(.top, 8)
                }
                .padding(.vertical, 12)
            } else {
                // Export Configuration Form
                VStack(alignment: .leading, spacing: 18) {
                    // Destination Folder
                    VStack(alignment: .leading, spacing: 6) {
                        Text("Destination Package Directory:")
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
                            
                            Button("Choose...") {
                                chooseDestination()
                            }
                        }
                    }
                    
                    // Transfer Mode (Copy vs Move)
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Transfer Mode:")
                            .font(.subheadline)
                            .fontWeight(.medium)
                        
                        VStack(alignment: .leading, spacing: 6) {
                            Button(action: { moveFiles = false }) {
                                HStack(alignment: .top, spacing: 8) {
                                    Image(systemName: !moveFiles ? "largecircle.fill.circle" : "circle")
                                        .foregroundColor(!moveFiles ? .accentColor : .secondary)
                                    VStack(alignment: .leading, spacing: 2) {
                                        Text("Copy Files (Safe Duplication)")
                                            .font(.subheadline)
                                            .fontWeight(!moveFiles ? .semibold : .regular)
                                        Text("Leaves original RAWs and sidecars in current library. Copies data into destination.")
                                            .font(.caption2)
                                            .foregroundColor(.secondary)
                                    }
                                }
                            }
                            .buttonStyle(.plain)
                            
                            Button(action: { moveFiles = true }) {
                                HStack(alignment: .top, spacing: 8) {
                                    Image(systemName: moveFiles ? "largecircle.fill.circle" : "circle")
                                        .foregroundColor(moveFiles ? .accentColor : .secondary)
                                    VStack(alignment: .leading, spacing: 2) {
                                        Text("Move Files (Relocate)")
                                            .font(.subheadline)
                                            .fontWeight(moveFiles ? .semibold : .regular)
                                        Text("Transfers photos and sidecars into new library folder. Removes items from current library.")
                                            .font(.caption2)
                                            .foregroundColor(.secondary)
                                    }
                                }
                            }
                            .buttonStyle(.plain)
                        }
                        .padding(10)
                        .background(Color.secondary.opacity(0.06))
                        .cornerRadius(8)
                    }
                    
                    // Options
                    Toggle("Mount as active Project Session in sidebar after export", isOn: $openAfterExport)
                        .font(.subheadline)
                    
                    if let err = errorMessage {
                        Text(err)
                            .font(.caption)
                            .foregroundColor(.red)
                    }
                }
                
                Divider()
                
                // Footer Actions
                HStack {
                    if isExporting {
                        ProgressView("Packaging \(targetCount) photos...")
                            .controlSize(.small)
                    }
                    
                    Spacer()
                    
                    Button("Cancel") {
                        dismiss()
                    }
                    .keyboardShortcut(.cancelAction)
                    .disabled(isExporting)
                    
                    Button(moveFiles ? "Move to Portable Library" : "Copy to Portable Library") {
                        performExport()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(isExporting || targetCount == 0)
                }
            }
        }
        .padding(24)
        .frame(width: 520)
        .onExitCommand {
            dismiss()
        }
    }
    
    private func chooseDestination() {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.canCreateDirectories = true
        panel.prompt = "Select Package Folder"
        
        if panel.runModal() == .OK, let url = panel.url {
            selectedDestination = url.path
        }
    }
    
    private func performExport() {
        isExporting = true
        errorMessage = nil
        
        viewModel.exportSelectedAsPackage(
            destinationFolder: selectedDestination,
            moveFiles: moveFiles,
            openAfter: openAfterExport
        ) { rep in
            self.isExporting = false
            if let rep = rep {
                self.report = rep
            } else {
                self.errorMessage = "Export failed. Please check folder permissions and disk space."
            }
        }
    }
}
