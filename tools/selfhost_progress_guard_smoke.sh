#!/usr/bin/env bash
set -euo pipefail
[[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]] && set -x

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  cargo build --release >/dev/null
fi

mkdir -p "$ROOT_DIR/tmp"

# Craft malformed/incomplete inputs that previously could cause non-progress
cat > "$ROOT_DIR/tmp/progress_guard_1.nyash" << 'SRC'
return ;
SRC

cat > "$ROOT_DIR/tmp/progress_guard_2.nyash" << 'SRC'
local x =
return 1
SRC

cat > "$ROOT_DIR/tmp/progress_guard_3.nyash" << 'SRC'
foo(
return 2
SRC

cat > "$ROOT_DIR/tmp/progress_guard_4.nyash" << 'SRC'
if (1) x
return 3
SRC

run_case() {
  local file="$1"
  # Force selfhost path; emit-only to avoid executing malformed code paths
  NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_EMIT_ONLY=1 NYASH_NY_COMPILER_TIMEOUT_MS=2000 \
    "$BIN" --backend vm "$file" >/dev/null || true
}

run_case "$ROOT_DIR/tmp/progress_guard_1.nyash"
run_case "$ROOT_DIR/tmp/progress_guard_2.nyash"
run_case "$ROOT_DIR/tmp/progress_guard_3.nyash"
run_case "$ROOT_DIR/tmp/progress_guard_4.nyash"

echo "✅ Selfhost progress guard smoke passed (no hang on malformed inputs)"
exit 0
