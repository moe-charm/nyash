#!/usr/bin/env bash
set -euo pipefail

root_dir="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root_dir"

viol=0

echo "[guard] Checking builder layer dependencies (origin→observe→rewrite)"

# Rule 1: origin/* must NOT import observe or rewrite
while IFS= read -r -d '' f; do
  if rg -n "use\\s+crate::mir::builder::(observe|rewrite)" "$f" >/dev/null; then
    echo "[ERROR] origin-layer import violation: $f"
    rg -n "use\\s+crate::mir::builder::(observe|rewrite)" "$f" || true
    viol=$((viol+1))
  fi
done < <(find src/mir/builder/origin -type f -name '*.rs' -print0 2>/dev/null || true)

# Rule 2: observe/* must NOT import rewrite or origin
while IFS= read -r -d '' f; do
  if rg -n "use\\s+crate::mir::builder::(rewrite|origin)" "$f" >/dev/null; then
    echo "[ERROR] observe-layer import violation: $f"
    rg -n "use\\s+crate::mir::builder::(rewrite|origin)" "$f" || true
    viol=$((viol+1))
  fi
done < <(find src/mir/builder/observe -type f -name '*.rs' -print0 2>/dev/null || true)

# Rule 3: rewrite/* must NOT import origin (observe is allowed)
while IFS= read -r -d '' f; do
  if rg -n "use\\s+crate::mir::builder::origin" "$f" >/dev/null; then
    echo "[ERROR] rewrite-layer import violation: $f"
    rg -n "use\\s+crate::mir::builder::origin" "$f" || true
    viol=$((viol+1))
  fi
done < <(find src/mir/builder/rewrite -type f -name '*.rs' -print0 2>/dev/null || true)

if [[ $viol -gt 0 ]]; then
  echo "[guard] FAILED: $viol violation(s) detected"
  exit 1
else
  echo "[guard] OK: No violations"
fi

