#!/bin/bash
# rc-only: P4 thin adapters via EmitMirFlow (constructor+method, extern/global)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
ensure_hako_toml
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING=1
export NYASH_USING_AST=1

TMP_DIR="/tmp/selfhost_p4_calls_rc_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
using "selfhost/compiler/pipeline_v2/emit_mir_flow.hako" as Emit
using selfhost.vm.mir_min as MirVmMin
static box Main { main() {
  // Constructor(ArrayBox) with no args, then size() via method_call
  local j1 = Emit.emit_constructor("ArrayBox", new ArrayBox())
  // Materialize result id inside Emit (SSA copy), then simply run
  MirVmMin.run(j1)

  // Method call with recv=0 then arg 0: String.size("abc")
  local args = new ArrayBox()
  args.push(0)
  local j2 = Emit.emit_method_call("size", 0, args)
  MirVmMin.run(j2)

  // Global (pure) call example: env.json.stringify([]) guarded in runner
  // Keep it no-op if not available; we only check that building works
  local g = new ArrayBox()
  local j3 = Emit.emit_global_call("env.json.stringify", g)
  MirVmMin.run(j3)

  // Extern op_eq(7,7)
  local v = new ArrayBox()
  v.push(7)
  v.push(7)
  local j4 = Emit.emit_extern_call("nyrt.ops.op_eq", v)
  MirVmMin.run(j4)
  return 0
} }
NY

"$NYASH_BIN" --backend vm "$TMP_DIR/driver.nyash" >/dev/null 2> >(filter_noise 1>&2) || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"
exit 0
