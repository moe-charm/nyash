#!/bin/bash
# selfhost_json_normalize_shapes.sh — Ensure JsonProgramBox normalizes Local/If/Loop/Return/Call shapes.

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/selfhost_json_norm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" <<'NYCODE'
using "apps/selfhost-compiler/boxes/json_program_box.hako"
using "apps/selfhost-compiler/boxes/emitter_box.hako"

static box Main {
  main() {
    local ast = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"Local\",\"name\":\"answer\",\"expr\":{\"type\":\"Call\",\"name\":\"identity\",\"args\":[]}},{\"type\":\"If\",\"cond\":{\"type\":\"Compare\",\"lhs\":{\"type\":\"Int\",\"value\":1},\"op\":\"==\",\"rhs\":{\"type\":\"Int\",\"value\":1}},\"then\":[{\"type\":\"Local\",\"name\":\"value\",\"expr\":{\"type\":\"Int\",\"value\":10}}],\"else\":[{\"type\":\"Local\",\"name\":\"value\",\"expr\":{\"type\":\"Int\",\"value\":5}}]},{\"type\":\"If\",\"cond\":{\"type\":\"Bool\",\"value\":true},\"then\":[],\"else\":[]},{\"type\":\"Loop\",\"cond\":{\"type\":\"Compare\",\"lhs\":{\"type\":\"Var\",\"name\":\"value\"},\"op\":\">\",\"rhs\":{\"type\":\"Int\",\"value\":0}},\"body\":[{\"type\":\"Expr\",\"expr\":{\"type\":\"Call\",\"name\":\"print\",\"args\":[{\"type\":\"Var\",\"name\":\"value\"}]}}]},{\"type\":\"Return\",\"expr\":{\"type\":\"Var\",\"name\":\"value\"}}]}"
    local json = EmitterBox.emit_program(ast, "[]")
    print(json)
    return 0
  }
}
NYCODE

out=$(NYASH_ALLOW_USING_FILE=1 NYASH_USING=1 NYASH_USING_AST=1 run_nyash_vm "$TMP_DIR/driver.nyash" --dev | awk '/^\{/{print; exit}')
[ -n "$out" ] || { log_error "json_norm: no JSON"; rm -rf "$TMP_DIR"; exit 1; }

echo "$out" | grep -F -q '"type":"Local","name":"answer","expr":{"type":"Call","name":"identity","args":[]}' || { log_error "json_norm: Local canonicalization failed"; rm -rf "$TMP_DIR"; exit 1; }
echo "$out" | grep -F -q '"type":"If","cond"' || { log_error "json_norm: If cond missing"; rm -rf "$TMP_DIR"; exit 1; }
echo "$out" | grep -F -q '"type":"If","cond":{"type":"Bool","value":true},"then":[],"else":[]' || { log_error "json_norm: If empty branches not preserved"; rm -rf "$TMP_DIR"; exit 1; }
echo "$out" | grep -F -q '"type":"Loop","cond"' || { log_error "json_norm: Loop cond missing"; rm -rf "$TMP_DIR"; exit 1; }
echo "$out" | grep -F -q '"type":"Call","name":"print"' || { log_error "json_norm: Call not normalized"; rm -rf "$TMP_DIR"; exit 1; }
echo "$out" | grep -F -q '"type":"Return","expr"' || { log_error "json_norm: Return expr missing"; rm -rf "$TMP_DIR"; exit 1; }

echo "$out" | grep -F -q '"meta":{"usings":[]}' || { log_error "json_norm: meta missing"; rm -rf "$TMP_DIR"; exit 1; }

log_success "selfhost_json_normalize_shapes"
rm -rf "$TMP_DIR"
exit 0
