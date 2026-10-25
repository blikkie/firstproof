#!/usr/bin/env bash
set -euo pipefail

# First Proof Unified Pre-Flight Sensor & Linter Suite
# Harness Engineering: Agent = Model + Harness (Martin Fowler)
# Enforces tests, linters, and typecheckers before handing off changes.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

PYTHON="python3"
if [[ -f "$ROOT_DIR/.venv/bin/python3" ]]; then
    PYTHON="$ROOT_DIR/.venv/bin/python3"
fi

MODE_ALL=0
WASM_BUILD_FLAG="--dev"

for arg in "$@"; do
    case "$arg" in
        -a|--all|--full)
            MODE_ALL=1
            ;;
        --release)
            WASM_BUILD_FLAG=""
            ;;
        -h|--help)
            echo "Usage: ./scripts/check.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -a, --all, --full   Run exhaustive sensor verification across all tracks"
            echo "  --release           Run wasm-pack in release mode with wasm-opt (slower)"
            echo "  -h, --help          Show this help message"
            exit 0
            ;;
    esac
done

echo "========================================================"
echo "  First Proof Pre-Flight Sensor & Linter Suite"
echo "========================================================"

# Step 1: Architectural Anti-Slop & Invariant Sensor
echo "==> [1/4] Running Architectural Anti-Slop & Invariant Sensor..."
"$PYTHON" scripts/anti_slop_sensor.py

# Path-aware track selection
if [[ "$MODE_ALL" -eq 1 ]]; then
    echo "ℹ️  Mode: Full exhaustive verification across all tracks (--all)"
    RUN_RUST=1
    RUN_WEB=1
    RUN_SWIFT=1
    RUN_BUILD=1
else
    CHANGED_FILES=$( {
        git diff --name-only HEAD 2>/dev/null || true
        git diff --name-only @{upstream}...HEAD 2>/dev/null || git diff --name-only HEAD~1..HEAD 2>/dev/null || true
        git status --porcelain 2>/dev/null | awk '{print $2}' || true
    } | sort -u | grep -v '^$' || true )

    if [[ -z "$CHANGED_FILES" ]]; then
        echo "ℹ️  No changes detected; running full sensor suite across all tracks."
        RUN_RUST=1
        RUN_WEB=1
        RUN_SWIFT=1
        RUN_BUILD=1
    else
        RUN_RUST=0
        RUN_WEB=0
        RUN_SWIFT=0
        RUN_BUILD=0

        while IFS= read -r file; do
            [[ -z "$file" ]] && continue
            # Root scripts, configs, or CI trigger all tracks
            if [[ "$file" =~ ^scripts/ || "$file" =~ ^\.github/ || "$file" == "Makefile" || "$file" == ".pre-commit-config.yaml" ]]; then
                RUN_RUST=1; RUN_WEB=1; RUN_SWIFT=1; RUN_BUILD=1
            fi
            # Shared shaders trigger all rendering tracks
            if [[ "$file" =~ shaders/ || "$file" =~ \.metal$ ]]; then
                RUN_RUST=1; RUN_WEB=1; RUN_SWIFT=1; RUN_BUILD=1
            fi
            # Rust core, engine, CLI, or workspace configs
            if [[ "$file" =~ ^firstproof-core/ || "$file" =~ ^firstproof-engine/ || "$file" =~ ^firstproof-cli/ || "$file" =~ ^Cargo\.(toml|lock) || "$file" == "deny.toml" ]]; then
                RUN_RUST=1
            fi
            # WASM spans both Rust and Web domains
            if [[ "$file" =~ ^firstproof-wasm/ ]]; then
                RUN_RUST=1; RUN_WEB=1; RUN_BUILD=1
            fi
            # Web, TypeScript, and web components
            if [[ "$file" =~ ^web/ ]]; then
                RUN_WEB=1; RUN_BUILD=1
            fi
            # Swift, AppKit, macOS views, and UniFFI Swift bridge
            if [[ "$file" =~ ^FirstProofApp/ || "$file" =~ ^FirstProofBridge/ || "$file" =~ ^firstproof-core/src/uniffi/ || "$file" == "Package.swift" ]]; then
                RUN_SWIFT=1
            fi
            # MCP tools
            if [[ "$file" =~ ^tools/mcp/ ]]; then
                RUN_RUST=1
            fi
        done <<< "$CHANGED_FILES"

        # Safety fallback: if changed files did not match specific rules, run everything
        if [[ "$RUN_RUST" -eq 0 && "$RUN_WEB" -eq 0 && "$RUN_SWIFT" -eq 0 ]]; then
            RUN_RUST=1; RUN_WEB=1; RUN_SWIFT=1; RUN_BUILD=1
        fi

        echo "ℹ️  Path-aware selective verification:"
        [[ "$RUN_RUST" -eq 1 ]] && echo "    • Rust Track: Active" || echo "    • Rust Track: Skipped (no changes)"
        [[ "$RUN_WEB" -eq 1 ]]  && echo "    • Web Track:  Active" || echo "    • Web Track:  Skipped (no changes)"
        [[ "$RUN_SWIFT" -eq 1 ]] && echo "    • Swift Track: Active" || echo "    • Swift Track: Skipped (no changes)"
    fi
