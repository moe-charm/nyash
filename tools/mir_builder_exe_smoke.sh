#!/usr/bin/env bash
# MIR Builder EXE smoke: Parser EXE -> JSON -> MIR builder (exe) -> run
set -euo pipefail
[[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]] && set -x

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

echo "[1/5] Build parser EXE bundle ..."
tools/build_compiler_exe.sh >/dev/null

echo "[2/5] Prepare sample source ..."
mkdir -p dist/nyash_compiler/tmp
echo 'return 1+2*3' > dist/nyash_compiler/tmp/sample_builder_smoke.nyash

echo "[3/5] Run parser EXE to JSON ..."
(cd dist/nyash_compiler && ./nyash_compiler tmp/sample_builder_smoke.nyash > sample_builder.json)

if ! head -n1 dist/nyash_compiler/sample_builder.json | grep -q '"kind":"Program"'; then
  echo "error: JSON does not look like a Program" >&2
  exit 2
fi

echo "[4/5] Build EXE via MIR builder ..."
# Prefer Rust binary if available; fallback to shell wrapper
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
exit 0
