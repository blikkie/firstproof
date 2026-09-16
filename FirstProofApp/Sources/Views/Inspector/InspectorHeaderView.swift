import SwiftUI
import AppKit
import FirstProofBridge

public struct PermanentHeaderCard: View {
    public let photo: PhotoRecord
    
    public init(photo: PhotoRecord) {
        self.photo = photo
    }
    
    public var body: some View {
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
}

public struct RatingControlsView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    public init() {}
    
    public var body: some View {
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
                            .font(.system(size: 13, weight: isSelected ? .bold : .regular))
                            .foregroundColor(isSelected ? .yellow : .secondary.opacity(0.6))
                            .padding(4)
                            .background(isSelected ? Color.yellow.opacity(0.15) : Color.clear)
                            .clipShape(RoundedRectangle(cornerRadius: 4))
                    }
                    .buttonStyle(.plain)
                    .help("Rate \(star) Star\(star > 1 ? "s" : "") (Key \(star))")
                }
                
                Button(action: {
                    viewModel.setRating(nil)
                }) {
                    Image(systemName: "xmark")
                        .font(.system(size: 11, weight: .medium))
                        .foregroundColor(.secondary)
                        .padding(4)
                }
                .buttonStyle(.plain)
                .help("Clear Rating (Key 0)")
            }
        }
    }
}

public struct FlagControlsView: View {
    @Environment(AppViewModel.self) private var viewModel
    
    public init() {}
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Flag:")
                .font(.caption)
                .foregroundColor(.secondary)
            
            HStack(spacing: 8) {
                let currentStatus = viewModel.selectedPhoto?.status.lowercased() ?? "none"
                let isPick = currentStatus == "pick"
                let isReject = currentStatus == "reject"
                
                Button(action: { viewModel.setStatus("pick") }) {
                    HStack(spacing: 5) {
                        Image(systemName: "flag.fill")
                            .font(.system(size: 11, weight: .semibold))
                        Text("Pick (P)")
                            .font(.caption)
                            .fontWeight(isPick ? .bold : .medium)
                    }
                    .foregroundColor(isPick ? .white : .green)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 5)
                    .background(isPick ? Color.green : Color.green.opacity(0.12))
                    .cornerRadius(6)
                    .overlay(
                        RoundedRectangle(cornerRadius: 6)
                            .stroke(isPick ? Color.green : Color.green.opacity(0.3), lineWidth: 1)
                    )
                }
                .buttonStyle(.plain)
                .help("Mark as Pick (P)")
                
                Button(action: { viewModel.setStatus("reject") }) {
                    HStack(spacing: 5) {
                        Image(systemName: "xmark.circle.fill")
                            .font(.system(size: 11, weight: .semibold))
                        Text("Reject (X)")
                            .font(.caption)
                            .fontWeight(isReject ? .bold : .medium)
                    }
                    .foregroundColor(isReject ? .white : .red)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 5)
                    .background(isReject ? Color.red : Color.red.opacity(0.12))
                    .cornerRadius(6)
                    .overlay(
                        RoundedRectangle(cornerRadius: 6)
                            .stroke(isReject ? Color.red : Color.red.opacity(0.3), lineWidth: 1)
                    )
                }
                .buttonStyle(.plain)
                .help("Mark as Reject (X)")
                
                Button(action: { viewModel.setStatus("none") }) {
                    Text("Clear (U)")
                        .font(.caption)
                        .foregroundColor(currentStatus == "none" ? .secondary : .primary)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 5)
                        .background(Color.secondary.opacity(0.08))
                        .cornerRadius(6)
                }
                .buttonStyle(.plain)
                .help("Unflag (U)")
            }
        }
    }
}

public struct BulkCullingPanel: View {
    @Environment(AppViewModel.self) private var viewModel
    
    public init() {}
    
    private var selectedCount: Int {
        viewModel.selectedPhotoPaths.count > 0 ? viewModel.selectedPhotoPaths.count : (viewModel.selectedPhoto != nil ? 1 : 0)
    }
    
    public var body: some View {
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
            
            RatingControlsView()
            FlagControlsView()
            
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
}
