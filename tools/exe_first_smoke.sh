#!/usr/bin/env bash
# EXE-first smoke: build the selfhost parser EXE and run a tiny program end-to-end.
set -euo pipefail

if [[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]]; then set -x; fi

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"
KERNEL="crates/hako_kernel/target/release/libhako_kernel.a"
ALT_KERNEL1="target/release/libhako_kernel.a"
ALT_KERNEL2="target/release/libnyash_kernel.a"
if [ ! -f "$KERNEL" ] && [ ! -f "$ALT_KERNEL1" ] && [ ! -f "$ALT_KERNEL2" ]; then
  echo "[SKIP] hako_kernel static runtime missing; skipping EXE-first" >&2
  exit 0
fi

echo "[1/4] Building parser EXE bundle ..."
NYASH_SKIP_REBUILD=1 NYASH_SKIP_PLUGIN_BUILD=1 bash tools/build_compiler_exe.sh >/dev/null

echo "[2/4] Preparing sample source ..."
mkdir -p dist/nyash_compiler/tmp
echo 'return 1+2*3' > dist/nyash_compiler/tmp/sample_exe_smoke.hako

echo "[3/4] Running parser EXE → JSON ..."
(cd dist/nyash_compiler && timeout -s KILL 60s ./nyash_compiler tmp/sample_exe_smoke.hako > sample.json)

if [[ "${NYASH_VALIDATE_EXE_JSON:-0}" == "1" ]]; then
  echo "[3.5/4] Validating JSON schema ..."
  python3 tools/validate_mir_json.py dist/nyash_compiler/sample.json
fi

if ! head -n1 dist/nyash_compiler/sample.json | grep -q '"kind":"Program"'; then
  echo "error: JSON does not look like a Program" >&2
  exit 2
fi

if [[ "${NYASH_EXE_BRIDGE:-0}" == "1" ]]; then
  echo "[4/4] Executing via bridge (pipe) to verify semantics ..."
  # Keep core minimal and deterministic
  export NYASH_DISABLE_PLUGINS=1
  set +e
  BIN="./target/release/hakorune"; [ -x "$BIN" ] || BIN="./target/release/hako"; [ -x "$BIN" ] || BIN="./target/release/nyash"
  timeout -s KILL 60s bash -c 'cat dist/nyash_compiler/sample.json | '"$BIN"' --ny-parser-pipe --backend vm >/dev/null'
  RC=$?
  set -e
  if [[ "$RC" -ne 7 ]]; then
    echo "error: expected exit code 7, got $RC" >&2
    exit 3
  fi
fi

echo "✅ EXE-first smoke passed (parser EXE emits Program header)"
exit 0
