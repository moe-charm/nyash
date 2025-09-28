#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

cargo build --release >/dev/null

export NYASH_ENABLE_USING=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING_AST=1
export NYASH_VM_PARSERBOX_BOOL=${NYASH_VM_PARSERBOX_BOOL:-0}

TO=${DEV_TIMEOUT_SEC:-60}
if [ "$TO" = "0" ]; then
  ./target/release/nyash --backend vm apps/dev/debug_program2_vm_if.nyash "$@"
else
  timeout "$TO" ./target/release/nyash --backend vm apps/dev/debug_program2_vm_if.nyash "$@"
fi
