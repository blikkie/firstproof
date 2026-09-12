# Project Rules & Workflows

- **Environment & Toolchain Errors**: Whenever encountering an error that stems from environment or development toolchain setup (e.g., SDK version mismatches, toolchain configuration issues, missing system packages/Xcode), STOP and check in with the user immediately rather than spending cycles trying complex workarounds.

- **Toolchain Sandbox Permissions**: Commands involving the Darwin linker or Swift compiler (`swift build`, `make bundle`) require `BypassSandbox: true`. Pure Rust tests (`cargo test --manifest-path lumenbox-core/Cargo.toml`) and git inspection can run in standard sandboxed mode.

- **Adversarial Code Reviews**: Before every commit and check-in, perform an adversarial review (probing diffs for race conditions, memory leaks/retain cycles, error boundary misses, out-of-order async responses, and main-thread bottlenecks).

- **Educational & Interactive Commit Tours**: Before every commit/PR check-in, provide a tour of the code changes and use it as a learning opportunity. Compare Rust and Swift concepts against Python and JavaScript (e.g., ownership vs garbage collection, static typing, FFI, memory layout, concurrency). Include clear code snippets, explain syntax/architecture decisions, and end with 1-2 interactive quiz questions to reinforce fluency in desktop app engineering.

- **Dual-Engine Math Parity**: The Metal GPU shader pipeline (`Shaders.metal`, `Geometry.metal`) and the Rust CPU pipeline (`export.rs`, `curve.rs`, `color/`) must maintain bit-level mathematical parity. Any new tone transform or slider adjustment must be implemented identically in both engines and verified with `cargo test --test agx_shader_parity` and `cargo test --test camera_profiles_parity`.

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

- **Proactive Customization Encoding**: Whenever a platform-specific quirk, non-obvious workaround, multi-step pipeline recipe, or architectural convention is established during development, proactively suggest encoding it into `.agents/AGENTS.md` or as a new skill under `.agents/skills/` so subsequent chats retain it automatically.
