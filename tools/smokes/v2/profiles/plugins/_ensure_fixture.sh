#!/bin/bash
# _ensure_fixture.sh - フィクスチャプラグイン自動準備

set -uo pipefail

detect_ext() {
  case "$(uname -s)" in
    Darwin) echo "dylib" ;;
    MINGW*|MSYS*|CYGWIN*|Windows_NT) echo "dll" ;;
    *) echo "so" ;;
  esac
}

lib_name_for() {
  local base="$1"  # e.g., nyash_fixture_plugin
  local ext="$2"
  if [ "$ext" = "dll" ]; then
      echo "${base}.dll"
  else
      echo "lib${base}.${ext}"
  fi
}

ensure_fixture_plugin() {
  local root="${NYASH_ROOT:-$(cd "$(dirname "$0")/../../../.." && pwd)}"
  local ext="$(detect_ext)"
  local out_dir="$root/plugins/nyash-fixture-plugin"
  local out_file="$out_dir/$(lib_name_for nyash_fixture_plugin "$ext")"

  mkdir -p "$out_dir"
  if [ -f "$out_file" ]; then
    echo "[INFO] Fixture plugin present: $out_file" >&2
    return 0
  fi

  echo "[INFO] Building fixture plugin (nyash-fixture-plugin) ..." >&2
  if cargo build --release -p nyash-fixture-plugin >/dev/null 2>&1; then
    local src=""
    case "$ext" in
      dll) src="$root/target/release/nyash_fixture_plugin.dll" ;;
      dylib) src="$root/target/release/libnyash_fixture_plugin.dylib" ;;
      so) src="$root/target/release/libnyash_fixture_plugin.so" ;;
    esac
    if [ -f "$src" ]; then
      cp -f "$src" "$out_file"
      echo "[INFO] Fixture plugin installed: $out_file" >&2
      return 0
    fi
  fi
  echo "[WARN] Could not build/install fixture plugin (will SKIP related tests)" >&2
  return 1
}

