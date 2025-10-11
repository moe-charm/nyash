#!/bin/bash
# wasm_compile_array_ops_wat.sh — compile to WAT and check ArrayBox ops are present

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

run_test_wasm_compile_array_ops_wat() {
  local tmpdir
  tmpdir=$(mktemp -d)
  local src="$tmpdir/app.hako"
  local outbase="$tmpdir/app_out"
  cat >"$src" <<'HK'
static box Main {
  main(){
    local a = new ArrayBox();
    a.push(1);
    a.push(2);
    print(a.size());
    return 0;
  }
}
HK
  # compile to WAT
  local outlog
  outlog=$(./target/release/nyash --compile-wasm --output "$outbase" "$src" 2>&1)
  if echo "$outlog" | grep -q "WASM backend not available"; then
    test_skip "wasm_compile_array_ops_wat" "wasm-backend feature not enabled"
    rm -rf "$tmpdir"
    return 0
  fi
  if ! echo "$outlog" | grep -q "WASM compilation successful"; then
    test_fail "wasm_compile_array_ops_wat" "compile failed"
    rm -rf "$tmpdir"
    return 1
  fi
  local wat="$outbase.wat"
  if [ ! -f "$wat" ]; then
    test_fail "wasm_compile_array_ops_wat" "no WAT output"
    rm -rf "$tmpdir"
    return 1
  fi
  # basic sanity: helpers appear
  if ! grep -q "array_len" "$wat"; then
    test_fail "wasm_compile_array_ops_wat" "missing array_len in WAT"
    rm -rf "$tmpdir"
    return 1
  fi
  if ! grep -q "array_push" "$wat"; then
    test_fail "wasm_compile_array_ops_wat" "missing array_push in WAT"
    rm -rf "$tmpdir"
    return 1
  fi
  if ! grep -q "array_get\|array_set" "$wat"; then
    test_fail "wasm_compile_array_ops_wat" "missing array_get/set in WAT"
    rm -rf "$tmpdir"
    return 1
  fi
  test_pass "wasm_compile_array_ops_wat"
  rm -rf "$tmpdir"
  return 0
}

run_test "wasm_compile_array_ops_wat" run_test_wasm_compile_array_ops_wat
