#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
LIB="$ROOT/tools/test/lib/shlib.sh"
if [[ ! -f "$LIB" ]]; then echo "lib/shlib.sh not found" >&2; exit 2; fi
source "$LIB"

TAG="all"
if [[ "${1:-}" == "--tag" && "${2:-}" != "" ]]; then TAG="$2"; shift 2; fi

# Discover tests
mapfile -t TESTS < <(find "$ROOT/tools/test" -type f -path '*/test.sh' | sort)
[[ ${#TESTS[@]} -eq 0 ]] && { echo "no tests found" >&2; exit 2; }

ok=0; fail=0; skip=0

for t in "${TESTS[@]}"; do
  case "$TAG" in
    fast)
      # Very small subset: crate-exe, bridge shortcircuit, and tiny LLVM checks
      if [[ "$t" != *"/smoke/crate-exe/"* && "$t" != *"/smoke/bridge/"* && "$t" != *"/smoke/llvm/quick/"* && "$t" != *"/smoke/llvm/ifmerge/"* && "$t" != *"/smoke/python/unit/"* ]]; then
        echo "[SKIP] $t"; skip=$((skip+1)); continue
      fi
      ;;
    all) ;;
    *) ;;
  esac
  echo "[RUN ] $t"
  if ( cd "$(dirname "$t")" && bash ./test.sh ); then
    echo "[ OK ] $t"
    ok=$((ok+1))
  else
    echo "[FAIL] $t"
    fail=$((fail+1))
  fi
done

echo "Summary: ok=$ok fail=$fail skip=$skip"
exit $([[ $fail -eq 0 ]] && echo 0 || echo 1)
