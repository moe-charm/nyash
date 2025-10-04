#!/bin/bash
# cli_entry_not_found.sh — Unknown --entry yields clear error, non-zero exit
# tags: selfhost,entry

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

JSON='{"functions":[{"name":"App.main","params":[],"blocks":[{"id":0,"instructions":[{"op":"const","dst":1,"value":{"type":"i64","value":1}},{"op":"ret","value":1}]}]}]}'

set -o pipefail
out=$(NYASH_PIPE_USE_PYVM=1 "$NYASH_BIN" --ny-parser-pipe --entry Foo.main --backend vm 2>&1 <<< "$JSON")
code=$?

if [ $code -eq 0 ]; then
  log_error "cli_entry_not_found: expected non-zero exit, got 0"
  exit 1
fi
if echo "$out" | grep -q "entry not found"; then
  log_success "cli_entry_not_found: reported entry not found"
  exit 0
else
  log_error "cli_entry_not_found: missing 'entry not found' in output"
  echo "$out" | tail -n 5 >&2
  exit 1
fi
