# Third-Party Notices & Open-Source Attributions

First Proof is dedicated to photographic restraint, honest color, and pure capture: *"Pure capture. Honest color. Zero digital plastic."*

The application code, algorithms, and Rust core engine of First Proof are licensed under the **MIT License** (see [LICENSE](LICENSE)). This document provides notices, attributions, and license summaries for all third-party libraries, datasets, and open-source projects incorporated into or used by First Proof.

---

## 🏛️ Foundational Lineage & Architectural Giants

First Proof stands on the shoulders of open-source visionaries and color scientists whose decades of research make honest optical computing possible:

1. **Troy Sobotka & The AgX Color Science Project**:
   - **Research**: Open-domain filmic tone mapping, wide-gamut dynamic range management, and natural highlight desaturation.
   - **Role in First Proof**: Core filmic tone mapping architecture in `AgxVivid` and `AgxNeutral` shaders.
   - **License**: MIT / Apache-2.0.

2. **Martí Maria & LittleCMS (`lcms2`)**:
   - **Research**: Industrial-grade, high-precision CMM (Color Management Module) and ICC profile transformation engine.
   - **Role in First Proof**: End-to-end ICC profile embedding and soft-proofing color transforms in export pipelines.
   - **License**: MIT License.

3. **Dave Coffin & RAW Decoding Pioneers**:
   - **Research**: `dcraw`, `libraw`, and the pure-Rust `rawler` ecosystem for reversing and decoding proprietary camera CFA sensor formats.
   - **Role in First Proof**: Unpacking raw Bayer and X-Trans camera sensors (CR2, CR3, ARW, NEF, RAF, RW2, ORF, PEF).
   - **License**: LGPL-2.1 / MIT.

4. **Torsten Bronger & The Lensfun Community**:
   - **Research**: Authoritative mathematical database of camera sensor crop factors and optical lens distortion/TCA/vignetting calibrations.
   - **Role in First Proof**: Real hardware calibration matching and distortion correction in the Optics workspace.
   - **License**: Creative Commons Attribution-ShareAlike 3.0 Unported (CC-BY-SA 3.0).

---

## 🔬 Dataset Carve-Outs

### Lensfun Lens Calibration Database (`assets/lensfun/`)
- **Upstream Project**: [Lensfun Database](https://lensfun.github.io/)
- **License**: Creative Commons Attribution-ShareAlike 3.0 Unported ([CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/))
- **Terms & Scope**:
  - The pre-indexed lens and camera calibration profiles bundled in `assets/lensfun/lensfun.json` are derived from the Lensfun project.
  - In accordance with collective work and mere aggregation principles under copyright law, this calibration dataset is quarantined within `assets/lensfun/` and does not extend copyleft obligations to First Proof application source code or binary distributions.
  - Complete license text is provided in `assets/lensfun/LICENSE.txt`.

---

## 📦 Core Rust Dependencies

All Rust dependencies are audited and enforced deterministically via `cargo-deny` (`deny.toml`) and `cargo-audit`.

| Package | Version | License | Primary Purpose |
| :--- | :--- | :--- | :--- |
| `rawler` | 0.8 | LGPL-2.1 | Pure-Rust RAW sensor decoding and metadata extraction |
| `lcms2` / `lcms2-sys` | 6.2 | MIT | LittleCMS2 ICC color management integration |
| `image` | 0.25 | MIT / Apache-2.0 | Pixel buffer containers and image decoding/encoding |
| `uniffi` | 0.28 | MPL-2.0 | High-performance Swift/AppKit and FFI bridging |
| `rusqlite` | 0.31 | MIT | Embedded SQLite catalog database management |
| `kamadak-exif` | 0.5 | BSD-2-Clause | EXIF metadata parsing from camera files |
| `xmp-writer` | 0.3 | MIT / Apache-2.0 | Non-destructive XMP sidecar serialization |
| `roxmltree` | 0.19 | MIT / Apache-2.0 | Fast, zero-allocation XML document parsing |
| `serde` / `serde_json` | 1.0 | MIT / Apache-2.0 | Serialization for edits, catalogs, and Lensfun profiles |
| `rayon` | 1.12 | MIT / Apache-2.0 | Multi-core parallel CPU demosaicing and batch exports |
| `sha2` | 0.10 | MIT / Apache-2.0 | Cryptographic SHA-256 asset and catalog hashing |
| `palette` | 0.7 | MIT / Apache-2.0 | Colorimetric mathematics, CIE XYZ, and color spaces |
| `bayer` | 0.1 | MIT / Apache-2.0 | Bayer color filter array demosaicing primitives |
| `jxl-oxide` | 0.12 | BSD-3-Clause | JPEG XL high-fidelity image decompression |
| `img-parts` | 0.4 | MIT / Apache-2.0 | ICC profile segment insertion for JPEG/TIFF/PNG exports |
| `glam` | 0.27 | MIT / Apache-2.0 | SIMD-accelerated 2D/3D affine geometry math |
| `chrono` | 0.4 | MIT / Apache-2.0 | ISO 8601 timestamps and EXIF date conversions |
| `uuid` | 1.0 | MIT / Apache-2.0 | UUID generation for active decode cancellation tokens |

---

## 🌐 Web & JavaScript Dependencies

| Package | Version | License | Primary Purpose |
| :--- | :--- | :--- | :--- |
| `typescript` | 5.4 | Apache-2.0 | Static typechecking for web darkroom |
| `vite` | 5.2 | MIT | Build tooling and development server |
| `vitest` | 2.1 | MIT | Unit and behavioral test runner |
| `happy-dom` | 20.14 | MIT | Headless DOM simulation for shortcut tests |
| `@biomejs/biome` | 2.5 | MIT / Apache-2.0 | Fast JavaScript/TypeScript linter and formatter |

---

## 🔒 Security & Vulnerability Auditing

To maintain software integrity:
1. **License Compliance**: Validated continuously using `cargo deny check licenses` against a strict permissive whitelist (`MIT`, `Apache-2.0`, `BSD-2/3`, `MPL-2.0`, `CC0`, `Zlib`).
2. **Advisory Monitoring**: Verified continuously via `cargo audit` against the RustSec Advisory Database.
3. **Secret Hygiene**: Scanned continuously with `gitleaks detect` to guarantee no credentials, tokens, or private keys enter the source tree.
