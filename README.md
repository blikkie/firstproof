# First Proof

> **Pure capture. Honest color. Zero digital plastic.**  
> An open, honest RAW photo darkroom for macOS, Linux, and the Web.  
> [firstproof.photo](https://firstproof.photo)

---

## Why First Proof Exists

First Proof was built out of a desire for an honest photo editor. It should show the world as it actually looks and shouldn’t require a million micro-adjustments just to look acceptable—the first proof should come out right.

> *As a software engineer I generally know better than to do things that are unreasonable to vibe code, but I also like to take an idea, a cheap model, and see how far I can get.*

Along the way, the project adopted a strict invariant: **no magic numbers or artificial heuristics to make photos artificially look "good."** Every single transformation is grounded either in optical sensor color science or physical darkroom sensitometry.

Finally, First Proof is about taking true ownership of your photos away from recurring cloud subscriptions and opaque centralized catalogs. Everything runs 100% locally on your machine, with non-destructive edits stored directly in standard open `.xmp` sidecars.

---

## Key Principles

* **Truth to Capture (Color Science Over Slop)**:  
  Internal adjustments operate in **Linear Rec.2020 wide-gamut** color space ($Y = 0.2627 R + 0.6780 G + 0.0593 B$). Highlights desaturate naturally as they approach clipping via **AgX filmic tone mapping**, preventing the harsh digital hue shifts and plastic skin tones typical of consumer RAW engines.
* **Restraint Over Bloat (The Anti-150-Slider Stance)**:  
  Zero synthetic AI pixel generation, zero sky replacements, zero beauty filters, and zero cloud lock-in. A streamlined control deck focusing on sensor reality: Exposure, White Balance, Filmic Tone Curves, Restrained Saturation, Detail, Optics, and Crop.
* **Dual-Domain Performance Asymmetry ("Wall for 10 Years")**:  
  - *Interactive Culling & Viewport*: 60–120 FPS fluid navigation and 10–15ms photo transitions using 2×2 Bayer box binning and GPU viewport texture caching.  
  - *Archival Export*: *"You only export it once, and then it hangs on your wall for 10 years."* High-fidelity directional demosaicing (RCD), LittleCMS color management, and full 16-bit master rendering without computational shortcuts.
* **Print & Web First**:  
  While the pipeline leverages wide-gamut Rec.2020 and Apple Liquid Retina XDR screens, the default target is calibrated SDR reproduction that translates honestly to physical prints and the universal web.

---

## Three Ways to Use First Proof

### 1. In Your Browser ([firstproof.photo](https://firstproof.photo))
First Proof runs directly in modern desktop browsers via WebAssembly and WebGL 2:
- 100% client-side: your photos never leave your device.
- Uses the File System Access API (`window.showDirectoryPicker()`) to stream local folders of RAW photos.
- Edits and ratings write back directly to adjacent `.xmp` sidecars on disk.

### 2. Native macOS App (`FirstProof.app`)
A fast, dedicated desktop application built for Apple Silicon:
- SwiftUI interface with native Metal GPU shaders for live 60/120 FPS slider updates.
- Bundled Lensfun database (1,000+ cameras, 1,500+ lenses) for automatic lens distortion correction.
- Multi-threaded batch export engine.

### 3. Headless Darkroom CLI (`firstproof-cli`)
A standalone command-line tool for scripting, batch demosaicing, and headless workflows:
```bash
# Develop a camera RAW to high-quality JPEG with default AgX Vivid
firstproof-cli develop photo.cr2 -o developed.jpg --exposure +0.25

# Convert proprietary RAW (.CR2, .NEF, .ARW) to archival DNG
firstproof-cli dng photo.cr2

# Inspect sensor CFA, black/white levels, and calibration
firstproof-cli inspect photo.dng
```

---

## Installation & macOS Gatekeeper Notice

Pre-built binaries are available on the [Releases](https://github.com/blikkie/firstproof/releases) page:
* **macOS Disk Image**: `FirstProof-0.6.1-macos-arm64.dmg`
* **CLI Tarball**: `firstproof-cli-0.6.1-macos-arm64.tar.gz`

> **macOS Gatekeeper Notice**:  
> First Proof release builds are ad-hoc signed and not notarized through Apple's paid developer program ($99/year). When opening the app for the first time, macOS will display an unidentified developer warning.
>
> To launch:
> 1. Right-click (or Control-click) **FirstProof.app** in Finder and choose **Open**, then confirm.
> 2. Or go to **System Settings > Privacy & Security** and click **Open Anyway**.
> 3. Or clear the quarantine attribute via Terminal:
>    ```bash
>    xattr -cr /Applications/FirstProof.app
>    ```

---

## Repository Architecture

The project is structured as a unified monorepo sharing color science and domain logic across native desktop and web:

```
├── firstproof-engine/     # Shared pure-Rust core: CFA unpacking, AgX tonemapping, curves, XMP
├── firstproof-core/       # Desktop Rust library: demosaicing (RCD), LittleCMS, Lensfun, SQLite
├── FirstProofApp/         # Native macOS application (SwiftUI + Metal shaders)
├── FirstProofBridge/      # UniFFI bridge bindings connecting Rust core to Swift
├── firstproof-wasm/       # WebAssembly compilation target (wasm-bindgen / wasm-pack)
├── web/                   # Web darkroom & static release site (TypeScript + WebGL 2 + Vite)
├── scripts/               # Pre-flight verification and anti-slop sensor suite
└── docs/                  # Architecture specs, color science notes, and manifesto
```

---

## Building from Source

### Prerequisites
- **Rust**: 1.75+ (`rustup default stable`)
- **Swift / Xcode**: Swift 5.9+ / Xcode 15+ (macOS only)
- **Node.js**: 20+ (`npm`)
- **wasm-pack**: `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`

### 1. Build Native macOS App Bundle
```bash
# Compile Rust core, generate UniFFI bridge, and build FirstProof.app
make bundle

# The standalone application bundle is output to dist/FirstProof.app
open dist/FirstProof.app
```

### 2. Build Headless CLI
```bash
make build-cli
# Output binary: dist/firstproof-cli
./dist/firstproof-cli --help
```

### 3. Run Web Darkroom Locally
```bash
# 1. Compile WebAssembly engine
cd firstproof-wasm
wasm-pack build --target web --out-dir pkg
cd ..

# 2. Run local web server
cd web
npm install
npm run dev
# Open http://localhost:5173 in Chrome, Edge, or Safari
```

### 4. Run Verification Suite
First Proof enforces mathematical parity, license compliance, and anti-slop invariants via an automated sensor suite:
```bash
# Fast targeted tests (Rust engine)
cargo test -p firstproof-engine

# Web tests & typechecking
npm --prefix web test
npm --prefix web run typecheck

# Full pre-flight verification gatekeeper
./scripts/check.sh
```

---

## Lineage & Acknowledgments

First Proof stands on the shoulders of foundational open color science and darkroom systems:

* **Troy Sobotka**: Groundbreaking research on AgX color science, open filmic tone mapping, and natural highlight desaturation.
* **Marti Maria**: LittleCMS (`lcms2`) open-source color management engine.
* **Dave Coffin & RAW Decoding Lineage**: `dcraw`, `libraw`, and the `rawler` project for deciphering proprietary camera Bayer formats.
* **Torsten Bronger & Lensfun Community**: Comprehensive open lens database and optical correction algorithms (CC-BY-SA 3.0).
* **CIE & Adobe Systems**: DNG specification, Adobe DCP camera profile specifications, and CIE 1931/1976 colorimetric standards.

See [THIRDPARTY.md](THIRDPARTY.md) for full license texts and copyright notices.

---

## License

First Proof is open source software licensed under the [MIT License](LICENSE).  
The bundled lens calibration database in `assets/lensfun/` is aggregated from the Lensfun Project under the [Creative Commons Attribution-ShareAlike 3.0 Unported](assets/lensfun/LICENSE.txt) (CC-BY-SA 3.0) license.
