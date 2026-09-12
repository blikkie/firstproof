# Project Rules & Workflows

- **Environment & Toolchain Errors**: Whenever encountering an error that stems from environment or development toolchain setup (e.g., SDK version mismatches, toolchain configuration issues, missing system packages/Xcode), STOP and check in with the user immediately rather than spending cycles trying complex workarounds.

- **Product Philosophy & Anti-Slop Invariants**:
  - **First Proof** (`firstproof.photo`) is dedicated to photographic restraint, authentic color, and pure capture: *"Pure capture. Honest color. Zero digital plastic."*
  - The color engine must do the heavy lifting so photos look 90% finished immediately without requiring 40 micro-adjustments or plastic HDR/AI filters.
  - Never add synthetic AI pixel generation, sky replacements, or hallucinatory filters. Every adjustment must correspond to real optical, sensor, or chemical darkroom principles.

- **Toolchain Sandbox Permissions**: Commands involving the Darwin linker or Swift compiler (`swift build`, `make bundle`) require `BypassSandbox: true`. Pure Rust tests (`cargo test --manifest-path firstproof-core/Cargo.toml`) and git inspection can run in standard sandboxed mode.

- **Adversarial Code Reviews**: Before every commit and check-in, perform an adversarial review (probing diffs for race conditions, memory leaks/retain cycles, error boundary misses, out-of-order async responses, and main-thread bottlenecks).

- **Educational & Interactive Commit Tours**: Before every commit/PR check-in, provide a tour of the code changes and use it as a learning opportunity. Compare Rust and Swift concepts against Python and JavaScript (e.g., ownership vs garbage collection, static typing, FFI, memory layout, concurrency). Include clear code snippets, explain syntax/architecture decisions, and end with 1-2 interactive quiz questions to reinforce fluency in desktop app engineering.

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

- **Real-Time Viewport Thumbnail Invalidation**:
  - Viewport adjustments (sliders, curves, rotation) must never trigger slow disk re-renders or LibRaw demosaicing.
  - Thumbnails are updated via sub-millisecond GPU viewport snapshots (`captureThumbnailSnapshot`) into in-memory `ThumbnailCache`, followed by `NotificationCenter.default.post(name: .thumbnailDidChange, object: path)`. Disk serialization is deferred until save or closing the photo.

- **Desktop Gesture Architecture & Hit Testing**:
  - Never attach `DragGesture` to dynamically repositioned or animated child views (e.g. views using `.position(point)` where `point` mutates during `.onChanged`). In macOS AppKit/SwiftUI hosting, moving the view anchor under the pointer causes the window server to drop pointer tracking or fire cancellation events, silently dropping `.onEnded` and breaking undo history.
  - Always use a **stationary full-canvas gesture container** (`Color.clear.contentShape(Rectangle()).gesture(DragGesture(...))`) that resolves hit targets (corners, edges, pan, rotate) at `startLocation`. Visual handles must be purely declarative render elements with `.allowsHitTesting(false)`.
  - Always guarantee cleanup on exit: whenever closing an interactive tool (via "Done" button, Return, Escape, or `.onDisappear`), commit any pending drag scrub (`if undoHistory.scrubStartEdits != nil { endSliderScrub(...) }`).

- **Inspector Layout & Aspect-Ratio Containment**:
  - In inspector sidebars (`ScrollView`/`VStack`), `GeometryReader` is greedy and unconstrained. Never place a square interactive graph (e.g. Tone Curve) in an arbitrary height frame without strict aspect-ratio containment.
  - Always use the bounded container pattern:
    ```swift
    Color.clear
        .aspectRatio(1.0, contentMode: .fit)
        .overlay(GeometryReader { geo in ... })
        .clipped()
    ```
  - This guarantees the layout engine allocates exact vertical space matching horizontal width, preventing layout overflow into adjacent disclosure groups.

- **Portable Catalogs on External Drives (`.firstproof/`)**:
  - Folder indexing on external storage volumes stores catalogs inside `<volume>/.firstproof/catalog.db` so the catalog travels with the external media.

- **Proactive Customization Encoding**: Whenever a platform-specific quirk, non-obvious workaround, multi-step pipeline recipe, or architectural convention is established during development, proactively suggest encoding it into `.agents/AGENTS.md` or as a new skill under `.agents/skills/` so subsequent chats retain it automatically.
