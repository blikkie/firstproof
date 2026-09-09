import Foundation
import SwiftUI
import AppKit
import LumenBoxBridge

extension AppViewModel {
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
    
    @MainActor
    public func promptOpenFolder() {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.prompt = "Open Session"
        
        if panel.runModal() == .OK, let url = panel.url {
            addFolder(at: url.path)
        }
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
    
    func scopeKey(for scope: LibraryScope) -> String {
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
