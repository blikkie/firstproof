import Foundation
import SwiftUI
import AppKit
import FirstProofBridge

extension AppViewModel {
    
    public func startDngConversionFlow(targetPhoto: PhotoRecord? = nil) {
        if let target = targetPhoto {
            if target.isProprietaryRaw {
                self.selectedPhoto = target
                self.selectedPhotoPaths = [target.filePath]
            }
        }
        guard !convertibleRawPhotosSelected.isEmpty else { return }
        self.dngBatchReport = nil
        self.dngBatchErrorMessage = nil
        self.dngProgressCurrent = 0
        self.dngProgressTotal = UInt32(convertibleRawPhotosSelected.count)
        self.dngProgressFileName = ""
        self.isDngConversionSheetPresented = true
    }
    
    public func executeBatchDngConversion(deleteOriginal: Bool) {
        let targets = convertibleRawPhotosSelected
        guard !targets.isEmpty else { return }
        
        self.isConvertingDng = true
        self.dngBatchErrorMessage = nil
        self.dngBatchReport = nil
        self.dngProgressCurrent = 0
        self.dngProgressTotal = UInt32(targets.count)
        self.dngProgressFileName = targets.first.map { URL(fileURLWithPath: $0.filePath).lastPathComponent } ?? ""
        
        let catalogFolder = currentFolderPath ?? mainLibraryPath
        let paths = targets.map(\.filePath)
        
        let tracker = DngBatchProgressTracker { [weak self] curr, tot, file in
            DispatchQueue.main.async {
                self?.dngProgressCurrent = curr
                self?.dngProgressTotal = tot
                self?.dngProgressFileName = file
            }
        }
        
        activeDngConversionTask = Task.detached(priority: .userInitiated) { [weak self] in
            do {
                let report = try batchConvertCatalogToDng(
                    catalogFolder: catalogFolder,
                    photoPaths: paths,
                    deleteOriginal: deleteOriginal,
                    progress: tracker
                )
                
                await MainActor.run {
                    guard let self = self else { return }
                    self.dngBatchReport = report
                    self.isConvertingDng = false
                    
                    // Build path mapping for converted items
                    var pathMapping: [String: String] = [:]
                    for r in report.reports {
                        pathMapping[r.sourcePath] = r.dngPath
                        
                        // Invalidate thumbnail cache
                        ThumbnailCache.shared.removeImage(for: r.sourcePath)
                        ThumbnailCache.shared.removeImage(for: r.dngPath)
                        NotificationCenter.default.post(name: .thumbnailDidChange, object: r.sourcePath)
                        NotificationCenter.default.post(name: .thumbnailDidChange, object: r.dngPath)
                    }
                    
                    // Update photos array in-place or append new DNG records
                    if deleteOriginal {
                        self.photos = self.photos.map { photo in
                            if let newPath = pathMapping[photo.filePath] {
                                let newName = URL(fileURLWithPath: newPath).lastPathComponent
                                let rep = report.reports.first(where: { $0.sourcePath == photo.filePath })
                                let newSize = rep.map { Int64($0.dngSizeBytes) } ?? photo.fileSize
                                return PhotoRecord(
                                    id: photo.id,
                                    filePath: newPath,
                                    fileName: newName,
                                    fileSize: newSize,
                                    modifiedTime: photo.modifiedTime,
                                    cameraMake: photo.cameraMake,
                                    cameraModel: photo.cameraModel,
                                    lensModel: photo.lensModel,
                                    iso: photo.iso,
                                    aperture: photo.aperture,
                                    shutterSpeed: photo.shutterSpeed,
                                    focalLength: photo.focalLength,
                                    takenTime: photo.takenTime,
                                    latitude: photo.latitude,
                                    longitude: photo.longitude,
                                    title: photo.title,
                                    description: photo.description,
                                    creator: photo.creator,
                                    copyright: photo.copyright,
                                    keywords: photo.keywords,
                                    rating: photo.rating,
                                    status: photo.status,
                                    hasXmp: photo.hasXmp,
                                    hasTakeout: photo.hasTakeout,
                                    createdAt: photo.createdAt,
                                    updatedAt: Int64(Date().timeIntervalSince1970)
                                )
                            }
                            return photo
                        }
                    } else {
                        // Non-destructive conversion: original RAW remains in library, new DNG added alongside
                        var newPhotos: [PhotoRecord] = []
                        for photo in self.photos {
                            newPhotos.append(photo)
                            if let newPath = pathMapping[photo.filePath] {
                                if !self.photos.contains(where: { $0.filePath == newPath }) {
                                    let newName = URL(fileURLWithPath: newPath).lastPathComponent
                                    let rep = report.reports.first(where: { $0.sourcePath == photo.filePath })
                                    let newSize = rep.map { Int64($0.dngSizeBytes) } ?? photo.fileSize
                                    let dngRecord = PhotoRecord(
                                        id: 0,
                                        filePath: newPath,
                                        fileName: newName,
                                        fileSize: newSize,
                                        modifiedTime: Int64(Date().timeIntervalSince1970),
                                        cameraMake: photo.cameraMake,
                                        cameraModel: photo.cameraModel,
                                        lensModel: photo.lensModel,
                                        iso: photo.iso,
                                        aperture: photo.aperture,
                                        shutterSpeed: photo.shutterSpeed,
                                        focalLength: photo.focalLength,
                                        takenTime: photo.takenTime,
                                        latitude: photo.latitude,
                                        longitude: photo.longitude,
                                        title: photo.title,
                                        description: photo.description,
                                        creator: photo.creator,
                                        copyright: photo.copyright,
                                        keywords: photo.keywords,
                                        rating: photo.rating,
                                        status: photo.status,
                                        hasXmp: true,
                                        hasTakeout: photo.hasTakeout,
                                        createdAt: Int64(Date().timeIntervalSince1970),
                                        updatedAt: Int64(Date().timeIntervalSince1970)
                                    )
                                    newPhotos.append(dngRecord)
                                }
                            }
                        }
                        self.photos = newPhotos
                    }
                    self.updateFilteredPhotos()
                    
                    // Update active selected photo
                    if let cur = self.selectedPhoto, let newPath = pathMapping[cur.filePath] {
                        if let updated = self.photos.first(where: { $0.filePath == newPath }) {
                            self.selectedPhoto = updated
                            self.currentlyLoadedPhotoPath = newPath
                        }
                    }
                    
                    // Update multi-selection set
                    if !self.selectedPhotoPaths.isEmpty {
                        var newSet = Set<String>()
                        for p in self.selectedPhotoPaths {
                            if let mapped = pathMapping[p] {
                                newSet.insert(mapped)
                            } else {
                                newSet.insert(p)
                            }
                        }
                        self.selectedPhotoPaths = newSet
                    }
                    
                    self.updateFilteredPhotos()
                }
            } catch {
                await MainActor.run {
                    self?.dngBatchErrorMessage = "Conversion failed: \(error.localizedDescription)"
                    self?.isConvertingDng = false
                }
            }
        }
    }
    
    public func cancelDngBatchConversion() {
        activeDngConversionTask?.cancel()
        activeDngConversionTask = nil
        isConvertingDng = false
        isDngConversionSheetPresented = false
    }
}

final class DngBatchProgressTracker: DngProgressCallback, @unchecked Sendable {
    let handler: (UInt32, UInt32, String) -> Void
    init(_ handler: @escaping (UInt32, UInt32, String) -> Void) {
        self.handler = handler
    }
    func onProgress(current: UInt32, total: UInt32, currentFile: String) {
        handler(current, total, currentFile)
    }
}
