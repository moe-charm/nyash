#!/bin/bash
# ベンチマーク比較スクリプト: Hakorune vs Python vs C
# 使い方: bash tools/bench_compare.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BENCH_DIR="$PROJECT_ROOT/apps/benchmarks"
MICRO_DIR="$BENCH_DIR/micro"
RESULTS_DIR="$PROJECT_ROOT/tmp/bench_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# 色定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

mkdir -p "$RESULTS_DIR"

echo "========================================"
echo "🔥 Hakorune Benchmark Comparison"
echo "========================================"
echo ""
echo "📊 言語: Hakorune vs Python vs C"
echo "📁 結果: $RESULTS_DIR/bench_compare_$TIMESTAMP.txt"
echo ""

RESULT_FILE="$RESULTS_DIR/bench_compare_$TIMESTAMP.txt"

# ヘッダー
{
    echo "Hakorune Benchmark Comparison Results"
    echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
    echo "========================================="
    echo ""
} > "$RESULT_FILE"

# C言語のコンパイル
echo "🔨 Compiling C benchmarks..."
C_EXES=()
for c_file in "$MICRO_DIR"/{01,02_fibonacci_loop,03,04,05,06,07,08}_*.c; do
    if [ -f "$c_file" ]; then
        base=$(basename "$c_file" .c)
        exe="$MICRO_DIR/${base}_exe"
        echo "  Compiling: $base.c → ${base}_exe"
        gcc -O2 -o "$exe" "$c_file"
        C_EXES+=("$exe")
    fi
done
echo ""

# ベンチマーク定義
declare -A BENCHMARKS
BENCHMARKS=(
    ["01_counter"]="カウンター"
    ["02_fibonacci"]="フィボナッチ"
    ["03_prime_check"]="素数判定"
    ["04_function_call_overhead"]="関数呼び出し"
    ["05_box_allocation"]="Box/Object作成"
    ["06_array_operations"]="配列操作"
    ["07_string_concat"]="文字列結合"
    ["08_map_operations"]="Map操作"
)

declare -A EXPECTED
EXPECTED=(
    ["01_counter"]="10"
    ["02_fibonacci"]="89"
    ["03_prime_check"]="1"
    ["04_function_call_overhead"]="1000"
    ["05_box_allocation"]="999"
    ["06_array_operations"]="499500"
    ["07_string_concat"]="100"
    ["08_map_operations"]="499500"
)

# 実行関数
run_benchmark() {
    local lang=$1
    local cmd=$2
    local name=$3
    local warmup=${4:-2}
    local repeat=${5:-10}

    # Warmup
    for ((i=0; i<warmup; i++)); do
        eval "$cmd" > /dev/null 2>&1
    done

    # Measure
    local total_ms=0
    local min_ms=9999999
    local max_ms=0

    for ((i=0; i<repeat; i++)); do
        local start=$(date +%s%3N)
        local output=$(eval "$cmd" 2>&1)
        local end=$(date +%s%3N)
        local elapsed=$((end - start))

        total_ms=$((total_ms + elapsed))
        [ $elapsed -lt $min_ms ] && min_ms=$elapsed
        [ $elapsed -gt $max_ms ] && max_ms=$elapsed
    done

    local avg_ms=$((total_ms / repeat))

    # 結果検証
    local result=$(echo "$output" | grep "Result:" | awk '{print $2}')
    local expected=${EXPECTED[$name]}
    local status="✅"
    if [ "$result" != "$expected" ]; then
        status="❌ (expected: $expected, got: $result)"
    fi

    echo "$lang,$name,$avg_ms,$min_ms,$max_ms,$result,$expected,$status"
}

echo "🏃 Running benchmarks (warmup=2, repeat=10)..."
echo ""

# 結果テーブルヘッダー
{
    echo "| ベンチマーク | Hakorune (ms) | Python (ms) | C (ms) | Hakorune/C | Python/C |"
    echo "|-------------|--------------|-------------|--------|-----------|----------|"
} >> "$RESULT_FILE"

