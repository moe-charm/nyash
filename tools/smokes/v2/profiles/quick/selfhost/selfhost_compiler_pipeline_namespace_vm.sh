#!/bin/bash
# selfhost_compiler_pipeline_namespace_vm.sh — Compiler → PipelineV2 with usings name normalization

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  # Ny source with using alias + simple call; hex encode and pass via --source-inline
  SRC='using selfhost.core.timer as Timer; return Timer.now()'
  HEX=$(echo -n "$SRC" | xxd -p -c 20000)
  out=$(run_nyash_vm apps/selfhost-compiler/compiler.hako -- --emit-mir --pipeline-v2 --source-inline "$HEX" 2>&1 | filter_noise)
  echo "$out" | grep -q 'selfhost.core.timer.now' || { echo "$out"; return 1; }
  return 0
}

run_test "selfhost_compiler_pipeline_namespace_vm" TEST_main
