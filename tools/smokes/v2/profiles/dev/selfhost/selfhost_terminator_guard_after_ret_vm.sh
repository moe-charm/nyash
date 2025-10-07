#!/bin/bash
# selfhost_terminator_guard_after_ret_vm.sh — Ensure builder guard emits stable error after ret

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Gate to dev-only (explicit enable)
if [[ "${SMOKES_ENABLE_TERMINATOR_GUARD:-}" != "1" ]]; then
  test_skip "TerminatorGuard dev test; set SMOKES_ENABLE_TERMINATOR_GUARD=1 to enable"
  exit 0
fi

TMP_DIR="/tmp/selfhost_terminator_guard_after_ret_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.common.json.mir_builder_min as MirJsonBuilderMin
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    // add_const → ret → add_const (should be blocked with unified error)
    local b = MirJsonBuilderMin.make()
      |> MirJsonBuilderMin.start_module()
      |> MirJsonBuilderMin.start_function("main")
      |> MirJsonBuilderMin.start_block(0)
      |> MirJsonBuilderMin.add_const(1, 42)
      |> MirJsonBuilderMin.add_ret(1)
      |> MirJsonBuilderMin.add_const(2, 7)  // should be blocked
      |> MirJsonBuilderMin.end_all()
    local j = MirJsonBuilderMin.to_string(b)
    return MirVmMin.run(j)
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev 2>&1)
echo "$out" | grep -q "\[ERROR\] TerminatorGuard: emit after terminator forbidden" || { echo "$out"; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

