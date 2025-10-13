#!/bin/bash
# selfhost_min_json_shape_if_vm.sh — ParserBox→EmitterBoxで If ノード存在を確認

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

# Gate selfhost min-json shape check for quick unless explicitly enabled
if [ "${SMOKES_ENABLE_SELFHOST_MIN:-0}" != "1" ]; then
  echo "SKIP: selfhost_min_json_shape_if_vm (set SMOKES_ENABLE_SELFHOST_MIN=1 to run)" >&2
  exit 0
fi


TMP_DIR="/tmp/selfhost_min_json_shape_if_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
using "apps/selfhost-compiler/boxes/parser_box.hako" as ParserBoxMod
using "apps/selfhost-compiler/boxes/json_program_box.hako"
using "apps/selfhost-compiler/boxes/emitter_box.hako" as EmitterBoxMod

static box Main {
  main() {
    // 直接AST JSONを供給して EmitterBox の正規化（meta注入）を検証する
    local ast = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"If\",\"cond\":{\"type\":\"Compare\",\"op\":\"Eq\",\"lhs\":{\"type\":\"Int\",\"value\":1},\"rhs\":{\"type\":\"Int\",\"value\":1}},\"then\":{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":10}},\"else\":{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":5}}}]}"
    local p = new ParserBox()
    local json = EmitterBox.emit_program(ast, p.get_usings_json())
    print(json)
    return 0
  }
}
NY

out=$(NYASH_ALLOW_USING_FILE=1 NYASH_USING=1 run_nyash_vm "$TMP_DIR/driver.nyash" --dev | awk '/^\{/{print; exit}')
[ -n "$out" ] || { log_error "min_json_shape_if: no JSON"; rm -rf "$TMP_DIR"; exit 1; }
echo "$out" | grep -q '"If"' || { log_error "min_json_shape_if: If not found"; rm -rf "$TMP_DIR"; exit 1; }

log_success "selfhost_min_json_shape_if_vm"
rm -rf "$TMP_DIR"
exit 0
