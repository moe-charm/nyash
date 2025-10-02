#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

cargo build --release >/dev/null

BIN="./target/release/hakorune"
if [ ! -x "$BIN" ]; then
  if [ -x "./target/release/hako" ]; then BIN="./target/release/hako"; else BIN="./target/release/nyash"; fi
fi

export NYASH_ENABLE_USING=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING_AST=1
export NYASH_VM_PARSERBOX_BOOL=${NYASH_VM_PARSERBOX_BOOL:-0}

TO=${DEV_TIMEOUT_SEC:-60}
if [ "$TO" = "0" ]; then
  "$BIN" --backend vm apps/dev/debug_program2_vm_loop.nyash "$@"
else
  timeout "$TO" "$BIN" --backend vm apps/dev/debug_program2_vm_loop.nyash "$@"
fi
