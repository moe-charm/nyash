#!/bin/bash
# wasm_compile_bench_suite_wat.sh — compile apps/benchmarks/wasm/basic/*.hako to WAT

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

run_test_wasm_compile_bench_suite_wat() {
  local root="$NYASH_ROOT/apps/benchmarks/wasm/basic"
  if [ ! -d "$root" ]; then
    test_skip "wasm_compile_bench_suite_wat" "benchmarks/wasm/basic not found"
    return 0
  fi
  local files count=0 ok=0
  files=$(ls "$root"/*.hako 2>/dev/null || true)
  if [ -z "$files" ]; then
    test_skip "wasm_compile_bench_suite_wat" "no .hako under benchmarks/wasm/basic"
    return 0
  fi
  for f in $files; do
    count=$((count+1))
    local outbase="/tmp/wasm_bench_${count}"
    local outlog
    outlog=$(./target/release/nyash --compile-wasm --output "$outbase" "$f" 2>&1)
    if echo "$outlog" | grep -q "WASM backend not available"; then
      test_skip "wasm_compile_bench_suite_wat" "wasm-backend not enabled"
      return 0
    fi
    if [ ! -f "${outbase}.wat" ]; then
      test_fail "wasm_compile_bench_suite_wat" "failed for $(basename "$f")"
      return 1
    fi
    # sanity: minimal helpers exist
    if ! grep -q "func \$malloc" "${outbase}.wat"; then
      test_fail "wasm_compile_bench_suite_wat" "missing malloc for $(basename "$f")"
      return 1
    fi
    ok=$((ok+1))
  done
  test_pass "wasm_compile_bench_suite_wat ($ok/$count)"
  return 0
}

run_test "wasm_compile_bench_suite_wat" run_test_wasm_compile_bench_suite_wat
