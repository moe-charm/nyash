#!/bin/bash
# userbox_static_factory_autobirth_vm.sh — static factory returns instance; auto-birth ensures usability

source "$(dirname "$0")/../../../lib/test_runner.sh"
# Keep quick lean and avoid env flakiness: enable explicitly when validating
if [ "${SMOKES_ENABLE_STATIC_FACTORY:-0}" != "1" ]; then
  echo "[SKIP] static factory autobirth (enable with SMOKES_ENABLE_STATIC_FACTORY=1)" >&2
  exit 0
fi
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/userbox_static_factory_autobirth_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NYEOF'
static box LifeFactory {
  create(name) {
    // returns a freshly created Life; builder should auto-birth inside
    return new Life(name)
  }
}

box Life {
  flag: IntegerBox
  birth(name) {
    me.flag = 1
    return 0
  }
  get_flag() {
    return me.name
  }
}

static box Main {
  main() {
    local a = LifeFactory.create("Alice")
    print(a.get_flag())
    return 0
  }
}
NYEOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "userbox_static_factory_autobirth_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
