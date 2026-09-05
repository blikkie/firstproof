import SwiftUI
import AppKit
import LumenBoxBridge

struct CanvasView: View {
    @Bindable var viewModel: AppViewModel
    @State private var isShowingOriginal: Bool = false
    
    var body: some View {
        VStack(spacing: 0) {
            // MARK: - Toolbar Header
            HStack(spacing: 12) {
                // View Mode Picker (Grid / Detail)
                Picker("View Mode", selection: $viewModel.canvasMode) {
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
                
                if viewModel.canvasMode == .detail && viewModel.selectedPhoto != nil {
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
                    GeometryReader { geo in
                        ScrollViewReader { proxy in
                            ScrollView {
                                LazyVGrid(
                                    columns: Array(repeating: GridItem(.flexible(minimum: 140, maximum: 240), spacing: 12), count: max(1, viewModel.gridColumnCount)),
                                    spacing: 12
                                ) {
                                    ForEach(viewModel.filteredPhotos, id: \.filePath) { photo in
                                        let isChecked = viewModel.selectedPhotoPaths.contains(photo.filePath)
                                        let isActive = viewModel.selectedPhoto?.filePath == photo.filePath
                                        let isSelected = viewModel.isBulkSelectionMode ? isChecked : (isActive || isChecked)
                                        
                                        GridThumbnailCell(
                                            photo: photo,
                                            isSelected: isSelected,
                                            isActive: isActive,
                                            isBulkMode: viewModel.isBulkSelectionMode,
                                            action: {
                                                handleCellClick(for: photo)
                                            },
                                            toggleAction: {
                                                viewModel.toggleBulkSelection(for: photo)
                                            },
                                            doubleClickAction: {
                                                viewModel.setActivePhoto(photo)
                                                viewModel.enterDetailMode()
                                            }
                                        )
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
                            .opacity(viewModel.canvasMode == .grid ? 1 : 0)
                            .allowsHitTesting(viewModel.canvasMode == .grid)
                            .onChange(of: viewModel.canvasMode) { _, newMode in
                                if newMode == .grid {
                                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) {
                                        scrollToFocused(proxy: proxy, animated: true)
                                    }
                                }
                            }
                            .onChange(of: viewModel.selectedPhoto?.filePath) { _, newPath in
                                if viewModel.canvasMode == .grid && newPath != nil {
                                    scrollToFocused(proxy: proxy, animated: true)
                                }
                            }
                            .onChange(of: viewModel.photos.count) { _, count in
                                if count > 0 && viewModel.canvasMode == .grid {
                                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                                        scrollToFocused(proxy: proxy, animated: false)
                                    }
                                }
                            }
                            .onAppear {
                                updateColumns(width: geo.size.width)
                                if viewModel.canvasMode == .grid {
                                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) {
                                        scrollToFocused(proxy: proxy, animated: false)
                                    }
                                }
                            }
                            .onChange(of: geo.size.width) { _, newWidth in
                                updateColumns(width: newWidth)
                            }
                        }
                    }
                    
                    // 2. Detail View (Metal GPU Canvas)
                    if viewModel.canvasMode == .detail {
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
                                    .overlay(alignment: .bottom) {
                                        if viewModel.isDemosaicing {
                                            HStack(spacing: 8) {
                                                ProgressView()
                                                    .controlSize(.small)
                                                Text(viewModel.selectedPhotoIsRaw ? "Developing 14-bit RAW..." : "Loading...")
                                                    .font(.system(size: 11, weight: .semibold))
                                                    .foregroundColor(.white)
                                            }
                                            .padding(.horizontal, 14)
                                            .padding(.vertical, 8)
                                            .background(.ultraThinMaterial)
                                            .background(Color.black.opacity(0.65))
                                            .cornerRadius(20)
                                            .overlay(
                                                RoundedRectangle(cornerRadius: 20)
                                                    .stroke(Color.white.opacity(0.2), lineWidth: 1)
                                            )
                                            .shadow(color: .black.opacity(0.35), radius: 8, x: 0, y: 2)
                                            .padding(.bottom, 24)
                                            .transition(.opacity.animation(.easeInOut(duration: 0.15)))
                                        }
                                    }
                            } else {
                                VStack(spacing: 12) {
                                    ProgressView()
                                        .controlSize(.regular)
                                    Text(viewModel.selectedPhotoIsRaw ? "Developing 14-bit RAW..." : "Loading...")
                                        .font(.caption)
                                        .foregroundColor(.secondary)
                                }
                            }
                        }
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .transition(.opacity)
                    }
                    
                    // 3. Floating Bulk Selection Status HUD
                    if viewModel.isBulkSelectionMode && viewModel.canvasMode == .grid {
                        VStack {
                            Spacer()
                            HStack(spacing: 10) {
                                Image(systemName: "checkmark.circle.fill")
                                    .foregroundColor(.accentColor)
                                    .font(.system(size: 13))
                                Text("\(viewModel.selectedPhotoPaths.count) Selected")
                                    .font(.system(size: 12, weight: .semibold))
                                    .foregroundColor(.white)
                                
                                Divider()
                                    .frame(height: 12)
                                
                                Text("Space: toggle • Return: detail • Esc: exit")
                                    .font(.system(size: 11))
                                    .foregroundColor(.white.opacity(0.7))
                                
                                Button("Done") {
                                    viewModel.exitBulkSelectionMode()
                                }
                                .buttonStyle(.borderedProminent)
                                .controlSize(.mini)
                            }
                            .padding(.horizontal, 14)
                            .padding(.vertical, 7)
                            .background(.ultraThinMaterial)
                            .background(Color.black.opacity(0.65))
                            .cornerRadius(20)
                            .overlay(
                                RoundedRectangle(cornerRadius: 20)
                                    .stroke(Color.white.opacity(0.2), lineWidth: 1)
                            )
                            .shadow(color: .black.opacity(0.35), radius: 8, x: 0, y: 2)
                            .padding(.bottom, 20)
                            .transition(.opacity.animation(.easeInOut(duration: 0.15)))
                        }
                        .allowsHitTesting(true)
                    }
                }
            }
        }
    }
    
    private func scrollToFocused(proxy: ScrollViewProxy, animated: Bool = true) {
        guard let target = viewModel.selectedPhoto?.filePath else { return }
        DispatchQueue.main.async {
            if animated {
                withAnimation(.easeInOut(duration: 0.2)) {
                    proxy.scrollTo(target, anchor: .center)
                }
            } else {
                proxy.scrollTo(target, anchor: .center)
            }
        }
    }
    
    private func updateColumns(width: CGFloat) {
        let availableWidth = max(0, width - 32)
        let itemWidth: CGFloat = 180
        let spacing: CGFloat = 12
        let cols = max(1, Int((availableWidth + spacing) / (itemWidth + spacing)))
        if viewModel.gridColumnCount != cols {
            viewModel.gridColumnCount = cols
        }
    }
    
    private func handleCellClick(for photo: PhotoRecord) {
        let flags = NSEvent.modifierFlags
        if flags.contains(.command) {
            viewModel.togglePhotoSelection(photo)
        } else if flags.contains(.shift) {
            viewModel.selectPhotoRange(to: photo)
        } else {
            if viewModel.isBulkSelectionMode {
                viewModel.setActivePhoto(photo)
            } else {
                viewModel.selectPhoto(photo)
            }
        }
    }
}

