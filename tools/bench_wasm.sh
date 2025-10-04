#!/bin/bash
# WASM Benchmark Suite - VM/WASM Parity Verification
set -e

BENCH_DIR="local_tests/bench"
TMP_DIR="/tmp/nyash_bench"
NYASH="./target/release/nyash"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "🔥 WASM Benchmark Suite Starting..."
echo

# Build if needed
if [ ! -f "$NYASH" ]; then
    echo "Building nyash..."
    cargo build --release
fi

mkdir -p "$TMP_DIR"

total=0
passed=0
failed=0

for bench in "$BENCH_DIR"/*.nyash; do
    if [ ! -f "$bench" ]; then
        echo "No benchmark files found in $BENCH_DIR"
        exit 1
    fi

    name=$(basename "$bench" .nyash)
    total=$((total + 1))

    # Extract expected value from comment
    expected=$(head -n 2 "$bench" | grep "期待値:" | sed 's/.*期待値: *\([0-9]*\).*/\1/')
    if [ -z "$expected" ]; then
        expected="UNKNOWN"
    fi

    echo -e "${BLUE}==== [$total] $name (expected: $expected) ====${NC}"

    # 1. MIR JSON生成
    echo -n "  MIR JSON generation... "
    if ! "$NYASH" --emit-mir-json "$TMP_DIR/$name.json" "$bench" >/dev/null 2>&1; then
        echo -e "${RED}FAILED (MIR generation error)${NC}"
        failed=$((failed + 1))
        continue
    fi
    echo "✓"

    # 2. WASM生成
    echo -n "  WASM compilation... "
    if ! python src/llvm_py/llvm_builder.py "$TMP_DIR/$name.json" --target wasm32 -o "$TMP_DIR/$name.wasm" >/dev/null 2>&1; then
        echo -e "${RED}FAILED (WASM compilation error)${NC}"
        failed=$((failed + 1))
        continue
    fi
    echo "✓"

    # 3. Export追加
    echo -n "  Adding exports... "
    if ! python tools/wasm_add_export.py "$TMP_DIR/$name.wasm" "$TMP_DIR/${name}_exp.wasm" "ny_main:func:0" >/dev/null 2>&1; then
        echo -e "${RED}FAILED (export addition error)${NC}"
        failed=$((failed + 1))
        continue
    fi
    echo "✓"

    # 4. WASM実行
    echo -n "  WASM execution... "
    wasm_output=$(node tools/wasm_runner.js "$TMP_DIR/${name}_exp.wasm" 2>&1)
    if ! echo "$wasm_output" | grep -q "returned:"; then
        echo -e "${RED}FAILED (WASM execution error)${NC}"
        echo "$wasm_output"
        failed=$((failed + 1))
        continue
    fi
    wasm_result=$(echo "$wasm_output" | grep "returned:" | awk '{print $NF}')
    echo "✓ (result: $wasm_result)"

    # 5. 期待値との比較
    if [ "$expected" == "UNKNOWN" ]; then
        echo -e "  ${GREEN}✅ PASS${NC}: result=$wasm_result (no expected value)"
        passed=$((passed + 1))
    elif [ "$wasm_result" == "$expected" ]; then
        echo -e "  ${GREEN}✅ PASS${NC}: expected=$expected actual=$wasm_result"
        passed=$((passed + 1))
    else
        echo -e "  ${RED}❌ FAIL${NC}: expected=$expected actual=$wasm_result"
        failed=$((failed + 1))
    fi
    echo
done

echo "========================================"
echo -e "${BLUE}Benchmark Results:${NC}"
echo "  Total:  $total"
echo -e "  ${GREEN}Passed: $passed${NC}"
if [ $failed -gt 0 ]; then
    echo -e "  ${RED}Failed: $failed${NC}"
else
    echo -e "  Failed: $failed"
fi
echo

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}🎉 All benchmarks passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some benchmarks failed${NC}"
    exit 1
fi
