#!/bin/bash
# json_v0_const_ret_vm.sh — Block0 const→ret returns literal (expect 42)
# tags: core

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/json_v0_const_ret_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF2'
using selfhost.vm.mir_min as MirVmMin
static box Main { main(){
  local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":42}},{\"op\":\"ret\",\"value\":1}]}]}]}"
  local v = MirVmMin._run_min(j)
  print(MirVmMin._int_to_str(v))
  return 0 }}
EOF2

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="42"
compare_outputs "$expected" "$out" "json_v0_const_ret_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
