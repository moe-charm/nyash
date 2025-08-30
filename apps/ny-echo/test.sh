#!/bin/bash
# ny-echo テストスクリプト

set -e

NYASH=${NYASH:-"../../target/release/nyash"}
SCRIPT="main.nyash"

echo "=== ny-echo Test Suite ==="

# Test 1: 基本エコー
echo "Test 1: Basic echo"
echo "Hello World" | $NYASH $SCRIPT > out1.txt
if [ "$(cat out1.txt)" == "Hello World" ]; then
    echo "✓ Basic echo passed"
else
    echo "✗ Basic echo failed"
    exit 1
fi

# Test 2: 大文字変換
echo "Test 2: Uppercase transformation"
echo "hello world" | $NYASH $SCRIPT --upper > out2.txt
if [ "$(cat out2.txt)" == "HELLO WORLD" ]; then
    echo "✓ Uppercase passed"
else
    echo "✗ Uppercase failed"
    exit 1
fi

# Test 3: 小文字変換
echo "Test 3: Lowercase transformation"
echo "HELLO WORLD" | $NYASH $SCRIPT --lower > out3.txt
if [ "$(cat out3.txt)" == "hello world" ]; then
    echo "✓ Lowercase passed"
else
    echo "✗ Lowercase failed"
    exit 1
fi

# Test 4: 複数行入力
echo "Test 4: Multiple lines"
printf "Line 1\nLine 2\nLine 3" | $NYASH $SCRIPT > out4.txt
if [ $(wc -l < out4.txt) -eq 3 ]; then
    echo "✓ Multiple lines passed"
else
    echo "✗ Multiple lines failed"
    exit 1
fi

# Test 5: パフォーマンステスト（1万行）
echo "Test 5: Performance test (10000 lines)"
seq 1 10000 | $NYASH $SCRIPT > out5.txt
if [ $(wc -l < out5.txt) -eq 10000 ]; then
    echo "✓ Performance test passed"
else
    echo "✗ Performance test failed"
    exit 1
fi

# Test 6: VM/JIT/AOT比較
echo "Test 6: Backend comparison"

# VM実行
echo "hello" | $NYASH --backend interpreter $SCRIPT > vm_out.txt
VM_HASH=$(sha256sum vm_out.txt | cut -d' ' -f1)

# JIT実行（利用可能な場合）
if $NYASH --help | grep -q "jit"; then
    echo "hello" | $NYASH --backend jit $SCRIPT > jit_out.txt
    JIT_HASH=$(sha256sum jit_out.txt | cut -d' ' -f1)
    
    if [ "$VM_HASH" == "$JIT_HASH" ]; then
        echo "✓ VM/JIT output matches"
    else
        echo "✗ VM/JIT output mismatch"
        exit 1
    fi
fi

# クリーンアップ
rm -f out*.txt vm_out.txt jit_out.txt

echo ""
echo "=== All tests passed! ==="