#!/bin/bash
# Phase 15.8: WASM Performance Benchmark Runner
# Compares WASM execution performance across different test cases

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo "=========================================="
echo "Phase 15.8: WASM Benchmark Suite"
echo "=========================================="
echo ""

# Check dependencies
if ! command -v node &> /dev/null; then
    echo -e "${RED}✗ Node.js not found${NC}"
    exit 1
fi

if ! python3 -c "import llvmlite" 2>/dev/null; then
    echo -e "${RED}✗ llvmlite not found${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Dependencies OK${NC}"
echo ""

# Benchmark configurations
declare -A BENCHMARKS=(
    ["fibonacci"]="/tmp/bench_fibonacci.json"
    ["factorial"]="/tmp/bench_factorial.json"
    ["sum_loop"]="/tmp/bench_sum_loop.json"
)

declare -A EXPECTED_RESULTS=(
    ["fibonacci"]="610"
    ["factorial"]="3628800"
    ["sum_loop"]="49995000"
)

declare -A DESCRIPTIONS=(
    ["fibonacci"]="Recursive Fibonacci(15)"
    ["factorial"]="Recursive Factorial(10)"
    ["sum_loop"]="Loop Sum(0..10000)"
)

TEMP_DIR="/tmp/wasm_bench_$$"
mkdir -p "$TEMP_DIR"

# Benchmark runner
run_benchmark() {
    local name=$1
    local json_path=$2
    local expected=$3
    local description=$4

    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Test: ${name}${NC}"
    echo -e "Description: ${description}"
    echo ""

    local wasm_path="$TEMP_DIR/${name}.wasm"

    # Build WASM
    echo -n "Building WASM... "
    if "$PROJECT_ROOT/tools/build_wasm.sh" "$json_path" -o "$wasm_path" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗ Build failed${NC}"
        return 1
    fi

    # Run benchmark with timing
    echo -n "Running benchmark... "

    # Create Node.js benchmark runner
    cat > "$TEMP_DIR/bench_runner.js" <<'EOF'
const fs = require('fs');
const wasmPath = process.argv[2];

// Runtime functions
const nyashRuntime = {
    'nyash.console.log': (strPtr) => strPtr,
    'nyash.box.from_i8_string': (ptr) => BigInt(ptr),
    'nyash.string.to_i8p_h': (handle) => {
        if (handle === undefined || handle === null) return 0;
        return Number(BigInt(handle) & 0xFFFFFFFFn);
    }
};

const wasiRuntime = {
    ny_check_safepoint: () => {},
    fd_write: () => 0,
    proc_exit: () => process.exit(0)
};

const importObject = {
    env: {
        __linear_memory: new WebAssembly.Memory({ initial: 1 }),
        ...nyashRuntime,
        ...wasiRuntime
    }
};

async function runBenchmark() {
    const wasmBuffer = fs.readFileSync(wasmPath);
    const { instance } = await WebAssembly.instantiate(wasmBuffer, importObject);

    const entryFn = instance.exports['Main.main'];

    // Warmup (3 runs)
    for (let i = 0; i < 3; i++) {
        entryFn();
    }

    // Benchmark (10 runs)
    const times = [];
    for (let i = 0; i < 10; i++) {
        const start = performance.now();
        const result = entryFn();
        const end = performance.now();
        times.push(end - start);
    }

    // Get final result
    const result = entryFn();

    // Calculate statistics
    const avg = times.reduce((a, b) => a + b) / times.length;
    const min = Math.min(...times);
    const max = Math.max(...times);
    const sorted = times.sort((a, b) => a - b);
    const median = sorted[Math.floor(sorted.length / 2)];

    console.log(JSON.stringify({
        result: result.toString(),
        avg: avg.toFixed(3),
        min: min.toFixed(3),
        max: max.toFixed(3),
        median: median.toFixed(3),
        runs: times.length
    }));
}

runBenchmark().catch(err => {
    console.error(JSON.stringify({ error: err.message }));
    process.exit(1);
});
EOF

    # Run benchmark
    local bench_result=$(node "$TEMP_DIR/bench_runner.js" "$wasm_path" 2>&1)

    if echo "$bench_result" | grep -q "error"; then
        echo -e "${RED}✗ Execution failed${NC}"
        echo "$bench_result"
        return 1
    fi

    echo -e "${GREEN}✓${NC}"
    echo ""

    # Parse results
    local result=$(echo "$bench_result" | jq -r '.result')
    local avg=$(echo "$bench_result" | jq -r '.avg')
    local min=$(echo "$bench_result" | jq -r '.min')
    local max=$(echo "$bench_result" | jq -r '.max')
    local median=$(echo "$bench_result" | jq -r '.median')

    # Verify result
    if [ "$result" == "$expected" ]; then
        echo -e "${GREEN}✓ Result: ${result} (correct)${NC}"
    else
        echo -e "${YELLOW}⚠ Result: ${result} (expected: ${expected})${NC}"
    fi

    # Display timing
    echo -e "Performance (10 runs):"
    echo -e "  Average: ${CYAN}${avg}ms${NC}"
    echo -e "  Median:  ${CYAN}${median}ms${NC}"
    echo -e "  Min:     ${GREEN}${min}ms${NC}"
    echo -e "  Max:     ${RED}${max}ms${NC}"
    echo ""
}

# Run all benchmarks
echo "Starting benchmark suite..."
echo ""

total=0
passed=0
failed=0

for name in "${!BENCHMARKS[@]}"; do
    json_path="${BENCHMARKS[$name]}"
    expected="${EXPECTED_RESULTS[$name]}"
    description="${DESCRIPTIONS[$name]}"

    ((total++))

    if run_benchmark "$name" "$json_path" "$expected" "$description"; then
        ((passed++))
    else
        ((failed++))
    fi
done

# Summary
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "Benchmark Summary"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "Total:  ${total}"
echo -e "Passed: ${GREEN}${passed}${NC}"
echo -e "Failed: ${RED}${failed}${NC}"
echo ""

# Cleanup
rm -rf "$TEMP_DIR"

if [ "$failed" -eq 0 ]; then
    echo -e "${GREEN}✅ All benchmarks completed successfully!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some benchmarks failed${NC}"
    exit 1
fi
