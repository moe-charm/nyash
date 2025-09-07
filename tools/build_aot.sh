#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT_DIR"

APP=${1:-apps/tests/mir-branch-ret/main.nyash}
OUT=${2:-app_aot}
OBJ_DIR=${OBJ_DIR:-target/aot_objects}
OBJ_BASENAME=$(basename "$APP" .nyash)
OBJ_PATH="$OBJ_DIR/$OBJ_BASENAME.o"

echo "[1/5] build nyash (cranelift-jit)"
cargo build --release --features cranelift-jit

echo "[2/5] build nyrt (static lib)"
cargo build -p nyrt --release

echo "[3/5] emit object (.o) via jit-direct"
mkdir -p "$OBJ_DIR"
env -u NYASH_OPT_DIAG_FORBID_LEGACY NYASH_SKIP_TOML_ENV=1 NYASH_PLUGIN_ONLY=1 NYASH_AOT_OBJECT_OUT="$OBJ_DIR" ./target/release/nyash --jit-direct "$APP"

if [[ ! -f "$OBJ_PATH" ]]; then
  echo "❌ object not found: $OBJ_PATH" >&2
  echo "Contents of $OBJ_DIR:" >&2
  ls -la "$OBJ_DIR" >&2 || true
  exit 1
fi
ls -l "$OBJ_PATH"

echo "[4/5] link with nyrt -> $OUT"
cc "$OBJ_PATH" \
  -L crates/nyrt/target/release \
  -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive \
  -lpthread -ldl -lm -o "$OUT"

echo "[5/5] run $OUT"
./"$OUT"
