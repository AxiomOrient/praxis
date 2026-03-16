#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP_DIR="$ROOT/apps/praxis-desktop"
DIST="$ROOT/dist"
VERSION="${PRAXISKIT_VERSION:-1.1.0}"
ARCHIVE="$DIST/praxis-${VERSION}-macos-app.zip"

mkdir -p "$DIST"

cd "$APP_DIR"
npm run build
npm run tauri -- build

APP_PATH="$(find src-tauri/target/release/bundle -maxdepth 3 -name 'praxis.app' | head -n 1)"
if [[ -z "$APP_PATH" ]]; then
  echo "praxis.app not found" >&2
  exit 1
fi

cd "$(dirname "$APP_PATH")"
zip -r "$ARCHIVE" "$(basename "$APP_PATH")"

echo "created $ARCHIVE"
