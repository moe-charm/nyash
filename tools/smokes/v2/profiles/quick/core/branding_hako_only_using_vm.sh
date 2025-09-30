#!/bin/bash
# branding_hako_only_using_vm.sh — Ensure hako.toml-only works for using/resolver (no nyash.toml present)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/ny_hako_only_using_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Write ONLY hako.toml (no nyash.toml)
cat > hako.toml << EOF
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
compare_outputs "$expected" "$output" "branding_hako_only_using_vm" || { cd /; rm -rf "$TEST_DIR"; exit 1; }

cd /
rm -rf "$TEST_DIR"
exit 0

