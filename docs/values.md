# First Proof: Brand Manifesto & Core Values

*Domain:* [firstproof.photo](https://firstproof.photo)  
*Tagline:* **Pure capture. Honest color. Zero digital plastic.**

---

## 1. The Value Proposition

First Proof is an editorial RAW developer and photo manager for macOS built on one core conviction: **the best photographs look right on the first proof.**

In the traditional darkroom and editorial print worlds, the "first proof" (or work print) is the direct, unmanipulated print from the negative. If the exposure, lens, and chemistry are sound, the image is already compelling. 

Modern photography software has lost this truth. Today's commercial tools subject photographers to hundreds of micro-sliders, aggressive artificial HDR tone-mapping, plastic skin-softening algorithms, and invasive synthetic generative "AI" replacements. The result is visual fatigue: images that look overcooked, synthetic, and homogenous.

First Proof exists to restore **photographic honesty and creative restraint**. We do the heavy mathematical lifting in our color engine so you don't have to spend 20 minutes micro-adjusting sliders to make an image look human again.

---

## 2. Core Pillars & Design Invariants

### I. Truth to Capture (Color Science First)
* **Wide-Gamut Linear Rec.2020 Working Space:** All tone adjustments, exposure compensation, white balance, and contrast curves operate strictly in un-clamped linear Rec.2020.
* **AgX Filmic Highlight Roll-off:** Highlights desaturate naturally as they approach clipping—emulating physical photochemical film—rather than shifting into harsh digital neon or plastic yellow artifacts.
* **Camera DCP Sensor Calibration:** Bayer sensor pixels are mapped directly from camera colorimeter profiles to standard observer spaces before tone shaping begins.
* **No Synthetic Slop:** First Proof does not generate synthetic pixels, hallucinate details, replace skies, or inject AI artifacts into your work.

### II. Restraint over Bloat (The Anti-150-Slider Stance)
* Every slider in First Proof corresponds to a real optical or chemical property.
* If an image requires 40 micro-adjustments just to look acceptable, the software’s baseline color science has failed. In First Proof, an image should look 90% finished the moment you open it.
* A streamlined control deck: Exposure, Balance, Filmic Tone Curves, Restrained Saturation, Detail, Optics, and Crop.

### III. Speed as Craft (Engineered for macOS & Apple Silicon)
* **Native Rust Core & Metal Compute:** LibRaw sensor unpacking, parallelized rayon CPU export pipelines, and sub-millisecond Metal GPU compute shaders.
* **Instant In-Memory Viewport Snapshots:** Thumbnail updates and slider scrubbing run at 60–120 fps on the GPU without thrashing disk I/O.
* **Sub-Second Launch:** No splash screens, no cloud authentication checks, no loading spinners.

### IV. Respect for the Artist's Machine, Privacy & Data Sovereignty
* **Portable Directory Catalogs (`.firstproof/catalog.db`):** Your library data travels with your photos on external SSDs and hard drives. No centralized catalog lock-in that corrupts your entire library.
* **100% Offline & Private:** Your photos never touch a remote server or training pipeline. First Proof requires no login, no cloud subscription, and zero telemetry.
* **Google Liberation Bridge (Ingest & Storage Culling):** Rather than locking you into cloud ecosystems, First Proof provides an intentional off-ramp: ingesting Google Pixel computational DNGs and Takeout archives to local storage, and providing fast triage (`P`/`X`) to reclaim cloud storage and own your masters.
* **Print & Web First:** While the pipeline leverages wide-gamut Rec.2020 and Apple Silicon displays, the ultimate reference target is authentic SDR reproduction that translates honestly to the web and fine-art physical paper prints.
* **Standard Open Metadata:** Full IPTC Core, Dublin Core, EXIF, and standard XMP sidecar synchronization (`https://firstproof.photo/ns/pm/1.0/`).

---

## 3. Brand Voice & Audience

### Target Audience
* **Editorial & Documentary Photographers:** Storytellers who value authenticity, speed, and real skin tones over hyper-stylized manipulation.
* **Analog / Hybrid Shooters:** Photographers who appreciate filmic contrast, realistic grain, and highlight rolloff without cheesy "vintage filter" presets.
* **Burnt-Out Lightroom Users:** Creators exhausted by Adobe's recurring subscription fees, sluggish performance, and relentless injection of AI marketing.

### Tone Guidelines
* **Direct, confident, unpretentious:** Speak like a master printer or darkroom technician, not a Silicon Valley growth hacker.
* **Anti-hype:** Avoid buzzwords like "supercharge", "revolutionary", "magic", or "AI-powered". Use terms of photographic craft: *gamut, roll-off, latitude, dynamic range, proof, contact sheet, density.*
