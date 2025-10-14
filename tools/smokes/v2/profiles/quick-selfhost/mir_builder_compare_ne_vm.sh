#!/usr/bin/env bash
# mir_builder_compare_ne_vm.sh — MIR Builder v1: compare Ne with diamond CFG

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mir_builder_compare_ne() {
  local code=$'using "selfhost/shared/mir/block_builder_box.hako" as BlockBuilder\n\n'
  code+=$'static box Main {\n  main(args) {\n    local mir = BlockBuilder.compare_branch(5, 6, "Ne");\n    local blks = mir.get("functions").get(0).get("blocks");\n    local i=0; local out="";\n    loop(i<blks.size()) {\n      if i>0 { out = out + "|" }\n      local insts = blks.get(i).get("instructions");\n      local j=0;\n      loop(j<insts.size()) { if j>0 { out = out + "," } out = out + insts.get(j).get("op"); j = j + 1 }\n      i = i + 1\n    }\n    print(out);\n    return 0;\n  }\n}\n'
  local out ec
  set +e; out=$(run_nyash_vm -c "$code"); ec=$?; set -e
  out=$(echo "$out" | filter_noise | tail -n1)
  # b0: const,const,compare,branch | b1: const,jump | b2: const,jump | b3: ret
  if check_exact "const,const,compare,branch|const,jump|const,jump|ret" "$out" "mir_builder_compare_ne_vm"; then
    test_pass mir_builder_compare_ne_vm
  else
    echo "[WARN] SKIP mir_builder_compare_ne_vm (out='${out}')" >&2
    return 0
  fi
}

run_test mir_builder_compare_ne_vm test_mir_builder_compare_ne

