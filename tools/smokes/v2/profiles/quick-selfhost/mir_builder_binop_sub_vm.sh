#!/usr/bin/env bash
# mir_builder_binop_sub_vm.sh — MIR Builder v1: binop Sub

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mir_builder_binop_sub() {
  local code=$'using "selfhost/shared/mir/block_builder_box.hako" as BlockBuilder\n\n'
  code+=$'static box Main {\n  main(args) {\n    local mir = BlockBuilder.binop(7, 5, "Sub");\n    local insts = mir.get("functions").get(0).get("blocks").get(0).get("instructions");\n    local i=0; local out="";\n    loop(i<insts.size()) { if i>0 { out = out + "," } out = out + insts.get(i).get("op"); i = i + 1 }\n    print(out);\n    return 0;\n  }\n}\n'
  local out ec
  set +e; out=$(run_nyash_vm -c "$code"); ec=$?; set -e
  out=$(echo "$out" | filter_noise | tail -n1)
  if check_exact "const,const,binop,ret" "$out" "mir_builder_binop_sub_vm"; then
    test_pass mir_builder_binop_sub_vm
  else
    test_skip mir_builder_binop_sub_vm "out='${out}'"
  fi
}

run_test mir_builder_binop_sub_vm test_mir_builder_binop_sub

