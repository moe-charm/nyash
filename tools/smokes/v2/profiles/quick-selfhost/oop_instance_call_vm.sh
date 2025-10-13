#!/bin/bash
# oop_instance_call_vm.sh — Instance method call should work in prod via builder rewrite

source "$(dirname "$0")/../../../lib/test_runner.sh"
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

# Force prod profile and disallow VM runtime fallback for user instance BoxCall
export NYASH_USING_PROFILE=prod
export NYASH_VM_USER_INSTANCE_BOXCALL=1
export NYASH_BUILDER_REWRITE_INSTANCE=0
export NYASH_CHECK_CONTRACTS=0

TEST_DIR="/tmp/oop_instance_call_vm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
static box Main {
  main() {
    local o = new MyBox()
    if o.value() == 7 { print("ok") } else { print("ng") }
    return 0
  }
}

box MyBox {
  value() { return 7 }
}
EOF

output=$("$NYASH_BIN" --backend vm driver.nyash --dev 2>&1 | filter_noise)
output=$(echo "$output" | tail -n 1 | tr -d '\r' | xargs)

if [ "$output" = "ok" ]; then
  log_success "oop_instance_call_vm (prod) ok"
  cd /
  rm -rf "$TEST_DIR"
  exit 0
else
  log_error "oop_instance_call_vm expected ok, got: $output"
  cd /
  rm -rf "$TEST_DIR"
  exit 1
fi

