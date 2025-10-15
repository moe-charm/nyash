#!/bin/bash
# selfhost_bootstrap_llvm.sh — M1: build selfhost compiler to LLVM EXE and sanity-run

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

# Skip gracefully when LLVM toolchain is not available
if ! command -v llvm-config-18 >/dev/null 2>&1; then
  echo "SKIP: selfhost_bootstrap_llvm (llvm-config-18 not found)" >&2
  exit 0
fi

ensure_hako_toml

APP_SRC="${NYASH_ROOT}/apps/selfhost-compiler/compiler.hako"
OUT_EXE="/tmp/hako_selfhost_compiler"

log_info "Building selfhost compiler to EXE via LLVM harness ..."
set -x
"${NYASH_ROOT}/tools/build_llvm.sh" "$APP_SRC" -o "$OUT_EXE" >/dev/null 2>&1 || true
set +x

if [ ! -x "$OUT_EXE" ]; then
  log_error "build_llvm.sh did not produce an executable at $OUT_EXE"
  exit 1
fi

log_info "Running selfhost compiler EXE (expect JSON header) ..."
HEAD_LINE=$("$OUT_EXE" -- --min-json 2>/dev/null | head -n1 | tr -d '\r' | xargs)
if [ -z "$HEAD_LINE" ]; then
  log_error "no output from selfhost compiler EXE"
  exit 1
fi
echo "$HEAD_LINE" | grep -E -q '\{"version":|\"kind\":' || {
  log_error "first line does not look like JSON header: $HEAD_LINE"
  exit 1
}

log_success "selfhost bootstrap M1 passed (EXE built and header emitted)"
exit 0
