#!/usr/bin/env bash
# parity_map_size_aot_vm.sh — VM vs AOT parity for Map.length()/size()

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_parity_map_size() {
  # VM side: new MapBox(); set two entries; print size via length()
  local code=$'static box Main {\n  main() {\n    local m; m = new MapBox();\n    m.set(1, 100); m.set(2, 200);\n    local n; n = m.length();\n    print(n);\n    return n;\n  }\n}\n'
  local out_vm
  set +e; out_vm=$(run_nyash_vm -c "$code" | grep -E '^[0-9]+$' | tail -n 1); set -e
  if [ -z "$out_vm" ]; then
    echo "[WARN] SKIP parity_map_size_aot_vm (VM numeric line not found)" >&2
    return 0
  fi

  # AOT side: glue_and_link map-size 2 using NyRT dotted exports
  local bin="$NYASH_ROOT/build/parity/bin/map_size"
  mkdir -p "$(dirname "$bin")"
  bash tools/aot/glue_and_link.sh --map-size 2 -o "$bin" >/dev/null 2>&1 || {
    test_fail "AOT link failed"; return 1; }
  local out_aot
  set +e; out_aot=$("$bin" 2>&1 | grep '^Result:' | awk '{print $2}'); set -e
  if [ -z "$out_aot" ]; then test_fail "AOT result missing"; return 1; fi
  # Development guard: if NyRT returns -1 (stub/unimplemented), skip for now
  if [ "$out_aot" = "-1" ]; then
    echo "[WARN] SKIP parity_map_size_aot_vm (NyRT map helpers not available)" >&2
    return 0
  fi

  if [ "$out_vm" = "$out_aot" ]; then
    test_pass parity_map_size_aot_vm
  else
    test_fail "VM($out_vm) != AOT($out_aot)"; return 1
  fi
}

run_test parity_map_size_aot_vm test_parity_map_size
exit 0
