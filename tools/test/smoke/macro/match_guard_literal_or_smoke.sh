#!/usr/bin/env bash
set -u

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/match/guard_literal_or.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_ENABLE=1

# 1) AST expanded (no PeekExpr)
out_ast=$("$bin" --dump-expanded-ast-json "$src")
if [[ "$out_ast" == *'"kind":"PeekExpr"'* ]]; then
  echo "[FAIL] Expanded AST still contains PeekExpr for guard-literal-or" >&2
  exit 2
fi

# 2) Runtime output check via VM
export NYASH_VM_USE_PY=1
out_run=$("$bin" --backend vm "$src" | tr -d '\r')
# allow trailing newline to be trimmed by command substitution
if [ "$out_run" != "20" ] && [ "$out_run" != $'20\n' ]; then
  echo "[FAIL] VM run unexpected output. Got:" >&2
  printf '%s\n' "$out_run" >&2
  exit 2
fi

echo "[OK] match guard literal OR smoke passed"