fi

# Step 2: Concurrent Multi-Toolchain Execution (Rust, Web, Swift)
echo "==> [2/4] Running Concurrent Verification Tracks..."

TMP_DIR="$(mktemp -d /tmp/firstproof_check_XXXXXX)"
trap 'rm -rf "$TMP_DIR"' EXIT

PID_RUST=""
PID_WEB=""
PID_SWIFT=""

# --- Track A: Rust Core & Engine ---
if [[ "$RUN_RUST" -eq 1 ]]; then
    (
        echo "    [Rust] Running license compliance gate..."
        cargo deny check licenses
        echo "    [Rust] Running workspace tests..."
        cargo test --workspace --quiet
        echo "    [Rust] Verifying canonical shader sync across Metal, WebGL, and WebGPU..."
        cargo run -p firstproof-core --bin firstproof-cli -- shaders --check
        echo "    [MCP] Running Model Context Protocol integration test..."
        "$PYTHON" tools/mcp/test_mcp.py
        echo "    [Rust] Running Clippy linter..."
        cargo clippy --workspace --all-targets -- -D warnings
        echo "    [Rust] Building & verifying WebAssembly..."
        (cd firstproof-wasm && wasm-pack build ${WASM_BUILD_FLAG} --target web --out-dir pkg >/dev/null 2>&1)
        if [[ ! -d "firstproof-wasm/pkg-node" ]]; then
            (cd firstproof-wasm && wasm-pack build ${WASM_BUILD_FLAG} --target nodejs --out-dir pkg-node >/dev/null 2>&1)
        fi
        node firstproof-wasm/test_wasm.js >/dev/null
        echo "    ✅ [Rust Track] Passed cleanly"
    ) > "$TMP_DIR/rust.log" 2>&1 &
    PID_RUST=$!
fi

# --- Track B: Web & TypeScript ---
if [[ "$RUN_WEB" -eq 1 ]]; then
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
fi

# --- Track C: Darwin / macOS Swift Verification ---
if [[ "$RUN_SWIFT" -eq 1 && "$(uname)" == "Darwin" ]]; then
    (
        echo "    [Swift] Verifying Swift packages and Metal targets..."
        swift build --package-path FirstProofBridge >/dev/null
        swift build --package-path FirstProofApp >/dev/null
        echo "    ✅ [Swift Track] Passed cleanly"
    ) > "$TMP_DIR/swift.log" 2>&1 &
    PID_SWIFT=$!
fi

# Wait and collect statuses
FAILED=0

if [[ -n "$PID_RUST" ]]; then
    if ! wait "$PID_RUST"; then
        echo "❌ [Rust Track] Failed! Log output:"
        cat "$TMP_DIR/rust.log"
        FAILED=1
    else
        cat "$TMP_DIR/rust.log"
    fi
fi

if [[ -n "$PID_WEB" ]]; then
    if ! wait "$PID_WEB"; then
        echo "❌ [Web Track] Failed! Log output:"
        cat "$TMP_DIR/web.log"
        FAILED=1
    else
        cat "$TMP_DIR/web.log"
    fi
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
if [[ "$RUN_BUILD" -eq 1 ]]; then
    echo "==> [3/4] Building web production assets..."
    npm --prefix web run build >/dev/null
else
    echo "==> [3/4] Skipping web production assets build (no web/WASM changes)"
fi

echo "========================================================"
echo "  ✅ All Pre-Flight Sensors & Linters Passed Cleanly!"
echo "========================================================"
