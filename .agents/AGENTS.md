# Project Rules & Invariants

## 1. Product Philosophy & Photographic Integrity
- **First Proof Core Creed**: *"Pure capture. Honest color. Zero digital plastic."* The color engine does the heavy lifting so photos look 90% finished immediately without plastic HDR or synthetic filters.
- **Strictly Optical/Chemical Darkroom Fundamentals**: Never introduce synthetic AI pixel generation, sky replacements, or hallucinatory filters. Every adjustment must correspond to real optical, sensor, or chemical darkroom principles.
- **Tone & Copy Restraint**: Quiet, factual, warm, understated. Never use breathless buzzwords (*"revolutionary"*, *"blazing-fast"*, *"game-changing"*), conversion hype, promotional urgency, or patronizing emojis.
- **Dual-Domain Performance Asymmetry ("Wall for 10 Years")**:
  - *Viewport & Culling*: Friction is fatal. Cut computational corners aggressively to lock 60 FPS and 10–15ms photo transitions (2×2 Bayer box binning, GPU snapshots, texture cache reuse).
  - *Archival Export & High-Fidelity Preview*: *"You only export it once, and then it hangs on your wall for 10 years."* Deep computational heaviness is worthwhile. Never compromise optical, demosaicing, or colorimetric quality for vanity export benchmarks.
- **Constructive Pushback**: When a requested change risks violating photographic integrity, pause and constructively push back with an optical alternative.

---

