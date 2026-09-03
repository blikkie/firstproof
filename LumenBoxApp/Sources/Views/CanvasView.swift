import SwiftUI
import AppKit
import LumenBoxBridge

enum CanvasMode {
    case grid
    case detail
}

struct CanvasView: View {
    @Bindable var viewModel: AppViewModel
    @State private var mode: CanvasMode = .grid
    @State private var isShowingOriginal: Bool = false
    
    var body: some View {
        VStack(spacing: 0) {
            // MARK: - Toolbar Header
            HStack(spacing: 12) {
                // View Mode Picker (Grid / Detail)
                Picker("View Mode", selection: $mode) {
                    Label("Grid", systemImage: "square.grid.3x3.fill").tag(CanvasMode.grid)
                    Label("Detail", systemImage: "photo.fill").tag(CanvasMode.detail)
                }
                .pickerStyle(.segmented)
                .frame(width: 150)
                
                // Active Scope Title & Photo Count
                HStack(spacing: 6) {
                    switch viewModel.currentScope {
                    case .mainLibrary:
                        Image(systemName: "photo.stack.fill")
                            .foregroundColor(.accentColor)
                            .font(.caption)
                        Text("Main Library")
                            .font(.subheadline)
                            .fontWeight(.semibold)
                    case .session(let folderPath):
                        let isExt = folderPath.hasPrefix("/Volumes/") && !folderPath.hasPrefix("/Volumes/Macintosh HD")
                        Image(systemName: isExt ? "externaldrive.fill" : "folder.fill")
                            .foregroundColor(isExt ? .orange : .accentColor)
                            .font(.caption)
                        Text(URL(fileURLWithPath: folderPath).lastPathComponent)
                            .font(.subheadline)
                            .fontWeight(.semibold)
                    }
                    
                    if viewModel.selectedPhotoPaths.count > 1 {
                        Text("(\(viewModel.filteredPhotos.count) • \(viewModel.selectedPhotoPaths.count) Selected)")
                            .font(.caption)
                            .foregroundColor(.accentColor)
                            .fontWeight(.medium)
                    } else {
                        Text("(\(viewModel.filteredPhotos.count))")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
                
                Spacer()
                
                if viewModel.selectedPhotoPaths.count > 1 || viewModel.selectedPhoto != nil {
                    Button(action: {
                        viewModel.isExportPackageSheetPresented = true
                    }) {
                        Label(viewModel.selectedPhotoPaths.count > 1 ? "Package (\(viewModel.selectedPhotoPaths.count))" : "Package", systemImage: "shippingbox.fill")
                    }
                    .buttonStyle(.bordered)
                    .help("Export Selection as Portable Library (Cmd+Shift+E)")
                }
                
                if mode == .detail && viewModel.selectedPhoto != nil {
                    Button(action: {}) {
                        Label("Hold for Original (\\)", systemImage: "eye.fill")
                    }
                    .buttonStyle(.bordered)
                    .simultaneousGesture(
                        DragGesture(minimumDistance: 0)
                            .onChanged { _ in isShowingOriginal = true }
                            .onEnded { _ in isShowingOriginal = false }
                    )
                }
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
            .background(Color(NSColor.windowBackgroundColor))
            
            Divider()
            
            // MARK: - Main Canvas Content
            if viewModel.photos.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "photo.on.rectangle.angled")
                        .font(.system(size: 48))
                        .foregroundColor(.secondary)
                    Text("No Photos Selected")
                        .font(.title2)
                        .fontWeight(.semibold)
                    Text("Open a directory from the left sidebar to start indexing RAW photos.")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .background(Color(NSColor.underPageBackgroundColor))
            } else {
                ZStack {
                    // 1. Persistent Grid View (Maintains exact scroll position in memory)
                    ScrollViewReader { proxy in
                        ScrollView {
                            LazyVGrid(columns: [GridItem(.adaptive(minimum: 160, maximum: 220), spacing: 12)], spacing: 12) {
                                ForEach(viewModel.filteredPhotos, id: \.filePath) { photo in
                                    let isSelected = viewModel.selectedPhotoPaths.contains(photo.filePath) || (viewModel.selectedPhotoPaths.isEmpty && viewModel.selectedPhoto?.filePath == photo.filePath)
                                    GridThumbnailCell(
                                        photo: photo,
                                        isSelected: isSelected,
                                        isMultiSelectActive: viewModel.selectedPhotoPaths.count > 1
                                    ) {
                                        handleCellClick(for: photo)
                                    }
                                    .id(photo.filePath)
                                    .contextMenu {
                                        Button("Export Selection as Portable Library...") {
                                            if !viewModel.selectedPhotoPaths.contains(photo.filePath) {
                                                viewModel.selectPhoto(photo)
                                            }
                                            viewModel.isExportPackageSheetPresented = true
                                        }
                                        Button("Export Rendered JPEG/TIFF...") {
                                            viewModel.selectPhoto(photo)
                                            viewModel.isExportSheetPresented = true
                                        }
                                        Divider()
                                        Button("Mark as Pick (P)") {
                                            viewModel.setStatus("pick")
                                        }
                                        Button("Mark as Reject (X)") {
                                            viewModel.setStatus("reject")
                                        }
                                        Button("Clear Flag") {
                                            viewModel.setStatus("none")
                                        }
                                        Divider()
                                        Button("Reveal in Finder") {
                                            NSWorkspace.shared.selectFile(photo.filePath, inFileViewerRootedAtPath: "")
                                        }
                                    }
                                }
                            }
                            .padding(16)
                        }
                        .background(Color(NSColor.underPageBackgroundColor))
                        .opacity(mode == .grid ? 1 : 0)
                        .allowsHitTesting(mode == .grid)
                        .onChange(of: mode) { _, newMode in
                            if newMode == .grid, let selected = viewModel.selectedPhoto {
                                proxy.scrollTo(selected.filePath, anchor: .center)
                            }
                        }
                    }
                    
                    // 2. Detail View (Metal GPU Canvas)
                    if mode == .detail {
                        ZStack {
                            Color(NSColor.underPageBackgroundColor)
                                .edgesIgnoringSafeArea(.all)
                            
                            if viewModel.previewImage != nil {
                                MetalCanvasView(viewModel: viewModel, isShowingOriginal: isShowingOriginal)
                                    .padding(24)
                                    .shadow(color: .black.opacity(0.3), radius: 12, x: 0, y: 6)
                                    .overlay(
                                        isShowingOriginal ?
                                        VStack {
                                            Text("ORIGINAL RAW (AS SHOT)")
                                                .font(.caption)
                                                .fontWeight(.bold)
                                                .padding(.horizontal, 12)
                                                .padding(.vertical, 6)
                                                .background(Color.black.opacity(0.8))
                                                .foregroundColor(.white)
                                                .cornerRadius(4)
                                            Spacer()
                                        }
                                        .padding(.top, 32) : nil
                                    )
                            } else {
                                ProgressView("Loading RAW Preview...")
                            }
                        }
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .transition(.opacity)
                    }
                }
            }
        }
    }
    
    private func handleCellClick(for photo: PhotoRecord) {
        let flags = NSEvent.modifierFlags
        if flags.contains(.command) {
            viewModel.togglePhotoSelection(photo)
        } else if flags.contains(.shift) {
            viewModel.selectPhotoRange(to: photo)
        } else {
            viewModel.selectPhoto(photo)
        }
    }
}

struct GridThumbnailCell: View, Equatable {
    let photo: PhotoRecord
    let isSelected: Bool
    let isMultiSelectActive: Bool
    let action: () -> Void
    
