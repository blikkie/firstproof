#!/usr/bin/env bash
set -euo pipefail

# First Proof Hardware & Lens Calibration Harvester
# Story 4.4: Standalone Desktop & CLI Release Packaging Pipeline
# Pulls upstream Lensfun CC-BY-SA 3.0 database and indexes it into a compact pre-indexed JSON database.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

LENSFUN_DIR="$ROOT_DIR/assets/lensfun"
LENSFUN_DB_DIR="$LENSFUN_DIR/db"
LENSFUN_JSON="$LENSFUN_DIR/lensfun.json"
APP_RESOURCES_DIR="$ROOT_DIR/FirstProofApp/Resources"
WEB_PUBLIC_DIR="$ROOT_DIR/web/public"

echo "========================================================"
echo "  First Proof Hardware & Lens Calibration Harvester"
echo "========================================================"

mkdir -p "$LENSFUN_DB_DIR"
mkdir -p "$APP_RESOURCES_DIR"

# Step 1: Download upstream Lensfun XML database snapshot
echo "==> [1/4] Fetching upstream Lensfun database snapshot from GitHub..."
TMP_TAR="$(mktemp /tmp/lensfun_snapshot_XXXXXX.tar.gz)"
curl -sL "https://github.com/lensfun/lensfun/archive/refs/heads/master.tar.gz" -o "$TMP_TAR"

echo "==> [2/4] Extracting calibration profiles into assets/lensfun/db/..."
tar -xz -C "$LENSFUN_DB_DIR" --strip-components=3 -f "$TMP_TAR" "lensfun-master/data/db"
rm -f "$TMP_TAR"

# Step 2: Write CC-BY-SA 3.0 Quarantine License
cat << 'EOF' > "$LENSFUN_DIR/LICENSE.txt"
Lensfun Lens Calibration Database
License: Creative Commons Attribution-ShareAlike 3.0 Unported (CC-BY-SA 3.0)
Upstream: https://github.com/lensfun/lensfun

This dataset contains optical distortion, transverse chromatic aberration, and vignetting
calibration profiles created and curated by the open-source Lensfun Project contributors.

In accordance with collective work / aggregation legal provisions, this dataset is quarantined
within assets/lensfun/ as a distinct data collection and does not alter the MIT license of
First Proof software code.

Attribution notice:
Lens profiles provided by the open-source Lensfun Project (CC-BY-SA 3.0).
EOF

# Step 3: Compile into pre-indexed lensfun.json
echo "==> [3/4] Compiling XML profiles into pre-indexed lensfun.json..."
cargo run -p firstproof-core --bin firstproof-cli -- bundle-lensfun "$LENSFUN_DB_DIR" "$LENSFUN_JSON"

# Step 4: Mirror to application resource bundles
echo "==> [4/4] Mirroring lensfun.json to desktop and web resource paths..."
cp "$LENSFUN_JSON" "$APP_RESOURCES_DIR/lensfun.json"

if [[ -d "$WEB_PUBLIC_DIR" ]]; then
    cp "$LENSFUN_JSON" "$WEB_PUBLIC_DIR/lensfun.json"
fi

TOTAL_XML=$(ls "$LENSFUN_DB_DIR"/*.xml 2>/dev/null | wc -l | tr -d ' ')
JSON_SIZE=$(du -h "$LENSFUN_JSON" | cut -f1)

echo "========================================================"
echo "  ✅ Hardware & Lens Calibration Harvester Complete!"
echo "     Indexed XMLs: $TOTAL_XML files"
echo "     Output:       assets/lensfun/lensfun.json ($JSON_SIZE)"
echo "     Desktop Res:  FirstProofApp/Resources/lensfun.json"
echo "========================================================"
