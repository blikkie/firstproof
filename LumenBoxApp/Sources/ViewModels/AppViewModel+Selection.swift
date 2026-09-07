import Foundation
import SwiftUI
import AppKit
import ImageIO
import LumenBoxBridge

extension AppViewModel {
    // MARK: - Photo Selection & Preview
    public func selectPhoto(_ photo: PhotoRecord) {
        self.selectedPhoto = photo
        if !isBulkSelectionMode {
            self.selectedPhotoPaths = [photo.filePath]
        }
        saveLastActivePhoto(for: currentScope, path: photo.filePath)
        loadPreview(for: photo)
    }
    
    public func setActivePhoto(_ photo: PhotoRecord) {
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
        resetFilmstripFollow()
        let list = filteredPhotos
        guard !list.isEmpty else { return }
        guard let current = selectedPhoto, let idx = list.firstIndex(where: { $0.filePath == current.filePath }) else {
            selectPhoto(list[0])
            return
        }
        let nextIdx = min(idx + 1, list.count - 1)
        navigateToPhoto(at: nextIdx, in: list)
    }
    
    public func selectPreviousPhoto() {
        resetFilmstripFollow()
        let list = filteredPhotos
        guard !list.isEmpty else { return }
        guard let current = selectedPhoto, let idx = list.firstIndex(where: { $0.filePath == current.filePath }) else {
            selectPhoto(list[0])
            return
        }
        let prevIdx = max(idx - 1, 0)
        navigateToPhoto(at: prevIdx, in: list)
    }
    
    public func moveGridSelection(deltaX: Int, deltaY: Int) {
        let list = filteredPhotos
        guard !list.isEmpty else { return }
        guard let current = selectedPhoto, let idx = list.firstIndex(where: { $0.filePath == current.filePath }) else {
            selectPhoto(list[0])
            return
        }
        let cols = max(1, gridColumnCount)
        let targetIdx = max(0, min(list.count - 1, idx + deltaX + (deltaY * cols)))
        navigateToPhoto(at: targetIdx, in: list)
    }
    
    func navigateToPhoto(at index: Int, in list: [PhotoRecord]) {
        let target = list[index]
        self.selectedPhoto = target
        if !isBulkSelectionMode {
            self.selectedPhotoPaths = [target.filePath]
        }
        saveLastActivePhoto(for: currentScope, path: target.filePath)
        loadPreview(for: target)
    }
    
