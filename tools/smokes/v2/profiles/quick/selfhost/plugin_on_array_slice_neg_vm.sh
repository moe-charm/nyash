#!/bin/bash
# plugin_on_array_slice_neg_vm.sh — plugin-on Array slice with negative end

source "$(dirname "$0")/../../../lib/test_runner.sh"
export NYASH_DISABLE_PLUGINS=0
export SMOKES_USE_PYVM=0
export NYASH_PLUGIN_ARRAY_SLICE_HANDLE=1
# Stage-2 host exports must be enabled at build time; we pass the env here as a hint
export HAKO_EXPORT_HOST=${HAKO_EXPORT_HOST:-1}
export NYASH_NYRT_SILENT_RESULT=0
export HAKO_PLUGIN_POLICY=
export NYASH_USE_PLUGIN_BUILTINS=0
export NYASH_BUILTIN_DISABLE_ARRAY=0
require_env || exit 2
preflight_plugins || exit 2

ensure_hako_toml

tmpfile=$(mktemp /tmp/plugin_on_array_slice_neg_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local a = new ArrayBox()
    a.push(1)
    a.push(2)
    // end<0 → clamp to len (current core policy)
    local b = a.slice(0, -1)
    if b.length() != 2 { return 201 }
    return 0
  }
}
SRC

# Run without the smoke filter to preserve the 'Result:' line
NYASH_VM_USE_PY=0 NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 "$NYASH_BIN" --backend vm "$tmpfile" >/dev/null 2>&1
exit_code=$?
rm -f "$tmpfile"

if [ "$exit_code" != "0" ]; then
  echo "FAIL: expected exit 0, got $exit_code" >&2
  exit 1
fi
echo "Result: 0"
exit 0

