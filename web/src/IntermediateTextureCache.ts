/**
 * High-Resolution Rendered Intermediate Texture Cache for First Proof Web
 *
 * Implements 1:1 parity with macOS ThumbnailCache.shared.intermediateImage.
 * Caches high-resolution developed frames (already having exposure, AgX, curves,
 * and geometry baked into pixels) in GPU-backed ImageBitmaps.
 *
 * When navigating back to previously viewed or pre-rendered photos:
 * - 0ms instant display from memory
 * - Passed to WebGL with u_isIntermediate = 1 (bypassing fragment color transforms
 *   and vertex rotation to maintain strict intermediate idempotence)
 * - Zero camera-JPEG flashing and zero redundant WASM re-decoding
 */

import type { EditSettingsPayload } from './WasmBridge';

export interface CachedIntermediate {
  bitmap: ImageBitmap;
  width: number;
  height: number;
  editsHash: string;
  timestamp: number;
}

export class IntermediateTextureCache {
  private static cache: Map<string, CachedIntermediate> = new Map();
  private static MAX_ENTRIES = 20; // Up to 20 full-resolution intermediate frames in memory (~100MB VRAM)

  /**
   * Computes a deterministic compact fingerprint hash of the edit parameters
   */
  public static hashEdits(edits: EditSettingsPayload): string {
    return JSON.stringify(edits);
  }

  /**
   * Retrieves a cached high-resolution intermediate bitmap matching the file and edits
   */
  public static get(
    filePath: string,
    edits?: EditSettingsPayload,
  ): CachedIntermediate | null {
    const entry = this.cache.get(filePath);
    if (!entry) return null;

    if (edits) {
      const currentHash = this.hashEdits(edits);
      if (entry.editsHash !== currentHash) {
        return null; // Stale edits - re-render required
      }
    }

    entry.timestamp = Date.now(); // Update LRU timestamp
    return entry;
  }

  /**
   * Stores a newly rendered intermediate bitmap in the LRU cache
   */
  public static set(
    filePath: string,
    edits: EditSettingsPayload,
    bitmap: ImageBitmap,
  ): void {
    // Evict oldest entries if cache exceeds capacity
    if (this.cache.size >= this.MAX_ENTRIES && !this.cache.has(filePath)) {
      let oldestKey: string | null = null;
      let oldestTime = Infinity;
      for (const [key, item] of this.cache.entries()) {
        if (item.timestamp < oldestTime) {
          oldestTime = item.timestamp;
          oldestKey = key;
        }
      }
      if (oldestKey) {
        const oldItem = this.cache.get(oldestKey);
        oldItem?.bitmap.close(); // Release underlying GPU memory
        this.cache.delete(oldestKey);
      }
    }

    const previous = this.cache.get(filePath);
    if (previous && previous.bitmap !== bitmap) {
      previous.bitmap.close();
    }

    this.cache.set(filePath, {
      bitmap,
      width: bitmap.width,
      height: bitmap.height,
      editsHash: this.hashEdits(edits),
      timestamp: Date.now(),
    });
  }

  /**
   * Invalidate a single photo from intermediate cache
   */
  public static invalidate(filePath: string): void {
    const entry = this.cache.get(filePath);
    if (entry) {
      entry.bitmap.close();
      this.cache.delete(filePath);
    }
  }

  /**
   * Clear all cached intermediate textures and release GPU memory
   */
  public static clear(): void {
    for (const item of this.cache.values()) {
      item.bitmap.close();
    }
    this.cache.clear();
  }
}
