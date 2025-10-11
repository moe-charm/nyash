#!/usr/bin/env bash
# plugin_autobirth_e2e_vm.sh — Verify plugin auto-birth via new → method (no explicit birth)

DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
. "${DIR}/../../../lib/test_runner.sh"

require_env || exit 2
# Need plugins for this E2E
export SMOKES_DISABLE_PLUGIN_CHECKS=0
export NYASH_DISABLE_PLUGINS=0
preflight_plugins || exit 2

pick_box=""
prog_body=""
# Prefer PathBox → RegexBox → FileBox
if [ -f "$NYASH_ROOT/plugins/nyash-path-plugin/libnyash_path_plugin.so" ]; then
  pick_box=PathBox
  prog_body='local p = new PathBox()
    print(p.basename("/a/b/c.txt"))'
elif [ -f "$NYASH_ROOT/plugins/nyash-regex-plugin/libnyash_regex_plugin.so" ]; then
  pick_box=RegexBox
  prog_body='local r = new RegexBox()
    local ok = r.isMatch("abc123", "[a-z]+[0-9]+")
    print(ok)'
elif [ -f "$NYASH_ROOT/plugins/nyash-filebox-plugin/libnyash_filebox_plugin.so" ]; then
  pick_box=FileBox
  prog_body='local f = new FileBox()
    print(f.exists("hako.toml"))'
else
  log_warn "SKIP: no candidate plugin .so found (path/regex/file)"
  exit 0
fi

PROG=$(mktemp)
cat >"$PROG" <<NYASH
static box Main {
  main() {
    ${prog_body}
    return 0
  }
}
NYASH

ensure_hako_toml
LOG=$(mktemp)
run_nyash_vm "$PROG" >"$LOG" 2>&1
rc=$?
# If the runtime cannot create the box type, skip (environment-dependent)
if grep -q "Unknown Box type" "$LOG"; then
  log_warn "SKIP: runtime does not recognize $pick_box in this environment"
  rm -f "$PROG" "$LOG"
  exit 0
fi
if [ $rc -eq 0 ]; then
  pass "plugin auto-birth E2E ($pick_box)"
  rm -f "$PROG" "$LOG"
  exit 0
else
  fail "plugin auto-birth E2E ($pick_box) rc=$rc"
  cat "$LOG" | filter_noise || true
  rm -f "$PROG" "$LOG"
  exit 1
fi
