import Foundation
import SwiftUI
import AppKit
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
    public var isExternalVolume: Bool = false
    public var isPortableCatalog: Bool = false
    
    // MARK: - Photos & Selection
    public var photos: [PhotoRecord] = []
    public var selectedPhoto: PhotoRecord? = nil
    public var previewImage: NSImage? = nil
    
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
    
    // MARK: - Rust Core Objects
    private var db: LumenBoxDatabase? = nil
    private var lensfun: LumenBoxLensfun? = nil
    
    public init() {
        self.lensfun = LumenBoxLensfun()
    }
    
    // MARK: - Directory Indexing & Loading
    public func openDirectory(at folderPath: String) {
        self.currentFolderPath = folderPath
        self.isExternalVolume = folderPath.hasPrefix("/Volumes/") && !folderPath.hasPrefix("/Volumes/Macintosh HD")
        
        let catalogPath = resolveCatalogPath(folderPath: folderPath, portable: self.isExternalVolume)
        
        do {
            let database = try LumenBoxDatabase.open(dbPath: catalogPath)
            self.db = database
            
            // Index directory asynchronously in background thread
            DispatchQueue.global(qos: .userInitiated).async { [weak self] in
                guard let self = self, let database = self.db else { return }
                _ = database.indexFolder(folderPath: folderPath)
                
                let fetchedPhotos = database.getAllPhotos()
                let takeoutList = scanDirectoryForTakeout(dirPath: folderPath)
                
                DispatchQueue.main.async {
                    self.photos = fetchedPhotos
                    self.takeoutMatches = takeoutList
                    if let first = fetchedPhotos.first {
                        self.selectPhoto(first)
                    }
                }
            }
        } catch {
            print("Failed to open database at \(catalogPath): \(error)")
        }
    }
    
    // MARK: - Photo Selection & Preview
    public func selectPhoto(_ photo: PhotoRecord) {
        self.selectedPhoto = photo
        
        // Extract embedded RAW preview JPEG in background
        DispatchQueue.global(qos: .userInteractive).async { [weak self] in
            guard let self = self else { return }
            let image = extractPreview(path: photo.filePath).flatMap { NSImage(data: $0) }
            
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
    
    // MARK: - Slider Reset Helpers
    public func resetSlider(_ type: EditSliderType) {
        switch type {
        case .exposure:
            exposure = 0.0
        case .contrast:
            contrast = 0.0
        case .temperature:
            temperature = 5500.0
        case .tint:
            tint = 0.0
        case .straighten:
            straightenAngle = 0.0
        }
    }
    
    public func resetAllSliders() {
        exposure = 0.0
        contrast = 0.0
        temperature = 5500.0
        tint = 0.0
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
