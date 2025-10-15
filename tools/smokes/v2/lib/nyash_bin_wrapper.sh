#!/usr/bin/env bash
set -euo pipefail

# Wrapper around NYASH_BIN_REAL that normalizes backend arguments.
# Replaces "--backend ." and "--backend=." with "--backend vm".

REAL_BIN="${NYASH_BIN_REAL:-}"
if [ -z "$REAL_BIN" ]; then
  echo "[wrapper] NYASH_BIN_REAL is not set" >&2
  exit 127
fi

args=()
expect_backend_val=0
for a in "$@"; do
  if [ "$expect_backend_val" = 1 ]; then
    v="$a"
    if [ "$v" = "." ] || [ -z "$v" ]; then v="vm"; fi
    args+=("$v")
    expect_backend_val=0
    continue
  fi
  case "$a" in
    --backend)
      args+=("--backend")
      expect_backend_val=1
      ;;
    --backend=*)
      v="${a#--backend=}"
      if [ "$v" = "." ] || [ -z "$v" ]; then v="vm"; fi
      args+=("--backend=$v")
      ;;
    *)
      args+=("$a")
      ;;
  esac
done

exec "$REAL_BIN" "${args[@]}"

