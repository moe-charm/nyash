#!/bin/bash
# json_query_min_vm.sh — Minimal JSON query (VM) single-case smoke

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Dev-time guards
export NYASH_DEV=1
# Allow file-using for this minimal driver include
export NYASH_ALLOW_USING_FILE=1
# Enable instance→function rewrite (ensures user-box methods are lowered to calls)
export NYASH_BUILDER_REWRITE_INSTANCE=1
# Keep tolerate-void as-is (harmless)
export NYASH_VM_TOLERATE_VOID=1

# Quick profile: enable json_query_min by default (heavy parser path)

TEST_DIR="/tmp/json_query_min_vm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > nyash.toml << EOF
[using.json_native]
path = "$NYASH_ROOT/apps/lib/json_native/"
main = "parser/parser.nyash"

[using.aliases]
json = "json_native"
EOF

# Probe heavy parser availability
probe=$(run_nyash_vm -c 'using json as JsonParserModule
static box Main { main() { local p = JsonParserModule.create_parser()  local r = p.parse("[]")  if r == null { print("null") } else { print("ok") } return 0 } }' --dev)
if [ "$probe" != "ok" ]; then
  test_skip "json_query_min_vm" "heavy parser unavailable in quick" || true
  cd /
  rm -rf "$TEST_DIR"
  exit 0
fi

cat > driver.nyash << 'EOF'
using json as JsonParserModule

static box Main {
  main() {
    local j = "{\"a\":{\"b\":[1,2,3]}}"
    local p = JsonParserModule.create_parser()
    local root = p.parse(j)
    if root == null { print("null") return 0 }
    local v = root.object_get("a").object_get("b").array_get(1)
    if v == null { print("null") } else { print(v.toString()) }
    return 0
  }
}
EOF

output=$(run_nyash_vm driver.nyash --dev)
expected=$(cat << 'TXT'
2
TXT
)

compare_outputs "$expected" "$output" "json_query_min_vm" || exit 1

cd /
rm -rf "$TEST_DIR"
