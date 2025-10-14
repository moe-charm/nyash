#!/usr/bin/env bash
# Frozen v1 Doctor — quick end-to-end validation for the frozen mint pipeline
# Runs: emit MIR JSON -> extern_c to .o -> link with NyRT -> run and check Result line

set -euo pipefail

ROOT="${NYASH_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
BIN="${ROOT}/target/release/hakorune"

echo "[doctor] root: $ROOT"

if ! command -v python3 >/dev/null 2>&1; then
  echo "[doctor] python3 not found" >&2
  exit 2
fi

python3 - <<'PY' || { echo "[doctor] llvmlite not importable" >&2; exit 2; }
try:
  import llvmlite.ir as _
  import llvmlite.binding as _
  print("[doctor] llvmlite OK")
except Exception as e:
  print("[doctor] llvmlite import error:", e)
  raise
PY

if [ ! -x "$BIN" ]; then
  echo "[doctor] building hakorune VM..." && cargo build --release >/dev/null
fi

echo "[doctor] building llvm_backend (cdylib) ..."
cargo build --release -p llvm_backend >/dev/null

NYRT_A="${ROOT}/crates/hako_kernel/target/release/libhako_kernel.a"
if [ ! -f "$NYRT_A" ]; then
  echo "[doctor] building hako_kernel (static NyRT) ..."
  set +e
  cargo build --release -p hako_kernel >/dev/null
  EC=$?
  set -e
  if [ $EC -ne 0 ] || [ ! -f "$NYRT_A" ]; then
    echo "[doctor] NyRT build unavailable; will use a tiny C main() stub instead" >&2
    NYRT_A=""
  fi
fi

export HAKO_FFI_LIB_PATHS="${ROOT}/target/release"
export HAKO_FFI_ALLOW_LIST=$(echo "${HAKO_FFI_ALLOW_LIST:-},llvm_compile_mir_to_object" | sed 's/^,//')

WORK="${ROOT}/build/doctor"
mkdir -p "$WORK/obj" "$WORK/mir" "$WORK/bin" "$WORK/ir"

SRC="${ROOT}/examples/simple_return.hako"
test -f "$SRC" || { echo "[doctor] sample source missing: $SRC" >&2; exit 2; }

echo "[doctor] emitting MIR JSON ..."
"$BIN" --backend mir --emit-mir-json "$WORK/mir/main.mir.json" "$SRC" >/dev/null

echo "[doctor] emitting object via extern_c ..."
tools/aot/emit_object_via_extern_c.sh "$WORK/mir/main.mir.json" "$WORK/obj/main.o" >/dev/null

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
echo "$OUT" | sed -E 's/^/[doctor] run | /'

if echo "$OUT" | grep -q "^Result: "; then
  echo "[doctor] PASS"
else
  echo "[doctor] FAIL (exit=$EC)" >&2
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
echo "$OUT2" | sed -E 's/^/[doctor] run ext | /'

if echo "$OUT2" | grep -q "^Result: "; then
  echo "[doctor] PASS (extended)"
  exit 0
fi
if [ ${EC2:-1} -eq 0 ]; then
  echo "[doctor] PASS (extended: exit=0)"
  exit 0
fi
echo "[doctor] FAIL (extended) exit=$EC2" >&2
exit ${EC2:-1}
