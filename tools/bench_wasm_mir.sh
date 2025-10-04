#!/bin/bash
# WASM Benchmark Suite - MIR JSON based
set -e

BENCH_MIR_DIR="local_tests/bench/mir"
TMP_DIR="/tmp/nyash_bench_wasm"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "🔥 WASM Benchmark Suite (MIR JSON) Starting..."
echo

mkdir -p "$TMP_DIR"

total=0
passed=0
failed=0

# Benchmark definitions: name:expected_value
declare -A benchmarks
benchmarks["01_loop_counter.json"]="10"

for bench_file in "${!benchmarks[@]}"; do
    bench_path="$BENCH_MIR_DIR/$bench_file"

    if [ ! -f "$bench_path" ]; then
        echo -e "${RED}File not found: $bench_path${NC}"
        continue
    fi

    name=$(basename "$bench_file" .json)
    expected="${benchmarks[$bench_file]}"
    total=$((total + 1))

    echo -e "${BLUE}==== [$total] $name (expected: $expected) ====${NC}"

    # 1. WASM compilation
    echo -n "  WASM compilation... "
    if ! python src/llvm_py/llvm_builder.py "$bench_path" --target wasm32 -o "$TMP_DIR/$name.wasm" >/dev/null 2>&1; then
        echo -e "${RED}FAILED${NC}"
        failed=$((failed + 1))
        continue
    fi
    echo "✓"

    # 2. Add exports
    echo -n "  Adding exports... "
    if ! python tools/wasm_add_export.py "$TMP_DIR/$name.wasm" "$TMP_DIR/${name}_exp.wasm" "ny_main:func:0" >/dev/null 2>&1; then
        echo -e "${RED}FAILED${NC}"
        failed=$((failed + 1))
        continue
    fi
    echo "✓"

    # 3. WASM execution
    echo -n "  WASM execution... "
    node tools/wasm_runner.js "$TMP_DIR/${name}_exp.wasm" > "$TMP_DIR/${name}_output.txt" 2>&1
    if ! grep -q "returned:" "$TMP_DIR/${name}_output.txt"; then
        echo -e "${RED}FAILED${NC}"
        cat "$TMP_DIR/${name}_output.txt"
        failed=$((failed + 1))
        continue
    fi
    wasm_result=$(grep "returned:" "$TMP_DIR/${name}_output.txt" | awk '{print $NF}')
    echo "✓ (result: $wasm_result)"

    # 4. Verification
    if [ "$wasm_result" == "$expected" ]; then
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

if [ $failed -eq 0 ] && [ $total -gt 0 ]; then
    echo -e "${GREEN}🎉 All benchmarks passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some benchmarks failed or no benchmarks found${NC}"
    exit 1
fi
