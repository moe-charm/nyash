#!/usr/bin/env bash
set -euo pipefail

# LLVM Phase 11.2 smoke test
# - Builds nyash with LLVM backend
# - Compiles example via --backend llvm (object emission inside)
# - Verifies object file presence and non-zero size

MODE=${1:-release}
BIN=./target/${MODE}/nyash
# Optional output dir for produced app_* binaries (default: current dir)
APP_BIN_DIR=${APP_BIN_DIR:-.}
mkdir -p "$APP_BIN_DIR"
# Fixed object output directory for stability
mkdir -p target/aot_objects
OBJ="$PWD/target/aot_objects/core_smoke.o"

if ! command -v llvm-config-18 >/dev/null 2>&1; then
  echo "error: llvm-config-18 not found. Please install LLVM 18 dev packages." >&2
  exit 2
fi

# Conditional LLVM prefix setup based on feature
LLVM_FEATURE=${NYASH_LLVM_FEATURE:-llvm}
if [[ "$LLVM_FEATURE" == "llvm-inkwell-legacy" ]]; then
  # Legacy inkwell needs LLVM_SYS_180_PREFIX
  _LLVMPREFIX=$(llvm-config-18 --prefix)
  export LLVM_SYS_181_PREFIX="${_LLVMPREFIX}"
  export LLVM_SYS_180_PREFIX="${_LLVMPREFIX}"
  echo "[llvm-smoke] Using legacy inkwell with LLVM_SYS_180_PREFIX=${_LLVMPREFIX}" >&2
else
  # llvm-harness (default) doesn't need LLVM_SYS_180_PREFIX
  echo "[llvm-smoke] Using llvm-harness (LLVM_SYS_180_PREFIX not required)" >&2
fi

# --- AOT smoke: apps/ny-llvm-bitops (bitwise & shift operations) ---
if [[ "${NYASH_LLVM_BITOPS_SMOKE:-0}" == "1" ]]; then
  echo "[llvm-smoke] building + linking apps/ny-llvm-bitops ..." >&2
  OBJ_BIT="$PWD/target/aot_objects/bitops_smoke.o"
  rm -f "$OBJ_BIT"
  NYASH_LLVM_OBJ_OUT="$OBJ_BIT" "$BIN" --backend llvm apps/tests/ny-llvm-bitops/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_BIT" ./tools/build_llvm.sh apps/tests/ny-llvm-bitops/main.nyash -o "$APP_BIN_DIR/app_bitops_llvm" >/dev/null || true
  echo "[llvm-smoke] running app_bitops_llvm ..." >&2
  out_bit=$("$APP_BIN_DIR/app_bitops_llvm" || true)
  echo "[llvm-smoke] output: $out_bit" >&2
  if ! echo "$out_bit" | grep -q "Result: 48"; then
    echo "error: ny-llvm-bitops unexpected output: $out_bit" >&2
    exit 1
  fi
  echo "[llvm-smoke] OK: bitwise/shift smoke passed" >&2
else
  echo "[llvm-smoke] skipping ny-llvm-bitops (set NYASH_LLVM_BITOPS_SMOKE=1 to enable)" >&2
fi

echo "[llvm-smoke] building nyash (${MODE}, feature=${LLVM_FEATURE})..." >&2
cargo build -q ${MODE:+--${MODE}} --features "${LLVM_FEATURE}"

echo "[llvm-smoke] running --backend llvm on examples/llvm11_core_smoke.nyash ..." >&2
rm -f "$OBJ"
NYASH_LLVM_USE_HARNESS=1 NYASH_LLVM_OBJ_OUT="$OBJ" "$BIN" --backend llvm examples/llvm11_core_smoke.nyash >/dev/null || true

if [[ ! -f "$OBJ" ]]; then
  echo "error: expected object not found: $OBJ" >&2
  exit 1
fi
if [[ ! -s "$OBJ" ]]; then
  echo "error: object is empty: $OBJ" >&2
  exit 1
fi

echo "[llvm-smoke] OK: object generated: $OBJ ($(stat -c%s "$OBJ") bytes)" >&2

