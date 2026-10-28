---
name: firstproof-code-review
description: >-
  Pre-commit adversarial code review, cross-platform parity audit, and automated test harness verification subagent.
  Executes in a fresh subagent to eliminate confirmation bias, running mechanical checks (./scripts/check.sh),
  anti-slop invariants, adversarial diff reviews (races, retain cycles, texture clobbering, magic numbers),
  cross-platform feature completeness (macOS, Web, CLI), and feedback loop performance monitoring before any commit tour.
---

# Pre-Commit Code Review & Cross-Platform Parity Protocol

Before presenting an educational commit tour or requesting sign-off to commit, First Proof requires an independent, rigorous code review. To eliminate confirmation bias and prevent conversational sunk-cost rationalizations, **this skill must be executed via an independent subagent** with a clean-slate evaluation of the working tree.

---

## 1. Subagent Invocation Mandate

1. **Unbiased Subagent Execution**:
   - The primary agent must launch an independent subagent with a clean prompt referencing this skill.
   - The subagent starts with zero conversational bias and evaluates the repository state strictly through computational sensors, static diff analysis, and architectural invariants.
2. **Review Scope**:
   - The subagent inspects `git status` and `git diff HEAD` (or `git diff main...HEAD`).
   - The subagent executes all mechanical verification sensors.
   - The subagent produces a structured **Code Review Verdict Report** (`PASSED` or `ACTION REQUIRED`).
   - The primary agent must resolve all blocking findings before delivering the educational tour to the user.

---

## 2. The 5 Review Pillars

The review subagent evaluates the diff against five strict pillars:

### Pillar 1: Mechanical Verification & Harness Execution
Run the complete automated sensor gatekeeper:
```bash
./scripts/check.sh
```
Verify that all concurrent tracks succeed:
- **Anti-Slop Sensor**: `./scripts/anti_slop_sensor.py` passes with zero violations.
- **License Compliance**: `cargo-deny` / license checks pass cleanly.
- **Rust Core & Engine Tests**: Workspace tests (`cargo test --workspace`) pass without failures.
- **Canonical Shader Parity**: WGSL cross-compilation via Naga (`firstproof-cli shaders --check`) confirms bit-level sync between `Shaders.metal`, `GeneratedGLSL.ts`, and `CanonicalWGSL.ts`.
- **MCP Sensor**: Model Context Protocol integration test passes.
- **Linters**:
  - `cargo clippy --workspace --all-targets` exits 0.
  - `npm --prefix web run lint` (Biome) exits 0.
  - `npm --prefix web run typecheck` (`tsc --noEmit`) exits 0.
  - `npm --prefix web test` (Vitest) passes all behavioral tests.
- **Swift Testing**: `swift test --package-path FirstProofApp` passes cleanly in-process.

---

### Pillar 2: AI Slop & Anti-Hedge Audit
Inspect every modified and newly added line in `git diff` for digital slop and speculative defensive patterns:
1. **Zero Defensive Shims for First-Party Code**:
   - Forbid `typeof x === 'function'`, `fn?: ...`, or optional property chaining on first-party internal crates (`firstproof-wasm`, `firstproof-core`, UniFFI bridge).
   - First-party code must be statically imported and directly invoked.
2. **Zero Semantic Fallbacks & Swallowed Errors**:
   - Forbid falling back to low-res/binned buffers when full linear is requested.
   - Forbid swallowing Rust UniFFI errors in Swift with `try? FirstProof...`—all errors must be propagated or handled via explicit `do-catch`.
   - Forbid empty JavaScript/TypeScript `catch {}` blocks.
3. **Anti-Plastic & Color Science Invariants**:
   - Forbid synthetic AI pixel generation, sky replacements, or hallucinatory filters.
   - Verify **Boundary Anchoring** ($f(0.0) \equiv 0.0, f(1.0) \equiv 1.0$) across all develop adjustments. Forbid additive digital floor lifts (`+ 0.016` or `+ 0.035`).
   - Verify that display RGB conversion is strictly the **very last step** of the pipeline (Linear Rec.2020 optical domain preserved throughout).
   - Verify strict separation of color models (never apply camera-matching JPEG heuristics to filmic AgX tone mappers).

