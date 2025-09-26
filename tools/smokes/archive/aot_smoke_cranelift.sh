#!/usr/bin/env bash
# Archived: Cranelift AOT smoke helper (not maintained currently)
set -euo pipefail

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

echo "[AOT] linking: $EXE (requires libnyrt.a present in target/release)"
cc "$OBJ" -L target/release -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive -lpthread -ldl -lm -o "$EXE"

echo "[AOT] run: ./$EXE"
"./$EXE" || true

