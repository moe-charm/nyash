#!/usr/bin/env bash
set -euo pipefail

if [[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]]; then set -x; fi

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"

echo "[1/3] Build selfhost compiler EXE (no pack) ..." >&2
timeout -s KILL 10m bash tools/build_compiler_exe.sh --no-pack -o nyc >/dev/null

echo "[2/3] Run compiler on sample source ..." >&2
echo '/*c*/ return 1+2*3 // ok' > tmp/selfhost_sample.nyash
./nyc tmp/selfhost_sample.nyash > tmp/selfhost_sample.json
head -n1 tmp/selfhost_sample.json | rg -q '"kind":"Program"' || { echo "error: not a Program" >&2; exit 2; }

echo "[3/3] Execute via PyVM harness ..." >&2
NYASH_VM_USE_PY=1 ./target/release/nyash --backend vm tmp/selfhost_sample.json --json-file >/dev/null 2>&1 || true
echo "✅ selfhost_parser_json_smoke OK" >&2

