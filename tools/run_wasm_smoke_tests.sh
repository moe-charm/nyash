#!/bin/bash
# Phase 15.8: WASM Smoke Tests Runner
# Runs all smoke tests and verifies expected output

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a single test
run_test() {
    local test_name=$1
    local test_file=$2
    local expected_value=$3

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    echo "----------------------------------------"
    echo "Running: $test_name"
    echo "Expected: $expected_value"

    # Build WASM
    local wasm_output="/tmp/${test_name}.wasm"
    if ! bash tools/build_wasm.sh "src/llvm_py/${test_file}" -o "$wasm_output" &> /tmp/build_log.txt; then
        echo -e "${RED}✗ Build failed${NC}"
        cat /tmp/build_log.txt
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi

    # Run WASM
    local output=$(node tools/wasm_runner.js "$wasm_output" 2>&1)
    local return_value=$(echo "$output" | grep -oP 'returned: \K\d+' || echo "ERROR")

    # Check result
    if [ "$return_value" == "$expected_value" ]; then
        echo -e "${GREEN}✓ PASSED${NC} (returned: $return_value)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}✗ FAILED${NC} (expected: $expected_value, got: $return_value)"
        echo "Full output:"
        echo "$output"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Main test suite
echo "========================================"
echo "Phase 15.8: WASM Smoke Test Suite"
echo "========================================"
echo ""

# Test 1: Arithmetic operations
run_test "arithmetic_smoke" "test_arithmetic_smoke.json" "6"

# Test 2: Compare operations
run_test "compare_smoke" "test_compare_smoke.json" "5"

# Test 3: Control flow (nested if)
run_test "control_flow_smoke" "test_control_flow_smoke.json" "111"

# Test 4: Hello World (if exists)
if [ -f "src/llvm_py/test_hello_world.json" ]; then
    run_test "hello_world_smoke" "test_hello_world.json" "0"
fi

# Summary
echo ""
echo "========================================"
echo "Test Summary"
echo "========================================"
echo "Total:  $TOTAL_TESTS"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi
