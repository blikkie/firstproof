import Foundation
import SwiftUI
import AppKit
import FirstProofBridge

extension AppViewModel {
    // MARK: - Scope Switching
    public func selectScope(_ scope: LibraryScope, preferredPhotoPath: String? = nil) {
        self.currentScope = scope
        switch scope {
        case .mainLibrary:
            UserDefaults.standard.set("main_library", forKey: "lastSelectedScope")
            openDirectory(at: mainLibraryPath, isMain: true, preferredPhotoPath: preferredPhotoPath)
        case .session(let folderPath):
            UserDefaults.standard.set(folderPath, forKey: "lastSelectedScope")
            openDirectory(at: folderPath, isMain: false, preferredPhotoPath: preferredPhotoPath)
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
    
    @discardableResult
    public func handleDroppedURLs(_ urls: [URL]) -> Bool {
        guard !urls.isEmpty else { return false }
        
        // Grab application and window focus immediately when receiving a drop from Finder
        DispatchQueue.main.async {
            NSApplication.shared.activate(ignoringOtherApps: true)
            if let window = NSApplication.shared.windows.first(where: { $0.canBecomeKey && $0.isVisible }) {
                window.makeKeyAndOrderFront(nil)
            }
        }
        
        let directories = urls.filter { url in
            var isDir: ObjCBool = false
            return FileManager.default.fileExists(atPath: url.path, isDirectory: &isDir) && isDir.boolValue
        }
        
        if !directories.isEmpty {
            for dir in directories {
                addFolder(at: dir.path)
            }
            return true
        }
        
        let supportedExtensions: Set<String> = [
            "cr2", "cr3", "nef", "arw", "dng", "orf", "rw2", "raf", "pef", "raw",
            "jpg", "jpeg", "png", "heic", "tiff", "tif"
        ]
        let imageFiles = urls.filter { url in
            supportedExtensions.contains(url.pathExtension.lowercased())
        }
        
        if !imageFiles.isEmpty {
            let parentDir = imageFiles.first?.deletingLastPathComponent().path
            promptImport(from: parentDir, selectedPaths: imageFiles.map(\.path))
            return true
        }
        
        return false
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
    public func openDirectory(at folderPath: String, isMain: Bool = false, preferredPhotoPath: String? = nil) {
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
        
        // Save any pending edits and cleanly checkpoint current catalog before changing directory
        saveCurrentEdits()
        checkpointDatabase()
        self.currentlyLoadedPhotoPath = nil

        // Reset state for new scope
        LookaheadPreloadService.shared.cancelPendingPreloads()
        PreviewRingBuffer.shared.clear()
        undoHistory.clearAll()
        self.photos = []
        self.selectedPhoto = nil
        self.previewImage = nil
        self.rawPreviewRecord = nil
        self.isIntermediatePreviewActive = false
        
        let catalogPath = resolveCatalogPath(folderPath: folderPath, portable: self.isExternalVolume)
        let t0 = CFAbsoluteTimeGetCurrent()
        
        activeCatalogTask?.cancel()
        activeCatalogTask = Task { [weak self] in
            guard let self = self else { return }
            do {
                let (database, cachedPhotos) = try await Task.detached(priority: .userInitiated) { () throws -> (FirstProofDatabase, [PhotoRecord]) in
                    let db = try FirstProofDatabase.open(dbPath: catalogPath)
                    let cached = db.getAllPhotos()
                    return (db, cached)
                }.value
                
                guard !Task.isCancelled else { return }
                self.db = database
                let tCached = CFAbsoluteTimeGetCurrent()
                print("⚡ [PERF] Cached photos loaded: \(cachedPhotos.count) photos in \(String(format: "%.1f", (tCached - t0) * 1000.0)) ms")
                
                if !cachedPhotos.isEmpty {
                    self.photos = cachedPhotos
                    if let target = preferredPhotoPath,
                       let match = cachedPhotos.first(where: { $0.filePath == target || URL(fileURLWithPath: $0.filePath).lastPathComponent == target }) {
                        self.selectPhoto(match)
                    } else if self.selectedPhoto == nil {
                        if let savedPath = self.lastActivePhoto(for: self.currentScope),
                           let saved = cachedPhotos.first(where: { $0.filePath == savedPath }) {
                            self.selectPhoto(saved)
                        } else if let first = cachedPhotos.first {
                            self.selectPhoto(first)
                        }
                    }
                }
                
                // 2. Background Batched Directory Indexer
                guard !Task.isCancelled else { return }
                let (count, finalPhotos) = await Task.detached(priority: .userInitiated) { () -> (UInt32, [PhotoRecord]) in
                    let indexedCount = database.indexFolder(folderPath: folderPath)
                    let all = database.getAllPhotos()
                    return (indexedCount, all)
                }.value
                
                guard !Task.isCancelled else { return }
                let tIndexed = CFAbsoluteTimeGetCurrent()
                print("📦 [PERF] Indexing complete: \(count) files in \(String(format: "%.1f", (tIndexed - tCached) * 1000.0)) ms")
                
                // 3. Publish Final Indexed Photos
                self.photos = finalPhotos
                self.isScanning = false
                self.scanningStatusMessage = nil
                if let target = preferredPhotoPath,
                   let match = finalPhotos.first(where: { $0.filePath == target || URL(fileURLWithPath: $0.filePath).lastPathComponent == target }) {
                    self.selectPhoto(match)
                } else if self.selectedPhoto == nil || !finalPhotos.contains(where: { $0.filePath == self.selectedPhoto?.filePath }) {
                    if let savedPath = self.lastActivePhoto(for: self.currentScope),
                       let saved = finalPhotos.first(where: { $0.filePath == savedPath }) {
                        self.selectPhoto(saved)
                    } else if let first = finalPhotos.first {
                        self.selectPhoto(first)
                    }
                }
            } catch {
                guard !Task.isCancelled else { return }
                print("Failed to open database at \(catalogPath): \(error)")
                // Self-healing fallback: if opening failed, clean corrupt catalog and retry once
                try? FileManager.default.removeItem(atPath: catalogPath)
                try? FileManager.default.removeItem(atPath: "\(catalogPath)-wal")
                try? FileManager.default.removeItem(atPath: "\(catalogPath)-shm")
                do {
                    let retryDb = try FirstProofDatabase.open(dbPath: catalogPath)
                    self.db = retryDb
                    _ = retryDb.indexFolder(folderPath: folderPath)
                    let freshPhotos = retryDb.getAllPhotos()
                    self.photos = freshPhotos
                    self.isScanning = false
                    self.scanningStatusMessage = nil
                    if let first = freshPhotos.first {
                        self.selectPhoto(first)
                    }
                    return
                } catch {
                    print("Failed to recreate database at \(catalogPath): \(error)")
                }
                self.photos = []
                self.isScanning = false
                self.scanningStatusMessage = nil
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
        var resolvedTargets = Array(selectedPhotoPaths)
        if resolvedTargets.isEmpty, let single = selectedPhoto?.filePath {
            resolvedTargets = [single]
        }
        guard !resolvedTargets.isEmpty else {
            completion(nil)
            return
        }
        let targets = resolvedTargets
        
        Task { [weak self] in
            guard let self = self else { return }
            do {
                let report = try await Task.detached(priority: .userInitiated) {
                    try exportPortablePackage(
                        sourcePhotoPaths: targets,
                        destinationFolder: destinationFolder,
                        moveFiles: moveFiles
                    )
                }.value
                
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
            } catch {
                print("Failed to export portable package: \(error)")
                completion(nil)
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
