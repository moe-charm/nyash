#!/bin/bash
# using_modules_alias2_vm.sh — Verify [modules] alias resolution (second alias; SKIP if env not ready)

source "$(dirname "$0")/../../../lib/test_runner.sh"
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# If the alias is not configured in hako.toml/nyash.toml, skip gracefully.
# Expect either selfhost.vm.mir_min or selfhost.vm.boxes.mir_vm_min to be registered under an alias 'selfhost.vm.mir_min'
ALIAS_OK=0
if rg -n "\[modules\]" -n hako.toml nyash.toml 2>/dev/null | grep -q .; then
  if rg -n "selfhost\.vm\.mir_min" hako.toml nyash.toml 2>/dev/null | grep -q .; then
    ALIAS_OK=1
  fi
fi
if [ "$ALIAS_OK" = "0" ]; then
  test_skip "using_modules_alias2_vm" "no [modules] alias configured; skipping"
  exit 0
fi

TMP_DIR="/tmp/using_modules_alias2_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NYEOF'
using selfhost.vm.mir_min as VmMin

static box Main {
  main() {
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":7}},{\"op\":\"ret\",\"value\":1}]}]}]}"
    local v = VmMin._run_min(j)
    print(VmMin._int_to_str(v))
    return 0
  }
}
NYEOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
compare_outputs "7" "$out" "using_modules_alias2_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
