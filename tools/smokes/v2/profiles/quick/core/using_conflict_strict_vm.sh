#!/bin/bash
# using_conflict_strict_vm.sh — NYASH_USING_CHECKS_STRICT=1 causes namespace conflict to fail-fast

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_USING_CHECKS_STRICT=1
export NYASH_USING_DIR_NS=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  local dir=$(mktemp -d)
  mkdir -p "$dir/apps/foo" "$dir/apps/pkg2/boxes"
  # Discovery will register apps/foo/bar.hako as foo.bar
  echo "// d1" > "$dir/apps/foo/bar.hako"
  # Override the same ns to a different path to create a conflict
  echo "// d2" > "$dir/apps/pkg2/boxes/a2.hako"
  cat > "$dir/hako.toml" <<TOML
[modules.overrides]
foo.bar = "apps/pkg2/boxes/a2.hako"
TOML
  export NYASH_USING_TEST_FORCE_ENV_ROOT=1
  export NYASH_ROOT="$dir"
  local program=$'static box Main { main(){ return 0 } }'
  local out
  out=$(run_nyash_vm -c "$program" 2>&1 | filter_noise)
  # Expect either JSON conflict or UsingError message
  echo "$out" | grep -E "modules_error.*\"conflict\"|workspace namespace conflict" >/dev/null || { echo "$out"; return 1; }
  return 0
}

run_test "using_conflict_strict_vm" TEST_main
