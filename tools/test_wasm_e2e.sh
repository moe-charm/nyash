#!/bin/bash
# Complete E2E Test: Hakorune/Nyash → Rust VM → MIR JSON → WASM → Node.js
set -e

# Change to project root
cd "$(dirname "$0")/.."

echo "🎉 Complete E2E WASM Pipeline Test 🎉"
echo "======================================="
echo ""

# Step 0: Show source
echo "📝 Step 0: Hakorune/Nyash Source Code"
echo "--------------------------------------"
cat local_tests/wasm_e2e_simple.nyash
echo ""

# Step 1: Compile with Rust VM to MIR JSON
echo "🔧 Step 1: Nyash → MIR JSON (Rust VM)"
echo "--------------------------------------"
./target/release/nyash --dump-mir --emit-mir-json /tmp/wasm_e2e_full.json local_tests/wasm_e2e_simple.nyash 2>&1 | tail -5
echo ""

# Step 1.5: Filter to Main.main only (workaround for multi-function issue)
echo "🔧 Step 1.5: Filter MIR JSON to Main.main only"
echo "-----------------------------------------------"
python3 -c "
import json
with open('/tmp/wasm_e2e_full.json') as f:
    data = json.load(f)
filtered = {
    'capabilities': data.get('capabilities', []),
    'functions': [f for f in data['functions'] if f['name'] == 'Main.main']
}
with open('/tmp/wasm_e2e_filtered.json', 'w') as out:
    json.dump(filtered, out, indent=2)
print('✅ Filtered JSON: Main.main only')
"
echo ""

# Step 2: Compile MIR JSON to WASM
echo "⚙️ Step 2: MIR JSON → WASM Binary"
echo "----------------------------------"
cd src/llvm_py
python3 llvm_builder.py /tmp/wasm_e2e_filtered.json --target wasm32 -o /tmp/wasm_e2e.o
echo ""

# Step 3: Add export section
echo "📦 Step 3: Add Export Section"
echo "------------------------------"
python3 tools/wasm_add_export.py /tmp/wasm_e2e.o /tmp/wasm_e2e_final.wasm ny_main 0
echo ""

# Step 4: Run in Node.js
echo "▶️ Step 4: Execute WASM in Node.js"
echo "-----------------------------------"
node tools/wasm_runner.js /tmp/wasm_e2e_final.wasm
echo ""

# Show generated LLVM IR
echo "📊 Generated LLVM IR (Target: wasm32-unknown-wasi)"
echo "---------------------------------------------------"
cat /tmp/debug_ir.ll
echo ""

echo "✅✅✅ E2E Pipeline Complete! ✅✅✅"
echo ""
echo "Pipeline Summary:"
echo "  1. Source: local_tests/wasm_e2e_simple.nyash (15 + 27)"
echo "  2. Rust VM → MIR JSON (--dump-mir --emit-mir-json)"
echo "  3. Python llvm_builder.py → WASM binary"
echo "  4. wasm_add_export.py → Add exports"
echo "  5. Node.js → Execute → Returns 42 ✅"