---

### Pillar 3: Bog-Standard Adversarial Code Review
Perform a deep, adversarial inspection of the diff looking for common failure modes:
1. **Concurrency & Race Conditions**:
   - Are async promises or network requests properly synchronized or cancelled when the user rapidly switches photos (e.g. `requestId` stamping in loaders)?
   - Are debounce timers cleared before setting new ones (`clearTimeout`, `Timer`)?
   - Are mutable states accessed safely across threads in Rust/Swift (`Mutex`, `RwLock`, actor isolation)?
2. **Memory & Lifecycle Management**:
   - Are event listeners cleaned up on unbind / `.onDisappear`?
   - Are WebGL/Metal textures, VAOs, or buffers leaked or overwritten without proper reallocation?
   - Are retain cycles prevented in Swift closures (`[weak self]`, `[weak viewModel]`)?
3. **Main-Thread Responsiveness**:
   - Are heavy demosaicing, large XMP parsing, or disk operations offloaded to background queues/workers?
   - Is keyboard event monitoring guarded by first-responder shielding so typing in metadata text fields never triggers single-key shortcuts?
4. **Anti-Magic Number Audit**:
   - Rigorously inspect every camera processing, color science, and develop adjustment for arbitrary magic numbers, localized scalar multipliers, or piecewise if/else heuristics.
   - Every parameter must trace directly to published optical, sensitometric, or sensor standards (e.g. CIE 1931, AgX, DNG specification, Lensfun optical polynomials).

---

### Pillar 4: Universal Cross-Platform Feature Completeness & Parity
**The Parity Invariant**: Every feature, slider, toggle, metadata field, or workflow touched **MUST be implemented on all supported platforms** unless explicitly documented and justified as platform-exclusive (e.g. macOS AppKit window management, external volume portable catalog dotfiles).

The subagent must verify cross-platform completeness:
| Layer | macOS Native (`FirstProofApp`) | Web Darkroom (`web/`) | Rust Engine / CLI (`firstproof-core`) |
| :--- | :--- | :--- | :--- |
| **Domain Models & Settings** | UniFFI `EditSettings` / `FirstProofBridge` | `EditSettingsPayload` in `firstproof-wasm` | `EditSettings` in `firstproof-engine` |
| **GPU Viewport Shaders** | `Shaders.metal` & `MetalCanvasView` | `GeneratedGLSL.ts` & `WebGLRenderer` | `darkroom.wgsl` & Naga cross-compilation |
| **Inspector Controls** | SwiftUI Inspector Workspace View | `index.html` & `InspectorSyncController` | Serialization & XMP sidecar persistence |
| **Optical / Calibration** | `FirstProofLensfun` bundle resolution | `WasmBridge.resolveLensCoefficients` | `LensfunDatabase` in `firstproof-engine` |
| **Export Parity** | `export.rs` / `encoders.rs` | `WasmBridge.exportRawFromScratchpad` | CLI headless export / ICC injection |

**Rule**: If a feature is implemented on macOS but left as a stub or hardcoded constant in Web (or vice versa), the review **FAILS**.

---

### Pillar 5: Feedback Loop Health & Performance Monitoring
The speed of the computational steering loop is vital. The subagent must inspect execution times and flag feedback loop regressions:
1. **Outer-Loop Gatekeeper Baseline**:
   - Healthy `./scripts/check.sh` runtime is **under 45 seconds**.
   - If `./scripts/check.sh` exceeds 60 seconds, investigate and flag the offending test or build step.
2. **Anti-Magic File & Slow Disk Test Isolation**:
   - Ensure pure unit tests do not depend on untracked magic RAW files (`FIRSTPROOF_TEST_RAW_DIR`, `FIRSTPROOF_PIXEL_SAMPLE`).
   - Ensure slow multi-MB real RAW decode benchmarks or disk transcode batches are properly gated behind `#[ignore = "..."]` so they do not degrade the fast inner loop.
   - Verify that redundant compilation passes are avoided in test scripts.

