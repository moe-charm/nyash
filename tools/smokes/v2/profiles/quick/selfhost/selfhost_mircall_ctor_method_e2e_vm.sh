#!/bin/bash
# selfhost_mircall_ctor_method_e2e_vm.sh — E2E: mir_call {Constructor→Method} via v1→v0 adapter, then --json-file (rc-only)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
ensure_hako_toml

TMP_DIR="/tmp/selfhost_mircall_ctor_method_e2e_vm_$$"
mkdir -p "$TMP_DIR"

# v1 JSON with two mir_call ops: new ArrayBox -> r1; r1.size() -> r2; ret r2
cat > "$TMP_DIR/mir_v1.json" << 'V1'
{"functions":[{"name":"main","params":[],"blocks":[{"id":0,"instructions":[
  {"op":"mir_call","dst":1,"callee":{"type":"Constructor","box_type":"ArrayBox"},"args":[]},
  {"op":"mir_call","dst":2,"callee":{"type":"Method","method":"size","receiver":1},"args":[]},
  {"op":"ret","value":2}
]}]}]}
V1

# Ny driver to convert v1→v0 using MirJsonV1Adapter, then print v0 JSON (quiet)
cat > "$TMP_DIR/adapter_driver.nyash" << 'DR'
using "selfhost/shared/json/mir_v1_adapter.hako" as V1
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

# Execute v0 JSON via --json-file; expect ret=0 (Array.size()=0), rc-only
"$NYASH_BIN" --backend vm --json-file mir_v0.json >/dev/null 2> >(filter_noise 1>&2)
rc=$?
popd >/dev/null
rm -rf "$TMP_DIR"
exit $rc

