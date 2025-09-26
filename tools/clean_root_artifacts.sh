#!/usr/bin/env bash
set -euo pipefail

# Non-destructive cleaner for stray build artifacts in repo root.
# Dry-run by default; pass --apply to actually remove.

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
APPLY=0
[[ "${1:-}" == "--apply" ]] && APPLY=1

cd "$ROOT_DIR"

patterns=(
  "app_*"              # e.g., app_parity_*, app_stage3_loop, etc.
  "*_app"              # e.g., test_filebox_app
  "*.o"                # stray .o files
)

echo "[clean-root] scanning..."
found=()
for pat in "${patterns[@]}"; do
  while IFS= read -r -d $'\0' f; do
    # skip directories and keep inside tools/, src/, apps/, etc.
    [[ -d "$f" ]] && continue
    case "$f" in
      src/*|apps/*|examples/*|tools/*|docs/*|tests/*|crates/*|plugins/*) continue ;;
    esac
    found+=("$f")
  done < <(find . -maxdepth 1 -name "$pat" -print0 2>/dev/null || true)
done

if (( ${#found[@]} == 0 )); then
  echo "[clean-root] nothing to clean"
  exit 0
fi

printf "[clean-root] candidates (%d):\n" "${#found[@]}"
printf '  %s\n' "${found[@]}"

if (( APPLY )); then
  echo "[clean-root] removing..."
  rm -f -- "${found[@]}"
  echo "[clean-root] done"
else
  echo "[clean-root] dry-run (no files removed). Use --apply to remove."
fi