# --- Stage-3 loop control smoke (break/continue) ---
if [[ "${NYASH_LLVM_STAGE3_SMOKE:-0}" == "1" ]]; then
  echo "[llvm-smoke] building + linking apps/tests/llvm_stage3_loop_only.nyash ..." >&2
  OBJ_STAGE3="$PWD/target/aot_objects/stage3_loop_smoke.o"
  rm -f "$OBJ_STAGE3"
  # Loop-only case: harness should succeed (no exceptions in IR)
  NYASH_LLVM_USE_HARNESS=1 NYASH_LLVM_OBJ_OUT="$OBJ_STAGE3" \
    "$BIN" --backend llvm apps/tests/llvm_stage3_loop_only.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_STAGE3" \
    ./tools/build_llvm.sh apps/tests/llvm_stage3_loop_only.nyash -o "$APP_BIN_DIR/app_stage3_loop" >/dev/null || true
  echo "[llvm-smoke] running app_stage3_loop ..." >&2
  out_stage3=$("$APP_BIN_DIR/app_stage3_loop" || true)
  echo "[llvm-smoke] output: $out_stage3" >&2
  if ! echo "$out_stage3" | grep -q "Result: 3"; then
    echo "error: stage3 loop smoke unexpected output: $out_stage3" >&2
    exit 1
  fi
  echo "[llvm-smoke] OK: Stage-3 break/continue smoke passed" >&2
else
  echo "[llvm-smoke] skipping Stage-3 loop smoke (set NYASH_LLVM_STAGE3_SMOKE=1 to enable)" >&2
fi

# --- AOT smoke: apps/ny-llvm-smoke (Array get/set/print) ---
if [[ "${NYASH_LLVM_ARRAY_SMOKE:-0}" == "1" ]]; then
  echo "[llvm-smoke] building + linking apps/ny-llvm-smoke ..." >&2
  # Pre-emit object explicitly (more stable)
  OBJ_ARRAY="$PWD/target/aot_objects/array_smoke.o"
  rm -f "$OBJ_ARRAY"
  NYASH_LLVM_OBJ_OUT="$OBJ_ARRAY" "$BIN" --backend llvm apps/tests/ny-llvm-smoke/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_ARRAY" ./tools/build_llvm.sh apps/tests/ny-llvm-smoke/main.nyash -o "$APP_BIN_DIR/app_link" >/dev/null
  echo "[llvm-smoke] running app_link ..." >&2
  out_smoke=$("$APP_BIN_DIR/app_link" || true)
  echo "[llvm-smoke] output: $out_smoke" >&2
  if ! echo "$out_smoke" | grep -q "Result: 3"; then
    echo "error: ny-llvm-smoke unexpected output: $out_smoke" >&2
    exit 1
  fi
else
  echo "[llvm-smoke] skipping ny-llvm-smoke (set NYASH_LLVM_ARRAY_SMOKE=1 to enable)" >&2
fi

# --- AOT smoke: apps/ny-array-llvm-ret (Array push/get, return value検証) ---
if [[ "${NYASH_LLVM_ARRAY_RET_SMOKE:-0}" == "1" ]] && [[ "${NYASH_DISABLE_PLUGINS:-0}" != "1" ]]; then
  echo "[llvm-smoke] building + linking apps/ny-array-llvm-ret ..." >&2
  # Ensure array plugin artifact exists (best-effort)
  if [[ -d plugins/nyash-array-plugin ]]; then
    (cd plugins/nyash-array-plugin && cargo build --release >/dev/null || true)
  fi
  OBJ_AR="$PWD/target/aot_objects/array_ret_smoke.o"
  rm -f "$OBJ_AR"
  NYASH_LLVM_OBJ_OUT="$OBJ_AR" "$BIN" --backend llvm apps/tests/ny-array-llvm-ret/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_AR" ./tools/build_llvm.sh apps/tests/ny-array-llvm-ret/main.nyash -o "$APP_BIN_DIR/app_array_ret_llvm" >/dev/null || true
  echo "[llvm-smoke] running app_array_ret_llvm ..." >&2
  out_ar=$("$APP_BIN_DIR/app_array_ret_llvm" || true)
  echo "[llvm-smoke] output: $out_ar" >&2
  if ! echo "$out_ar" | grep -q "Result: 3"; then
    echo "error: ny-array-llvm-ret unexpected output: $out_ar" >&2
    exit 1
  fi
  echo "[llvm-smoke] OK: Array return smoke passed" >&2
else
  echo "[llvm-smoke] skipping ny-array-llvm-ret (set NYASH_LLVM_ARRAY_RET_SMOKE=1 to enable)" >&2
fi

# --- AOT smoke: apps/ny-echo-lite (readLine -> print) ---
if [[ "${NYASH_LLVM_ECHO_SMOKE:-0}" == "1" ]]; then
  echo "[llvm-smoke] building + linking apps/ny-echo-lite ..." >&2
  OBJ_ECHO="$PWD/target/aot_objects/echo_smoke.o"
  rm -f "$OBJ_ECHO"
  NYASH_LLVM_OBJ_OUT="$OBJ_ECHO" "$BIN" --backend llvm apps/tests/ny-echo-lite/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_ECHO" ./tools/build_llvm.sh apps/tests/ny-echo-lite/main.nyash -o "$APP_BIN_DIR/app_echo_llvm" >/dev/null
  echo "[llvm-smoke] running app_echo_llvm with stdin ..." >&2
  echo "hello-llvm" | "$APP_BIN_DIR/app_echo_llvm" > /tmp/ny_echo_llvm.out || true
  read -r first_line < /tmp/ny_echo_llvm.out || true
  echo "[llvm-smoke] echo stdout first line: $first_line" >&2
  if ! grep -q "^Result:" /tmp/ny_echo_llvm.out; then
    echo "error: ny-echo-lite did not produce Result line (runtime)" >&2
    exit 1
  fi
  echo "[llvm-smoke] OK: AOT smokes passed (array/echo)" >&2
