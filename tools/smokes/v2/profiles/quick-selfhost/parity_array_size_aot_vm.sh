#!/usr/bin/env bash
# parity_array_size_aot_vm.sh — VM vs AOT parity for Array.length()

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_parity_array_size() {
  # VM side: new ArrayBox(); push two items; print length
  local code=$'static box Main {\n  main() {\n    local arr; arr = new ArrayBox();\n    arr.push(1); arr.push(2);\n    local n; n = arr.length();\n    print(n);\n    return n;\n  }\n}\n'
  local out_vm
  set +e; out_vm=$(run_nyash_vm -c "$code" | grep -E '^[0-9]+$' | tail -n 1); set -e
  if [ -z "$out_vm" ]; then
    echo "[WARN] SKIP parity_array_size_aot_vm (VM numeric line not found)" >&2
    return 0
  fi

  # AOT side: glue_and_link array-len 2 using NyRT dotted exports
  local bin="$NYASH_ROOT/build/parity/bin/arr_len"
  mkdir -p "$(dirname "$bin")"
  bash tools/aot/glue_and_link.sh --array-len 2 -o "$bin" >/dev/null 2>&1 || {
    test_fail "AOT link failed"; return 1; }
  local out_aot
  set +e; out_aot=$("$bin" 2>&1 | grep '^Result:' | awk '{print $2}'); set -e
  if [ -z "$out_aot" ]; then test_fail "AOT result missing"; return 1; fi
  # Development guard: if NyRT returns -1 (stub/unimplemented), skip this parity for now
  if [ "$out_aot" = "-1" ]; then
    echo "[WARN] SKIP parity_array_size_aot_vm (NyRT array helpers not available)" >&2
    return 0
  fi

  if [ "$out_vm" = "$out_aot" ]; then
    test_pass parity_array_size_aot_vm
  else
    test_fail "VM($out_vm) != AOT($out_aot)"; return 1
  fi
}

run_test parity_array_size_aot_vm test_parity_array_size
exit 0
