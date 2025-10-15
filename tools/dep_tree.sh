#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ENTRY=${1:-apps/selfhost/ny-parser-nyash/main.nyash}

NYASH_DISABLE_PLUGINS=0 NYASH_CLI_VERBOSE=0 NYASH_USE_PLUGIN_BUILTINS=1 \
  "$ROOT_DIR/target/release/nyash" --backend interpreter \
  "$ROOT_DIR/selfhost/tools/dep_tree_main.hako" <<<"$ENTRY"
