---
name: lumenbox-build-and-bundle
description: >-
  Build and package the LumenBox macOS application bundle, compile the Rust core, regenerate UniFFI Swift bridge bindings, and run native verification tests. Use whenever building, running, packaging, or testing LumenBox.
---

# LumenBox Build, Bundle & Test Guide

This skill documents the build pipeline, FFI bridge generation, and application packaging for LumenBox (Rust + UniFFI + Swift + Metal).

## Build Tools & Sandbox Rules

- `swift build` and `make bundle` invoke the Darwin linker (`ld`) and system headers. They require running with `BypassSandbox: true`.
- Pure Rust commands (`cargo check`, `cargo test`) can execute in standard sandboxed mode (`BypassSandbox: false`).

---

## 1. Complete Application Bundle (`make bundle`)

To produce the production/debug standalone macOS application at `dist/LumenBox.app`:

```bash
make bundle
```

This single command automates the entire 6-step build pipeline:
1. **Rust Core Compilation**:
   ```bash
   cargo build --release --target aarch64-apple-darwin -p lumenbox-core
   ```
2. **UniFFI Swift & C Header Generation**:
   ```bash
   cargo run --manifest-path lumenbox-core/Cargo.toml --target aarch64-apple-darwin \
     --features uniffi/cli --bin uniffi-bindgen -- generate \
     --library target/aarch64-apple-darwin/release/liblumenbox_core.dylib \
     --language swift \
     --out-dir LumenBoxBridge/Sources/LumenBoxBridge/ \
     --no-format --metadata-no-deps
   ```
3. **Module Mapping & Static Archive Copy**:
   Moves `lumenbox_coreFFI.h` and modulemap to `LumenBoxBridge/Sources/LumenBoxCoreFFI/include/` and copies `liblumenbox_core.a`.
4. **Swift Bridge Compilation**:
   ```bash
   swift build --package-path LumenBoxBridge
   ```
5. **Swift App Compilation**:
   ```bash
   swift build --package-path LumenBoxApp
   ```
6. **Bundle Assembly & Ad-hoc Code Signing**:
   Copies binary to `dist/LumenBox.app/Contents/MacOS/LumenBoxApp`, injects `Info.plist`, Metal shaders, and app icon, then signs:
   ```bash
   codesign --force --deep --sign - dist/LumenBox.app
   ```

---

## 2. Fast Development Iteration (`swift build`)

When only Swift UI or ViewModels have changed (and no Rust structs or bridge methods changed):

```bash
swift build --package-path LumenBoxApp
```
*(Run with `BypassSandbox: true`)*

---

## 3. Running Core & Parity Tests

Always run Rust core tests before check-in:

```bash
# Run all unit tests and integration tests
cargo test --manifest-path lumenbox-core/Cargo.toml

# Run GPU Metal vs CPU Rust dual-engine parity test
cargo test --manifest-path lumenbox-core/Cargo.toml --test agx_shader_parity

# Run camera color profile parity tests
cargo test --manifest-path lumenbox-core/Cargo.toml --test camera_profiles_parity
```

---

## 4. Launching the App

```bash
open dist/LumenBox.app
```
*(Optionally launch from terminal to see stdout/stderr logs)*:
```bash
./dist/LumenBox.app/Contents/MacOS/LumenBoxApp
```
