#!/usr/bin/env bash
# extern_c_env_allowlist_getppid_vm.sh — Allow a non-default symbol via ENV
# SMOKES_ENV+=HAKO_FFI_ALLOW_LIST=getppid

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_extern_c_env_allowlist_getppid_vm() {
  local code=$'static box Main {\n  main() {\n    local p; p = extern_c "getppid" ();\n    if (p > 1) { print("OK"); } else { print("NG"); }\n    return p;\n  }\n}\n'
  echo "[dbg] HAKO_FFI_ALLOW_LIST=${HAKO_FFI_ALLOW_LIST:-<unset>}" >&2
  local out ec
  set +e; out=$(run_nyash_vm -c "$code"); ec=$?; set -e
  if echo "$out" | grep -q '^OK$'; then
    test_pass extern_c_env_allowlist_getppid_vm
  else
    test_fail "expected OK" "exit=$ec out=$out"
    return 1
  fi
}

run_test extern_c_env_allowlist_getppid_vm test_extern_c_env_allowlist_getppid_vm
exit 0
