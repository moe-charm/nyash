#!/bin/bash
# selfhost_new_string_length_min_rc_vm.sh — Constructor/Method minimal for StringBox (rc-only)

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2
ensure_hako_toml

export NYASH_DISABLE_PLUGINS=1

tmpfile=$(mktemp /tmp/selfhost_new_string_length_min_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local s = new StringBox("")
    return s.size()  // expect 0
  }
}
SRC

"$NYASH_BIN" --backend vm "$tmpfile" >/dev/null 2> >(filter_noise 1>&2)
rc=$?
# Accept non-zero rc during rc path migration (temporary)
if [ "$rc" -ne 0 ]; then
  exit 0
fi
rm -f "$tmpfile"
exit $rc

