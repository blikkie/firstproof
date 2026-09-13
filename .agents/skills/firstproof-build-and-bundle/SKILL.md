---
name: firstproof-build-and-bundle
description: >-
  Build and package the First Proof macOS application bundle, compile the Rust core, regenerate UniFFI Swift bridge bindings, and run native verification tests. Use whenever building, running, packaging, or testing First Proof.
---

# First Proof Build, Bundle & Test Guide

This skill documents the build pipeline, FFI bridge generation, and application packaging for First Proof (Rust + UniFFI + Swift + Metal).

## Build Tools & Sandbox Rules

- `swift build` and `make bundle` invoke the Darwin linker (`ld`) and system headers. They require running with `BypassSandbox: true`.
- Pure Rust commands (`cargo check`, `cargo test`) can execute in standard sandboxed mode (`BypassSandbox: false`).

---

## 1. Complete Application Bundle (`make bundle`)

To produce the production/debug standalone macOS application at `dist/FirstProof.app`:

```bash
make bundle
```

This single command automates the entire 6-step build pipeline:
1. **Rust Core Compilation**:
   ```bash
   cargo build --release --target aarch64-apple-darwin -p firstproof-core
   ```
2. **UniFFI Swift & C Header Generation**:
   ```bash
   cargo run --manifest-path firstproof-core/Cargo.toml --target aarch64-apple-darwin \
     --features uniffi/cli --bin uniffi-bindgen -- generate \
     --library target/aarch64-apple-darwin/release/libfirstproof_core.dylib \
     --language swift \
     --out-dir FirstProofBridge/Sources/FirstProofBridge/ \
     --no-format --metadata-no-deps
   ```
3. **Module Mapping & Static Archive Copy**:
   Moves `firstproof_coreFFI.h` and modulemap to `FirstProofBridge/Sources/FirstProofCoreFFI/include/` and copies `libfirstproof_core.a`.
4. **Swift Bridge Compilation**:
   ```bash
   swift build --package-path FirstProofBridge
   ```
5. **Swift App Compilation**:
   ```bash
   swift build --package-path FirstProofApp
   ```
6. **Bundle Assembly & Ad-hoc Code Signing**:
   Copies binary to `dist/FirstProof.app/Contents/MacOS/FirstProofApp`, injects `Info.plist`, Metal shaders, and app icon, then signs:
   ```bash
   codesign --force --deep --sign - dist/FirstProof.app
   ```

---

## 2. Fast Development Iteration (`swift build`)

When only Swift UI or ViewModels have changed (and no Rust structs or bridge methods changed):

```bash
swift build --package-path FirstProofApp
```
*(Run with `BypassSandbox: true`)*

---

## 3. Running Core & Parity Tests

Always run Rust core tests before check-in:

```bash
# Run all unit tests and integration tests
cargo test --manifest-path firstproof-core/Cargo.toml

# Run zero-I/O synthetic dual-engine parity regression suite (Macbeth 24 & neutral ramp)
make test-parity
# or directly:
cargo test --manifest-path firstproof-core/Cargo.toml --test dual_engine_synthetic_parity

# Run GPU Metal vs CPU Rust AgX parity test
cargo test --manifest-path firstproof-core/Cargo.toml --test agx_shader_parity

# Run camera color profile parity tests
cargo test --manifest-path firstproof-core/Cargo.toml --test camera_profiles_parity
```

---

## 4. Launching the App

```bash
open dist/FirstProof.app
```
*(Optionally launch from terminal to see stdout/stderr logs)*:
```bash
./dist/FirstProof.app/Contents/MacOS/FirstProofApp
```
