# ADR-0005: XMP Sidecars as Archival Truth, SQLite as Ephemeral Index

- **Status**: Accepted
- **Date**: 2026-03-25
- **Deciders**: blikkie, Antigravity
- **Consulted / Impacted Subsystems**: `firstproof-core/src/xmp.rs`, `firstproof-core/src/db.rs`, `FirstProofApp`, `web`

## Context & Problem Statement
A photo darkroom must store non-destructive edit parameters, ratings, color labels, flags, and IPTC/EXIF metadata. Photographers frequently store photos across multiple external hard drives, move folders, work across different computers, or use multiple tools (Lightroom, Capture One, DigiKam). If edits and metadata live exclusively in a monolithic centralized database, moving photo files or wiping the database destroys the user's edits.

## Decision Drivers
- **Archival portability**: If an external drive containing photos and sidecars is plugged into a different machine running First Proof, all edits and metadata must immediately load without requiring catalog export/import dances.
- **Interoperability**: Edits and metadata should adhere to open industry standards (Adobe XMP, IPTC Core, Dublin Core) where possible.
- **Zero lock-in**: Deleting the First Proof database file must never result in data loss.

## Considered Options
- **Option 1**: Monolithic SQLite database storing all photo metadata and edits in relational tables; no sidecar files.
- **Option 2**: In-place RAW modification (writing tags directly into EXIF/TIFF headers of original RAW files).
- **Option 3**: Standard industry `.xmp` sidecars next to RAW captures as the primary durable source of truth, with SQLite / browser `localStorage` acting as an ephemeral query cache and fast index.

## Decision Outcome
Chosen option: **Option 3 (XMP as Source of Truth, SQLite as Ephemeral Index)**.
All adjustments, culling states (pick/reject/stars), and metadata are serialized to standard XML/XMP sidecars (`<filename>.xmp`). The catalog database (`catalog.db` on macOS, `<volume>/.firstproof/catalog.db` on external volumes, `localStorage` on web) is treated as a disposable, rebuildable acceleration index.

### Positive Consequences
- True portability across drives and operating systems.
- Safe from database corruption: wiping `catalog.db` and re-indexing regenerates full state from `.xmp` sidecars.
- Interoperable with third-party DAMs and raw converters reading XMP namespaces (`crs:`, `dc:`, `photoshop:`).

### Negative Consequences & Accepted Trade-offs
- File system clutter: every edited photo has an accompanying `.xmp` file.
- Changes made externally must be detected and re-synced into the SQLite cache via file modification time stamps (`mtime`).

## Implementation Links & Code Anchors
- XMP Serialization & Parsing: [firstproof-engine/src/xmp.rs](../../firstproof-engine/src/xmp.rs)
- Catalog Database: [firstproof-core/src/db.rs](../../firstproof-core/src/db.rs)
- Web Sidecar Ingestion: [web/src/services/PhotoLoaderService.ts](../../web/src/services/PhotoLoaderService.ts)

