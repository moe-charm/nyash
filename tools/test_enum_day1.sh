#!/usr/bin/env bash
# Day 1 Enum Parser Test Suite
# Purpose: Execute all tests for @enum parsing (TDD approach)

set -e  # Exit on first failure

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HAKO="$PROJECT_ROOT/target/release/hako"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper functions
pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((PASSED_TESTS++))
    ((TOTAL_TESTS++))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    echo -e "  ${RED}Error: $2${NC}"
    ((FAILED_TESTS++))
    ((TOTAL_TESTS++))
}

section() {
    echo ""
    echo -e "${YELLOW}========================================${NC}"
    echo -e "${YELLOW}$1${NC}"
    echo -e "${YELLOW}========================================${NC}"
}

# ============================================================================
# PHASE 1: UNIT TESTS (Rust)
# ============================================================================

section "Phase 1: Unit Tests (Rust)"

echo "Building project..."
if cargo build --release 2>&1 | grep -q "error"; then
    fail "Cargo build" "Compilation errors"
    exit 1
else
    pass "Cargo build"
fi

echo ""
echo "Running Rust unit tests..."
if cargo test --lib enum_parser::tests 2>&1 | tee /tmp/enum_test_output.txt; then
    # Count passed tests from output
    UNIT_PASSED=$(grep -c "test.*ok" /tmp/enum_test_output.txt || echo "0")
    pass "Unit tests ($UNIT_PASSED tests)"
else
    UNIT_FAILED=$(grep -c "test.*FAILED" /tmp/enum_test_output.txt || echo "unknown")
    fail "Unit tests" "$UNIT_FAILED test(s) failed"
fi

# ============================================================================
# PHASE 2: INTEGRATION TESTS (Hakorune programs)
# ============================================================================

section "Phase 2: Integration Tests (Hakorune)"

# Test 1: Basic parsing
echo "Test 1: Basic @enum parsing..."
if NYASH_DISABLE_PLUGINS=1 "$HAKO" apps/tests/enum/test_enum_parse_basic.hako > /tmp/enum_int1.txt 2>&1; then
    if grep -q "ENUM_BASIC" /tmp/enum_int1.txt; then
        pass "Integration Test 1: Basic parsing"
    else
        fail "Integration Test 1" "Expected output not found"
        cat /tmp/enum_int1.txt
    fi
else
    fail "Integration Test 1" "Parser crashed or returned error"
    cat /tmp/enum_int1.txt
fi

# Test 2: Option-like enum
echo "Test 2: Option-like @enum (unit variant)..."
if NYASH_DISABLE_PLUGINS=1 "$HAKO" apps/tests/enum/test_enum_parse_option.hako > /tmp/enum_int2.txt 2>&1; then
    if grep -q "ENUM_OPTION" /tmp/enum_int2.txt; then
        pass "Integration Test 2: Option-like enum"
    else
        fail "Integration Test 2" "Expected output not found"
        cat /tmp/enum_int2.txt
    fi
else
    fail "Integration Test 2" "Parser crashed or returned error"
    cat /tmp/enum_int2.txt
fi

# Test 3: Multi-variant enum
echo "Test 3: Multi-variant @enum..."
if NYASH_DISABLE_PLUGINS=1 "$HAKO" apps/tests/enum/test_enum_parse_multi.hako > /tmp/enum_int3.txt 2>&1; then
    if grep -q "ENUM_MULTI" /tmp/enum_int3.txt; then
        pass "Integration Test 3: Multi-variant enum"
    else
        fail "Integration Test 3" "Expected output not found"
        cat /tmp/enum_int3.txt
    fi
else
    fail "Integration Test 3" "Parser crashed or returned error"
    cat /tmp/enum_int3.txt
fi

# ============================================================================
# PHASE 3: END-TO-END TEST
# ============================================================================

section "Phase 3: End-to-End Test"

echo "E2E Test: Full pipeline..."
if NYASH_DISABLE_PLUGINS=1 "$HAKO" apps/tests/enum/test_enum_e2e_minimal.hako > /tmp/enum_e2e.txt 2>&1; then
    if grep -q "ENUM_E2E" /tmp/enum_e2e.txt; then
        pass "E2E Test: Full pipeline"
    else
        fail "E2E Test" "Expected output not found"
        cat /tmp/enum_e2e.txt
    fi
else
    fail "E2E Test" "Execution failed"
    cat /tmp/enum_e2e.txt
fi

# ============================================================================
# SUMMARY
# ============================================================================

section "Test Summary"

echo "Total tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
echo -e "${RED}Failed: $FAILED_TESTS${NC}"

if [ "$FAILED_TESTS" -eq 0 ]; then
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}ALL TESTS PASSED! 🎉${NC}"
    echo -e "${GREEN}Day 1 parser implementation complete!${NC}"
    echo -e "${GREEN}========================================${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}========================================${NC}"
    echo -e "${RED}SOME TESTS FAILED ❌${NC}"
    echo -e "${RED}Fix errors and re-run tests${NC}"
    echo -e "${RED}========================================${NC}"
    exit 1
fi