    public func loadPreview(for photo: PhotoRecord) {
        // Save edits of previously active photo before switching
        if let prevPath = currentlyLoadedPhotoPath, prevPath != photo.filePath {
            let editsSnapshot = currentEdits
            DispatchQueue.global(qos: .utility).async { [weak self] in
                self?.syncXmp(for: prevPath, rating: nil, status: nil, edits: editsSnapshot)
            }
        }
        self.currentlyLoadedPhotoPath = photo.filePath
        
        // Resolve Lensfun lens distortion coefficients
        if let make = photo.cameraMake,
           let model = photo.cameraModel,
           let lens = photo.lensModel,
           let focal = photo.focalLength {
            self.lensCoefficients = self.lensfun?.resolveCoefficients(
                cameraMake: make,
                cameraModel: model,
                lensModel: lens,
                focalLength: focal,
                aperture: photo.aperture
            )
        } else {
            self.lensCoefficients = nil
        }
        
        let token = UUID()
        self.activeDecodeToken = token
        self.isDemosaicing = true
        
        self.rawPreviewRecord = nil
        self.currentGainMapRecord = nil
        
        // Fast switch: immediately present cached thumbnail so user can scrub instantly
        if let cached = ThumbnailCache.shared.image(for: photo.filePath) {
            self.previewImage = cached
        } else {
            self.previewImage = nil
        }
        
        // Decode in background: extract clean lower-res camera JPEG preview
        DispatchQueue.global(qos: .userInteractive).async { [weak self] in
            guard let self = self else { return }
            guard self.activeDecodeToken == token else { return }
            
            var preview: NSImage? = nil
            var gainMap: GainMapRecord? = nil
            
            if !self.selectedPhotoIsRaw {
                let ext = (photo.filePath as NSString).pathExtension.lowercased()
                if ext == "jpg" || ext == "jpeg" {
                    if hasUltraHdrGainMap(filePath: photo.filePath) {
                        gainMap = extractUltraHdrGainMap(filePath: photo.filePath)
                    }
                }
            }
            
            // 1. Fast Embedded Camera JPEG Preview (<15ms, exact camera color profile)
            if let previewData = extractPreview(path: photo.filePath) {
                if let src = CGImageSourceCreateWithData(previewData as CFData, nil) {
                    let options: [CFString: Any] = [
                        kCGImageSourceCreateThumbnailFromImageAlways: true,
                        kCGImageSourceCreateThumbnailWithTransform: true,
                        kCGImageSourceThumbnailMaxPixelSize: 2560,
                        kCGImageSourceShouldCacheImmediately: true
                    ]
                    if let cgThumb = CGImageSourceCreateThumbnailAtIndex(src, 0, options as CFDictionary) {
                        preview = NSImage(cgImage: cgThumb, size: NSSize(width: cgThumb.width, height: cgThumb.height))
                    }
                }
            }
            
            // 2. Fallback for non-RAW master images (PNG, TIFF, JPEG)
            if preview == nil && !self.selectedPhotoIsRaw {
                let url = URL(fileURLWithPath: photo.filePath) as CFURL
                if let src = CGImageSourceCreateWithURL(url, nil) {
                    if let cgThumb = CGImageSourceCreateImageAtIndex(src, 0, nil) {
                        preview = NSImage(cgImage: cgThumb, size: NSSize(width: cgThumb.width, height: cgThumb.height))
                    }
                }
            }
            
            DispatchQueue.main.async {
                if self.activeDecodeToken == token && self.selectedPhoto?.filePath == photo.filePath {
                    self.currentGainMapRecord = gainMap
                    if let finalImage = preview {
                        if !(self.selectedPhotoIsRaw && !self.isDemosaicing) {
                            self.previewImage = finalImage
                        }
                        ThumbnailCache.shared.setImage(finalImage, for: photo.filePath)
                    }
                    if !self.selectedPhotoIsRaw {
                        self.isDemosaicing = false
                    }
                }
            }
            
            // 3. Stage 2 Progressive Hybrid Promotion: Stream linear RAW half-float buffer to GPU
            if self.selectedPhotoIsRaw {
                DispatchQueue.global(qos: .userInitiated).async { [weak self] in
                    guard let self = self else { return }
                    guard self.activeDecodeToken == token else { return }
                    
                    if let rawRecord = decodeRawPreview(path: photo.filePath, maxDimension: nil) {
                        DispatchQueue.main.async {
                            if self.activeDecodeToken == token && self.selectedPhoto?.filePath == photo.filePath {
                                self.rawPreviewRecord = rawRecord
                                self.isDemosaicing = false
                            }
                        }
                        return
                    }
                    
                    DispatchQueue.main.async {
                        if self.activeDecodeToken == token {
                            self.isDemosaicing = false
                        }
                    }
                }
            }
        }
        
        // Determine baseline EXIF camera orientation
        if let meta = extractMetadata(path: photo.filePath), let orient = meta.orientation {
            let dec = decodeExifOrientation(orientation: orient)
            self.defaultExifRotation = Int(dec.rotation)
            self.defaultExifFlipH = dec.flipHorizontal
            self.defaultExifFlipV = dec.flipVertical
        } else {
            self.defaultExifRotation = 0
            self.defaultExifFlipH = false
            self.defaultExifFlipV = false
        }
        
        // Sync XMP edit sliders if sidecar exists
        if let xmp = readXmp(path: photo.filePath) {
            self.applyEdits(xmp.edits)
        } else {
            let defaultEdits = EditSettings(
                exposure: 0.0,
                contrast: 0.0,
                highlights: 0.0,
                shadows: 0.0,
                whites: 0.0,
                blacks: 0.0,
                vibrance: 0.0,
                saturation: 0.0,
                deepBlue: 0.0,
                foliage: 0.0,
                temperature: 5500.0,
                tint: 0.0,
                cropLeft: 0.0,
                cropTop: 0.0,
                cropRight: 1.0,
                cropBottom: 1.0,
                cropAngle: 0.0,
                perspectiveVertical: 0.0,
                perspectiveHorizontal: 0.0,
                cropConstrain: true,
                rotation: Int32(self.defaultExifRotation),
                flipHorizontal: self.defaultExifFlipH,
                flipVertical: self.defaultExifFlipV,
                toneProfile: .agxVivid,
                lutName: nil,
                lutIntensity: 1.0,
                sharpeningAmount: photo.isRaw ? 20.0 : 0.0,
                sharpeningRadius: 1.0,
                sharpeningMasking: photo.isRaw ? 15.0 : 0.0,
                noiseReduction: 0.0,
                curves: .defaultIdentity
            )
            self.applyEdits(defaultEdits)
            self.isCropToolActive = false
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
        
        let activeSnapshot = currentEdits
        let activePath = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath
        DispatchQueue.global(qos: .utility).async { [weak self] in
            guard let self = self else { return }
            for path in targets {
                let edits = (path == activePath) ? activeSnapshot : nil
                self.syncXmp(for: path, rating: newRating, status: nil, edits: edits)
            }
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
        
        let activeSnapshot = currentEdits
        let activePath = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath
        DispatchQueue.global(qos: .utility).async { [weak self] in
            guard let self = self else { return }
            for path in targets {
                let edits = (path == activePath) ? activeSnapshot : nil
                self.syncXmp(for: path, rating: nil, status: newStatus, edits: edits)
            }
        }
    }
    
    func syncXmp(for filePath: String, rating: Int32??, status: String?, edits: EditSettings? = nil) {
        let existingXmp = readXmp(path: filePath)
        
        let currentPhoto = photos.first(where: { $0.filePath == filePath })
        let finalRating: Int32?
        if let r = rating {
            finalRating = r
        } else {
            finalRating = currentPhoto?.rating
        }
        let finalStatus = status ?? currentPhoto?.status ?? "none"
        
        let finalEdits = edits ?? existingXmp?.edits ?? EditSettings(
            exposure: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
            vibrance: 0.0,
            saturation: 0.0,
            deepBlue: 0.0,
            foliage: 0.0,
            temperature: 5500.0,
            tint: 0.0,
            cropLeft: 0.0,
            cropTop: 0.0,
            cropRight: 1.0,
            cropBottom: 1.0,
            cropAngle: 0.0,
            perspectiveVertical: 0.0,
            perspectiveHorizontal: 0.0,
            cropConstrain: true,
            rotation: 0,
            flipHorizontal: false,
            flipVertical: false,
            toneProfile: .agxVivid,
            lutName: nil,
            lutIntensity: 1.0,
            sharpeningAmount: 0.0,
            sharpeningRadius: 1.0,
            sharpeningMasking: 0.0,
            noiseReduction: 0.0,
            curves: .defaultIdentity
        )
        
        let record = XmpDataRecord(
            rating: finalRating,
            status: finalStatus,
            title: existingXmp?.title,
            description: existingXmp?.description,
            latitude: currentPhoto?.latitude,
            longitude: currentPhoto?.longitude,
            edits: finalEdits
        )
        _ = writeXmp(path: filePath, record: record)
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
