#!/bin/bash
# json_unterminated_string_vm.sh — Unterminated string should produce ERROR token (VM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/json_unterminated_string_vm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > nyash.toml << EOF
[using.json_lexer]
path = "$NYASH_ROOT/apps/lib/json_native/lexer/"
main = "tokenizer.nyash"

[using.aliases]
JsonTokenizer = "json_lexer"
EOF

cat > driver.nyash << 'EOF'
using JsonTokenizer as JsonTokenizer

static box Main {
  main() {
    // Unterminated string literal should yield an ERROR token from tokenizer
    local t = new JsonTokenizer("")
    t.set_input("\"unterminated")
    local tokens = t.tokenize()
    if t.has_errors() {
      // Print first error message payload
      local e = t.get_errors().get(0)
      print(e.get_value())
    } else {
      // Fallback: print first token type (unexpected)
      print(tokens.get(0).get_type())
    }
    return 0
  }
}
EOF

output=$(run_nyash_vm driver.nyash --dev)
output=$(echo "$output" | tail -n 1 | tr -d '\r' | xargs)

expected="Unterminated string literal"
compare_outputs "$expected" "$output" "json_unterminated_string_vm"

cd /
rm -rf "$TEST_DIR"
