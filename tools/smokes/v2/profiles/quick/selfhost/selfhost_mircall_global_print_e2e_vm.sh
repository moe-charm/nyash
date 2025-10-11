#!/bin/bash
# selfhost_mircall_global_print_e2e_vm.sh — E2E: mir_call Global(print) via v1→v0 adapter, then --json-file (rc-only)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
ensure_hako_toml

TMP_DIR="/tmp/selfhost_mircall_global_print_e2e_vm_$$"
mkdir -p "$TMP_DIR"

# v1 JSON: const 42 -> r1; mir_call Global("print")(r1); ret 0
cat > "$TMP_DIR/mir_v1.json" << 'V1'
{"functions":[{"name":"main","params":[],"blocks":[{"id":0,"instructions":[
  {"op":"const","dst":1,"value":{"type":"i64","value":42}},
  {"op":"mir_call","dst":2,"callee":{"type":"Global","name":"print"},"args":[1]},
  {"op":"const","dst":3,"value":{"type":"i64","value":0}},
  {"op":"ret","value":3}
]}]}]}
V1

cat > "$TMP_DIR/adapter_driver.nyash" << 'DR'
using "apps/selfhost/common/json/mir_v1_adapter.hako" as V1
static box Main { main() {
  local s = include_file("mir_v1.json")
  local j0 = V1.to_v0(s)
  print(j0)
  return 0
} }
DR

pushd "$TMP_DIR" >/dev/null
NYASH_JSON_ONLY=1 "$NYASH_BIN" --backend vm adapter_driver.nyash > mir_v0.json 2> >(filter_noise 1>&2)
rc_conv=$?
if [ $rc_conv -ne 0 ]; then popd >/dev/null; rm -rf "$TMP_DIR"; exit $rc_conv; fi

"$NYASH_BIN" --backend vm --json-file mir_v0.json >/dev/null 2> >(filter_noise 1>&2)
rc=$?
popd >/dev/null
rm -rf "$TMP_DIR"
exit $rc

