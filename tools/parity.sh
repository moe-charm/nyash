#!/usr/bin/env bash
set -euo pipefail

if [[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]]; then
  set -x
fi

usage() {
  cat << USAGE
Nyash parity runner — compare two execution paths on the same .nyash

Usage: tools/parity.sh [options] <app.nyash>

Options:
  --lhs <mode>   Left  mode: pyvm|llvmlite|vm   (default: pyvm)
  --rhs <mode>   Right mode: pyvm|llvmlite|vm   (default: llvmlite)
  --timeout <s>  Timeout seconds for each run     (default: 12)
  --show-diff    Show unified diff when different

Compares stdout (normalized) and exit codes. Returns 0 when equal.
USAGE
}

APP=""
LHS="pyvm"
RHS="llvmlite"
TIMEOUT="12"
SHOW_DIFF=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage; exit 0;;
    --lhs) LHS="$2"; shift 2;;
    --rhs) RHS="$2"; shift 2;;
    --timeout) TIMEOUT="$2"; shift 2;;
    --show-diff) SHOW_DIFF=1; shift;;
    *) APP="$1"; shift;;
  esac
done

if [[ -z "$APP" ]]; then
  usage; exit 1
fi
if [[ ! -f "$APP" ]]; then
  echo "error: app not found: $APP" >&2
  exit 2
fi

ROOT=$(cd "$(dirname "$0")/.." && pwd)
NYASH_BIN="$ROOT/target/release/nyash"
if [[ ! -x "$NYASH_BIN" ]]; then
  echo "[build] nyash not found; building release ..." >&2
  (cd "$ROOT" && cargo build --release >/dev/null)
fi

has_cmd() { command -v "$1" >/dev/null 2>&1; }

normalize() {
  # Remove runner/plugin noise and blank lines
  sed -E \
    -e 's/\r$//' \
    -e '/^\[ConsoleBox\]/d' \
    -e '/^\[FileBox\]/d' \
    -e '/^\[plugin-loader\]/d' \
    -e '/^\[Runner\//d' \
    -e '/^DEBUG:/d' \
    -e '/^🔌/d' \
    -e '/^✅/d' \
    -e '/^🚀/d' \
    -e '/^⚡/d' \
    -e '/^🦀/d' \
    -e '/^🧠/d' \
    -e '/^📊/d' \
    -e '/^Result(Type)?\(/d' \
    -e '/^Result:/d' \
    -e '/^$/d'
}

run_pyvm() {
  local app="$1"
  local out code
  if has_cmd timeout; then
    out=$(NYASH_VM_USE_PY=1 timeout "${TIMEOUT}s" "$NYASH_BIN" --backend vm "$app" 2>&1) || code=$?
  else
    out=$(NYASH_VM_USE_PY=1 "$NYASH_BIN" --backend vm "$app" 2>&1) || code=$?
  fi
  code=${code:-0}
  printf '%s' "$out" | normalize
  echo "__EXIT_CODE__=$code"
}

run_vm() {
  local app="$1"
  local out code
  if has_cmd timeout; then
    out=$(timeout "${TIMEOUT}s" "$NYASH_BIN" --backend vm "$app" 2>&1) || code=$?
  else
    out=$("$NYASH_BIN" --backend vm "$app" 2>&1) || code=$?
  fi
  code=${code:-0}
  printf '%s' "$out" | normalize
  echo "__EXIT_CODE__=$code"
}

run_llvmlite() {
  local app="$1"
  if ! has_cmd llvm-config-18; then
    echo "error: llvm-config-18 not found (required for llvmlite parity)." >&2
    exit 3
  fi
  local stem exe
  stem=$(basename "$app"); stem=${stem%.nyash}
  exe="$ROOT/app_parity_${stem}"
  NYASH_LLVM_FEATURE=llvm "${ROOT}/tools/build_llvm.sh" "$app" -o "$exe" >/dev/null || true
  if [[ ! -x "$exe" ]]; then
    echo "error: failed to build llvmlite executable: $exe" >&2
    exit 4
  fi
  local out code
  if has_cmd timeout; then
    out=$(timeout "${TIMEOUT}s" "$exe" 2>&1) || code=$?
  else
    out=$("$exe" 2>&1) || code=$?
  fi
  code=${code:-0}
  printf '%s' "$out" | normalize
  echo "__EXIT_CODE__=$code"
}

run_mode() {
  local mode="$1" app="$2"
  case "$mode" in
    pyvm) run_pyvm "$app" ;;
    vm) run_vm "$app" ;;
    llvmlite) run_llvmlite "$app" ;;
    *) echo "error: unknown mode: $mode" >&2; exit 5;;
  esac
}

LEFT=$(run_mode "$LHS" "$APP")
RIGHT=$(run_mode "$RHS" "$APP")

LCODE=$(printf '%s\n' "$LEFT" | sed -n 's/^__EXIT_CODE__=//p')
RCODE=$(printf '%s\n' "$RIGHT" | sed -n 's/^__EXIT_CODE__=//p')
LOUT=$(printf '%s\n' "$LEFT" | sed '/^__EXIT_CODE__=/d')
ROUT=$(printf '%s\n' "$RIGHT" | sed '/^__EXIT_CODE__=/d')

STATUS=0
if [[ "$LCODE" != "$RCODE" ]]; then
  echo "[parity] exit code differs: $LHS=$LCODE, $RHS=$RCODE" >&2
  STATUS=1
fi
if [[ "$LOUT" != "$ROUT" ]]; then
  echo "[parity] stdout differs" >&2
  if [[ "$SHOW_DIFF" -eq 1 ]]; then
    diff -u <(printf '%s\n' "$LOUT") <(printf '%s\n' "$ROUT") || true
  fi
  STATUS=1
fi

if [[ "$STATUS" -eq 0 ]]; then
  echo "✅ parity OK ($LHS == $RHS)" >&2
else
  echo "❌ parity mismatch ($LHS != $RHS)" >&2
fi
exit "$STATUS"
