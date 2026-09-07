import SwiftUI
import AppKit
import LumenBoxBridge

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
    @Bindable public var viewModel: AppViewModel
    
    public init(viewModel: AppViewModel) {
        self.viewModel = viewModel
    }
    
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
}

public struct FlagControlsView: View {
    @Bindable public var viewModel: AppViewModel
    
    public init(viewModel: AppViewModel) {
        self.viewModel = viewModel
    }
    
    public var body: some View {
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
}

public struct BulkCullingPanel: View {
    @Bindable public var viewModel: AppViewModel
    
    public init(viewModel: AppViewModel) {
        self.viewModel = viewModel
    }
    
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
            
            RatingControlsView(viewModel: viewModel)
            FlagControlsView(viewModel: viewModel)
            
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
