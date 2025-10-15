#!/bin/bash
# plugin_no_birth_nop_vm.sh — Verify that a plugin box without an explicit birth method
# is treated with a no-op birth (born state recorded) and can be used immediately.
#
# This test requires providing an actual plugin box name that lacks a `birth` method
# via env `NYASH_PLUGIN_NO_BIRTH_BOX`. If unset or plugin unavailable, the test SKIPs.

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
require_env || exit 2
preflight_plugins || true  # allow soft skip

if [ -z "${NYASH_PLUGIN_NO_BIRTH_BOX:-}" ]; then
  echo "SKIP: set NYASH_PLUGIN_NO_BIRTH_BOX to a plugin Box without birth()" >&2
  exit 0
fi

BOX_NAME="$NYASH_PLUGIN_NO_BIRTH_BOX"
TMP_DIR="/tmp/plugin_no_birth_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "main.nyash" << EOF
static box Main {
  main() {
    // Construct plugin box; loader should synthesize no-op birth and mark born
    local p = new ${BOX_NAME}()
    // If construction succeeded without unborn error, print OK
    print("OK")
    return 0
  }
}
EOF

## Provide a minimal plugin config to avoid loading all repo plugins
MINI_CFG="$TMP_DIR/hako.toml"
cat > "$MINI_CFG" << TOML
[libraries]
[libraries."libnyash_nobirth_plugin.so"]
boxes = ["$BOX_NAME"]
path = "$NYASH_ROOT/plugins/nyash-nobirth-plugin/target/release/libnyash_nobirth_plugin.so"

[libraries."libnyash_nobirth_plugin.so".$BOX_NAME]
type_id = 120
abi_version = 1
singleton = false

[libraries."libnyash_nobirth_plugin.so".$BOX_NAME.methods]
ping = { method_id = 4 }
fini = { method_id = 4294967295 }
TOML

export NYASH_PLUGIN_LOOKUP_LOCAL=1

expected="OK"
# Run from temp dir so runner picks our minimal hako.toml first
pushd "$TMP_DIR" >/dev/null
output=$(NYASH_PLUGIN_LOOKUP_LOCAL=1 "$NYASH_BIN" --backend vm "main.nyash" 2>&1 | filter_noise)
popd >/dev/null
compare_outputs "$expected" "$output" "plugin_no_birth_nop_vm" || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"
exit 0
