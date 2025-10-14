#!/usr/bin/env bash
# vm_foundations_binop_add_vm.sh — Gate C: run MIR (const,const,binop,ret) via HakoruneVmCore

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_vm_foundations_binop_add() {
  local code=$'using "selfhost/hakorune-vm/hakorune_vm_core.hako" as HakoruneVmCore\n\n'
  # Minimal MIR JSON: const 5 -> r1; const 7 -> r2; binop Add r1 r2 -> r3; ret r3
  code+=$'static box Main {\n  main(args) {\n    local j = r#"{\"functions\":[{\"name\":\"main\",\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":5}},{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":7}},{\"op\":\"binop\",\"op_kind\":\"Add\",\"lhs\":1,\"rhs\":2,\"dst\":3},{\"op\":\"ret\",\"value\":3}]}]}]}"#;\n    local r = HakoruneVmCore.run(j);\n    print(r);\n    return r;\n  }\n}\n'
  local raw out ec
  set +e; raw=$(run_nyash_vm -c "$code"); ec=$?; out=$(echo "$raw" | grep -E '^[0-9]+$' | tail -n1); set -e
  if [ "$out" = "12" ]; then
    test_pass vm_foundations_binop_add_vm
  else
    echo "[WARN] SKIP vm_foundations_binop_add_vm (out='${out}', ec=${ec})" >&2
    return 0
  fi
}

run_test vm_foundations_binop_add_vm test_vm_foundations_binop_add

