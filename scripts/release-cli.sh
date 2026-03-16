#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-aarch64-apple-darwin}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist"
VERSION="${PRAXISKIT_VERSION:-1.1.0}"
BIN_NAME="praxis"
ARCHIVE="$DIST/${BIN_NAME}-${VERSION}-${TARGET}.tar.gz"

mkdir -p "$DIST"

cd "$ROOT"
cargo build --release -p praxis-cli --target "$TARGET"

TMP_DIR="$(mktemp -d)"
cp "$ROOT/target/$TARGET/release/$BIN_NAME" "$TMP_DIR/$BIN_NAME"
tar -C "$TMP_DIR" -czf "$ARCHIVE" "$BIN_NAME"
rm -rf "$TMP_DIR"

echo "created $ARCHIVE"
