#!/usr/bin/env bash
set -euo pipefail

# Nyash self-hosting dev loop helper
# Goals:
# - Avoid repeated Rust builds; iterate on .nyash scripts only
# - One-time ensure binary exists; then run with VM (default) or MIR
# - Optional watch mode that re-runs on file changes (uses entr/inotifywait if available)

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

SCRIPT="apps/selfhost-minimal/main.nyash"
BACKEND="vm"
WATCH=0
LOAD_STD=0
VERBOSE=0
EXTRA_ARGS=()

usage() {
  cat <<EOF
Usage: tools/dev_selfhost_loop.sh [options] [script.nyash] [-- ARGS]

Options:
  --watch           Re-run on file changes (apps/**/*.nyash)
  --backend <mode>  interpreter|mir|vm (default: vm)
  --std             Load Ny std scripts from nyash.toml ([ny_plugins])
  -v, --verbose     Verbose CLI output
  -h, --help        Show this help

Examples:
  # One-off run (VM), minimal selfhost sample
  tools/dev_selfhost_loop.sh apps/selfhost-minimal/main.nyash

  # Watch mode with Ny std libs loaded
  tools/dev_selfhost_loop.sh --watch --std apps/selfhost/ny-parser-nyash/main.nyash

EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --watch) WATCH=1; shift ;;
    --backend) BACKEND="$2"; shift 2 ;;
    --std) LOAD_STD=1; shift ;;
    -v|--verbose) VERBOSE=1; shift ;;
    -h|--help) usage; exit 0 ;;
    --) shift; EXTRA_ARGS=("${@}"); break ;;
    *.nyash) SCRIPT="$1"; shift ;;
    *) EXTRA_ARGS+=("$1"); shift ;;
  esac
done

if [[ ! -f "$BIN" ]]; then
  echo "[dev] nyash binary not found; building release (one-time)..."
  (cd "$ROOT_DIR" && cargo build --release --features cranelift-jit)
fi

run_once() {
  local envs=("NYASH_DISABLE_PLUGINS=1")
  if [[ "$LOAD_STD" -eq 1 ]]; then
    envs+=("NYASH_LOAD_NY_PLUGINS=1")
  fi
  if [[ "$VERBOSE" -eq 1 ]]; then
    envs+=("NYASH_CLI_VERBOSE=1")
  fi
  echo "[dev] running: ${envs[*]} $BIN --backend $BACKEND $SCRIPT ${EXTRA_ARGS[*]}"
  (cd "$ROOT_DIR" && \
    env ${envs[@]} "$BIN" --backend "$BACKEND" "$SCRIPT" "${EXTRA_ARGS[@]}" )
}

if [[ "$WATCH" -eq 0 ]]; then
  run_once
  exit $?
fi

# Watch mode
echo "[dev] watch mode ON (backend=$BACKEND, std=$LOAD_STD)"
FILES_CMD="rg --files --glob 'apps/**/*.nyash'"

if command -v entr >/dev/null 2>&1; then
  # Use entr for reliable cross-platform watching
  echo "[dev] using 'entr' for file watching"
  # Feed a stable, sorted file list to entr; rerun on any change
  eval "$FILES_CMD" | sort | entr -rs "bash -lc '$(printf %q "$ROOT_DIR")/tools/dev_selfhost_loop.sh --backend $(printf %q "$BACKEND") $( ((LOAD_STD)) && echo --std ) $( ((VERBOSE)) && echo -v ) $(printf %q "$SCRIPT") ${EXTRA_ARGS:+-- ${EXTRA_ARGS[*]@Q}}'"
  exit $?
fi

if command -v inotifywait >/dev/null 2>&1; then
  echo "[dev] using 'inotifywait' for file watching"
  while true; do
    run_once || true
    # Block until any .nyash under apps changes
    inotifywait -qq -r -e close_write,create,delete,move "$ROOT_DIR/apps" || true
  done
  exit 0
fi

# Fallback: naive polling on mtime hash every 1s
echo "[dev] no watcher found; falling back to 1s polling"
prev_hash=""
while true; do
  cur_hash=$( (cd "$ROOT_DIR" && eval "$FILES_CMD" | xargs -r stat -c '%Y %n' 2>/dev/null | md5sum | awk '{print $1}') )
  if [[ "$cur_hash" != "$prev_hash" ]]; then
    prev_hash="$cur_hash"
    run_once || true
  fi
  sleep 1
done
