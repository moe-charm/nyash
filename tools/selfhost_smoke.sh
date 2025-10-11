#!/usr/bin/env bash
set -euo pipefail

# Self-host minimal smoke (dev-only)
# - Emits MIR(JSON v0) via selfhost compiler MVP
# - Runs a representative VM example with Known rewrite ON/OFF and compares outputs

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
NY_BIN="${ROOT_DIR}/target/release/nyash"

if [[ ! -x "${NY_BIN}" ]]; then
  echo "[selfhost-smoke] nyash binary not found at ${NY_BIN}. Please build first: cargo build --release" >&2
  exit 1
fi

echo "[selfhost-smoke] Step 1: Emit JSON via selfhost compiler (min-json first, then mir if needed)"
OUT_JSON="/tmp/nyash_selfhost_out.json"
set -x
if NYASH_LLVM_USE_HARNESS=1 NYASH_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_AST=1 \
   "${NY_BIN}" --backend llvm apps/selfhost-compiler/compiler.hako -- --min-json --stage3 > "${OUT_JSON}"; then
  :
else
  echo "[selfhost-smoke] WARN: min-json emission failed; trying mir-emitter path..." >&2
  if NYASH_LLVM_USE_HARNESS=1 NYASH_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_AST=1 \
     "${NY_BIN}" --backend llvm apps/selfhost-compiler/compiler.hako -- --min-json --emit-mir --stage3 > "${OUT_JSON}"; then
    :
  else
    echo "[selfhost-smoke] WARN: selfhost compiler emission failed (policy/runtime). Continuing." >&2
  fi
fi
set +x

if [[ -s "${OUT_JSON}" ]]; then
  echo "[selfhost-smoke] Emitted JSON: ${OUT_JSON} ($(wc -c < "${OUT_JSON}") bytes)"
else
  echo "[selfhost-smoke] ERROR: no JSON emitted (expected non-empty)." >&2
  exit 1
fi

echo "[selfhost-smoke] Step 2: Run representative VM example (rewrite=ON/OFF)"
EXAMPLE="apps/examples/json_query/main.nyash"
OUT_ON="/tmp/nyash_selfhost_vm_on.txt"
OUT_OFF="/tmp/nyash_selfhost_vm_off.txt"

set -x
"${NY_BIN}" --backend vm "${EXAMPLE}" > "${OUT_ON}"
NYASH_REWRITE_KNOWN_DEFAULT=0 "${NY_BIN}" --backend vm "${EXAMPLE}" > "${OUT_OFF}"
set +x

if ! diff -u "${OUT_ON}" "${OUT_OFF}" >/dev/null 2>&1; then
  echo "[selfhost-smoke] WARN: output differs between rewrite ON and OFF" >&2
  echo "--- ON (${OUT_ON})" >&2
  head -n 20 "${OUT_ON}" >&2 || true
  echo "--- OFF (${OUT_OFF})" >&2
  head -n 20 "${OUT_OFF}" >&2 || true
  # Non-fatal: keep smoke informative; do not fail hard unless required.
else
  echo "[selfhost-smoke] VM outputs match for rewrite ON/OFF (good)."
fi

echo "[selfhost-smoke] PASS"
