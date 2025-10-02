#!/usr/bin/env bash
# VM Benchmark System - Rust VM Only
# Rust VMベンチマークシステム（簡易版）

set -euo pipefail

# 色付き出力
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ベンチマークファイル定義
BENCH_DIR="apps/benchmarks"
BENCHMARKS=(
    "01_counter.nyash:10:カウンター"
    "02_fibonacci.nyash:89:フィボナッチ"
    "03_prime_check.nyash:1:素数判定"
)

# 結果ディレクトリ
RESULTS_DIR="tmp/bench_results"
mkdir -p "$RESULTS_DIR"

# ログファイル
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULT_JSON="$RESULTS_DIR/bench_vm_${TIMESTAMP}.json"

echo -e "${BLUE}🔥 HakoRune VM Benchmark System${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# JSON初期化
echo "{" > "$RESULT_JSON"
echo '  "timestamp": "'$(date -Iseconds)'",' >> "$RESULT_JSON"
echo '  "backend": "Rust VM",' >> "$RESULT_JSON"
echo '  "benchmarks": {' >> "$RESULT_JSON"

first_bench=true

# 各ベンチマークを実行
for bench_entry in "${BENCHMARKS[@]}"; do
    IFS=':' read -r bench_file expected_result bench_name <<< "$bench_entry"
    bench_path="$BENCH_DIR/$bench_file"

    if [ ! -f "$bench_path" ]; then
        echo -e "${RED}✗ ベンチマークファイルが見つかりません: $bench_path${NC}"
        continue
    fi

    echo -e "${YELLOW}📊 ベンチマーク: $bench_name${NC} ($bench_file)"

    # JSON区切り
    if [ "$first_bench" = false ]; then
        echo "    ," >> "$RESULT_JSON"
    fi
    first_bench=false

    echo "    \"$bench_file\": {" >> "$RESULT_JSON"
    echo "      \"name\": \"$bench_name\"," >> "$RESULT_JSON"
    echo "      \"expected\": $expected_result," >> "$RESULT_JSON"

    # 実行時間計測（5回実行して平均）
    vm_times=()
    results=()

    for i in {1..5}; do
        start_ns=$(date +%s%N)
        result=$(NYASH_QUIET=1 ./target/release/hako "$bench_path" 2>&1 | grep "^Result:" | sed 's/Result: //' | tr -d '\n')
        end_ns=$(date +%s%N)
        elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
        vm_times+=($elapsed_ms)
        results+=($result)
    done

    # 平均時間計算
    total_time=0
    for time in "${vm_times[@]}"; do
        total_time=$((total_time + time))
    done
    vm_avg=$((total_time / 5))

    # 最小・最大時間
    vm_min=${vm_times[0]}
    vm_max=${vm_times[0]}
    for time in "${vm_times[@]}"; do
        if [ "$time" -lt "$vm_min" ]; then
            vm_min=$time
        fi
        if [ "$time" -gt "$vm_max" ]; then
            vm_max=$time
        fi
    done

    # 結果検証
    result=${results[0]}
    if [ "$result" = "$expected_result" ]; then
        echo -e "  ${GREEN}✓${NC} 結果: $result (期待値: $expected_result) ${GREEN}OK${NC}"
        vm_status="PASS"
    else
        echo -e "  ${RED}✗${NC} 結果: $result (期待値: $expected_result) ${RED}FAIL${NC}"
        vm_status="FAIL"
    fi

    echo -e "  ⏱  平均時間: ${GREEN}${vm_avg}ms${NC}"
    echo -e "  📊 最小/最大: ${vm_min}ms / ${vm_max}ms"
    echo -e "  📈 5回の実行: ${vm_times[0]}ms, ${vm_times[1]}ms, ${vm_times[2]}ms, ${vm_times[3]}ms, ${vm_times[4]}ms"
    echo ""

    # JSON出力
    echo "      \"result\": $result," >> "$RESULT_JSON"
    echo "      \"time_ms_avg\": $vm_avg," >> "$RESULT_JSON"
    echo "      \"time_ms_min\": $vm_min," >> "$RESULT_JSON"
    echo "      \"time_ms_max\": $vm_max," >> "$RESULT_JSON"
    echo "      \"times\": [${vm_times[0]}, ${vm_times[1]}, ${vm_times[2]}, ${vm_times[3]}, ${vm_times[4]}]," >> "$RESULT_JSON"
    echo "      \"status\": \"$vm_status\"" >> "$RESULT_JSON"
    echo "    }" >> "$RESULT_JSON"
done

# JSON終了
echo "  }" >> "$RESULT_JSON"
echo "}" >> "$RESULT_JSON"

echo -e "${GREEN}✅ ベンチマーク完了！${NC}"
echo -e "${BLUE}📄 結果: $RESULT_JSON${NC}"
echo ""

# サマリー表示
echo -e "${YELLOW}📊 サマリー${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
printf "%-20s %-12s %-12s %-10s\n" "ベンチマーク" "平均 (ms)" "最小 (ms)" "最大 (ms)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# JSONから結果抽出してサマリー表示
for bench_entry in "${BENCHMARKS[@]}"; do
    IFS=':' read -r bench_file expected_result bench_name <<< "$bench_entry"
    printf "%-20s " "$bench_name"

    # 平均時間抽出
    avg_time=$(grep -A 10 "\"$bench_file\"" "$RESULT_JSON" | grep '"time_ms_avg"' | grep -oP '\d+')
    printf "%-12s " "${avg_time}ms"

    # 最小時間抽出
    min_time=$(grep -A 10 "\"$bench_file\"" "$RESULT_JSON" | grep '"time_ms_min"' | grep -oP '\d+')
    printf "%-12s " "${min_time}ms"

    # 最大時間抽出
    max_time=$(grep -A 10 "\"$bench_file\"" "$RESULT_JSON" | grep '"time_ms_max"' | grep -oP '\d+')
    printf "%-10s\n" "${max_time}ms"
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${GREEN}🎉 Rust VMベンチマーク完了！${NC}"
