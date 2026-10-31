import Foundation
import FirstProofBridge

public struct MetadataUpdate: Sendable {
    public var title: String?
    public var description: String?
    public var creator: String?
    public var copyright: String?
    public var keywords: [String]
    
    public init(
        title: String?,
        description: String?,
        creator: String?,
        copyright: String?,
        keywords: [String]
    ) {
        self.title = title
        self.description = description
        self.creator = creator
        self.copyright = copyright
        self.keywords = keywords
    }
}

/// Unified serialized coordinator for all XMP sidecar read-merge-write operations.
/// Serializes disk access across concurrent rating, flag, develop edits, and metadata edits
/// to prevent race conditions and field clobbering.
public actor XmpSyncCoordinator {
    public static let shared = XmpSyncCoordinator()
    
    public func sync(
        filePath: String,
        rating: Int32?? = nil,
        status: String? = nil,
        edits: EditSettings? = nil,
        metadata: MetadataUpdate? = nil,
        inMemoryRating: Int32? = nil,
        inMemoryStatus: String? = nil,
        inMemoryLat: Double? = nil,
        inMemoryLon: Double? = nil,
        fallbackTitle: String? = nil,
        fallbackDescription: String? = nil,
        fallbackCreator: String? = nil,
        fallbackCopyright: String? = nil,
        fallbackKeywords: [String] = []
    ) {
        let existingXmp = readXmp(path: filePath)
        
        // 1. Edits resolution
        let finalEdits: EditSettings
        if let e = edits {
            finalEdits = e
        } else if let e = existingXmp?.edits {
            finalEdits = e
        } else {
            var baseline = EditSettings.defaultIdentity
            if let asShot = getAsShotWhiteBalance(path: filePath) {
                baseline.temperature = asShot.temperature
                baseline.tint = asShot.tint
            }
            if let orient = extractMetadata(path: filePath)?.orientation {
                let dec = decodeExifOrientation(orientation: orient)
                baseline.rotation = Int32(dec.rotation)
                baseline.flipHorizontal = dec.flipHorizontal
                baseline.flipVertical = dec.flipVertical
            }
            baseline.lensCorrectionEnabled = UserDefaults.standard.object(forKey: "defaultAutoDistortion") as? Bool ?? false
            baseline.tcaCorrectionEnabled = UserDefaults.standard.object(forKey: "defaultAutoTca") as? Bool ?? true
            baseline.vignettingCorrectionEnabled = UserDefaults.standard.object(forKey: "defaultAutoVignetting") as? Bool ?? false
            finalEdits = baseline
        }
        
        // 2. Metadata resolution (cleared fields stay cleared, never resurrecting from existing XMP)
        let finalTitle: String?
        let finalDescription: String?
        let finalCreator: String?
        let finalCopyright: String?
        let finalKeywords: [String]
        
        if let m = metadata {
            finalTitle = (m.title?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false) ? m.title : nil
            finalDescription = (m.description?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false) ? m.description : nil
            finalCreator = (m.creator?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false) ? m.creator : nil
            finalCopyright = (m.copyright?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false) ? m.copyright : nil
            finalKeywords = m.keywords
        } else if let existing = existingXmp {
            finalTitle = existing.title
            finalDescription = existing.description
            finalCreator = existing.creator
            finalCopyright = existing.copyright
            finalKeywords = existing.keywords
        } else {
            finalTitle = (fallbackTitle?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false) ? fallbackTitle : nil
            finalDescription = (fallbackDescription?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false) ? fallbackDescription : nil
            finalCreator = (fallbackCreator?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false) ? fallbackCreator : nil
            finalCopyright = (fallbackCopyright?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false) ? fallbackCopyright : nil
            finalKeywords = fallbackKeywords
        }
        
        // 3. Rating resolution
        let finalRating: Int32?
        if let explicitRating = rating {
            finalRating = explicitRating
        } else {
            finalRating = existingXmp?.rating ?? inMemoryRating
        }
        
        // 4. Status resolution
        let finalStatus: String
        if let s = status {
            finalStatus = s
        } else if let s = existingXmp?.status, !s.isEmpty && s.lowercased() != "none" {
            finalStatus = s
        } else if let s = inMemoryStatus, !s.isEmpty && s.lowercased() != "none" {
            finalStatus = s
        } else {
            finalStatus = "none"
        }
        
        let finalLat = existingXmp?.latitude ?? inMemoryLat
        let finalLon = existingXmp?.longitude ?? inMemoryLon
        
        let record = XmpDataRecord(
            rating: finalRating,
            status: finalStatus,
            title: finalTitle,
            description: finalDescription,
            creator: finalCreator,
            copyright: finalCopyright,
            keywords: finalKeywords,
            latitude: finalLat,
            longitude: finalLon,
            edits: finalEdits
        )
        _ = writeXmp(path: filePath, record: record)
    }
}
