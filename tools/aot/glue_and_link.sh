#!/usr/bin/env bash
set -euo pipefail

# Generate ny_main glue and link with NyRT + given objects
# Usage examples:
#   tools/aot/glue_and_link.sh --ret 0 -o build/glue_ret0
#   tools/aot/glue_and_link.sh --concat-len hi yo -o build/glue_hiyo
#   tools/aot/glue_and_link.sh --ret 42 -o build/app build/obj/program.o

if ! command -v clang >/dev/null 2>&1; then
  echo "[link][ERROR] clang not found (sudo apt-get install -y clang)" >&2
  exit 2
fi

OUT=build/glue_app
SPECS=()
OBJS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    -o|--out) OUT="$2"; shift 2 ;;
    --ret|--concat-len) SPECS+=("$1" "$2" ${3:-}); if [ "$1" = "--concat-len" ]; then shift 3; else shift 2; fi ;;
    --array-len|--map-size) SPECS+=("$1" "$2"); shift 2 ;;
    *) OBJS+=("$1"); shift ;;
  esac
done

mkdir -p "$(dirname "$OUT")"

if [ ${#SPECS[@]} -eq 0 ]; then
  echo "[link][ERROR] must specify one of: --ret N | --concat-len A B | --array-len N | --map-size N" >&2
  exit 2
fi

# Generate glue
case "${SPECS[0]}" in
  --ret)         bash tools/aot/gen_ny_main.sh --ret "${SPECS[1]}" ;;
  --concat-len)  bash tools/aot/gen_ny_main.sh --concat-len "${SPECS[1]}" "${SPECS[2]}" ;;
  --array-len)   bash tools/aot/gen_ny_main.sh --array-len "${SPECS[1]}" ;;
  --map-size)    bash tools/aot/gen_ny_main.sh --map-size "${SPECS[1]}" ;;
esac

# Link with NyRT and optional extra objects
NYRT_A=target/release/libhako_kernel.a
if [ ! -f "$NYRT_A" ]; then
  echo "[link][WARN] $NYRT_A not found; attempting link without NyRT" >&2
  tools/aot/link_with_clang.sh -o "$OUT" build/ny_main_glue.o ${OBJS[@]:+"${OBJS[@]}"}
else
  tools/aot/link_with_clang.sh -o "$OUT" build/ny_main_glue.o ${OBJS[@]:+"${OBJS[@]}"} --nyrt "$NYRT_A"
fi
echo "[link] done: $OUT" >&2