## 2. Color Science & Pipeline Invariants
- **Terminal RGB Conversion Invariant**: Conversion to display/output RGB (sRGB, Display P3, gamma encoding) is strictly the **very last step of the entire pipeline**, both in GPU shaders (`Shaders.metal`, `FragmentShader.ts`) and CPU export pipelines (`export/pipeline.rs`, `encoders.rs`).
- **Linear Rec.2020 Working Gamut**: All internal adjustments (exposure, contrast, tone curves, vibrance, saturation, split toning, detail sharpening, film grain, print media) execute in **Linear Rec.2020 wide-gamut** space ($Y = 0.2627 R + 0.6780 G + 0.0593 B$), never gamma-encoded sRGB.
- **Sensitometric Film Grain Placement**: Film grain simulates crystal/dye cloud response to scene photons. Evaluate grain exclusively in the wide-gamut optical domain before display tone mapping.
- **Strict Boundary Anchoring ($f(0.0) \equiv 0.0, f(1.0) \equiv 1.0$)**: Never introduce additive digital floor lifts (`+ 0.016` or `+ 0.035`) into develop adjustments. Substrate paper base reflection belongs strictly in Soft Proofing ICC profiles, never develop adjustments.
- **Continuous Math Over Magic Numbers**: Tonal curves must be smooth $C^1$ analytical functions with strict monotonicity ($f'(x) > 0$). Piecewise logic or hard threshold knees that cause solarization or posterization are strictly forbidden.
- **Separation of Color Models**: Never apply closed-domain camera JPEG heuristics to Troy Sobotka's open-domain AgX filmic tone mapper. For diagnostics, consult `.agents/skills/firstproof-color-science-diagnostics/`.
- **SDR Viewport Default**: Default canvas to standard dynamic range (`isHdrEnabled = false`) so tonal decisions remain calibrated for print and web. EDR is an explicit user opt-in.

---

## 3. Harness Engineering & Fast Inner Loop
- **Agent = Model + Harness**: Quality is enforced by deterministic computational sensors (tests, typecheckers, linters, parity assertions), not prompt self-critique.
- **Inner-Loop vs. Outer-Loop Verification**:
  - *Fast Inner Loop (Targeted Only)*: During iterative coding, run strictly sub-second targeted sensors:
    - Rust: `cargo test -p <crate> --test <test_name>`
    - Web: `npx --prefix web vitest run <path>`
    - Swift: incremental `swift build` or `swift test`
  - *Outer Loop (Pre-Commit Full Gate Only)*: Run `./scripts/check.sh` strictly **once**, immediately before presenting the educational commit tour.
- **Anti-Hedge & Zero-Shim Invariant**: Never add defensive shims (`typeof x === 'function'`, `fn?: ...`) or semantic fallbacks for first-party internal crates (`firstproof-wasm`, `firstproof-core`, UniFFI bridge). Statically import and invoke directly. Never swallow Rust UniFFI errors with `try? FirstProof...`—propagate or handle via `do-catch`.
- **Pre-Commit Code Review Subagent**: Before presenting an educational commit tour, execute the `firstproof-code-review` skill to evaluate the clean working tree against the 6 core review pillars.

---

## 4. Cross-Platform Parity & Architecture
- **Universal Parity Invariant**: Every feature, slider, toggle, metadata field, or workflow touched must be implemented across all supported targets:
  - macOS Native: SwiftUI / Metal (`FirstProofApp`)
  - Web Darkroom: WebGL / TypeScript (`web/`)
  - Rust Core / WASM: `firstproof-core`, `firstproof-engine`, `firstproof-wasm`
- **Shared Rust Primitives**: Common RAW decoding, Bayer binning, tone curve evaluation, and color math must live in `firstproof-engine` rather than being reimplemented in Swift or TypeScript.
- **Canonical WGSL Shaders**: Viewport shaders are authored once in WGSL (`firstproof-engine/shaders/`) and transpiled to MSL, GLSL, and WebGPU via Naga (`cargo run -p firstproof-core --bin firstproof-cli -- shaders`).
- **Post-Public Enum Immutability**: Never renumber or delete existing serialized enum discriminants or ID slots in domain models. Retired entries must be marked `Reserved` or alias to neutral defaults.
- **Native Platform Citizen**: Respect host OS idioms:
  - macOS: Application Support in `~/Library/Application Support/FirstProof/`, Caches in `~/Library/Caches/photo.firstproof.app/`. Never scatter Unix dotfiles in `$HOME`.
  - Linux/CLI: Adhere to XDG Base Directory specification.
  - Portable catalogs: Removable drives store catalogs at `<volume>/.firstproof/catalog.db`.

---

## 5. UI, Viewport & Gesture Architecture
- **Zero Camera-JPEG Viewport Flashing**: Viewport selection streams directly into 10–15ms 2×2 Bayer box binning (`decodeRawPreviewBinned`). Never display the embedded camera JPEG for RAW files.
- **Intermediate Bypass & Idempotence**: Viewport snapshots already have geometry and tone mapping baked in. When rendering intermediates, pass `isIntermediate = 1` and identity geometry to bypass fragment color transforms.
- **Keyboard First-Responder Shielding**: Single-key shortcuts (`1`..`5`, `P`/`X`/`U`, `C`, `Z`, `[`, `]`, `\`) must **never** fire while typing in editable text fields. Verify the window's first responder is not an active text field before intercepting keys.
- **Stationary Canvas Gesture Container**: Never attach `DragGesture` to moving child views. Use stationary full-canvas hit testing with declarative handles (`.allowsHitTesting(false)`). Always commit pending scrubs on exit.
- **Inspector Layout Containment**: Interactive square controls (Tone Curve, Color Wheel) in scroll views must use `SquareAspectLayout` to prevent vertical layout jitter.
- **Real-Time Thumbnail Invalidation**: Viewport edits update in-memory `ThumbnailCache` via GPU snapshots (`captureThumbnailSnapshot`) and post `.thumbnailDidChange`. Disk serialization is deferred until save or closing the photo.

---

## 6. Token Efficiency & Reactive Execution
- **Strict Zero-Polling Invariant**: When a long command is launched via `run_command` in the background, **never call `schedule` or `manage_task(status)` in a loop**. Simply inform the user and stop calling tools. The harness will automatically wake the agent when the task terminates.
- **Token Defense for Generated Files**: Never read `firstproof_core.swift` or `firstproof_engine.swift` (enforced by `.agents/hooks.json`). Inspect `firstproof-core/src/lib.rs` or `firstproof-engine/src/lib.rs` for signatures.
- **Slices for Large Files**: Always provide `StartLine` and `EndLine` (target 80–120 lines per slice) when reading files over 300 lines.
- **Targeted Searches**: Always scope `grep_search` with specific `Includes` and `SearchPath` rather than searching the workspace root.
- **Backlog Focus**: When inspecting `docs/roadmap/backlog.md`, read only the top 25 lines (`StartLine: 1, EndLine: 25`). Archive completed tasks to `docs/roadmap/completed.md`.
- **Educational Tours (No Quizzes)**: Present an educational tour of changes before requesting sign-off to commit. Focus on what was built, explain key concepts, and provide concrete manual testing instructions for perceptual changes.
