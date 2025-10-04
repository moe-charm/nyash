#!/bin/bash
# WASM Benchmark Runner - Hako Source Benchmarks
# Runs Hako → MIR JSON → WASM pipeline and measures performance

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

PROJECT_ROOT="/home/tomoaki/git/hakorune-wasm"
NYASH="$PROJECT_ROOT/target/release/nyash"
BUILD_WASM="$PROJECT_ROOT/tools/build_wasm.sh"
WASM_RUNNER="$PROJECT_ROOT/tools/wasm_runner.js"
BENCHMARK_DIR="$PROJECT_ROOT/apps/benchmarks/wasm"

# Check prerequisites
if [ ! -f "$NYASH" ]; then
  echo -e "${RED}❌ Error: nyash binary not found${NC}"
  echo "Please run: cargo build --release"
  exit 1
fi

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🚀 WASM Benchmark Suite (Hako Source)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

total=0
passed=0
failed=0

# Find all .hako benchmark files
while IFS= read -r hako_file; do
  total=$((total + 1))

  # Extract benchmark name
  name=$(basename "$hako_file" .hako)
  category=$(basename "$(dirname "$hako_file")")

  echo -e "${BLUE}[$total] $category/$name${NC}"

  # Prepare temp files
  mir_json="/tmp/bench_${name}.json"
  wasm_out="/tmp/bench_${name}.wasm"

  # Step 1: Hako → MIR JSON
  echo -e "${YELLOW}  Compiling Hako → MIR JSON...${NC}"
  if ! "$NYASH" --emit-mir-json "$mir_json" "$hako_file" >/dev/null 2>&1; then
    echo -e "${RED}  ❌ Compilation failed${NC}"
    failed=$((failed + 1))
    echo ""
    continue
  fi

  # Step 2: MIR JSON → WASM
  echo -e "${YELLOW}  Building WASM...${NC}"
  if ! bash "$BUILD_WASM" "$mir_json" -o "$wasm_out" >/dev/null 2>&1; then
    echo -e "${RED}  ❌ WASM build failed${NC}"
    failed=$((failed + 1))
    echo ""
    continue
  fi

  # Step 3: Run WASM and measure time
  echo -e "${YELLOW}  Running WASM...${NC}"

  start_ms=$(node -e "console.log(Date.now())")
  output=$(node "$WASM_RUNNER" "$wasm_out" 2>&1) || true
  end_ms=$(node -e "console.log(Date.now())")
  elapsed=$((end_ms - start_ms))

  # Extract return value
  if echo "$output" | grep -q "returned:"; then
    result=$(echo "$output" | grep "returned:" | awk '{print $NF}')
    echo -e "${GREEN}  ✅ PASS: result=$result, time=${elapsed}ms${NC}"
    passed=$((passed + 1))
  else
    echo -e "${RED}  ❌ FAIL: No return value${NC}"
    echo "$output" | head -5
    failed=$((failed + 1))
  fi

  echo ""
done < <(find "$BENCHMARK_DIR" -name "*.hako" | sort)

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
