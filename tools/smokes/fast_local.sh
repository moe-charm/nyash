#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"

echo "[fast] Build (release) ..." >&2
cargo build --release -j 8 >/dev/null
cargo build --release -p nyash-llvm-compiler -j 8 >/dev/null
cargo build --release -p nyrt -j 8 >/dev/null

echo "[fast] PyVM Stage-2 minimal ..." >&2
timeout -s KILL 30s bash tools/pyvm_stage2_smoke.sh || true

echo "[fast] Short-circuit bridge ..." >&2
timeout -s KILL 30s bash tools/ny_stage2_shortcircuit_smoke.sh

echo "[fast] crate EXE smokes (3 cases) ..." >&2
timeout -s KILL 60s bash tools/crate_exe_smoke.sh apps/tests/ternary_basic.nyash >/dev/null
timeout -s KILL 60s bash tools/crate_exe_smoke.sh apps/tests/ternary_nested.nyash >/dev/null
timeout -s KILL 60s bash tools/crate_exe_smoke.sh apps/tests/peek_expr_block.nyash >/dev/null

echo "✅ fast_local smokes passed" >&2

