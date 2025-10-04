#!/usr/bin/env bash
# Unified Benchmark System - Rust VM, LLVM, WASM
# ChatGPT Pro Design (apps/benchmarks/DESIGN.md)
# Updated: 2025-10-03 - 2-Phase separation (Preparation vs Measurement)

set -euo pipefail

# 色付き出力
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ==================== Configuration ====================
DEFAULT_WARMUP=10
DEFAULT_REPEAT=50
DEFAULT_BACKEND="all"  # all, vm, llvm, wasm

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

# Temp directory
TMP_DIR="/tmp/hakorune_bench_$$"
mkdir -p "$TMP_DIR"

# ==================== Argument Parsing ====================
BACKEND="$DEFAULT_BACKEND"
WARMUP="$DEFAULT_WARMUP"
REPEAT="$DEFAULT_REPEAT"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --backend)
            BACKEND="$2"
            shift 2
            ;;
        --warmup)
            WARMUP="$2"
            shift 2
            ;;
        --repeat)
            REPEAT="$2"
            shift 2
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Usage: $0 [--backend all|vm|llvm|wasm] [--warmup N] [--repeat N]"
            exit 1
            ;;
    esac
done

# ログファイル
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULT_JSON="$RESULTS_DIR/bench_${TIMESTAMP}.json"

# ==================== Helper Functions ====================
cleanup() {
    if [[ -d "$TMP_DIR" ]]; then
        rm -rf "$TMP_DIR"
    fi
}
# NOTE: trap cleanup EXIT removed to prevent premature cleanup during Phase 1 builds
# cleanup is now called explicitly after Phase 1 and at script end

# Get current time in nanoseconds
get_time_ns() {
    date +%s%N
}

# Calculate statistics
calc_mean() {
    local sum=0
    local count=$#
    for val in "$@"; do
        sum=$((sum + val))
    done
    echo $((sum / count))
}

