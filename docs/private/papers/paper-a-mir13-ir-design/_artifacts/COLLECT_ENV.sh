#!/usr/bin/env bash
set -euo pipefail

OUT_DIR=$(cd "$(dirname "$0")" && pwd)
OUT_FILE="$OUT_DIR/ENVIRONMENT.txt"

{
  echo "== Datetime =="
  date -Iseconds || date
  echo
  echo "== OS =="
  uname -a || true
  lsb_release -a 2>/dev/null || true
  sw_vers 2>/dev/null || true
  systeminfo 2>/dev/null | head -n 30 || true
  echo
  echo "== CPU =="
  lscpu 2>/dev/null || sysctl -a 2>/dev/null | grep machdep.cpu || true
  echo
  echo "== Rust toolchain =="
  rustc --version 2>/dev/null || true
  cargo --version 2>/dev/null || true
  echo
  echo "== Git =="
  git rev-parse HEAD 2>/dev/null || true
  echo
  echo "== Cranelift/JIT features =="
  rg -n "cranelift|jit" -S ../../../../ -g '!target' 2>/dev/null || true
} > "$OUT_FILE"

echo "[DONE] Wrote $OUT_FILE"

