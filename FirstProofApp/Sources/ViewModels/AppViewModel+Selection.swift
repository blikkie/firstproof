import Foundation
import SwiftUI
import AppKit
import ImageIO
import FirstProofBridge

extension AppViewModel {
    // MARK: - Photo Selection & Preview
    public func selectPhoto(_ photo: PhotoRecord) {
        self.lastNavDirection = .neutral
        self.selectedPhoto = photo
        if !isBulkSelectionMode {
            self.selectedPhotoPaths = [photo.filePath]
        }
        saveLastActivePhoto(for: currentScope, path: photo.filePath)
        loadPreview(for: photo)
    }
    
    public func setActivePhoto(_ photo: PhotoRecord) {
        self.lastNavDirection = .neutral
        self.selectedPhoto = photo
        saveLastActivePhoto(for: currentScope, path: photo.filePath)
        loadPreview(for: photo)
    }
    
    public func handleSpacebarInGrid() {
        guard !filteredPhotos.isEmpty else { return }
        
        if !isBulkSelectionMode {
            self.isBulkSelectionMode = true
            if let current = selectedPhoto {
                self.selectedPhotoPaths = [current.filePath]
            } else if let first = filteredPhotos.first {
                self.selectedPhoto = first
                self.selectedPhotoPaths = [first.filePath]
                loadPreview(for: first)
            }
        } else {
            guard let current = selectedPhoto else { return }
            if selectedPhotoPaths.contains(current.filePath) {
                selectedPhotoPaths.remove(current.filePath)
            } else {
                selectedPhotoPaths.insert(current.filePath)
            }
        }
    }
    
    public func exitBulkSelectionMode() {
        self.isBulkSelectionMode = false
        if let current = selectedPhoto {
            self.selectedPhotoPaths = [current.filePath]
        } else if let firstSelected = filteredPhotos.first(where: { selectedPhotoPaths.contains($0.filePath) }) {
            self.selectedPhoto = firstSelected
            self.selectedPhotoPaths = [firstSelected.filePath]
            loadPreview(for: firstSelected)
        }
    }
    
    public func toggleBulkSelection(for photo: PhotoRecord) {
        self.isBulkSelectionMode = true
        self.selectedPhoto = photo
        if selectedPhotoPaths.contains(photo.filePath) {
            selectedPhotoPaths.remove(photo.filePath)
        } else {
            selectedPhotoPaths.insert(photo.filePath)
        }
        saveLastActivePhoto(for: currentScope, path: photo.filePath)
        loadPreview(for: photo)
    }
    
    public func togglePhotoSelection(_ photo: PhotoRecord) {
        if selectedPhotoPaths.contains(photo.filePath) {
            selectedPhotoPaths.remove(photo.filePath)
            if selectedPhoto?.filePath == photo.filePath {
                if let next = photos.first(where: { selectedPhotoPaths.contains($0.filePath) }) {
                    self.selectedPhoto = next
                    loadPreview(for: next)
                } else {
                    self.selectedPhoto = nil
                    self.previewImage = nil
                }
            }
        } else {
            selectedPhotoPaths.insert(photo.filePath)
            self.selectedPhoto = photo
            loadPreview(for: photo)
        }
    }
    
    public func selectPhotoRange(to photo: PhotoRecord) {
        let currentList = filteredPhotos
        guard let targetIndex = currentList.firstIndex(where: { $0.filePath == photo.filePath }) else { return }
        let anchorIndex: Int
        if let anchor = selectedPhoto, let idx = currentList.firstIndex(where: { $0.filePath == anchor.filePath }) {
            anchorIndex = idx
        } else {
            anchorIndex = 0
        }
        
        let start = min(anchorIndex, targetIndex)
        let end = max(anchorIndex, targetIndex)
        
        for i in start...end {
            selectedPhotoPaths.insert(currentList[i].filePath)
        }
        self.selectedPhoto = photo
        loadPreview(for: photo)
    }
    
    public func selectAllPhotos() {
        let currentList = filteredPhotos
        self.selectedPhotoPaths = Set(currentList.map { $0.filePath })
        if selectedPhoto == nil, let first = currentList.first {
            self.selectedPhoto = first
            loadPreview(for: first)
        }
    }
    
    public func clearSelection() {
        self.isBulkSelectionMode = false
        if let current = selectedPhoto {
            self.selectedPhotoPaths = [current.filePath]
        } else {
            self.selectedPhotoPaths.removeAll()
        }
    }
    
    public func enterDetailMode() {
        resetFilmstripFollow()
        if selectedPhoto == nil, let first = filteredPhotos.first {
            selectPhoto(first)
        }
        self.canvasMode = .detail
    }
    
