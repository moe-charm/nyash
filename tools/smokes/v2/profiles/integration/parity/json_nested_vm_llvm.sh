#!/bin/bash
# json_nested_vm_llvm.sh — VM vs LLVM parity for nested JSON samples

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# LLVM availability check
# Harness-first: rely on run_nyash_llvm() to decide availability

TEST_DIR="/tmp/json_parity_nested_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > nyash.toml << EOF
[using.json_native]
path = "$NYASH_ROOT/apps/lib/json_native/"
main = "parser/parser.nyash"

[using.aliases]
json = "json_native"
EOF

cat > driver.nyash << 'EOF'
using json as JsonParserModule

static box Main {
  main() {
    local samples = new ArrayBox()
    samples.push("[1,[2,3],{\"x\":[4]}]")
    samples.push("{\"a\":{\"b\":[1,2]},\"c\":\"d\"}")
    samples.push("{\"n\":-1e-3,\"z\":0.0}")

    local i = 0
    loop(i < samples.length()) {
      local s = samples.get(i)
      local p = JsonParserModule.create_parser()
      local r = p.parse(s)
      if (r == null) { print("null"); } else { print(r.toString()) }
      i = i + 1
    }
    return 0
  }
}
EOF

# Run both backends under dev defaults
output_vm=$(run_nyash_vm driver.nyash --dev)
NYASH_LLVM_USE_HARNESS=1 output_llvm=$(run_nyash_llvm driver.nyash --dev)

compare_outputs "$output_vm" "$output_llvm" "json_nested_vm_llvm_parity" || exit 1

cd /
rm -rf "$TEST_DIR"
