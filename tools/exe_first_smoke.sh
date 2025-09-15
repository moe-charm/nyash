#!/usr/bin/env bash
# EXE-first smoke: build the selfhost parser EXE and run a tiny program end-to-end.
set -euo pipefail

if [[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]]; then set -x; fi

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

echo "[1/4] Building parser EXE bundle ..."
tools/build_compiler_exe.sh >/dev/null

echo "[2/4] Preparing sample source ..."
mkdir -p dist/nyash_compiler/tmp
echo 'return 1+2*3' > dist/nyash_compiler/tmp/sample_exe_smoke.nyash

echo "[3/4] Running parser EXE → JSON ..."
(cd dist/nyash_compiler && ./nyash_compiler tmp/sample_exe_smoke.nyash > sample.json)

if ! head -n1 dist/nyash_compiler/sample.json | grep -q '"kind":"Program"'; then
  echo "error: JSON does not look like a Program" >&2
  exit 2
fi

echo "[4/4] Executing via bridge (pipe) to verify semantics ..."
# Keep core minimal and deterministic
export NYASH_DISABLE_PLUGINS=1
set +e
cat dist/nyash_compiler/sample.json | ./target/release/nyash --ny-parser-pipe --backend vm >/dev/null
RC=$?
set -e
if [[ "$RC" -ne 7 ]]; then
  echo "error: expected exit code 7, got $RC" >&2
  exit 3
fi

echo "✅ EXE-first smoke passed (parser EXE + bridge run)"
exit 0

