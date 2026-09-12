import SwiftUI
import AppKit
import ImageIO
import CryptoKit
import FirstProofBridge

public final class ThumbnailCache {
    public static let shared = ThumbnailCache()
    private let cache = NSCache<NSString, NSImage>()
    public let fastQueue: OperationQueue
    public let renderQueue: OperationQueue
    public let queue: OperationQueue
    public let diskCacheURL: URL
    private let maxDiskCacheBytes: Int64 = 500 * 1024 * 1024 // 500 MB capacity limit
    
    private init() {
        cache.countLimit = 1200
        
        let ioQ = OperationQueue()
        ioQ.name = "photo.firstproof.ioThumbnailQueue"
        ioQ.maxConcurrentOperationCount = 2
        ioQ.qualityOfService = .utility
        self.queue = ioQ
        
        let fastQ = OperationQueue()
        fastQ.name = "photo.firstproof.fastThumbnailQueue"
        fastQ.maxConcurrentOperationCount = 6
        fastQ.qualityOfService = .userInitiated
        self.fastQueue = fastQ
        
        let renderQ = OperationQueue()
        renderQ.name = "photo.firstproof.renderThumbnailQueue"
        renderQ.maxConcurrentOperationCount = 2
        renderQ.qualityOfService = .utility
        self.renderQueue = renderQ
        
        let cachesDir = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first
            ?? URL(fileURLWithPath: NSTemporaryDirectory())
        let thumbDir = cachesDir.appendingPathComponent("photo.firstproof.app/thumbnails", isDirectory: true)
        try? FileManager.default.createDirectory(at: thumbDir, withIntermediateDirectories: true, attributes: nil)
        self.diskCacheURL = thumbDir
    }
    
    // MARK: - Deterministic Cache Key (SHA-256)
    public static func cacheKey(filePath: String, edits: EditSettings? = nil) -> String {
        var signature = filePath
        if let attrs = try? FileManager.default.attributesOfItem(atPath: filePath),
           let modDate = attrs[.modificationDate] as? Date {
            signature += "_\(modDate.timeIntervalSince1970)"
        }
        if let e = edits {
            signature += "_" + editsCacheSignature(edits: e)
        }
        let digest = SHA256.hash(data: Data(signature.utf8))
        return digest.compactMap { String(format: "%02x", $0) }.joined()
    }
    
    // MARK: - Two-Tier Lookup (RAM -> Disk)
    public func image(for filePath: String, edits: EditSettings? = nil) -> NSImage? {
        let key = ThumbnailCache.cacheKey(filePath: filePath, edits: edits)
        
        // Tier 1: In-memory NSCache
        if let mem = cache.object(forKey: key as NSString) {
            return mem
        }
        if edits == nil, let mem = cache.object(forKey: filePath as NSString) {
            return mem
        }
        
        // Tier 2: Persistent On-Disk Cache
        let fileURL = diskCacheURL.appendingPathComponent("\(key).jpg")
        if FileManager.default.fileExists(atPath: fileURL.path),
           let diskImg = NSImage(contentsOf: fileURL) {
            cache.setObject(diskImg, forKey: key as NSString)
            cache.setObject(diskImg, forKey: filePath as NSString)
            
            // Touch modification date asynchronously for LRU tracking
            queue.addOperation {
                try? FileManager.default.setAttributes([.modificationDate: Date()], ofItemAtPath: fileURL.path)
            }
            return diskImg
        }
        
        return nil
    }
    
    public func setImage(_ image: NSImage, for path: String) {
        cache.setObject(image, forKey: path as NSString)
    }
    
    // MARK: - Save Edited Thumbnail & Invalidate Views
    public func saveEditedThumbnail(_ image: NSImage, for filePath: String, edits: EditSettings, broadcast: Bool = true) {
        let key = ThumbnailCache.cacheKey(filePath: filePath, edits: edits)
        
        // 1. Update in-memory cache
        cache.setObject(image, forKey: key as NSString)
        cache.setObject(image, forKey: filePath as NSString)
        
        // 2. Persist to disk on background queue
        let targetURL = diskCacheURL.appendingPathComponent("\(key).jpg")
        queue.addOperation { [weak self] in
            guard let self = self else { return }
            if let tiff = image.tiffRepresentation,
               let rep = NSBitmapImageRep(data: tiff),
               let jpegData = rep.representation(using: .jpeg, properties: [.compressionFactor: 0.85]) {
                try? jpegData.write(to: targetURL, options: .atomic)
            }
            self.enforceDiskCacheLimitIfNeeded()
        }
        
        // 3. Post notification on Main Actor if requested
        if broadcast {
            DispatchQueue.main.async {
                NotificationCenter.default.post(name: .thumbnailDidChange, object: filePath)
            }
        }
    }
    
    public func fastCachedImage(for filePath: String) -> NSImage? {
        let fastKey = "fast_\(filePath)"
        if let mem = cache.object(forKey: fastKey as NSString) {
            return mem
        }
        if let mem = cache.object(forKey: filePath as NSString) {
            return mem
        }
        return nil
    }
    
