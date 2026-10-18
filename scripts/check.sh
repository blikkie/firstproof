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
echo "==> [1/4] Running Architectural Anti-Slop & Invariant Sensor..."
python3 scripts/anti_slop_sensor.py

# Step 2: Concurrent Multi-Toolchain Execution (Rust, Web, Swift)
echo "==> [2/4] Running Concurrent Verification Tracks (Rust, Web, Swift)..."

TMP_DIR="$(mktemp -d /tmp/firstproof_check_XXXXXX)"
trap 'rm -rf "$TMP_DIR"' EXIT

# --- Track A: Rust Core & Engine ---
(
    echo "    [Rust] Running license compliance gate..."
    cargo deny check licenses
    echo "    [Rust] Running workspace tests..."
    cargo test --workspace --quiet
    echo "    [Rust] Verifying canonical shader sync across Metal, WebGL, and WebGPU..."
    cargo run -p firstproof-core --bin firstproof-cli -- shaders --check
    echo "    [MCP] Running Model Context Protocol integration test..."
    python3 tools/mcp/test_mcp.py
    echo "    [Rust] Running Clippy linter..."

    cargo clippy --workspace --all-targets -- -D warnings
    echo "    [Rust] Building & verifying WebAssembly..."
    (cd firstproof-wasm && wasm-pack build --target web --out-dir pkg >/dev/null 2>&1)
    if [[ ! -d "firstproof-wasm/pkg-node" ]]; then
        (cd firstproof-wasm && wasm-pack build --target nodejs --out-dir pkg-node >/dev/null 2>&1)
    fi
    node firstproof-wasm/test_wasm.js >/dev/null
    echo "    ✅ [Rust Track] Passed cleanly"
) > "$TMP_DIR/rust.log" 2>&1 &
PID_RUST=$!

# --- Track B: Web & TypeScript ---
(
    echo "    [Web] Running Biome linter..."
    npm --prefix web run lint
    echo "    [Web] Running TypeScript typecheck..."
    npm --prefix web run typecheck
    echo "    [Web] Running Vitest behavioral tests..."
    npm --prefix web run test
    echo "    ✅ [Web Track] Passed cleanly"
) > "$TMP_DIR/web.log" 2>&1 &
PID_WEB=$!

# --- Track C: Darwin / macOS Swift Verification ---
if [[ "$(uname)" == "Darwin" ]]; then
    (
        echo "    [Swift] Verifying Swift packages and Metal targets..."
        swift build --package-path FirstProofBridge >/dev/null
        swift build --package-path FirstProofApp >/dev/null
        echo "    ✅ [Swift Track] Passed cleanly"
    ) > "$TMP_DIR/swift.log" 2>&1 &
    PID_SWIFT=$!
else
    PID_SWIFT=""
fi

# Wait and collect statuses
FAILED=0

if ! wait "$PID_RUST"; then
    echo "❌ [Rust Track] Failed! Log output:"
    cat "$TMP_DIR/rust.log"
    FAILED=1
else
    cat "$TMP_DIR/rust.log"
fi

if ! wait "$PID_WEB"; then
    echo "❌ [Web Track] Failed! Log output:"
    cat "$TMP_DIR/web.log"
    FAILED=1
else
    cat "$TMP_DIR/web.log"
fi

if [[ -n "$PID_SWIFT" ]]; then
    if ! wait "$PID_SWIFT"; then
        echo "❌ [Swift Track] Failed! Log output:"
        cat "$TMP_DIR/swift.log"
        FAILED=1
    else
        cat "$TMP_DIR/swift.log"
    fi
fi

if [[ "$FAILED" -ne 0 ]]; then
    echo "========================================================"
    echo "  ❌ Pre-Flight Verification Failed!"
    echo "========================================================"
    exit 1
fi

# Step 3: Web Production Assets Build (depends on both WASM and Web passing)
echo "==> [3/4] Building web production assets..."
npm --prefix web run build >/dev/null

echo "========================================================"
echo "  ✅ All Pre-Flight Sensors & Linters Passed Cleanly!"
echo "========================================================"
