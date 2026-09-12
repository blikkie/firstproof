import SwiftUI
import AppKit
import FirstProofBridge

public struct LibrarySidebarView: View {
    @Bindable public var viewModel: AppViewModel
    @State private var hoveredFolderPath: String? = nil
    
    public init(viewModel: AppViewModel) {
        self.viewModel = viewModel
    }
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            // MARK: - 1. Main Library Section
            VStack(alignment: .leading, spacing: 6) {
                Text("MAIN LIBRARY")
                    .font(.caption2)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                    .padding(.horizontal, 12)
                
                let isMainSelected = viewModel.currentScope == .mainLibrary
                
                Button(action: {
                    viewModel.ratingFilter = nil
                    viewModel.statusFilter = nil
                    viewModel.selectScope(.mainLibrary)
                }) {
                    HStack(spacing: 8) {
                        Image(systemName: "photo.stack.fill")
                            .foregroundColor(isMainSelected ? .accentColor : .secondary)
                            .font(.subheadline)
                        
                        Text("All Photos")
                            .font(.subheadline)
                            .fontWeight(isMainSelected && viewModel.ratingFilter == nil && viewModel.statusFilter == nil ? .semibold : .regular)
                        
                        Spacer()
                        
                        if isMainSelected {
                            Text("\(viewModel.photos.count)")
                                .font(.caption2)
                                .foregroundColor(.secondary)
                        }
                    }
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
                    .background(isMainSelected && viewModel.ratingFilter == nil && viewModel.statusFilter == nil ? Color.accentColor.opacity(0.18) : Color.clear)
                    .cornerRadius(6)
                }
                .buttonStyle(.plain)
                .padding(.horizontal, 4)
            }
            .padding(.top, 10)
            
            Divider()
                .padding(.horizontal, 8)
            
            // MARK: - 2. Project Sessions Section
            VStack(alignment: .leading, spacing: 6) {
                HStack {
                    Text("PROJECT SESSIONS")
                        .font(.caption2)
                        .fontWeight(.bold)
                        .foregroundColor(.secondary)
                    Spacer()
                    Button(action: { viewModel.promptOpenFolder() }) {
                        Image(systemName: "plus.circle.fill")
                            .font(.subheadline)
                            .foregroundColor(.accentColor)
                    }
                    .buttonStyle(.plain)
                    .help("Open Project Folder as Portable Session (Cmd+O)")
                }
                .padding(.horizontal, 12)
                
                if viewModel.openedFolders.isEmpty {
                    Button(action: { viewModel.promptOpenFolder() }) {
                        HStack(spacing: 6) {
                            Image(systemName: "folder.badge.plus")
                            Text("Open Session...")
                        }
                        .font(.caption)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.horizontal, 10)
                        .padding(.vertical, 6)
                    }
                    .buttonStyle(.bordered)
                    .padding(.horizontal, 8)
                } else {
                    ScrollView {
                        VStack(spacing: 2) {
                            ForEach(viewModel.openedFolders, id: \.self) { folderPath in
                                SessionItemRowView(
                                    viewModel: viewModel,
                                    folderPath: folderPath,
                                    hoveredFolderPath: $hoveredFolderPath
                                )
                            }
                        }
                        .padding(.horizontal, 4)
                    }
                    .frame(maxHeight: 180)
                }
            }
            
            Divider()
                .padding(.horizontal, 8)
            
            // MARK: - 3. Filters & Culling Section
            VStack(alignment: .leading, spacing: 6) {
                Text("FILTERS")
                    .font(.caption2)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                    .padding(.horizontal, 12)
                
                // 1..5 Star Ratings
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
                                .font(.caption2)
                                .foregroundColor(.secondary)
                        }
                        .padding(.horizontal, 10)
                        .padding(.vertical, 4)
                        .background(viewModel.ratingFilter == star ? Color.yellow.opacity(0.15) : Color.clear)
                        .cornerRadius(6)
                    }
                    .buttonStyle(.plain)
                    .padding(.horizontal, 4)
                }
                
                // Pick / Reject Flags
                Button(action: {
                    viewModel.statusFilter = (viewModel.statusFilter == "pick") ? nil : "pick"
                }) {
                    HStack {
                        Image(systemName: "flag.fill")
                            .foregroundColor(.green)
                            .font(.caption)
                        Text("Picks")
                            .font(.subheadline)
                        Spacer()
                        let count = viewModel.photos.filter { $0.status == "pick" }.count
                        Text("\(count)")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                    .padding(.horizontal, 10)
                    .padding(.vertical, 4)
                    .background(viewModel.statusFilter == "pick" ? Color.green.opacity(0.15) : Color.clear)
                    .cornerRadius(6)
                }
                .buttonStyle(.plain)
                .padding(.horizontal, 4)
                
                Button(action: {
                    viewModel.statusFilter = (viewModel.statusFilter == "reject") ? nil : "reject"
                }) {
                    HStack {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundColor(.red)
                            .font(.caption)
                        Text("Rejects")
                            .font(.subheadline)
                        Spacer()
                        let count = viewModel.photos.filter { $0.status == "reject" }.count
                        Text("\(count)")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                    .padding(.horizontal, 10)
                    .padding(.vertical, 4)
                    .background(viewModel.statusFilter == "reject" ? Color.red.opacity(0.15) : Color.clear)
                    .cornerRadius(6)
                }
                .buttonStyle(.plain)
                .padding(.horizontal, 4)
            }
            
            Spacer()
            
            // MARK: - Footer Scanning Indicator
            if viewModel.isScanning {
                HStack(spacing: 8) {
                    ProgressView()
                        .controlSize(.small)
                    Text(viewModel.scanningStatusMessage ?? "Scanning...")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                        .lineLimit(1)
                }
                .padding(.horizontal, 12)
                .padding(.bottom, 8)
            }
        }
        .frame(minWidth: 200, idealWidth: 220, maxWidth: 260)
        .background(Color(NSColor.controlBackgroundColor))
    }
}

