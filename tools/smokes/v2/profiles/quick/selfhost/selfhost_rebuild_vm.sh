#!/bin/bash
# selfhost_rebuild_vm.sh — M2: rebuild compiler source with selfhost compiler EXE and sanity-check JSON

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

ensure_hako_toml

# Ensure EXE exists (reuse M1 path/policy)
APP_SRC="${NYASH_ROOT}/apps/selfhost-compiler/compiler.hako"
OUT_EXE="/tmp/hako_selfhost_compiler"
if [ ! -x "$OUT_EXE" ]; then
KERNEL="${NYASH_ROOT}/crates/hako_kernel/target/release/libhako_kernel.a"
if [ ! -f "$KERNEL" ]; then
  echo "SKIP: selfhost_rebuild_vm (kernel missing)" >&2
  exit 0
fi

  if ! command -v llvm-config-18 >/dev/null 2>&1; then
    echo "SKIP: selfhost_rebuild_vm (llvm-config-18 not found; EXE not present)" >&2
    exit 0
  fi
  log_info "Building selfhost compiler EXE (bootstrap) ..."
  "${NYASH_ROOT}/tools/build_llvm.sh" "$APP_SRC" -o "$OUT_EXE" >/dev/null 2>&1 || true
  if [ ! -x "$OUT_EXE" ]; then
    log_error "build_llvm.sh did not produce an executable at $OUT_EXE"
    exit 1
  fi
fi

log_info "Running selfhost compiler EXE on its own source (Stage-1 JSON) ..."
# Run without --min-json to force parse+emit path (sanity of read_all/parse/emit)
JSON_OUT=$("$OUT_EXE" "$APP_SRC" 2>/dev/null | head -n 1 | tr -d '\r' | xargs)
if [ -z "$JSON_OUT" ]; then
  log_error "no output from selfhost compiler EXE"
  exit 1
fi
echo "$JSON_OUT" | grep -q '\"kind\":\"Program\"' || {
  log_error "first line does not include kind:\"Program\": $JSON_OUT"
  exit 1
}

log_success "selfhost M2 passed (self-rebuild emits Stage-1 JSON)"
exit 0

