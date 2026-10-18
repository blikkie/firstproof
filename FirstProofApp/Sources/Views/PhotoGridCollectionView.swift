import SwiftUI
import AppKit

// MARK: - Photo Collection View Item
final class PhotoGridItemView: NSCollectionViewItem {
    static let reuseIdentifier = NSUserInterfaceItemIdentifier("PhotoGridItemView")
    
    private var hostingView: NSHostingView<AnyView>?
    
    func configure(with view: AnyView) {
        if let hv = hostingView {
            hv.rootView = view
        } else {
            let hv = NSHostingView(rootView: view)
            hv.translatesAutoresizingMaskIntoConstraints = false
            self.view.addSubview(hv)
            NSLayoutConstraint.activate([
                hv.leadingAnchor.constraint(equalTo: self.view.leadingAnchor),
                hv.trailingAnchor.constraint(equalTo: self.view.trailingAnchor),
                hv.topAnchor.constraint(equalTo: self.view.topAnchor),
                hv.bottomAnchor.constraint(equalTo: self.view.bottomAnchor)
            ])
            self.hostingView = hv
        }
    }
}

// MARK: - Anchored Collection View Layout
final class AnchoredGridLayout: NSCollectionViewFlowLayout {
    var onColumnsChanged: ((Int) -> Void)?
    private var anchorItemIndex: Int?
    private var lastWidth: CGFloat = 0
    private var lastCols: Int = 0
    
    override func prepare() {
        super.prepare()
        guard let cv = collectionView else { return }
        let currentWidth = cv.bounds.width
        guard currentWidth > 50 else { return }
        
        let availableWidth = max(100, currentWidth - 32)
        let targetWidth: CGFloat = 180
        let spacing: CGFloat = 12
        let cols = max(1, Int((availableWidth + spacing) / (targetWidth + spacing)))
        
        if lastWidth != currentWidth {
            let shouldAlign = !cv.inLiveResize && lastCols > 0 && lastCols != cols
            if shouldAlign {
                captureAnchor()
            }
            
            lastWidth = currentWidth
            let colsChanged = (lastCols != cols)
            lastCols = cols
            updateItemSize(for: currentWidth)
            
            if colsChanged {
                onColumnsChanged?(cols)
            }
            
            if shouldAlign {
                restoreAnchor()
            }
        }
    }
    
    func updateItemSize(for width: CGFloat) {
        let availableWidth = max(100, width - 32)
        let targetWidth: CGFloat = 180
        let spacing: CGFloat = 12
        let cols = max(1, Int((availableWidth + spacing) / (targetWidth + spacing)))
        let totalSpacing = CGFloat(cols - 1) * spacing
        let itemW = floor((availableWidth - totalSpacing) / CGFloat(cols))
        let itemH = round(itemW / 1.5 + 38)
        
        self.minimumInteritemSpacing = spacing
        self.minimumLineSpacing = spacing
        self.sectionInset = NSEdgeInsets(top: 16, left: 16, bottom: 16, right: 16)
        self.itemSize = NSSize(width: max(100, itemW), height: max(100, itemH))
    }
    
    func captureAnchor() {
        guard let cv = collectionView, let sv = cv.enclosingScrollView else { return }
        let scrollY = sv.contentView.bounds.origin.y
        if scrollY <= 24 {
            anchorItemIndex = 0
            return
        }
        
        let probePoint = NSPoint(x: sectionInset.left + 24, y: scrollY + sectionInset.top + 24)
        if let ip = cv.indexPathForItem(at: probePoint) {
            anchorItemIndex = ip.item
        } else {
            let visible = cv.indexPathsForVisibleItems()
            anchorItemIndex = visible.map(\.item).min()
        }
    }
    
