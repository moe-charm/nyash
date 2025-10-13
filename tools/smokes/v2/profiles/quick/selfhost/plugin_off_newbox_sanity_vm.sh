#!/bin/bash
# plugin_off_newbox_sanity_vm.sh — plugins OFF sanity, ensure VM runs basic program

source "$(dirname "$0")/../../../lib/test_runner.sh"
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_PYVM=0
require_env || exit 2

tmpfile=$(mktemp /tmp/plugin_off_newbox_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main { main() { return 7 } }
SRC

out_vm=$(run_nyash_vm "$tmpfile" | awk '/^Result:/{print $0}' | head -n1 | tr -d '\r' | xargs)
rm -f "$tmpfile"

if [ "$out_vm" != "Result: 7" ]; then
  echo "FAIL: expected 'Result: 7', got '$out_vm'" >&2
  exit 1
fi

echo "$out_vm"
exit 0