# 各ベンチマークを実行
for bench_name in 01_counter 02_fibonacci 03_prime_check 04_function_call_overhead 05_box_allocation 06_array_operations 07_string_concat 08_map_operations; do
    japanese_name=${BENCHMARKS[$bench_name]}

    echo -e "${CYAN}📊 $japanese_name ($bench_name)${NC}"

    # Hakorune
    hakorune_file="$BENCH_DIR/${bench_name}.nyash"
    if [ -f "$hakorune_file" ]; then
        hakorune_result=$(run_benchmark "Hakorune" "NYASH_QUIET=1 '$PROJECT_ROOT/target/release/hako' '$hakorune_file'" "$bench_name")
        hakorune_ms=$(echo "$hakorune_result" | cut -d',' -f3)
        hakorune_status=$(echo "$hakorune_result" | cut -d',' -f8-)
        echo -e "  ${GREEN}Hakorune${NC}: ${hakorune_ms}ms $hakorune_status"
    else
        hakorune_ms="N/A"
        echo -e "  ${YELLOW}Hakorune${NC}: File not found"
    fi

    # Python
    if [ "$bench_name" == "02_fibonacci" ]; then
        # fibonacciはループ版を使う
        python_file="$MICRO_DIR/02_fibonacci_loop.py"
    else
        python_file="$MICRO_DIR/${bench_name//_overhead/}.py"
    fi

    if [ -f "$python_file" ]; then
        python_result=$(run_benchmark "Python" "python3 '$python_file'" "$bench_name")
        python_ms=$(echo "$python_result" | cut -d',' -f3)
        python_status=$(echo "$python_result" | cut -d',' -f8-)
        echo -e "  ${BLUE}Python${NC}:   ${python_ms}ms $python_status"
    else
        python_ms="N/A"
        echo -e "  ${YELLOW}Python${NC}:   File not found ($python_file)"
    fi

    # C
    if [ "$bench_name" == "02_fibonacci" ]; then
        c_exe="$MICRO_DIR/02_fibonacci_loop_exe"
    else
        c_exe="$MICRO_DIR/${bench_name//_overhead/}_exe"
    fi

    if [ -f "$c_exe" ]; then
        c_result=$(run_benchmark "C" "'$c_exe'" "$bench_name")
        c_ms=$(echo "$c_result" | cut -d',' -f3)
        c_status=$(echo "$c_result" | cut -d',' -f8-)
        echo -e "  ${RED}C${NC}:        ${c_ms}ms $c_status"
    else
        c_ms="N/A"
        echo -e "  ${YELLOW}C${NC}:        Executable not found ($c_exe)"
    fi

    # 比率計算
    if [ "$hakorune_ms" != "N/A" ] && [ "$c_ms" != "N/A" ] && [ "$c_ms" -gt 0 ]; then
        hakorune_c_ratio=$(awk "BEGIN {printf \"%.2f\", $hakorune_ms / $c_ms}")
    else
        hakorune_c_ratio="N/A"
    fi

    if [ "$python_ms" != "N/A" ] && [ "$c_ms" != "N/A" ] && [ "$c_ms" -gt 0 ]; then
        python_c_ratio=$(awk "BEGIN {printf \"%.2f\", $python_ms / $c_ms}")
    else
        python_c_ratio="N/A"
    fi

    # 結果テーブルに追記
    {
        echo "| $japanese_name | $hakorune_ms | $python_ms | $c_ms | ${hakorune_c_ratio}x | ${python_c_ratio}x |"
    } >> "$RESULT_FILE"

    echo ""
done

echo "========================================"
echo "✅ Benchmark complete!"
echo "📄 Results saved to: $RESULT_FILE"
echo ""
echo "📊 Summary:"
cat "$RESULT_FILE"
echo ""
