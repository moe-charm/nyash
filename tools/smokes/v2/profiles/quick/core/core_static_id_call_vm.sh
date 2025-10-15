#!/bin/bash
# core_static_id_call_vm.sh — static box arg passing (Foo.id) should pass first arg

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Dev gate to avoid breaking quick until VM fix lands
if [[ "${SMOKES_ENABLE_STATIC_ARG:-}" != "1" ]]; then
  test_skip "Static-arg dev tests gated; set SMOKES_ENABLE_STATIC_ARG=1"
  exit 0
fi

TMP_DIR="/tmp/core_static_id_call_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Foo { id(x) { return x } }
static box Main { main() {
  local v = Foo.id(42)
  if (v == 42) { print("ok") } else { print("ng") }
  return 0
} }
EOF

export NYASH_JSON_STRINGIFY_DEV=1
out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | filter_noise | tail -n 1 | tr -d '\r' | xargs)
expected="ok"
compare_outputs "$expected" "$out" "core_static_id_call_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
