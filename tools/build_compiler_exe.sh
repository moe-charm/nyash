#!/usr/bin/env bash
set -euo pipefail

if [[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]]; then
  set -x
fi

usage() {
  cat << USAGE
Usage: tools/build_compiler_exe.sh [-o <name>] [--no-pack]

Builds the selfhost Nyash parser as a native EXE using the LLVM harness,
and stages a runnable bundle with required plugin (FileBox) and nyash.toml.

Options:
  -o <name>   Output executable name (default: nyash_compiler)
  --no-pack   Do not create dist/ bundle; only build the executable in repo root

Examples:
  tools/build_compiler_exe.sh
  tools/build_compiler_exe.sh -o nyc
USAGE
}

OUT="nyash_compiler"
PACK=1
while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage; exit 0 ;;
    -o) OUT="$2"; shift 2 ;;
    --no-pack) PACK=0; shift ;;
    *) echo "unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

if ! command -v llvm-config-18 >/dev/null 2>&1; then
  echo "error: llvm-config-18 not found (install LLVM 18 dev)." >&2
  exit 2
fi

# 1) Build nyash with LLVM harness
echo "[1/4] Building nyash (LLVM harness) ..."
_LLVMPREFIX=$(llvm-config-18 --prefix)
LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" \
  cargo build --release -j 24 --features llvm >/dev/null

# 2) Emit + link compiler.nyash → EXE
echo "[2/4] Emitting + linking selfhost compiler ..."
tools/build_llvm.sh apps/selfhost-compiler/compiler.nyash -o "$OUT"

if [[ "$PACK" == "0" ]]; then
  echo "✅ Built: ./$OUT"
  exit 0
fi

# 3) Build FileBox plugin (required when reading files)
echo "[3/4] Building FileBox plugin ..."
unset NYASH_DISABLE_PLUGINS || true
cargo build -p nyash-filebox-plugin --release >/dev/null

# 4) Stage dist/ bundle
echo "[4/4] Staging dist bundle ..."
DIST="dist/nyash_compiler"
rm -rf "$DIST"
mkdir -p "$DIST/plugins/nyash-filebox-plugin/target/release" "$DIST/tmp"
cp -f "$OUT" "$DIST/"

# Copy plugin binary (platform-specific extension). Copy entire release dir for safety.
cp -a plugins/nyash-filebox-plugin/target/release/. "$DIST/plugins/nyash-filebox-plugin/target/release/" || true

# Minimal nyash.toml for runtime (FileBox only)
cat > "$DIST/nyash.toml" << 'TOML'
[libraries]
[libraries."libnyash_filebox_plugin"]
boxes = ["FileBox"]
path = "./plugins/nyash-filebox-plugin/target/release/libnyash_filebox_plugin"

[libraries."libnyash_filebox_plugin".FileBox]
type_id = 6

[libraries."libnyash_filebox_plugin".FileBox.methods]
birth = { method_id = 0 }
open  = { method_id = 1, args = ["path", "mode"] }
read  = { method_id = 2 }
write = { method_id = 3, args = ["data"] }
close = { method_id = 4 }
fini  = { method_id = 4294967295 }
TOML

echo "✅ Done: $DIST"
echo "   Usage:"
echo "     echo 'return 1+2*3' > $DIST/tmp/sample.nyash"
echo "     (cd $DIST && ./$(basename "$OUT") tmp/sample.nyash > sample.json)"
echo "     head -n1 sample.json"

exit 0

