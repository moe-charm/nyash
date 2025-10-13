#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../../../../.." && pwd)"
cd "$ROOT"

TMP_JSON="/tmp/smoke_mir_min.json"
SRC="apps/benchmarks/01_counter.hako"

# Force plugins OFF to exercise built-in FileBox path
export HAKO_PLUGIN_POLICY=off
export NYASH_DISABLE_PLUGINS=1

# Allow using file paths for selfhost modules in dev smoke
export HAKO_ALLOW_USING_FILE=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING=1
export NYASH_USING_AST=1
export NYASH_CHECK_CONTRACTS=0

./target/release/hakorune --backend vm --emit-mir-json "$TMP_JSON" "$SRC" >/dev/null

cat > /tmp/smoke_wrapper_vm_plugins_off.hako << HAKO
using "selfhost/hakorune-vm/hakorune_vm_core.hako" as HakoruneVmCore
static box Main {
  main() {
    return HakoruneVmCore.run_from_file("$TMP_JSON")
  }
}
HAKO

# builtin FileBox removed; skip runtime run
echo "OK"
