#!/bin/bash
# using_modules_alias_hakorune_vm_min_vm.sh — Test [modules] alias hakorune.vm.mir_min via hako.toml

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_USING=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_alias_hakorune_vm_min_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
using hakorune.vm.mir_min as HakoruneVm

static box Main {
  main() {
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"ret\",\"value\":1}]}]}]}"
    local v = HakoruneVm.run_min(j)
    print("" + v)
    return 0
  }
}
