#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "usage: $0 <path/to/app.nyash>" >&2
  exit 1
fi

APP="$1"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT_DIR/target/release/nyash"

echo "[cross] build VM/JIT binary"
cargo build --release --features cranelift-jit >/dev/null

run_vm() {
  echo "[cross] VM: $APP"
  "$BIN" --backend vm "$APP" | tee /tmp/ny_vm.out
}

run_aot() {
  if [ ! -x "$ROOT_DIR/tools/build_llvm.sh" ]; then
    echo "[cross] skip AOT (tools/build_llvm.sh not found)"; return 0; fi
  echo "[cross] AOT emit+link: $APP"
  pushd "$ROOT_DIR" >/dev/null
  tools/build_llvm.sh "$APP" -o /tmp/ny_app >/dev/null
  echo "[cross] EXE: /tmp/ny_app"
  /tmp/ny_app | tee /tmp/ny_exe.out || true
  popd >/dev/null
}

extract_result() {
  rg -n "^Result: " -N "$1" | sed -E 's/.*Result: //' || true
}

run_vm
run_aot

VM_RES=$(extract_result /tmp/ny_vm.out)
EXE_RES=$(extract_result /tmp/ny_exe.out)

echo "[cross] VM Result:  $VM_RES"
if [ -n "$EXE_RES" ]; then echo "[cross] EXE Result: $EXE_RES"; fi

if [ -n "$EXE_RES" ] && [ "$VM_RES" != "$EXE_RES" ]; then
  echo "[cross] mismatch between VM and EXE" >&2
  exit 2
fi
echo "[cross] OK"