    public func enterGridMode() {
        self.canvasMode = .grid
    }
    
    public func selectNextPhoto() {
        self.lastNavDirection = .forward
        resetFilmstripFollow()
        let list = filteredPhotos
        guard !list.isEmpty else { return }
        guard let current = selectedPhoto, let idx = list.firstIndex(where: { $0.filePath == current.filePath }) else {
            selectPhoto(list[0])
            return
        }
        let nextIdx = min(idx + 1, list.count - 1)
        guard nextIdx != idx else { return }
        navigateToPhoto(at: nextIdx, in: list)
    }
    
    public func selectPreviousPhoto() {
        self.lastNavDirection = .backward
        resetFilmstripFollow()
        let list = filteredPhotos
        guard !list.isEmpty else { return }
        guard let current = selectedPhoto, let idx = list.firstIndex(where: { $0.filePath == current.filePath }) else {
            selectPhoto(list[0])
            return
        }
        let prevIdx = max(idx - 1, 0)
        guard prevIdx != idx else { return }
        navigateToPhoto(at: prevIdx, in: list)
    }
    
    public func moveGridSelection(deltaX: Int, deltaY: Int) {
        if deltaX > 0 || deltaY > 0 {
            self.lastNavDirection = .forward
        } else if deltaX < 0 || deltaY < 0 {
            self.lastNavDirection = .backward
        } else {
            self.lastNavDirection = .neutral
        }
        let list = filteredPhotos
        guard !list.isEmpty else { return }
        guard let current = selectedPhoto, let idx = list.firstIndex(where: { $0.filePath == current.filePath }) else {
            selectPhoto(list[0])
            return
        }
        let cols = max(1, gridColumnCount)
        let targetIdx = max(0, min(list.count - 1, idx + deltaX + (deltaY * cols)))
        guard targetIdx != idx else { return }
        navigateToPhoto(at: targetIdx, in: list)
    }
    
    func navigateToPhoto(at index: Int, in list: [PhotoRecord]) {
        let target = list[index]
        guard target.filePath != selectedPhoto?.filePath || (previewImage == nil && rawPreviewRecord == nil) else {
            return
        }
        self.selectedPhoto = target
        if !isBulkSelectionMode {
            self.selectedPhotoPaths = [target.filePath]
        }
        saveLastActivePhoto(for: currentScope, path: target.filePath)
        loadPreview(for: target)
    }
    
