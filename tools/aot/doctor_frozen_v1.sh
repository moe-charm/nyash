#!/usr/bin/env bash
# Frozen v1 Doctor — quick end-to-end validation for the frozen mint pipeline
# Runs: emit MIR JSON -> extern_c to .o -> (optional) link with NyRT -> run and check Result line

set -euo pipefail

SHOW_HELP=0
NO_BUILD=0
VERBOSE=0

for arg in "$@"; do
  case "$arg" in
    --help|-h) SHOW_HELP=1 ;;
    --no-build) NO_BUILD=1 ;;
    --verbose|-v) VERBOSE=1 ;;
    *) echo "[doctor][WARN] Unknown arg: $arg" ;;
  esac
done

if [ $SHOW_HELP -eq 1 ]; then
  cat <<'H'
Frozen v1 Doctor — options:
  --no-build   Skip cargo build steps (assumes artifacts present)
  --verbose    Print extra diagnostics
  --help       Show this message
H
  exit 0
fi

ROOT="${NYASH_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
BIN="${ROOT}/target/release/hakorune"

echo "[doctor] root: $ROOT"

# Preflight: python + llvmlite
if ! command -v python3 >/dev/null 2>&1; then
  echo "[doctor][ERROR] python3 not found. Install: sudo apt-get install -y python3 python3-pip" >&2
  exit 2
fi
python3 - <<'PY' || { echo "[doctor][ERROR] llvmlite not importable. Try: pip install llvmlite" >&2; exit 2; }
try:
  import llvmlite.ir as _
  import llvmlite.binding as _
  print("[doctor][OK] llvmlite import")
except Exception as e:
  print("[doctor][ERROR] llvmlite import:", e)
  raise
PY

# Build VM if missing
if [ ! -x "$BIN" ] && [ $NO_BUILD -eq 0 ]; then
  echo "[doctor] building hakorune VM..." && cargo build --release >/dev/null
fi

# Build llvm_backend cdylib
if [ $NO_BUILD -eq 0 ]; then
  echo "[doctor] building llvm_backend (cdylib) ..."
  cargo build --release -p llvm_backend >/dev/null
fi

# Optional NyRT (static runtime)
NYRT_A="${ROOT}/target/release/libhako_kernel.a"
if [ ! -f "$NYRT_A" ] && [ $NO_BUILD -eq 0 ]; then
  echo "[doctor] building hako_kernel (static NyRT) ..."
  set +e
  cargo build --release -p hako_kernel >/dev/null
  EC=$?
  set -e
  if [ $EC -ne 0 ] || [ ! -f "$NYRT_A" ]; then
    echo "[doctor][WARN] NyRT not available; will use a tiny C main() stub (link still attempted)" >&2
    NYRT_A=""
  fi
fi

# Tooling check
DO_SKIP_LINK=0
if ! command -v clang >/dev/null 2>&1; then
  echo "[doctor][WARN] clang not found. Link step will be skipped." >&2
  echo "[doctor][HINT] Linux/WSL: sudo apt-get install -y clang" >&2
  echo "[doctor][HINT] Windows(MSVC): install LLVM (e.g., C:\\LLVM-18) and use clang.exe" >&2
  DO_SKIP_LINK=1
fi

# Allowlist + lib path for extern_c
export HAKO_FFI_LIB_PATHS="${ROOT}/target/release"
if [[ ",${HAKO_FFI_ALLOW_LIST:-}," != *",llvm_compile_mir_to_object,"* ]]; then
  export HAKO_FFI_ALLOW_LIST=$(echo "${HAKO_FFI_ALLOW_LIST:-},llvm_compile_mir_to_object" | sed 's/^,//')
  echo "[doctor][OK] allowlist extended: HAKO_FFI_ALLOW_LIST=$HAKO_FFI_ALLOW_LIST"
fi

WORK="${ROOT}/build/doctor"
mkdir -p "$WORK/obj" "$WORK/mir" "$WORK/bin" "$WORK/ir"

SRC="${ROOT}/examples/simple_return.hako"
test -f "$SRC" || { echo "[doctor][ERROR] sample source missing: $SRC" >&2; exit 2; }

echo "[doctor] emitting MIR JSON ..."
"$BIN" --backend mir --emit-mir-json "$WORK/mir/main.mir.json" "$SRC" >/dev/null

echo "[doctor] emitting object via extern_c ..."
tools/aot/emit_object_via_extern_c.sh "$WORK/mir/main.mir.json" "$WORK/obj/main.o" >/dev/null

if [ $DO_SKIP_LINK -eq 1 ]; then
  echo "[doctor][SKIP] link step (clang not found). Object at: $WORK/obj/main.o"
  echo "[doctor][HINT] To link: sudo apt-get install -y clang; then run tools/aot/link_with_clang.sh"
  exit 0
fi

echo "[doctor] linking ..."
if [ -n "$NYRT_A" ]; then
  tools/aot/link_with_clang.sh -o "$WORK/bin/hako-doctor" "$WORK/obj/main.o" --nyrt "$NYRT_A" >/dev/null
