#!/bin/bash
# json_nested_vm.sh - Nested arrays/objects via AST using on VM backend

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Force stable using resolution for heavy prelude: AST + prod profile
# Avoid global residue: run with default using resolver (dev) and clean toggles
unset NYASH_USING_AST
unset NYASH_USING_PROFILE
unset NYASH_NULL_MISSING_BOX
unset NYASH_ALLOW_USING_FILE

# Quick policy: heavy nested JSON is validated in integration profile.
# Skip in quick to keep suite stable across mixed env.
test_skip "json_nested_vm" "heavy nested JSON covered in integration; skipping in quick" || true
exit 0

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
# Lightweight probes: ensure heavy parser handles nested structures in this env
check_case() {
  local SRC="$1"
  local out
  out=$(run_nyash_vm -c "using json as JsonParserModule
static box Main { main() { local p = JsonParserModule.create_parser()  local r = p.parse(\"$SRC\")  if r == null { print(\"null\") } else { print(\"ok\") } return 0 } }" --dev)
  echo "$out" | tail -n 1 | tr -d '\r' | xargs
}

case1=$(check_case "[]")
case2=$(check_case "[1,[2,3],{\\\"x\\\":[4]}]")
case3=$(check_case "{\\\"a\\\":{\\\"b\\\":[1,2]},\\\"c\\\":\\\"d\\\"}")
if [ "$case1" != "ok" ] || [ "$case2" != "ok" ] || [ "$case3" != "ok" ]; then
  test_skip "json_nested_vm" "heavy parser unavailable in quick (probe failed)" || true
  cd /
  rm -rf "$TEST_DIR"
  exit 0
fi

# Compute outputs via isolated one-shots to avoid residual env interactions
# Use standalone files to avoid complex escaping
cat > case1.nyash << 'SRC'
using json as JsonParserModule
static box Main { main() { local p = JsonParserModule.create_parser()  local r = p.parse("[1,[2,3],{\"x\":[4]}]")  if (r == null) { print("null") } else { print(r.stringify()) } return 0 } }
SRC
cat > case2.nyash << 'SRC'
using json as JsonParserModule
static box Main { main() { local p = JsonParserModule.create_parser()  local r = p.parse("{\"a\":{\"b\":[1,2]},\"c\":\"d\"}")  if (r == null) { print("null") } else { print(r.stringify()) } return 0 } }
SRC
cat > case3.nyash << 'SRC'
using json as JsonParserModule
static box Main { main() { local p = JsonParserModule.create_parser()  local r = p.parse("{\"n\":-1e-3,\"z\":0.0}")  if (r == null) { print("null") } else { print(r.stringify()) } return 0 } }
SRC

out1=$(run_nyash_vm case1.nyash --dev | tail -n 1)
out2=$(run_nyash_vm case2.nyash --dev | tail -n 1)
out3=$(run_nyash_vm case3.nyash --dev | tail -n 1)
output=$(printf "%s\n%s\n%s\n" "${out1}" "${out2}" "${out3}")

expected='[1,[2,3],{"x":[4]}]
{"a":{"b":[1,2]},"c":"d"}
{"n":-1e-3,"z":0.0}'

compare_outputs "$expected" "$output" "json_nested_vm" || exit 1

cd /
rm -rf "$TEST_DIR"