    public func loadPreview(for photo: PhotoRecord) {
        // Save edits and pending metadata of previously active photo before switching
        if let prevPath = currentlyLoadedPhotoPath, prevPath != photo.filePath {
            self.flushPendingMetadataSave()
            let editsSnapshot = currentEdits
            self.syncXmp(for: prevPath, rating: nil, status: nil, edits: editsSnapshot)
        }
        self.currentlyLoadedPhotoPath = photo.filePath
        self.loadMetadataForSelectedPhoto()
        
        // Resolve Lensfun lens distortion coefficients
        let resolvedCoeffs: LensCorrectionCoefficientsRecord?
        if let make = photo.cameraMake,
           let model = photo.cameraModel,
           let lens = photo.lensModel,
           let focal = photo.focalLength {
            resolvedCoeffs = self.lensfun?.resolveCoefficients(
                cameraMake: make,
                cameraModel: model,
                lensModel: lens,
                focalLength: focal,
                aperture: photo.aperture
            )
        } else {
            resolvedCoeffs = nil
        }
        
        // 1. Determine baseline EXIF camera orientation
        let exifRotation: Int
        let exifFlipH: Bool
        let exifFlipV: Bool
        if let meta = extractMetadata(path: photo.filePath), let orient = meta.orientation {
            let dec = decodeExifOrientation(orientation: orient)
            exifRotation = Int(dec.rotation)
            exifFlipH = dec.flipHorizontal
            exifFlipV = dec.flipVertical
        } else {
            exifRotation = 0
            exifFlipH = false
            exifFlipV = false
        }
        
        // 2. Resolve target edits from XMP sidecar or baseline defaults
        let targetEdits: EditSettings
        if let xmp = readXmp(path: photo.filePath) {
            targetEdits = xmp.edits
        } else {
            var defaultEdits = EditSettings.defaultIdentity
            defaultEdits.rotation = Int32(exifRotation)
            defaultEdits.flipHorizontal = exifFlipH
            defaultEdits.flipVertical = exifFlipV
            defaultEdits.sharpeningAmount = photo.isRaw ? 20.0 : 0.0
            defaultEdits.sharpeningMasking = photo.isRaw ? 15.0 : 0.0
            targetEdits = defaultEdits
        }
        
        let token = UUID()
        self.activeDecodeToken = token
        self.activeDecodeTask?.cancel()
        self.isDemosaicing = true
        
        self.currentlyLoadedPhotoPath = photo.filePath
        self.defaultExifRotation = exifRotation
        self.defaultExifFlipH = exifFlipH
        self.defaultExifFlipV = exifFlipV
        self.lensCoefficients = resolvedCoeffs
        self.applyEdits(targetEdits)
        self.isCropToolActive = false
        self.currentGainMapRecord = nil
        
        if self.selectedPhotoIsRaw {
            // RAW Workflow: 100% Honest First Proof Color. Zero Camera JPEG Flashing.
            // Tier 0: In-Memory Ring Buffer Check (0ms instant transition for pre-warmed neighbor)
            let preloadedRecord = PreviewRingBuffer.shared.getRawPreview(for: photo.filePath)
            
            if let preloaded = preloadedRecord {
                self.rawPreviewRecord = preloaded
                self.previewImage = nil
                self.isIntermediatePreviewActive = false
                self.isDemosaicing = false
            } else if let intermediate = ThumbnailCache.shared.intermediateImage(for: photo.filePath, edits: targetEdits) {
                // Tier 1: Instant First Proof Intermediate render matching targetEdits
                self.previewImage = intermediate
                self.rawPreviewRecord = nil
                self.isIntermediatePreviewActive = true
                self.isDemosaicing = false
            } else {
                // Tier 2: Neutral dark transition with developing spinner while 10-15ms decode runs
                self.previewImage = nil
                self.rawPreviewRecord = nil
                self.isIntermediatePreviewActive = false
                self.isDemosaicing = true
            }
            
            // Immediate 10-15ms RAW binned decode directly into Linear Rec.2020 (if not already preloaded)
            // Followed by full-resolution demosaic stream for 100% zoom pixel peeping
            self.activeDecodeTask = Task { [weak self] in
                guard let self = self else { return }
                guard self.activeDecodeToken == token else { return }
                
                // Stage 2A: Viewport-aware 2x2 Bayer box binning (10-15ms fast preview if uncached)
                if preloadedRecord == nil {
                    let binned = await Task.detached(priority: .userInitiated) {
                        decodeRawPreviewBinned(path: photo.filePath)
                    }.value
                    
                    guard !Task.isCancelled, self.activeDecodeToken == token else { return }
                    if let binnedRecord = binned {
                        PreviewRingBuffer.shared.storeRawPreview(binnedRecord, for: photo.filePath)
                        if self.selectedPhoto?.filePath == photo.filePath {
                            self.rawPreviewRecord = binnedRecord
                            self.previewImage = nil
                            self.isIntermediatePreviewActive = false
                            self.isDemosaicing = false
                        }
                    }
                }
                
                guard !Task.isCancelled, self.activeDecodeToken == token else { return }
                
                // Stage 2B: Full-resolution demosaic stream for 100% zoom pixel peeping
                let full = await Task.detached(priority: .userInitiated) {
                    decodeRawPreview(path: photo.filePath, maxDimension: nil)
                }.value
                
                guard !Task.isCancelled, self.activeDecodeToken == token else { return }
                if let fullRecord = full {
                    if self.selectedPhoto?.filePath == photo.filePath {
                        self.rawPreviewRecord = fullRecord
                        self.isIntermediatePreviewActive = false
                        self.isDemosaicing = false
                    }
                } else {
                    self.isDemosaicing = false
                }
            }
        } else {
            // Non-RAW Workflow: Standard master images (PNG, TIFF, JPEG)
            if let cached = ThumbnailCache.shared.image(for: photo.filePath) {
                self.previewImage = cached
                self.rawPreviewRecord = nil
                self.isIntermediatePreviewActive = false
                self.isDemosaicing = false
            } else {
                self.previewImage = nil
                self.rawPreviewRecord = nil
                self.isIntermediatePreviewActive = false
                self.isDemosaicing = true
            }
            
            self.activeDecodeTask = Task { [weak self] in
                guard let self = self else { return }
                guard self.activeDecodeToken == token else { return }
                
                let (preview, gainMap) = await Task.detached(priority: .userInitiated) { () -> (NSImage?, GainMapRecord?) in
                    var preview: NSImage? = nil
                    var gainMap: GainMapRecord? = nil
                    
                    let ext = (photo.filePath as NSString).pathExtension.lowercased()
                    if ext == "jpg" || ext == "jpeg" {
                        if hasUltraHdrGainMap(filePath: photo.filePath) {
                            gainMap = extractUltraHdrGainMap(filePath: photo.filePath)
                        }
                    }
                    
                    let url = URL(fileURLWithPath: photo.filePath) as CFURL
                    if let src = CGImageSourceCreateWithURL(url, nil) {
                        if let cgThumb = CGImageSourceCreateImageAtIndex(src, 0, nil) {
                            preview = NSImage(cgImage: cgThumb, size: NSSize(width: cgThumb.width, height: cgThumb.height))
                        }
                    }
                    return (preview, gainMap)
                }.value
                
                guard !Task.isCancelled, self.activeDecodeToken == token else { return }
                if self.selectedPhoto?.filePath == photo.filePath {
                    self.currentGainMapRecord = gainMap
                    if let finalImage = preview {
                        self.previewImage = finalImage
                        ThumbnailCache.shared.setImage(finalImage, for: photo.filePath)
                    }
                    self.rawPreviewRecord = nil
                    self.isIntermediatePreviewActive = false
                    self.isDemosaicing = false
                }
            }
        }
        
        // Trigger asynchronous directional lookahead prefetch for adjacent neighbors
        let currentList = self.filteredPhotos
        if let idx = currentList.firstIndex(where: { $0.filePath == photo.filePath }) {
            LookaheadPreloadService.shared.preloadNeighbors(
                around: idx,
                in: currentList,
                direction: self.lastNavDirection
            )
        }
    }
    
