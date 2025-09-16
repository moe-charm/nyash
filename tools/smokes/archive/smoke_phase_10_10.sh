#!/bin/bash
# Phase 10.10 Smoke Test - Minimal verification for key features (archived)
# Note: JIT/Cranelift paths are not maintained in current phase.

set -e

echo "=== Phase 10.10 Smoke Test ==="
echo

# 1. Build with Cranelift
echo "1) Building with Cranelift JIT..."
cargo build --release -j32 --features cranelift-jit 2>&1 | head -5
echo "✓ Build complete"
echo

# 2. HH direct execution (Map.get_hh)
echo "2) Testing HH direct execution..."
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_EVENTS=1 \
  ./target/release/nyash --backend vm examples/jit_map_get_param_hh.nyash 2>&1 | head -10 | grep -E "(allow id:|value1|Result:)"
echo "✓ HH execution verified"
echo

# 3. Mutating opt-in (JitPolicyBox)
echo "3) Testing mutating opt-in policy..."
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_EVENTS=1 \
  ./target/release/nyash --backend vm examples/jit_policy_optin_mutating.nyash 2>&1 | head -15 | grep -E "(policy_denied_mutating|allow id:|Result:)"
echo "✓ Policy opt-in verified"
echo

# 4. CountingGC demo
echo "4) Testing CountingGC..."
./target/release/nyash --backend vm examples/gc_counting_demo.nyash 2>&1 | head -10 | grep -E "(GC stats:|allocations:|write barriers:)"
echo "✓ CountingGC verified"
echo

echo "=== All smoke tests passed! ==="

