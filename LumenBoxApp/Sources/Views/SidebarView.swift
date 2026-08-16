import SwiftUI
import AppKit
import LumenBoxBridge

struct SidebarView: View {
    @Bindable var viewModel: AppViewModel
    
    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            // MARK: - Header & Open Folder Action
            VStack(alignment: .leading, spacing: 8) {
                Text("LIBRARY")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                
                Button(action: selectFolder) {
                    HStack {
                        Image(systemName: "folder.badge.plus")
                        Text(viewModel.currentFolderPath == nil ? "Open Directory..." : "Change Directory...")
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .buttonStyle(.borderedProminent)
                .tint(.accentColor)
                
                if let path = viewModel.currentFolderPath {
                    HStack(spacing: 6) {
                        Image(systemName: viewModel.isExternalVolume ? "externaldrive.fill" : "macbook")
                            .foregroundColor(viewModel.isExternalVolume ? .orange : .blue)
                        Text(URL(fileURLWithPath: path).lastPathComponent)
                            .font(.subheadline)
                            .fontWeight(.medium)
                            .lineLimit(1)
                    }
                    .padding(.top, 4)
                    
                    if viewModel.isExternalVolume {
                        HStack(spacing: 4) {
                            Image(systemName: "checkmark.seal.fill")
                                .font(.caption)
                                .foregroundColor(.green)
                            Text("Portable Catalog (.lumenbox)")
                                .font(.caption2)
                                .foregroundColor(.secondary)
                        }
                    }
                }
            }
            .padding(.horizontal, 12)
            .padding(.top, 12)
            
            Divider()
            
            // MARK: - Rating Filters
            VStack(alignment: .leading, spacing: 8) {
                Text("FILTERS")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                    .padding(.horizontal, 12)
                
                Button(action: { viewModel.ratingFilter = nil }) {
                    HStack {
                        Image(systemName: "photo.stack")
                        Text("All Photos")
                        Spacer()
                        Text("\(viewModel.photos.count)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 4)
                    .background(viewModel.ratingFilter == nil && viewModel.statusFilter == nil ? Color.accentColor.opacity(0.15) : Color.clear)
                    .cornerRadius(6)
                }
                .buttonStyle(.plain)
                
                // Star ratings
                ForEach(1...5, id: \.self) { star in
                    Button(action: {
                        viewModel.ratingFilter = (viewModel.ratingFilter == star) ? nil : star
                    }) {
                        HStack {
                            HStack(spacing: 2) {
                                ForEach(1...star, id: \.self) { _ in
                                    Image(systemName: "star.fill")
                                        .font(.caption2)
                                        .foregroundColor(.yellow)
                                }
                            }
                            Spacer()
                            let count = viewModel.photos.filter { $0.rating == Int32(star) }.count
                            Text("\(count)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                        .padding(.horizontal, 12)
                        .padding(.vertical, 4)
                        .background(viewModel.ratingFilter == star ? Color.yellow.opacity(0.15) : Color.clear)
                        .cornerRadius(6)
                    }
                    .buttonStyle(.plain)
                }
                
                // Pick / Reject Flags
                Button(action: {
                    viewModel.statusFilter = (viewModel.statusFilter == "pick") ? nil : "pick"
                }) {
                    HStack {
                        Image(systemName: "flag.fill")
                            .foregroundColor(.green)
                        Text("Picks")
                        Spacer()
                        let count = viewModel.photos.filter { $0.status == "pick" }.count
                        Text("\(count)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 4)
                    .background(viewModel.statusFilter == "pick" ? Color.green.opacity(0.15) : Color.clear)
                    .cornerRadius(6)
                }
                .buttonStyle(.plain)
                
                Button(action: {
                    viewModel.statusFilter = (viewModel.statusFilter == "reject") ? nil : "reject"
                }) {
                    HStack {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundColor(.red)
                        Text("Rejects")
                        Spacer()
                        let count = viewModel.photos.filter { $0.status == "reject" }.count
                        Text("\(count)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 4)
                    .background(viewModel.statusFilter == "reject" ? Color.red.opacity(0.15) : Color.clear)
                    .cornerRadius(6)
                }
                .buttonStyle(.plain)
            }
            
            Spacer()
        }
        .frame(minWidth: 200, idealWidth: 220, maxWidth: 260)
        .background(Color(NSColor.controlBackgroundColor))
    }
    
    private func selectFolder() {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.prompt = "Select Directory"
        
        if panel.runModal() == .OK, let url = panel.url {
            viewModel.openDirectory(at: url.path)
        }
    }
}