    func restoreAnchor() {
        guard let index = anchorItemIndex else { return }
        anchorItemIndex = nil
        
        DispatchQueue.main.async { [weak self] in
            guard let self = self, let cv = self.collectionView, let sv = cv.enclosingScrollView else { return }
            guard index < cv.numberOfItems(inSection: 0) else { return }
            
            if index == 0 {
                sv.contentView.scroll(to: NSPoint(x: 0, y: 0))
                sv.reflectScrolledClipView(sv.contentView)
                return
            }
            
            cv.scrollToItems(at: [IndexPath(item: index, section: 0)], scrollPosition: .top)
            if let attr = cv.layoutAttributesForItem(at: IndexPath(item: index, section: 0)) {
                let targetY = max(0, attr.frame.origin.y - self.sectionInset.top)
                sv.contentView.scroll(to: NSPoint(x: 0, y: targetY))
                sv.reflectScrolledClipView(sv.contentView)
            }
        }
    }
}

// MARK: - Custom Collection View with Native Resize Tracking
final class ResizableCollectionView: NSCollectionView {
    var onResizeStart: (() -> Void)?
    var onResizeEnd: (() -> Void)?
    
    override func viewWillStartLiveResize() {
        super.viewWillStartLiveResize()
        onResizeStart?()
    }
    
    override func viewDidEndLiveResize() {
        super.viewDidEndLiveResize()
        onResizeEnd?()
    }
}

// MARK: - PhotoGridCollectionView (NSViewRepresentable)
struct PhotoGridCollectionView: NSViewRepresentable {
    let photos: [PhotoRecord]
    let selectedPhoto: PhotoRecord?
    let selectedPhotoPaths: Set<String>
    let isBulkMode: Bool
    let currentFolderPath: String?
    let canvasMode: CanvasMode
    
    let onCellClick: (PhotoRecord) -> Void
    let onToggleBulk: (PhotoRecord) -> Void
    let onDoubleClick: (PhotoRecord) -> Void
    let onExportPackage: (PhotoRecord) -> Void
    let onExportRendered: (PhotoRecord) -> Void
    let onSetStatus: (PhotoRecord, String) -> Void
    var onColumnsChanged: ((Int) -> Void)? = nil
    
