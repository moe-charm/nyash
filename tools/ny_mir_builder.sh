#!/usr/bin/env bash
# ny_mir_builder.sh — Minimal MIR Builder CLI (shell wrapper)
# Purpose: consume Nyash JSON IR and emit {obj|exe|ll|json} using the existing nyash LLVM harness.

set -euo pipefail
[[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]] && set -x

usage() {
  cat << USAGE
Usage: tools/ny_mir_builder.sh [--in <file>|--stdin] [--emit {obj|exe|ll|json}] -o <out> [--target <triple>] [--nyrt <path>] [--quiet] [--verify-llvm]

Notes:
  - This is a Phase-15 shell wrapper that leverages the nyash LLVM harness.
  - Input must be Nyash JSON IR (v0/v1). When --stdin is used, reads from stdin.
  - For --emit exe, NyRT must be built (crates/nyrt). Use default paths if --nyrt omitted.
USAGE
}

IN_MODE="stdin"   # stdin | file
IN_FILE=""
EMIT="obj"        # obj | exe | ll | json
OUT=""
TARGET=""
NYRT_DIR=""
VERIFY=0
QUIET=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage; exit 0 ;;
    --in) IN_MODE="file"; IN_FILE="$2"; shift 2 ;;
    --stdin) IN_MODE="stdin"; shift ;;
    --emit) EMIT="$2"; shift 2 ;;
    -o) OUT="$2"; shift 2 ;;
    --target) TARGET="$2"; shift 2 ;;
    --nyrt) NYRT_DIR="$2"; shift 2 ;;
    --verify-llvm) VERIFY=1; shift ;;
    --quiet) QUIET=1; shift ;;
    *) echo "unknown arg: $1" >&2; usage; exit 2 ;;
  esac
done

if [[ -z "$OUT" ]]; then
  case "$EMIT" in
    obj) OUT="$(pwd)/target/aot_objects/a.o" ;;
    ll)  OUT="$(pwd)/target/aot_objects/a.ll" ;;
    exe) OUT="a.out" ;;
    json) OUT="/dev/stdout" ;;
    *) echo "error: invalid emit kind: $EMIT" >&2; exit 2 ;;
  esac
fi

if ! command -v llvm-config-18 >/dev/null 2>&1; then
  echo "error: llvm-config-18 not found (install LLVM 18 dev)" >&2
  exit 3
fi

# Build nyash + NyRT as needed
_LLVMPREFIX=$(llvm-config-18 --prefix)
LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" \
  cargo build --release -j 24 --features llvm >/dev/null
if [[ "$EMIT" == "exe" ]]; then
  (cd crates/nyrt && cargo build --release -j 24 >/dev/null)
fi

mkdir -p "$PWD/target/aot_objects"

# Prepare input
_STDIN_BUF=""
if [[ "$IN_MODE" == "stdin" ]]; then
  # Read all to a temp file to allow re-use
  _TMP_JSON=$(mktemp)
  cat > "$_TMP_JSON"
  IN_FILE="$_TMP_JSON"
fi

cleanup() { [[ -n "${_TMP_JSON:-}" && -f "$_TMP_JSON" ]] && rm -f "$_TMP_JSON" || true; }
trap cleanup EXIT

case "$EMIT" in
  json)
    # Normalization placeholder: currently pass-through
    cat "$IN_FILE" > "$OUT"
    [[ "$QUIET" == "0" ]] && echo "OK json:$OUT"
    ;;
  ll)
    # Ask nyash harness to dump LLVM IR (if supported via env)
    export NYASH_LLVM_DUMP_LL=1
    export NYASH_LLVM_LL_OUT="$OUT"
    if [[ "$VERIFY" == "1" ]]; then export NYASH_LLVM_VERIFY=1; fi
    cat "$IN_FILE" | NYASH_LLVM_USE_HARNESS=1 LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" \
      ./target/release/nyash --backend llvm --ny-parser-pipe >/dev/null || true
    if [[ ! -f "$OUT" ]]; then echo "error: failed to produce $OUT" >&2; exit 4; fi
    [[ "$QUIET" == "0" ]] && echo "OK ll:$OUT"
    ;;
  obj)
    export NYASH_LLVM_OBJ_OUT="$OUT"
    if [[ "$VERIFY" == "1" ]]; then export NYASH_LLVM_VERIFY=1; fi
    rm -f "$OUT"
    cat "$IN_FILE" | NYASH_LLVM_USE_HARNESS=1 LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" \
      ./target/release/nyash --backend llvm --ny-parser-pipe >/dev/null || true
    if [[ ! -f "$OUT" ]]; then echo "error: failed to produce $OUT" >&2; exit 4; fi
    [[ "$QUIET" == "0" ]] && echo "OK obj:$OUT"
    ;;
  exe)
    # Emit obj then link
    OBJ="$PWD/target/aot_objects/__tmp_builder.o"
    export NYASH_LLVM_OBJ_OUT="$OBJ"
    if [[ "$VERIFY" == "1" ]]; then export NYASH_LLVM_VERIFY=1; fi
    rm -f "$OBJ"
    cat "$IN_FILE" | NYASH_LLVM_USE_HARNESS=1 LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" \
      ./target/release/nyash --backend llvm --ny-parser-pipe >/dev/null || true
    if [[ ! -f "$OBJ" ]]; then echo "error: failed to produce object $OBJ" >&2; exit 4; fi
    # Link with NyRT
    NYRT_BASE=${NYRT_DIR:-"$PWD/crates/nyrt"}
    cc "$OBJ" \
      -L target/release \
      -L "$NYRT_BASE/target/release" \
      -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive \
      -lpthread -ldl -lm -o "$OUT"
    [[ "$QUIET" == "0" ]] && echo "OK exe:$OUT"
    ;;
  *) echo "error: invalid emit kind: $EMIT" >&2; exit 2 ;;
esac

exit 0