    public func setFastCachedImage(_ image: NSImage, for filePath: String) {
        let fastKey = "fast_\(filePath)"
        cache.setObject(image, forKey: fastKey as NSString)
        cache.setObject(image, forKey: filePath as NSString)
    }
    
    public func removeImage(for path: String) {
        cache.removeObject(forKey: path as NSString)
        cache.removeObject(forKey: "fast_\(path)" as NSString)
    }
    
    // MARK: - Fast Embedded Preview Extraction (2-5 ms)
    public static func extractFastPreview(filePath: String, xmp: XmpDataRecord? = nil) -> NSImage? {
        var loaded: NSImage? = nil
        
        // 1. Fast-path for RAW photos: Extract embedded camera JPEG preview
        if let data = extractPreview(path: filePath) {
            if let src = CGImageSourceCreateWithData(data as CFData, nil) {
                let options: [CFString: Any] = [
                    kCGImageSourceCreateThumbnailFromImageAlways: true,
                    kCGImageSourceCreateThumbnailWithTransform: true,
                    kCGImageSourceThumbnailMaxPixelSize: 320,
                    kCGImageSourceShouldCacheImmediately: true
                ]
                if let cgThumb = CGImageSourceCreateThumbnailAtIndex(src, 0, options as CFDictionary) {
                    loaded = NSImage(cgImage: cgThumb, size: NSSize(width: cgThumb.width, height: cgThumb.height))
                }
            }
        }
        
        // 2. Fallback for non-RAW images (JPEGs, PNGs)
        if loaded == nil {
            let url = URL(fileURLWithPath: filePath) as CFURL
            if let src = CGImageSourceCreateWithURL(url, nil) {
                let options: [CFString: Any] = [
                    kCGImageSourceCreateThumbnailFromImageAlways: true,
                    kCGImageSourceCreateThumbnailWithTransform: true,
                    kCGImageSourceThumbnailMaxPixelSize: 320,
                    kCGImageSourceShouldCacheImmediately: true
                ]
                if let cgThumb = CGImageSourceCreateThumbnailAtIndex(src, 0, options as CFDictionary) {
                    loaded = NSImage(cgImage: cgThumb, size: NSSize(width: cgThumb.width, height: cgThumb.height))
                }
            }
        }
        
        // 3. Apply rotation / flips from XMP or EXIF
        if var base = loaded {
            let userRot = Int(xmp?.edits.rotation ?? 0)
            let userFlipH = xmp?.edits.flipHorizontal ?? false
            let userFlipV = xmp?.edits.flipVertical ?? false
            
            let meta = extractMetadata(path: filePath)
            let exifDec = meta?.orientation.map { decodeExifOrientation(orientation: $0) }
            let exifRot = Int(exifDec?.rotation ?? 0)
            let exifFlipH = exifDec?.flipHorizontal ?? false
            let exifFlipV = exifDec?.flipVertical ?? false
            
            let deltaRot = (userRot - exifRot + 360) % 360
            let deltaFlipH = userFlipH != exifFlipH
            let deltaFlipV = userFlipV != exifFlipV
            
            if deltaRot != 0 || deltaFlipH || deltaFlipV {
                base = base.transformed(rotationDegrees: CGFloat(deltaRot), flipH: deltaFlipH, flipV: deltaFlipV)
                loaded = base
            }
        }
        
        return loaded
    }
    
    // MARK: - LRU Cache Eviction
    private func enforceDiskCacheLimitIfNeeded() {
        let fm = FileManager.default
        guard let enumerator = fm.enumerator(
            at: diskCacheURL,
            includingPropertiesForKeys: [.fileSizeKey, .contentModificationDateKey],
            options: [.skipsHiddenFiles]
        ) else { return }
        
        var files: [(url: URL, size: Int64, date: Date)] = []
        var totalBytes: Int64 = 0
        
        for case let fileURL as URL in enumerator {
            if let res = try? fileURL.resourceValues(forKeys: [.fileSizeKey, .contentModificationDateKey]),
               let size = res.fileSize,
               let date = res.contentModificationDate {
                let s = Int64(size)
                totalBytes += s
                files.append((fileURL, s, date))
            }
        }
        
        if totalBytes > maxDiskCacheBytes {
            files.sort { $0.date < $1.date } // Oldest first
            let targetBytes = Int64(Double(maxDiskCacheBytes) * 0.8) // Prune oldest 20%
            var currentBytes = totalBytes
            for f in files {
                if currentBytes <= targetBytes { break }
                try? fm.removeItem(at: f.url)
                currentBytes -= f.size
            }
        }
    }
}

public struct AsyncThumbnailView: View {
    let filePath: String
    @State private var image: NSImage? = nil
    @State private var fastOperation: Operation? = nil
    @State private var renderOperation: Operation? = nil
    
    public init(filePath: String) {
        self.filePath = filePath
    }
    
