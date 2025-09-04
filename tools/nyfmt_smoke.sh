#!/usr/bin/env bash
set -euo pipefail

echo "[nyfmt-smoke] NYFMT_POC=${NYFMT_POC:-}"
echo "[nyfmt-smoke] PoC placeholder (no runtime changes). Shows docs and examples."

if [[ "${NYFMT_POC:-}" == "1" ]]; then
  echo "[nyfmt-smoke] Running PoC guidance..."
  echo "- Read: docs/tools/nyfmt/NYFMT_POC_ROADMAP.md"
  echo "- Mapping: docs/development/roadmap/phases/phase-12.7/ancp-specs/ANCP-Reversible-Mapping-v1.md"
  if [[ -d "apps/nyfmt-poc" ]]; then
    echo "- Examples found under apps/nyfmt-poc/ (documentation only)"
    ls -1 apps/nyfmt-poc | sed 's/^/  * /'
    echo ""
    echo "Example triad (Before/Canonical/Round-Trip) hints are in each file comments."
  else
    echo "- No examples directory yet (create apps/nyfmt-poc/ to try snippets)"
  fi
else
  echo "[nyfmt-smoke] Set NYFMT_POC=1 to enable PoC guidance output."
fi

echo "[nyfmt-smoke] Done."
