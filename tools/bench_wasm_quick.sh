#!/usr/bin/env bash
# Quick WASM Benchmark Runner - Uses existing MIR JSON files
set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MIR_DIR="$PROJECT_ROOT/local_tests/bench/mir"
WASM_OUT="/tmp/nyash_bench_wasm"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

mkdir -p "$WASM_OUT"

echo -e "${BLUE}🚀 Quick WASM Benchmark Runner${NC}"
echo ""

total=0
passed=0
failed=0

for mir_json in "$MIR_DIR"/*.json; do
    [ -e "$mir_json" ] || continue
    
    name=$(basename "$mir_json" .json)
    total=$((total + 1))
    
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}[$total] $name${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    # Compile MIR JSON → WASM
    echo -e "${YELLOW}🔧 Compiling to WASM...${NC}"
    if ! python3 "$PROJECT_ROOT/src/llvm_py/llvm_builder.py" "$mir_json" --target wasm32 -o "$WASM_OUT/${name}.wasm" 2>&1 | tail -2; then
        echo -e "${RED}❌ WASM compilation failed${NC}"
        failed=$((failed + 1))
        continue
    fi
    
    # Add export
    echo -e "${YELLOW}📦 Adding exports...${NC}"
    if ! python3 "$PROJECT_ROOT/tools/wasm_add_export.py" "$WASM_OUT/${name}.wasm" "$WASM_OUT/${name}_exp.wasm" "ny_main:func:0" 2>&1 | tail -2; then
        echo -e "${RED}❌ Export addition failed${NC}"
        failed=$((failed + 1))
        continue
    fi
    
    # Execute WASM
    echo -e "${YELLOW}▶️  Executing WASM...${NC}"
    set +e
    result=$(node "$PROJECT_ROOT/src/llvm_py/tools/wasm_runner.js" "$WASM_OUT/${name}_exp.wasm" 2>&1)
    exit_code=$?
    set -e
    
    echo "$result" | grep -E "(🚀|✅)"
    
    if echo "$result" | grep -q "returned:"; then
        returned_value=$(echo "$result" | grep "returned:" | awk '{print $NF}')
        echo -e "${GREEN}✅ PASS: returned=$returned_value, exit_code=$exit_code${NC}"
        passed=$((passed + 1))
    else
        echo -e "${RED}❌ FAIL: No return value found${NC}"
        echo "$result"
        failed=$((failed + 1))
    fi
    
    echo ""
done

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}📊 Results: Total=$total Passed=$passed Failed=$failed${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ $failed -eq 0 ] && [ $total -gt 0 ]; then
    echo -e "${GREEN}🎉 All benchmarks passed!${NC}"
    exit 0
elif [ $total -eq 0 ]; then
    echo -e "${YELLOW}⚠️  No benchmark files found in $MIR_DIR${NC}"
    exit 1
else
    echo -e "${RED}❌ Some benchmarks failed${NC}"
    exit 1
fi
