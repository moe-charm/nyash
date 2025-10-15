#!/bin/bash
# Gate: strict entry policy varies; run only when enabled
if [ "${SMOKES_ENABLE_STRICT_MAIN:-0}" != "1" ]; then
  echo "SKIP: enable with SMOKES_ENABLE_STRICT_MAIN=1" >&2
  exit 0
fi
# strict_missing_main.sh — Strict policy: App.main only → error (no implicit adoption)
# tags: selfhost,entry

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

cat > /tmp/entry_app_only_$$.nyash << 'EOF'
static box App {
  main() {
    print("APP")
    return 0
  }
}
EOF

output=$(run_nyash_vm /tmp/entry_app_only_$$.nyash)
code=$?
rm -f /tmp/entry_app_only_$$.nyash

if [ $code -eq 0 ]; then
  log_error "strict_missing_main: expected non-zero exit, got 0"
  exit 1
fi
if echo "$output" | grep -qi "missing main"; then
  log_success "strict_missing_main: failed as expected with missing main"
  exit 0
else
  log_error "strict_missing_main: expected error mentioning missing main; got: $(echo "$output" | tail -n 2)"
  exit 1
fi
