#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

start=$(date +%s)
OUT=$(mktemp)

echo "[papers] Running quick profile ..." >&2
if bash tools/smokes/v2/run.sh --profile quick >"$OUT" 2>&1; then
  status=0
else
  status=$?
fi

elapsed=$(( $(date +%s) - start ))
total=$(grep -E "^Profile: quick|^Total:|^Passed:|^Failed:" -n "$OUT" | sed -n 's/.*Total: \([0-9]\+\).*/\1/p' | tail -n1)
passed=$(grep -E "^Profile: quick|^Total:|^Passed:|^Failed:" -n "$OUT" | sed -n 's/.*Passed: \([0-9]\+\).*/\1/p' | tail -n1)
failed=$(grep -E "^Profile: quick|^Total:|^Passed:|^Failed:" -n "$OUT" | sed -n 's/.*Failed: \([0-9]\+\).*/\1/p' | tail -n1)

echo "{\"profile\":\"quick\",\"total\":${total:-0},\"passed\":${passed:-0},\"failed\":${failed:-0},\"elapsed_sec\":$elapsed,\"exit\":$status}" | tee /dev/stderr
rm -f "$OUT"
exit 0

