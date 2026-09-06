import SwiftUI
import AppKit
import ImageIO
import LumenBoxBridge

public final class ThumbnailCache {
    public static let shared = ThumbnailCache()
    private let cache = NSCache<NSString, NSImage>()
    public let queue: OperationQueue
    
    private init() {
        cache.countLimit = 1200
        
        let q = OperationQueue()
        q.name = "com.lumenbox.thumbnailQueue"
        q.maxConcurrentOperationCount = 3
        q.qualityOfService = .utility
        self.queue = q
    }
    
    public func image(for path: String) -> NSImage? {
        cache.object(forKey: path as NSString)
    }
    
    public func setImage(_ image: NSImage, for path: String) {
        cache.setObject(image, forKey: path as NSString)
    }
}

public struct AsyncThumbnailView: View {
    let filePath: String
    @State private var image: NSImage? = nil
    @State private var loadOperation: Operation? = nil
    
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
            loadOperation?.cancel()
            loadOperation = nil
        }
    }
    
    private func loadThumbnail() {
        if let cached = ThumbnailCache.shared.image(for: filePath) {
            self.image = cached
            return
        }
        
        let op = BlockOperation()
        op.queuePriority = .veryHigh // Prioritize active viewport thumbnails
        
        op.addExecutionBlock { [weak op] in
            guard let op = op, !op.isCancelled else { return }
            
            if let cached = ThumbnailCache.shared.image(for: filePath) {
                DispatchQueue.main.async {
                    self.image = cached
                }
                return
            }
            
            var loaded: NSImage? = nil
            
            // Fast-path: Extract embedded camera JPEG preview (exact camera color profile, zero raw distortion)
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
            
            // Fallback for non-RAW images (JPEGs, PNGs)
            if loaded == nil && !op.isCancelled {
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
            
            if !op.isCancelled {
                let finalImg = loaded ?? NSImage(size: NSSize(width: 1, height: 1))
                ThumbnailCache.shared.setImage(finalImg, for: filePath)
                if let validImg = loaded {
                    DispatchQueue.main.async {
                        self.image = validImg
                    }
                }
            }
        }
        
        self.loadOperation = op
        ThumbnailCache.shared.queue.addOperation(op)
    }
}