    func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }
    
    func makeNSView(context: Context) -> NSScrollView {
        let scrollView = NSScrollView()
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = false
        scrollView.autohidesScrollers = true
        scrollView.drawsBackground = false
        
        let layout = AnchoredGridLayout()
        layout.updateItemSize(for: 800)
        layout.onColumnsChanged = { [weak coordinator = context.coordinator] cols in
            coordinator?.parent.onColumnsChanged?(cols)
        }
        
        let collectionView = ResizableCollectionView()
        collectionView.collectionViewLayout = layout
        collectionView.dataSource = context.coordinator
        collectionView.delegate = context.coordinator
        collectionView.isSelectable = false
        collectionView.backgroundColors = [.clear]
        
        collectionView.register(PhotoGridItemView.self, forItemWithIdentifier: PhotoGridItemView.reuseIdentifier)
        
        collectionView.onResizeStart = { [weak layout] in
            layout?.captureAnchor()
        }
        collectionView.onResizeEnd = { [weak layout] in
            layout?.restoreAnchor()
        }
        
        scrollView.documentView = collectionView
        context.coordinator.collectionView = collectionView
        context.coordinator.layout = layout
        context.coordinator.scrollView = scrollView
        
        return scrollView
    }
    
    func updateNSView(_ nsView: NSScrollView, context: Context) {
        context.coordinator.parent = self
        context.coordinator.reloadDataIfNeeded()
    }
    
    // MARK: - Coordinator
    @MainActor
    final class Coordinator: NSObject, NSCollectionViewDataSource, NSCollectionViewDelegateFlowLayout {
        var parent: PhotoGridCollectionView
        weak var collectionView: NSCollectionView?
        weak var layout: AnchoredGridLayout?
        weak var scrollView: NSScrollView?
        
        private var lastPhotoCount: Int = 0
        private var lastSelectedPath: String? = nil
        private var lastSelectedPaths: Set<String> = []
        private var lastSelectedCount: Int = 0
        private var lastBulkMode: Bool = false
        private var lastFolderPath: String? = nil
        private var lastCanvasMode: CanvasMode = .grid
        private var currentPhotos: [PhotoRecord] = []
        
        private var hasScrolledInitial: Bool = false
        private var isUserDirectClick: Bool = false
        
        init(_ parent: PhotoGridCollectionView) {
            self.parent = parent
            self.lastPhotoCount = parent.photos.count
            self.lastSelectedPath = parent.selectedPhoto?.filePath
            self.lastSelectedPaths = parent.selectedPhotoPaths
            self.lastSelectedCount = parent.selectedPhotoPaths.count
            self.lastBulkMode = parent.isBulkMode
            self.lastFolderPath = parent.currentFolderPath
            self.lastCanvasMode = parent.canvasMode
            self.currentPhotos = parent.photos
        }
        
        func reloadDataIfNeeded() {
            let count = parent.photos.count
            let path = parent.selectedPhoto?.filePath
            let selCount = parent.selectedPhotoPaths.count
            let bulk = parent.isBulkMode
            let folder = parent.currentFolderPath
            let mode = parent.canvasMode
            
            let folderChanged = folder != lastFolderPath
            if folderChanged {
                lastFolderPath = folder
                hasScrolledInitial = false
            }
            
            let countChanged = count != lastPhotoCount
            let selectionChanged = path != lastSelectedPath
            let selCountChanged = selCount != lastSelectedCount
            let bulkChanged = bulk != lastBulkMode
            let modeChanged = mode != lastCanvasMode
            let photosChanged = currentPhotos != parent.photos
            
            let structuralChanged = countChanged || bulkChanged || photosChanged || folderChanged
            let selectionOnlyChanged = !structuralChanged && (selectionChanged || selCountChanged)
            
            if structuralChanged {
                // If the photo count changed (e.g. toggling Originals Only / All Copies),
                // capture the top visible original in the viewport so the user remains
                // anchored in the exact same visual ballpark.
                let anchorPhoto = countChanged ? findTopVisiblePhoto(in: currentPhotos) : nil
                
                lastPhotoCount = count
                lastSelectedPath = path
                lastSelectedPaths = parent.selectedPhotoPaths
                lastSelectedCount = selCount
                lastBulkMode = bulk
                currentPhotos = parent.photos
                collectionView?.reloadData()
                
                if let anchor = anchorPhoto, countChanged {
                    let newPhotos = parent.photos
                    DispatchQueue.main.async { [weak self] in
                        guard let self = self, let cv = self.collectionView, let sv = self.scrollView else { return }
                        if let targetIndex = self.findBestMatchIndex(for: anchor, in: newPhotos),
                           targetIndex < cv.numberOfItems(inSection: 0) {
                            if targetIndex == 0 {
                                sv.contentView.scroll(to: NSPoint(x: 0, y: 0))
                                sv.reflectScrolledClipView(sv.contentView)
                            } else {
                                cv.scrollToItems(at: [IndexPath(item: targetIndex, section: 0)], scrollPosition: .top)
                                if let attr = cv.layoutAttributesForItem(at: IndexPath(item: targetIndex, section: 0)) {
                                    let targetY = max(0, attr.frame.origin.y - (self.layout?.sectionInset.top ?? 16))
                                    sv.contentView.scroll(to: NSPoint(x: 0, y: targetY))
                                    sv.reflectScrolledClipView(sv.contentView)
                                }
                            }
                        }
                    }
                }
            } else if selectionOnlyChanged {
                // Pinpointed in-place selection refresh: reload ONLY items whose selection state changed!
                // Prevents recycling NSCollectionViewItems across untouched cells which causes previews to flash.
                var changedPaths = lastSelectedPaths.symmetricDifference(parent.selectedPhotoPaths)
                if let oldPath = lastSelectedPath, oldPath != path {
                    changedPaths.insert(oldPath)
                }
                if let newPath = path, newPath != lastSelectedPath {
                    changedPaths.insert(newPath)
                }
                
                lastSelectedPath = path
                lastSelectedPaths = parent.selectedPhotoPaths
                lastSelectedCount = selCount
                
                if !changedPaths.isEmpty, let cv = collectionView {
                    let changedIndexPaths = Set(changedPaths.compactMap { p -> IndexPath? in
                        guard let idx = parent.photos.firstIndex(where: { $0.filePath == p }) else { return nil }
                        return IndexPath(item: idx, section: 0)
                    })
                    if !changedIndexPaths.isEmpty {
                        NSAnimationContext.beginGrouping()
                        NSAnimationContext.current.duration = 0
                        cv.reloadItems(at: changedIndexPaths)
                        NSAnimationContext.endGrouping()
                    }
                }
            }
            
            let switchedToGrid = modeChanged && mode == .grid
            lastCanvasMode = mode
            
            // 1. Initial scroll on startup / restart or folder change
            if !hasScrolledInitial && count > 0 && parent.selectedPhoto != nil {
                hasScrolledInitial = true
                DispatchQueue.main.async { [weak self] in
                    self?.scrollToSelected(centered: true)
                }
            } else if switchedToGrid && parent.selectedPhoto != nil {
                // 2. Returning from detail mode back to grid mode
                DispatchQueue.main.async { [weak self] in
                    self?.scrollToSelected(centered: true)
                }
            } else if !countChanged && selectionChanged && !isUserDirectClick && parent.selectedPhoto != nil {
                // 3. Selection change via keyboard navigation (arrow keys)
                DispatchQueue.main.async { [weak self] in
                    self?.scrollToSelected(centered: false)
                }
            }
            
            isUserDirectClick = false
        }
        
        func findTopVisiblePhoto(in photos: [PhotoRecord]) -> PhotoRecord? {
            guard let cv = collectionView, let sv = scrollView, !photos.isEmpty else { return nil }
            let scrollY = sv.contentView.bounds.origin.y
            if scrollY <= 24 {
                return photos.first
            }
            let probePoint = NSPoint(x: (layout?.sectionInset.left ?? 16) + 24, y: scrollY + (layout?.sectionInset.top ?? 16) + 24)
            if let ip = cv.indexPathForItem(at: probePoint), ip.item < photos.count {
                return photos[ip.item]
            }
            let visible = cv.indexPathsForVisibleItems()
            if let minItem = visible.map(\.item).min(), minItem < photos.count {
                return photos[minItem]
            }
            return photos.first
        }
        
        func findBestMatchIndex(for anchor: PhotoRecord, in newPhotos: [PhotoRecord]) -> Int? {
            // 1. Direct path match
            if let idx = newPhotos.firstIndex(where: { $0.filePath == anchor.filePath }) {
                return idx
            }
            
            // 2. Directory + normalized stem match (maps derivatives <-> original negatives)
            let anchorDir = URL(fileURLWithPath: anchor.filePath).deletingLastPathComponent().path
            let anchorStem = canonicalNormalizedStem(for: anchor.filePath)
            if let idx = newPhotos.firstIndex(where: {
                let dir = URL(fileURLWithPath: $0.filePath).deletingLastPathComponent().path
                let stem = canonicalNormalizedStem(for: $0.filePath)
                return dir == anchorDir && stem == anchorStem
            }) {
                return idx
            }
            
            // 3. Fallback: closest EXIF capture timestamp
            if let t = anchor.takenTime {
                let candidates = newPhotos.enumerated().filter { $0.element.takenTime != nil }
                if let closest = candidates.min(by: { abs($0.element.takenTime! - t) < abs($1.element.takenTime! - t) }) {
                    return closest.offset
                }
            }
            
            return nil
        }
        
        func scrollToSelected(centered: Bool) {
            guard let cv = collectionView, let sv = scrollView else { return }
            guard let selected = parent.selectedPhoto else { return }
            guard let index = parent.photos.firstIndex(where: { $0.filePath == selected.filePath }) else { return }
            
            // If view bounds are not yet ready (window layout still initializing), retry on next runloop cycle
            if cv.bounds.width <= 100 || sv.contentView.bounds.height <= 100 {
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) { [weak self] in
                    self?.scrollToSelected(centered: centered)
                }
                return
            }
            
            // Ensure layout attributes match current width
            layout?.updateItemSize(for: cv.bounds.width)
            
            // If collection view item count hasn't caught up with parent.photos, reload and retry
            if cv.numberOfItems(inSection: 0) < parent.photos.count {
                cv.reloadData()
                DispatchQueue.main.async { [weak self] in
                    self?.scrollToSelected(centered: centered)
                }
                return
            }
            
            guard index < cv.numberOfItems(inSection: 0) else { return }
            
            guard let attr = cv.layoutAttributesForItem(at: IndexPath(item: index, section: 0)) else {
                cv.scrollToItems(at: [IndexPath(item: index, section: 0)], scrollPosition: centered ? .centeredVertically : .bottom)
                return
            }
            
            let itemFrame = attr.frame
            let itemTop = itemFrame.origin.y
            let itemBottom = itemFrame.origin.y + itemFrame.size.height
            let visibleRect = sv.contentView.bounds
            let padding: CGFloat = 16
            
            if centered {
                if index == 0 {
                    sv.contentView.scroll(to: NSPoint(x: 0, y: 0))
                } else {
                    let targetY = max(0, itemTop - (visibleRect.size.height - itemFrame.size.height) / 2.0)
                    sv.contentView.scroll(to: NSPoint(x: 0, y: targetY))
                }
                sv.reflectScrolledClipView(sv.contentView)
            } else {
                // Follow focus: ensure item is comfortably within the visible viewport with breathing room
                if itemTop - padding < visibleRect.origin.y {
                    // Item is off the top: scroll up to reveal
                    let targetY = max(0, itemTop - padding)
                    sv.contentView.scroll(to: NSPoint(x: 0, y: targetY))
                    sv.reflectScrolledClipView(sv.contentView)
                } else if itemBottom + padding > visibleRect.origin.y + visibleRect.size.height {
                    // Item is off the bottom: scroll down to reveal
                    let contentH = max(cv.frame.height, layout?.collectionViewContentSize.height ?? cv.bounds.height)
                    let maxScrollY = max(0, contentH - visibleRect.size.height)
                    let targetY = min(maxScrollY, itemBottom + padding - visibleRect.size.height)
                    sv.contentView.scroll(to: NSPoint(x: 0, y: targetY))
                    sv.reflectScrolledClipView(sv.contentView)
                }
            }
        }
        
        // MARK: - NSCollectionViewDataSource
        func numberOfSections(in collectionView: NSCollectionView) -> Int {
            1
        }
        
        func collectionView(_ collectionView: NSCollectionView, numberOfItemsInSection section: Int) -> Int {
            parent.photos.count
        }
        
        func collectionView(_ collectionView: NSCollectionView, itemForRepresentedObjectAt indexPath: IndexPath) -> NSCollectionViewItem {
            let item = collectionView.makeItem(withIdentifier: PhotoGridItemView.reuseIdentifier, for: indexPath) as! PhotoGridItemView
            guard indexPath.item < parent.photos.count else { return item }
            let photo = parent.photos[indexPath.item]
            
            let isChecked = parent.selectedPhotoPaths.contains(photo.filePath)
            let isActive = parent.selectedPhoto?.filePath == photo.filePath
            let isSelected = parent.isBulkMode ? isChecked : (isActive || isChecked)
            
            let cellView = GridThumbnailCell(
                photo: photo,
                isSelected: isSelected,
                isActive: isActive,
                isBulkMode: parent.isBulkMode,
                action: { [weak self] in
                    self?.isUserDirectClick = true
                    self?.parent.onCellClick(photo)
                },
                toggleAction: { [weak self] in
                    self?.isUserDirectClick = true
                    self?.parent.onToggleBulk(photo)
                },
                doubleClickAction: { [weak self] in
                    self?.isUserDirectClick = true
                    self?.parent.onDoubleClick(photo)
                }
            )
            .id(photo.filePath)
            .contextMenu { [weak self] in
                Button("Export Selection as Portable Library...") {
                    self?.parent.onExportPackage(photo)
                }
                Button("Export Rendered JPEG/TIFF...") {
                    self?.parent.onExportRendered(photo)
                }
                Divider()
                Button("Mark as Pick (P)") {
                    self?.parent.onSetStatus(photo, "pick")
                }
                Button("Mark as Reject (X)") {
                    self?.parent.onSetStatus(photo, "reject")
                }
                Button("Clear Flag") {
                    self?.parent.onSetStatus(photo, "none")
                }
                Divider()
                Button("Reveal in Finder") {
                    NSWorkspace.shared.selectFile(photo.filePath, inFileViewerRootedAtPath: "")
                }
            }
            
            item.configure(with: AnyView(cellView))
            return item
        }
    }
}
