#!/bin/bash
# Gate: method resolution edge-case; skip under rapid bring-up
if [ "${SMOKES_ENABLE_METHOD_RESOLUTION_EDGE:-0}" != "1" ]; then
  echo "SKIP: enable with SMOKES_ENABLE_METHOD_RESOLUTION_EDGE=1" >&2
  exit 0
fi
# method_resolution_is_eof_vm.sh — Ensure class-scoped method resolution works (no cross-class leak)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/ny_is_eof_vm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > nyash.toml << EOF
[using.scanner]
path = "$NYASH_ROOT/apps/lib/json_native/lexer/"
main = "scanner.nyash"

[using.token]
path = "$NYASH_ROOT/apps/lib/json_native/lexer/"
main = "token.nyash"

[using.aliases]
JsonScanner = "scanner"
JsonToken = "token"
EOF

cat > driver.nyash << 'EOF'
using JsonScanner as JsonScanner
using JsonToken as JsonToken

static box Main {
  main() {
    // Token EOF should be true
    local t = new JsonToken("EOF", "", 0, 0)
    print(t.is_eof())

    // Scanner EOF should be false when input is non-empty
    local s = new JsonScanner("x")
    print(s.is_eof())

    // Union-like: pick token path deterministically
    local cond = true
    local obj
    if cond { obj = new JsonToken("EOF", "", 0, 0) } else { obj = new JsonScanner("") }
    print(obj.is_eof())
    return 0
  }
}
EOF

expected=$(cat << 'TXT'
true
false
true
TXT
)

output=$(run_nyash_vm driver.nyash)
compare_outputs "$expected" "$output" "method_resolution_is_eof_vm" || exit 1

cd /
rm -rf "$TEST_DIR"
