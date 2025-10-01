#!/bin/bash
# strict_flow_main_ok.sh — Strict policy accepts only Main.main by default
# tags: selfhost,entry

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

# Use VM (default). Program defines Main.main only → should run OK
cat > /tmp/entry_main_ok_$$.nyash << 'EOF'
static box Main {
  main() {
    print("OK")
    return 0
  }
}
EOF

output=$(run_nyash_vm /tmp/entry_main_ok_$$.nyash)
code=$?
rm -f /tmp/entry_main_ok_$$.nyash

if [ $code -ne 0 ]; then
  log_error "strict_flow_main_ok: expected exit 0, got $code"
  exit 1
fi
# Check print side-effect
if echo "$output" | tail -n 1 | grep -q "OK"; then
  log_success "strict_flow_main_ok: ran Main.main successfully"
  exit 0
else
  log_error "strict_flow_main_ok: missing OK print"
  exit 1
fi
