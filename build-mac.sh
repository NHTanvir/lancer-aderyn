#!/usr/bin/env bash
set -euo pipefail

# macOS build for lancer-aderyn. Produces build/lancer-aderyn and, with
# cargo-bundle available, a lancer-aderyn.app bundle (GUI is the default mode).

REPO="$(cd "$(dirname "$0")" && pwd)"
BUILD="$REPO/build"
BIN="lancer-aderyn"

echo "================================================"
echo "  lancer-aderyn Build (macOS)"
echo "================================================"
echo ""
echo "  Repo   : $REPO"
echo "  Output : $BUILD/$BIN"
echo ""

echo "[1/3] Building release binary..."
cargo build --release

echo "[2/3] Copying to build/..."
mkdir -p "$BUILD"
cp "$REPO/target/release/$BIN" "$BUILD/"

echo "[3/3] Bundling .app (optional, requires cargo-bundle)..."
if command -v cargo-bundle >/dev/null 2>&1; then
    cargo bundle --release || echo "  (cargo bundle failed; the plain binary is still in build/)"
else
    echo "  cargo-bundle not installed; skipping .app bundle."
    echo "  Install with: cargo install cargo-bundle"
fi

echo ""
echo "================================================"
echo "  Done!  Binary ready: $BUILD/$BIN"
echo "================================================"
