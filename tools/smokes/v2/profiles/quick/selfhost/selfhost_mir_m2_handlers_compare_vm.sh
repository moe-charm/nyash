#!/bin/bash
# selfhost_mir_m2_handlers_compare_vm.sh — OpHandlersBox compare(Eq) unit smoke (box-only)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
# Load both boxes under test
export NYASH_MODULES="${NYASH_MODULES:+$NYASH_MODULES,}selfhost.vm.boxes.op_handlers=apps/selfhost/vm/boxes/op_handlers.hako,selfhost.vm.boxes.instruction_scanner=apps/selfhost/vm/boxes/instruction_scanner.hako"
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi
if [ "${SMOKES_SELFHOST_M2M3_ENABLE:-0}" != "1" ]; then test_skip "selfhost M2/M3 gated (set SMOKES_SELFHOST_M2M3_ENABLE=1)"; exit 0; fi


TMP_DIR="/tmp/selfhost_mir_m2_handlers_compare_vm_$$"
mkdir -p "$TMP_DIR"
cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.boxes.op_handlers as OpHandlersBox
using selfhost.vm.boxes.instruction_scanner as InstructionScannerBox

static box Main {
  main() {
    // Minimal instruction segment (inside the [ ... ] of blocks[0].instructions)
    local seg = "{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":7}},{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":7}},{\"op\":\"compare\",\"dst\":3,\"cmp\":\"Eq\",\"lhs\":1,\"rhs\":2},{\"op\":\"ret\",\"value\":3}"
    local regs = new MapBox()
    local p = 0
    local steps = 0
    local iter = 0
    loop(true) {
      steps = steps + 1
      if steps > 50 {
        print("DBG: step guard")
        break
      }
      local rec = InstructionScannerBox.next(seg, p)
      if rec == null {
        print("DBG: rec=null")
        break
      }
      local s = rec.get("start")
      local e = rec.get("end")
      iter = iter + 1
      local opBox = rec.get("op")
      print("DBG: s=" + (""+s) + ", e=" + (""+e) + ", iter=" + (""+iter) + ", opBox=" + opBox)
      local piece = seg.substring(s, e)
      print("DBG: piece.head=" + piece.substring(0, piece.length() < 80 ? piece.length() : 80))
      if iter == 1 || iter == 2 {
        OpHandlersBox.handle_const(piece, regs)
        print("DBG: const dst1=" + (""+regs.get("1")) + ", dst2=" + (""+regs.get("2")))
      }
      if iter == 3 {
        OpHandlersBox.handle_compare(piece, regs)
        print("DBG: cmp dst3=" + (""+regs.get("3")))
      }
      if iter == 4 {
        local rid = 3
        local v = regs.get("" + rid)
        // print numeric result
        print("" + v)
        return 0
      }
      if e <= p { p = p + 1 } else { p = e }
    }
    print("-1")
    return 0
  }
}
EOF

raw_output=$(run_nyash_vm "$TMP_DIR/driver.nyash")
if [ "${SMOKES_DEV_LOG:-0}" = "1" ]; then
  echo "----- [DEV LOG] full output begin -----" >&2
  echo "$raw_output" >&2
  echo "----- [DEV LOG] full output end -----" >&2
fi
output=$(echo "$raw_output" | awk '/^[[:space:]]*-?[0-9]+[[:space:]]*$/ { val=$0 } END { gsub(/\r/,"",val); gsub(/^[[:space:]]+|[[:space:]]+$/ , "", val); print val }')
expected="1"
if [ "$output" = "$expected" ]; then
  log_success "selfhost_mir_m2_handlers_compare_vm prints $expected (box-only)"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "selfhost_mir_m2_handlers_compare_vm expected $expected, got: $output"
  rm -rf "$TMP_DIR"
  exit 1
fi
