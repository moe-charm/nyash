#!/bin/bash
# 言語対決ベンチマーク実行スクリプト
# sum_loop を Nyash (VM/LLVM/WASM) vs Python vs C で比較

set -e

echo "🎮 Language Shootout: sum_loop Benchmark (5 seconds)"
echo "===================================================="
echo ""

# カラー定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 結果保存用
RESULTS_FILE="benchmarks/shootout_results.txt"
> "$RESULTS_FILE"

# ディレクトリ確認
if [ ! -f "benchmarks/sum_loop_bench.hako" ]; then
    echo "❌ Error: benchmarks/sum_loop_bench.hako not found"
    exit 1
fi

# ===================================
# 1. C言語版（-O3最適化）
# ===================================
echo -e "${BLUE}1️⃣ C (gcc -O3)${NC}"
echo "-----------------------------------"

gcc -O3 -o benchmarks/sum_loop_bench_c benchmarks/sum_loop_bench.c -lrt 2>&1 || {
    echo "⚠️  Compilation failed, trying without -lrt"
    gcc -O3 -o benchmarks/sum_loop_bench_c benchmarks/sum_loop_bench.c
}

C_OUTPUT=$(./benchmarks/sum_loop_bench_c)
echo "$C_OUTPUT"
C_OPS=$(echo "$C_OUTPUT" | grep "Ops/sec:" | awk '{print $2}')
echo "C,-O3,$C_OPS" >> "$RESULTS_FILE"
echo ""

# ===================================
# 2. Python版
# ===================================
echo -e "${YELLOW}2️⃣ Python 3${NC}"
echo "-----------------------------------"

chmod +x benchmarks/sum_loop_bench.py
PYTHON_OUTPUT=$(python3 benchmarks/sum_loop_bench.py)
echo "$PYTHON_OUTPUT"
PYTHON_OPS=$(echo "$PYTHON_OUTPUT" | grep "Ops/sec:" | awk '{print $2}')
echo "Python,3.x,$PYTHON_OPS" >> "$RESULTS_FILE"
echo ""

# ===================================
# 3. Nyash VM版
# ===================================
echo -e "${GREEN}3️⃣ Nyash VM${NC}"
echo "-----------------------------------"

NYASH_DISABLE_PLUGINS=1 ./target/release/hako --backend vm benchmarks/sum_loop_bench.hako > /tmp/nyash_vm.txt 2>&1 || true
cat /tmp/nyash_vm.txt | grep -E "Iterations:|Elapsed:|Ops/sec:|Sum:"
VM_OPS=$(cat /tmp/nyash_vm.txt | grep "Ops/sec:" | awk '{print $2}')
echo "Nyash,VM,$VM_OPS" >> "$RESULTS_FILE"
echo ""

# ===================================
# 4. Nyash LLVM版
# ===================================
echo -e "${GREEN}4️⃣ Nyash LLVM${NC}"
echo "-----------------------------------"

NYASH_DISABLE_PLUGINS=1 NYASH_LLVM_USE_HARNESS=1 ./target/release/hako --backend llvm benchmarks/sum_loop_bench.hako > /tmp/nyash_llvm.txt 2>&1 || true
cat /tmp/nyash_llvm.txt | grep -E "Iterations:|Elapsed:|Ops/sec:|Sum:"
LLVM_OPS=$(cat /tmp/nyash_llvm.txt | grep "Ops/sec:" | awk '{print $2}')
echo "Nyash,LLVM,$LLVM_OPS" >> "$RESULTS_FILE"
echo ""

# ===================================
# 5. 結果比較表
# ===================================
echo ""
echo "===================================================="
echo "📊 Results Summary"
echo "===================================================="
echo ""

# 最大値を探す（基準）
MAX_OPS=$(awk -F',' '{print $3}' "$RESULTS_FILE" | sort -rn | head -1)

echo "| Language    | Backend | Ops/sec      | Relative |"
echo "|-------------|---------|--------------|----------|"

while IFS=',' read -r lang backend ops; do
    if [ -n "$ops" ] && [ "$ops" != "0" ]; then
        # 相対速度計算（整数演算）
        relative=$(awk "BEGIN {printf \"%.2f\", $ops / $MAX_OPS}")
        printf "| %-11s | %-7s | %12s | %8s |\n" "$lang" "$backend" "$ops" "${relative}x"
    fi
done < "$RESULTS_FILE"

echo ""
echo "💾 Detailed results saved to: $RESULTS_FILE"
echo ""
echo "🏆 Winner: $(head -1 "$RESULTS_FILE" | awk -F',' '{print $1" "$2}')"