---

### Pillar 6: Tone & Copy Restraint Audit
Inspect any newly added or modified copy, UI labels, tooltips, onboarding, error messages, dialogs, docstrings, or commit messages in the diff:
1. **Understated, Factual, Warm Tone**:
   - First Proof is dedicated to photographic restraint and pure capture: *"Pure capture. Honest color. Zero digital plastic."*
   - Copy must read quiet, factual, warm, and understated. We are not a hungry startup pitching users; less is more; the software speaks through its optical output.
2. **Strictly Forbidden Marketing Hype & Buzzwords**:
   - Forbid breathless buzzwords: *"revolutionary"*, *"blazing-fast"*, *"game-changing"*, *"magical"*, *"all-in-one"*, *"next-gen"*, *"AI-powered"*.
   - Forbid conversion pressure, promotional urgency, artificial scarcity, or patronizing emojis in interface copy.
   - UI descriptions should simply describe the physical, optical, or sensitometric reality (e.g. *"Subtractive 3-layer dye cloud simulation"*, *"Cosine-4th radial peripheral falloff correction"*).

---

## 3. Subagent Review Verdict Contract

The review subagent must end its report with a clear, structured summary:

```markdown
# 🛡️ Code Review & Parity Audit Report

## 1. Executive Verdict
**[PASSED | ACTION REQUIRED]**
*(Concise 1-2 sentence executive summary of review findings)*

## 2. Verification Matrix
| Sensor Track | Command / Check | Result | Duration |
| :--- | :--- | :--- | :--- |
| **Mechanical Sensor Gate** | `./scripts/check.sh` | PASS / FAIL | XX.Xs |
| **Anti-Slop Sensor** | `./scripts/anti_slop_sensor.py` | PASS / FAIL | X.Xs |
| **Adversarial Diff Audit** | Concurrency, Memory, Logic | PASS / FAIL | - |
| **Cross-Platform Parity** | macOS / Web / Engine Sync | PASS / FAIL | - |
| **Feedback Loop Health** | Check suite duration (<45s) | PASS / REGRESSION | XX.Xs |
| **Tone & Copy Restraint** | Factual, quiet, zero hype | PASS / FAIL | - |

## 3. Detailed Findings & Blocking Issues
*(If any issues were found, list file paths, line numbers, and actionable remediation steps. If none, state: "Zero blocking issues discovered.")*

## 4. Cross-Platform Parity Audit
- **Feature Scope**: *(What features/subsystems were touched)*
- **Native macOS Implementation**: *(Status & files)*
- **Web Darkroom Implementation**: *(Status & files)*
- **Shared Rust Engine / WASM**: *(Status & files)*
- **Parity Status**: `100% Verified Parity` or `Divergence Identified`

## 5. Feedback Loop Health Telemetry
- Total `./scripts/check.sh` duration: `XX.X seconds`
- Regression status: `Healthy (no slow disk/magic file bloat)`

## 6. Prescribed Manual Testing Instructions
*(Mandatory if rendering, shaders, color science, tone curves, grain, or pointer gestures were modified)*
1. **Test Fixtures**: *(e.g. CR2, DNG, RAF captures to test)*
2. **Interactive Controls**: *(e.g. Sliders, toggles, workspaces to scrub)*
3. **Perceptual Inspection**: *(Concrete visual or optical artifacts to observe)*
```

---

## 4. Primary Agent Workflow Integration

Whenever an agent finishes implementing or modifying code:
1. **Do NOT run a commit tour immediately.**
2. Launch the `firstproof-code-review` subagent.
3. If the subagent returns `ACTION REQUIRED`, fix all issues, re-run targeted inner-loop tests, and re-verify.
4. Once the subagent returns `PASSED`, include the subagent's audit summary and manual verification instructions in the educational commit tour presented to the user.
5. Only execute `git commit` after user review and explicit confirmation.
