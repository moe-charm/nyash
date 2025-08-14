#!/bin/bash

# AOT Performance Test Script
# Tests the new AOT compilation functionality

echo "🚀 Nyash AOT Performance Test Suite"
echo "======================================"
echo

# Test file
cat > test_performance.nyash << 'EOF'
// Performance test program
local i, sum, max
sum = 0
max = 100
i = 0

// Simple loop for computation
loop (i < max) {
    sum = sum + i
    i = i + 1
    if (i >= max) {
        break
    }
}
EOF

echo "📝 Test Program:"
cat test_performance.nyash
echo
echo "======================================"

# Test 1: Interpreter Backend
echo "🔍 Test 1: Interpreter Backend"
echo "------------------------------"
time ./target/release/nyash test_performance.nyash
echo

# Test 2: VM Backend
echo "🔍 Test 2: VM Backend"
echo "---------------------"
time ./target/release/nyash --backend vm test_performance.nyash
echo

# Test 3: WASM Compilation
echo "🔍 Test 3: WASM Compilation"
echo "---------------------------"
time ./target/release/nyash --compile-wasm test_performance.nyash -o test_performance.wat
echo "📊 WASM file size:"
ls -lh test_performance.wat 2>/dev/null || echo "WASM compilation failed"
echo

# Test 4: AOT Compilation
echo "🔍 Test 4: AOT Compilation"
echo "--------------------------"
time ./target/release/nyash --compile-native test_performance.nyash -o test_performance
echo "📊 AOT file size:"
ls -lh test_performance.cwasm 2>/dev/null || echo "AOT compilation failed"
echo

# Test 5: AOT Short Form
echo "🔍 Test 5: AOT Short Form (--aot)"
echo "---------------------------------"
time ./target/release/nyash --aot test_performance.nyash
echo

echo "🎉 Performance Test Complete!"
echo "=============================="
echo

# Cleanup
rm -f test_performance.nyash test_performance.wat test_performance.cwasm