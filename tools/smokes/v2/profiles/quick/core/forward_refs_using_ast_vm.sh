#!/bin/bash
# Gate: forward-references using AST path; skip by default
if [ "${SMOKES_ENABLE_FORWARD_REFS:-0}" != "1" ]; then
  echo "SKIP: enable with SMOKES_ENABLE_FORWARD_REFS=1" >&2
  exit 0
fi
# forward_refs_using_ast_vm.sh - Forward references across using (AST prelude, VM backend)

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/forward_refs_using_ast_vm_$$"
mkdir -p "$TEST_DIR/lib/u"
cd "$TEST_DIR"

# nyash.toml with a package + alias
cat > nyash.toml << EOF
[using.u]
path = "lib/u"

[using.aliases]
util = "u"
EOF

# Library package: lib/u/u.nyash
cat > lib/u/u.nyash << 'EOF'
static box Util {
  id() { return 13 }
}

box Parser {
  ok() { return true }
}
EOF

# Main uses the package alias and exercises forward refs (bare id, new Parser)
cat > main.nyash << 'EOF'
using util

static box Main {
  main() {
    print(id())            // bare call to static method in prelude
    local p = new Parser() // instance box declared in prelude
    print(p.ok())
    return 0
  }
}
EOF

expected=$(cat << 'TXT'
13
true
TXT
)

# Enable AST prelude in dev profile and run on VM (override NYASH_ROOT for local lib)
export NYASH_USING_PROFILE=dev
export NYASH_USING_AST=1
export NYASH_ROOT="$TEST_DIR"
output=$(run_nyash_vm main.nyash)
compare_outputs "$expected" "$output" "forward_refs_using_ast_vm" || exit 1

cd /
rm -rf "$TEST_DIR"
