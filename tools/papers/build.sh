#!/usr/bin/env bash
set -euo pipefail
# Build Nyash papers (Markdown -> PDF/LaTeX)
# Outputs to docs/private/out/
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
BASE="$ROOT/nyash"
OUT_DIR="$BASE/docs/private/out"
PANDOC=${PANDOC:-$(command -v pandoc || true)}
JP_HEADER="$BASE/docs/private/papers/pandoc/jp_header.tex"
need() { command -v "$1" >/dev/null 2>&1 || { echo "error: missing command: $1" >&2; exit 1; }; }
if [[ -z "$PANDOC" ]]; then echo "error: pandoc not found" >&2; exit 1; fi
mkdir -p "$OUT_DIR"
run_pandoc_jp() {
  local in_md="$1" name="$2"
  [[ -f "$in_md" ]] || { echo "[skip] not found: $in_md"; return 0; }
  need lualatex
  echo "[JP] $name ← $in_md"
  local dir; dir=$(dirname "$in_md"); local file; file=$(basename "$in_md")
  ( cd "$dir" && "$PANDOC" --pdf-engine=lualatex --metadata=lang:ja --from markdown --toc --include-in-header="$JP_HEADER" -o "$OUT_DIR/${name}.pdf" "$file" )
  ( cd "$dir" && "$PANDOC" -s "$file" -t latex --include-in-header="$JP_HEADER" -o "$OUT_DIR/${name}.tex" )
}
run_pandoc_en() {
  local in_md="$1" name="$2"
  [[ -f "$in_md" ]] || { echo "[skip] not found: $in_md"; return 0; }
  echo "[EN] $name ← $in_md"
  local dir; dir=$(dirname "$in_md"); local file; file=$(basename "$in_md")
  ( cd "$dir" && "$PANDOC" --pdf-engine=pdflatex --from markdown --toc -o "$OUT_DIR/${name}.pdf" "$file" )
  ( cd "$dir" && "$PANDOC" -s "$file" -t latex -o "$OUT_DIR/${name}.tex" )
}
A_JP="$BASE/docs/private/papers/paper-a-mir13-ir-design/main-paper-jp.md"
A_EN="$BASE/docs/private/papers/paper-a-mir13-ir-design/main-paper.md"
B_JP="$BASE/docs/private/papers/paper-b-nyash-execution-model/main-paper-jp.md"
B_EN="$BASE/docs/private/papers/paper-b-nyash-execution-model/main-paper.md"
case "${1:-all}" in
  a-jp) run_pandoc_jp "$A_JP" paper-a-jp ;;
  a-en) run_pandoc_en "$A_EN" paper-a-en ;;
  b-jp) run_pandoc_jp "$B_JP" paper-b-jp ;;
  b-en) run_pandoc_en "$B_EN" paper-b-en ;;
  all)
    run_pandoc_jp "$A_JP" paper-a-jp
    run_pandoc_en "$A_EN" paper-a-en
    run_pandoc_jp "$B_JP" paper-b-jp
    run_pandoc_en "$B_EN" paper-b-en
    ;;
  *) echo "Usage: $0 [a-jp|a-en|b-jp|b-en|all]" >&2; exit 2 ;;
esac
echo "[out] $OUT_DIR"
