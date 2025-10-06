#!/bin/bash
# using_modules_alias_toml_only_vm.sh — [modules] resolver E2E without env override

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_USING=1
require_env || exit 2
preflight_plugins || exit 2

# Ensure we don't provide NYASH_MODULES; rely on hako.toml only
unset NYASH_MODULES || true

TMP_DIR="/tmp/using_modules_alias_toml_only_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // Minimal inline JSON → expect int result back (0/1)
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"ret\",\"value\":1}]}]}]}"
    local v = MirVmMin._run_min(j)
    print("" + v)
    return 0
  }
}
