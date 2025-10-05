#!/bin/bash
# Gate: forward-references 2-pass parser path; skip by default
if [ "${SMOKES_ENABLE_FORWARD_REFS:-0}" != "1" ]; then
  echo "SKIP: enable with SMOKES_ENABLE_FORWARD_REFS=1" >&2
  exit 0
fi
# forward_refs_2pass.sh - Forward references resolved via declaration indexing (2-pass)

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/forward_refs_2pass_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > main.nyash << 'EOF'
static box Main {
  main() {
    // Bare function call to a static method declared later
    print(id())

    // new to a user-defined box declared later
    local p = new Parser()
    print(p.ok())
    return 0
  }
}

static box Util {
  id() { return 7 }
}

box Parser {
  ok() { return true }
}
EOF

expected=$(cat << 'TXT'
7
true
TXT
)

output=$(run_nyash_vm main.nyash)
compare_outputs "$expected" "$output" "forward_refs_2pass" || exit 1

cd /
rm -rf "$TEST_DIR"
