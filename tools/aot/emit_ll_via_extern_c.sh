#!/usr/bin/env bash
# Emit a .ll (LLVM IR) file from MIR JSON via extern_c + libllvm_backend
# Usage: tools/aot/emit_ll_via_extern_c.sh <mir.json> <out.ll>

set -euo pipefail

if [ $# -ne 2 ]; then
  echo "Usage: $0 <mir.json> <out.ll>" >&2
  exit 2
fi

MIR_JSON="$1"
OUT_LL="$2"

if [ ! -f "$MIR_JSON" ]; then
  echo "MIR JSON not found: $MIR_JSON" >&2
  exit 2
fi

ROOT="${NYASH_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
BIN="${ROOT}/target/release/hakorune"
if [ ! -x "$BIN" ]; then
  echo "hakorune not built at $BIN. Run: cargo build --release" >&2
  exit 2
fi

# Ensure allowlist and library search path
export HAKO_FFI_ALLOW_LIST="${HAKO_FFI_ALLOW_LIST:-}"
if ! echo "$HAKO_FFI_ALLOW_LIST" | grep -q '\<llvm_compile_mir_to_ll\>'; then
  export HAKO_FFI_ALLOW_LIST=$(echo "${HAKO_FFI_ALLOW_LIST:+$HAKO_FFI_ALLOW_LIST,}llvm_compile_mir_to_ll")
fi

if [ -z "${HAKO_FFI_LIB_PATHS:-}" ]; then
  export HAKO_FFI_LIB_PATHS="${ROOT}/target/release"
fi

TMP_SRC="/tmp/hako_emit_ll_$$.hako"
cat > "$TMP_SRC" << 'HK'
static box Main {
  main() {
    // inputs compiled inline by wrapper
    local in;  in  = "__IN__";
    local out; out = "__OUT__";

    local rc; rc = extern_c "llvm_compile_mir_to_ll" (in, out);
    if (rc == 0) { print("OK"); } else { print("NG"); }
    return rc;
  }
}
HK

# Inline paths
sed -i -e "s@__IN__@${MIR_JSON}@g" -e "s@__OUT__@${OUT_LL}@g" "$TMP_SRC"

set +e
OUT=$("$BIN" --backend vm "$TMP_SRC" 2>&1)
EC=$?
set -e
rm -f "$TMP_SRC"

echo "$OUT" | grep -q '^OK$' && exit 0 || {
  echo "$OUT" | sed -E 's/^/  /' >&2
  exit ${EC:-1}
}

