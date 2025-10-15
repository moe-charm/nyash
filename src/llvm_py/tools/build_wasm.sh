#!/bin/bash
# Build WASM binary from MIR JSON

set -e

if [ $# -lt 1 ]; then
    echo "Usage: $0 <input.json> [-o output.wasm]"
    exit 1
fi

INPUT="$1"
OUTPUT="${2:-output.wasm}"

if [ "$2" = "-o" ] && [ -n "$3" ]; then
    OUTPUT="$3"
fi

echo "Building WASM from $INPUT..."

# Generate LLVM IR with WASM target
python3 llvm_builder.py "$INPUT" -o /tmp/temp.o --target wasm32

# Check if .wasm file was generated
if [ ! -f /tmp/temp.wasm ]; then
    echo "Error: WASM file not generated"
    exit 1
fi

# Move to output location
mv /tmp/temp.wasm "$OUTPUT"

echo "✅ WASM binary generated: $OUTPUT"
