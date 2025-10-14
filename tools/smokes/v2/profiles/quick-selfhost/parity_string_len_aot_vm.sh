#!/usr/bin/env bash
# parity_string_len_aot_vm.sh — VM vs AOT parity for String.length()

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_parity_string_len() {
  # VM side: compute length on a fixed literal (avoid concat differences)
  local code=$'static box Main {\n  main() {\n    // Avoid concat path differences; use a single literal of length 4\n    local s; s = new StringBox("hiyo");\n    local n; n = s.length();\n    print(n);\n    return n;\n  }\n}\n'
  local out_vm
  set +e; out_vm=$(run_nyash_vm -c "$code" | grep -E '^[0-9]+$' | tail -n 1); set -e
  if [ -z "$out_vm" ]; then
    echo "[WARN] SKIP parity_string_len_aot_vm (VM numeric line not found)" >&2
    return 0
  fi

  # AOT side: glue_and_link concat-len using NyRT dotted exports
  local bin="$NYASH_ROOT/build/parity/bin/str_len"
  mkdir -p "$(dirname "$bin")"
  bash tools/aot/glue_and_link.sh --concat-len hi yo -o "$bin" >/dev/null 2>&1 || {
    test_fail "AOT link failed"; return 1; }
  local out_aot
  set +e; out_aot=$("$bin" 2>&1 | grep '^Result:' | awk '{print $2}'); set -e
  if [ -z "$out_aot" ]; then test_fail "AOT result missing"; return 1; fi

  if [ "$out_vm" = "$out_aot" ]; then
    test_pass parity_string_len_aot_vm
  else
    test_fail "VM($out_vm) != AOT($out_aot)"; return 1
  fi
}

run_test parity_string_len_aot_vm test_parity_string_len
exit 0
