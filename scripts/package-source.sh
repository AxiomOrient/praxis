#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist"
VERSION="${PRAXISKIT_VERSION:-1.1.0}"
NAME="praxis-${VERSION}-source"
OUT="$DIST/${NAME}.zip"

mkdir -p "$DIST"
rm -f "$OUT"

cd "$ROOT"
zip -r "$OUT" . \
  -x 'dist/*' \
  -x 'target/*' \
  -x 'apps/praxis-desktop/node_modules/*' \
  -x '.git/*' \
  -x '.DS_Store'

echo "created $OUT"
