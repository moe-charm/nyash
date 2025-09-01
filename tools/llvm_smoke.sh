#!/usr/bin/env bash
set -euo pipefail

# LLVM Phase 11.2 smoke test
# - Builds nyash with LLVM backend
# - Compiles example via --backend llvm (object emission inside)
# - Verifies object file presence and non-zero size

MODE=${1:-release}
BIN=./target/${MODE}/nyash
# Fixed object output directory for stability
mkdir -p target/aot_objects
OBJ="$PWD/target/aot_objects/core_smoke.o"

if ! command -v llvm-config-18 >/dev/null 2>&1; then
  echo "error: llvm-config-18 not found. Please install LLVM 18 dev packages." >&2
  exit 2
fi

echo "[llvm-smoke] building nyash (${MODE}, feature=llvm)..." >&2
# Support both llvm-sys 180/181 by exporting both prefixes to the same value
_LLVMPREFIX=$(llvm-config-18 --prefix)
LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" cargo build -q ${MODE:+--${MODE}} --features llvm

echo "[llvm-smoke] running --backend llvm on examples/llvm11_core_smoke.nyash ..." >&2
rm -f "$OBJ"
NYASH_LLVM_OBJ_OUT="$OBJ" LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" "$BIN" --backend llvm examples/llvm11_core_smoke.nyash >/dev/null || true

if [[ ! -f "$OBJ" ]]; then
  echo "error: expected object not found: $OBJ" >&2
  exit 1
fi
if [[ ! -s "$OBJ" ]]; then
  echo "error: object is empty: $OBJ" >&2
  exit 1
fi

echo "[llvm-smoke] OK: object generated: $OBJ ($(stat -c%s "$OBJ") bytes)" >&2

# --- AOT smoke: apps/ny-llvm-smoke (Array get/set/print) ---
if [[ "${NYASH_LLVM_ARRAY_SMOKE:-0}" == "1" ]]; then
  echo "[llvm-smoke] building + linking apps/ny-llvm-smoke ..." >&2
  # Pre-emit object explicitly (more stable)
  OBJ_ARRAY="$PWD/target/aot_objects/array_smoke.o"
  rm -f "$OBJ_ARRAY"
  NYASH_LLVM_ALLOW_BY_NAME=1 NYASH_LLVM_OBJ_OUT="$OBJ_ARRAY" LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" "$BIN" --backend llvm apps/tests/ny-llvm-smoke/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_ARRAY" ./tools/build_llvm.sh apps/tests/ny-llvm-smoke/main.nyash -o app_link >/dev/null
  echo "[llvm-smoke] running app_link ..." >&2
  out_smoke=$(./app_link || true)
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
  NYASH_LLVM_ALLOW_BY_NAME=1 NYASH_LLVM_OBJ_OUT="$OBJ_AR" LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" "$BIN" --backend llvm apps/tests/ny-array-llvm-ret/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_AR" ./tools/build_llvm.sh apps/tests/ny-array-llvm-ret/main.nyash -o app_array_ret_llvm >/dev/null || true
  echo "[llvm-smoke] running app_array_ret_llvm ..." >&2
  out_ar=$(./app_array_ret_llvm || true)
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
  NYASH_LLVM_ALLOW_BY_NAME=1 NYASH_LLVM_OBJ_OUT="$OBJ_ECHO" LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" "$BIN" --backend llvm apps/tests/ny-echo-lite/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_ECHO" ./tools/build_llvm.sh apps/tests/ny-echo-lite/main.nyash -o app_echo_llvm >/dev/null
  echo "[llvm-smoke] running app_echo_llvm with stdin ..." >&2
  echo "hello-llvm" | ./app_echo_llvm > /tmp/ny_echo_llvm.out || true
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
  NYASH_LLVM_ALLOW_BY_NAME=1 NYASH_LLVM_OBJ_OUT="$OBJ_MAP" LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" "$BIN" --backend llvm apps/tests/ny-map-llvm-smoke/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_MAP" ./tools/build_llvm.sh apps/tests/ny-map-llvm-smoke/main.nyash -o app_map_llvm >/dev/null || true
  echo "[llvm-smoke] running app_map_llvm ..." >&2
  out_map=$(./app_map_llvm || true)
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
  NYASH_LLVM_ALLOW_BY_NAME=1 NYASH_LLVM_OBJ_OUT="$OBJ_V" LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" "$BIN" --backend llvm apps/tests/ny-vinvoke-smoke/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_V" ./tools/build_llvm.sh apps/tests/ny-vinvoke-smoke/main.nyash -o app_vinvoke_llvm >/dev/null || true
  echo "[llvm-smoke] running app_vinvoke_llvm ..." >&2
  out_v=$(./app_vinvoke_llvm || true)
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
  NYASH_LLVM_ALLOW_BY_NAME=1 NYASH_LLVM_OBJ_OUT="$OBJ_VR" LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" "$BIN" --backend llvm apps/tests/ny-vinvoke-llvm-ret/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_VR" ./tools/build_llvm.sh apps/tests/ny-vinvoke-llvm-ret/main.nyash -o app_vinvoke_ret_llvm >/dev/null || true
  echo "[llvm-smoke] running app_vinvoke_ret_llvm ..." >&2
  out_vr=$(./app_vinvoke_ret_llvm || true)
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
  NYASH_LLVM_ALLOW_BY_NAME=1 NYASH_LLVM_OBJ_OUT="$OBJ_SIZE" LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" "$BIN" --backend llvm apps/tests/ny-vinvoke-llvm-ret-size/main.nyash >/dev/null || true
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ_SIZE" ./tools/build_llvm.sh apps/tests/ny-vinvoke-llvm-ret-size/main.nyash -o app_vinvoke_ret_size_llvm >/dev/null || true
  echo "[llvm-smoke] running app_vinvoke_ret_size_llvm ..." >&2
  out_size=$(./app_vinvoke_ret_size_llvm || true)
  echo "[llvm-smoke] output: $out_size" >&2
  if ! echo "$out_size" | grep -q "Result: 1"; then
    echo "error: ny-vinvoke-llvm-ret-size unexpected output: $out_size" >&2
    exit 1
  fi
  echo "[llvm-smoke] OK: fixed-length by-id invoke size() smoke passed" >&2
else
  echo "[llvm-smoke] skipping size() return smoke (included in NYASH_LLVM_VINVOKE_RET_SMOKE=1)" >&2
fi
