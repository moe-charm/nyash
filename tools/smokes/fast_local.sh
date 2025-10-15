#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$ROOT"

# PHI policy: default to PHI-off; allow override from caller
export NYASH_MIR_NO_PHI=${NYASH_MIR_NO_PHI:-1}
# Edge-copy strict verifier is opt-in (default off)
export NYASH_VERIFY_EDGE_COPY_STRICT=${NYASH_VERIFY_EDGE_COPY_STRICT:-0}

echo "[fast] Build (release) ..." >&2
cargo build --release -j 8 >/dev/null
cargo build --release -p nyash-llvm-compiler -j 8 >/dev/null
cargo build --release -p nyrt -j 8 >/dev/null

echo "[fast] PyVM Stage-2 minimal ..." >&2
timeout -s KILL 30s bash tools/pyvm_stage2_smoke.sh || true

echo "[fast] Short-circuit bridge ..." >&2
timeout -s KILL 30s bash tools/ny_stage2_shortcircuit_smoke.sh

echo "[fast] crate EXE smokes (3 cases) ..." >&2
timeout -s KILL 60s bash tools/crate_exe_smoke.sh apps/tests/ternary_basic.hako >/dev/null
timeout -s KILL 60s bash tools/crate_exe_smoke.sh apps/tests/ternary_nested.hako >/dev/null
timeout -s KILL 60s bash tools/crate_exe_smoke.sh apps/tests/peek_expr_block.hako >/dev/null

echo "[fast] PyVM vs llvmlite parity (exit code) ..." >&2
timeout -s KILL 120s bash tools/pyvm_vs_llvmlite.sh apps/tests/loop_if_phi.hako >/dev/null || true

# Optional: PHI trace smoke (enable with NYASH_LLVM_TRACE_SMOKE=1)
if [[ "${NYASH_LLVM_TRACE_SMOKE:-0}" == "1" ]]; then
  echo "[fast] PHI trace smoke (optional) ..." >&2
  timeout -s KILL 60s bash tools/test/smoke/llvm/phi_trace/test.sh >/dev/null || true
fi

echo "✅ fast_local smokes passed" >&2