    public var body: some View {
        ZStack {
            Color.black.opacity(0.2) // Neutral letterbox/pillarbox background
            
            if let image = image {
                Image(nsImage: image)
                    .resizable()
                    .aspectRatio(contentMode: .fit)
            } else {
                ProgressView()
                    .controlSize(.small)
            }
        }
        .clipped()
        .onAppear {
            loadThumbnail()
        }
        .onDisappear {
            // Cancel background decode if scrolled out of viewport
            fastOperation?.cancel()
            fastOperation = nil
            renderOperation?.cancel()
            renderOperation = nil
        }
        .onReceive(NotificationCenter.default.publisher(for: .thumbnailDidChange)) { note in
            if let targetPath = note.object as? String, targetPath == filePath {
                if let updated = ThumbnailCache.shared.image(for: filePath) {
                    self.image = updated
                } else {
                    self.image = nil
                    loadThumbnail()
                }
            }
        }
    }
    
    private func loadThumbnail() {
        let xmp = readXmp(path: filePath)
        let hasEdits = xmp.map { editsHasModifications(edits: $0.edits, asShotTemp: 5500.0, asShotTint: 0.0) } ?? false
        
        // Exact edited thumbnail hit (RAM or Disk)
        if let cached = ThumbnailCache.shared.image(for: filePath, edits: xmp?.edits) {
            self.image = cached
            return
        }
        
        // Check if fast preview is already in RAM cache
        if let fastCached = ThumbnailCache.shared.fastCachedImage(for: filePath) {
            self.image = fastCached
            if !hasEdits { return }
        }
        
        // STAGE 1: Instant embedded camera JPEG extraction (2-5 ms)
        let fastOp = BlockOperation()
        fastOp.queuePriority = .veryHigh
        fastOp.addExecutionBlock { [weak fastOp] in
            guard let fastOp = fastOp, !fastOp.isCancelled else { return }
            
            if let fastCached = ThumbnailCache.shared.fastCachedImage(for: filePath) {
                DispatchQueue.main.async {
                    if self.image == nil {
                        self.image = fastCached
                    }
                }
                return
            }
            
            if let fastImg = ThumbnailCache.extractFastPreview(filePath: filePath, xmp: xmp) {
                ThumbnailCache.shared.setFastCachedImage(fastImg, for: filePath)
                DispatchQueue.main.async {
                    if self.image == nil {
                        self.image = fastImg
                    }
                }
            }
        }
        self.fastOperation = fastOp
        ThumbnailCache.shared.fastQueue.addOperation(fastOp)
        
        // STAGE 2: If photo has custom edits, enqueue refinement render
        if hasEdits, let edits = xmp?.edits {
            let renderOp = BlockOperation()
            renderOp.queuePriority = .normal
            renderOp.addExecutionBlock { [weak renderOp] in
                guard let renderOp = renderOp, !renderOp.isCancelled else { return }
                
                // Check if another thread completed it
                if let cached = ThumbnailCache.shared.image(for: filePath, edits: edits) {
                    DispatchQueue.main.async {
                        self.image = cached
                    }
                    return
                }
                
                let key = ThumbnailCache.cacheKey(filePath: filePath, edits: edits)
                let tempThumbPath = ThumbnailCache.shared.diskCacheURL.appendingPathComponent("\(key).jpg").path
                let success = exportPhoto(
                    inputPath: filePath,
                    outputPath: tempThumbPath,
                    edits: edits,
                    lensCoeffs: nil,
                    quality: 80,
                    maxDimension: 320,
                    colorSpace: nil,
                    renderingIntent: nil,
                    ultraHdr: false
                )
                
                guard !renderOp.isCancelled else { return }
                
                if success, let thumbImg = NSImage(contentsOfFile: tempThumbPath) {
                    ThumbnailCache.shared.saveEditedThumbnail(thumbImg, for: filePath, edits: edits, broadcast: false)
                    DispatchQueue.main.async {
                        withAnimation(.easeInOut(duration: 0.15)) {
                            self.image = thumbImg
                        }
                    }
                }
            }
            self.renderOperation = renderOp
            ThumbnailCache.shared.renderQueue.addOperation(renderOp)
        }
    }
}

extension NSImage {
    public func transformed(rotationDegrees: CGFloat, flipH: Bool, flipV: Bool) -> NSImage {
        guard rotationDegrees != 0 || flipH || flipV else { return self }
        
        let rot = Int(rotationDegrees.rounded()) % 360
        let is90or270 = rot == 90 || rot == 270
        let newSize = is90or270 ? NSSize(width: size.height, height: size.width) : size
        
        let newImage = NSImage(size: newSize)
        newImage.lockFocus()
        
        let transform = NSAffineTransform()
        transform.translateX(by: newSize.width / 2.0, yBy: newSize.height / 2.0)
        transform.rotate(byDegrees: -CGFloat(rot))
        transform.scaleX(by: flipH ? -1.0 : 1.0, yBy: flipV ? -1.0 : 1.0)
        transform.translateX(by: -size.width / 2.0, yBy: -size.height / 2.0)
        transform.concat()
        
        draw(at: .zero, from: NSRect(origin: .zero, size: size), operation: .copy, fraction: 1.0)
        
        newImage.unlockFocus()
        return newImage
    }
}

extension Notification.Name {
    public static let thumbnailDidChange = Notification.Name("photo.firstproof.thumbnailDidChange")
}
