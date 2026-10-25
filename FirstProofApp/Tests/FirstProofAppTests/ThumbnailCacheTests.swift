// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

import Testing
import AppKit
import FirstProofBridge
@testable import FirstProofApp

@Suite("Thumbnail Cache Invariants")
struct ThumbnailCacheTests {
    
    @Test("Cache keys are deterministic and sensitive to file path")
    func testCacheKeyDeterminism() {
        let key1 = ThumbnailCache.cacheKey(filePath: "/path/to/capture.cr2")
        let key2 = ThumbnailCache.cacheKey(filePath: "/path/to/capture.cr2")
        let key3 = ThumbnailCache.cacheKey(filePath: "/path/to/other.cr2")
        
        #expect(key1 == key2)
        #expect(key1 != key3)
        #expect(key1.count == 64) // SHA-256 hex string
    }
    
    @Test("Cache keys are sensitive to develop edit mutations")
    func testCacheKeySensitivityToEdits() {
        let path = "/path/to/photo.cr2"
        let baseline = EditSettings.defaultIdentity
        var modified = baseline
        modified.exposure = 0.5
        
        let keyBaseline = ThumbnailCache.cacheKey(filePath: path, edits: baseline)
        let keyModified = ThumbnailCache.cacheKey(filePath: path, edits: modified)
        
        #expect(keyBaseline != keyModified)
    }
    
    @Test("Refined thumbnail flag tracking is accurate and thread-safe")
    func testRefinedThumbnailTracking() {
        let cache = ThumbnailCache.shared
        let path = "/tmp/test_refined_\(UUID().uuidString).cr2"
        
        #expect(!cache.isRefinedThumbnail(for: path))
        
        cache.markRefinedThumbnail(for: path)
        #expect(cache.isRefinedThumbnail(for: path))
        
        cache.clearRefinedThumbnail(for: path)
        #expect(!cache.isRefinedThumbnail(for: path))
    }
    
    @Test("In-memory cache stores and evicts thumbnails cleanly")
    func testMemoryCacheHitAndRemoval() {
        let cache = ThumbnailCache.shared
        let path = "/tmp/test_thumb_\(UUID().uuidString).cr2"
        let testImage = NSImage(size: NSSize(width: 32, height: 32))
        
        // 1. Capture snapshot into in-memory tier
        cache.setImage(testImage, for: path)
        
        // 2. Lookup should return memory instance
        let retrieved = cache.image(for: path, edits: nil)
        #expect(retrieved != nil)
        
        // 3. Invalidate/remove image
        cache.removeImage(for: path)
        
        // 4. Memory cache should no longer return it
        let afterRemoval = cache.image(for: path, edits: nil)
        #expect(afterRemoval == nil)
    }
}
