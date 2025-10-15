#!/usr/bin/env bash
set -euo pipefail
# SSOT validator for specs/type_registry.toml
# - Ensures no method name maps to multiple different slots within a type
# - Warns on exact duplicate entries for (name, slot)

SPEC=${1:-specs/type_registry.toml}
if [ ! -f "$SPEC" ]; then
  echo "SSOT: not found: $SPEC" >&2
  exit 2
fi

current=""
declare -A seen_name_slot # key: type|name -> slot
declare -A dup_exact      # key: type|name|slot -> count
ok=0; err=0

while IFS= read -r line; do
  s=$(echo "$line" | sed -E 's/[[:space:]]+//g')
  [[ -z "$s" || "$s" =~ ^# ]] && continue
  if [[ "$s" =~ ^\[type\.(.+)\]$ ]]; then
    current=${BASH_REMATCH[1]}
    continue
  fi
  if [[ "$s" =~ ^\{name=\"([^\"]+)\",arities=\[[^\]]*\],slot=([0-9]+)\},?$ ]]; then
    name=${BASH_REMATCH[1]}
    slot=${BASH_REMATCH[2]}
    key="$current|$name"
    k2="$current|$name|$slot"
    if [[ -n "${seen_name_slot[$key]:-}" && "${seen_name_slot[$key]}" != "$slot" ]]; then
      echo "SSOT ERROR: $current.$name maps to multiple slots: ${seen_name_slot[$key]} and $slot" >&2
      ((err++))
    else
      seen_name_slot[$key]="$slot"
    fi
    dup_exact[$k2]=$(( ${dup_exact[$k2]:-0} + 1 ))
  fi
done < "$SPEC"

for k in "${!dup_exact[@]}"; do
  c=${dup_exact[$k]}
  if (( c > 1 )); then
    IFS='|' read -r ty nm sl <<< "$k"
    echo "SSOT WARN: duplicate entries for $ty.$nm (slot=$sl) count=$c" >&2
  fi
done

if (( err > 0 )); then
  echo "SSOT validation: FAIL ($err errors)" >&2
  exit 1
else
  echo "SSOT validation: OK" >&2
fi

