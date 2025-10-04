#!/bin/bash
# cli_entry_ok.sh — CLI --entry allows selecting App.main via pipe+PyVM
# tags: selfhost,entry

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

# Minimal MIR JSON v0 with App.main returning 7
JSON='{"functions":[{"name":"App.main","params":[],"blocks":[{"id":0,"instructions":[{"op":"const","dst":1,"value":{"type":"i64","value":7}},{"op":"ret","value":1}]}]}]}'

# Use pipe path + PyVM harness; expect exit code 7
set -o pipefail
NYASH_PIPE_USE_PYVM=1 "$NYASH_BIN" --ny-parser-pipe --entry App.main --backend vm 2>/dev/null <<< "$JSON"
code=$?

if [ $code -eq 7 ]; then
  log_success "cli_entry_ok: PyVM ran App.main and exited 7"
  exit 0
else
  log_error "cli_entry_ok: expected exit 7, got $code"
  exit 1
fi