    // MARK: - Rating & Pick Status (Single & Bulk)
    public func setRating(_ rating: Int32?) {
        guard let db = db else { return }
        var targets = Array(selectedPhotoPaths)
        if targets.isEmpty, let single = selectedPhoto?.filePath {
            targets = [single]
        }
        guard !targets.isEmpty else { return }
        
        let targetPhotos = photos.filter { targets.contains($0.filePath) }
        let allMatch = targetPhotos.allSatisfy { $0.rating == rating }
        let newRating: Int32? = allMatch ? nil : rating
        
        var transitions: [RatingTransition] = []
        for path in targets {
            let oldR = photos.first(where: { $0.filePath == path })?.rating
            if oldR != newRating {
                transitions.append(RatingTransition(path: path, oldRating: oldR, newRating: newRating))
            }
        }
        
        _ = db.updateRatingStatusBatch(filePaths: targets, rating: newRating, updateRating: true, status: nil)
        
        let targetSet = Set(targets)
        for i in photos.indices {
            if targetSet.contains(photos[i].filePath) {
                photos[i].rating = newRating
            }
        }
        if let current = selectedPhoto, targetSet.contains(current.filePath) {
            selectedPhoto?.rating = newRating
        }
        
        if !transitions.isEmpty {
            let title: String
            if let r = newRating {
                let stars = String(repeating: "★", count: Int(r))
                title = targets.count > 1 ? "Set Rating \(stars) (\(targets.count) photos)" : "Set Rating \(stars)"
            } else {
                title = targets.count > 1 ? "Clear Rating (\(targets.count) photos)" : "Clear Rating"
            }
            let primaryPath = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath ?? targets.first ?? ""
            undoHistory.registerAction(title: title, photoPath: primaryPath, payload: .rating(targets: transitions))
        }
        
        let activeSnapshot = currentEdits
        let activePath = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath
        for path in targets {
            let edits = (path == activePath) ? activeSnapshot : nil
            self.syncXmp(for: path, rating: newRating, status: nil, edits: edits)
        }
    }
    
    public func setStatus(_ status: String) {
        guard let db = db else { return }
        var targets = Array(selectedPhotoPaths)
        if targets.isEmpty, let single = selectedPhoto?.filePath {
            targets = [single]
        }
        guard !targets.isEmpty else { return }
        
        let targetPhotos = photos.filter { targets.contains($0.filePath) }
        let allMatch = targetPhotos.allSatisfy { $0.status == status }
        let newStatus = allMatch ? "none" : status
        
        var transitions: [StatusTransition] = []
        for path in targets {
            let oldS = photos.first(where: { $0.filePath == path })?.status ?? "none"
            if oldS != newStatus {
                transitions.append(StatusTransition(path: path, oldStatus: oldS, newStatus: newStatus))
            }
        }
        
        _ = db.updateRatingStatusBatch(filePaths: targets, rating: nil, updateRating: false, status: newStatus)
        
        let targetSet = Set(targets)
        for i in photos.indices {
            if targetSet.contains(photos[i].filePath) {
                photos[i].status = newStatus
            }
        }
        if let current = selectedPhoto, targetSet.contains(current.filePath) {
            selectedPhoto?.status = newStatus
        }
        
        if !transitions.isEmpty {
            let statusLabel: String
            switch newStatus {
            case "pick": statusLabel = "Pick"
            case "reject": statusLabel = "Reject"
            default: statusLabel = "Unflag"
            }
            let title = targets.count > 1 ? "Set \(statusLabel) (\(targets.count) photos)" : "Set \(statusLabel)"
            let primaryPath = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath ?? targets.first ?? ""
            undoHistory.registerAction(title: title, photoPath: primaryPath, payload: .status(targets: transitions))
        }
        
        let activeSnapshot = currentEdits
        let activePath = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath
        for path in targets {
            let edits = (path == activePath) ? activeSnapshot : nil
            self.syncXmp(for: path, rating: nil, status: newStatus, edits: edits)
        }
    }
    
