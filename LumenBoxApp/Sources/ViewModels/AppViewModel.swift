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

@Observable
public final class AppViewModel {
    // MARK: - Navigation & Directory State
    public var currentFolderPath: String? = nil
    public var openedFolders: [String] = []
    public var isExternalVolume: Bool = false
    public var isPortableCatalog: Bool = false
    public var isScanning: Bool = false
    public var scanningStatusMessage: String? = nil
    
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
            openDirectory(at: target)
        } else if let firstFolder = self.openedFolders.first {
            openDirectory(at: firstFolder)
        }
    }
    
    // MARK: - Multi-Folder Workspace Management
    public func addFolder(at folderPath: String) {
        if !openedFolders.contains(folderPath) {
            openedFolders.append(folderPath)
            UserDefaults.standard.set(openedFolders, forKey: "openedFolders")
        }
        openDirectory(at: folderPath)
    }
    
    public func removeFolder(at folderPath: String) {
        openedFolders.removeAll { $0 == folderPath }
        UserDefaults.standard.set(openedFolders, forKey: "openedFolders")
        
        if currentFolderPath == folderPath {
            if let next = openedFolders.first {
                openDirectory(at: next)
            } else {
                currentFolderPath = nil
                photos = []
                selectedPhoto = nil
                previewImage = nil
                UserDefaults.standard.removeObject(forKey: "lastOpenedDirectory")
            }
        }
    }
    
    // MARK: - Directory Indexing & Loading
    public func openDirectory(at folderPath: String) {
        UserDefaults.standard.set(folderPath, forKey: "lastOpenedDirectory")
        self.currentFolderPath = folderPath
        self.isExternalVolume = folderPath.hasPrefix("/Volumes/") && !folderPath.hasPrefix("/Volumes/Macintosh HD")
        self.isScanning = true
        self.scanningStatusMessage = "Scanning \(URL(fileURLWithPath: folderPath).lastPathComponent)..."
        
        let catalogPath = resolveCatalogPath(folderPath: folderPath, portable: self.isExternalVolume)
        
        // 1. Instant 0ms Initial Load: Display existing cached photos immediately
        if let database = try? LumenBoxDatabase.open(dbPath: catalogPath) {
            self.db = database
            let existing = database.getAllPhotos()
            if !existing.isEmpty {
                self.photos = existing
                if self.selectedPhoto == nil, let first = existing.first {
                    self.selectPhoto(first)
                }
            }
        }
        
        // 2. Background Streaming Indexer with progressive UI refresh
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }
            do {
                let database = try LumenBoxDatabase.open(dbPath: catalogPath)
                self.db = database
                
                // Progressive stream timer: Refreshes UI as new photos are indexed
                var isDone = false
                let timer = DispatchSource.makeTimerSource(queue: DispatchQueue.global(qos: .default))
                timer.schedule(deadline: .now() + 0.35, repeating: 0.35)
                timer.setEventHandler { [weak self] in
                    guard let self = self, !isDone, let db = self.db else { return }
                    let currentPhotos = db.getAllPhotos()
                    DispatchQueue.main.async {
                        if currentPhotos.count != self.photos.count {
                            self.photos = currentPhotos
                            if self.selectedPhoto == nil, let first = currentPhotos.first {
                                self.selectPhoto(first)
                            }
                        }
                    }
                }
                timer.resume()
                
                // Run optimized Rust core batched folder indexer
                _ = database.indexFolder(folderPath: folderPath)
                
                isDone = true
                timer.cancel()
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
