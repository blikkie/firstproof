import SwiftUI
import AppKit
import FirstProofBridge

struct CanvasView: View {
    @Environment(AppViewModel.self) private var viewModel
    @State private var showHdrUnavailablePopover: Bool = false
    @State private var isDropTargeted: Bool = false
    
    var body: some View {
        @Bindable var viewModel = viewModel
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
                        Text("(\(viewModel.filteredPhotos.count) Photos)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
                
                Spacer()
                
                if viewModel.canvasMode == .grid {
                    Button {
                        viewModel.toggleHideDerivativeClones()
                    } label: {
                        HStack(spacing: 5) {
                            Image(systemName: viewModel.hideDerivativeClones ? "square.stack.3d.down.right.fill" : "square.stack.3d.down.right")
                                .font(.system(size: 11, weight: .semibold))
                            Text(viewModel.hideDerivativeClones ? "Originals Only" : "All Copies")
                                .font(.system(size: 11, weight: .semibold, design: .rounded))
                        }
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(
                            Capsule()
                                .fill(viewModel.hideDerivativeClones ? Color.accentColor.opacity(0.18) : Color.secondary.opacity(0.12))
                        )
                        .overlay(
                            Capsule()
                                .stroke(viewModel.hideDerivativeClones ? Color.accentColor.opacity(0.4) : Color.clear, lineWidth: 1)
                        )
                        .foregroundColor(viewModel.hideDerivativeClones ? .primary : .secondary)
                    }
                    .buttonStyle(.plain)
                    .help(viewModel.hideDerivativeClones 
                        ? "Originals Only: Showing authentic original captures (excludes export copies like _edited.jpg, -1.dng). Click to show all files." 
                        : "Showing all files including export copies. Click to show Originals Only.")
                }
                
                if viewModel.canvasMode == .detail {
                    // Apple EDR & HDR Radiance Toggle (Story 18.1 - Detail / Loupe View only)
                    let isHdrReady = viewModel.isScreenHdrCapable && viewModel.edrHeadroom > 1.0
                    let isHdrActive = isHdrReady && viewModel.isHdrEnabled
                    
                    Button {
                        if isHdrReady {
                            withAnimation(.easeInOut(duration: 0.2)) {
                                viewModel.isHdrEnabled.toggle()
                            }
                        } else {
                            showHdrUnavailablePopover = true
                            DispatchQueue.main.asyncAfter(deadline: .now() + 4.0) {
                                showHdrUnavailablePopover = false
                            }
                        }
                    } label: {
                        HStack(spacing: 5) {
                            Image(systemName: isHdrActive ? "sun.max.trianglebadge.exclamationmark.fill" : "sun.max.fill")
                                .font(.system(size: 11, weight: .bold))
                                .foregroundColor(isHdrActive ? .yellow : .secondary)
                            
                            if isHdrReady {
                                Text(viewModel.isHdrEnabled ? String(format: "HDR %.1fx", viewModel.edrHeadroom) : "SDR")
                                    .font(.system(size: 11, weight: .semibold, design: .rounded))
                                    .foregroundColor(viewModel.isHdrEnabled ? .primary : .secondary)
                            } else {
                                Text("SDR (1.0x)")
                                    .font(.system(size: 11, weight: .semibold, design: .rounded))
                                    .foregroundColor(.secondary)
                            }
                        }
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(
                            Capsule()
                                .fill(isHdrActive ? Color.yellow.opacity(0.18) : Color.secondary.opacity(0.12))
                        )
                        .overlay(
                            Capsule()
                                .stroke(isHdrActive ? Color.yellow.opacity(0.4) : Color.clear, lineWidth: 1)
                        )
                    }
                    .buttonStyle(.plain)
                    .help(isHdrReady 
                        ? (viewModel.isHdrEnabled ? "HDR Radiance active (\(String(format: "%.1fx", viewModel.edrHeadroom)) headroom). Click for SDR soft-proofing." : "SDR soft-proofing mode. Click for HDR Radiance.")
                        : "Current display is SDR (headroom: 1.0x). Click for info on unlocking HDR.")
                    .popover(isPresented: $showHdrUnavailablePopover, arrowEdge: .bottom) {
                        VStack(alignment: .leading, spacing: 8) {
                            HStack(spacing: 6) {
                                Image(systemName: "sun.max.trianglebadge.exclamationmark.fill")
                                    .foregroundColor(.yellow)
                                    .font(.headline)
                                Text("HDR Unavailable on This Display")
                                    .font(.headline)
                            }
                            
                            Text("Your current display is running in Standard Dynamic Range (1.0× headroom), so highlights cannot physically illuminate beyond standard SDR white.")
                                .font(.caption)
                                .foregroundColor(.secondary)
                                .fixedSize(horizontal: false, vertical: true)
                            
                            Divider()
                            
                            VStack(alignment: .leading, spacing: 4) {
                                Text("To experience HDR / EDR:")
                                    .font(.caption)
                                    .fontWeight(.semibold)
                                
                                HStack(alignment: .top, spacing: 5) {
                                    Text("•").foregroundColor(.secondary)
                                    Text("Connect an Apple Liquid Retina XDR (MacBook Pro), Studio Display, or HDR10 monitor.")
                                        .font(.caption2)
                                        .foregroundColor(.secondary)
                                }
                                
                                HStack(alignment: .top, spacing: 5) {
                                    Text("•").foregroundColor(.secondary)
                                    Text("Dimming display brightness in macOS Control Center can unlock EDR headroom relative to SDR white.")
                                        .font(.caption2)
                                        .foregroundColor(.secondary)
                                }
                                
                                HStack(alignment: .top, spacing: 5) {
                                    Text("•").foregroundColor(.secondary)
                                    Text("Export with \"Ultra HDR (ISO 21496-1)\" to view the gain map on iPhone, Android, and HDR browsers.")
                                        .font(.caption2)
                                        .foregroundColor(.secondary)
                                }
                            }
                        }
                        .padding(14)
                        .frame(width: 320)
                    }
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
                    Text("No Photos in Library")
                        .font(.title2)
                        .fontWeight(.semibold)
                    Text("Drag and drop a folder or RAW files here, or open a session from the sidebar.")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .background(Color(NSColor.underPageBackgroundColor))
            } else {
                ZStack {
                    PhotoGridCollectionView(
                        photos: viewModel.filteredPhotos,
                        selectedPhoto: viewModel.selectedPhoto,
                        selectedPhotoPaths: viewModel.selectedPhotoPaths,
                        isBulkMode: viewModel.isBulkSelectionMode,
                        currentFolderPath: viewModel.currentFolderPath,
                        canvasMode: viewModel.canvasMode,
                        onCellClick: { photo in
                            handleCellClick(for: photo)
                        },
                        onToggleBulk: { photo in
                            viewModel.toggleBulkSelection(for: photo)
                        },
                        onDoubleClick: { photo in
                            viewModel.setActivePhoto(photo)
                            viewModel.enterDetailMode()
                        },
                        onExportPackage: { photo in
                            if !viewModel.selectedPhotoPaths.contains(photo.filePath) {
                                viewModel.selectPhoto(photo)
                            }
                            viewModel.isExportPackageSheetPresented = true
                        },
                        onExportRendered: { photo in
                            viewModel.selectPhoto(photo)
                            viewModel.isExportSheetPresented = true
                        },
                        onSetStatus: { photo, status in
                            if !viewModel.selectedPhotoPaths.contains(photo.filePath) && viewModel.selectedPhoto?.filePath != photo.filePath {
                                viewModel.selectPhoto(photo)
                            }
                            viewModel.setStatus(status)
                        },
                        onColumnsChanged: { cols in
                            DispatchQueue.main.async {
                                if viewModel.gridColumnCount != cols {
                                    viewModel.gridColumnCount = cols
                                }
                            }
                        }
                    )
                    .background(Color(NSColor.underPageBackgroundColor))
                    .opacity(viewModel.canvasMode == .grid ? 1 : 0)
                    .allowsHitTesting(viewModel.canvasMode == .grid)
                    
                    // 2. Detail View (Metal GPU Canvas & Docked Control Bar)
                    if viewModel.canvasMode == .detail {
                        VStack(spacing: 0) {
                            ZStack {
                                Color(NSColor.underPageBackgroundColor)
                                    .edgesIgnoringSafeArea(.all)
                                
                                if viewModel.previewImage != nil || viewModel.rawPreviewRecord != nil {
                                    DetailMetalCanvasContainer(isShowingOriginal: viewModel.isShowingOriginal)
                                        .padding(16)
                                        .shadow(color: .black.opacity(0.3), radius: 12, x: 0, y: 6)
                                        .overlay(
                                            viewModel.isShowingOriginal ?
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
                                            .padding(.top, 24) : nil
                                        )
                                        .overlay(alignment: .bottom) {
                                            if viewModel.isDemosaicing && viewModel.rawPreviewRecord == nil {
                                                HStack(spacing: 8) {
                                                    ProgressView()
                                                        .controlSize(.small)
                                                    Text(viewModel.selectedPhotoIsRaw ? "Developing 14-bit RAW..." : "Loading...")
                                                        .font(.system(size: 11, weight: .semibold))
                                                        .foregroundColor(.white)
                                                }
                                                .padding(.horizontal, 14)
                                                .padding(.vertical, 8)
                                                .background(Color(red: 0.11, green: 0.11, blue: 0.12).opacity(0.92))
                                                .cornerRadius(20)
                                                .overlay(
                                                    RoundedRectangle(cornerRadius: 20)
                                                        .stroke(Color.white.opacity(0.2), lineWidth: 1)
                                                )
                                                .shadow(color: .black.opacity(0.35), radius: 8, x: 0, y: 2)
                                                .padding(.bottom, 16)
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
                            
                            // Docked Control Bar (Never occludes the photo canvas)
                            detailControlBar
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
                                    .symbolRenderingMode(.palette)
                                    .foregroundStyle(Color.black, Color.white)
                                    .font(.system(size: 14, weight: .semibold))
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
                            .background(Color(red: 0.11, green: 0.11, blue: 0.12).opacity(0.92))
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
                    
                    // 4. Docked Background Batch Export Progress Indicator
                    if viewModel.isBatchExporting {
                        VStack {
                            Spacer()
                            HStack(spacing: 14) {
                                ProgressView(value: viewModel.batchExportProgress)
                                    .progressViewStyle(.linear)
                                    .frame(width: 120)
                                
                                VStack(alignment: .leading, spacing: 1) {
                                    Text(viewModel.batchExportStatusText)
                                        .font(.system(size: 11, weight: .semibold))
                                        .foregroundColor(.white)
                                    Text("\(viewModel.batchExportCompletedCount) of \(viewModel.batchExportTotalCount) completed")
                                        .font(.system(size: 10))
                                        .foregroundColor(.white.opacity(0.7))
                                }
                                
                                Button {
                                    viewModel.cancelBatchExport()
                                } label: {
                                    Image(systemName: "xmark.circle.fill")
                                        .font(.system(size: 14))
                                        .foregroundColor(.white.opacity(0.7))
                                }
                                .buttonStyle(.plain)
                                .help("Cancel Batch Export")
                            }
                            .padding(.horizontal, 16)
                            .padding(.vertical, 10)
                            .background(Color(red: 0.11, green: 0.11, blue: 0.12).opacity(0.95))
                            .cornerRadius(16)
                            .overlay(
                                RoundedRectangle(cornerRadius: 16)
                                    .stroke(Color.white.opacity(0.2), lineWidth: 1)
                            )
                            .shadow(color: .black.opacity(0.4), radius: 10, x: 0, y: 4)
                            .padding(.bottom, viewModel.isBulkSelectionMode ? 64 : 20)
                            .transition(.move(edge: .bottom).combined(with: .opacity))
                        }
                        .allowsHitTesting(true)
                    }
                }
            }
        }
        .overlay {
            if isDropTargeted {
                ZStack {
                    Color.black.opacity(0.45)
                    RoundedRectangle(cornerRadius: 12)
                        .strokeBorder(Color.accentColor, style: StrokeStyle(lineWidth: 3, dash: [8]))
                        .padding(16)
                    
                    VStack(spacing: 12) {
                        Image(systemName: "arrow.down.doc.fill")
                            .font(.system(size: 44))
                            .foregroundColor(.accentColor)
                        Text("Drop to Open Session or Import")
                            .font(.headline)
                            .foregroundColor(.white)
                        Text("Drop folders to open as a session, or RAW files to import")
                            .font(.subheadline)
                            .foregroundColor(.white.opacity(0.8))
                    }
                    .padding(24)
                    .background(Color(red: 0.11, green: 0.11, blue: 0.12).opacity(0.95))
                    .cornerRadius(16)
                    .shadow(radius: 20)
                }
                .allowsHitTesting(false)
                .transition(.opacity.animation(.easeInOut(duration: 0.15)))
            }
        }
        .dropDestination(for: URL.self) { urls, _ in
            NSApplication.shared.activate(ignoringOtherApps: true)
            return viewModel.handleDroppedURLs(urls)
        } isTargeted: { targeted in
            withAnimation(.easeInOut(duration: 0.15)) {
                isDropTargeted = targeted
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
            if viewModel.isBulkSelectionMode {
                viewModel.setActivePhoto(photo)
            } else {
                viewModel.selectPhoto(photo)
            }
        }
    }
    
    // MARK: - Docked Detail Control Bar
    private var detailControlBar: some View {
        HStack(spacing: 12) {
            // 1. Zoom Mode Segment (Fit / 1:1 Loupe)
            HStack(spacing: 6) {
                Button(action: {
                    viewModel.zoomToFit()
                }) {
                    Text("Fit")
                        .font(.caption)
                        .fontWeight(.medium)
                }
                .buttonStyle(.bordered)
                .tint(viewModel.zoomScale <= 1.05 ? .accentColor : nil)
                .controlSize(.small)
                .help("Fit Photo to Viewport (Cmd+0)")
                
                Button(action: {
                    viewModel.zoomTo100()
                }) {
                    Text("100%")
                        .font(.caption)
                        .fontWeight(.medium)
                }
                .buttonStyle(.bordered)
                .tint(abs(viewModel.zoomScale - viewModel.zoomScale100) < 0.1 && viewModel.isZoomed ? .accentColor : nil)
                .controlSize(.small)
                .help("Zoom to 100% Physical Pixels (Opt+Cmd+0)")
                
                Button(action: {
                    viewModel.zoomTo200()
                }) {
                    Text("200%")
                        .font(.caption)
                        .fontWeight(.medium)
                }
                .buttonStyle(.bordered)
                .tint(abs(viewModel.zoomScale - (viewModel.zoomScale100 * 2.0)) < 0.25 && viewModel.isZoomed ? .accentColor : nil)
                .controlSize(.small)
                .help("Zoom to 200% Retina Loupe (Double Click Canvas)")
                
                Text(viewModel.zoomReadoutText)
                    .font(.system(size: 11, design: .monospaced))
                    .foregroundColor(.secondary)
                    .frame(minWidth: 54, alignment: .leading)
            }
            
            Divider()
                .frame(height: 16)
            
            // 2. Quick 90-degree Rotate & Flip
            HStack(spacing: 4) {
                Button(action: {
                    viewModel.rotateCounterClockwise()
                }) {
                    Image(systemName: "rotate.left")
                        .font(.caption)
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
                .help("Rotate 90° Counter-Clockwise (Cmd+[)")
                
                Button(action: {
                    viewModel.rotateClockwise()
                }) {
                    Image(systemName: "rotate.right")
                        .font(.caption)
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
                .help("Rotate 90° Clockwise (Cmd+])")
                
                Button(action: {
                    viewModel.toggleFlipHorizontal()
                }) {
                    Image(systemName: "flip.horizontal")
                        .font(.caption)
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
                .tint(viewModel.flipHorizontal ? .accentColor : nil)
                .help("Flip Horizontal")
                
                Button(action: {
                    viewModel.toggleFlipVertical()
                }) {
                    Image(systemName: "arrow.up.and.down.righttriangle.up.righttriangle.down")
                        .font(.caption)
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
                .tint(viewModel.flipVertical ? .accentColor : nil)
                .help("Flip Vertical")
            }
            
            Divider()
                .frame(height: 16)
            
            // Quick Crop Tool Toggle
            Button(action: {
                viewModel.toggleCropTool()
            }) {
                Label(viewModel.isCropToolActive ? "Done Crop (Return)" : "Crop (C)", systemImage: "crop")
                    .font(.caption)
            }
            .buttonStyle(.bordered)
            .tint(viewModel.isCropToolActive ? .accentColor : nil)
            .controlSize(.small)
            .help("Toggle Crop & Straighten Tool (C)")
            
            Divider()
                .frame(height: 16)
            
            // 3. Before / After Comparison Peek
            Button(action: {}) {
                Label("Hold for Original (\\)", systemImage: "eye.fill")
                    .font(.caption)
            }
            .buttonStyle(.bordered)
            .controlSize(.small)
            .simultaneousGesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { _ in viewModel.isShowingOriginal = true }
                    .onEnded { _ in viewModel.isShowingOriginal = false }
            )
            
            // 4. Quick Reset All Edits
            Button(action: {
                viewModel.resetAllSliders()
            }) {
                Label("Reset", systemImage: "arrow.counterclockwise")
                    .font(.caption)
            }
            .buttonStyle(.bordered)
            .controlSize(.small)
            .disabled(!viewModel.hasCustomEdits)
            .help("Reset All Edits to As-Shot (Cmd+Shift+R)")
            
            Spacer()
            
            // 5. Image Format & Dimension Readout
            if let photo = viewModel.selectedPhoto {
                if let raw = viewModel.rawPreviewRecord {
                    Text("\(raw.width) × \(raw.height) • 14-bit RAW")
                        .font(.system(size: 11, design: .monospaced))
                        .foregroundColor(.secondary)
                } else if let img = viewModel.previewImage {
                    let w = Int(img.size.width)
                    let h = Int(img.size.height)
                    let ext = URL(fileURLWithPath: photo.filePath).pathExtension.uppercased()
                    Text("\(w) × \(h) • \(ext)")
                        .font(.system(size: 11, design: .monospaced))
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 6)
        .background(Color(NSColor.windowBackgroundColor))
        .overlay(
            Rectangle()
                .frame(height: 1)
                .foregroundColor(Color(NSColor.separatorColor)),
            alignment: .top
        )
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
        lhs.photo.status.caseInsensitiveCompare(rhs.photo.status) == .orderedSame &&
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
                    
                    let status = photo.status.lowercased()
                    if status == "pick" {
                        Image(systemName: "flag.fill")
                            .foregroundColor(.green)
                            .padding(6)
                    } else if status == "reject" {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundColor(.red)
                            .padding(6)
                    }
                }
                
                // Selection indicator badge
                if isSelected {
                    Image(systemName: "checkmark.circle.fill")
                        .symbolRenderingMode(.palette)
                        .foregroundStyle(Color.black, Color.white)
                        .font(.system(size: 16, weight: .semibold))
                        .padding(6)
                        .shadow(color: .black.opacity(0.35), radius: 2, x: 0, y: 1)
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
            
            HStack(spacing: 1) {
                if let rating = photo.rating, rating > 0 {
                    ForEach(1...Int(rating), id: \.self) { _ in
                        Image(systemName: "star.fill")
                            .font(.system(size: 8))
                            .foregroundColor(.yellow)
                    }
                } else {
                    // Empty placeholder reserving consistent height so layout never jumps
                    Image(systemName: "star")
                        .font(.system(size: 8))
                        .foregroundColor(.clear)
                }
            }
            .frame(height: 10, alignment: .leading)
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

// MARK: - Dedicated Detail Canvas Container
// Evaluated independently by SwiftUI to track live slider and orientation changes at 120 FPS
// without invalidating the parent CanvasView or the background LazyVGrid hierarchy.
struct DetailMetalCanvasContainer: View {
    @Environment(AppViewModel.self) private var viewModel
    let isShowingOriginal: Bool
    
    var body: some View {
        ZStack {
            MetalCanvasView(
                viewModel: viewModel,
                edits: viewModel.currentEdits,
                zoomScale: viewModel.zoomScale,
                panOffset: viewModel.panOffset,
                isShowingOriginal: isShowingOriginal,
                isCropToolActive: viewModel.isCropToolActive
            )
            
            if viewModel.isCropToolActive && !isShowingOriginal {
                CropOverlayView()
            }
        }
    }
}

