#!/bin/bash
# selfhost_json_normalize_edges.sh — JsonProgramBox edge-case normalization (null arrays, deep nesting)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/selfhost_json_norm_edges_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" <<'NYCODE'
using "apps/selfhost-compiler/boxes/json_program_box.hako"
using "apps/selfhost-compiler/boxes/emitter_box.hako"

static box Main {
  main() {
    local ast = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"Loop\",\"cond\":{\"type\":\"Bool\",\"value\":true},\"body\":null},{\"type\":\"Expr\",\"expr\":{\"type\":\"Call\",\"name\":\"debug\",\"args\":null}},{\"type\":\"Return\",\"expr\":{\"type\":\"Method\",\"recv\":{\"type\":\"Var\",\"name\":\"acc\"},\"method\":\"push\",\"args\":[{\"type\":\"Call\",\"name\":\"child\",\"args\":[ {\"type\":\"Null\"} ]}]}}]}"
    local json = EmitterBox.emit_program(ast, "[]")
    print(json)
    return 0
  }
}
NYCODE

out=$(NYASH_ALLOW_USING_FILE=1 NYASH_ENABLE_USING=1 NYASH_USING_AST=1 run_nyash_vm "$TMP_DIR/driver.nyash" --dev | awk '/^\{/{print; exit}')
[ -n "$out" ] || { log_error "json_norm_edges: no JSON"; rm -rf "$TMP_DIR"; exit 1; }

echo "$out" | grep -F -q '"type":"Loop","cond":{"type":"Bool","value":true},"body":[]' || { log_error "json_norm_edges: Loop body not normalized to []"; rm -rf "$TMP_DIR"; exit 1; }
echo "$out" | grep -F -q '"type":"Call","name":"debug","args":[]' || { log_error "json_norm_edges: Call args null not normalized"; rm -rf "$TMP_DIR"; exit 1; }
echo "$out" | grep -F -q '"type":"Call","name":"child","args":[{"type":"Null"}]' || { log_error "json_norm_edges: nested call/Null not preserved"; rm -rf "$TMP_DIR"; exit 1; }

echo "$out" | grep -F -q '"meta":{"usings":[]}' || { log_error "json_norm_edges: meta missing"; rm -rf "$TMP_DIR"; exit 1; }

log_success "selfhost_json_normalize_edges"
rm -rf "$TMP_DIR"
exit 0
