#!/usr/bin/env bash
# AOT smoke (Cranelift) — DRYRUN skeleton (Windows-first)
#
# Usage:
#   ./tools/aot_smoke_cranelift.sh [release|debug]
# Env:
#   CLIF_SMOKE_RUN=1        # actually execute steps (default: dry-run only)
#   CODEX_NOTIFY_TAIL=100   # for CI/logging callers (optional)
#   NYASH_LINK_VERBOSE=1    # echo link commands (when run)
#   NYASH_DISABLE_PLUGINS=1 # plugin-dependent smokes off
#   NYASH_CLIF_*            # feature toggles (see docs/tests/aot_smoke_cranelift.md)

set -euo pipefail

MODE=${1:-release}
case "$MODE" in
  release|debug) : ;;
  *) echo "Usage: $0 [release|debug]" >&2; exit 2;;
esac

RUN=${CLIF_SMOKE_RUN:-0}
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
TARGET_DIR="$ROOT_DIR/target"
OBJ_DIR="$TARGET_DIR/aot_objects"
EXE_WIN="$TARGET_DIR/app_clif.exe"
EXE_NIX="$TARGET_DIR/app_clif"

banner() { printf '\n[clif-aot-smoke] %s\n' "$*"; }
info()   { printf '[clif-aot-smoke] %s\n' "$*"; }
skip()   { printf '[clif-aot-smoke] skipping %s (enable env to run)\n' "$*"; }

banner "Cranelift AOT Smoke (mode=$MODE, dry-run=$([ "$RUN" = 1 ] && echo off || echo on))"

mkdir -p "$OBJ_DIR"
OBJ_OUT="$OBJ_DIR/core_smoke.$([ "$(uname -s)" = "Windows_NT" ] && echo obj || echo o)"
NYASH_BIN="$ROOT_DIR/target/$MODE/nyash"

# 1) Build nyash with cranelift
banner "building nyash (features=cranelift-jit)"
if [ "$RUN" = 1 ]; then
  cargo build --$MODE --features cranelift-jit
else
  info "DRYRUN: cargo build --$MODE --features cranelift-jit"
fi

# 2) Emit object via backend=cranelift (PoC path; may be stub until implemented)
banner "emitting object via --backend cranelift (PoC)"
if [ "$RUN" = 1 ]; then
  if [ ! -x "$NYASH_BIN" ]; then
    echo "nyash binary not found: $NYASH_BIN" >&2; exit 2
  fi
  NYASH_AOT_OBJECT_OUT="$OBJ_OUT" "$NYASH_BIN" --backend cranelift apps/hello/main.nyash || true
  if [ ! -s "$OBJ_OUT" ]; then
    echo "object not generated (expected PoC path)." >&2; exit 1
  fi
  info "OK: object generated: $OBJ_OUT ($(stat -c%s "$OBJ_OUT" 2>/dev/null || wc -c <"$OBJ_OUT")) bytes)"
else
  info "DRYRUN: NYASH_AOT_OBJECT_OUT=\"$OBJ_OUT\" $NYASH_BIN --backend cranelift apps/hello/main.nyash"
  info "DRYRUN: touch $OBJ_OUT (pretend non-empty)"
fi

# 3) Link (Windows-first). In DRYRUN, just print the command.
banner "linking app (Windows-first)"
if [ "$RUN" = 1 ]; then
  case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*|Windows_NT)
      if command -v link >/dev/null 2>&1; then
        info "using MSVC link.exe"
        link /OUT:"$EXE_WIN" "$OBJ_OUT" nyrt.lib || { echo "link failed" >&2; exit 1; }
        OUT_BIN="$EXE_WIN"
      elif command -v lld-link >/dev/null 2>&1; then
        info "using lld-link"
        lld-link -OUT:"$EXE_WIN" "$OBJ_OUT" nyrt.lib || { echo "lld-link failed" >&2; exit 1; }
        OUT_BIN="$EXE_WIN"
      else
        echo "no Windows linker found (link.exe/lld-link)" >&2; exit 2
      fi
      ;;
    *)
      if command -v cc >/dev/null 2>&1; then
        cc -o "$EXE_NIX" "$OBJ_OUT" "$TARGET_DIR/release/libnyrt.a" -ldl -lpthread || { echo "cc link failed" >&2; exit 1; }
        OUT_BIN="$EXE_NIX"
      else
        echo "no cc found for Unix link" >&2; exit 2
      fi
      ;;
  esac
else
  case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*|Windows_NT)
      info "DRYRUN: link /OUT:$EXE_WIN $OBJ_OUT nyrt.lib  (or lld-link)"
      ;;
    *)
      info "DRYRUN: cc -o $EXE_NIX $OBJ_OUT target/release/libnyrt.a -ldl -lpthread"
      ;;
  esac
fi

# 4) Run and verify
banner "run and verify output"
if [ "$RUN" = 1 ]; then
  if [ -z "${OUT_BIN:-}" ] || [ ! -x "$OUT_BIN" ]; then
    echo "no output binary to run" >&2; exit 1
  fi
  set +e
  OUTPUT="$($OUT_BIN 2>&1)"; RC=$?
  set -e
  echo "$OUTPUT"
  echo "$OUTPUT" | grep -q "Result:" || { echo "unexpected output" >&2; exit 1; }
  info "OK: smoke passed"
else
  info "DRYRUN: ./app_clif[.exe] → expect a line including: 'Result: 3' or 'Result: 42'"
  info "DRYRUN complete"
fi

exit 0

