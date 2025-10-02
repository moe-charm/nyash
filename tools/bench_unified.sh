#!/usr/bin/env bash
# Unified Benchmark System - Rust VM, LLVM, WASM
# 統一ベンチマークシステム

set -euo pipefail

# 色付き出力
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ベンチマークファイル定義
BENCH_DIR="local_tests/bench"
BENCHMARKS=(
    "01_counter.nyash:10:カウンター"
    "02_fibonacci.nyash:55:フィボナッチ"
    "03_prime_check.nyash:1:素数判定"
)

# 結果ディレクトリ
RESULTS_DIR="tmp/bench_results"
mkdir -p "$RESULTS_DIR"

# ログファイル
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULT_JSON="$RESULTS_DIR/bench_${TIMESTAMP}.json"

echo -e "${BLUE}🔥 HakoRune Unified Benchmark System${NC}"
echo -e "${BLUE}====================================${NC}"
echo ""

# JSON初期化
echo "{" > "$RESULT_JSON"
echo '  "timestamp": "'$(date -Iseconds)'",' >> "$RESULT_JSON"
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
    echo ""

    # JSON区切り
    if [ "$first_bench" = false ]; then
        echo "    ," >> "$RESULT_JSON"
    fi
    first_bench=false

    echo "    \"$bench_file\": {" >> "$RESULT_JSON"
    echo "      \"name\": \"$bench_name\"," >> "$RESULT_JSON"
    echo "      \"expected\": $expected_result," >> "$RESULT_JSON"
    echo "      \"results\": {" >> "$RESULT_JSON"

    #
    # 1. Rust VM ベンチマーク
    #
    echo -e "  ${BLUE}[1/3] Rust VM${NC}"

    # 出力ファイル
    vm_out="$RESULTS_DIR/${bench_file%.nyash}_vm.txt"

    # 実行時間計測（3回平均）
    vm_times=()
    for i in {1..3}; do
        start_ns=$(date +%s%N)
        result=$(env NYASH_QUIET=1 ./target/release/hako "$bench_path" 2>&1 | grep "^Result:" | sed 's/Result: //' | tr -d '\n')
        end_ns=$(date +%s%N)
        elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
        vm_times+=($elapsed_ms)
    done

    # 平均時間計算
    vm_avg=$(( (${vm_times[0]} + ${vm_times[1]} + ${vm_times[2]}) / 3 ))

    # 結果検証
    if [ "$result" = "$expected_result" ]; then
        echo -e "    ${GREEN}✓${NC} 結果: $result (期待値: $expected_result) ${GREEN}OK${NC}"
        vm_status="PASS"
    else
        echo -e "    ${RED}✗${NC} 結果: $result (期待値: $expected_result) ${RED}FAIL${NC}"
        vm_status="FAIL"
    fi
    echo -e "    ⏱  平均時間: ${vm_avg}ms (${vm_times[0]}ms, ${vm_times[1]}ms, ${vm_times[2]}ms)"
    echo ""

    # JSON出力
    echo "        \"vm\": {" >> "$RESULT_JSON"
    echo "          \"result\": $result," >> "$RESULT_JSON"
    echo "          \"time_ms\": $vm_avg," >> "$RESULT_JSON"
    echo "          \"times\": [${vm_times[0]}, ${vm_times[1]}, ${vm_times[2]}]," >> "$RESULT_JSON"
    echo "          \"status\": \"$vm_status\"" >> "$RESULT_JSON"
    echo "        }," >> "$RESULT_JSON"

    #
    # 2. LLVM ベンチマーク（llvmliteハーネス one-shot）
    #
    echo -e "  ${BLUE}[2/3] LLVM (llvmlite harness)${NC}"

    # LLVM実行時間計測（3回平均）
    llvm_times=()
    for i in {1..3}; do
        start_ns=$(date +%s%N)
        result=$(env NYASH_QUIET=1 NYASH_NYRT_SILENT_RESULT=1 NYASH_LLVM_USE_HARNESS=1 NYASH_NY_LLVM_COMPILER=target/release/ny-llvmc NYASH_EMIT_EXE_NYRT=target/release ./target/release/hako --backend llvm "$bench_path" 2>&1 | grep "^Result:" | sed 's/Result: //' | tr -d '\n')
        end_ns=$(date +%s%N)
        elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
        llvm_times+=($elapsed_ms)
    done

    # 平均時間計算
    llvm_avg=$(( (${llvm_times[0]} + ${llvm_times[1]} + ${llvm_times[2]}) / 3 ))

    # 結果検証
    if [ "$result" = "$expected_result" ]; then
        echo -e "    ${GREEN}✓${NC} 結果: $result (期待値: $expected_result) ${GREEN}OK${NC}"
        llvm_status="PASS"
    else
        echo -e "    ${RED}✗${NC} 結果: $result (期待値: $expected_result) ${RED}FAIL${NC}"
        llvm_status="FAIL"
    fi
    echo -e "    ⏱  平均時間: ${llvm_avg}ms (${llvm_times[0]}ms, ${llvm_times[1]}ms, ${llvm_times[2]}ms)"
    echo ""

    # JSON出力
    echo "        \"llvm\": {" >> "$RESULT_JSON"
    echo "          \"result\": $result," >> "$RESULT_JSON"
    echo "          \"time_ms\": $llvm_avg," >> "$RESULT_JSON"
    echo "          \"times\": [${llvm_times[0]}, ${llvm_times[1]}, ${llvm_times[2]}]," >> "$RESULT_JSON"
    echo "          \"status\": \"$llvm_status\"" >> "$RESULT_JSON"
    echo "        }," >> "$RESULT_JSON"

    #
    # 3. WASM ベンチマーク（TODO）
    #
    echo -e "  ${BLUE}[3/3] WASM${NC}"
    echo -e "    ${YELLOW}⚠ WASM実装は次のフェーズで追加予定${NC}"
    echo ""

    # JSON出力
    echo "        \"wasm\": {" >> "$RESULT_JSON"
    echo "          \"status\": \"TODO\"" >> "$RESULT_JSON"
    echo "        }" >> "$RESULT_JSON"

    echo "      }" >> "$RESULT_JSON"
    echo "    }" >> "$RESULT_JSON"

    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
