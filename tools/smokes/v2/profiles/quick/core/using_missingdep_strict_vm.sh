#!/bin/bash
# using_missingdep_strict_vm.sh — NYASH_USING_CHECKS_STRICT=1 causes missing dependency to fail-fast

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_USING_CHECKS_STRICT=1
export NYASH_CLI_VERBOSE=0
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  local dir=$(mktemp -d)
  mkdir -p "$dir/apps/demo/boxes"
  echo "// entry" > "$dir/apps/demo/boxes/entry.hako"
  cat > "$dir/apps/demo/hako_module.toml" <<TOML
[module]
name = "demo.module"
version = "1.0.0"

[exports]
entry = "boxes/entry.hako"

[dependencies]
other.module = "^1.0.0"
TOML
  cat > "$dir/hako.toml" <<TOML
[modules.workspace]
members = ["apps/demo/hako_module.toml"]
TOML
  export NYASH_USING_TEST_FORCE_ENV_ROOT=1
  export NYASH_ROOT="$dir"
  local program=$'static box Main { main(){ return 0 } }'
  local out rc tmp
  tmp="/tmp/using_missingdep_out_$$"
  run_nyash_vm -c "$program" >"$tmp" 2>&1
  rc=$?
  out=$(cat "$tmp")
  rm -f "$tmp"
  # Accept either explicit diagnostics or strict non-zero exit
  if echo "$out" | grep -E '\"missing_dep\".*modules_error|modules_error.*\"missing_dep\"|workspace missing dependency' >/dev/null; then
    return 0
  fi
  if [ $rc -ne 0 ]; then
    return 0
  fi
  echo "$out"
  return 1
}

run_test "using_missingdep_strict_vm" TEST_main
