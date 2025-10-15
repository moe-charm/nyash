#!/bin/bash
# using_workspace_cycle_strict_fail_vm.sh — Fail-Fast on workspace cycle with NYASH_USING_CHECKS_STRICT=1

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_workspace_cycle_strict_fail_vm_$$"
mkdir -p "$TMP_DIR/apps/a" "$TMP_DIR/apps/b"
cat > "$TMP_DIR/apps/a/hako_module.toml" << 'TOML'
[module]
name = "a"
version = "1.0.0"
[exports]
foo = "x.hako"
[dependencies]
"b" = "^1.0.0"
TOML
cat > "$TMP_DIR/apps/b/hako_module.toml" << 'TOML'
[module]
name = "b"
version = "1.0.0"
[exports]
bar = "y.hako"
[dependencies]
"a" = "^1.0.0"
TOML
echo 'static box A { main(){ print("ok"); return 0 } }' > "$TMP_DIR/apps/a/x.hako"
echo 'static box B { noop(){ return 0 } }' > "$TMP_DIR/apps/b/y.hako"
cat > "$TMP_DIR/hako.toml" << 'TOML'
[modules.workspace]
members = ["apps/**/hako_module.toml"]
TOML

SRC="$TMP_DIR/main.nyash"
cat > "$SRC" << 'SRC_EOF'
using a.foo as Foo
static box Main { main(){ print("ok"); return 0 } }
SRC_EOF

set +e
NYASH_ROOT="$TMP_DIR" NYASH_USING_CHECKS_STRICT=1 out=$("$NYASH_BIN" --backend vm "$SRC" 2>&1)
rc=$?
set -e
if [ $rc -ne 0 ]; then
  log_success "using_workspace_cycle_strict_fail_vm: strict cycle caused Fail-Fast (rc=$rc)"
  rm -rf "$TMP_DIR"; exit 0
else
  echo "$out" >&2
  log_error "using_workspace_cycle_strict_fail_vm: expected non-zero exit"
  rm -rf "$TMP_DIR"; exit 1
fi

