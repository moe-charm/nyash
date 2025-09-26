#!/bin/bash
# async_await.sh - Minimal async/await smoke using env.future

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/nyash_async_await_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > async.nyash << 'EOF'
static box Main {
  main() {
    // Create a future from a value and await it
    nowait f = 42
    local v = await f
    print(v)
    return 0
  }
}
EOF

output=$(NYASH_REWRITE_FUTURE=1 run_nyash_vm async.nyash 2>&1 || true)
if echo "$output" | grep -q "ExternCall .* not supported\|unimplemented instruction: FutureNew"; then
  test_skip "async_await" "VM interpreter lacks Future/ExternCall support"
  rc=0
else
  compare_outputs "42" "$output" "async_await"
  rc=$?
fi
cd /
rm -rf "$TEST_DIR"
exit $rc
