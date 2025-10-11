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
  local path = "'""$tmp""'"
  f.open(path, "w")
  f.write("hello")
  local data = f.read()
  print("" + data)
  return 0
}}
'
  out=$(NYASH_DEBUG_PLUGIN=1 run_nyash_vm -c "$code" )
  # Expect one line: hello
  last=$(echo "$out" | grep -v '^\[' | grep -v '^$' | grep -v '^Result:' | tail -n 1 | tr -d '\n')
  if [ "$last" != "hello" ]; then { test_fail "filebox read check failed (got '$last')"; return 1; }; fi
  test_pass "filebox_write_read_vm"
}

run_test "filebox_write_read_vm" test_filebox_write_read_vm
