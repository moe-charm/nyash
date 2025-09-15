#!/usr/bin/env bash
set -euo pipefail
set +H  # disable history expansion to allow '!'

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  echo "[build] nyash (release) ..." >&2
  (cd "$ROOT_DIR" && cargo build --release >/dev/null)
fi

TMP_DIR="$ROOT_DIR/tmp"
mkdir -p "$TMP_DIR"

pass() { echo "✅ $1" >&2; }
fail() { echo "❌ $1" >&2; echo "$2" | sed -n '1,160p' >&2; exit 1; }

run_case() {
  local program="$1" want="$2" name="$3"
  local f="$TMP_DIR/stage2_cmp_tmp.ny" out code
  printf '%s\n' "$program" > "$f"
  out=$(NYASH_VM_USE_PY=1 "$BIN" --backend vm "$f" 2>&1) || code=$?
  code=${code:-0}
  [[ "$code" -eq "$want" ]] && pass "$name" || fail "$name" "__EXIT_CODE__=$code"
}

run_case $'static box Main {\n  main(args){\n    return (2==2)\n  }\n}' 1 "compare =="
run_case $'static box Main {\n  main(args){\n    return (2!=2)\n  }\n}' 0 "compare !="
run_case $'static box Main {\n  main(args){\n    return (2<3)\n  }\n}' 1 "compare <"
run_case $'static box Main {\n  main(args){\n    return (3<2)\n  }\n}' 0 "compare < false"
run_case $'static box Main {\n  main(args){\n    return (2<=2)\n  }\n}' 1 "compare <="
run_case $'static box Main {\n  main(args){\n    return (3>2)\n  }\n}' 1 "compare >"
run_case $'static box Main {\n  main(args){\n    return (2>=2)\n  }\n}' 1 "compare >="

echo "All Stage-2 compare smokes (PyVM) PASS" >&2
