import SwiftUI
import AppKit
import ImageIO
import LumenBoxBridge

public final class ThumbnailCache {
    public static let shared = ThumbnailCache()
    private let cache = NSCache<NSString, NSImage>()
    public let queue: OperationQueue
    
    private init() {
        cache.countLimit = 800
        
        let q = OperationQueue()
        q.name = "com.lumenbox.thumbnailQueue"
        q.maxConcurrentOperationCount = 4 // Limit concurrency to prevent CPU/memory thrashing
        q.qualityOfService = .userInitiated
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
    }
    
    private func loadThumbnail() {
        if let cached = ThumbnailCache.shared.image(for: filePath) {
            self.image = cached
            return
        }
        
        ThumbnailCache.shared.queue.addOperation {
            // Check cache again in case it was populated while waiting in queue
            if let cached = ThumbnailCache.shared.image(for: filePath) {
                DispatchQueue.main.async {
                    self.image = cached
                }
                return
            }
            
            var loaded: NSImage? = nil
            
            // Fast-path: Hardware-accelerated ImageIO downsampled decode (320px max dimension)
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
            
            // Fallback: Rust extractPreview downsampled via ImageIO
            if loaded == nil, let data = extractPreview(path: filePath) {
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
            
            if let img = loaded {
                ThumbnailCache.shared.setImage(img, for: filePath)
                DispatchQueue.main.async {
                    self.image = img
                }
            }
        }
    }
}
