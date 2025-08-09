#!/bin/bash
# 🚀 Nyash WASM Build Script

set -e  # Exit on error

echo "🐱 Building Nyash WebAssembly..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found! Installing..."
    cargo install wasm-pack
fi

# Go to project root
cd "$(dirname "$0")/../.."

# Build WASM package
echo "🔨 Building WASM package..."
wasm-pack build --target web --out-dir projects/nyash-wasm/pkg

# Return to wasm project directory
cd projects/nyash-wasm

echo "✅ Build complete!"
echo ""
echo "🌐 To test in browser:"
echo "1. python3 -m http.server 8000"
echo "2. Open: http://localhost:8000/nyash_playground.html"
echo ""
echo "📁 Generated files in pkg/:"
ls -la pkg/ 2>/dev/null || echo "   (run build first)"