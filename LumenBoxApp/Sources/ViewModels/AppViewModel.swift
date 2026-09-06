import Foundation
import SwiftUI
import AppKit
import ImageIO
import LumenBoxBridge

public enum EditSliderType {
    case exposure
    case contrast
    case temperature
    case tint
    case straighten
    case lutIntensity
}

public enum CanvasMode: Hashable {
    case grid
    case detail
}

public enum LibraryScope: Hashable, Equatable {
    case mainLibrary
    case session(folderPath: String)
}

public enum InspectorTab: String, CaseIterable, Identifiable {
    case light = "Light"
    case color = "Color"
    case optics = "Optics"
    case detail = "Detail"
    case metadata = "Info"
    
    public var id: String { rawValue }
    
    public var iconName: String {
        switch self {
        case .light: return "sun.max.fill"
        case .color: return "paintpalette.fill"
        case .optics: return "crop.rotate"
        case .detail: return "slider.horizontal.2.square"
        case .metadata: return "info.circle.fill"
        }
    }
}

@Observable
public final class AppViewModel {
    // MARK: - Navigation & Directory State
    public var currentScope: LibraryScope = .mainLibrary
    public var canvasMode: CanvasMode = .grid
    public var currentFolderPath: String? = nil
    public var openedFolders: [String] = []
    public var isExternalVolume: Bool = false
    public var isPortableCatalog: Bool = false
    public var isScanning: Bool = false
    public var scanningStatusMessage: String? = nil
    
    // Main Library Path (~/Pictures/LumenBox Library)
    public var mainLibraryPath: String {
        let pictures = FileManager.default.urls(for: .picturesDirectory, in: .userDomainMask).first ?? URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent("Pictures")
        let lumenBoxLib = pictures.appendingPathComponent("LumenBox Library")
        try? FileManager.default.createDirectory(at: lumenBoxLib, withIntermediateDirectories: true)
        return lumenBoxLib.path
    }
    
    // MARK: - Photos & Selection
    public var photos: [PhotoRecord] = []
    public var selectedPhoto: PhotoRecord? = nil
    public var selectedPhotoPaths: Set<String> = []
    public var isBulkSelectionMode: Bool = false
    public var previewImage: NSImage? = nil
    public var rawPreviewRecord: RawPreviewRecord? = nil
    public var isDemosaicing: Bool = false
    @ObservationIgnored private var activeDecodeToken: UUID = UUID()
    public var isExportSheetPresented: Bool = false
    public var isExportPackageSheetPresented: Bool = false
    
    public var selectedPhotoIsRaw: Bool {
        guard let path = selectedPhoto?.filePath else { return false }
        let ext = URL(fileURLWithPath: path).pathExtension.lowercased()
        return ["cr2", "cr3", "nef", "arw", "dng", "orf", "rw2", "raf", "pef", "raw"].contains(ext)
    }
    
    // MARK: - Filters
    public var ratingFilter: Int? = nil // nil = All, 1..5
    public var statusFilter: String? = nil // nil = All, "pick", "reject"
    
    // MARK: - Edit Sliders (P0 + Lens Corrections)
    public var exposure: Float = 0.0      // -5.0 .. +5.0 EV
    public var contrast: Float = 0.0      // -100 .. +100
    public var temperature: Float = 5500.0 // 2000K .. 50000K
    public var tint: Float = 0.0          // -150 .. +150
    public var straightenAngle: Float = 0.0 // -45 .. +45 deg
    public var lensCorrectionEnabled: Bool = true
    public var toneProfile: ColorProfileMode = .agxVivid
    public var selectedLutName: String? = nil
    public var lutIntensity: Float = 1.0
    public var availableLutTitles: [String] = []
    public var customLuts: [CubeLutRecord] = []
    
    // MARK: - Workspace Navigation & Tabs
    public var selectedInspectorTab: InspectorTab = .light
    public var rotationDegrees: Int = 0 // 0, 90, 180, 270
    
