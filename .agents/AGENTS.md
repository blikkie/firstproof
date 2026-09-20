# Project Rules & Workflows

- **Environment & Toolchain Errors**: Whenever encountering an error that stems from environment or development toolchain setup (e.g., SDK version mismatches, toolchain configuration issues, missing system packages/Xcode), STOP and check in with the user immediately rather than spending cycles trying complex workarounds.

- **Product Philosophy & Anti-Slop Invariants**:
  - **First Proof** (`firstproof.photo`) is dedicated to photographic restraint, authentic color, and pure capture: *"Pure capture. Honest color. Zero digital plastic."*
  - The color engine must do the heavy lifting so photos look 90% finished immediately without requiring 40 micro-adjustments or plastic HDR/AI filters.
  - Never add synthetic AI pixel generation, sky replacements, or hallucinatory filters. Every adjustment must correspond to real optical, sensor, or chemical darkroom principles.
  - **Constructive Pushback & Values Alignment Guardrail**: When a suggested feature, adjustment, or roadmap change risks violating the product philosophy (e.g. introduces plastic aesthetics, hallucinatory AI, slider bloat, or deviates from optical/chemical darkroom fundamentals), do not blindly comply. Pause and constructively push back: articulate the tension with First Proof's core values, uncover the underlying photographic intent, and propose an alternative that stays true to honest capture, restraint, and optical integrity.

- **Toolchain Sandbox Permissions**: Commands involving the Darwin linker or Swift compiler (`swift build`, `make bundle`) require `BypassSandbox: true`. Pure Rust tests (`cargo test --manifest-path firstproof-core/Cargo.toml`) and git inspection can run in standard sandboxed mode.

- **Adversarial Code Reviews**: Before every commit and check-in, perform an adversarial review (probing diffs for race conditions, memory leaks/retain cycles, error boundary misses, out-of-order async responses, and main-thread bottlenecks).

- **Verification Gatekeeping Before Commit Tours**:
  - Whenever implementation work is completed, ALWAYS first prompt the user with the concrete manual verification steps committed to in the implementation plan (e.g. running the app, testing UI controls, validating shortcuts or image renders).
  - Explicitly pause and remind the user to perform these manual tests so we verify that the tool works as intended in the real environment.
  - Only after manual verification is confirmed should the adversarial review and educational commit tour be generated.

- **Educational Commit Tours (No Quizzes)**: Once manual verification is confirmed and before every commit/PR check-in, provide a tour of the code changes:
  - Focus strictly on what was built and explain the concrete rationale behind each change made.
  - Whenever a new concept, system primitive, or pattern is introduced into the codebase, provide an educational deep-dive explaining how that concept works under the hood.
  - Do not include quizzes or external language comparisons unless directly relevant to understanding the architecture.

