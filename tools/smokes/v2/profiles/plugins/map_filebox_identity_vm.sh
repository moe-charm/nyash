#!/bin/bash
# map_filebox_identity_vm.sh — Plugins: Map stores FileBox host handle identity

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_map_filebox_identity_vm() {
  local tmp="/tmp/nyash_map_filebox_identity_$$.txt"
  rm -f "$tmp" 2>/dev/null || true
  local code='static box Main { main() {
    local path = "'"'"'$tmp'"'"'"
    local fb = new FileBox()
    fb.open(path, "w")
    local m = new MapBox()
    m.set("file", fb)
    local fb2 = m.get("file")
    fb2.write("xyz")
    local data = fb.read()
    if data == "xyz" { print("file-ok") } else { print("file-ng") }
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code"  | grep -v '^Result:')
  rm -f "$tmp" 2>/dev/null || true
  local last; last=$(echo "$out" | tail -n 1)
  if [[ "$last" == "file-ok" ]]; then
    return 0
  fi
  compare_outputs "file-ok" "$last" "map_filebox_identity_vm"
}

run_test "map_filebox_identity_vm" test_map_filebox_identity_vm