else
  # Fallback: compile a minimal C stub that calls ny_main()
  STUB_C="$WORK/obj/nyrt_stub.c"
  cat > "$STUB_C" <<'C'
  #include <stdio.h>
  #include <stdint.h>
  extern int64_t ny_main(void);
  int main(void){
    int64_t v = ny_main();
    printf("Result: %lld\n", (long long)v);
    return (int)(v & 0xFF);
  }
C
  clang -c "$STUB_C" -o "$WORK/obj/nyrt_stub.o"
  # Provide minimal stubs for runtime exports referenced by the object (x86_64 only)
  ASM_STUB="$WORK/obj/nyrt_min_stubs.S"
  cat > "$ASM_STUB" <<'ASM'
  .text
  .globl nyash.box.from_i8_string
  .type nyash.box.from_i8_string,@function
nyash.box.from_i8_string:
  xor %rax,%rax
  ret
  .globl nyash.string.concat_hh
  .type nyash.string.concat_hh,@function
nyash.string.concat_hh:
  xor %rax,%rax
  ret
ASM
  clang -c "$ASM_STUB" -o "$WORK/obj/nyrt_min_stubs.o"
  tools/aot/link_with_clang.sh -o "$WORK/bin/hako-doctor" \
    "$WORK/obj/main.o" "$WORK/obj/nyrt_stub.o" "$WORK/obj/nyrt_min_stubs.o" >/dev/null
fi

echo "[doctor] running ..."
set +e
OUT=$("$WORK/bin/hako-doctor" 2>&1)
EC=$?
set -e
if [ $VERBOSE -eq 1 ]; then echo "$OUT" | sed -E 's/^/[doctor] run | /'; fi

if echo "$OUT" | grep -q "^Result: "; then
  echo "[doctor][OK] run: Result line detected"
else
  echo "[doctor][ERROR] run: no Result line (exit=$EC). Check allowlist/lib paths/nyrt." >&2
  echo "[doctor][HINT] Ensure: HAKO_FFI_ALLOW_LIST includes llvm_compile_mir_to_object" >&2
  echo "[doctor][HINT] Ensure: HAKO_FFI_LIB_PATHS includes $(printf '%q' "$ROOT/target/release")" >&2
  echo "[doctor][HINT] If NyRT missing, doctor auto-stubs; for parity, build -p hako_kernel" >&2
  exit ${EC:-1}
fi

echo "[doctor] extended: building extra libs (parser/mir_builder/vm) ..."
cat > "$WORK/multi_parser.hako" <<'HK'
static box Parser {
  header() { return 0 }
}
HK
cat > "$WORK/multi_mir_builder.hako" <<'HK'
static box MirBuilder {
  build() { return 0 }
}
HK
cat > "$WORK/multi_vm.hako" <<'HK'
static box VM {
  run() { return 0 }
}
HK

"$BIN" --backend mir --emit-mir-json "$WORK/mir/parser.mir.json" "$WORK/multi_parser.hako" >/dev/null
"$BIN" --backend mir --emit-mir-json "$WORK/mir/mir_builder.mir.json" "$WORK/multi_mir_builder.hako" >/dev/null
"$BIN" --backend mir --emit-mir-json "$WORK/mir/vm.mir.json" "$WORK/multi_vm.hako" >/dev/null

NYASH_LLVM_NO_NY_MAIN=1 tools/aot/emit_object_via_extern_c.sh "$WORK/mir/parser.mir.json" "$WORK/obj/parser.o" >/dev/null
NYASH_LLVM_NO_NY_MAIN=1 tools/aot/emit_object_via_extern_c.sh "$WORK/mir/mir_builder.mir.json" "$WORK/obj/mir_builder.o" >/dev/null
NYASH_LLVM_NO_NY_MAIN=1 tools/aot/emit_object_via_extern_c.sh "$WORK/mir/vm.mir.json" "$WORK/obj/vm.o" >/dev/null

echo "[doctor] extended: relinking with extra libs ..."
ALLOW_MULTI='--extra -Wl,--allow-multiple-definition'
if [ -n "$NYRT_A" ]; then
  tools/aot/link_with_clang.sh -o "$WORK/bin/hako-doctor-ext" \
    "$WORK/obj/main.o" "$WORK/obj/parser.o" "$WORK/obj/mir_builder.o" "$WORK/obj/vm.o" --nyrt "$NYRT_A" $ALLOW_MULTI >/dev/null
else
  tools/aot/link_with_clang.sh -o "$WORK/bin/hako-doctor-ext" \
    "$WORK/obj/main.o" "$WORK/obj/parser.o" "$WORK/obj/mir_builder.o" "$WORK/obj/vm.o" \
    "$WORK/obj/nyrt_stub.o" "$WORK/obj/nyrt_min_stubs.o" $ALLOW_MULTI >/dev/null
fi

set +e
OUT2=$("$WORK/bin/hako-doctor-ext" 2>&1)
EC2=$?
set -e
if [ $VERBOSE -eq 1 ]; then echo "$OUT2" | sed -E 's/^/[doctor] run ext | /'; fi

if echo "$OUT2" | grep -q "^Result: "; then
  echo "[doctor][OK] extended run: Result line detected"
  exit 0
fi
if [ ${EC2:-1} -eq 0 ]; then
  echo "[doctor][OK] extended run: exit=0"
  exit 0
fi
echo "[doctor][ERROR] extended run failed (exit=$EC2)." >&2
exit ${EC2:-1}
