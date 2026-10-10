# Project Rules & Workflows

- **Environment & Toolchain Errors**: Whenever encountering an error that stems from environment or development toolchain setup (e.g., SDK version mismatches, toolchain configuration issues, missing system packages/Xcode), STOP and check in with the user immediately rather than spending cycles trying complex workarounds.

- **Product Philosophy & Anti-Slop Invariants**:
  - **First Proof** (`firstproof.photo`) is dedicated to photographic restraint, authentic color, and pure capture: *"Pure capture. Honest color. Zero digital plastic."*
  - The color engine must do the heavy lifting so photos look 90% finished immediately without requiring 40 micro-adjustments or plastic HDR/AI filters.
  - Never add synthetic AI pixel generation, sky replacements, or hallucinatory filters. Every adjustment must correspond to real optical, sensor, or chemical darkroom principles.
  - **Constructive Pushback & Values Alignment Guardrail**: When a suggested feature, adjustment, or roadmap change risks violating the product philosophy (e.g. introduces plastic aesthetics, hallucinatory AI, slider bloat, or deviates from optical/chemical darkroom fundamentals), do not blindly comply. Pause and constructively push back: articulate the tension with First Proof's core values, uncover the underlying photographic intent, and propose an alternative that stays true to honest capture, restraint, and optical integrity.

- **Toolchain Sandbox Permissions**: Commands involving the Darwin linker or Swift compiler (`swift build`, `make bundle`) require `BypassSandbox: true`. Pure Rust tests (`cargo test --manifest-path firstproof-core/Cargo.toml`) and git inspection can run in standard sandboxed mode.

- **Harness Engineering & The Computational Steering Loop**:
  - **Agent = Model + Harness**: Never rely solely on inferential prompt text or model self-critique to guarantee quality. System integrity must be enforced by automated, deterministic computational feedback sensors (tests, typecheckers, linters, parity assertions).
  - **Keep Quality Left**: Catch regressions in the agent's fast inner loop before handing off to the user.
  - **The Steering Loop**: Whenever an error, edge-case failure, or architectural regression is encountered, do not just patch the code or add verbal instructions to a prompt. Feed it back into the harness: add a permanent computational sensor (a unit test, parity assertion, or fitness function) so that class of failure is structurally impossible in the future.
  - **Lifecycle Hooks vs. Prompt Rules Decision Framework**:
    - **When to Use a Lifecycle Hook (`.agents/hooks.json`)**: If a policy can be stated as *"Never execute tool X with argument Y"* (e.g. reading 4,400-line generated files like `firstproof_core.swift`) or *"Always run check Z before action W"*, it MUST be codified as a deterministic lifecycle hook rather than an inferential prompt guideline. Hooks execute out-of-band, cost zero context tokens during normal execution, and physically prevent regressions by denying unauthorized actions or rewriting unsafe invocations.
    - **Anti-Hedge & First-Party Internal Invariant**:
      - Never introduce speculative defensive shims, `typeof x === 'function'` existence checks, optional property wrappers (`fn?: ...`), or runtime fallback branches for first-party internal crates (`firstproof-wasm`, `firstproof-core`, UniFFI bridge). First-party modules are compiled from the same monorepo.
      - Statically import and invoke internal functions directly.
      - Never degrade functionality through semantic fallbacks (e.g. falling back to 2×2 binned preview when full linear is requested) or dead manual pointer math.
      - Never swallow Rust UniFFI errors in Swift with `try? FirstProof...`—always propagate or log specific Rust errors via do-catch.
      - Enforced deterministically by `./scripts/anti_slop_sensor.py`, step 1 of `./scripts/check.sh`, and `.pre-commit-config.yaml` (`pre-commit`).
  - **Inner-Loop vs. Outer-Loop Verification Policy**:
    - **Fast Inner Loop (Targeted Sensors Only)**: During rapid iterative coding, NEVER run the slow full `./scripts/check.sh` or `make check` suite (which runs multi-minute DNG conversions, full opcode matrices, WASM packing, and production bundling). Run strictly targeted, sub-second sensors matching the touched files:
      - Rust core/engine math & parity: `cargo test --manifest-path firstproof-core/Cargo.toml --test <test_name>`
      - Web / TypeScript / shaders: `npx --prefix web vitest run <path>`
      - Swift / AppKit: incremental `swift build` or `swift test`
    - **Outer Loop (Pre-Commit Full Gate Only)**: Run the exhaustive unified sensor suite (`./scripts/check.sh` or `make check`) strictly **once, at the very end of the task in the outer loop**, immediately before presenting the educational commit tour and requesting user sign-off.
  - **Print Media Boundary Anchoring & Substrate Decoupling Invariant**:
    - Print media archetypes model authentic chemical darkroom reactions (coupled dyes, azo dye destruction, silver halides, noble metals, Prussian blue iron salts, archival selenium)—never digital "matte" filters or inkjet paper substrates.
    - **Strict Boundary Anchoring Invariant ($f(0.0) \equiv 0.0, f(1.0) \equiv 1.0$)**: Never introduce additive digital floor lifts (`+ 0.016` or `+ 0.035`) into develop adjustments. Additive floor lifts starve physical printers of carbon black ink ($D_{\max}$), double-flatten physical prints, and cast artificial milky veils on calibrated HDR/EDR screens. Noble metals (Platinum/Palladium) achieve open luminous shadows through analytical cubic toe cushioning and colloidal harmonic toning, never floor lifting.
    - **Substrate Decoupling**: Paper substrates (unbleached cotton rag, alpha cellulose, baryta base reflection, OBA white points, ink saturation limits) belong exclusively in **Soft Proofing via ICC profiles (Story 3.4)**, never develop adjustments.