else
  echo "[llvm-smoke] skipping ny-echo-lite (set NYASH_LLVM_ECHO_SMOKE=1 to enable)" >&2
fi

# --- AOT smoke: apps/ny-map-llvm-smoke (Map by-id plugin path) ---
if [[ "${NYASH_LLVM_MAP_SMOKE:-0}" == "1" ]] && [[ "${NYASH_DISABLE_PLUGINS:-0}" != "1" ]]; then
  echo "[llvm-smoke] building + linking apps/ny-map-llvm-smoke ..." >&2
  # Try to build minimal required plugin if present
  if [[ -d plugins/nyash-map-plugin ]]; then
    (cd plugins/nyash-map-plugin && cargo build --release >/dev/null || true)
  fi
  # Pre-emit object to avoid current lowering gaps, then link
  OBJ_MAP="$PWD/target/aot_objects/map_smoke.o"
  rm -f "$OBJ_MAP"
  NYASH_LLVM_OBJ_OUT="$OBJ_MAP" "$BIN" --backend llvm apps/tests/ny-map-llvm-smoke/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_MAP" ./tools/build_llvm.sh apps/tests/ny-map-llvm-smoke/main.nyash -o "$APP_BIN_DIR/app_map_llvm" >/dev/null || true
  echo "[llvm-smoke] running app_map_llvm ..." >&2
  out_map=$("$APP_BIN_DIR/app_map_llvm" || true)
  echo "[llvm-smoke] output: $out_map" >&2
  if ! echo "$out_map" | grep -q "Map: v=42" || ! echo "$out_map" | grep -q "size=1"; then
    echo "error: ny-map-llvm-smoke unexpected output: $out_map" >&2
    exit 1
  fi
  echo "[llvm-smoke] OK: Map by-id plugin smoke passed" >&2
else
  echo "[llvm-smoke] skipping Map smoke (set NYASH_LLVM_MAP_SMOKE=1 to enable; requires plugins)" >&2
fi

# --- AOT smoke: apps/ny-vinvoke-smoke (variable-length invoke by-id) ---
if [[ "${NYASH_LLVM_VINVOKE_SMOKE:-0}" == "1" ]] && [[ "${NYASH_DISABLE_PLUGINS:-0}" != "1" ]]; then
  echo "[llvm-smoke] building + linking apps/ny-vinvoke-smoke ..." >&2
  if [[ -d plugins/nyash-map-plugin ]]; then
    (cd plugins/nyash-map-plugin && cargo build --release >/dev/null || true)
  fi
  OBJ_V="$PWD/target/aot_objects/vinvoke_smoke.o"
  rm -f "$OBJ_V"
  NYASH_LLVM_OBJ_OUT="$OBJ_V" "$BIN" --backend llvm apps/tests/ny-vinvoke-smoke/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_V" ./tools/build_llvm.sh apps/tests/ny-vinvoke-smoke/main.nyash -o "$APP_BIN_DIR/app_vinvoke_llvm" >/dev/null || true
  echo "[llvm-smoke] running app_vinvoke_llvm ..." >&2
  out_v=$("$APP_BIN_DIR/app_vinvoke_llvm" || true)
  echo "[llvm-smoke] output: $out_v" >&2
  if ! echo "$out_v" | grep -q "VInvokeRc: 42"; then
    echo "error: ny-vinvoke-smoke unexpected output: $out_v" >&2
    exit 1
  fi
  echo "[llvm-smoke] OK: variable-length by-id invoke smoke passed" >&2
else
  echo "[llvm-smoke] skipping VInvoke smoke (set NYASH_LLVM_VINVOKE_SMOKE=1 to enable; requires plugins)" >&2
fi