    public var hasCustomEdits: Bool {
        exposure != 0.0 ||
        contrast != 0.0 ||
        abs(temperature - asShotTemperature) >= 1.0 ||
        tint != asShotTint ||
        straightenAngle != 0.0 ||
        rotationDegrees != 0 ||
        toneProfile != .agxVivid ||
        selectedLutName != nil
    }
    
    public func rotateClockwise() {
        rotationDegrees = (rotationDegrees + 90) % 360
        saveCurrentEdits()
    }
    
    public func rotateCounterClockwise() {
        rotationDegrees = (rotationDegrees + 270) % 360
        saveCurrentEdits()
    }
    
    public var allLutTitles: [String] {
        availableLutTitles + customLuts.map(\.title)
    }
    
    public func resolveLutRecord(for title: String?) -> CubeLutRecord? {
        guard let title = title, !title.isEmpty else { return nil }
        if let custom = customLuts.first(where: { $0.title == title }) {
            return custom
        }
        return getBuiltinCubeLut(title: title, size: 33)
    }
    
    // MARK: - Viewport Zoom & Pan
    public var zoomScale: Float = 1.0
    public var panOffset: CGPoint = .zero
    public var zoomScale100: Float = 1.0
    
    public var isZoomed: Bool {
        zoomScale > 1.05
    }
    
    public var zoomPercentage: Int {
        if zoomScale <= 1.02 {
            let fitRatio = (1.0 / max(1.0, zoomScale100)) * 100.0
            return max(1, Int(round(fitRatio)))
        } else {
            let actualRatio = (zoomScale / max(1.0, zoomScale100)) * 100.0
            return max(1, Int(round(actualRatio)))
        }
    }
    
    public var zoomReadoutText: String {
        if zoomScale <= 1.02 {
            return "Fit (\(zoomPercentage)%)"
        } else {
            return "\(zoomPercentage)%"
        }
    }
    
    public func zoomToFit() {
        zoomScale = 1.0
        panOffset = .zero
    }
    
    public func zoomTo100() {
        zoomScale = max(1.0, zoomScale100)
        panOffset = .zero
    }
    
