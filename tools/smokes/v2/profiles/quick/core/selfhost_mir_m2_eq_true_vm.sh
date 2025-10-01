#!/bin/bash
# selfhost_mir_m2_eq_true_vm.sh — MirVmMin M2 compare(Eq) true → prints 1
# tags: selfhost

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Enabled: Mini‑VM compare/ret segment tightened

# Dev-time guards
export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_BUILDER_REWRITE_INSTANCE=1

# Build a tiny driver that uses MirVmMin and embeds JSON inline
TMP_DIR="/tmp/selfhost_mir_m2_eq_true_vm_$$"
mkdir -p "$TMP_DIR"
cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":7}},{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":7}},{\"op\":\"compare\",\"dst\":3,\"cmp\":\"Eq\",\"lhs\":1,\"rhs\":2},{\"op\":\"ret\",\"value\":3}]}]}]}"
    local v = MirVmMin._run_min(j)
    print(MirVmMin._int_to_str(v))
    return 0
  }
}
EOF

output=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev)
output=$(echo "$output" | tail -n 1 | tr -d '\r' | xargs)

expected="1"
if [ "$output" = "$expected" ]; then
  log_success "selfhost_mir_m2_eq_true_vm prints $expected"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "selfhost_mir_m2_eq_true_vm expected $expected, got: $output"
  rm -rf "$TMP_DIR"
  exit 1
fi