# --- AOT smoke: apps/ny-vinvoke-llvm-ret (variable-length invoke by-id, return value検証) ---
if [[ "${NYASH_LLVM_VINVOKE_RET_SMOKE:-0}" == "1" ]] && [[ "${NYASH_DISABLE_PLUGINS:-0}" != "1" ]]; then
  echo "[llvm-smoke] building + linking apps/ny-vinvoke-llvm-ret ..." >&2
  if [[ -d plugins/nyash-map-plugin ]]; then
    (cd plugins/nyash-map-plugin && cargo build --release >/dev/null || true)
  fi
  OBJ_VR="$PWD/target/aot_objects/vinvoke_ret_smoke.o"
  rm -f "$OBJ_VR"
  NYASH_LLVM_OBJ_OUT="$OBJ_VR" "$BIN" --backend llvm apps/tests/ny-vinvoke-llvm-ret/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_VR" ./tools/build_llvm.sh apps/tests/ny-vinvoke-llvm-ret/main.nyash -o "$APP_BIN_DIR/app_vinvoke_ret_llvm" >/dev/null || true
  echo "[llvm-smoke] running app_vinvoke_ret_llvm ..." >&2
  out_vr=$("$APP_BIN_DIR/app_vinvoke_ret_llvm" || true)
  echo "[llvm-smoke] output: $out_vr" >&2
  if ! echo "$out_vr" | grep -q "Result: 42"; then
    echo "error: ny-vinvoke-llvm-ret unexpected output: $out_vr" >&2
    exit 1
  fi
  echo "[llvm-smoke] OK: variable-length by-id invoke (return) smoke passed" >&2
else
  echo "[llvm-smoke] skipping VInvoke return smoke (set NYASH_LLVM_VINVOKE_RET_SMOKE=1 to enable; requires plugins)" >&2
fi

# --- AOT smoke: apps/ny-vinvoke-llvm-ret-size (fixed-length invoke by-id, size() return value検証) ---
if [[ "${NYASH_LLVM_VINVOKE_RET_SMOKE:-0}" == "1" ]] && [[ "${NYASH_DISABLE_PLUGINS:-0}" != "1" ]]; then
  echo "[llvm-smoke] building + linking apps/ny-vinvoke-llvm-ret-size ..." >&2
  if [[ -d plugins/nyash-map-plugin ]]; then
    (cd plugins/nyash-map-plugin && cargo build --release >/dev/null || true)
  fi
  OBJ_SIZE="$PWD/target/aot_objects/vinvoke_size_smoke.o"
  rm -f "$OBJ_SIZE"
  NYASH_LLVM_OBJ_OUT="$OBJ_SIZE" "$BIN" --backend llvm apps/tests/ny-vinvoke-llvm-ret-size/main.nyash >/dev/null || true

  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_SIZE" ./tools/build_llvm.sh apps/tests/ny-vinvoke-llvm-ret-size/main.nyash -o "$APP_BIN_DIR/app_vinvoke_ret_size_llvm" >/dev/null || true
  echo "[llvm-smoke] running app_vinvoke_ret_size_llvm ..." >&2
  out_size=$("$APP_BIN_DIR/app_vinvoke_ret_size_llvm" || true)
  echo "[llvm-smoke] output: $out_size" >&2
  if ! echo "$out_size" | grep -q "Result: 1"; then
    echo "error: ny-vinvoke-llvm-ret-size unexpected output: $out_size" >&2
    exit 1
  fi
  echo "[llvm-smoke] OK: fixed-length by-id invoke size() smoke passed" >&2
else
  echo "[llvm-smoke] skipping size() return smoke (included in NYASH_LLVM_VINVOKE_RET_SMOKE=1)" >&2
fi

# --- AOT smoke: plugin return values (CounterBox.get, StringBox.concat) ---
if [[ "${NYASH_LLVM_PLUGIN_RET_SMOKE:-0}" == "1" ]] && [[ "${NYASH_DISABLE_PLUGINS:-0}" != "1" ]]; then
  echo "[llvm-smoke] building + linking apps/ny-plugin-ret-llvm-smoke ..." >&2
  OBJ_RET="$PWD/target/aot_objects/plugin_ret_smoke.o"
  rm -f "$OBJ_RET"
  NYASH_LLVM_OBJ_OUT="$OBJ_RET" "$BIN" --backend llvm apps/tests/ny-plugin-ret-llvm-smoke/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_RET" ./tools/build_llvm.sh apps/tests/ny-plugin-ret-llvm-smoke/main.nyash -o "$APP_BIN_DIR/app_plugin_ret_llvm" >/dev/null || true
  echo "[llvm-smoke] running app_plugin_ret_llvm ..." >&2
  out_ret=$("$APP_BIN_DIR/app_plugin_ret_llvm" || true)
  echo "[llvm-smoke] output: $out_ret" >&2
  if ! echo "$out_ret" | grep -q "S=abCD" || ! echo "$out_ret" | grep -q "Result: 1"; then
    echo "error: plugin-ret-smoke unexpected output: $out_ret" >&2
    exit 1
  fi
  echo "[llvm-smoke] OK: plugin return (int/string) smoke passed" >&2
else
  echo "[llvm-smoke] skipping plugin return smoke (set NYASH_LLVM_PLUGIN_RET_SMOKE=1 to enable; requires plugins)" >&2
fi
