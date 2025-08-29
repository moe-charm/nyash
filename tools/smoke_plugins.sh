#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  echo "[smoke] building nyash (release, cranelift-jit)..." >&2
  (cd "$ROOT_DIR" && cargo build --release --features cranelift-jit >/dev/null)
fi

# Build required plugins (release)
build_plugin() {
  local dir=$1
  if [[ -d "$ROOT_DIR/$dir" ]]; then
    echo "[smoke] building $dir ..." >&2
    (cd "$ROOT_DIR/$dir" && cargo build --release >/dev/null)
  fi
}

build_plugin plugins/nyash-python-plugin
build_plugin plugins/nyash-integer-plugin
build_plugin plugins/nyash-console-plugin
build_plugin plugins/nyash-math-plugin

export NYASH_CLI_VERBOSE=1
export NYASH_PLUGIN_STRICT=1
export NYASH_USE_PLUGIN_BUILTINS=1
export NYASH_PLUGIN_OVERRIDE_TYPES="ArrayBox,MapBox,ConsoleBox"

run_case() {
  local name=$1
  local file=$2
  echo "[smoke] case=$name file=$file" >&2
  "$BIN" --backend vm "$ROOT_DIR/$file" >/dev/null
  echo "[smoke] ok: $name" >&2
}

# Core plugin demos
run_case py_math_sqrt_demo examples/py_math_sqrt_demo.nyash
run_case integer_plugin_demo examples/integer_plugin_demo.nyash
run_case console_demo examples/console_demo.nyash
run_case math_time_demo examples/math_time_demo.nyash

echo "[smoke] all green" >&2

# Second pass: disable builtins and re-run key cases
echo "[smoke] second pass with NYASH_DISABLE_BUILTINS=1" >&2
NYASH_DISABLE_BUILTINS=1 \
  "$BIN" --backend vm "$ROOT_DIR/examples/console_demo.nyash" >/dev/null
NYASH_DISABLE_BUILTINS=1 \
  "$BIN" --backend vm "$ROOT_DIR/examples/math_time_demo.nyash" >/dev/null
echo "[smoke] all green (builtins disabled)" >&2
