#!/usr/bin/env bash
# call_trace_diff.sh — Compare VM runtime call trace vs LLVM static call listing for a Nyash file
# Usage: tools/dev/call_trace_diff.sh <file.nyash> [--args 'extra args'] [--kinds 'Method,Global,BoxCall,PluginInvoke']

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT_DIR/target/release/nyash"
FILE="${1:-}"
shift || true
EXTRA_ARGS=( )
KINDS_FILTER=""
if [[ "${1:-}" == "--args" ]]; then
  shift
  IFS=' ' read -r -a EXTRA_ARGS <<< "${1:-}"
  shift || true
fi
if [[ "${1:-}" == "--kinds" ]]; then
  shift
  KINDS_FILTER="${1:-}"
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

# Optional kinds filter (Method,Global,BoxCall,PluginInvoke)
if [[ -n "$KINDS_FILTER" ]]; then
  IFS=',' read -r -a KLIST <<< "$KINDS_FILTER"
  PATTERN="$(printf '%s\n' "${KLIST[@]}" | sed 's/$/:/; s/^/^/; s/$/|/' | tr -d '\n' | sed 's/|$//')"
  if [[ -n "$PATTERN" ]]; then
    grep -E "$PATTERN" "$VM_SEQ" > "$VM_SEQ.filt" || true && mv "$VM_SEQ.filt" "$VM_SEQ"
    grep -E "$PATTERN" "$LLVM_SEQ" > "$LLVM_SEQ.filt" || true && mv "$LLVM_SEQ.filt" "$LLVM_SEQ"
  fi
fi

echo "== VM runtime calls (ordered) =="
nl -ba "$VM_SEQ" || true
echo ""
echo "== LLVM static call sites (unordered) =="
nl -ba "$LLVM_SEQ" || true
echo ""

echo "== Diff (VM vs LLVM) =="
diff -u "$LLVM_SEQ" "$VM_SEQ" || true

echo "Logs: $TMPDIR"

# Set-based summary (order-insensitive)
VM_SET="$TMPDIR/vm.set"
LLVM_SET="$TMPDIR/llvm.set"
sort -u "$VM_SEQ" > "$VM_SET" || true
sort -u "$LLVM_SEQ" > "$LLVM_SET" || true

ONLY_VM="$TMPDIR/only_vm.set"
ONLY_LLVM="$TMPDIR/only_llvm.set"
comm -23 "$VM_SET" "$LLVM_SET" > "$ONLY_VM" || true
comm -13 "$VM_SET" "$LLVM_SET" > "$ONLY_LLVM" || true

echo ""
echo "== Set Summary (order-insensitive) =="
echo "VM unique:   $(wc -l < "$ONLY_VM" | tr -d ' ')"
echo "LLVM unique: $(wc -l < "$ONLY_LLVM" | tr -d ' ')"
if [[ -s "$ONLY_VM" ]]; then
  echo "-- Only in VM --"; nl -ba "$ONLY_VM"; fi
if [[ -s "$ONLY_LLVM" ]]; then
  echo "-- Only in LLVM --"; nl -ba "$ONLY_LLVM"; fi

if [[ ! -s "$ONLY_VM" ]]; then
  echo "Result: OK (VM ⊆ LLVM)"
else
  echo "Result: DIFF (VM has entries not present in LLVM)"
fi
exit 0
