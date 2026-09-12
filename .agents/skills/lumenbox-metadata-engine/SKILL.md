---
name: lumenbox-metadata-engine
description: >-
  Reference and workflow for managing metadata (IPTC Core, Dublin Core, EXIF, GPS, and XMP sidecars) across SQLite, Rust xmp.rs/raw.rs, and the Swift detail metadata panel. Use when modifying or adding metadata fields, XMP ingestion/serialization, or culling workflows.
---

# LumenBox Semantic Metadata & XMP Engine

This skill documents how metadata is ingested, presented, modified, and synchronized across SQLite, camera RAW EXIF, and `.xmp` sidecar files in LumenBox.

---

## 1. Five-Tier Semantic Hierarchy

Metadata in LumenBox is organized into 5 functional tiers:

1. **Tier 1: Triage & Culling (Always Visible at Top)**
   - Star Rating (`0`..`5`) — keyboard `0`..`5`
   - Flag Status (`pick`, `reject`, `none`) — keyboard `P`, `X`, `U`
   - High-contrast green (`pick`) and red (`reject`) active badge states.
2. **Tier 2: Content & IPTC (Editable)**
   - `Title` (`dc:title`, `photoshop:Headline`)
   - `Caption` / `Description` (`dc:description`)
   - `Keywords` (`dc:subject`, `lr:hierarchicalSubject`)
   - `Creator` / Photographer (`dc:creator`, `photoshop:By-line`)
   - `Copyright Notice` (`dc:rights`)
3. **Tier 3: Capture & Exposure EXIF (Read-Only)**
   - 4-tile Exposure Card: Shutter Speed, Aperture ($f$-number), ISO, Focal Length.
   - Camera Make & Model (`Make`, `Model`).
   - Lens Model (`LensModel`).
   - Capture Date & Time (`DateTimeOriginal`).
4. **Tier 4: Location & Geotagging**
   - GPS Latitude & Longitude with clickable "Open in Apple Maps" integration (`maps.apple.com/?q=lat,lon`).
5. **Tier 5: File & Sidecars**
   - File format, Megapixels, Dimensions, File Size in MB, and `.xmp` sidecar status.

---

## 2. Ingestion & Fallback Precedence

During catalog scanning or single-photo load:
1. **Camera RAW EXIF**: Primary camera tags (`Artist` $\to$ creator, `Copyright` $\to$ copyright, exposure values, GPS) are extracted in `raw.rs`.
2. **Sidecar `.xmp`**: If a sidecar file exists (`photo.jpg.xmp` or `photo.xmp`), values in the sidecar take precedence over camera EXIF.
3. **Database Cache**: Values are persisted to SQLite table `photos` for fast filtering in Grid view.

---

## 3. Two-Way Asynchronous Sync & Debouncing

When the photographer edits metadata in [`MetadataSidebarView.swift`](file:///Users/blikkie/src/LumenBox/LumenBoxApp/Sources/Views/MetadataSidebarView.swift):
1. **Debounce (400ms)**: Typing in text fields is debounced to prevent hammering disk I/O on every keystroke.
2. **SQLite Update**: Batch or single update via `db.updatePhotoMetadata(...)`.
3. **Sidecar Write**: Dispatched to background utility queue `syncXmp(...)`, serializing Dublin Core, Photoshop, and Adobe namespaces in XML UTF-8.
