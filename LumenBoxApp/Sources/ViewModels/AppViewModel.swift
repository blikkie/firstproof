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
}

public enum LibraryScope: Hashable, Equatable {
    case mainLibrary
    case session(folderPath: String)
}

@Observable
public final class AppViewModel {
    // MARK: - Navigation & Directory State
    public var currentScope: LibraryScope = .mainLibrary
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
    public var previewImage: NSImage? = nil
    public var isExportSheetPresented: Bool = false
    
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
    
    // MARK: - Takeout & Metadata
    public var takeoutMatches: [TakeoutMatchRecord] = []
    public var lensCoefficients: LensCorrectionCoefficientsRecord? = nil
    
    // MARK: - Rust Core Objects
    private var db: LumenBoxDatabase? = nil
    private var lensfun: LumenBoxLensfun? = nil
    
    public init() {
        self.lensfun = LumenBoxLensfun()
        
        // Restore multi-project opened folder list
        let savedFolders = UserDefaults.standard.stringArray(forKey: "openedFolders") ?? []
        self.openedFolders = savedFolders.filter { FileManager.default.fileExists(atPath: $0) }
        
        let lastPath = UserDefaults.standard.string(forKey: "lastOpenedDirectory")
        if let target = lastPath, FileManager.default.fileExists(atPath: target) {
            if !self.openedFolders.contains(target) {
                self.openedFolders.append(target)
            }
            selectScope(.session(folderPath: target))
        } else if let firstFolder = self.openedFolders.first {
            selectScope(.session(folderPath: firstFolder))
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
            if let next = openedFolders.first {
                selectScope(.session(folderPath: next))
            } else {
                selectScope(.mainLibrary)
            }
        }
    }
    
    // MARK: - Directory Indexing & Loading
    public func openDirectory(at folderPath: String, isMain: Bool = false) {
        if !isMain {
            UserDefaults.standard.set(folderPath, forKey: "lastOpenedDirectory")
        }
        self.currentFolderPath = folderPath
        self.isExternalVolume = folderPath.hasPrefix("/Volumes/") && !folderPath.hasPrefix("/Volumes/Macintosh HD")
        self.isScanning = true
        self.scanningStatusMessage = "Scanning \(isMain ? "Main Library" : URL(fileURLWithPath: folderPath).lastPathComponent)..."
        
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
                        if self.selectedPhoto == nil, let first = cachedPhotos.first {
                            self.selectPhoto(first)
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
                    if self.selectedPhoto == nil, let first = finalPhotos.first {
                        self.selectPhoto(first)
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
        
        // Decode authentic 14-bit RAW sensor demosaic in background
        DispatchQueue.global(qos: .userInteractive).async { [weak self] in
            guard let self = self else { return }
            var image: NSImage? = nil
            
            // 1. Authentic Rust Core 14-bit RAW demosaicing & calibrated sRGB pipeline
            if let raw = decodeRawPreview(path: photo.filePath, maxDimension: 2560) {
                let width = Int(raw.width)
                let height = Int(raw.height)
                let colorSpace = CGColorSpaceCreateDeviceRGB()
                if let provider = CGDataProvider(data: raw.data as CFData),
                   let cgImg = CGImage(
                       width: width,
                       height: height,
                       bitsPerComponent: 8,
                       bitsPerPixel: 32,
                       bytesPerRow: width * 4,
                       space: colorSpace,
                       bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedLast.rawValue),
                       provider: provider,
                       decode: nil,
                       shouldInterpolate: true,
                       intent: .defaultIntent
                   ) {
                    image = NSImage(cgImage: cgImg, size: NSSize(width: width, height: height))
                }
            }
            
            // 2. Fallback for non-RAW master images (PNG, TIFF, JPEG)
            if image == nil {
                let url = URL(fileURLWithPath: photo.filePath) as CFURL
                if let src = CGImageSourceCreateWithURL(url, nil) {
                    let options: [CFString: Any] = [
                        kCGImageSourceCreateThumbnailFromImageAlways: true,
                        kCGImageSourceCreateThumbnailWithTransform: true,
                        kCGImageSourceThumbnailMaxPixelSize: 3840
                    ]
                    if let cgThumb = CGImageSourceCreateThumbnailAtIndex(src, 0, options as CFDictionary) {
                        image = NSImage(cgImage: cgThumb, size: NSSize(width: cgThumb.width, height: cgThumb.height))
                    }
                }
            }
            
            DispatchQueue.main.async {
                // Prevent out-of-order async race condition: only update if photo is still selected!
                if self.selectedPhoto?.filePath == photo.filePath {
                    self.previewImage = image
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
        } else {
            resetAllSliders()
        }
    }
    
    // MARK: - Rating & Pick Status
    public func setRating(_ rating: Int32?) {
        guard let current = selectedPhoto, let db = db else { return }
        let newRating = (current.rating == rating) ? nil : rating
        let success = db.updateRatingStatus(filePath: current.filePath, rating: newRating, status: current.status)
        if success {
            if let index = photos.firstIndex(where: { $0.filePath == current.filePath }) {
                photos[index].rating = newRating
                selectedPhoto = photos[index]
            }
            syncXmpForCurrentPhoto()
        }
    }
    
    public func setStatus(_ status: String) {
        guard let current = selectedPhoto, let db = db else { return }
        let newStatus = (current.status == status) ? "none" : status
        let success = db.updateRatingStatus(filePath: current.filePath, rating: current.rating, status: newStatus)
        if success {
            if let index = photos.firstIndex(where: { $0.filePath == current.filePath }) {
                photos[index].status = newStatus
                selectedPhoto = photos[index]
            }
            syncXmpForCurrentPhoto()
        }
    }
    
    private func syncXmpForCurrentPhoto() {
        guard let current = selectedPhoto else { return }
        let xmpPath = URL(fileURLWithPath: current.filePath).deletingPathExtension().appendingPathExtension("xmp").path
        let existingXmp = readXmp(path: current.filePath)
        
        let record = XmpDataRecord(
            rating: current.rating,
            status: current.status,
            title: existingXmp?.title,
            description: existingXmp?.description,
            latitude: current.latitude,
            longitude: current.longitude,
            edits: EditSettings(
                exposure: self.exposure,
                contrast: self.contrast,
                temperature: self.temperature,
                tint: self.tint,
                cropLeft: 0.0,
                cropTop: 0.0,
                cropRight: 1.0,
                cropBottom: 1.0,
                cropAngle: self.straightenAngle
            )
        )
        _ = writeXmp(path: xmpPath, record: record)
    }
    
    // MARK: - As-Shot Baselines
    public var asShotTemperature: Float = 5500.0
    public var asShotTint: Float = 0.0
    
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
        }
    }
    
    public func resetAllSliders() {
        exposure = 0.0
        contrast = 0.0
        temperature = asShotTemperature
        tint = asShotTint
        straightenAngle = 0.0
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
