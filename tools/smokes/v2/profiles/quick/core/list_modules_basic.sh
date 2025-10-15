#!/bin/bash
# list_modules_basic.sh — Verify --list-modules dry-run prints auto-discovered modules
# tags: core, modules

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

# If binary is not built, skip (runner resolves NYASH_BIN automatically)
if [ ! -x "$NYASH_BIN" ]; then
  test_skip "list_modules_basic" "binary not built"
  exit 0
fi

out=$("$NYASH_BIN" --list-modules 2>/dev/null)

# Expect to see at least these path→namespace pairs in auto list
ok=0
if echo "$out" | grep -q "selfhost.vm.boxes.mir_vm_min"; then ok=$((ok+1)); fi
if echo "$out" | grep -q "hakorune.vm.boxes.inst_scan"; then ok=$((ok+1)); fi

if [ "$ok" -ge 1 ]; then
  log_success "list_modules_basic: discovered at least one expected module"
  exit 0
else
  echo "$out" | tail -n 50 >&2
  log_error "list_modules_basic: expected modules not found in --list-modules output"
  exit 1
fi