    public func toggleZoom(at pointInNDC: CGPoint = .zero) {
        if isZoomed {
            zoomToFit()
        } else {
            zoomScale = max(1.0, zoomScale100)
            panOffset = CGPoint(x: -pointInNDC.x * CGFloat(zoomScale - 1.0), y: -pointInNDC.y * CGFloat(zoomScale - 1.0))
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
    
    // MARK: - Takeout & Metadata
    public var takeoutMatches: [TakeoutMatchRecord] = []
    public var lensCoefficients: LensCorrectionCoefficientsRecord? = nil
    
    // MARK: - Rust Core Objects
    private var db: LumenBoxDatabase? = nil
    private var lensfun: LumenBoxLensfun? = nil
    
    public init() {
        self.lensfun = LumenBoxLensfun()
        self.availableLutTitles = getBuiltinCubeLutTitles()
        
        // Restore multi-project opened folder list (preserves disconnected external drive sessions)
        self.openedFolders = UserDefaults.standard.stringArray(forKey: "openedFolders") ?? []
        
        let lastPath = UserDefaults.standard.string(forKey: "lastOpenedDirectory")
        if let target = lastPath, FileManager.default.fileExists(atPath: target) {
            if !self.openedFolders.contains(target) {
                self.openedFolders.append(target)
            }
            selectScope(.session(folderPath: target))
        } else if let firstAvailable = self.openedFolders.first(where: { FileManager.default.fileExists(atPath: $0) }) {
            selectScope(.session(folderPath: firstAvailable))
        } else {
            selectScope(.mainLibrary)
        }
    }
    
    // MARK: - Scope Switching
    public func selectScope(_ scope: LibraryScope) {
        self.currentScope = scope
        switch scope {
        case .mainLibrary:
            openDirectory(at: mainLibraryPath, isMain: true)
        case .session(let folderPath):
            openDirectory(at: folderPath, isMain: false)
        }
    }
    
    // MARK: - Multi-Folder Workspace Management
    public func addFolder(at folderPath: String) {
        if !openedFolders.contains(folderPath) {
            openedFolders.append(folderPath)
            UserDefaults.standard.set(openedFolders, forKey: "openedFolders")
        }
        selectScope(.session(folderPath: folderPath))
    }
    
    public func removeFolder(at folderPath: String) {
        openedFolders.removeAll { $0 == folderPath }
        UserDefaults.standard.set(openedFolders, forKey: "openedFolders")
        
        if case .session(let path) = currentScope, path == folderPath {
            if let next = openedFolders.first(where: { FileManager.default.fileExists(atPath: $0) }) {
                selectScope(.session(folderPath: next))
            } else {
                selectScope(.mainLibrary)
            }
        }
    }
    
    // MARK: - Last Active Image Tracking Per Library
    public func saveLastActivePhoto(for scope: LibraryScope, path: String) {
        let key = scopeKey(for: scope)
        var map = UserDefaults.standard.dictionary(forKey: "lastActivePhotoPerScope") as? [String: String] ?? [:]
        map[key] = path
        UserDefaults.standard.set(map, forKey: "lastActivePhotoPerScope")
    }
    
    public func lastActivePhoto(for scope: LibraryScope) -> String? {
        let key = scopeKey(for: scope)
        let map = UserDefaults.standard.dictionary(forKey: "lastActivePhotoPerScope") as? [String: String] ?? [:]
        return map[key]
    }
    
    private func scopeKey(for scope: LibraryScope) -> String {
        switch scope {
        case .mainLibrary:
            return "main_library"
        case .session(let folderPath):
            return folderPath
        }
    }
    
    // MARK: - Directory Indexing & Loading
    public func openDirectory(at folderPath: String, isMain: Bool = false) {
        if !isMain {
            UserDefaults.standard.set(folderPath, forKey: "lastOpenedDirectory")
        }
        self.currentFolderPath = folderPath
        self.isExternalVolume = folderPath.hasPrefix("/Volumes/") && !folderPath.hasPrefix("/Volumes/Macintosh HD")
        
        // Graceful handling of disconnected/unmounted external drives:
        guard FileManager.default.fileExists(atPath: folderPath) else {
            self.photos = []
            self.selectedPhoto = nil
            self.previewImage = nil
            self.isScanning = false
            self.scanningStatusMessage = "Storage device is not mounted"
            return
        }
        
        self.isScanning = true
        self.scanningStatusMessage = "Scanning \(isMain ? "Main Library" : URL(fileURLWithPath: folderPath).lastPathComponent)..."
        
        // Save any pending edits before changing directory
        saveCurrentEdits()
        self.currentlyLoadedPhotoPath = nil

        // Reset state for new scope
        self.photos = []
        self.selectedPhoto = nil
        self.previewImage = nil
        
        let catalogPath = resolveCatalogPath(folderPath: folderPath, portable: self.isExternalVolume)
        let t0 = CFAbsoluteTimeGetCurrent()
        
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }
            do {
                let database = try LumenBoxDatabase.open(dbPath: catalogPath)
                self.db = database
                
                // 1. Instant Async Initial Load: Read cached photos off main thread
                let cachedPhotos = database.getAllPhotos()
                let tCached = CFAbsoluteTimeGetCurrent()
                print("⚡ [PERF] Cached photos loaded: \(cachedPhotos.count) photos in \(String(format: "%.1f", (tCached - t0) * 1000.0)) ms")
                
                if !cachedPhotos.isEmpty {
                    DispatchQueue.main.async {
                        self.photos = cachedPhotos
                        if self.selectedPhoto == nil {
                            if let savedPath = self.lastActivePhoto(for: self.currentScope),
                               let saved = cachedPhotos.first(where: { $0.filePath == savedPath }) {
                                self.selectPhoto(saved)
                            } else if let first = cachedPhotos.first {
                                self.selectPhoto(first)
                            }
                        }
                    }
                }
                
                // 2. Background Batched Directory Indexer
                let count = database.indexFolder(folderPath: folderPath)
                let tIndexed = CFAbsoluteTimeGetCurrent()
                print("📦 [PERF] Indexing complete: \(count) files in \(String(format: "%.1f", (tIndexed - tCached) * 1000.0)) ms")
                
                // 3. Publish Final Indexed Photos
                let finalPhotos = database.getAllPhotos()
                
                DispatchQueue.main.async {
                    self.photos = finalPhotos
                    self.isScanning = false
                    self.scanningStatusMessage = nil
                    if self.selectedPhoto == nil || !finalPhotos.contains(where: { $0.filePath == self.selectedPhoto?.filePath }) {
                        if let savedPath = self.lastActivePhoto(for: self.currentScope),
                           let saved = finalPhotos.first(where: { $0.filePath == savedPath }) {
                            self.selectPhoto(saved)
                        } else if let first = finalPhotos.first {
                            self.selectPhoto(first)
                        }
                    }
                }
            } catch {
                print("Failed to open database at \(catalogPath): \(error)")
                DispatchQueue.main.async {
                    self.isScanning = false
                    self.scanningStatusMessage = nil
                }
            }
        }
    }
    
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
            // Disassociate active cursor from bulk selection set
            self.isBulkSelectionMode = true
            if let current = selectedPhoto {
                self.selectedPhotoPaths = [current.filePath]
            } else if let first = filteredPhotos.first {
                self.selectedPhoto = first
                self.selectedPhotoPaths = [first.filePath]
                loadPreview(for: first)
            }
        } else {
            // Toggle active photo in/out of bulk selection set
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
    
    public var gridColumnCount: Int = 4
    
    public func enterDetailMode() {
        if selectedPhoto == nil, let first = filteredPhotos.first {
            selectPhoto(first)
        }
        self.canvasMode = .detail
    }
    
    public func enterGridMode() {
        self.canvasMode = .grid
    }
    
    public func selectNextPhoto() {
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
    
    private func navigateToPhoto(at index: Int, in list: [PhotoRecord]) {
        let target = list[index]
        self.selectedPhoto = target
        if !isBulkSelectionMode {
            self.selectedPhotoPaths = [target.filePath]
        }
        saveLastActivePhoto(for: currentScope, path: target.filePath)
        loadPreview(for: target)
    }
    
    private var currentlyLoadedPhotoPath: String? = nil
    
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
        
        // Fast switch: immediately present cached thumbnail so user can scrub instantly
        if let cached = ThumbnailCache.shared.image(for: photo.filePath) {
            self.previewImage = cached
        } else {
            self.previewImage = nil
        }
        
        // Decode in background: extract clean lower-res camera JPEG preview (neutral colors, zero tricks)
        DispatchQueue.global(qos: .userInteractive).async { [weak self] in
            guard let self = self else { return }
            guard self.activeDecodeToken == token else { return }
            
            var preview: NSImage? = nil
            
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
                    let options: [CFString: Any] = [
                        kCGImageSourceCreateThumbnailFromImageAlways: true,
                        kCGImageSourceCreateThumbnailWithTransform: true,
                        kCGImageSourceThumbnailMaxPixelSize: 3840
                    ]
                    if let cgThumb = CGImageSourceCreateThumbnailAtIndex(src, 0, options as CFDictionary) {
                        preview = NSImage(cgImage: cgThumb, size: NSSize(width: cgThumb.width, height: cgThumb.height))
                    }
                }
            }
            
            DispatchQueue.main.async {
                if self.activeDecodeToken == token && self.selectedPhoto?.filePath == photo.filePath {
                    if let finalImage = preview {
                        // If it's a RAW file and Stage 2 demosaicing already finished, do not overwrite with thumbnail
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
                    
                    if let rawRecord = decodeRawPreview(path: photo.filePath, maxDimension: 2560) {
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
        
        // Sync XMP edit sliders if sidecar exists
        if let xmp = readXmp(path: photo.filePath) {
            self.exposure = xmp.edits.exposure
            self.contrast = xmp.edits.contrast
            self.temperature = xmp.edits.temperature
            self.tint = xmp.edits.tint
            self.straightenAngle = xmp.edits.cropAngle
            self.toneProfile = xmp.edits.toneProfile
            self.selectedLutName = xmp.edits.lutName
            self.lutIntensity = xmp.edits.lutIntensity
        } else {
            self.exposure = 0.0
            self.contrast = 0.0
            self.temperature = 5500.0
            self.tint = 0.0
            self.straightenAngle = 0.0
            self.toneProfile = .agxVivid
            self.selectedLutName = nil
            self.lutIntensity = 1.0
        }
    }
    
    // MARK: - Portable Package Export
    public func exportSelectedAsPackage(
        destinationFolder: String,
        moveFiles: Bool,
        openAfter: Bool,
        completion: @escaping (ExportPackageReport?) -> Void
    ) {
        var targets = Array(selectedPhotoPaths)
        if targets.isEmpty, let single = selectedPhoto?.filePath {
            targets = [single]
        }
        guard !targets.isEmpty else {
            completion(nil)
            return
        }
        
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }
            do {
                let report = try exportPortablePackage(
                    sourcePhotoPaths: targets,
                    destinationFolder: destinationFolder,
                    moveFiles: moveFiles
                )
                
                DispatchQueue.main.async {
                    if moveFiles {
                        let movedSet = Set(targets)
                        self.photos.removeAll { movedSet.contains($0.filePath) }
                        self.selectedPhotoPaths.subtract(movedSet)
                        if let current = self.selectedPhoto, movedSet.contains(current.filePath) {
                            self.selectedPhoto = self.photos.first
                            if let first = self.photos.first {
                                self.loadPreview(for: first)
                            } else {
                                self.previewImage = nil
                            }
                        }
                    }
                    
                    if openAfter {
                        self.addFolder(at: destinationFolder)
                    }
                    
                    completion(report)
                }
            } catch {
                print("Failed to export portable package: \(error)")
                DispatchQueue.main.async {
                    completion(nil)
                }
            }
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
        
        // Determine new rating: if all selected photos currently have this rating, toggle off to nil; otherwise apply rating
        let targetPhotos = photos.filter { targets.contains($0.filePath) }
        let allMatch = targetPhotos.allSatisfy { $0.rating == rating }
        let newRating: Int32? = allMatch ? nil : rating
        
        // 1. Transactional SQLite batch update
        _ = db.updateRatingStatusBatch(filePaths: targets, rating: newRating, updateRating: true, status: nil)
        
        // 2. In-Memory State Updates
        let targetSet = Set(targets)
        for i in photos.indices {
            if targetSet.contains(photos[i].filePath) {
                photos[i].rating = newRating
            }
        }
        if let current = selectedPhoto, targetSet.contains(current.filePath) {
            selectedPhoto?.rating = newRating
        }
        
        // 3. Asynchronous XMP Sidecar Sync
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
        
        // Determine new status: if all selected photos have this status, toggle to "none"; otherwise apply status
        let targetPhotos = photos.filter { targets.contains($0.filePath) }
        let allMatch = targetPhotos.allSatisfy { $0.status == status }
        let newStatus = allMatch ? "none" : status
        
        // 1. Transactional SQLite batch update
        _ = db.updateRatingStatusBatch(filePaths: targets, rating: nil, updateRating: false, status: newStatus)
        
        // 2. In-Memory State Updates
        let targetSet = Set(targets)
        for i in photos.indices {
            if targetSet.contains(photos[i].filePath) {
                photos[i].status = newStatus
            }
        }
        if let current = selectedPhoto, targetSet.contains(current.filePath) {
            selectedPhoto?.status = newStatus
        }
        
        // 3. Asynchronous XMP Sidecar Sync
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
    
    // MARK: - Current Edits Snapshot
    public var currentEdits: EditSettings {
        EditSettings(
            exposure: self.exposure,
            contrast: self.contrast,
            temperature: self.temperature,
            tint: self.tint,
            cropLeft: 0.0,
            cropTop: 0.0,
            cropRight: 1.0,
            cropBottom: 1.0,
            cropAngle: self.straightenAngle,
            toneProfile: self.toneProfile,
            lutName: self.selectedLutName,
            lutIntensity: self.lutIntensity
        )
    }
    
    private func syncXmp(for filePath: String, rating: Int32??, status: String?, edits: EditSettings? = nil) {
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
            temperature: 5500.0,
            tint: 0.0,
            cropLeft: 0.0,
            cropTop: 0.0,
            cropRight: 1.0,
            cropBottom: 1.0,
            cropAngle: 0.0,
            toneProfile: .agxVivid,
            lutName: nil,
            lutIntensity: 1.0
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
    
    public func saveCurrentEdits() {
        guard let path = currentlyLoadedPhotoPath ?? selectedPhoto?.filePath else { return }
        let editsSnapshot = currentEdits
        DispatchQueue.global(qos: .utility).async { [weak self] in
            self?.syncXmp(for: path, rating: nil, status: nil, edits: editsSnapshot)
        }
    }
    
    // MARK: - As-Shot Baselines
    public var asShotTemperature: Float = 5500.0
    public var asShotTint: Float = 0.0
    
    // MARK: - Auto Development Adjustments
    public func applyAutoAdjustments() {
        guard let photo = selectedPhoto else { return }
        
        // 1. Fast-path: Calculate from already decoded half-float preview buffer
        if let rawRecord = rawPreviewRecord {
            if let adj = computeAutoAdjustmentsFromBuffer(width: rawRecord.width, height: rawRecord.height, data: rawRecord.data) {
                withAnimation(.easeInOut(duration: 0.15)) {
                    self.exposure = adj.exposure
                    self.contrast = adj.contrast
                }
                saveCurrentEdits()
                return
            }
        }
        
        // 2. Background path: Decode from file
        let path = photo.filePath
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let adj = computeAutoAdjustments(path: path) else { return }
            DispatchQueue.main.async {
                guard let self = self, self.selectedPhoto?.filePath == path else { return }
                withAnimation(.easeInOut(duration: 0.15)) {
                    self.exposure = adj.exposure
                    self.contrast = adj.contrast
                }
                self.saveCurrentEdits()
            }
        }
    }
    
    // MARK: - Slider Reset Helpers
    public func resetSlider(_ type: EditSliderType) {
        switch type {
        case .exposure:
            exposure = 0.0
        case .contrast:
            contrast = 0.0
        case .temperature:
            temperature = asShotTemperature
        case .tint:
            tint = asShotTint
        case .straighten:
            straightenAngle = 0.0
        case .lutIntensity:
            lutIntensity = 1.0
        }
        saveCurrentEdits()
    }
    
    public func resetAllSliders() {
        exposure = 0.0
        contrast = 0.0
        temperature = asShotTemperature
        tint = asShotTint
        straightenAngle = 0.0
        rotationDegrees = 0
        toneProfile = .agxVivid
        selectedLutName = nil
        lutIntensity = 1.0
        saveCurrentEdits()
    }
    
    // MARK: - Custom 3D LUT Import
    public func importCustomLut() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.init(filenameExtension: "cube")].compactMap { $0 }
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.canChooseFiles = true
        panel.title = "Import 3D LUT (.cube)"
        
        if panel.runModal() == .OK, let url = panel.url {
            if let customLut = loadCubeLut(path: url.path) {
                if !self.customLuts.contains(where: { $0.title == customLut.title }) {
                    self.customLuts.append(customLut)
                }
                self.selectedLutName = customLut.title
                self.lutIntensity = 1.0
                saveCurrentEdits()
            }
        }
    }
    
    // MARK: - Filtered Photo List
    public var filteredPhotos: [PhotoRecord] {
        photos.filter { photo in
            if let rating = ratingFilter, photo.rating != Int32(rating) {
                return false
            }
            if let status = statusFilter, photo.status != status {
                return false
            }
            return true
        }
    }
}
