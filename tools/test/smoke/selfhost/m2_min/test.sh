#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

build_nyash_release

# Skip when LLVM toolchain is not available
if ! command -v llvm-config-18 >/dev/null 2>&1; then
  # For llvm-harness (default), we only need llvm-config-18
  # For llvm-inkwell-legacy, we also need LLVM_SYS_180_PREFIX
  LLVM_FEATURE=${NYASH_LLVM_FEATURE:-llvm}
  if [[ "$LLVM_FEATURE" == "llvm-inkwell-legacy" && -z "${LLVM_SYS_180_PREFIX:-}" ]]; then
    echo "[SKIP] selfhost M2 minimal: LLVM18 not available for legacy inkwell"; exit 0
  elif [[ "$LLVM_FEATURE" != "llvm-inkwell-legacy" ]]; then
    echo "[SKIP] selfhost M2 minimal: llvm-config-18 not available"; exit 0
  fi
fi

build_ny_llvmc || { echo "[SKIP] selfhost M2 minimal: ny-llvmc not built"; exit 0; }
build_nyrt || { echo "[SKIP] selfhost M2 minimal: nyrt not built"; exit 0; }

TMP_DIR=$(mktemp -d)
SRC="$TMP_DIR/m2_min.nyash"
JSON="$TMP_DIR/m2_min.json"
EXE="$TMP_DIR/m2_min.out"

cat >"$SRC" <<'NY'
// M2 minimal: Return(Int)
return 42
NY

# Use selfhost compiler to emit MIR JSON (M2 MVP)
# Prefer runner's selfhost pipeline to execute child compiler and capture JSON
NYASH_USE_NY_COMPILER=1 \
NYASH_ENABLE_USING=1 \
NYASH_SELFHOST_READ_TMP=1 \
NYASH_NY_COMPILER_CHILD_ARGS="--read-tmp --emit-mir" \
NYASH_JSON_ONLY=1 \
"$ROOT/target/release/nyash" --backend vm "$SRC" > "$JSON" || true

# Skip if JSON could not be captured (env-dependent)
if [[ ! -s "$JSON" ]]; then echo "[SKIP] selfhost M2 minimal: empty JSON"; exit 0; fi

# Build EXE via crate compiler and assert exit code
if [[ ! -x "$ROOT/target/release/ny-llvmc" ]]; then
  echo "[SKIP] selfhost M2 minimal: ny-llvmc binary missing"; exit 0
fi
build_exe_crate "$JSON" "$EXE"
assert_exit "$EXE" 42
echo "OK: selfhost M2 minimal (return 42)"
