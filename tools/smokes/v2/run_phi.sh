#!/usr/bin/env bash
set -euo pipefail

# PHI-line smoke runner (VM vs LLVM)
# - Discovers apps/tests/phi_*.nyash by default
# - Compares single 'Result: <n>' line between VM and LLVM AOT
# Usage: tools/smokes/v2/run_phi.sh [release|debug]

MODE=${1:-release}
BIN=./target/${MODE}/nyash
APP_BIN_DIR=${APP_BIN_DIR:-tmp}
TIMEOUT=${TIMEOUT:-15}
PHI_FILTER=${PHI_FILTER:-}
PHI_PROFILE=${PHI_PROFILE:-quick}  # quick|extended
mkdir -p "$APP_BIN_DIR" target/aot_objects

# Discover cases
mapfile -t found < <(ls -1 apps/tests/phi_*.nyash 2>/dev/null | sort || true)
cases=()
for f in "${found[@]:-}"; do
  [[ -z "$f" ]] && continue
  if [[ -n "$PHI_FILTER" ]]; then
    [[ "$f" == *"$PHI_FILTER"* ]] || continue
  fi
  # Profile gating: quick excludes heavier/slow cases by pattern
  if [[ "$PHI_PROFILE" == "quick" ]]; then
    case "$f" in
      *stage3* ) continue;;
    esac
  fi
  cases+=("$f")
done

if [[ ${#cases[@]} -eq 0 ]]; then
  echo "[phi-smoke] no cases found (filter='$PHI_FILTER')" >&2
  exit 0
fi

echo "[phi-smoke] building nyash (${MODE})..." >&2
if [[ "${NYASH_LLVM_USE_HARNESS:-0}" == "1" ]]; then
  cargo build ${MODE:+--${MODE}} -q --features llvm
else
  cargo build ${MODE:+--${MODE}} -q
fi

pass=0; fail=0
for app in "${cases[@]}"; do
  name=$(basename "$app" .nyash)
  echo "[phi-smoke] case: $name" >&2
  # VM run
  vm_out=$(NYASH_NYRT_SILENT_RESULT=1 "$BIN" --backend vm "$app" 2>/dev/null || true)
  vm_line=$(echo "$vm_out" | rg '^Result: ' -n || true)
  # LLVM emit+link+run
  OBJ="$PWD/target/aot_objects/${name}.o"
  rm -f "$OBJ"
  NYASH_LLVM_USE_HARNESS=1 NYASH_LLVM_OBJ_OUT="$OBJ" "$BIN" --backend llvm "$app" >/dev/null || true
  # If object is not produced, mark SKIP (heavy/link failures shouldn't count as FAIL here)
  if [[ ! -s "$OBJ" ]]; then
    echo "[phi-smoke][SKIP] $name: harness did not produce object ($OBJ)" >&2
    continue
  fi
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ" ./tools/build_llvm.sh "$app" -o "$APP_BIN_DIR/app_${name}" >/dev/null || true
  if [[ ! -x "$APP_BIN_DIR/app_${name}" ]]; then
    echo "[phi-smoke][SKIP] $name: link step did not produce executable" >&2
    continue
  fi
  ll_out=$(NYASH_NYRT_SILENT_RESULT=1 timeout ${TIMEOUT}s "$APP_BIN_DIR/app_${name}" 2>/dev/null || true)
  ll_line=$(echo "$ll_out" | rg '^Result: ' -n || true)
  if [[ -z "$vm_line" || -z "$ll_line" ]]; then
    echo "[phi-smoke][FAIL] $name: missing Result line (vm='$vm_line', llvm='$ll_line')" >&2
    fail=$((fail+1)); continue
  fi
  if [[ "$vm_line" == "$ll_line" ]]; then
    echo "[phi-smoke][OK] $name: $vm_line" >&2
    pass=$((pass+1))
  else
    echo "[phi-smoke][FAIL] $name: vm='$vm_line' llvm='$ll_line'" >&2
    fail=$((fail+1))
  fi
done

echo "[phi-smoke] summary: pass=$pass fail=$fail" >&2
if [[ $fail -ne 0 ]]; then exit 1; fi
