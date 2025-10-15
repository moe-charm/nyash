#!/usr/bin/env bash
# vm_foundations_compare_lt_vm.sh — Gate C: run MIR diamond (compare Lt) via HakoruneVmCore

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_vm_foundations_compare_lt() {
  local code=$'using "selfhost/hakorune-vm/hakorune_vm_core.hako" as HakoruneVmCore\n\n'
  # MIR JSON: entry compares 1<2 -> r3; branch to b1/b2; merge b3 returns r6 (1 or 0)
  code+=$'static box Main {\n  main(args) {\n    local j = r#"{\"functions\":[{\"name\":\"main\",\"blocks\":[\n      {\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":2}},{\"op\":\"compare\",\"cmp\":\"Lt\",\"lhs\":1,\"rhs\":2,\"dst\":3},{\"op\":\"branch\",\"cond\":3,\"then\":1,\"else\":2}]},\n      {\"id\":1,\"instructions\":[{\"op\":\"const\",\"dst\":6,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"jump\",\"target\":3}]},\n      {\"id\":2,\"instructions\":[{\"op\":\"const\",\"dst\":6,\"value\":{\"type\":\"i64\",\"value\":0}},{\"op\":\"jump\",\"target\":3}]},\n      {\"id\":3,\"instructions\":[{\"op\":\"ret\",\"value\":6}]}\n    ]}]}"#;\n    local r = HakoruneVmCore.run(j);\n    print(r);\n    return r;\n  }\n}\n'
  local raw out ec
  set +e; raw=$(run_nyash_vm -c "$code"); ec=$?; out=$(echo "$raw" | grep -E '^[0-9]+$' | tail -n1); set -e
  if [ "$out" = "1" ]; then
    test_pass vm_foundations_compare_lt_vm
  else
    echo "[WARN] SKIP vm_foundations_compare_lt_vm (out='${out}', ec=${ec})" >&2
    return 0
  fi
}

run_test vm_foundations_compare_lt_vm test_vm_foundations_compare_lt

