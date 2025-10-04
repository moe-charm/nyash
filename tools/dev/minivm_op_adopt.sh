#!/usr/bin/env bash
# minivm_op_adopt.sh — Injects {"__op_adopt__":1} into JSON v0 and runs VM
# Purpose: enable Mini‑VM compare parity observation without touching the runner.
# Usage: tools/dev/minivm_op_adopt.sh <json_v0_file> [-- extra nyash args]

set -euo pipefail

if [ $# -lt 1 ]; then
  echo "usage: $0 <json_v0_file> [-- extra nyash args]" >&2
  exit 2
fi

JSON="$1"; shift || true
ARGS=("$@")

if [ ! -f "$JSON" ]; then
  echo "error: file not found: $JSON" >&2
  exit 2
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"  # tools/
REPO_ROOT="$(cd "$ROOT_DIR/.." && pwd)"

# Resolve nyash binary (prefer release)
NYASH_BIN="${NYASH_BIN:-}"
if [ -z "${NYASH_BIN:-}" ]; then
  for cand in "$REPO_ROOT/target/release/nyash" "$REPO_ROOT/target/debug/nyash"; do
    if [ -x "$cand" ]; then NYASH_BIN="$cand"; break; fi
  done
fi
if [ -z "${NYASH_BIN:-}" ]; then
  echo "error: nyash binary not found (set NYASH_BIN or build target)" >&2
  exit 2
fi

TMP_JSON="/tmp/minivm_op_adopt_$$.json"
trap 'rm -f "$TMP_JSON"' EXIT

# Inject marker after opening '{'
if head -c 1 "$JSON" | grep -q '{'; then
  # naive insert: {"__op_adopt__":1,<rest>
  # Handle empty object edge safely
  python3 - "$JSON" > "$TMP_JSON" << 'PY2'
import sys
p=sys.argv[1]
s=open(p,'r',encoding='utf-8').read()
if not s.strip().startswith('{'):
    sys.stdout.write(s); sys.exit(0)
idx=s.find('{')
if idx<0:
    sys.stdout.write(s); sys.exit(0)
rest=s[idx+1:]
if rest.strip().startswith('}'):
    out=s[:idx+1] + '"__op_adopt__":1' + s[idx+1:]
else:
    out=s[:idx+1] + '"__op_adopt__":1,' + s[idx+1:]
sys.stdout.write(out)
PY2
else
  cp "$JSON" "$TMP_JSON"
fi

export NYASH_JSON_V0_DIRECT=1
exec "$NYASH_BIN" --backend vm "${ARGS[@]}" < "$TMP_JSON"
