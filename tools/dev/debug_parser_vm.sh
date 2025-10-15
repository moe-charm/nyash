#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

# Build (release) if not already built
cargo build --release >/dev/null

export NYASH_USING=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING_AST=1

# Dev traces (toggle as needed)
export NYASH_RESOLVE_TRACE=${NYASH_RESOLVE_TRACE:-0}
export NYASH_LOCAL_SSA_TRACE=${NYASH_LOCAL_SSA_TRACE:-0}
export NYASH_MAT_TRACE=${NYASH_MAT_TRACE:-0}
export NYASH_VARMAP_TRACE=${NYASH_VARMAP_TRACE:-0}
export NYASH_VM_TRACE=${NYASH_VM_TRACE:-0}
export NYASH_VM_PARSERBOX_BOOL=${NYASH_VM_PARSERBOX_BOOL:-0}

./target/release/nyash --backend vm apps/dev/debug_parser_vm.nyash "$@"
