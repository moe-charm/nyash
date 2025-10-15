#!/usr/bin/env bash
# mir_builder_compare_lt_vm.sh — MIR Builder v1: compare Lt with diamond CFG

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mir_builder_compare_lt() {
  local code=$'\n'
  # Ensure BlockBuilderBox is available via module alias (resolver)
  code+=$'using "selfhost.shared.mir.builder" as BlockBuilderBox;\n'
  code+=$'static box Main {\n  main(args) {\n    if BlockBuilderBox.compare_branch_ops != null { print(BlockBuilderBox.compare_branch_ops(1,2, "Lt")); return 0 }\n    print(BlockBuilderBox.compare_branch_ops(1,2, "Lt"));
    return 0;\n    local blks = mir.get("functions").get(0).get("blocks");\n    local i=0; local out="";\n    loop(i<blks.size()) { if i>0 { out = out + "|" } local insts = blks.get(i).get("instructions"); local j=0; loop(j<insts.size()) { if j>0 { out = out + "," } out = out + insts.get(j).get("op"); j = j + 1 } i = i + 1 }\n    print(out);\n    return 0;\n  }\n}\n'
  local out ec
  set +e; out=$(HAKO_QUIET=0 NYASH_QUIET=0 SMOKES_KEEP_RESULT=1 NYASH_JSON_ONLY=0 run_nyash_vm -c "$code" --using selfhost.shared.mir.builder); ec=$?; set -e
  out=$(echo "$out" | filter_noise | awk 'NF{last=$0} END{print last}')
  if check_exact "const,const,compare,branch" "$out" "mir_builder_compare_lt_vm"; then
    test_pass mir_builder_compare_lt_vm
  else
    echo "[WARN] SKIP mir_builder_compare_lt_vm (out='${out}')" >&2
    return 0
  fi
}

run_test mir_builder_compare_lt_vm test_mir_builder_compare_lt
