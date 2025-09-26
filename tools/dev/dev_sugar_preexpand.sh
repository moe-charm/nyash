#!/usr/bin/env bash
set -euo pipefail

# Dev sugar pre-expander (safe, zero-semantics changes)
# Applies, in order:
#  - @name[: Type] = expr  => local name[: Type] = expr
#  - elif cond {           => else if cond {
#  - a += b (and -=, *=, /=) as statements
#  - i++ / i-- as statements (not expressions)
# Usage:
#   tools/dev/dev_sugar_preexpand.sh < input.nyash > output.nyash
#   or tools/dev/dev_sugar_preexpand.sh input.nyash > output.nyash

in="${1:-}"
if [ -n "$in" ]; then
  exec <"$in"
fi

# 1) @ local alias
"$(dirname "$0")/at_local_preexpand.sh" | \

# 2) compound assignments (+=, -=, *=, /=), statements only; skip comment lines
sed -E \
  -e '/^[[:space:]]*\/\//b' \
  -e 's/^([[:space:]]*)([A-Za-z_][A-Za-z0-9_]*)[[:space:]]*\+=\s*(.+)$/\1\2 = \2 + \3/' \
  -e 's/^([[:space:]]*)([A-Za-z_][A-Za-z0-9_]*)[[:space:]]*\-=\s*(.+)$/\1\2 = \2 - \3/' \
  -e 's/^([[:space:]]*)([A-Za-z_][A-Za-z0-9_]*)[[:space:]]*\*=\s*(.+)$/\1\2 = \2 * \3/' \
  -e 's/^([[:space:]]*)([A-Za-z_][A-Za-z0-9_]*)[[:space:]]*\/=\s*(.+)$/\1\2 = \2 \/ \3/' | \

# 3) i++ / i-- as statements, skip comment lines
sed -E \
  -e '/^[[:space:]]*\/\//b' \
  -e 's/^([[:space:]]*)([A-Za-z_][A-Za-z0-9_]*)[[:space:]]*\+\+[[:space:]]*$/\1\2 = \2 + 1/' \
  -e 's/^([[:space:]]*)([A-Za-z_][A-Za-z0-9_]*)[[:space:]]*--[[:space:]]*$/\1\2 = \2 - 1/' | \

# 4) when cond { ... } → if cond { ... } (line-head only), skip comments
sed -E '/^[[:space:]]*\/\//b; s/^([[:space:]]*)when\b/\1if/' | \

# 5) print! expr → print(expr) (line-head only; allow zero or more spaces after '!')
sed -E '/^[[:space:]]*\/\//b; s/^([[:space:]]*)print![[:space:]]*(.+)$/\1print(\2)/'
