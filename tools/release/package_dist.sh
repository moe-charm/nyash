#!/usr/bin/env bash
set -euo pipefail

DIST=dist
mkdir -p "$DIST/pkg"

# Create per-platform archives if artifacts exist
pkg_one() {
  local name="$1" ; shift
  local out="$DIST/pkg/$name"
  rm -f "$out" || true
  if [[ "$name" == *.zip ]]; then
    zip -j "$out" "$@" >/dev/null
  else
    tar -C "$DIST" -czf "$out" $(for f in "$@"; do basename "$f"; done)
  fi
  echo "[pkg] $out"
}

cp_if_exists() { # src -> dist
  local src="$1"; local base; base=$(basename "$src")
  if [ -f "$src" ]; then cp -f "$src" "$DIST/$base"; echo "$DIST/$base"; else echo ""; fi
}

# Pick latest release notes (v1.1 preferred, fallback to v1.0)
find_notes() {
  local cand1="dist/RELEASE_NOTES_v1.1-frozen.md"
  local cand0="dist/RELEASE_NOTES_v1.0-frozen.md"
  if [ -f "$cand1" ]; then echo "$cand1"; elif [ -f "$cand0" ]; then echo "$cand0"; else echo ""; fi
}

# Linux
if [ -f "$DIST/hako-frozen-v1-linux-x64" ]; then
  QS="$(cp_if_exists docs/guides/README_FROZEN_QUICKSTART.md)"
  RN="$(find_notes)"
  pkg_one hakorune-frozen-v1-linux-x64.tar.gz \
    "$DIST/hako-frozen-v1-linux-x64" "$RN" "$DIST/HASHES.txt" ${QS:+"$QS"}
fi

# Windows GNU
if [ -f "$DIST/hako-frozen-v1-win-x64-gnu.exe" ]; then
  QS="$(cp_if_exists docs/guides/README_FROZEN_QUICKSTART.md)"
  RN="$(find_notes)"
  pkg_one hakorune-frozen-v1-win-x64-gnu.zip \
    "$DIST/hako-frozen-v1-win-x64-gnu.exe" "$RN" "$DIST/HASHES.txt" ${QS:+"$QS"}
fi

# Windows MSVC
if [ -f "$DIST/hako-frozen-v1-win-x64-msvc.exe" ]; then
  QS="$(cp_if_exists docs/guides/README_FROZEN_QUICKSTART.md)"
  RN="$(find_notes)"
  pkg_one hakorune-frozen-v1-win-x64-msvc.zip \
    "$DIST/hako-frozen-v1-win-x64-msvc.exe" "$RN" "$DIST/HASHES.txt" ${QS:+"$QS"}
fi

# All-in-one
if [ -f "$DIST/hako-frozen-v1-linux-x64" ] && [ -f "$DIST/hako-frozen-v1-win-x64-msvc.exe" ]; then
  QS="$(cp_if_exists docs/guides/README_FROZEN_QUICKSTART.md)"
  RN="$(find_notes)"
  pkg_one hakorune-frozen-v1-all.zip \
    "$DIST/hako-frozen-v1-linux-x64" "$DIST/hako-frozen-v1-win-x64-msvc.exe" \
    "$RN" "$DIST/HASHES.txt" ${QS:+"$QS"}
fi

# Manifest
python3 tools/release/make_release_manifest.py >/dev/null || true

echo "[pkg] done"
