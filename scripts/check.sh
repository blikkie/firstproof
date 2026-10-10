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

# Step 1: Architectural Anti-Slop & Invariant Sensor
echo "==> [1/9] Running Architectural Anti-Slop & Invariant Sensor..."
python3 scripts/anti_slop_sensor.py

# Step 2: Rust Workspace Tests
echo "==> [2/9] Running Rust workspace tests..."
cargo test --workspace --quiet

# Step 3: Rust Clippy Lint Suite
echo "==> [3/9] Running Rust Clippy linter..."
cargo clippy --workspace --all-targets -- -D warnings

# Step 4: Web TypeScript Typecheck
echo "==> [4/9] Running TypeScript typecheck (web)..."
npm --prefix web run typecheck

# Step 5: Web Biome Linter
echo "==> [5/9] Running Biome linter (web)..."
npm --prefix web run lint

# Step 6: Web Vitest Behavioral Suite
echo "==> [6/9] Running Vitest behavioral tests (web)..."
npm --prefix web run test

# Step 7: WebAssembly Package & Tests
echo "==> [7/9] Verifying WebAssembly build & tests..."
(cd firstproof-wasm && wasm-pack build --target web --out-dir pkg >/dev/null 2>&1)
if [[ ! -d "firstproof-wasm/pkg-node" ]]; then
    (cd firstproof-wasm && wasm-pack build --target nodejs --out-dir pkg-node >/dev/null 2>&1)
fi
node firstproof-wasm/test_wasm.js >/dev/null

# Step 8: Web Client Build
echo "==> [8/9] Building web production assets..."
npm --prefix web run build >/dev/null

# Step 9: Darwin / macOS Swift Verification
echo "==> [9/9] Checking native Swift targets..."
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
