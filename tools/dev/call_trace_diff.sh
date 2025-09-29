#!/usr/bin/env bash
# call_trace_diff.sh — Compare VM runtime call trace vs LLVM static call listing for a Nyash file
# Usage: tools/dev/call_trace_diff.sh <file.nyash> [--args 'extra args']

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT_DIR/target/release/nyash"
FILE="${1:-}"
shift || true
EXTRA_ARGS=( )
if [[ "${1:-}" == "--args" ]]; then
  shift
  IFS=' ' read -r -a EXTRA_ARGS <<< "${1:-}"
  shift || true
fi

if [[ -z "$FILE" || ! -f "$FILE" ]]; then
  echo "Usage: $0 <file.nyash> [--args 'extra args']" >&2
  exit 2
fi
if [[ ! -x "$BIN" ]]; then
  echo "Nyash binary not found: $BIN (run 'cargo build --release')" >&2
  exit 2
fi

TMPDIR="/tmp/nyash_calltrace_$$"
mkdir -p "$TMPDIR"

# VM runtime trace
VM_LOG="$TMPDIR/vm.log"
{ NYASH_CALL_TRACE=1 "$BIN" --backend vm "$FILE" "${EXTRA_ARGS[@]}" || true; } 2>"$VM_LOG" >/dev/null
VM_SEQ="$TMPDIR/vm.seq"
grep '^{"kind":"call"' "$VM_LOG" | sed -E 's/.*"callee":"([^"]+)".*/\1/' > "$VM_SEQ" || true

# LLVM static call listing (harness)
LLVM_LOG="$TMPDIR/llvm.log"
{ NYASH_CALL_TRACE=1 NYASH_LLVM_USE_HARNESS=1 "$BIN" --backend llvm "$FILE" "${EXTRA_ARGS[@]}" || true; } 2>"$LLVM_LOG" >/dev/null
LLVM_SEQ="$TMPDIR/llvm.seq"
# Accept either plain text (legacy) or JSON lines (current) from LLVM side
(
  # Legacy format: lines starting with 'call_static'
  grep '^call_static' "$LLVM_LOG" | sed -E 's/^call_static +//'
  # JSON format: {"kind":"call_static", "callee":"..." ...}
  grep -E '^\{\"kind\":\"call_static\"' "$LLVM_LOG" | sed -E 's/.*\"callee\":\"([^\"]+)\".*/\1/'
) > "$LLVM_SEQ" || true

echo "== VM runtime calls (ordered) =="
nl -ba "$VM_SEQ" || true
echo ""
echo "== LLVM static call sites (unordered) =="
nl -ba "$LLVM_SEQ" || true
echo ""

echo "== Diff (VM vs LLVM) =="
diff -u "$LLVM_SEQ" "$VM_SEQ" || true

echo "Logs: $TMPDIR"
exit 0