struct GridThumbnailCell: View, Equatable {
    let photo: PhotoRecord
    let isSelected: Bool
    let isActive: Bool
    let isBulkMode: Bool
    let action: () -> Void
    let toggleAction: () -> Void
    let doubleClickAction: () -> Void
    
    static func == (lhs: GridThumbnailCell, rhs: GridThumbnailCell) -> Bool {
        lhs.photo.filePath == rhs.photo.filePath &&
        lhs.photo.rating == rhs.photo.rating &&
        lhs.photo.status == rhs.photo.status &&
        lhs.isSelected == rhs.isSelected &&
        lhs.isActive == rhs.isActive &&
        lhs.isBulkMode == rhs.isBulkMode
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
                
                // Selection indicator badge
                if isSelected {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.accentColor)
                        .background(Circle().fill(Color.white).padding(2))
                        .font(.system(size: 16))
                        .padding(6)
                        .shadow(radius: 2)
                        .onTapGesture {
                            toggleAction()
                        }
                } else if isBulkMode {
                    Image(systemName: "circle")
                        .foregroundColor(.white.opacity(0.7))
                        .background(Circle().fill(Color.black.opacity(0.35)).padding(1))
                        .font(.system(size: 16))
                        .padding(6)
                        .onTapGesture {
                            toggleAction()
                        }
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
        .background(isSelected ? Color.accentColor.opacity(0.22) : Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(
                    isActive ? (isBulkMode && !isSelected ? Color.white.opacity(0.9) : Color.accentColor) : (isSelected ? Color.accentColor.opacity(0.6) : Color.clear),
                    lineWidth: isActive ? 2.5 : 1.5
                )
        )
        .contentShape(Rectangle())
        .onTapGesture(count: 2) {
            doubleClickAction()
        }
        .simultaneousGesture(
            TapGesture().onEnded {
                action()
            }
        )
    }
}
