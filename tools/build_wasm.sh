#!/bin/bash
# Phase 15.8: MIR JSON → WASM Binary Build Script
# Hakorune WASM Build Pipeline

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default values
INPUT_JSON=""
OUTPUT_WASM=""
VERBOSE=0
USE_LLVM_TOOLS=0

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

usage() {
    cat <<EOF
Usage: $0 <input.json> -o <output.wasm> [options]

Phase 15.8: Build WASM binary from MIR JSON

Options:
    -o, --output FILE      Output WASM file (required)
    -v, --verbose          Verbose output
    --use-llvm-tools       Use LLC and wasm-ld instead of llvmlite
    -h, --help             Show this help

Examples:
    # Using llvmlite (direct, no toolchain needed)
    $0 test.json -o test.wasm

    # Using LLVM toolchain (if available)
    $0 test.json -o test.wasm --use-llvm-tools

Phase 15.8 Status:
    ✅ llvmlite → WASM binary generation
    ⚠️  Function export requires LLVM toolchain
    ⏸️  Full WASI support (Week 3)

Prerequisites:
    - Python 3.8+ with llvmlite
    - (Optional) LLVM toolchain 14+ (llc, wasm-ld)
    - (Optional) wabt tools (wasm-objdump, wasm2wat)

EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--output)
            OUTPUT_WASM="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=1
            shift
            ;;
        --use-llvm-tools)
            USE_LLVM_TOOLS=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            if [[ -z "$INPUT_JSON" ]]; then
                INPUT_JSON="$1"
            else
                echo -e "${RED}Error: Unknown argument: $1${NC}" >&2
                usage
                exit 1
            fi
            shift
            ;;
    esac
done

# Validate arguments
if [[ -z "$INPUT_JSON" ]]; then
    echo -e "${RED}Error: Input JSON file required${NC}" >&2
    usage
    exit 1
fi

if [[ -z "$OUTPUT_WASM" ]]; then
    echo -e "${RED}Error: Output WASM file required (-o)${NC}" >&2
    usage
    exit 1
fi

if [[ ! -f "$INPUT_JSON" ]]; then
    echo -e "${RED}Error: Input file not found: $INPUT_JSON${NC}" >&2
    exit 1
fi

echo "========================================"
echo "Phase 15.8: WASM Build Pipeline"
echo "========================================"
echo "Input:  $INPUT_JSON"
echo "Output: $OUTPUT_WASM"
echo ""

# Check Python and llvmlite
echo "1. Checking dependencies..."
if ! python3 -c "import llvmlite" 2>/dev/null; then
    echo -e "${RED}✗ llvmlite not found${NC}"
    echo "Install: pip install llvmlite"
    exit 1
fi
LLVMLITE_VER=$(python3 -c "import llvmlite; print(llvmlite.__version__)")
echo -e "${GREEN}✓ llvmlite ${LLVMLITE_VER}${NC}"

# Build WASM using llvmlite
echo ""
echo "2. Generating WASM binary..."
if [[ $VERBOSE -eq 1 ]]; then
    python3 "$PROJECT_ROOT/src/llvm_py/llvm_builder.py" "$INPUT_JSON" \
        -o "$OUTPUT_WASM" --target wasm32
else
    python3 "$PROJECT_ROOT/src/llvm_py/llvm_builder.py" "$INPUT_JSON" \
        -o "$OUTPUT_WASM" --target wasm32 2>&1 | grep -v "^$"
fi

if [[ ! -f "$OUTPUT_WASM" ]]; then
    echo -e "${RED}✗ WASM generation failed${NC}" >&2
    exit 1
fi

echo -e "${GREEN}✓ WASM binary generated${NC}"

# Add export section (workaround for llvmlite limitation)
echo ""
echo "2.5. Adding export section..."
TEMP_WASM="${OUTPUT_WASM}.tmp"
mv "$OUTPUT_WASM" "$TEMP_WASM"

python3 "$PROJECT_ROOT/tools/wasm_add_export.py" "$TEMP_WASM" "$OUTPUT_WASM" ny_main:func:auto 2>&1 | grep -E "(✓|Input|Output|Auto-resolved|Found)" || true
rm "$TEMP_WASM"

if [[ ! -f "$OUTPUT_WASM" ]]; then
    echo -e "${RED}✗ Export section addition failed${NC}" >&2
    exit 1
fi

echo -e "${GREEN}✓ Export section added${NC}"

# Verify WASM binary
echo ""
echo "3. Verifying WASM binary..."
WASM_SIZE=$(stat -c%s "$OUTPUT_WASM" 2>/dev/null || stat -f%z "$OUTPUT_WASM" 2>/dev/null)
echo "   Size: $WASM_SIZE bytes"

# Check magic number
MAGIC=$(xxd -p -l 4 "$OUTPUT_WASM" 2>/dev/null || echo "")
if [[ "$MAGIC" == "0061736d" ]]; then
    echo -e "${GREEN}✓ Valid WASM binary (magic: 0x$MAGIC)${NC}"
else
    echo -e "${YELLOW}⚠ Warning: Unexpected magic number: 0x$MAGIC${NC}"
fi

# Optional: Use wabt tools if available
if command -v wasm-objdump &>/dev/null; then
    echo ""
    echo "4. WASM module info (wabt):"
    wasm-objdump -h "$OUTPUT_WASM" 2>/dev/null || true
fi

echo ""
echo "========================================"
echo -e "${GREEN}✅ Build Complete${NC}"
echo "========================================"
echo ""
echo "Run with Node.js:"
echo "  node $PROJECT_ROOT/tools/wasm_runner.js $OUTPUT_WASM"
echo ""
echo "⚠️  Known Limitation (Phase 15.8 Week 1):"
echo "   Functions may not be exported correctly with llvmlite alone."
echo "   Full WASM support requires LLVM toolchain (llc, wasm-ld)."
echo "   See: docs/development/roadmap/phases/phase-15.8/README.md"
echo ""
