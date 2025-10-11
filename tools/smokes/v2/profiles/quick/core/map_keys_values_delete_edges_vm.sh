#!/bin/bash
# map_keys_values_delete_edges_vm.sh — quick: minimal keys/values/delete edges

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

run_test_map_kv_delete_edges_vm() {
  local code='static box Main { main() {
    local m = new MapBox();
    // empty
    if m.size() != 0 { return 10 }
    local ks0 = m.keys();
    local vs0 = m.values();
    if ks0.size() != 0 || vs0.size() != 0 { return 11 }
    // set two entries
    m.set("a", 1);
    m.set("b", 2);
    if m.size() != 2 { return 12 }
    // delete one (ignore return value across variants)
    m.delete("a");
    if m.size() != 1 { return 13 }
    if m.has("a") { return 14 }
    local ks = m.keys();
    local vs = m.values();
    if ks.size() != 1 || vs.size() != 1 { return 15 }
    // Do not enforce order; just check remaining is b/2 via linear scan
    local ok = 0; local i = 0;
    loop(i < ks.size()) { if ks.get(i) == "b" && vs.get(i) == 2 { ok = 1 } i = i + 1 }
    if ok != 1 { return 16 }
    return 0
  }}'
  run_nyash_vm -c "$code" >/dev/null
  local rc=$?
  if [ $rc -ne 0 ]; then echo "FAIL: rc=$rc"; return 1; fi
  return 0
}

run_test "map_keys_values_delete_edges_vm" run_test_map_kv_delete_edges_vm