private struct SessionItemRowView: View {
    var viewModel: AppViewModel
    let folderPath: String
    @Binding var hoveredFolderPath: String?
    
    private var isAvailable: Bool {
        FileManager.default.fileExists(atPath: folderPath)
    }
    
    private var isSelected: Bool {
        viewModel.currentScope == .session(folderPath: folderPath)
    }
    
    private var isExt: Bool {
        folderPath.hasPrefix("/Volumes/") && !folderPath.hasPrefix("/Volumes/Macintosh HD")
    }
    
    private var folderName: String {
        URL(fileURLWithPath: folderPath).lastPathComponent
    }
    
    var body: some View {
        HStack(spacing: 8) {
            if !isAvailable {
                Image(systemName: "externaldrive.badge.xmark")
                    .foregroundColor(.orange)
                    .font(.caption)
            } else {
                Image(systemName: isExt ? "externaldrive.fill" : "folder.fill")
                    .foregroundColor(isSelected ? .accentColor : .secondary)
                    .font(.caption)
            }
            
            VStack(alignment: .leading, spacing: 1) {
                Text(folderName)
                    .font(.subheadline)
                    .fontWeight(isSelected ? .semibold : .regular)
                    .lineLimit(1)
                
                if !isAvailable {
                    Text("Drive Disconnected")
                        .font(.caption2)
                        .foregroundColor(.orange)
                }
            }
            
            Spacer()
            
            if hoveredFolderPath == folderPath {
                Button(action: {
                    viewModel.removeFolder(at: folderPath)
                }) {
                    Image(systemName: "xmark.circle")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
                .help("Remove Session from Sidebar")
            }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(isSelected ? Color.accentColor.opacity(0.18) : (hoveredFolderPath == folderPath ? Color.secondary.opacity(0.08) : Color.clear))
        .cornerRadius(6)
        .contentShape(Rectangle())
        .onTapGesture {
            if isAvailable {
                viewModel.selectScope(.session(folderPath: folderPath))
            }
        }
        .onHover { isHovered in
            hoveredFolderPath = isHovered ? folderPath : nil
        }
        .contextMenu {
            Button("Reveal in Finder") {
                NSWorkspace.shared.selectFile(nil, inFileViewerRootedAtPath: folderPath)
            }
            Divider()
            Button("Close Session") {
                viewModel.removeFolder(at: folderPath)
            }
        }
    }
}

