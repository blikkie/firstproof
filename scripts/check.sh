#!/usr/bin/env bash
set -euo pipefail

# First Proof Unified Pre-Flight Sensor & Linter Suite
# Harness Engineering: Agent = Model + Harness (Martin Fowler)
# Enforces tests, linters, and typecheckers before handing off changes.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "========================================================"
echo "  First Proof Pre-Flight Sensor & Linter Suite"
echo "========================================================"

# Step 1: Rust Workspace Tests
echo "==> [1/8] Running Rust workspace tests..."
cargo test --workspace --quiet

# Step 2: Rust Clippy Lint Suite
echo "==> [2/8] Running Rust Clippy linter..."
cargo clippy --workspace --all-targets -- -D warnings

# Step 3: Web TypeScript Typecheck
echo "==> [3/8] Running TypeScript typecheck (web)..."
npm --prefix web run typecheck

# Step 4: Web Biome Linter
echo "==> [4/8] Running Biome linter (web)..."
npm --prefix web run lint

# Step 5: Web Vitest Behavioral Suite
echo "==> [5/8] Running Vitest behavioral tests (web)..."
npm --prefix web run test

# Step 6: WebAssembly Package & Tests
echo "==> [6/8] Verifying WebAssembly build & tests..."
(cd firstproof-wasm && wasm-pack build --target web --out-dir pkg >/dev/null 2>&1)
node firstproof-wasm/test_wasm.js >/dev/null

# Step 7: Web Client Build
echo "==> [7/8] Building web production assets..."
npm --prefix web run build >/dev/null

# Step 8: Darwin / macOS Swift Verification
echo "==> [8/8] Checking native Swift targets..."
if [[ "$(uname)" == "Darwin" ]]; then
    echo "    Running on macOS: verifying Swift packages and tests..."
    swift build --package-path FirstProofBridge
    swift build --package-path FirstProofApp
else
    echo "    Non-Darwin host ($(uname)): skipping macOS Swift/Metal compilation."
fi

echo "========================================================"
echo "  ✅ All Pre-Flight Sensors & Linters Passed Cleanly!"
echo "========================================================"
