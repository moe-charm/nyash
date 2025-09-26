#!/usr/bin/env bash
set -euo pipefail

# Pre-expand dev sugar: line-head "@name[: Type] = expr" -> "local name[: Type] = expr"
# Usage:
#   tools/dev/at_local_preexpand.sh < input.nyash > output.nyash
#   or tools/dev/at_local_preexpand.sh input.nyash > output.nyash

in="${1:-}"
if [ -n "$in" ]; then
  exec <"$in"
fi

sed -E 's/^([[:space:]]*)@([A-Za-z_][A-Za-z0-9_]*)([[:space:]]*:[[:space:]]*[A-Za-z_][A-Za-z0-9_]*)?[[:space:]]*=/\1local \2\3 =/'