- **Desktop Keyboard Architecture & First-Responder Shielding**:
  - Single-key shortcuts (`1`..`5` for ratings, `P`/`X`/`U` for flags, `C` for crop, `Z` for zoom, `[`/`]` for rotation, `\` for original peek) must **never** fire while the user is actively typing in editable text fields (e.g. Title, Caption, Keywords, Creator in the metadata panel).
  - Any local keyboard event monitor in `MainView.swift` or AppKit hosting must always verify that the window's first responder is not an active text field before intercepting keys:
    ```swift
    if let responder = window.firstResponder as? NSTextView,
       responder.isFieldEditor || (responder.isSelectable && responder.isEditable) {
        return event
    }
    ```

- **Color Science Pipeline & Working Space Invariants**:
  - Internal image adjustments (exposure, contrast, tone curves, saturation, film grain) must strictly execute in **Linear Rec.2020 wide-gamut** color space, NOT gamma-encoded sRGB or display P3.
  - Raw Bayer CFA sensor pixels are mapped into linear Rec.2020 via camera matrix / Adobe DCP profiles before any tone adjustments occur.
  - Tone-mapping to display luminance occurs at the end of the pipeline via AgX (Vivid/Neutral) or Camera DCP profiles.
  - Final offline exports must maintain bit-level parity with GPU viewport rendering and embed color space ICC profiles using `img-parts` and `lcms2`.
  - **Print & Web First (SDR Viewport Default)**: Even when running on an HDR/EDR-capable screen (e.g. MacBook Liquid Retina XDR, Pro Display XDR), the viewport canvas must always initialize and default to standard dynamic range (`isHdrEnabled = false`). EDR is an explicit user opt-in inspection mode, ensuring the photographer's default tonal decisions remain calibrated for print and universal web viewing rather than eye-searing local hardware radiance.

- **Dual-Engine Math Parity**: The Metal GPU shader pipeline (`Shaders.metal`, `Geometry.metal`) and the Rust CPU pipeline (`export.rs`, `curve.rs`, `color/`) must maintain bit-level mathematical parity. Any new tone transform or slider adjustment must be implemented identically in both engines and verified with `cargo test --test agx_shader_parity` and `cargo test --test camera_profiles_parity`.

- **Shared Rust Primitives & Web Parity Invariant**:
  - Never duplicate RAW decoding, Bayer binning, tone curve evaluation, or color math logic ad-hoc inside `firstproof-wasm`.
  - WASM decoders and calculations must strictly consume shared domain models (`RawSensorFrame`, `EditSettings`, `evaluate_curve_lut`) defined in `firstproof-types` to guarantee 1:1 color science, sensitometric math, and channel mapping parity across native macOS and web darkroom targets.

- **Physical Sensitometric Math Over Magic Numbers & Piecewise Hacks**:
  - Emulations (film stocks, recording emulsions, print media, enlarger filtration, toner chemistry) must always be grounded in continuous physical, optical, and sensitometric math—never arbitrary "magic numbers", hard thresholds, or piecewise if/else hacks.
  - Tonal curves must be smooth $C^1$ or $C^\infty$ analytical functions (e.g. bounded cubic polynomial S-curves $f(x) = x + \alpha \cdot x(1-x)(2x-1)$, smooth Hermite shoulders, or sigmoidal density curves). Fractional power laws $x^\gamma$ with infinite derivative ($f' \to \infty$) at bounds are forbidden.
  - Spectral absorption must obey physical laws (e.g. subtractive Beer-Lambert law $T = e^{-\alpha \cdot D}$ for dye clouds and pigments, multiplicative physical reflectance $I \cdot W_{\text{base}}$ for paper base warmth, quadratic surface scatter $f(1-x)^2$ for uncalendered rag matte).
  - Piecewise logic with discontinuous first derivatives ($C^0$ or discontinuous slopes) creates posterization, harsh solarization knees, and unnatural clipping. Always formulate adjustments as continuous mathematical manifolds.

- **Real-Time Viewport Thumbnail Invalidation**:
  - Viewport adjustments (sliders, curves, rotation) must never trigger slow disk re-renders or LibRaw demosaicing.
  - Thumbnails are updated via sub-millisecond GPU viewport snapshots (`captureThumbnailSnapshot`) into in-memory `ThumbnailCache`, followed by `NotificationCenter.default.post(name: .thumbnailDidChange, object: path)`. Disk serialization is deferred until save or closing the photo.

- **Zero Camera-JPEG Viewport Flashing & High-Res Intermediates**:
  - The viewport canvas must **never** display or flash the camera manufacturer's baked JPEG (`extractPreview`) for RAW files. Embedded camera JPEGs carry proprietary non-linear tone curves, artificial saturation boosts, and consumer sharpening that destroy the photographer's visual adaptation during burst culling.
  - RAW photo selection must stream directly into First Proof's 10–15ms 2×2 Bayer box binning engine (`decodeRawPreviewBinned`).
  - Intermediate preview lookups must strictly query First Proof's high-res rendered intermediates (`intermediateImage`), preserving authentic Linear Rec.2020 / AgX color from frame 0.
  - **Intermediate Bypass & Idempotence Invariant**: High-res intermediate snapshots (`renderThumbnailSnapshot`) have geometry (crop, rotation) and tone-mapping already baked into pixels. When rendered in `MetalCanvasView`, they must pass `isIntermediate = 1` in `ShaderUniforms` and identity geometry (`rotation: 0`, `crop: [0,0,1,1]`) to bypass all fragment color transforms (`sampledColor` returned directly) and vertex rotation. Never pass intermediate textures through AgX or geometry transforms a second time.
  - **Up-Front Edit Staging & Predictable Dark Transitions**: When switching photos in `loadPreview(for:)`, baseline EXIF orientation and XMP edit records must always be resolved and applied *before* updating preview state so stale edits from previous photos never flash. If a photo has no cached intermediate, immediately clear to the neutral dark canvas with the developing spinner for the 10–15ms until atomic promotion, avoiding visual confusion from retaining previous photo pixels.

- **Desktop Gesture Architecture & Hit Testing**:
  - Never attach `DragGesture` to dynamically repositioned or animated child views (e.g. views using `.position(point)` where `point` mutates during `.onChanged`). In macOS AppKit/SwiftUI hosting, moving the view anchor under the pointer causes the window server to drop pointer tracking or fire cancellation events, silently dropping `.onEnded` and breaking undo history.
  - Always use a **stationary full-canvas gesture container** (`Color.clear.contentShape(Rectangle()).gesture(DragGesture(...))`) that resolves hit targets (corners, edges, pan, rotate) at `startLocation`. Visual handles must be purely declarative render elements with `.allowsHitTesting(false)`.
  - Always guarantee cleanup on exit: whenever closing an interactive tool (via "Done" button, Return, Escape, or `.onDisappear`), commit any pending drag scrub (`if undoHistory.scrubStartEdits != nil { endSliderScrub(...) }`).

- **Inspector Layout & Aspect-Ratio Containment**:
  - In inspector sidebars (`ScrollView`/`VStack`), `GeometryReader` is greedy and unconstrained along the vertical axis. Never place a square interactive graph (e.g. Tone Curve, Color Wheel) in an arbitrary height frame without strict aspect-ratio containment.
  - Always use the native `SquareAspectLayout` container pattern (`SquareAspectLayout: Layout`):
    ```swift
    SquareAspectLayout {
        GeometryReader { geo in ... }
    }
    .clipped()
    ```
  - This natively participates in SwiftUI's layout negotiation (`sizeThatFits` computes exact width matching height), reserving exact vertical space in scroll views and preventing layout overflow or jitter into adjacent disclosure groups.

- **Portable Catalogs on External Drives (`.firstproof/`)**:
  - Folder indexing on external storage volumes stores catalogs inside `<volume>/.firstproof/catalog.db` so the catalog travels with the external media.

- **Reference-First Architecture & Zero Greenfield Hallucination**:
  - Whenever building, porting, or refactoring subsystems in `web/`, `firstproof-wasm/`, or secondary targets where a native implementation already exists in `FirstProofApp/` (macOS Swift/Metal) or `firstproof-core/` (Rust):
  - **Never invent ad-hoc layout, math, or interaction models from general web/platform training priors.** (e.g. do not default to CSS `object-fit: contain` when native uses full-bleed vertex quad scaling; do not place floating tool dialogs in the photo center when native uses stationary bottom HUDs; do not hardcode landscape aspect ratios when native inverts them for portrait images).
  - **Shared Library Extraction Preference**: When analyzing whether an algorithm, state transition, coordinate mapping, or mathematical model is being reimplemented or ported, the default preference MUST be to **extract it into the shared Rust library (`firstproof-types`)** rather than duplicating it across Swift and TypeScript. Only view hierarchy layout and platform event monitors belong in host code.
  - Always execute the **Reference Deconstruction Protocol** (`firstproof-subsystem-parity` skill) BEFORE writing target code:
    1. **Identify Authoritative Reference Files** in `FirstProofApp/` or `firstproof-core/`.
    2. **Extract the 4 Core Primitives**: Coordinate Spaces, Viewport/Sizing Negotiation, State Lifecycle & HUD Placement, and Orientation/Edge Cases.
    3. **Generate a Porting Contract Matrix** mapping Swift/Metal constructs 1:1 to target equivalents (or extracting them into `firstproof-types`).
    4. **Mechanically Transcribe the Architecture** to preserve identical visual styling, mathematical guarantees, and interaction ergonomics.

- **Proactive Customization Encoding**: Whenever a platform-specific quirk, non-obvious workaround, multi-step pipeline recipe, or architectural convention is established during development, proactively suggest encoding it into `.agents/AGENTS.md` or as a new skill under `.agents/skills/` so subsequent chats retain it automatically.

- **Token Efficiency Invariants**:
  - Never view `firstproof_core.swift` directly (4,400+ generated lines); inspect `firstproof-core/src/lib.rs` for exported FFI signatures or specific Rust sub-modules.
  - Slices on large files: always provide `StartLine` and `EndLine` (target ~80–120 lines per slice) when viewing files over 300 lines instead of dumping full files into context.
  - Targeted searches: always scope `grep_search` with `Includes` (e.g. `*.swift`, `*.rs`) and specific subdirectories (`SearchPath`) rather than searching the workspace root.

- **Story Completion & Backlog Hygiene**:
  - **Top-of-File Focus**: `docs/roadmap/backlog.md` must always maintain an **Immediate Next Up** section at the very top (first 20 lines). When asked "what's next on the backlog", agents must inspect ONLY lines 1–25 (`StartLine: 1, EndLine: 25`) rather than loading the full document. Only view or load the full backlog when adding, completing, or reorganizing stories.
  - **Archive on Completion**: Whenever a story or roadmap task is completed, move its description and verification record from the active backlog (`docs/roadmap/backlog.md`) into the archive file (`docs/roadmap/completed.md`). The active backlog must strictly retain pending, in-progress, or deferred work to preserve minimal token context.
