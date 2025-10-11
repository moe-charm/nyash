#!/bin/bash
# plugin_on_min_rc_vm.sh — quick 常時: plugin-on 代表（rc-only）

source "$(dirname "$0")/../../../lib/test_runner.sh"

# Force plugin-on env overlay for this single test
export SMOKES_PROFILE_ENV=plugin-on
export NYASH_DISABLE_PLUGINS=0
export SMOKES_DISABLE_PLUGIN_CHECKS=0

require_env || exit 2
preflight_plugins || exit 2
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