calc_median() {
    local sorted=($(printf '%s\n' "$@" | sort -n))
    local count=${#sorted[@]}
    local mid=$((count / 2))

    if (( count % 2 == 0 )); then
        echo $(( (sorted[mid-1] + sorted[mid]) / 2 ))
    else
        echo "${sorted[$mid]}"
    fi
}

# ==================== Banner ====================
echo -e "${BLUE}🔥 HakoRune Unified Benchmark System${NC}"
echo -e "${BLUE}====================================${NC}"
echo -e "${BLUE}Design: ChatGPT Pro (apps/benchmarks/DESIGN.md)${NC}"
echo -e "${BLUE}Principle: Preparation vs Measurement separation!${NC}"
echo -e "${BLUE}Backend: ${BACKEND}${NC}"
echo -e "${BLUE}Warmup: ${WARMUP} iterations${NC}"
echo -e "${BLUE}Repeat: ${REPEAT} iterations${NC}"
echo ""

# ==================== Pre-build nyash (avoid Cargo lock conflicts) ====================
if [[ "$BACKEND" == "all" || "$BACKEND" == "llvm" ]]; then
    echo -e "${BLUE}🔧 Pre-building nyash with LLVM features...${NC}"
    LLVM_FEATURE=${NYASH_LLVM_FEATURE:-llvm}
    if CARGO_INCREMENTAL=1 cargo build --release -j 24 -p nyash-rust --features "$LLVM_FEATURE" >/dev/null 2>&1; then
        echo -e "  ${GREEN}✓${NC} nyash binary ready"
    else
        echo -e "  ${RED}✗${NC} nyash build failed"
        exit 1
    fi

    echo -e "${BLUE}🔧 Pre-building Nyash Kernel...${NC}"
    if ( cd crates/nyash_kernel && cargo build --release -j 24 >/dev/null 2>&1 ); then
        echo -e "  ${GREEN}✓${NC} Nyash Kernel ready"
    else
        echo -e "  ${RED}✗${NC} Nyash Kernel build failed"
        exit 1
    fi

    # Export flag to skip cargo build in build_llvm.sh
    export NYASH_BENCH_SKIP_NYASH_BUILD=1
    echo ""
fi

# ==================== Phase 1: Preparation ====================
echo -e "${YELLOW}📦 Phase 1: Preparation (build once, NOT measured)${NC}"
echo ""

declare -A LLVM_EXES
declare -A WASM_FILES

for bench_entry in "${BENCHMARKS[@]}"; do
    IFS=':' read -r bench_file expected_result bench_name <<< "$bench_entry"
    bench_path="$BENCH_DIR/$bench_file"
    bench_stem="${bench_file%.nyash}"

    if [ ! -f "$bench_path" ]; then
        echo -e "${RED}✗ Benchmark not found: $bench_path${NC}"
        continue
    fi

    # LLVM Preparation
    if [[ "$BACKEND" == "all" || "$BACKEND" == "llvm" ]]; then
        echo -n "  [LLVM] Building $bench_name... "
        TMP_LLVM_EXE="$TMP_DIR/${bench_stem}_llvm"
        if env NYASH_MIR_UNIFIED_CALL=1 NYASH_LLVM_AUTO_SAFEPOINT=${NYASH_LLVM_AUTO_SAFEPOINT:-0} \
             bash tools/build_llvm.sh "$bench_path" -o "$TMP_LLVM_EXE" >/dev/null 2>&1; then
            LLVM_EXES["$bench_file"]="$TMP_LLVM_EXE"
            echo -e "${GREEN}✓${NC} ($(stat -c%s "$TMP_LLVM_EXE" | numfmt --to=iec))"
        else
            echo -e "${RED}✗ Build failed${NC}"
        fi
    fi

    # WASM Preparation
    if [[ "$BACKEND" == "all" || "$BACKEND" == "wasm" ]]; then
        echo -n "  [WASM] Building $bench_name... "
        TMP_MIR_JSON="$TMP_DIR/${bench_stem}.json"
        TMP_WASM="$TMP_DIR/${bench_stem}.wasm"

        env NYASH_DISABLE_PLUGINS=1 ./target/release/hako --emit-mir-json "$TMP_MIR_JSON" "$bench_path" >/dev/null 2>&1

        if [[ ! -f "$TMP_MIR_JSON" ]]; then
            echo -e "${RED}✗ MIR JSON generation failed${NC}"
            continue
        fi

        export NYASH_LLVM_AUTO_SAFEPOINT=0
        if bash tools/build_wasm.sh "$TMP_MIR_JSON" -o "$TMP_WASM" >/dev/null 2>&1; then
            WASM_FILES["$bench_file"]="$TMP_WASM"
            echo -e "${GREEN}✓${NC} ($(stat -c%s "$TMP_WASM") bytes)"
        else
            echo -e "${RED}✗ Build failed${NC}"
        fi
    fi
done

echo ""
echo -e "${YELLOW}⏱  Phase 2: Measurement (run N times, MEASURED)${NC}"
echo ""

# JSON初期化
echo "{" > "$RESULT_JSON"
echo "  \"timestamp\": \"$(date -Iseconds)\"," >> "$RESULT_JSON"
echo "  \"config\": {" >> "$RESULT_JSON"
echo "    \"backend\": \"$BACKEND\"," >> "$RESULT_JSON"
echo "    \"warmup\": $WARMUP," >> "$RESULT_JSON"
echo "    \"repeat\": $REPEAT" >> "$RESULT_JSON"
echo "  }," >> "$RESULT_JSON"
echo "  \"benchmarks\": {" >> "$RESULT_JSON"

first_bench=true

# ==================== Phase 2: Measurement Loop ====================
for bench_entry in "${BENCHMARKS[@]}"; do
    IFS=':' read -r bench_file expected_result bench_name <<< "$bench_entry"
    bench_path="$BENCH_DIR/$bench_file"

    if [ ! -f "$bench_path" ]; then
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
    if [[ "$BACKEND" == "all" || "$BACKEND" == "vm" ]]; then
        echo -e "  ${BLUE}[1/3] Rust VM${NC}"

        # Warmup
        echo -n "    Warmup... "
        for i in $(seq 1 $WARMUP); do
            env NYASH_QUIET=1 NYASH_DISABLE_PLUGINS=1 ./target/release/hako "$bench_path" >/dev/null 2>&1
        done
        echo -e "${GREEN}✓${NC}"

        # Measurement
        vm_times=()
        for i in $(seq 1 $REPEAT); do
            start_ns=$(get_time_ns)
            result=$(env NYASH_QUIET=1 NYASH_DISABLE_PLUGINS=1 ./target/release/hako "$bench_path" 2>&1 | grep "^Result:" | sed 's/Result: //' | tr -d '\n')
            end_ns=$(get_time_ns)
            elapsed_ns=$((end_ns - start_ns))
            vm_times+=($elapsed_ns)
        done

        # 統計計算
        vm_mean_ns=$(calc_mean "${vm_times[@]}")
        vm_median_ns=$(calc_median "${vm_times[@]}")
        vm_mean_ms=$((vm_mean_ns / 1000000))
        vm_median_ms=$((vm_median_ns / 1000000))

        # 結果検証
        if [ "$result" = "$expected_result" ]; then
            echo -e "    ${GREEN}✓${NC} 結果: $result (期待値: $expected_result) ${GREEN}OK${NC}"
            vm_status="PASS"
        else
            echo -e "    ${RED}✗${NC} 結果: $result (期待値: $expected_result) ${RED}FAIL${NC}"
            vm_status="FAIL"
        fi
        echo -e "    ⏱  Mean: ${vm_mean_ms}ms | Median: ${vm_median_ms}ms"
        echo ""

        # JSON出力
        echo "        \"vm\": {" >> "$RESULT_JSON"
        echo "          \"result\": $result," >> "$RESULT_JSON"
        echo "          \"mean_ns\": $vm_mean_ns," >> "$RESULT_JSON"
        echo "          \"mean_ms\": $vm_mean_ms," >> "$RESULT_JSON"
        echo "          \"median_ns\": $vm_median_ns," >> "$RESULT_JSON"
        echo "          \"median_ms\": $vm_median_ms," >> "$RESULT_JSON"
        echo "          \"status\": \"$vm_status\"" >> "$RESULT_JSON"
        echo "        }," >> "$RESULT_JSON"
    fi

    #
    # 2. LLVM ベンチマーク（準備済み実行ファイル使用）
    #
    if [[ "$BACKEND" == "all" || "$BACKEND" == "llvm" ]]; then
        echo -e "  ${BLUE}[2/3] LLVM (pre-built executable)${NC}"

        TMP_LLVM_EXE="${LLVM_EXES[$bench_file]:-}"
        if [[ -z "$TMP_LLVM_EXE" || ! -x "$TMP_LLVM_EXE" ]]; then
            echo -e "    ${RED}✗ Executable not available${NC}"
            echo "        \"llvm\": { \"status\": \"ERROR\" }," >> "$RESULT_JSON"
        else
            # Warmup
            echo -n "    Warmup... "
            for i in $(seq 1 $WARMUP); do
                env NYASH_DISABLE_PLUGINS=1 NYASH_NYRT_SILENT_RESULT=1 "$TMP_LLVM_EXE" >/dev/null 2>&1
            done
            echo -e "${GREEN}✓${NC}"

            # Measurement
            llvm_times=()
            for i in $(seq 1 $REPEAT); do
                start_ns=$(get_time_ns)
                result=$(env NYASH_DISABLE_PLUGINS=1 NYASH_NYRT_SILENT_RESULT=1 "$TMP_LLVM_EXE" 2>&1 | grep "^Result:" | head -1 | sed 's/Result: //' | tr -d '\n')
                end_ns=$(get_time_ns)
                elapsed_ns=$((end_ns - start_ns))
                llvm_times+=($elapsed_ns)
            done

            # 統計計算
            llvm_mean_ns=$(calc_mean "${llvm_times[@]}")
            llvm_median_ns=$(calc_median "${llvm_times[@]}")
            llvm_mean_ms=$((llvm_mean_ns / 1000000))
            llvm_median_ms=$((llvm_median_ns / 1000000))

            # 結果検証
            if [ "$result" = "$expected_result" ]; then
                echo -e "    ${GREEN}✓${NC} 結果: $result (期待値: $expected_result) ${GREEN}OK${NC}"
                llvm_status="PASS"
            else
                echo -e "    ${RED}✗${NC} 結果: $result (期待値: $expected_result) ${RED}FAIL${NC}"
                llvm_status="FAIL"
            fi
            echo -e "    ⏱  Mean: ${llvm_mean_ms}ms | Median: ${llvm_median_ms}ms"
            echo ""

            # JSON出力
            echo "        \"llvm\": {" >> "$RESULT_JSON"
            echo "          \"result\": $result," >> "$RESULT_JSON"
            echo "          \"mean_ns\": $llvm_mean_ns," >> "$RESULT_JSON"
            echo "          \"mean_ms\": $llvm_mean_ms," >> "$RESULT_JSON"
            echo "          \"median_ns\": $llvm_median_ns," >> "$RESULT_JSON"
            echo "          \"median_ms\": $llvm_median_ms," >> "$RESULT_JSON"
            echo "          \"status\": \"$llvm_status\"" >> "$RESULT_JSON"
            echo "        }," >> "$RESULT_JSON"
        fi
    fi

    #
    # 3. WASM ベンチマーク（準備済み.wasm使用）
    #
    if [[ "$BACKEND" == "all" || "$BACKEND" == "wasm" ]]; then
        echo -e "  ${BLUE}[3/3] WASM (pre-built .wasm)${NC}"

        TMP_WASM="${WASM_FILES[$bench_file]:-}"
        if [[ -z "$TMP_WASM" || ! -f "$TMP_WASM" ]]; then
            echo -e "    ${RED}✗ WASM file not available${NC}"
            echo "        \"wasm\": { \"status\": \"ERROR\" }" >> "$RESULT_JSON"
        else
            # Warmup
            echo -n "    Warmup... "
            for i in $(seq 1 $WARMUP); do
                node tools/wasm_runner.js "$TMP_WASM" >/dev/null 2>&1
            done
            echo -e "${GREEN}✓${NC}"

            # Measurement
            wasm_times=()
            for i in $(seq 1 $REPEAT); do
                start_ns=$(get_time_ns)
                result=$(node tools/wasm_runner.js "$TMP_WASM" 2>&1 | grep -oP 'returned: \K\d+' | head -1)
                end_ns=$(get_time_ns)
                elapsed_ns=$((end_ns - start_ns))
                wasm_times+=($elapsed_ns)
            done

            # 統計計算
            wasm_mean_ns=$(calc_mean "${wasm_times[@]}")
            wasm_median_ns=$(calc_median "${wasm_times[@]}")
            wasm_mean_ms=$((wasm_mean_ns / 1000000))
            wasm_median_ms=$((wasm_median_ns / 1000000))

            # 結果検証
            if [ "$result" = "$expected_result" ]; then
                echo -e "    ${GREEN}✓${NC} 結果: $result (期待値: $expected_result) ${GREEN}OK${NC}"
                wasm_status="PASS"
            else
                echo -e "    ${RED}✗${NC} 結果: $result (期待値: $expected_result) ${RED}FAIL${NC}"
                wasm_status="FAIL"
            fi
            echo -e "    ⏱  Mean: ${wasm_mean_ms}ms | Median: ${wasm_median_ms}ms"
            echo ""

            # JSON出力
            echo "        \"wasm\": {" >> "$RESULT_JSON"
            echo "          \"result\": $result," >> "$RESULT_JSON"
            echo "          \"mean_ns\": $wasm_mean_ns," >> "$RESULT_JSON"
            echo "          \"mean_ms\": $wasm_mean_ms," >> "$RESULT_JSON"
            echo "          \"median_ns\": $wasm_median_ns," >> "$RESULT_JSON"
            echo "          \"median_ms\": $wasm_median_ms," >> "$RESULT_JSON"
            echo "          \"status\": \"$wasm_status\"" >> "$RESULT_JSON"
            echo "        }" >> "$RESULT_JSON"
        fi
    fi

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

# ==================== Summary Table ====================
echo -e "${YELLOW}📊 サマリー${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
printf "%-20s %-12s %-12s %-12s\n" "ベンチマーク" "VM (ms)" "LLVM (ms)" "WASM (ms)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

for bench_entry in "${BENCHMARKS[@]}"; do
    IFS=':' read -r bench_file expected_result bench_name <<< "$bench_entry"
    printf "%-20s " "$bench_name"

    # VM時間抽出
    if [[ "$BACKEND" == "all" || "$BACKEND" == "vm" ]]; then
        vm_time=$(grep -A 30 "\"$bench_file\"" "$RESULT_JSON" | grep '\"mean_ms\"' | head -1 | grep -oP '\d+' || echo "N/A")
        printf "%-12s " "${vm_time}"
    else
        printf "%-12s " "-"
    fi

    # LLVM時間抽出
    if [[ "$BACKEND" == "all" || "$BACKEND" == "llvm" ]]; then
        llvm_time=$(grep -A 50 "\"$bench_file\"" "$RESULT_JSON" | grep -A 10 '\"llvm\"' | grep '\"mean_ms\"' | grep -oP '\d+' || echo "N/A")
        printf "%-12s " "${llvm_time}"
    else
        printf "%-12s " "-"
    fi

    # WASM時間抽出
    if [[ "$BACKEND" == "all" || "$BACKEND" == "wasm" ]]; then
        wasm_time=$(grep -A 50 "\"$bench_file\"" "$RESULT_JSON" | grep -A 10 '\"wasm\"' | grep '\"mean_ms\"' | grep -oP '\d+' || echo "N/A")
        printf "%-12s\n" "${wasm_time}"
    else
        printf "%-12s\n" "-"
    fi
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${GREEN}🎉 ベンチマークシステム完了！${NC}"

# 一時ディレクトリクリーンアップ（Phase 2完了後）
cleanup
