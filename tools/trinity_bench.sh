#!/usr/bin/env bash
# Trinity Benchmark - 3 Backend Comparison (VM / LLVM / WASM)
# Usage: ./tools/trinity_bench.sh [benchmark_name]

set -e

BENCH_NAME="${1:-01_loop_counter}"
BENCH_FILE="local_tests/bench/nyash/${BENCH_NAME}.nyash"
MIR_JSON="/tmp/${BENCH_NAME}_mir.json"
WASM_FILE="/tmp/${BENCH_NAME}.wasm"
WASM_EXPORT="/tmp/${BENCH_NAME}_exp.wasm"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

if [ ! -f "$BENCH_FILE" ]; then
    echo -e "${RED}❌ Benchmark not found: $BENCH_FILE${NC}"
    exit 1
fi

echo -e "${BLUE}🎯 Trinity Benchmark: $BENCH_NAME${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Test 1: Rust VM (Development/Debug)
echo -e "${BLUE}1️⃣  Rust VM Backend${NC}"
echo -e "   ${YELLOW}Command:${NC} ./target/release/nyash --backend vm $BENCH_FILE"
set +e
OUTPUT_VM=$(./target/release/nyash --backend vm "$BENCH_FILE" 2>&1)
EXIT_VM=$?
set -e

if [ $EXIT_VM -eq 0 ]; then
    RESULT_VM=$EXIT_VM
    echo -e "   ${GREEN}✅ Exit Code: $RESULT_VM${NC}"
else
    RESULT_VM=$EXIT_VM
    echo -e "   ${RED}❌ Exit Code: $RESULT_VM${NC}"
fi

# Test 2: LLVM Backend (Production/Optimized)
echo ""
echo -e "${BLUE}2️⃣  LLVM Backend (Python/llvmlite)${NC}"
echo -e "   ${YELLOW}Command:${NC} NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm $BENCH_FILE"

# Generate MIR JSON for LLVM
./target/release/nyash --emit-mir-json "$MIR_JSON" "$BENCH_FILE" 2>/dev/null

# Compile MIR JSON to native object
set +e
cd src/llvm_py
LLVM_OUTPUT=$(python3 llvm_builder.py "$MIR_JSON" -o /tmp/${BENCH_NAME}.o 2>&1)
LLVM_STATUS=$?
cd ../..
set -e

if [ $LLVM_STATUS -eq 0 ]; then
    echo -e "   ${GREEN}✅ LLVM compilation successful${NC}"
    RESULT_LLVM="Compiled"
else
    echo -e "   ${RED}❌ LLVM compilation failed${NC}"
    echo -e "   ${YELLOW}Debug:${NC} $LLVM_OUTPUT"
    RESULT_LLVM="Failed"
fi

# Test 3: WASM Backend (Edge/Browser)
echo ""
echo -e "${BLUE}3️⃣  WASM Backend${NC}"
echo -e "   ${YELLOW}Pipeline:${NC} MIR JSON → WASM → Node.js"

# Compile MIR JSON to WASM
set +e
cd src/llvm_py
python3 llvm_builder.py --target wasm32 "$MIR_JSON" -o "$WASM_FILE" 2>/dev/null
WASM_COMPILE=$?

if [ $WASM_COMPILE -eq 0 ]; then
    # Add export
    python3 ../../tools/wasm_add_export.py "$WASM_FILE" "$WASM_EXPORT" "ny_main:func:0" 2>/dev/null

    # Execute WASM
    WASM_OUTPUT=$(node tools/wasm_runner.js "$WASM_EXPORT" 2>&1)
    WASM_EXEC=$?
    cd ../..

    if [ $WASM_EXEC -eq 0 ]; then
        RESULT_WASM=$(echo "$WASM_OUTPUT" | grep "returned:" | grep -oE '[0-9]+' | tail -1)
        if [ -z "$RESULT_WASM" ]; then
            RESULT_WASM=$WASM_EXEC
        fi
        echo -e "   ${GREEN}✅ Exit Code: $RESULT_WASM${NC}"
    else
        RESULT_WASM="Error"
        echo -e "   ${RED}❌ Execution failed${NC}"
    fi
else
    cd ../..
    RESULT_WASM="Compile Error"
    echo -e "   ${RED}❌ WASM compilation failed${NC}"
fi
set -e

# Summary
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}📊 Trinity Benchmark Results:${NC}"
echo -e "   ${YELLOW}Rust VM:${NC}  $RESULT_VM"
echo -e "   ${YELLOW}LLVM:${NC}     $RESULT_LLVM"
echo -e "   ${YELLOW}WASM:${NC}     $RESULT_WASM"

# Verify consistency
if [ "$RESULT_VM" = "$RESULT_WASM" ] && [ -n "$RESULT_VM" ] && [ "$RESULT_VM" != "Error" ]; then
    echo ""
    echo -e "${GREEN}✅ VM and WASM backends consistent!${NC}"
    exit 0
else
    echo ""
    echo -e "${YELLOW}⚠️  Results differ or incomplete${NC}"
    exit 1
fi
