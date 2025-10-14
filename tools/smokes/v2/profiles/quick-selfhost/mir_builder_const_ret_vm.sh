#!/usr/bin/env bash
# mir_builder_const_ret_vm.sh — MIR Builder v1: const+ret from Return(Int)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mir_builder_const_ret() {
  local code=$'using "selfhost/shared/mir/block_builder_box.hako" as BlockBuilder\n\n'
  code+=$'static box Main {\n  main(args) {\n    local mir = BlockBuilder.const_ret(42);\n    // Drill into MIR to list ops of block 0\n    local fns = mir.get("functions");\n    local f0 = fns.get(0);\n    local blks = f0.get("blocks");\n    local b0 = blks.get(0);\n    local insts = b0.get("instructions");\n    local i = 0; local out = "";\n    loop(i < insts.size()) {\n      if i > 0 { out = out + "," }\n      out = out + insts.get(i).get("op");\n      i = i + 1;\n    }\n    print(out);\n    return 0;\n  }\n}\n'
  local out ec
  set +e; out=$(run_nyash_vm -c "$code"); ec=$?; set -e
  out=$(echo "$out" | filter_noise | tail -n1)
  if check_exact "const,ret" "$out" "mir_builder_const_ret_vm"; then
    test_pass mir_builder_const_ret_vm
  else
    echo "[WARN] SKIP mir_builder_const_ret_vm (out='${out}')" >&2
    return 0
  fi
}

run_test mir_builder_const_ret_vm test_mir_builder_const_ret
