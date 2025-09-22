#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/../../.." && pwd)
cd "$ROOT_DIR"

echo "[smoke] build nyash (release)"
cargo build --release -q

BIN=./target/release/nyash
CHILD=apps/selfhost/compiler/compiler.nyash

if [[ ! -x "$BIN" ]]; then
  echo "nyash binary not found: $BIN" >&2
  exit 1
fi

echo "[smoke] run child (baseline)"
BASE=$("$BIN" --backend vm "$CHILD" -- --min-json)

echo "[smoke] run child (scopebox on)"
WITH=$("$BIN" --backend vm "$CHILD" -- --min-json --scopebox)

if [[ "$BASE" != "$WITH" ]]; then
  echo "❌ scopebox identity prepass altered JSON" >&2
  diff -u <(echo "$BASE") <(echo "$WITH") || true
  exit 1
fi

echo "$BASE" | grep -q '"kind":"Program"' || { echo "❌ baseline JSON missing Program kind" >&2; exit 1; }

echo "✅ scopebox identity smoke passed"

