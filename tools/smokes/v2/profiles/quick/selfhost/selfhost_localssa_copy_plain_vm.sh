#!/bin/bash
# selfhost_localssa_copy_plain_vm.sh — LocalSSA.ensure_cond inserts a plain JSON copy object

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

# Dev gate: enable explicitly to run this diagnostic smoke
if [ "${NYASH_LOCALSSA_ENABLE:-0}" != "1" ]; then
  test_skip "selfhost_localssa_copy_plain_vm" "localssa dev smoke disabled (set NYASH_LOCALSSA_ENABLE=1)" || true
  exit 0
fi

TMP_DIR="/tmp/selfhost_localssa_copy_plain_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "apps/selfhost-compiler/builder/ssa/local.hako" as LocalSSA

static box Main {
  main() {
    // Pre-constructed single string (no +) to avoid plugin dependencies
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"jump\",\"target\":1}]},{\"id\":1,\"instructions\":[{\"op\":\"branch\",\"cond\":1,\"then\":2,\"else\":3}]},{\"id\":2,\"instructions\":[{\"op\":\"ret\",\"value\":1}]},{\"id\":3,\"instructions\":[{\"op\":\"ret\",\"value\":0}]}]}]}"
    local out = LocalSSA.ensure_cond(j)
    print(out)
    return 0
  }
}
EOF

 # Probe string addition support (plugins may be required in this environment)
ADDP=$(run_nyash_vm -c 'static box Main { main() { print("a" + "b") return 0 } }' --dev 2>&1 | tail -n 1 | tr -d '\r' | xargs)
if echo "$ADDP" | grep -qi 'unsupported binop Add'; then
  test_skip "selfhost_localssa_copy_plain_vm" "String ops unavailable (plugins missing); skipping" || true
  rm -rf "$TMP_DIR"
  exit 0
fi

OUT=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)

# Expect a plain JSON object for copy at some position (not an escaped string)
if echo "$OUT" | grep -q '{"op":"copy"'; then
  log_success "selfhost_localssa_copy_plain_vm"
  rm -rf "$TMP_DIR"
  exit 0
fi

log_error "selfhost_localssa_copy_plain_vm: plain copy object not found"
rm -rf "$TMP_DIR"
exit 1
