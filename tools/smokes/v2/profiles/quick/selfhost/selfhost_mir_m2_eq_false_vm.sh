#!/bin/bash
# selfhost_mir_m2_eq_false_vm.sh — MirVmMin M2 compare(Eq) false → prints 0

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_TIMEOUT_SEC=${SMOKES_TIMEOUT_SEC:-25}
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

# Enabled: Mini‑VM compare/ret segment tightened

# Dev-time guards
export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_BUILDER_REWRITE_INSTANCE=1

# Build a tiny driver that uses MirVmMin and embeds JSON inline
TMP_DIR="/tmp/selfhost_mir_m2_eq_false_vm_$$"
mkdir -p "$TMP_DIR"
cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "apps/selfhost/vm/boxes/mir_vm_min.hako" as MirVmMin

static box Main {
  main() {
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":7}},{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":8}},{\"op\":\"compare\",\"dst\":3,\"cmp\":\"Eq\",\"lhs\":1,\"rhs\":2},{\"op\":\"ret\",\"value\":3}]}]}]}"
    local v = MirVmMin._run_min(j)
    print(MirVmMin._int_to_str(v))
    return 0
  }
}
EOF

raw_output=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev)
if [ "${SMOKES_DEV_LOG:-0}" = "1" ]; then
  echo "----- [DEV LOG] full output begin -----" >&2
  echo "$raw_output" >&2
  echo "----- [DEV LOG] full output end -----" >&2
fi
# Extract the last numeric line from output to avoid dev logs (e.g., [using/alias])
output=$(echo "$raw_output" | awk '/^[[:space:]]*-?[0-9]+[[:space:]]*$/ { val=$0 } END { gsub(/\r/,"",val); gsub(/^[[:space:]]+|[[:space:]]+$/ , "", val); print val }')
# Fallback: last non-bracketed line
if [ -z "$output" ]; then
  output=$(echo "$raw_output" | grep -v '^\[' | tail -n 1 | tr -d '\r' | xargs)
fi

expected="0"
if [ "$output" = "$expected" ]; then
  log_success "selfhost_mir_m2_eq_false_vm prints $expected"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "selfhost_mir_m2_eq_false_vm expected $expected, got: $output"
  rm -rf "$TMP_DIR"
  exit 1
fi
