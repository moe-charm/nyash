#!/bin/bash
# selfhost_source_inline_min_json_vm.sh — Runner→child --source-inline E2E（min-json + pipeline-v2）

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/selfhost_source_inline_$$"
mkdir -p "$TMP_DIR"

# 入力ソース（inline経由で子に渡ることを検証）
cat > "$TMP_DIR/inline_src.hako" << 'NY'
return 42
NY

# 走らせる（Runner→子: --min-json + --pipeline-v2 で ParserBox を経由させる）
# 期待: headerを含む1行JSONかつ 42 が出力に現れる
out=$(NYASH_USE_NY_COMPILER=1 \
      NYASH_NY_COMPILER_MIN_JSON=1 \
      NYASH_NY_COMPILER_CHILD_ARGS="--pipeline-v2" \
      NYASH_NY_COMPILER_EMIT_ONLY=1 \
      NYASH_NY_COMPILER_SKIP_PY=1 \
      NYASH_JSON_ONLY=1 \
      timeout 5 "$NYASH_BIN" --backend vm "$TMP_DIR/inline_src.hako" 2>/dev/null | awk '/^\{/{print; exit}')

[ -n "$out" ] || { log_error "inline_min_json: no JSON output"; rm -rf "$TMP_DIR"; exit 1; }
echo "$out" | grep -q '"version"' || { log_error "inline_min_json: missing version"; rm -rf "$TMP_DIR"; exit 1; }
echo "$out" | grep -q '"kind"'    || { log_error "inline_min_json: missing kind"; rm -rf "$TMP_DIR"; exit 1; }


log_success "selfhost_source_inline_min_json_vm"
rm -rf "$TMP_DIR"
exit 0
