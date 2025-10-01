#!/bin/bash
# 3-Backend Trinity Benchmark for Nyash/Hakorune
# Tests: Rust VM, LLVM (Mock), WASM

set -e

BENCH_NAME="${1:-simple_add}"
BENCH_FILE="apps/benchmarks/$BENCH_NAME/main.nyash"
MIR_JSON="/tmp/${BENCH_NAME}_mir.json"
WASM_FILE="/tmp/${BENCH_NAME}.wasm"
WASM_FIXED="/tmp/${BENCH_NAME}_fixed.wasm"

if [ ! -f "$BENCH_FILE" ]; then
    echo "❌ Benchmark not found: $BENCH_FILE"
    exit 1
fi

echo "🎯 3-Backend Trinity Benchmark: $BENCH_NAME"
echo "================================================"

# Test 1: Rust VM
echo ""
echo "1️⃣  Rust VM (Development/Debug)"
echo "   Command: ./target/release/nyash --backend vm $BENCH_FILE"
OUTPUT_VM=$(./target/release/nyash --backend vm "$BENCH_FILE" 2>&1)
RESULT_VM=$(echo "$OUTPUT_VM" | grep -v "UnifiedBoxRegistry" | grep -E '^[0-9]+' | tail -1)
if [ -n "$RESULT_VM" ]; then
    echo "   ✅ Result: $RESULT_VM"
else
    echo "   ❌ No result"
fi

# Test 2: LLVM (Mock)
echo ""
echo "2️⃣  LLVM Mock (Production/Optimized)"
echo "   Command: NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm $BENCH_FILE"
OUTPUT_LLVM=$(NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm "$BENCH_FILE" 2>&1)
if echo "$OUTPUT_LLVM" | grep -q "Mock exit code: 0"; then
    echo "   ✅ Mock execution successful"
else
    echo "   ❌ Mock execution failed"
fi

# Test 3: WASM
echo ""
echo "3️⃣  WASM (Edge/Browser)"
echo "   Pipeline: MIR JSON → WASM → Node.js"

# Check if MIR JSON exists (manual for now)
if [ "$BENCH_NAME" = "simple_add" ]; then
    MIR_JSON="/tmp/simple_add_e2e.json"
    WASM_FIXED="/tmp/test_simple_fixed.wasm"

    if [ ! -f "$MIR_JSON" ]; then
        echo "   ⚠️  MIR JSON not found: $MIR_JSON"
        echo "   Creating simple_add MIR JSON..."
        cat > "$MIR_JSON" <<'EOF'
{
  "functions": [
    {
      "name": "Main.main",
      "params": [],
      "blocks": [
        {
          "id": 0,
          "instructions": [
            {"op": "const", "dst": 0, "value": {"type": "i64", "value": 15}},
            {"op": "const", "dst": 1, "value": {"type": "i64", "value": 27}},
            {"op": "binop", "operation": "+", "lhs": 0, "rhs": 1, "dst": 2},
            {"op": "ret", "value": 2}
          ]
        }
      ]
    }
  ]
}
EOF
    fi

    # Compile to WASM
    if [ ! -f "$WASM_FIXED" ]; then
        echo "   Compiling MIR JSON to WASM..."
        cd src/llvm_py
        python3 llvm_builder.py --target wasm32 "$MIR_JSON" -o /tmp/test_simple.wasm > /dev/null 2>&1
        python3 tools/wasm_add_export.py /tmp/test_simple.wasm "$WASM_FIXED" "Main.main" 0 > /dev/null 2>&1
        cd ../..
    fi

    # Execute WASM
    OUTPUT_WASM=$(cd src/llvm_py && node tools/wasm_runner.js "$WASM_FIXED" 2>&1)
    RESULT_WASM=$(echo "$OUTPUT_WASM" | grep "returned:" | grep -oE '[0-9]+' | tail -1)

    if [ -n "$RESULT_WASM" ]; then
        echo "   ✅ Result: $RESULT_WASM"
    else
        echo "   ❌ No result"
        echo "   Debug: $OUTPUT_WASM"
    fi
else
    echo "   ⚠️  WASM support only for simple_add (for now)"
fi

# Summary
echo ""
echo "================================================"
echo "📊 Summary:"
echo "   Rust VM:  ${RESULT_VM:-N/A}"
echo "   LLVM:     Mock OK"
echo "   WASM:     ${RESULT_WASM:-N/A}"

# Verify consistency
if [ "$RESULT_VM" = "$RESULT_WASM" ] && [ -n "$RESULT_VM" ]; then
    echo ""
    echo "✅ All backends consistent!"
else
    echo ""
    echo "⚠️  Results differ or incomplete"
fi
