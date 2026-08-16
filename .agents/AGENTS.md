# Project Rules & Workflows

- **Environment & Toolchain Errors**: Whenever encountering an error that stems from environment or development toolchain setup (e.g., SDK version mismatches, toolchain configuration issues, missing system packages/Xcode), STOP and check in with the user immediately rather than spending cycles trying complex workarounds.

- **Adversarial Code Reviews**: Before every commit and check-in, perform an adversarial review (probing diffs for race conditions, memory leaks/retain cycles, error boundary misses, out-of-order async responses, and main-thread bottlenecks).

- **Educational & Interactive Commit Tours**: Before every commit/PR check-in, provide a tour of the code changes and use it as a learning opportunity. Compare Rust and Swift concepts against Python and JavaScript (e.g., ownership vs garbage collection, static typing, FFI, memory layout, concurrency). Include clear code snippets, explain syntax/architecture decisions, and end with 1-2 interactive quiz questions to reinforce fluency in desktop app engineering.

