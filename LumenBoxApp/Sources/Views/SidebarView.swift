import SwiftUI
import AppKit
import LumenBoxBridge

struct SidebarView: View {
    @Bindable var viewModel: AppViewModel
    @State private var hoveredFolderPath: String? = nil
    
    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            // MARK: - Projects & Folders Header
            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Text("PROJECTS & FOLDERS")
                        .font(.caption)
                        .fontWeight(.bold)
                        .foregroundColor(.secondary)
                    Spacer()
                    Button(action: selectFolder) {
                        Image(systemName: "plus.circle.fill")
                            .font(.subheadline)
                            .foregroundColor(.accentColor)
                    }
                    .buttonStyle(.plain)
                    .help("Add Project Folder to Workspace")
                }
                
                // Opened Folders List
                if viewModel.openedFolders.isEmpty {
                    Button(action: selectFolder) {
                        HStack {
                            Image(systemName: "folder.badge.plus")
                            Text("Add Folder...")
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .buttonStyle(.borderedProminent)
                    .tint(.accentColor)
                } else {
                    VStack(spacing: 4) {
                        ForEach(viewModel.openedFolders, id: \.self) { folderPath in
                            let isSelected = viewModel.currentFolderPath == folderPath
                            let isExt = folderPath.hasPrefix("/Volumes/") && !folderPath.hasPrefix("/Volumes/Macintosh HD")
                            
                            HStack(spacing: 6) {
                                Image(systemName: isExt ? "externaldrive.fill" : "folder.fill")
                                    .foregroundColor(isSelected ? .accentColor : (isExt ? .orange : .secondary))
                                    .font(.caption)
                                
                                Text(URL(fileURLWithPath: folderPath).lastPathComponent)
                                    .font(.subheadline)
                                    .fontWeight(isSelected ? .semibold : .regular)
                                    .lineLimit(1)
                                    .truncationMode(.middle)
                                
                                Spacer()
                                
                                if hoveredFolderPath == folderPath || isSelected {
                                    Button(action: {
                                        viewModel.removeFolder(at: folderPath)
                                    }) {
                                        Image(systemName: "xmark")
                                            .font(.caption2)
                                            .foregroundColor(.secondary)
                                    }
                                    .buttonStyle(.plain)
                                    .help("Remove from Workspace")
                                }
                            }
                            .padding(.horizontal, 8)
                            .padding(.vertical, 6)
                            .background(isSelected ? Color.accentColor.opacity(0.18) : (hoveredFolderPath == folderPath ? Color.secondary.opacity(0.1) : Color.clear))
                            .cornerRadius(6)
                            .contentShape(Rectangle())
                            .onTapGesture {
                                if viewModel.currentFolderPath != folderPath {
                                    viewModel.openDirectory(at: folderPath)
                                }
                            }
                            .onHover { isHovered in
                                hoveredFolderPath = isHovered ? folderPath : nil
                            }
                        }
                    }
                }
                
                // Active Scanning Progress Indicator
                if viewModel.isScanning {
                    HStack(spacing: 8) {
                        ProgressView()
                            .controlSize(.small)
                        Text(viewModel.scanningStatusMessage ?? "Scanning folder...")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                            .lineLimit(1)
                    }
                    .padding(.top, 4)
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
            viewModel.addFolder(at: url.path)
        }
    }
}
