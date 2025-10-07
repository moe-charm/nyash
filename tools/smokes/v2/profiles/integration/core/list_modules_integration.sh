#!/bin/bash
# list_modules_integration.sh — Verify --list-modules prints modules (integration profile)
# tags: core, modules, integration

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

out=$("$NYASH_BIN" --list-modules 2>/dev/null)

if echo "$out" | grep -q "selfhost.vm.boxes.mir_vm_min"; then
  log_success "list_modules_integration: auto-discovery shows expected module"
  exit 0
else
  echo "$out" | tail -n 50 >&2
  log_error "list_modules_integration: expected module not found"
  exit 1
fi

