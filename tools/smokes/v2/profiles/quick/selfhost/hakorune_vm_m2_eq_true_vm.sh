#!/bin/bash
# hakorune_vm_m2_eq_true_vm.sh — HakoruneVmMin M2 compare(Eq) true → prints 1
# tags: selfhost hakorune

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_TIMEOUT_SEC=${SMOKES_TIMEOUT_SEC:-12}
export NYASH_VM_MAX_INSTRUCTIONS=2000000
export NYASH_VM_MAX_BLOCK_EXEC=400000
export NYASH_OPERATOR_BOX_COMPARE_ADOPT=0
export NYASH_OPERATOR_BOX_PRELUDE=0
require_env || exit 2
preflight_plugins || exit 2

unset NYASH_DEV || true
unset NYASH_OPERATOR_BOX_ALL || true
export NYASH_ENABLE_USING=1

TMP_DIR="/tmp/hakorune_vm_m2_eq_true_vm_$$"
mkdir -p "$TMP_DIR"
cat > "$TMP_DIR/driver.nyash" << 'EOF_NY'
using hakorune.vm.mir_min as MirVmMin

static box Main {
  main() {
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":7}},{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":7}},{\"op\":\"compare\",\"dst\":3,\"cmp\":\"Eq\",\"lhs\":1,\"rhs\":2},{\"op\":\"ret\",\"value\":3}]}]}]}"
    local v = MirVmMin.run_min(j)
    print(MirVmMin._int_to_str(v))
    return 0
  }
}
EOF_NY

raw_output=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev)
if [ "${SMOKES_DEV_LOG:-0}" = "1" ]; then
  echo "----- [DEV LOG] full output begin -----" >&2
  echo "$raw_output" >&2
  echo "----- [DEV LOG] full output end -----" >&2
fi
output=$(echo "$raw_output" | awk '/^[[:space:]]*-?[0-9]+[[:space:]]*$/ { val=$0 } END { gsub(/\r/,"",val); gsub(/^[[:space:]]+|[[:space:]]+$/ , "", val); print val }')
if [ -z "$output" ]; then
  output=$(echo "$raw_output" | grep -v '^\[' | tail -n 1 | tr -d '\r' | xargs)
fi

expected="1"
if [ "$output" = "$expected" ]; then
  log_success "hakorune_vm_m2_eq_true_vm prints $expected"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "hakorune_vm_m2_eq_true_vm expected $expected, got: $output"
  rm -rf "$TMP_DIR"
  exit 1
fi
