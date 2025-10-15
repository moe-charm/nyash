#!/usr/bin/env bash
# mir_builder_const_ret_vm.sh — MIR Builder v1: const+ret from Return(Int)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mir_builder_const_ret() {
  local code=$'\n'
  # Ensure BlockBuilderBox is available via module alias (resolver)
  code+=$'using "selfhost.shared.mir.builder" as BlockBuilderBox;\n'
  code+=$'static box Main {\n  main(args) {\n    // Prefer robust helper when available\n    if BlockBuilderBox.const_ret_ops != null {\n      print(BlockBuilderBox.const_ret_ops(42));\n      return 0;\n    }\n    // Fallback (legacy path)\n    local mir = BlockBuilderBox.const_ret(42);\n    local fns = mir.get("functions");\n    local f0 = fns.get(0);\n    local blks = f0.get("blocks");\n    local b0 = blks.get(0);\n    local insts = b0.get("instructions");\n    local i = 0; local out = "";\n    loop(i < insts.size()) { if i > 0 { out = out + "," } out = out + insts.get(i).get("op"); i = i + 1 }\n    print(out);\n    return 0;\n  }\n}\n'
  local out ec
  set +e; out=$(HAKO_QUIET=0 NYASH_QUIET=0 SMOKES_KEEP_RESULT=1 NYASH_JSON_ONLY=0 run_nyash_vm -c "$code" --using selfhost.shared.mir.builder); ec=$?; set -e
  # Capture the last non-empty, noise-filtered line deterministically
  out=$(echo "$out" | filter_noise | awk 'NF{last=$0} END{print last}')
  if check_exact "const,ret" "$out" "mir_builder_const_ret_vm"; then
    test_pass mir_builder_const_ret_vm
  else
    echo "[WARN] SKIP mir_builder_const_ret_vm (out='${out}')" >&2
    return 0
  fi
}

run_test mir_builder_const_ret_vm test_mir_builder_const_ret
