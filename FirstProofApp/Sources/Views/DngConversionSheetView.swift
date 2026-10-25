import SwiftUI
import AppKit
import FirstProofBridge

@MainActor
public struct DngConversionSheetView: View {
    @Environment(AppViewModel.self) private var viewModel
    @Environment(\.dismiss) private var dismiss
    
    @State private var deleteOriginal: Bool = false
    
    public init() {}
    
    private var candidateCount: Int {
        viewModel.convertibleRawPhotosSelected.count
    }
    
    public var body: some View {
        VStack(spacing: 20) {
            // Header
            HStack(spacing: 12) {
                Image(systemName: "archivebox.fill")
                    .foregroundColor(.accentColor)
                    .font(.system(size: 28))
                
                VStack(alignment: .leading, spacing: 2) {
                    Text("Convert to Archival DNG")
                        .font(.headline)
                    Text("Transcode proprietary camera negatives into open-standard Adobe DNG 1.4+ containers with bit-level CFA verification.")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
                
                if !viewModel.isConvertingDng && viewModel.dngBatchReport == nil {
                    Button("Cancel") {
                        dismiss()
                    }
                    .keyboardShortcut(.cancelAction)
                }
            }
            
            Divider()
            
            if let report = viewModel.dngBatchReport {
                reportView(report)
            } else if viewModel.isConvertingDng {
                convertingProgressView
            } else {
                configurationForm
            }
        }
        .padding(24)
        .frame(width: 560)
    }
    
    // MARK: - Configuration Form
    private var configurationForm: some View {
        VStack(alignment: .leading, spacing: 16) {
            // Scope & Information Box
            VStack(alignment: .leading, spacing: 10) {
                Text("CONVERSION SCOPE")
                    .font(.caption2)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                
                HStack(spacing: 10) {
                    Image(systemName: "photo.stack.fill")
                        .foregroundColor(.accentColor)
                        .font(.title2)
                    
                    VStack(alignment: .leading, spacing: 2) {
                        Text("\(candidateCount) Proprietary Camera RAW\(candidateCount == 1 ? "" : "s") Selected")
                            .font(.subheadline)
                            .fontWeight(.medium)
                        Text("Silently skips raster images (.jpg, .png, .tif) and existing .dng containers.")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                    Spacer()
                }
                .padding(10)
                .background(Color.secondary.opacity(0.06))
                .cornerRadius(6)
            }
            
            // Archival Invariants Info
            VStack(alignment: .leading, spacing: 8) {
                Text("ARCHIVAL INVARIANTS")
                    .font(.caption2)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                
                VStack(alignment: .leading, spacing: 6) {
                    featureRow(icon: "checkmark.shield.fill", color: .green, title: "Lossless Bayer CFA Encapsulation", detail: "Preserves exact un-demosaiced sensor mosaic via Lossless JPEG-92.")
                    featureRow(icon: "lock.shield.fill", color: .blue, title: "Cryptographic Read-Back Verification", detail: "Computes SHA-256 before and after write, asserting bit-for-bit parity off disk.")
                    featureRow(icon: "doc.text.fill", color: .orange, title: "Sidecar Continuity & Catalog In-Place Sync", detail: "Migrates ratings, pick flags, and XMP edits seamlessly without database clobbering.")
                }
                .padding(10)
                .background(Color.secondary.opacity(0.06))
                .cornerRadius(6)
            }
            
            // Deletion Safety Option
            VStack(alignment: .leading, spacing: 6) {
                Text("SOURCE FILE HANDLING")
                    .font(.caption2)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                
                Toggle("Delete original RAW negatives after verified conversion", isOn: $deleteOriginal)
                    .font(.subheadline)
                
                if deleteOriginal {
                    Text("⚠️ Source negatives will only be unlinked after independent disk read-back confirms 100% cryptographic SHA-256 CFA match.")
                        .font(.caption2)
                        .foregroundColor(.orange)
                } else {
                    Text("Keeps original proprietary RAW files intact alongside the newly created .dng files.")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
            }
            .padding(10)
            .background(Color.secondary.opacity(0.06))
            .cornerRadius(6)
            
            if let errorMsg = viewModel.dngBatchErrorMessage {
                Text(errorMsg)
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
                
                Button(action: {
                    viewModel.executeBatchDngConversion(deleteOriginal: deleteOriginal)
                }) {
                    Text(candidateCount > 0
                        ? "Convert \(candidateCount) Negative\(candidateCount == 1 ? "" : "s") to DNG"
                        : "Convert to DNG")
                }
                .buttonStyle(.borderedProminent)
                .disabled(candidateCount == 0)
            }
        }
    }
    
    private func featureRow(icon: String, color: Color, title: String, detail: String) -> some View {
        HStack(alignment: .top, spacing: 8) {
            Image(systemName: icon)
                .foregroundColor(color)
                .font(.caption)
                .frame(width: 14)
                .padding(.top, 2)
            
            VStack(alignment: .leading, spacing: 1) {
                Text(title)
                    .font(.caption)
                    .fontWeight(.medium)
                Text(detail)
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
        }
    }
    
    // MARK: - Progress View
    private var convertingProgressView: some View {
        VStack(spacing: 16) {
            VStack(spacing: 12) {
                ProgressView(
                    value: Double(viewModel.dngProgressCurrent),
                    total: Double(max(1, viewModel.dngProgressTotal))
                )
                .progressViewStyle(.linear)
                .frame(maxWidth: 420)
                
                HStack {
                    Text("Transcoding \(viewModel.dngProgressCurrent) of \(viewModel.dngProgressTotal) negatives...")
                        .font(.system(.subheadline, design: .rounded))
                        .fontWeight(.medium)
                    
                    Spacer()
                    
                    if viewModel.dngProgressTotal > 0 {
                        let pct = Int((Double(viewModel.dngProgressCurrent) / Double(max(1, viewModel.dngProgressTotal))) * 100)
                        Text("\(pct)%")
                            .font(.system(.subheadline, design: .monospaced))
                            .foregroundColor(.secondary)
                    }
                }
                .frame(maxWidth: 420)
                
                if !viewModel.dngProgressFileName.isEmpty {
                    Text(viewModel.dngProgressFileName)
                        .font(.system(.caption2, design: .monospaced))
                        .foregroundColor(.secondary)
                        .lineLimit(1)
                        .truncationMode(.middle)
                        .frame(maxWidth: 420)
                }
            }
            .padding(.top, 16)
            
            Divider()
            
            HStack {
                Spacer()
                Button("Cancel") {
                    viewModel.cancelDngBatchConversion()
                }
            }
        }
        .frame(maxWidth: .infinity, minHeight: 180)
    }
    
    // MARK: - Report View
    private func reportView(_ report: DngBatchReportRecord) -> some View {
        let transcodedReports = report.reports.filter { !$0.alreadyExisted }
        let reconciledReports = report.reports.filter { $0.alreadyExisted }
        let deletedCount = report.reports.filter { $0.deletedOriginal }.count
        
        let transcodedCount = transcodedReports.count
        let reconciledCount = reconciledReports.count
        
        let headline: String = {
            if report.failedCount > 0 {
                return "Conversion Finished with Warnings"
            }
            if transcodedCount > 0 && reconciledCount == 0 {
                return "Archival DNG Conversion Complete!"
            }
            if transcodedCount == 0 && reconciledCount > 0 {
                return "Archival DNG Reconciliation Complete!"
            }
            return "DNG Conversion & Reconciliation Complete!"
        }()
        
        let detailText: String = {
            if transcodedCount == 0 && reconciledCount > 0 {
                if deletedCount > 0 {
                    return "Verified existing DNG container\(reconciledCount == 1 ? "" : "s") off disk. Duplicate original\(deletedCount == 1 ? "" : "s") safely deleted and sidecars merged."
                } else {
                    return "Verified existing DNG container\(reconciledCount == 1 ? "" : "s") off disk. Original negatives preserved and sidecars merged."
                }
            } else if deletedCount > 0 {
                return "Transcoded to open-standard DNG with bit-level CFA verification. Original negatives safely deleted and sidecars updated."
            } else {
                return "Transcoded to open-standard DNG with bit-level CFA verification. Original negatives preserved on disk."
            }
        }()

        return VStack(spacing: 16) {
            Image(systemName: report.failedCount == 0 ? "checkmark.circle.fill" : "exclamationmark.triangle.fill")
                .foregroundColor(report.failedCount == 0 ? .green : .orange)
                .font(.system(size: 44))
            
            Text(headline)
                .font(.headline)
            
            // Stats Grid
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                if transcodedCount > 0 {
                    statTile(title: "Transcoded DNGs", value: "\(transcodedCount)", icon: "archivebox.fill", color: .green)
                }
                if reconciledCount > 0 {
                    statTile(title: "Reconciled Existing DNGs", value: "\(reconciledCount)", icon: "arrow.triangle.merge", color: .blue)
                }
                if deletedCount > 0 {
                    statTile(title: "Originals Safely Deleted", value: "\(deletedCount)", icon: "trash.fill", color: .teal)
                }
                if report.skippedCount > 0 {
                    statTile(title: "Skipped (Raster / Non-RAW)", value: "\(report.skippedCount)", icon: "arrow.uturn.right.circle", color: .secondary)
                }
                
                if transcodedCount > 0 {
                    let sourceMB = Double(transcodedReports.map(\.sourceSizeBytes).reduce(0, +)) / 1_048_576.0
                    let dngMB = Double(transcodedReports.map(\.dngSizeBytes).reduce(0, +)) / 1_048_576.0
                    let savedMB = max(0.0, sourceMB - dngMB)
                    let pct = sourceMB > 0 ? (savedMB / sourceMB) * 100.0 : 0.0
                    
                    if sourceMB > 0 && savedMB > 0 {
                        statTile(
                            title: "Space Reduction",
                            value: String(format: "%.1f MB (%.1f%%)", savedMB, pct),
                            icon: "arrow.down.right.and.arrow.up.left",
                            color: .purple
                        )
                    }
                }
                
                if report.spaceReclaimedBytes > 0 {
                    let reclaimedMB = Double(report.spaceReclaimedBytes) / 1_048_576.0
                    statTile(
                        title: "Disk Space Reclaimed",
                        value: reclaimedMB > 1024 ? String(format: "%.2f GB", reclaimedMB / 1024.0) : String(format: "%.1f MB", reclaimedMB),
                        icon: "externaldrive.badge.minus",
                        color: .teal
                    )
                }
                
                if report.failedCount > 0 {
                    statTile(title: "Failed Conversions", value: "\(report.failedCount)", icon: "xmark.octagon.fill", color: .red)
                }
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
            
            Text(detailText)
                .font(.caption)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
            
            Divider()
            
            HStack {
                Spacer()
                Button("Done") {
                    dismiss()
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
}
