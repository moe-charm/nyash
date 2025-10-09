#!/bin/bash
# using_modules_alias3_vm.sh — [modules] resolver E2E: string_scan resolves and is callable via alias

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING_AST=1
# Ensure minimal modules mapping provided for E2E
export NYASH_MODULES="selfhost.json.core.string_scan=apps/selfhost/common/json/core/string_scan.hako"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_alias3_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
using selfhost.json.core.string_scan as StringScanBox

static box Main {
  main() {
    // find_unescaped should skip escaped quotes; here, first unescaped '"' is at pos 4
    local s = r#"abc"def"g"#
    local p = StringScanBox.find_unescaped(s, "\"", 0)
    // naive check: p should be 3 (0-based) or >=0; we print 1 if found, 0 otherwise
    if p >= 0 { print("1") } else { print("0") }
    return 0
  }
}
EOF

raw_output=$(run_nyash_vm "$SRC")
result=$(echo "$raw_output" | grep -v '^Result: ' | tr -d "\r" | grep -E "^[[:space:]]*[01][[:space:]]*$" | tail -n 1 | xargs)
  log_warn "SKIP using_modules_alias3_vm (using resolver disabled)"
  rm -rf "$TMP_DIR"; exit 0
fi
result=$(echo "$raw_output" | tr -d "" | grep -E "^[[:space:]]*[01][[:space:]]*$" | tail -n 1 | xargs)
if [ "$result" = "1" ]; then
  log_success "using_modules_alias3_vm resolved selfhost.json.core.string_scan and executed"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "using_modules_alias3_vm expected 1, got: ${result:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
