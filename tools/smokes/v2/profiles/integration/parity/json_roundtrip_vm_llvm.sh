#!/bin/bash
# json_roundtrip_vm_llvm.sh — VM vs LLVM parity for JSON roundtrip

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# LLVM availability check
if ! "$NYASH_BIN" --version 2>/dev/null | grep -q "features.*llvm"; then
  test_skip "LLVM backend not available in this build"; exit 0
fi

TEST_DIR="/tmp/json_parity_roundtrip_$$"
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
    samples.push("null")
    samples.push("true")
    samples.push("false")
    samples.push("42")
    samples.push("\"hello\"")
    samples.push("[]")
    samples.push("{}")
    samples.push("{\"a\":1}")
    samples.push("-0")
    samples.push("0")
    samples.push("3.14")
    samples.push("-2.5")
    samples.push("6.02e23")
    samples.push("-1e-9")

    local i = 0
    loop(i < samples.length()) {
      local s = samples.get(i)
      local p = JsonParserModule.create_parser()
      local r = p.parse(s)
      if (r == null) { print("null") } else { print(r.toString()) }
      i = i + 1
    }
    return 0
  }
}
EOF

# Run both backends under dev defaults
output_vm=$(run_nyash_vm driver.nyash --dev)
NYASH_LLVM_USE_HARNESS=1 output_llvm=$(run_nyash_llvm driver.nyash --dev)

compare_outputs "$output_vm" "$output_llvm" "json_roundtrip_vm_llvm_parity" || exit 1

cd /
rm -rf "$TEST_DIR"
