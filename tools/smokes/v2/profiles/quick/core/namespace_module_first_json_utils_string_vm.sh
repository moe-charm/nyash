#!/bin/bash
# namespace_module_first_json_utils_string_vm.sh — Verify module-first resolution for selfhost.json.utils.string

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_USING=1
export NYASH_NS_POLICY=module-first
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/namespace_module_first_json_utils_string_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
using selfhost.json.utils.string as StringUtils

static box Main {
  main() {
    if StringUtils.starts_with("abcdef", "ab") { print("1") } else { print("0") }
    return 0
  }
}

EOF

out=$(run_nyash_vm "$SRC" | grep -v '^Result: ' | tail -n 1 | tr -d "\r" | xargs)
expected="1"
compare_outputs "$expected" "$out" "namespace_module_first_json_utils_string_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

