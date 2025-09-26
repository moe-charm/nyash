#!/bin/bash
# json_nested_vm.sh - Nested arrays/objects via AST using on VM backend

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/json_nested_vm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Quick profile: enabled by default (was opt-in)

cat > nyash.toml << EOF
[using.json_native]
path = "$NYASH_ROOT/apps/lib/json_native/"
main = "parser/parser.nyash"

[using.json_node]
path = "$NYASH_ROOT/apps/lib/json_native/core/node.nyash"

[using.aliases]
json = "json_native"
JsonNode = "json_node"
EOF

# Probe heavy parser availability; skip gracefully if not ready
probe=$(run_nyash_vm -c 'using json as JsonParserModule
static box Main { main() { local p = JsonParserModule.create_parser() ; local r = p.parse("[]") ; if r == null { print("null") } else { print("ok") } return 0 } }' --dev)
if [ "$probe" != "ok" ]; then
  test_skip "json_nested_vm" "heavy parser unavailable in quick" || true
  cd /
  rm -rf "$TEST_DIR"
  exit 0
fi

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
      if (r == null) { print("null") } else { print(r.toString()) }
      i = i + 1
    }
    return 0
  }
}
EOF

expected=$(cat << 'TXT'
[1,[2,3],{"x":[4]}]
{"a":{"b":[1,2]},"c":"d"}
{"n":-1e-3,"z":0.0}
TXT
)

output=$(run_nyash_vm driver.nyash --dev)
compare_outputs "$expected" "$output" "json_nested_vm" || exit 1

cd /
rm -rf "$TEST_DIR"