done

# JSON終了
echo "  }" >> "$RESULT_JSON"
echo "}" >> "$RESULT_JSON"

echo -e "${GREEN}✅ ベンチマーク完了！${NC}"
echo -e "${BLUE}📄 結果: $RESULT_JSON${NC}"
echo ""

# 簡易サマリー表示
echo -e "${YELLOW}📊 サマリー${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
printf "%-20s %-10s %-10s %-10s\n" "ベンチマーク" "VM (ms)" "LLVM (ms)" "速度比"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# JSONから結果抽出してサマリー表示（簡易版）
for bench_entry in "${BENCHMARKS[@]}"; do
    IFS=':' read -r bench_file expected_result bench_name <<< "$bench_entry"
    printf "%-20s " "$bench_name"

    # VM時間抽出
    vm_time=$(grep -A 20 "\"$bench_file\"" "$RESULT_JSON" | grep '"time_ms"' | head -1 | grep -oP '\d+')
    printf "%-10s " "${vm_time}ms"

    # LLVM時間抽出
    llvm_time=$(grep -A 20 "\"$bench_file\"" "$RESULT_JSON" | grep '"time_ms"' | tail -1 | grep -oP '\d+')
    printf "%-10s " "${llvm_time}ms"

    # 速度比計算
    if [ -n "$vm_time" ] && [ -n "$llvm_time" ] && [ "$llvm_time" -gt 0 ]; then
        ratio=$(awk "BEGIN {printf \"%.2f\", $vm_time / $llvm_time}")
        printf "%.2fx\n" "$ratio"
    else
        printf "N/A\n"
    fi
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${GREEN}🎉 ベンチマークシステム完了！${NC}"
