import SwiftUI
import AppKit
import ImageIO
import LumenBoxBridge

public final class ThumbnailCache {
    public static let shared = ThumbnailCache()
    private let cache = NSCache<NSString, NSImage>()
    
    private init() {
        cache.countLimit = 500 // Cache up to 500 thumbnails in memory
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
        
        DispatchQueue.global(qos: .userInteractive).async {
            var loaded: NSImage? = nil
            
            // 1. Try Rust extractPreview (Embedded RAW JPEG)
            if let data = extractPreview(path: filePath), let img = NSImage(data: data) {
                loaded = img
            }
            
            // 2. Hardware-accelerated Apple Silicon ImageIO Thumbnail fallback
            if loaded == nil {
                let url = URL(fileURLWithPath: filePath) as CFURL
                if let src = CGImageSourceCreateWithURL(url, nil) {
                    let options: [CFString: Any] = [
                        kCGImageSourceCreateThumbnailFromImageAlways: true,
                        kCGImageSourceCreateThumbnailWithTransform: true,
                        kCGImageSourceThumbnailMaxPixelSize: 320
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