    static func == (lhs: GridThumbnailCell, rhs: GridThumbnailCell) -> Bool {
        lhs.photo.filePath == rhs.photo.filePath &&
        lhs.photo.rating == rhs.photo.rating &&
        lhs.photo.status == rhs.photo.status &&
        lhs.isSelected == rhs.isSelected &&
        lhs.isMultiSelectActive == rhs.isMultiSelectActive
    }
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            ZStack(alignment: .topLeading) {
                ZStack(alignment: .bottomTrailing) {
                    AsyncThumbnailView(filePath: photo.filePath)
                        .aspectRatio(3.0 / 2.0, contentMode: .fit)
                        .frame(maxWidth: .infinity)
                        .clipShape(RoundedRectangle(cornerRadius: 6))
                    
                    if photo.status == "pick" {
                        Image(systemName: "flag.fill")
                            .foregroundColor(.green)
                            .padding(6)
                    } else if photo.status == "reject" {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundColor(.red)
                            .padding(6)
                    }
                }
                
                // Multi-select or selected badge
                if isSelected {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.accentColor)
                        .background(Circle().fill(Color.white).padding(2))
                        .font(.system(size: 16))
                        .padding(6)
                        .shadow(radius: 2)
                }
            }
            
            Text(URL(fileURLWithPath: photo.filePath).lastPathComponent)
                .font(.caption)
                .lineLimit(1)
            
            if let rating = photo.rating, rating > 0 {
                HStack(spacing: 1) {
                    ForEach(1...Int(rating), id: \.self) { _ in
                        Image(systemName: "star.fill")
                            .font(.system(size: 8))
                            .foregroundColor(.yellow)
                    }
                }
            }
        }
        .padding(8)
        .background(isSelected ? Color.accentColor.opacity(0.25) : Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(isSelected ? Color.accentColor : Color.clear, lineWidth: 2)
        )
        .contentShape(Rectangle())
        .onTapGesture(perform: action)
    }
}
