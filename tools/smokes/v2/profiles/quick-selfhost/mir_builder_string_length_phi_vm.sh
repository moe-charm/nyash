#!/usr/bin/env bash
# mir_builder_string_length_phi_vm.sh — ensure phi-merged string receiver hits extern length safely

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

run_mir_builder_string_length_phi() {
  local code=$'static box Main {\n  main(args) {\n    local source;\n    if (args.size() > 0) {\n      source = "ny";\n    } else {\n      source = "hako";\n    }\n    local len = source.length();\n    print("phi/string.len=" + len.toString());\n    return len;\n  }\n}\n'
  local out ec
  set +e; out=$(run_nyash_vm -c "$code"); ec=$?; set -e
  if [[ $ec -ne 0 ]]; then
    echo "mir_builder_string_length_phi_vm: VM exited with $ec" >&2
    return $ec
  fi
  out=$(echo "$out" | filter_noise | awk 'NF{last=$0} END{print last}')
  check_exact "phi/string.len=4" "$out" "mir_builder_string_length_phi_vm"
}

run_test mir_builder_string_length_phi_vm run_mir_builder_string_length_phi
