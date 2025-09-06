#!/usr/bin/env bash
set -euo pipefail

# Cranelift JIT-AOT smoke: emit object via --jit-direct and link with nyrt
# Usage: tools/aot_smoke_cranelift.sh [app_path] [out_basename]

APP=${1:-apps/smokes/jit_aot_string_min.nyash}
BASE=${2:-app}

BIN=./target/release/nyash
OBJ_DIR=target/aot_objects
OBJ=$OBJ_DIR/${BASE}.o
EXE=${BASE}

mkdir -p "$OBJ_DIR"

echo "[AOT] building core (if needed)"
cargo build --release --features cranelift-jit >/dev/null 2>&1 || true

echo "[AOT] lowering: $APP -> $OBJ"
NYASH_DISABLE_PLUGINS=1 NYASH_AOT_OBJECT_OUT="$OBJ" "$BIN" --jit-direct "$APP"

# Ensure NyRT static library exists (libnyrt.a)
if [ ! -f crates/nyrt/target/release/libnyrt.a ] && [ ! -f target/release/libnyrt.a ] && [ ! -f target/release/nyrt.lib ]; then
  echo "[AOT] building NyRT (static runtime)"
  (cd crates/nyrt && cargo build --release >/dev/null)
fi

echo "[AOT] linking: $EXE"
# Prefer the workspace root target, then crates/nyrt/target
cc "$OBJ" \
  -L target/release \
  -L crates/nyrt/target/release \
  -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive \
  -lpthread -ldl -lm \
  -o "$EXE"

echo "[AOT] run: ./$EXE"
"./$EXE" || true
