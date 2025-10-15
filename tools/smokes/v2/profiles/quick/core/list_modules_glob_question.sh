#!/bin/bash
# list_modules_glob_question.sh — Verify ? glob expands workspace members

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/list_modules_glob_question_$$"
mkdir -p "$TMP_DIR/apps/mod1"
cat > "$TMP_DIR/apps/mod1/hako_module.toml" << 'TOML'
[module]
name = "glob.qdemo"
version = "1.0.0"

[exports]
q = "qbox.hako"
TOML
cat > "$TMP_DIR/apps/mod1/qbox.hako" << 'SRC'
static box QBox { noop() { return 0 } }
SRC
cat > "$TMP_DIR/hako.toml" << 'TOML'
[modules.workspace]
members = ["apps/mod?/hako_module.toml"]
TOML

out=$(NYASH_ROOT="$TMP_DIR" NYASH_USING_TEST_FORCE_ENV_ROOT=1 "$NYASH_BIN" --list-modules 2>/dev/null)
echo "$out" | grep -q "glob\.qdemo\.q\|mod1\.qbox" && {
  log_success "list_modules_glob_question: matched [workspace:hako_module] entry"
  rm -rf "$TMP_DIR"; exit 0; }
echo "$out" | tail -n 50 >&2
log_error "list_modules_glob_question: expected workspace entry not found"
rm -rf "$TMP_DIR"; exit 1
