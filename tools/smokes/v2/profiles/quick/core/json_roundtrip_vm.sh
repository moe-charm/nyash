#!/bin/bash
# json_roundtrip_vm.sh - JSON parse/stringify roundtrip via AST using on VM backend

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/json_roundtrip_vm_$$"
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
    // Order aligned with expected block below
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

expected=$(cat << 'TXT'
null
true
false
42
"hello"
[]
{}
{"a":1}
0
0
3.14
-2.5
6.02e23
-1e-9
TXT
)

output=$(NYASH_USING_PROFILE=dev NYASH_USING_AST=1 run_nyash_vm driver.nyash)
compare_outputs "$expected" "$output" "json_roundtrip_vm" || exit 1

cd /
rm -rf "$TEST_DIR"
