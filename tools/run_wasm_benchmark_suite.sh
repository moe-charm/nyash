#!/bin/bash
# WASM Benchmark Suite Runner
# Runs multiple WASM benchmarks and collects timing data

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

PROJECT_ROOT="/home/tomoaki/git/hakorune-wasm"
WASM_RUNNER="$PROJECT_ROOT/tools/wasm_runner.js"
BUILD_SCRIPT="$PROJECT_ROOT/tools/build_wasm.sh"

# Benchmark list (name, json_file, expected_result)
declare -a BENCHMARKS=(
  # Smoke tests
  "arithmetic:$PROJECT_ROOT/src/llvm_py/test_arithmetic_smoke.json:6"
  "compare:$PROJECT_ROOT/src/llvm_py/test_compare_smoke.json:5"
  "control_flow:$PROJECT_ROOT/src/llvm_py/test_control_flow_smoke.json:111"
  "binop_all:$PROJECT_ROOT/src/llvm_py/test_binop_all.json:44"

  # Performance benchmarks (simple MIR JSON, iterative only, i32 range)
  "factorial_12:$PROJECT_ROOT/src/llvm_py/bench_factorial_simple.json:479001600"
  "power_2_30:$PROJECT_ROOT/src/llvm_py/bench_power_loop.json:1073741824"
  "sum_10k:$PROJECT_ROOT/src/llvm_py/bench_sum_loop_simple.json:49995000"
)

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🚀 WASM Benchmark Suite${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

total=0
passed=0
failed=0

for bench in "${BENCHMARKS[@]}"; do
  IFS=':' read -r name json_file expected <<< "$bench"
  total=$((total + 1))

  wasm_out="/tmp/bench_${name}.wasm"

  echo -e "${BLUE}[$total] $name${NC}"
  echo -e "${YELLOW}  Building WASM...${NC}"

  # Build WASM
  if ! bash "$BUILD_SCRIPT" "$json_file" -o "$wasm_out" >/dev/null 2>&1; then
    echo -e "${RED}  ❌ Build failed${NC}"
    failed=$((failed + 1))
    continue
  fi

  echo -e "${YELLOW}  Running WASM...${NC}"

  # Run WASM and measure time
  start_ms=$(node -e "console.log(Date.now())")
  output=$(node "$WASM_RUNNER" "$wasm_out" 2>&1) || true
  end_ms=$(node -e "console.log(Date.now())")
  elapsed=$((end_ms - start_ms))

  # Extract return value
  if echo "$output" | grep -q "returned:"; then
    result=$(echo "$output" | grep "returned:" | awk '{print $NF}')

    if [ "$result" == "$expected" ]; then
      echo -e "${GREEN}  ✅ PASS: result=$result, time=${elapsed}ms${NC}"
      passed=$((passed + 1))
    else
      echo -e "${RED}  ❌ FAIL: expected=$expected, got=$result${NC}"
      failed=$((failed + 1))
    fi
  else
    echo -e "${RED}  ❌ FAIL: No return value${NC}"
    echo "$output"
    failed=$((failed + 1))
  fi

  echo ""
done

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}📊 Results: Total=$total Passed=${GREEN}$passed${NC} Failed=${RED}$failed${NC}${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ $failed -eq 0 ] && [ $total -gt 0 ]; then
  echo -e "${GREEN}🎉 All benchmarks passed!${NC}"
  exit 0
else
  echo -e "${RED}❌ Some benchmarks failed${NC}"
  exit 1
fi
