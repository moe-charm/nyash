#!/bin/bash
# selfhost_emit_compare_cfg3_copy_vm.sh — Directly exercise EmitCompareBox.emit_compare_cfg3 with materialize=1

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Gate selfhost emit compare (materialize copy) check for quick unless enabled
if [ "${SMOKES_ENABLE_SELFHOST_EMIT:-0}" != "1" ]; then
  echo "SKIP: selfhost_emit_compare_cfg3_copy_vm (set SMOKES_ENABLE_SELFHOST_EMIT=1 to run)" >&2
  exit 0
fi


# Allow file-path using for pipeline boxes
export NYASH_ENABLE_USING=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_emit_compare_cfg3_copy_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "apps/selfhost-compiler/pipeline_v2/emit_compare_box.hako" as E
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // 6 > 3, materialize=1
    local j = E.emit_compare_cfg3(6, 3, "Gt", 1, 0)
    print(j)
    return MirVmMin.run(j)
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev 2>/dev/null)
json=$(echo "$out" | tr -d '\r' | awk 'match($0,/^\{.*\}$/){line=$0} END{print line}')

if ! echo "$json" | grep -q '"op":"copy"'; then
  log_error "missing materialize copy in JSON"
  echo "$json" >&2
  rm -rf "$TMP_DIR"; exit 1
fi

val=$(echo "$out" | tail -n 1 | tr -d '\r' | xargs)
compare_outputs "1" "$val" "emit_compare_cfg3_copy value" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
log_success "emit_compare_cfg3 produces copy and returns 1"
exit 0

