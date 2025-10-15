#!/bin/bash
# list_modules_glob_starstar.sh — Verify ** glob expands workspace members

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/list_modules_glob_starstar_$$"
mkdir -p "$TMP_DIR/apps/demo/deeper"
cat > "$TMP_DIR/apps/demo/deeper/hako_module.toml" << 'TOML'
[module]
name = "glob.demo"
version = "1.0.0"

[exports]
box = "box.hako"
TOML
cat > "$TMP_DIR/apps/demo/deeper/box.hako" << 'SRC'
static box DemoBox { hello() { print("ok"); return 0 } }
SRC
cat > "$TMP_DIR/hako.toml" << 'TOML'
[modules.workspace]
members = ["apps/**/hako_module.toml"]
TOML

out=$(NYASH_ROOT="$TMP_DIR" NYASH_USING_TEST_FORCE_ENV_ROOT=1 "$NYASH_BIN" --list-modules 2>/dev/null)
echo "$out" | grep -q "glob\.demo\.box\|demo\.deeper\.box" && {
  log_success "list_modules_glob_starstar: matched [workspace:hako_module] entry"
  rm -rf "$TMP_DIR"; exit 0; }
echo "$out" | tail -n 50 >&2
log_error "list_modules_glob_starstar: expected workspace entry not found"
rm -rf "$TMP_DIR"; exit 1
