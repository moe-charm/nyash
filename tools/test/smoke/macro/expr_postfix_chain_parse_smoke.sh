#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_PARSER_STAGE3=1

tmp="$root/tmp/expr_postfix_chain_tmp.nyash"
cat > "$tmp" <<'SRC'
function main(args) {
  obj.m1().m2() catch { print("ok") }
}
SRC

# Expect parse success and run-time exit 0
"$bin" --backend vm "$tmp" >/dev/null 2>&1 && echo "[OK] postfix chain parse passed" && exit 0
echo "[FAIL] postfix chain parse failed" >&2
exit 2

