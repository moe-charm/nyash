#!/usr/bin/env bash
set -euo pipefail
[[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]] && set -x

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  (cd "$ROOT_DIR" && cargo build --release >/dev/null)
fi

TMP="$ROOT_DIR/tmp/cleanup_smoke"
mkdir -p "$TMP"

pass() { echo "✅ $1" >&2; }
fail() { echo "❌ $1" >&2; echo "$2" >&2; exit 1; }

# A) method-postfix cleanup: default forbids return
NAME_A="method_postfix_cleanup_forbid_return"
set +e
OUT_A=$(NYASH_METHOD_CATCH=1 NYASH_PARSER_STAGE3=1 "$BIN" --backend vm "$ROOT_DIR/apps/tests/method_postfix_finally_only.nyash" 2>&1)
CODE_A=$?
set -e
[[ "$CODE_A" != 0 ]] && pass "$NAME_A" || fail "$NAME_A" "$OUT_A"

# B) method-postfix cleanup: allow return returns 42
NAME_B="method_postfix_cleanup_allow_return"
set +e
OUT_B=$(NYASH_METHOD_CATCH=1 NYASH_PARSER_STAGE3=1 NYASH_CLEANUP_ALLOW_RETURN=1 "$BIN" --backend vm "$ROOT_DIR/apps/tests/method_postfix_finally_only.nyash" 2>&1)
CODE_B=$?
set -e
[[ "$CODE_B" == 42 ]] && pass "$NAME_B" || fail "$NAME_B" "$OUT_B"

# C) cleanup throw: default forbids throw
cat > "$TMP/cleanup_throw.nyash" << 'NYASH'
static box Main {
  main(args) {
    try {
      return 1
    } catch (e) {
      return 2
    } cleanup {
      throw 9
    }
  }
}
NYASH
NAME_C="cleanup_forbid_throw"
set +e
OUT_C=$(NYASH_PARSER_STAGE3=1 "$BIN" --backend vm "$TMP/cleanup_throw.nyash" 2>&1)
CODE_C=$?
set -e
[[ "$CODE_C" != 0 ]] && pass "$NAME_C" || fail "$NAME_C" "$OUT_C"

# D) cleanup throw: allow throw passes through (program returns 0 via default path here)
NAME_D="cleanup_allow_throw"
set +e
OUT_D=$(NYASH_PARSER_STAGE3=1 NYASH_CLEANUP_ALLOW_THROW=1 "$BIN" --backend vm "$TMP/cleanup_throw.nyash" 2>&1)
CODE_D=$?
set -e
[[ "$CODE_D" == 0 ]] && pass "$NAME_D" || fail "$NAME_D" "$OUT_D"

echo "All cleanup smokes PASS" >&2
exit 0
