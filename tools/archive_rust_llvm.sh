#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
SRC_DIR="$ROOT_DIR/src/backend/llvm"
ARCHIVED_DIR="$ROOT_DIR/archived"
LEGACY_DIR="$ROOT_DIR/src/backend/llvm_legacy"

mkdir -p "$ARCHIVED_DIR"

if [ ! -d "$SRC_DIR" ]; then
  echo "[archive] nothing to archive: $SRC_DIR not found" >&2
  exit 0
fi

STAMP=$(date +%Y%m%d)
TAR_PATH="$ARCHIVED_DIR/rust_llvm_${STAMP}.tar.gz"

echo "[archive] creating archive: $TAR_PATH"
tar -czf "$TAR_PATH" -C "$ROOT_DIR" \
  src/backend/llvm \
  --exclude="*.o" --exclude="*.so" --exclude="target" || true

if [ -d "$LEGACY_DIR" ]; then
  echo "[archive] legacy directory already exists: $LEGACY_DIR"
else
  echo "[archive] moving to legacy: $LEGACY_DIR"
  mv "$SRC_DIR" "$LEGACY_DIR"
  echo "# DEPRECATED - Use src/llvm_py/ instead" > "$LEGACY_DIR/DEPRECATED.md"
fi

echo "[archive] done."

