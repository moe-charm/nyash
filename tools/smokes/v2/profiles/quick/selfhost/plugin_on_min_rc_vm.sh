#!/bin/bash
# plugin_on_min_rc_vm.sh — quick 常時: plugin-on 代表（rc-only）

source "$(dirname "$0")/../../../lib/test_runner.sh"

# Force plugin-on env overlay for this single test
export SMOKES_PROFILE_ENV=plugin-on
export NYASH_DISABLE_PLUGINS=0
export SMOKES_DISABLE_PLUGIN_CHECKS=0

require_env || exit 2
preflight_plugins || exit 2
precheck_src=$(mktemp /tmp/plugin_on_pre_XXXX.hako)
cat >"$precheck_src" << 'SRC'
static box Main { main() { local m = new MapBox(); return 0 } }
SRC
run_nyash_vm "$precheck_src" >/dev/null
pre_rc=$?
rm -f "$precheck_src"
if [ $pre_rc -ne 0 ]; then
  echo "SKIP: plugins not available (precheck rc=$pre_rc)" >&2
  exit 0
fi
ensure_hako_toml

tmpfile=$(mktemp /tmp/plugin_on_min_rc_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main { main() { return 0 } }
SRC

# rc-only: just run and return; output is filtered by test runner
"$NYASH_BIN" --backend vm "$tmpfile" >/dev/null 2> >(filter_noise 1>&2)
rc=$?
rm -f "$tmpfile"
exit $rc

