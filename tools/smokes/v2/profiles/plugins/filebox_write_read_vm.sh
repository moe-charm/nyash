#!/bin/bash
# filebox_write_read_vm.sh — FileBox write/read basic (plugins profile)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_filebox_write_read_vm() {
  export HAKO_PLUGIN_POLICY=${HAKO_PLUGIN_POLICY:-auto}
  # Use a temp file; ensure clean
  local tmp="/tmp/nyash_plugin_filebox_wr_$$.txt"
  rm -f "$tmp" 2>/dev/null || true
  local code='
static box Main { main() {
  local f = new FileBox()
  f.open("'"$tmp"'", "w")
  local n = f.write("hello")
  f.close()
  local g = new FileBox()
  local data = g.read("'"$tmp"'")
  print("" + n)
  print("" + data)
  return 0
}}
'
  out=$(run_nyash_vm -c "$code" --dev)
  # Expect two lines: 5 and hello
  first=$(echo "$out" | head -n 1 | tr -d '
')
  second=$(echo "$out" | tail -n 1 | tr -d '
')
  if [ "$first" != "5" ]; then { test_fail "bytes written not 5 (got '$first')"; return 1; }; fi
  if [ "$second" != "hello" ]; then { test_fail "file content not 'hello' (got '$second')"; return 1; }; fi
  test_pass "filebox_write_read_vm"
}

run_test "filebox_write_read_vm" test_filebox_write_read_vm
