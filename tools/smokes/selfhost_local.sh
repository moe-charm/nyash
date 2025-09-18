#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"

echo "[selfhost] Build compiler EXE ..." >&2
timeout -s KILL 10m bash tools/build_compiler_exe.sh --no-pack -o nyc >/dev/null

echo "[selfhost] Parse -> JSON (with comments/escapes) ..." >&2
cat > tmp/selfhost_src_smoke.nyash << 'SRC'
// hello
return (1 + 2*3) // 7
SRC

./nyc tmp/selfhost_src_smoke.nyash > tmp/selfhost_src_smoke.json
head -n1 tmp/selfhost_src_smoke.json | rg -q '"kind":"Program"'

echo "[selfhost] Execute JSON via PyVM ..." >&2
NYASH_VM_USE_PY=1 ./target/release/nyash --backend vm tmp/selfhost_src_smoke.json --json-file >/dev/null 2>&1 || true

echo "✅ selfhost_local OK" >&2

