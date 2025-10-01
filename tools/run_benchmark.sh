#!/bin/bash
# Hakorune/Nyash 3-Backend Benchmark Runner (Simple Version)

set -e

if [ $# -eq 0 ]; then
    echo "Usage: $0 <benchmark_name>"
    echo ""
    echo "Available benchmarks:"
    ls -1 apps/benchmarks/ 2>/dev/null | grep -v README || echo "  (none yet)"
    exit 1
fi

BENCH_NAME=$1
BENCH_FILE="apps/benchmarks/$BENCH_NAME/main.nyash"

if [ ! -f "$BENCH_FILE" ]; then
    echo "❌ Benchmark not found: $BENCH_FILE"
    exit 1
fi

echo "🏃 Running benchmark: $BENCH_NAME"
echo "=============================================="
echo ""

# Extract expected result from comment
EXPECTED=$(grep -oP '(?<=// → )\d+' "$BENCH_FILE" || echo "N/A")
echo "Expected result: $EXPECTED"
echo ""

# 1. Rust VM
echo "📊 [1/3] Rust VM Backend"
echo "------------------------"
# Get the last line that's a number (filter out stderr and empty lines)
OUTPUT_VM=$(./target/release/nyash --backend vm "$BENCH_FILE" 2>&1)
RESULT_VM=$(echo "$OUTPUT_VM" | grep -v "UnifiedBoxRegistry" | grep -E '^[0-9]+$' | tail -1)
# If empty, mark as ERROR
if [ -z "$RESULT_VM" ]; then
    RESULT_VM="ERROR"
fi
echo "Result: $RESULT_VM"
echo ""

# 2. LLVM (Native exe)
echo "📊 [2/3] LLVM Backend (Native)"
echo "-------------------------------"
if ./target/release/nyash --version 2>/dev/null | grep -q "features.*llvm"; then
    RESULT_LLVM=$(NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm "$BENCH_FILE" 2>&1 | tail -1 || echo "ERROR")
    echo "Result: $RESULT_LLVM"
else
    echo "⚠️  LLVM backend not available (rebuild with --features llvm)"
    RESULT_LLVM="N/A"
fi
echo ""

# 3. LLVM (WASM)
echo "📊 [3/3] LLVM Backend (WASM)"
echo "----------------------------"

# Generate MIR JSON
./target/release/nyash --dump-mir --emit-mir-json /tmp/bench.json "$BENCH_FILE" 2>&1 > /dev/null

# Filter to main function only (workaround for multi-function issue)
python3 -c "
import json
with open('/tmp/bench.json') as f:
    data = json.load(f)
# Find Fibonacci.main or similar
main_funcs = [f for f in data['functions'] if 'main' in f['name']]
if not main_funcs:
    print('Error: No main function found', file=sys.stderr)
    sys.exit(1)
filtered = {
    'capabilities': data.get('capabilities', []),
    'functions': main_funcs
}
with open('/tmp/bench_filtered.json', 'w') as out:
    json.dump(filtered, out)
" 2>&1

if [ $? -eq 0 ]; then
    # Compile to WASM
    cd src/llvm_py
    python3 llvm_builder.py /tmp/bench_filtered.json --target wasm32 -o /tmp/bench.o 2>&1 > /dev/null
    python3 tools/wasm_add_export.py /tmp/bench.o /tmp/bench.wasm ny_main 0 2>&1 > /dev/null

    # Run in Node.js
    RESULT_WASM=$(node tools/wasm_runner.js /tmp/bench.wasm 2>&1 | grep -oP '(?<=returned: )\d+' || echo "ERROR")
    cd ../..
    echo "Result: $RESULT_WASM"
else
    RESULT_WASM="ERROR"
    echo "Result: ERROR (MIR JSON filtering failed)"
fi
echo ""

# Summary
echo "=============================================="
echo "📈 Benchmark Summary"
echo "=============================================="
printf "%-20s %-10s %-10s\n" "Backend" "Result" "Status"
echo "----------------------------------------------"

# Check VM
if [ "$RESULT_VM" = "$EXPECTED" ]; then
    STATUS_VM="✅"
elif [ "$RESULT_VM" = "ERROR" ]; then
    STATUS_VM="❌ ERROR"
else
    STATUS_VM="❌ WRONG"
fi
printf "%-20s %-10s %-10s\n" "Rust VM" "$RESULT_VM" "$STATUS_VM"

# Check LLVM
if [ "$RESULT_LLVM" = "N/A" ]; then
    STATUS_LLVM="⚠️  N/A"
elif [ "$RESULT_LLVM" = "$EXPECTED" ]; then
    STATUS_LLVM="✅"
elif [ "$RESULT_LLVM" = "ERROR" ]; then
    STATUS_LLVM="❌ ERROR"
else
    STATUS_LLVM="❌ WRONG"
fi
printf "%-20s %-10s %-10s\n" "LLVM (Native)" "$RESULT_LLVM" "$STATUS_LLVM"

# Check WASM
if [ "$RESULT_WASM" = "$EXPECTED" ]; then
    STATUS_WASM="✅"
elif [ "$RESULT_WASM" = "ERROR" ]; then
    STATUS_WASM="❌ ERROR"
else
    STATUS_WASM="❌ WRONG"
fi
printf "%-20s %-10s %-10s\n" "LLVM (WASM)" "$RESULT_WASM" "$STATUS_WASM"

echo "=============================================="

# Overall status
if [ "$RESULT_VM" = "$EXPECTED" ] && [ "$RESULT_LLVM" = "$EXPECTED" ] && [ "$RESULT_WASM" = "$EXPECTED" ]; then
    echo "✅ All backends agree! Correctness verified."
    exit 0
elif [ "$RESULT_VM" = "$EXPECTED" ] && [ "$RESULT_WASM" = "$EXPECTED" ]; then
    echo "✅ VM and WASM agree! (LLVM: $STATUS_LLVM)"
    exit 0
else
    echo "❌ Backend mismatch or errors detected."
    exit 1
fi
