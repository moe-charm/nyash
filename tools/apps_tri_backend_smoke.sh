#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT_DIR/target/release/nyash"

default_apps=(
  "$ROOT_DIR/apps/tests/mir-branch-ret/main.nyash"
  "$ROOT_DIR/apps/tests/semantics-unified/main.nyash"
  "$ROOT_DIR/apps/tests/async-await-min/main.nyash"
  "$ROOT_DIR/apps/tests/gc-sync-stress/main.nyash"
)

if [ $# -gt 0 ]; then
  apps=("$@")
else
  apps=("${default_apps[@]}")
fi

echo "[build] nyash (release, cranelift-jit)"
cargo build --release --features cranelift-jit >/dev/null

run_case() {
  local app="$1"
  echo "\n=== $app ==="
  echo "[script] interpreter"
  timeout 15s "$BIN" "$app" >/tmp/ny_script.out || true
  echo "[vm]"
  timeout 15s "$BIN" --backend vm "$app" >/tmp/ny_vm.out || true
  echo "[jit] vm+jit-exec"
  timeout 15s "$BIN" --backend vm --jit-exec --jit-hostcall "$app" >/tmp/ny_jit.out || true
  # Summarize
  for mode in script vm jit; do
    local f="/tmp/ny_${mode}.out"
    local rc_line
    rc_line=$(rg -n "^Result: " -N "$f" || true)
    echo "  [$mode] ${rc_line:-no Result line}"
  done
}

for a in "${apps[@]}"; do
  if [ -f "$a" ]; then run_case "$a"; else echo "skip (not found): $a"; fi
done

echo "\n[done] tri-backend smoke complete"
