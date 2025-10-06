#!/bin/bash
# flow_using_alias_vm.sh — using/alias + Flow 呼び出し（パリティは別スモークに委ね）

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_flow_using_alias() {
  export NYASH_ENABLE_FLOW=1
  local TDIR="/tmp/flow_using_alias_vm_$$"
  mkdir -p "$TDIR/lib/flow_utils"
  cd "$TDIR"

  # nyash.toml: using に flow_utils を登録し、alias を貼る
  cat > nyash.toml << 'EOF'
[using.flow_utils]
path = "lib/flow_utils/"
main = "utils.nyash"

[using.aliases]
FU = "flow_utils"

[using]
paths = ["lib"]
EOF

  # 提供側: Flow 定義
  cat > lib/flow_utils/utils.nyash << 'EOF'
flow Utils {
  add(a, b) { return a + b }
}
EOF

  # 呼び出し側: alias 経由で Utils.add を呼ぶ
  cat > main.nyash << 'EOF'
using FU
flow Main {
  main() {
    local v
    v = Utils.add(10, 20)
    print(v)
    return 0
  }
}
EOF

  local output rc
  output=$(run_nyash_vm main.nyash 2>&1 | grep -v '^Result: ')
  compare_outputs "30" "$output" "flow_using_alias_vm"
  rc=$?
  cd /
  rm -rf "$TDIR"
  return $rc
}

run_test "flow_using_alias_vm" test_flow_using_alias

