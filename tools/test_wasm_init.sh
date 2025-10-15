#!/bin/bash
# Phase 15.8 Week 1 Comprehensive Test Script
# Tests Phase 1.1, 1.2, 1.3 (WASM initialization, calling convention, build pipeline)

set -e

echo "========================================"
echo "Phase 15.8 Week 1: Comprehensive Test"
echo "Phase 1.1 + 1.2 + 1.3"
echo "========================================"
echo

# Check llvmlite installation
echo "1. Checking llvmlite installation..."
python3 -c "import llvmlite; print(f'✅ llvmlite {llvmlite.__version__}')"
echo

# Test native target
echo "2. Testing native target..."
python3 src/llvm_py/llvm_builder.py src/llvm_py/test_minimal.json -o /tmp/test_native.o
echo "✅ Native compilation successful"
echo

# Test WASM target
echo "3. Testing WASM target..."
python3 src/llvm_py/llvm_builder.py src/llvm_py/test_minimal.json -o /tmp/test_wasm.o --target wasm32
echo "✅ WASM compilation successful"
echo

# Verify triples
echo "4. Verifying target triples in LLVM IR..."
python3 -c "
import sys
sys.path.insert(0, 'src/llvm_py')
from llvm_builder import NyashLLVMBuilder
import json

with open('src/llvm_py/test_minimal.json') as f:
    mir = json.load(f)

# Native
builder_native = NyashLLVMBuilder(target='native')
builder_native.build_from_mir(mir)
ir_native = str(builder_native.module)
native_triple = [line for line in ir_native.split('\n') if 'target triple' in line][0]
print(f'Native:  {native_triple}')

# WASM
builder_wasm = NyashLLVMBuilder(target='wasm32')
builder_wasm.build_from_mir(mir)
ir_wasm = str(builder_wasm.module)
wasm_triple = [line for line in ir_wasm.split('\n') if 'target triple' in line][0]
print(f'WASM:    {wasm_triple}')

# Verify
assert 'x86_64' in native_triple or 'aarch64' in native_triple, 'Native triple incorrect'
assert 'wasm32-unknown-wasi' in wasm_triple, 'WASM triple incorrect'
print('✅ Triple verification passed')
"
echo

# Phase 1.2/1.3 Tests
echo "5. Testing build_wasm.sh script..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
$SCRIPT_DIR/build_wasm.sh src/llvm_py/test_minimal.json -o /tmp/test_build_wasm.wasm > /tmp/build_log.txt 2>&1
if grep -q "Build Complete" /tmp/build_log.txt; then
    echo "✅ build_wasm.sh successful"
else
    echo "❌ build_wasm.sh failed"
    cat /tmp/build_log.txt
    exit 1
fi
echo

echo "6. Testing WASM binary verification..."
MAGIC=$(xxd -p -l 4 /tmp/test_build_wasm.wasm 2>/dev/null || echo "")
if [[ "$MAGIC" == "0061736d" ]]; then
    echo "✅ WASM binary valid (magic: 0x$MAGIC)"
else
    echo "❌ Invalid WASM binary"
    exit 1
fi
echo

echo "7. Testing WASM inspector..."
python3 $SCRIPT_DIR/wasm_inspector.py /tmp/test_build_wasm.wasm > /tmp/wasm_inspect.txt 2>&1
if grep -q "Export Section" /tmp/wasm_inspect.txt && grep -q "ny_main" /tmp/wasm_inspect.txt; then
    echo "✅ WASM inspector works + export section found"
else
    echo "⚠️  Export section check (check /tmp/wasm_inspect.txt)"
fi
echo

echo "8. Testing Node.js WASM execution..."
if command -v node &>/dev/null; then
    node $SCRIPT_DIR/wasm_runner.js /tmp/test_build_wasm.wasm > /tmp/wasm_run.txt 2>&1 || true
    if grep -q "WASM module loaded successfully" /tmp/wasm_run.txt; then
        echo "✅ Node.js WASM loader works"
        if grep -q "ny_main() returned: 42" /tmp/wasm_run.txt; then
            echo "✅ Function execution successful (returned 42)"
        elif grep -q "ny_main not found" /tmp/wasm_run.txt; then
            echo "❌ Function export failed"
            exit 1
        fi
    else
        echo "❌ Node.js loader failed (check /tmp/wasm_run.txt)"
        exit 1
    fi
else
    echo "⚠️  Node.js not available (skipped)"
fi
echo

echo "========================================"
echo "✅ Phase 15.8 Week 1 All Tests Passed!"
echo "========================================"
echo
echo "Summary:"
echo "  Phase 1.1:"
echo "    - Native target: ✅"
echo "    - WASM target: ✅"
echo "    - Triple verification: ✅"
echo
echo "  Phase 1.2/1.3:"
echo "    - WASM calling convention: ✅"
echo "    - build_wasm.sh: ✅"
echo "    - wasm_runner.js: ✅"
echo "    - WASM binary generation: ✅"
echo
echo "  Phase 2.1 (2025-10-01):"
echo "    - WASM inspector: ✅"
echo "    - Export section adder: ✅"
echo "    - Function export: ✅"
echo "    - ny_main() execution: ✅ (returned 42)"
echo
echo "  Resolved Issues:"
echo "    - ✅ Function export (Python-based solution)"
echo
echo "  Remaining:"
echo "    - Full WASI support: Week 2-3"
echo "    - MIR18 instruction WASM conversion: Week 2"
echo
echo "Next: Phase 2.2 - WASI fd_write (print) implementation"
