import Foundation
import AppKit
import ImageIO
import FirstProofBridge

/// Navigation direction for ordering lookahead prefetch priority.
public enum NavigationDirection: Sendable {
    case forward
    case backward
    case neutral
}

/// Bounded, thread-safe in-memory ring buffer (LRU) for decoded raw previews.
/// Prevents repetitive decodes and enables 0ms perceived navigation during burst culling.
public final class PreviewRingBuffer: @unchecked Sendable {
    public static let shared = PreviewRingBuffer(capacity: 5)
    
    private let lock = NSLock()
    private let capacity: Int
    private var cache: [String: RawPreviewRecord] = [:]
    private var lruOrder: [String] = []
    
    public init(capacity: Int = 5) {
        self.capacity = max(1, capacity)
    }
    
    /// Retrieve a pre-decoded RawPreviewRecord if present, updating its LRU position.
    public func getRawPreview(for path: String) -> RawPreviewRecord? {
        lock.lock()
        defer { lock.unlock() }
        
        guard let record = cache[path] else { return nil }
        // Move path to the back of LRU order (most recently used)
        if let idx = lruOrder.firstIndex(of: path) {
            lruOrder.remove(at: idx)
            lruOrder.append(path)
        }
        return record
    }
    
    /// Store a pre-decoded RawPreviewRecord, evicting the least-recently used record if capacity is exceeded.
    public func storeRawPreview(_ record: RawPreviewRecord, for path: String) {
        lock.lock()
        defer { lock.unlock() }
        
        if cache[path] != nil {
            cache[path] = record
            if let idx = lruOrder.firstIndex(of: path) {
                lruOrder.remove(at: idx)
                lruOrder.append(path)
            }
            return
        }
        
        // Evict oldest entries until within capacity limit
        while lruOrder.count >= capacity {
            let oldest = lruOrder.removeFirst()
            cache.removeValue(forKey: oldest)
        }
        
        cache[path] = record
        lruOrder.append(path)
    }
    
    /// Fast check if a record is already present in the ring buffer.
    public func contains(path: String) -> Bool {
        lock.lock()
        defer { lock.unlock() }
        return cache[path] != nil
    }
    
    /// Invalidate a specific path (e.g. after edit or deletion).
    public func remove(for path: String) {
        lock.lock()
        defer { lock.unlock() }
        cache.removeValue(forKey: path)
        if let idx = lruOrder.firstIndex(of: path) {
            lruOrder.remove(at: idx)
        }
    }
    
    /// Clear all records from the ring buffer.
    public func clear() {
        lock.lock()
        defer { lock.unlock() }
        cache.removeAll()
        lruOrder.removeAll()
    }
    
    /// Current number of stored previews.
    public var count: Int {
        lock.lock()
        defer { lock.unlock() }
        return cache.count
    }
}

/// Asynchronous background prefetch worker for adjacent neighbor photos.
/// Warmed frames are placed into `PreviewRingBuffer` and `ThumbnailCache` on low-priority queues.
public final class LookaheadPreloadService: @unchecked Sendable {
    public static let shared = LookaheadPreloadService()
    
    private let lookaheadQueue: DispatchQueue
    private let lock = NSLock()
    private var activePreloadToken: UUID = UUID()
    
    private init() {
        self.lookaheadQueue = DispatchQueue(
            label: "photo.firstproof.lookaheadQueue",
            qos: .utility
        )
    }
    
    /// Cancel any scheduled or running lookahead tasks.
    public func cancelPendingPreloads() {
        lock.lock()
        activePreloadToken = UUID()
        lock.unlock()
    }
    
    private func isTokenActive(_ token: UUID) -> Bool {
        lock.lock()
        defer { lock.unlock() }
        return activePreloadToken == token
    }
    
    /// Asynchronously preload neighbor frames around `currentIndex` within `photos`.
    /// Priority order is determined by `direction`.
    public func preloadNeighbors(
        around currentIndex: Int,
        in photos: [PhotoRecord],
        direction: NavigationDirection = .neutral
    ) {
        guard !photos.isEmpty else { return }
        
        // 1. Determine prioritized target indices based on direction
        let rawIndices: [Int]
        switch direction {
        case .forward:
            rawIndices = [currentIndex + 1, currentIndex + 2, currentIndex - 1]
        case .backward:
            rawIndices = [currentIndex - 1, currentIndex - 2, currentIndex + 1]
        case .neutral:
            rawIndices = [currentIndex + 1, currentIndex - 1, currentIndex + 2, currentIndex - 2]
        }
        
        // 2. Filter to valid, in-bounds, unique non-current indices
        var seen = Set<Int>()
        var targetIndices: [Int] = []
        for idx in rawIndices {
            if idx >= 0 && idx < photos.count && idx != currentIndex && !seen.contains(idx) {
                seen.insert(idx)
                targetIndices.append(idx)
            }
        }
        
        guard !targetIndices.isEmpty else { return }
        
        // 3. Atomically issue a new token to supersede any previous lookahead task
        let token = UUID()
        lock.lock()
        activePreloadToken = token
        lock.unlock()
        
        // Snapshot target photos for thread safety
        let candidates = targetIndices.compactMap { idx -> PhotoRecord? in
            guard idx >= 0 && idx < photos.count else { return nil }
            return photos[idx]
        }
        
        // 4. Dispatch prefetch sequence to low-priority background queue
        lookaheadQueue.async { [weak self] in
            guard let self = self else { return }
            
            for candidate in candidates {
                // Check if this prefetch job has been superseded by user navigation
                guard self.isTokenActive(token) else { return }
                
                let path = candidate.filePath
                
                if candidate.isRaw {
                    // Check if already in ring buffer
                    if PreviewRingBuffer.shared.contains(path: path) {
                        continue
                    }
                    
                    // Warm up in-memory intermediate if present on disk
                    _ = ThumbnailCache.shared.intermediateImage(for: path, edits: nil)
                    
                    // Perform high-throughput 2x2 Bayer box binned decode (10-15ms)
                    if let binnedRecord = decodeRawPreviewBinned(path: path) {
                        guard self.isTokenActive(token) else { return }
                        PreviewRingBuffer.shared.storeRawPreview(binnedRecord, for: path)
                    }
                } else {
                    // Non-RAW (JPEG, TIFF, PNG): warm up ThumbnailCache memory cache
                    if ThumbnailCache.shared.image(for: path) == nil {
                        let url = URL(fileURLWithPath: path) as CFURL
                        if let src = CGImageSourceCreateWithURL(url, nil),
                           let cgThumb = CGImageSourceCreateImageAtIndex(src, 0, nil) {
                            let img = NSImage(cgImage: cgThumb, size: NSSize(width: cgThumb.width, height: cgThumb.height))
                            ThumbnailCache.shared.setImage(img, for: path)
                        }
                    }
                }
            }
        }
    }
}
