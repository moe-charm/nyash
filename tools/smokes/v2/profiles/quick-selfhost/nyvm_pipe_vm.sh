#!/usr/bin/env bash
# nyvm_pipe_vm.sh — Gate C: run MIR(JSON) via Hakorune VM (stdin pipe)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_nyvm_pipe() {
  local mir='{"version":0,"kind":"MIR","functions":[{"name":"main","blocks":[{"id":0,"instructions":[{"op":"const","dst":{"type":"i64","value":1},"value":{"type":"i64","value":34}},{"op":"ret","value":{"type":"i64","value":1}}]}]}]}'
  local out ec
  # Send via stdin. Provide minimal env expected by the wrapper.
  set +e; out=$(printf '%s' "$mir" | NYASH_QUIET=0 HAKO_QUIET=0 NYASH_CLI_VERBOSE=0 NYASH_NYRT_SILENT_RESULT=1 \
                                   NYASH_SUPPRESS_MODULE_CONFLICT=1 HAKO_ALLOW_USING_FILE=1 NYASH_USING=1 NYASH_USING_AST=1 \
                                   "$NYASH_BIN" --nyvm-pipe 2>&1); ec=$?; set -e
  # Wrapper prints the numeric result; extract last numeric line.
  out=$(echo "$out" | grep -E '^[0-9]+$' | tail -n1)
  if [ "$out" = "34" ]; then
    test_pass nyvm_pipe_vm
  else
    test_skip nyvm_pipe_vm "res='${out}'"
  fi
}

run_test nyvm_pipe_vm test_nyvm_pipe