    func syncXmp(for filePath: String, rating: Int32??, status: String?, edits: EditSettings? = nil) {
        let currentPhoto = photos.first(where: { $0.filePath == filePath })
        let finalRating: Int32?
        if let r = rating {
            finalRating = r
        } else {
            finalRating = currentPhoto?.rating
        }
        let finalStatus = status ?? currentPhoto?.status ?? "none"
        let photoKeywords = currentPhoto?.keywords
        let photoTitle = currentPhoto?.title
        let photoDescription = currentPhoto?.description
        let photoCreator = currentPhoto?.creator
        let photoCopyright = currentPhoto?.copyright
        let photoLat = currentPhoto?.latitude
        let photoLon = currentPhoto?.longitude

        Task.detached(priority: .utility) {
            let existingXmp = readXmp(path: filePath)
            let finalEdits = edits ?? existingXmp?.edits ?? EditSettings.defaultIdentity
            let splitKeywords: [String]
            if let kw = photoKeywords {
                splitKeywords = kw.split(separator: ",").map { $0.trimmingCharacters(in: .whitespaces) }.filter { !$0.isEmpty }
            } else {
                splitKeywords = existingXmp?.keywords ?? []
            }

            let record = XmpDataRecord(
                rating: finalRating ?? existingXmp?.rating,
                status: finalStatus,
                title: photoTitle ?? existingXmp?.title,
                description: photoDescription ?? existingXmp?.description,
                creator: photoCreator ?? existingXmp?.creator,
                copyright: photoCopyright ?? existingXmp?.copyright,
                keywords: splitKeywords,
                latitude: photoLat ?? existingXmp?.latitude,
                longitude: photoLon ?? existingXmp?.longitude,
                edits: finalEdits
            )
            _ = writeXmp(path: filePath, record: record)
        }
    }
    
    // MARK: - Viewport Zoom & Pan
    public func zoomToFit() {
        withAnimation(.easeInOut(duration: 0.15)) {
            zoomScale = 1.0
            panOffset = .zero
        }
    }
    
    public func zoomTo100() {
        let target = zoomScale100 > 1.05 ? zoomScale100 : 3.0
        withAnimation(.easeInOut(duration: 0.15)) {
            zoomScale = target
            panOffset = .zero
        }
    }
    
    public func zoomTo200() {
        let base = zoomScale100 > 1.05 ? zoomScale100 : 3.0
        withAnimation(.easeInOut(duration: 0.15)) {
            zoomScale = base * 2.0
            panOffset = .zero
        }
    }
    
    public func toggleZoom(at pointInNDC: CGPoint = .zero) {
        if isZoomed {
            zoomToFit()
        } else {
            let base = zoomScale100 > 1.05 ? zoomScale100 : 3.0
            let target = base * 2.0 // Double-click targets 200% Retina Loupe
            withAnimation(.easeInOut(duration: 0.15)) {
                zoomScale = target
                panOffset = CGPoint(x: -pointInNDC.x * CGFloat(zoomScale - 1.0), y: -pointInNDC.y * CGFloat(zoomScale - 1.0))
            }
        }
    }
    
    public func zoomIn() {
        zoomScale = min(zoomScale * 1.3, max(16.0, zoomScale100 * 8.0))
    }
    
    public func zoomOut() {
        zoomScale = max(zoomScale / 1.3, 1.0)
        if zoomScale <= 1.05 {
            panOffset = .zero
        }
    }
    
    public func applyPanDelta(_ delta: CGSize) {
        panOffset.x += delta.width
        panOffset.y += delta.height
    }
    
    public func applyZoomScale(_ scale: Float) {
        zoomScale = min(max(scale, 0.5), 10.0)
        if zoomScale <= 1.05 {
            panOffset = .zero
        }
    }
}
