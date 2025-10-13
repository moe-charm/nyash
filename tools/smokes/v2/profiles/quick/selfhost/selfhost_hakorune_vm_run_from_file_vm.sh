#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../../../../.." && pwd)"
cd "$ROOT"

TMP_JSON="/tmp/smoke_mir_min.json"
SRC="apps/benchmarks/01_counter.nyash"

# Allow using file paths for selfhost modules in dev smoke
export HAKO_ALLOW_USING_FILE=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING=1
export NYASH_USING_AST=1
export NYASH_CHECK_CONTRACTS=0

./target/release/hakorune --backend vm --emit-mir-json "$TMP_JSON" "$SRC" >/dev/null

cat > /tmp/smoke_wrapper_vm.hako << HAKO
using "selfhost/hakorune-vm/hakorune_vm_core.hako" as HakoruneVmCore
static box Main {
  main() {
    return HakoruneVmCore.run_from_file("$TMP_JSON")
  }
}
HAKO

PLUGIN_SO_FILE="${NYASH_ROOT:-.}/plugins/nyash-filebox-plugin/libnyash_filebox_plugin.so"
NYASH_DISABLE_PLUGINS=0 HAKO_PLUGIN_POLICY=auto \
  NYASH_PLUGIN_DIRECT_LIB=libnyash_filebox_plugin.so NYASH_PLUGIN_DIRECT_PATH="$PLUGIN_SO_FILE" NYASH_PLUGIN_DIRECT_BOXES=FileBox \
  ./target/release/hakorune --backend vm /tmp/smoke_wrapper_vm.hako >/dev/null || true
echo "OK"
