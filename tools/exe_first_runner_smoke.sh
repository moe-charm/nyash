#!/usr/bin/env bash
# Runner EXE-first smoke: use nyash with NYASH_USE_NY_COMPILER_EXE=1 to parse via external EXE
set -euo pipefail
[[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]] && set -x

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

echo "[1/4] Build parser EXE bundle ..."
tools/build_compiler_exe.sh >/dev/null

echo "[2/4] Prepare sample source ..."
mkdir -p tmp
echo 'return 1+2*3' > tmp/exe_first_runner_smoke.nyash

echo "[3/4] Run nyash with EXE-first parser ..."
cargo build --release >/dev/null
set +e
NYASH_USE_NY_COMPILER=1 NYASH_USE_NY_COMPILER_EXE=1 \
  ./target/release/nyash --backend vm tmp/exe_first_runner_smoke.nyash >/dev/null
RC=$?
set -e

echo "[4/4] Verify exit code ..."
if [[ "$RC" -ne 7 ]]; then
  echo "error: expected exit code 7, got $RC" >&2
  exit 3
fi

echo "✅ Runner EXE-first smoke passed"
exit 0

