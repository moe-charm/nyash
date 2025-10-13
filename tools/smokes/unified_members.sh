#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

export NYASH_ENABLE_UNIFIED_MEMBERS=1
export NYASH_LLVM_USE_HARNESS=1

echo "[smoke] unified_members_basic (header-first)"
./target/release/nyash --backend llvm apps/tests/unified_members_basic.hako

echo "[smoke] unified_members_block_first (nyash-mode)"
./target/release/nyash --backend llvm apps/tests/unified_members_block_first.hako

echo "[smoke] unified_members_once_cache (once cached)"
./target/release/nyash --backend llvm apps/tests/unified_members_once_cache.hako

echo "[smoke] OK"
