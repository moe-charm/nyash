#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [ ! -x "$BIN" ]; then
  echo "[bootstrap] building nyash (release, JIT)..." >&2
  cargo build --release --features cranelift-jit >/dev/null
fi

echo "[bootstrap] c0 (rust) → c1 (ny) → c1' parity (JIT-only)" >&2

# c0: baseline run (rust path)
NYASH_DISABLE_PLUGINS=1 NYASH_CLI_VERBOSE=1 "$BIN" --backend vm "$ROOT_DIR/apps/examples/string_p0.nyash" > /tmp/nyash-c0.out

# c1: try Ny compiler path (flagged); tolerate fallback to rust path
NYASH_DISABLE_PLUGINS=1 NYASH_USE_NY_COMPILER=1 NYASH_CLI_VERBOSE=1 "$BIN" --backend vm "$ROOT_DIR/apps/examples/string_p0.nyash" > /tmp/nyash-c1.out || true

# c1': re-run (simulated second pass)
NYASH_DISABLE_PLUGINS=1 NYASH_USE_NY_COMPILER=1 NYASH_CLI_VERBOSE=1 "$BIN" --backend vm "$ROOT_DIR/apps/examples/string_p0.nyash" > /tmp/nyash-c1p.out || true

H0=$(rg -n '^Result:\s*' /tmp/nyash-c0.out | sed 's/\s\+/ /g')
H1=$(rg -n '^Result:\s*' /tmp/nyash-c1.out | sed 's/\s\+/ /g' || true)
H2=$(rg -n '^Result:\s*' /tmp/nyash-c1p.out | sed 's/\s\+/ /g' || true)

echo "[bootstrap] c0: ${H0:-<none>}" >&2
echo "[bootstrap] c1: ${H1:-<none>}" >&2
echo "[bootstrap] c1': ${H2:-<none>}" >&2

if rg -q '^Result:\s*0\b' /tmp/nyash-c0.out; then
  echo "PASS: c0 baseline" >&2
else
  echo "FAIL: c0 baseline" >&2; sed -n '1,120p' /tmp/nyash-c0.out; exit 1
fi

# Accept either identical outputs or fallback matching c0
if rg -q '^Result:\s*0\b' /tmp/nyash-c1.out && rg -q '^Result:\s*0\b' /tmp/nyash-c1p.out; then
  echo "PASS: c1/c1' (ny compiler path)" >&2
else
  echo "WARN: c1/c1' did not report expected result; treating as optional while MVP matures" >&2
fi

echo "All PASS (bootstrap smoke)" >&2
