#!/usr/bin/env bash
set -euo pipefail

# Common helpers for tools/test

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
# Silence NyRT standardized result line in tests by default
export NYASH_NYRT_SILENT_RESULT=${NYASH_NYRT_SILENT_RESULT:-1}

msg() { echo "$*" >&2; }

require_cmd() { command -v "$1" >/dev/null 2>&1 || { msg "missing command: $1"; return 1; }; }

assert_exit() {
  local cmd=$1 expected=$2
  set +e
  bash -lc "$cmd"
  local code=$?
  set -e
  if [[ "$code" -ne "$expected" ]]; then
    msg "assert_exit failed: expected=$expected got=$code cmd=$cmd"
    return 1
  fi
}

assert_grep() {
  local pattern=$1; shift
  local text
  text=$(cat)
  echo "$text" | rg -q "$pattern" || { msg "assert_grep failed: pattern '$pattern'\n$text"; return 1; }
}

build_nyash_release() { (cd "$ROOT_DIR" && cargo build --release -j 8 >/dev/null); }
build_ny_llvmc() { (cd "$ROOT_DIR" && cargo build --release -p nyash-llvm-compiler -j 8 >/dev/null); }
build_nyrt() { (cd "$ROOT_DIR/crates/nyrt" && cargo build --release -j 8 >/dev/null); }

emit_json() { # args: src out_json
  "$ROOT_DIR/target/release/nyash" --emit-mir-json "$2" --backend mir "$1" >/dev/null
}

run_pyvm_json() { # args: json_path
  require_cmd python3
  python3 "$ROOT_DIR/tools/pyvm_runner.py" --in "$1"
}

build_exe_crate() { # args: in_json out_exe
  "$ROOT_DIR/target/release/ny-llvmc" --in "$1" --emit exe --nyrt "$ROOT_DIR/target/release" --out "$2" --harness "$ROOT_DIR/tools/llvmlite_harness.py"
}
