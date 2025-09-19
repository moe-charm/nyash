#!/usr/bin/env bash
set -euo pipefail

me_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$me_dir/../../../.." && pwd)
bin="$repo_root/target/release/nyash"

file="${1:-apps/tests/macro_test_args.nyash}"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_ENABLE=1

args='{
  "test_top_level": [ {"i":1}, {"s":"x"} ],
  "B.test_static": [ 2 ],
  "B.test_instance": { "args": [ {"s":"y"} ], "instance": { "ctor": "new" } }
}'

NYASH_TEST_ARGS_JSON="$args" \
"$bin" --run-tests --test-entry wrap --test-return tests "$file"

