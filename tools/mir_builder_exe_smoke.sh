#!/usr/bin/env bash
# MIR Builder EXE smoke: Parser EXE -> JSON -> MIR builder (exe) -> run
set -euo pipefail
[[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]] && set -x

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

KERNEL="crates/hako_kernel/target/release/libhako_kernel.a"
ALT_KERNEL1="target/release/libhako_kernel.a"
ALT_KERNEL2="target/release/libnyash_kernel.a"
if [ ! -f "$KERNEL" ] && [ ! -f "$ALT_KERNEL1" ] && [ ! -f "$ALT_KERNEL2" ]; then
  echo "[SKIP] hako_kernel static runtime missing; skipping MIR builder EXE" >&2
  exit 0
fi
echo "[1/5] Build parser EXE bundle ..."
NYASH_SKIP_REBUILD=1 NYASH_SKIP_PLUGIN_BUILD=1 bash tools/build_compiler_exe.sh >/dev/null

echo "[2/5] Prepare sample source ..."
mkdir -p dist/nyash_compiler/tmp
echo 'return 1+2*3' > dist/nyash_compiler/tmp/sample_builder_smoke.hako

echo "[3/5] Run parser EXE to JSON ..."
(cd dist/nyash_compiler && ./nyash_compiler tmp/sample_builder_smoke.hako > sample_builder.json)

if ! head -n1 dist/nyash_compiler/sample_builder.json | grep -q '"kind":"Program"'; then
  echo "error: JSON does not look like a Program" >&2
  exit 2
fi

if [[ "${NYASH_MIR_BUILDER_EXE:-0}" == "1" ]]; then
  echo "[4/5] Build EXE via MIR builder ..."
  cargo build --release --features llvm >/dev/null
  if [[ -x target/release/ny_mir_builder ]]; then
    ./target/release/ny_mir_builder --in dist/nyash_compiler/sample_builder.json --emit exe -o ./__mir_builder_out
  else
    ./tools/ny_mir_builder.sh --in dist/nyash_compiler/sample_builder.json --emit exe -o ./__mir_builder_out
  fi
  echo "[5/5] Run built EXE and verify ..."
  set +e
  ./__mir_builder_out >/dev/null
  RC=$?
  set -e
  rm -f ./__mir_builder_out
  if [[ "$RC" -ne 7 ]]; then
    echo "error: expected exit code 7, got $RC" >&2
    exit 3
  fi
  echo "✅ MIR builder EXE smoke passed (parser EXE → builder EXE → run)"
else
  echo "✅ MIR builder smoke (JSON header) passed"
fi
exit 0
