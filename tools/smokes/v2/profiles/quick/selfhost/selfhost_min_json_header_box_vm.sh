#!/bin/bash
# selfhost_min_json_header_box_vm.sh — Ensure --min-json via HeaderEmitBox works when AST merge is enabled

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

# Requires AST prelude enabled so that using lines are merged
out=$(NYASH_DISABLE_PLUGINS=1 \
      NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING=1 NYASH_USING_AST=1 \
      run_nyash_vm "$NYASH_ROOT/apps/selfhost-compiler/compiler.hako" -- --min-json --emit-header-box | \
      awk 'match($0,/^\{/) {print; exit}')

# Expect header to contain version/kind keys
echo "$out" | grep -q '"version"' || { log_error "missing version in header (header box)"; exit 1; }
echo "$out" | grep -q '"kind"'    || { log_error "missing kind in header (header box)"; exit 1; }
echo "$out" | grep -q '"kind":"Program"' || { log_error "unexpected kind (want Program): $out"; exit 1; }
exit 0
