#!/bin/bash
# E2E Test: Hakorune/Nyash Source → MIR JSON → WASM → Node.js
set -e

echo "🚀 E2E WASM Pipeline Test"
echo "=========================="
echo ""

# Step 0: Show source
echo "📝 Step 0: Hakorune/Nyash Source"
echo "--------------------------------"
cat ../../local_tests/wasm_e2e_simple.nyash
echo ""

# Step 1: MIR JSON (currently manual, automated emission TODO)
echo "🔧 Step 1: MIR JSON"
echo "-------------------"
echo "Using manually created wasm_e2e_test.json"
echo "TODO: Automate this step with Nyash compiler"
echo ""

# Step 2: Compile to WASM
echo "⚙️ Step 2: Compile MIR JSON → WASM"
echo "----------------------------------"
python3 ../llvm_builder.py ../wasm_e2e_test.json --target wasm32 -o /tmp/wasm_e2e.o
echo ""

# Step 3: Add export section
echo "📦 Step 3: Add Export Section"
echo "------------------------------"
python3 wasm_add_export.py /tmp/wasm_e2e.o /tmp/wasm_e2e.wasm ny_main 0
echo ""

# Step 4: Run in Node.js
echo "▶️ Step 4: Execute WASM in Node.js"
echo "-----------------------------------"
node wasm_runner.js /tmp/wasm_e2e.wasm
echo ""

# Show generated LLVM IR
echo "📊 Generated LLVM IR"
echo "--------------------"
cat /tmp/debug_ir.ll
echo ""

echo "✅ E2E Pipeline Complete!"
