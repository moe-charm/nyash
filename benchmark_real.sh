#!/bin/bash

echo "🚀 真のWASM実行性能ベンチマーク"
echo "================================="

# 実行回数
ITERATIONS=100

echo "📊 測定回数: $ITERATIONS"
echo

# 1. インタープリター測定
echo "1️⃣ インタープリター実行測定"
interpreter_total=0
for i in $(seq 1 $ITERATIONS); do
    start_time=$(date +%s%N)
    ./target/release/nyash test_local_vars.nyash >/dev/null 2>&1
    end_time=$(date +%s%N)
    duration=$((($end_time - $start_time) / 1000000))  # ms
    interpreter_total=$(($interpreter_total + $duration))
done
interpreter_avg=$(echo "scale=2; $interpreter_total / $ITERATIONS" | bc)
echo "   平均実行時間: ${interpreter_avg} ms"

# 2. VM測定  
echo "2️⃣ VM実行測定"
vm_total=0
for i in $(seq 1 $ITERATIONS); do
    start_time=$(date +%s%N)
    ./target/release/nyash --backend vm test_local_vars.nyash >/dev/null 2>&1
    end_time=$(date +%s%N)
    duration=$((($end_time - $start_time) / 1000000))  # ms
    vm_total=$(($vm_total + $duration))
done
vm_avg=$(echo "scale=2; $vm_total / $ITERATIONS" | bc)
echo "   平均実行時間: ${vm_avg} ms"

# 3. WASM実行測定（wasmtime）
echo "3️⃣ WASM実行測定（wasmtime）"
wasm_total=0
for i in $(seq 1 $ITERATIONS); do
    start_time=$(date +%s%N)
    $HOME/.wasmtime/bin/wasmtime run bench_simple.wat --invoke main >/dev/null 2>&1
    end_time=$(date +%s%N)
    duration=$((($end_time - $start_time) / 1000000))  # ms
    wasm_total=$(($wasm_total + $duration))
done
wasm_avg=$(echo "scale=2; $wasm_total / $ITERATIONS" | bc)
echo "   平均実行時間: ${wasm_avg} ms"

# 4. 結果比較
echo
echo "📈 性能比較結果"
echo "==============="
echo "インタープリター: ${interpreter_avg} ms (1x baseline)"
echo "VM:              ${vm_avg} ms"
echo "WASM (wasmtime): ${wasm_avg} ms"

# 速度比計算
vm_speedup=$(echo "scale=1; $interpreter_avg / $vm_avg" | bc)
wasm_speedup=$(echo "scale=1; $interpreter_avg / $wasm_avg" | bc)

echo
echo "🏆 速度向上比較"
echo "VM:              ${vm_speedup}x faster"
echo "WASM:            ${wasm_speedup}x faster"