- **Adversarial Code Reviews & Anti-Magic Number Audit**: Before every commit and check-in, perform an adversarial review:
  - Probe diffs for race conditions, memory leaks/retain cycles, WebGL/Metal texture clobbering, error boundary misses, out-of-order async responses, and main-thread bottlenecks.
  - **Audit for Arbitrary Constants & Heuristics**: Rigorously inspect every camera processing, color science, and exposure adjustment in the diff for arbitrary magic numbers, localized scalar multipliers, or piecewise if/else heuristics. Any new numerical parameter must trace directly to published sensitometric standards (e.g. DNG specification, Troy Sobotka's AgX specification, CIE photopic luminance) rather than trial-and-error curve fitting.

- **Verification Gatekeeping & Harness Delegation Framework**:
  - **Always Tour Before Committing**: Never execute `git commit` before presenting the educational tour and review of changes to the user. The tour must always be delivered first so the user knows exactly what was built, why each change was made, and what to verify before committing.
  - **Mandatory Manual Testing Instructions for Perceptual & Visual Changes**:
    - Automated sensors in `./scripts/check.sh` (`make check`, Rust unit tests, architectural fitness functions, Vitest behavioral tests, parity assertions) catch computational and mathematical regressions in the fast inner loop.
    - However, whenever changes introduce or modify **color rendering formulations, visual shader aesthetics, tone curves, emulsion/print media curves, film grain look, or interactive pointer gestures/windowing**, ALWAYS provide explicit, actionable manual testing instructions in the tour:
      1. Which test fixtures, camera profiles, or real RAW captures to load.
      2. Which specific sliders, workspace tabs, or emulsion/print archetypes to toggle or scrub.
      3. What concrete perceptual rendering behaviors, tonal roll-offs, or optical artifacts to inspect in the viewport.
    - Never skip these verification instructions even when all automated parity tests pass, because human visual inspection remains essential for perceptual color science.

- **Educational Commit Tours (No Quizzes)**: Always present the tour of changes BEFORE requesting or creating any commit:
  - Focus strictly on what was built and explain the concrete rationale behind each change made.
  - Whenever a new concept, system primitive, or pattern is introduced into the codebase, provide an educational deep-dive explaining how that concept works under the hood.
  - Pair the tour with the concrete manual verification instructions whenever rendering or interactivity was touched.
  - Do not include quizzes or external language comparisons unless directly relevant to understanding the architecture.
  - Await user review and confirmation before executing the commit.

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
  - **Terminal RGB Conversion Invariant**: Conversion to display/output RGB (sRGB, Display P3, or gamma encoding) must strictly be the **VERY LAST step of the entire pipeline**, both in GPU viewport shaders (`Shaders.metal`, `FragmentShader.ts`) and CPU export pipelines (`export/pipeline.rs`, `encoders.rs`).
  - Internal image adjustments (exposure, contrast, tone curves, vibrance, saturation, split toning, detail sharpening, film grain, print media) must strictly execute in **Linear Rec.2020 wide-gamut** color space ($Y = 0.2627 R + 0.6780 G + 0.0593 B$), NOT gamma-encoded sRGB or display P3.
  - Never apply color adjustments, saturation, tone curves, or sharpening to narrow sRGB or gamma-encoded values. Doing so truncates wide-gamut hues, distorts $R/G$ chromaticity ratios, and produces muddy skin tones or posterized clipping knees.
  - Raw Bayer CFA sensor pixels are mapped into linear Rec.2020 via camera matrix / Adobe DCP profiles before any tone adjustments occur.
  - Tone-mapping to display luminance occurs at the very end of the pipeline via AgX (Vivid/Neutral) or Camera DCP profiles.
  - **Physical Sensitometric Grain & Emulsion Placement**: Film grain simulates the physical response of silver halide crystals or dye clouds reacting to incident scene photons in the optical/chemical domain before optical tonality and print transfer. Applying grain post-tone-mapping in narrow gamma sRGB non-linearly distorts shadow noise floors. Grain must be evaluated in the wide-gamut optical domain.
  - Final offline exports must maintain bit-level parity with GPU viewport rendering and embed color space ICC profiles using `img-parts` and `lcms2`.
  - **Print & Web First (SDR Viewport Default)**: Even when running on an HDR/EDR-capable screen (e.g. MacBook Liquid Retina XDR, Pro Display XDR), the viewport canvas must always initialize and default to standard dynamic range (`isHdrEnabled = false`). EDR is an explicit user opt-in inspection mode, ensuring the photographer's default tonal decisions remain calibrated for print and universal web viewing rather than eye-searing local hardware radiance.

- **Multi-Engine Math Parity**: The Metal GPU shader pipeline (`Shaders.metal`, `Geometry.metal`), WebGL 2 pipeline (`FragmentShader.ts`), and the Rust CPU pipeline (`export.rs`, `curve.rs`, `color/`) must maintain bit-level mathematical parity across all targets. Any new tone transform or slider adjustment must be implemented identically across engines and verified with `cargo test --test agx_shader_parity` and `cargo test --test camera_profiles_parity`.

- **Shared Rust Primitives & Web Parity Invariant**:
  - Never duplicate RAW decoding, Bayer binning, tone curve evaluation, or color math logic ad-hoc inside `firstproof-wasm`.
  - WASM decoders and calculations must strictly consume shared domain models (`RawSensorFrame`, `EditSettings`, `evaluate_curve_lut`) defined in `firstproof-engine` to guarantee 1:1 color science, sensitometric math, and channel mapping parity across native macOS and web darkroom targets.

- **Post-Public Enum Immutability & Slot Preservation Invariant**:
  - Once First Proof is publicly released (v0.1+), **never delete or renumber existing enum discriminants or ID slots** in serialized domain models (`PrintMediaMode`, `EmulsionArchetype`, `EnlargerFilterMode`, etc.).
  - Renumbering discriminants breaks backward compatibility with user catalogs (`catalog.db`), existing `.xmp` sidecars, and cross-platform FFI bridges.
  - When an archetype, filter, or adjustment is retired post-release, its numeric discriminant must be permanently **marked as `Reserved` or aliased to fallback/unset (`None`)**, gracefully falling back to neutral defaults during deserialization without shifting adjacent IDs.

- **Physical Sensitometric Math Over Magic Numbers & Piecewise Hacks**:
  - Emulations (film stocks, recording emulsions, print media, enlarger filtration, toner chemistry) must always be grounded in continuous physical, optical, and sensitometric math—never arbitrary "magic numbers", hard thresholds, or piecewise if/else hacks.
  - Tonal curves must be smooth $C^1$ or $C^\infty$ analytical functions (e.g. bounded cubic polynomial S-curves $f(x) = x + \alpha \cdot x(1-x)(2x-1)$, smooth Hermite shoulders, or sigmoidal density curves). Fractional power laws $x^\gamma$ with infinite derivative ($f' \to \infty$) at bounds are forbidden.
  - Spectral absorption must obey physical laws (e.g. subtractive Beer-Lambert law $T = e^{-\alpha \cdot D}$ for dye clouds and pigments, multiplicative physical reflectance $I \cdot W_{\text{base}}$ for paper base warmth, quadratic surface scatter $f(1-x)^2$ for uncalendered rag matte).
  - Piecewise logic with discontinuous first derivatives ($C^0$ or discontinuous slopes) creates posterization, harsh solarization knees, and unnatural clipping. Always formulate adjustments as continuous mathematical manifolds.

- **First-Principles Transfer Invariants Over Downstream Curve Fitting**:
  - When encountering color casts, clipping, muddy tones, or blowout, **never apply localized scalar multipliers, arbitrary hue offsets, or downstream saturation tweaks** to mask symptoms.
  - **Boundary Anchoring Invariant**: Transfer curves must satisfy $f(0.0) \equiv 0.0$ and $f(1.0) \equiv 1.0$ exactly. A transfer function must never plunge negative or cross zero prematurely inside the dynamic range.
  - **Strict Monotonicity Invariant**: Transfer curves must have $f'(x) > 0$ for all $x \in (0, 1)$. Flat spots, premature clipping, or downward slopes are strictly forbidden.
  - **Continuous First Derivative ($C^1$)**: Transitions between shoulders, toes, and midtones must possess matching first derivatives without slope discontinuities or hard knees.
  - If a tone curve flatlines below $-3.7\text{ EV}$, no downstream color tuning can ever recover it. Always fix the transfer function at the root.

- **Global Field & Wedge Verification Invariant (Never Spot-Check in Isolation)**:
  - Never declare an image quality, color science, or develop adjustment "resolved" based on a single pixel coordinate, isolated crop, or hand-picked spot audit (Spot-Check Confirmation Bias).
  - Every color science adjustment must pass three levels of global verification:
    1. **Synthetic Greyscale Wedge (-10 EV to +6.5 EV)**: Proves that zero stops are clipped or flattened across the entire 16.5-stop dynamic range.
    2. **Full-Frame Clipping Telemetry**: Proves that real RAW shadow floors and highlight ceilings do not exhibit artificial spikes at 0.0 or 1.0.
    3. **Decomposed CIELAB Error ($\Delta L^*, \Delta C^*, \Delta h$)**: Distinguishes whether an error is caused by luminance transfer ($\Delta L^*$), chroma scale ($\Delta C^*$), or hue rotation ($\Delta h$), preventing luminance errors from being masked with chroma hacks.

- **Strict Separation of Color Models (No Model Conflation)**:
  - Never benchmark an open-domain filmic tone mapper against a closed-domain manufacturer camera curve:
  - **Camera Matching (`CameraStandard`)**: Emulates Canon/Sony/Nikon DIGIC processing via Adobe DCP ForwardMatrix and proprietary camera S-curves. It is the **only** profile that should be benchmarked against camera reference JPEGs.
  - **AgX Filmic (`AgxFilmic`, `AgxVivid`)**: Troy Sobotka's open-domain cinematic tone mapper designed for 16.5 stops of scene radiance and natural highlight desaturation. It must **never** be hacked with arbitrary "memory color" offsets (`h -= 18`, `h -= 3`, `sat_boost = 0.18`) to mimic a camera JPEG.
  - Conflating the two produces slop: artificial heuristics that destroy wide-gamut filmic highlights and crush shadows.

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
  - **Shared Library Extraction Preference**: When analyzing whether an algorithm, state transition, coordinate mapping, or mathematical model is being reimplemented or ported, the default preference MUST be to **extract it into the shared Rust library (`firstproof-engine`)** rather than duplicating it across Swift and TypeScript. Only view hierarchy layout and platform event monitors belong in host code.
  - Always execute the **Reference Deconstruction Protocol** (`firstproof-subsystem-parity` skill) BEFORE writing target code:
    1. **Identify Authoritative Reference Files** in `FirstProofApp/` or `firstproof-core/`.
    2. **Extract the 4 Core Primitives**: Coordinate Spaces, Viewport/Sizing Negotiation, State Lifecycle & HUD Placement, and Orientation/Edge Cases.
    3. **Generate a Porting Contract Matrix** mapping Swift/Metal constructs 1:1 to target equivalents (or extracting them into `firstproof-engine`).
    4. **Mechanically Transcribe the Architecture** to preserve identical visual styling, mathematical guarantees, and interaction ergonomics.

- **Proactive Customization Encoding**: Whenever a platform-specific quirk, non-obvious workaround, multi-step pipeline recipe, or architectural convention is established during development, proactively suggest encoding it into `.agents/AGENTS.md` or as a new skill under `.agents/skills/` so subsequent chats retain it automatically.

- **Token Efficiency Invariants**:
  - Never view `firstproof_core.swift` directly (4,400+ generated lines; enforced structurally by `.agents/hooks.json` token-defense hook); inspect `firstproof-core/src/lib.rs` for exported FFI signatures or specific Rust sub-modules.
  - Slices on large files: always provide `StartLine` and `EndLine` (target ~80–120 lines per slice) when viewing files over 300 lines instead of dumping full files into context.
  - Targeted searches: always scope `grep_search` with `Includes` (e.g. `*.swift`, `*.rs`) and specific subdirectories (`SearchPath`) rather than searching the workspace root.

- **Story Completion & Backlog Hygiene**:
  - **Top-of-File Focus**: `docs/roadmap/backlog.md` must always maintain an **Immediate Next Up** section at the very top (first 20 lines). When asked "what's next on the backlog", agents must inspect ONLY lines 1–25 (`StartLine: 1, EndLine: 25`) rather than loading the full document. Only view or load the full backlog when adding, completing, or reorganizing stories.
  - **Archive on Completion**: Whenever a story or roadmap task is completed, move its description and verification record from the active backlog (`docs/roadmap/backlog.md`) into the archive file (`docs/roadmap/completed.md`). The active backlog must strictly retain pending, in-progress, or deferred work to preserve minimal token context.
