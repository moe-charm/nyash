#!/bin/bash
# method_resolution_is_eof_vm_llvm.sh — VM vs LLVM parity for class-scoped method resolution

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# LLVM availability check
# Harness-first: rely on run_nyash_llvm() to decide availability

TEST_DIR="/tmp/ny_is_eof_parity_$$"
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
    local t = new JsonToken("EOF", "", 0, 0)
    print(t.is_eof())
    local s = new JsonScanner("x")
    print(s.is_eof())
    local obj = new JsonToken("EOF", "", 0, 0)
    print(obj.is_eof())
    return 0
  }
}
EOF

output_vm=$(run_nyash_vm driver.nyash --dev)
NYASH_LLVM_USE_HARNESS=1 output_llvm=$(run_nyash_llvm driver.nyash --dev)

compare_outputs "$output_vm" "$output_llvm" "method_resolution_is_eof_vm_llvm" || exit 1

cd /
rm -rf "$TEST_DIR"
