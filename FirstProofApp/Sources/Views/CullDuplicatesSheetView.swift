import SwiftUI
import AppKit
import FirstProofBridge

@MainActor
public struct CullDuplicatesSheetView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    public init() {}
    
    private var allRowsKept: Bool {
        !viewModel.cullClusters.isEmpty && viewModel.cullClusters.allSatisfy(\.isKept)
    }
    
    private var totalMarkedCount: Int {
        viewModel.cullClusters.reduce(0) { $0 + $1.markedDuplicatesCount }
    }
    
    private var totalMarkedBytes: Int64 {
        viewModel.cullClusters.reduce(0) { $0 + $1.markedDuplicatesBytes }
    }
    
    public var body: some View {
        VStack(spacing: 0) {
            // MARK: - Header
            HStack(alignment: .center) {
                VStack(alignment: .leading, spacing: 3) {
                    Text("Cull Duplicates")
                        .font(.title2)
                        .fontWeight(.semibold)
                    
                    Text("\(viewModel.cullScopeName) — \(viewModel.cullClusters.count) capture sets with duplicate copies")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
                
                if !viewModel.cullClusters.isEmpty {
                    HStack(spacing: 8) {
                        Button("Keep All (Exempt)") {
                            for cluster in viewModel.cullClusters {
                                cluster.isKept = true
                                for dup in cluster.duplicates {
                                    dup.isKept = true
                                }
                            }
                        }
                        .buttonStyle(.bordered)
                        .controlSize(.small)
                        .help("Exempt all duplicates from deletion")
                        
                        Button("Deselect All (Empty)") {
                            for cluster in viewModel.cullClusters {
                                cluster.isKept = false
                                for dup in cluster.duplicates {
                                    dup.isKept = false
                                }
                            }
                        }
                        .buttonStyle(.bordered)
                        .controlSize(.small)
                        .help("Default state: all duplicates queued for culling")
                    }
                }
            }
            .padding(.horizontal, 24)
            .padding(.top, 20)
            .padding(.bottom, 16)
            
            Divider()
            
            // MARK: - Column Headers
            HStack(spacing: 16) {
                // Column 1: Keep (Checkbox defaults to empty; checking exempts from deletion)
                HStack(spacing: 4) {
                    Toggle("", isOn: Binding(
                        get: { allRowsKept },
                        set: { newValue in
                            for cluster in viewModel.cullClusters {
                                cluster.isKept = newValue
                                for dup in cluster.duplicates {
                                    dup.isKept = newValue
                                }
                            }
                        }
                    ))
                    .toggleStyle(.checkbox)
                    .labelsHidden()
                    .help("Check to keep all duplicates (exempt from deletion)")
                    
                    Text("KEEP")
                        .font(.caption2)
                        .fontWeight(.bold)
                        .foregroundColor(.secondary)
                }
                .frame(width: 75, alignment: .center)
                
                // Column 2: Original
                Text("ORIGINAL")
                    .font(.caption2)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                    .frame(width: 270, alignment: .leading)
                
                // Column 3: Duplicates
                Text("DUPLICATES (WRAPPING)")
                    .font(.caption2)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .padding(.horizontal, 24)
            .padding(.vertical, 8)
            .background(Color(nsColor: .controlBackgroundColor).opacity(0.5))
            
            Divider()
            
            // MARK: - Content Body
            if viewModel.isScanningDuplicates {
                VStack(spacing: 16) {
                    Spacer()
                    ProgressView()
                        .controlSize(.large)
                    Text("Scanning for duplicate clones and derivative exports...")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                    Spacer()
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if viewModel.cullClusters.isEmpty {
                VStack(spacing: 12) {
                    Spacer()
                    Image(systemName: "checkmark.circle.fill")
                        .font(.system(size: 40))
                        .foregroundColor(.green)
                    Text("No Duplicates Found")
                        .font(.headline)
                    Text("All captures in \(viewModel.cullScopeName) are unique original captures.")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                    Spacer()
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(viewModel.cullClusters) { row in
                            CullRowView(row: row)
                            Divider()
                                .padding(.leading, 24)
                        }
                    }
                    .padding(.vertical, 8)
                }
            }
            
            Divider()
            
            // MARK: - Footer Actions
            HStack(alignment: .center) {
                VStack(alignment: .leading, spacing: 2) {
                    if totalMarkedCount > 0 {
                        HStack(spacing: 6) {
                            Image(systemName: "trash")
                                .foregroundColor(.red)
                            Text("\(totalMarkedCount) duplicate files queued for trash (\(formatByteCount(totalMarkedBytes)))")
                                .font(.subheadline)
                                .fontWeight(.medium)
                        }
                    } else {
                        HStack(spacing: 6) {
                            Image(systemName: "checkmark.shield.fill")
                                .foregroundColor(.green)
                            Text("All duplicate files are kept (exempted from deletion)")
                                .font(.subheadline)
                                .foregroundColor(.secondary)
                        }
                    }
                    
                    Text("Files will be safely moved to macOS Trash and can be restored if needed.")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
                
                Button("Cancel") {
                    viewModel.isCullDuplicatesSheetPresented = false
                }
                .keyboardShortcut(.escape, modifiers: [])
                
                Button(action: {
                    viewModel.performCullDuplicates()
                }) {
                    HStack(spacing: 6) {
                        Image(systemName: "trash.fill")
                        Text(totalMarkedCount > 0 ? "Move \(totalMarkedCount) to Trash" : "Move to Trash")
                    }
                    .fontWeight(.semibold)
                }
                .buttonStyle(.borderedProminent)
                .tint(.red)
                .disabled(totalMarkedCount == 0)
                .keyboardShortcut(.defaultAction)
            }
            .padding(.horizontal, 24)
            .padding(.vertical, 14)
            .background(Color(nsColor: .windowBackgroundColor))
        }
        .frame(minWidth: 920, idealWidth: 1040, minHeight: 560, idealHeight: 680)
        .onKeyPress(.escape) {
            viewModel.isCullDuplicatesSheetPresented = false
            return .handled
        }
    }
    
    private func formatByteCount(_ bytes: Int64) -> String {
        let bcf = ByteCountFormatter()
        bcf.allowedUnits = [.useAll]
        bcf.countStyle = .file
        return bcf.string(fromByteCount: bytes)
    }
}

// MARK: - Single Capture Set Row
@MainActor
struct CullRowView: View {
    @Bindable var row: DuplicateRowModel
    
    var body: some View {
        HStack(alignment: .top, spacing: 16) {
            // Column 1: Keep Checkbox (Defaults to empty; checking exempts row from trash)
            VStack {
                Toggle("", isOn: $row.isKept)
                    .toggleStyle(.checkbox)
                    .labelsHidden()
                    .onChange(of: row.isKept) { _, newKept in
                        for dup in row.duplicates {
                            dup.isKept = newKept
                        }
                    }
                    .help(row.isKept ? "Kept: duplicates in this capture are exempted from deletion" : "Check Keep to exempt duplicates in this capture from deletion")
            }
            .frame(width: 75, alignment: .center)
            .padding(.top, 12)
            
            // Column 2: Original Negative Card
            OriginalCardView(photo: row.original, isRowKept: row.isKept)
                .frame(width: 270)
            
            // Column 3: Duplicates (Wrapping Grid)
            LazyVGrid(
                columns: [GridItem(.adaptive(minimum: 175, maximum: 220), spacing: 10)],
                alignment: .leading,
                spacing: 10
            ) {
                ForEach(row.duplicates) { dup in
                    DuplicateCardView(
                        duplicate: dup,
                        isRowKept: row.isKept,
                        onMakeOriginal: {
                            row.swapOriginal(with: dup)
                        }
                    )
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(.horizontal, 24)
        .padding(.vertical, 10)
        .background(row.isKept ? Color.green.opacity(0.06) : Color.clear)
    }
}

// MARK: - Original Card
@MainActor
struct OriginalCardView: View {
    let photo: PhotoRecord
    let isRowKept: Bool
    
    private var formatBadge: (text: String, color: Color) {
        let ext = URL(fileURLWithPath: photo.filePath).pathExtension.uppercased()
        let rawExts = ["CR2", "CR3", "NEF", "ARW", "ORF", "RW2", "RAF", "PEF", "RAW"]
        if rawExts.contains(ext) {
            return (ext, .green)
        } else if ext == "DNG" {
            return ("DNG", .blue)
        } else {
            return (ext, .orange)
        }
    }
    
    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            ZStack(alignment: .topTrailing) {
                AsyncThumbnailView(filePath: photo.filePath)
                    .frame(height: 110)
                    .frame(maxWidth: .infinity)
                    .cornerRadius(6)
                
                Text(formatBadge.text)
                    .font(.system(size: 9, weight: .bold))
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(formatBadge.color.opacity(0.85))
                    .foregroundColor(.white)
                    .cornerRadius(4)
                    .padding(5)
            }
            
            VStack(alignment: .leading, spacing: 2) {
                HStack {
                    Text(photo.fileName)
                        .font(.caption)
                        .fontWeight(.semibold)
                        .lineLimit(1)
                        .truncationMode(.middle)
                    
                    Spacer()
                    
                    Text("ORIGINAL")
                        .font(.system(size: 8, weight: .bold))
                        .padding(.horizontal, 4)
                        .padding(.vertical, 1)
                        .background(Color.green.opacity(0.2))
                        .foregroundColor(.green)
                        .cornerRadius(3)
                }
                
                HStack {
                    Text(formatByteCount(photo.fileSize))
                        .font(.caption2)
                        .foregroundColor(.secondary)
                    
                    if let folder = URL(fileURLWithPath: photo.filePath).deletingLastPathComponent().pathComponents.last {
                        Text("•")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                        Text(folder)
                            .font(.caption2)
                            .foregroundColor(.secondary)
                            .lineLimit(1)
                    }
                }
            }
        }
        .padding(8)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(Color(nsColor: .controlBackgroundColor))
                .overlay(
                    RoundedRectangle(cornerRadius: 8)
                        .stroke(Color.green.opacity(0.35), lineWidth: 1)
                )
        )
    }
    
    private func formatByteCount(_ bytes: Int64) -> String {
        let bcf = ByteCountFormatter()
        bcf.allowedUnits = [.useAll]
        bcf.countStyle = .file
        return bcf.string(fromByteCount: bytes)
    }
}

// MARK: - Duplicate Clone Card (Wrapping)
@MainActor
struct DuplicateCardView: View {
    @Bindable var duplicate: DuplicateItemModel
    let isRowKept: Bool
    let onMakeOriginal: () -> Void
    
    /// A duplicate is kept (exempted from deletion) if either the row is kept or this item is kept
    private var isExempted: Bool {
        isRowKept || duplicate.isKept
    }
    
    private var formatBadge: (text: String, color: Color) {
        let ext = URL(fileURLWithPath: duplicate.photo.filePath).pathExtension.uppercased()
        let rawExts = ["CR2", "CR3", "NEF", "ARW", "ORF", "RW2", "RAF", "PEF", "RAW"]
        if rawExts.contains(ext) {
            return (ext, .green)
        } else if ext == "DNG" {
            return ("DNG", .blue)
        } else {
            return (ext, .orange)
        }
    }
    
    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            ZStack(alignment: .topTrailing) {
                AsyncThumbnailView(filePath: duplicate.photo.filePath)
                    .frame(height: 85)
                    .frame(maxWidth: .infinity)
                    .cornerRadius(6)
                
                HStack(spacing: 3) {
                    Text(formatBadge.text)
                        .font(.system(size: 8, weight: .bold))
                        .padding(.horizontal, 4)
                        .padding(.vertical, 1)
                        .background(formatBadge.color.opacity(0.85))
                        .foregroundColor(.white)
                        .cornerRadius(3)
                }
                .padding(4)
            }
            
            VStack(alignment: .leading, spacing: 4) {
                Text(duplicate.photo.fileName)
                    .font(.caption2)
                    .fontWeight(.medium)
                    .lineLimit(1)
                    .truncationMode(.middle)
                
                HStack {
                    Text(formatByteCount(duplicate.photo.fileSize))
                        .font(.system(size: 10))
                        .foregroundColor(.secondary)
                    
                    Spacer()
                    
                    // Keep / Trash Toggle Pill (Click to toggle exemption)
                    Button(action: {
                        duplicate.isKept.toggle()
                    }) {
                        HStack(spacing: 3) {
                            if isExempted {
                                Image(systemName: "checkmark.circle.fill")
                                    .font(.system(size: 8))
                                Text("Kept")
                                    .font(.system(size: 9, weight: .semibold))
                            } else {
                                Image(systemName: "trash")
                                    .font(.system(size: 8))
                                Text("To Trash")
                                    .font(.system(size: 9, weight: .semibold))
                            }
                        }
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(isExempted ? Color.green.opacity(0.18) : Color.red.opacity(0.18))
                        .foregroundColor(isExempted ? .green : .red)
                        .cornerRadius(4)
                    }
                    .buttonStyle(.plain)
                    .help(isExempted ? "Exempted from deletion. Click to queue for trash." : "Queued for trash. Click to keep and exempt from deletion.")
                }
                
                // Promote to Original action
                Button(action: onMakeOriginal) {
                    HStack(spacing: 3) {
                        Image(systemName: "arrow.triangle.2.circlepath")
                            .font(.system(size: 8))
                        Text("Make Original")
                            .font(.system(size: 9))
                    }
                    .foregroundColor(.accentColor)
                }
                .buttonStyle(.plain)
                .help("Set this file as the original and demote the current original")
                .padding(.top, 2)
            }
        }
        .padding(6)
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(Color(nsColor: .controlBackgroundColor).opacity(0.7))
                .overlay(
                    RoundedRectangle(cornerRadius: 6)
                        .stroke(
                            isExempted ? Color.green.opacity(0.3) : Color.red.opacity(0.35),
                            lineWidth: 1
                        )
                )
        )
    }
    
    private func formatByteCount(_ bytes: Int64) -> String {
        let bcf = ByteCountFormatter()
        bcf.allowedUnits = [.useAll]
        bcf.countStyle = .file
        return bcf.string(fromByteCount: bytes)
    }
}
