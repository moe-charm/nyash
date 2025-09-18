#!/usr/bin/env bash
set -euo pipefail

# Lints docs for deprecated surface keyword 'finally'.
# Allows occurrences under docs/archive and docs/private only (historical/papers).

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$ROOT_DIR"

violations=$(rg -n "\bfinally\b" docs \
  --glob '!docs/archive/**' --glob '!docs/private/**' \
  --glob '!docs/reference/ir/**' \
  --glob '!docs/reference/architecture/parser_mvp_stage3.md' \
  --glob '!docs/reference/language/LANGUAGE_REFERENCE_2025.md' \
  --glob '!docs/development/**' --glob '!docs/phases/**' --glob '!docs/papers/**' | wc -l | tr -d ' ')

if [[ "$violations" != "0" ]]; then
  echo "❌ docs lint: found deprecated 'finally' mentions outside archive/private ($violations hits)" >&2
  rg -n "\bfinally\b" docs \
    --glob '!docs/archive/**' --glob '!docs/private/**' \
    --glob '!docs/reference/ir/**' \
    --glob '!docs/reference/architecture/parser_mvp_stage3.md' \
    --glob '!docs/reference/language/LANGUAGE_REFERENCE_2025.md' \
    --glob '!docs/development/**' --glob '!docs/phases/**' --glob '!docs/papers/**'
  exit 1
fi

echo "✅ docs lint: no forbidden 'finally' mentions found." >&2
exit 0
