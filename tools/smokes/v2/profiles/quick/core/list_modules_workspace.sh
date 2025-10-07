#!/bin/bash
# list_modules_workspace.sh — Verify --list-modules prints [workspace] entries for module.toml members
# tags: core, modules

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

if [ ! -x "$NYASH_BIN" ]; then
  test_skip "list_modules_workspace" "binary not built"
  exit 0
fi

out=$("$NYASH_BIN" --list-modules 2>/dev/null)

ok=0
if echo "$out" | grep -q "^\[workspace\] selfhost\.vm\.entry"; then ok=$((ok+1)); fi
if echo "$out" | grep -q "^\[workspace\] hakorune\.vm\.entry"; then ok=$((ok+1)); fi

if [ "$ok" -ge 1 ]; then
  log_success "list_modules_workspace: workspace entries visible"
  exit 0
else
  echo "$out" | tail -n 80 >&2
  log_error "list_modules_workspace: expected [workspace] entries not found"
  exit 1
fi
