#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

${ROOT_DIR}/target/release/nyash ${ROOT_DIR}/apps/ny-parser-nyash/main.nyash
