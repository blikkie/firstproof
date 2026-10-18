---
name: firstproof-version-bump
description: >-
  Increment and synchronize project release version tags across Rust (Cargo.toml, Cargo.lock), Apple (Info.plist CFBundleShortVersionString and CFBundleVersion), Web (package.json, index.html), Makefile, and README.md. Use whenever preparing, tagging, or releasing a patch, minor, or major version of First Proof.
---

# First Proof Version Bumping & Release Skill

This skill documents how to atomically increment First Proof's version across its multiple language and platform ecosystems.

## Ecosystem Version Matrix

First Proof maintains synchronized version numbers across 5 environments:
1. **Rust Workspace**:
   - `Cargo.toml` (`[workspace.package].version`)
   - `firstproof-core/Cargo.toml`
   - `firstproof-engine/Cargo.toml`
   - `firstproof-wasm/Cargo.toml`
   - `Cargo.lock`
2. **macOS AppKit / SwiftUI**:
   - `FirstProofApp/Resources/Info.plist`:
     - `CFBundleShortVersionString` (e.g. `0.1.2`)
     - `CFBundleVersion` (integer build number, incremented by 1 each bump)
3. **Build Pipeline**:
   - `Makefile` (`VERSION ?= ...`)
4. **Web Darkroom & Distribution**:
   - `web/package.json` (`"version": "..."`)
   - `web/index.html` (header badge, hero version pill, download asset links)
5. **Documentation**:
   - `README.md` (release asset download names)

---

## Automated Bumping via `scripts/bump_version.py`

**Always use the automated bumper script** instead of manually searching and replacing strings:

```bash
# Bump patch version (e.g., 0.1.1 -> 0.1.2)
./scripts/bump_version.py patch

# Bump minor version (e.g., 0.1.2 -> 0.2.0)
./scripts/bump_version.py minor

# Set an explicit semantic version
./scripts/bump_version.py 0.2.0
```

The script automatically:
- Validates semantic versioning format (`X.Y.Z`).
- Updates all 9 files across Rust, Swift, Web, and Make.
- Increments `CFBundleVersion` build integer in `Info.plist`.
- Runs `cargo check --workspace` to ensure `Cargo.lock` is cleanly updated.

---

## Post-Bump Verification & Tagging Workflow

1. **Verify Integrity**:
   ```bash
   ./scripts/check.sh
   ```
2. **Review Diff**:
   ```bash
   git diff Cargo.toml FirstProofApp/Resources/Info.plist Makefile web/package.json
   ```
3. **Commit & Tag**:
   ```bash
   git add -A
   git commit -m "release(vX.Y.Z): <summary of changes>"
   git tag -a vX.Y.Z -m "Release vX.Y.Z: <release notes>"
   ```
