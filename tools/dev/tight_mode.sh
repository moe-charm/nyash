#!/usr/bin/env bash
# Tight Mode for token/time saving in local navigation
# Usage: source tools/dev/tight_mode.sh

export CODEX_NOTIFY_TAIL=${CODEX_NOTIFY_TAIL:-60}
export NYASH_CLI_VERBOSE=${NYASH_CLI_VERBOSE:-0}

# rg wrapper: limit matches, honor .ignore, search hidden only when requested
rg50(){ rg -n -m 50 --colors 'path:fg:blue' --colors 'match:fg:yellow' "$@"; }

echo "[tight] Enabled. Use rg50 and scope searches to subdirs."

