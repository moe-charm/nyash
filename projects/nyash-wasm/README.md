# 🌐 Nyash WebAssembly Project (Archived / Unmaintained)

Status: This directory contains an older WASM/browser prototype. It is not part of CI and may not build with current Nyash. Instructions below are historical and provided as-is.

## 🚀 Quick Start (experimental)

```bash
# Install wasm-pack (if not already installed)
cargo install wasm-pack

# Build WASM module
cd /mnt/c/git/nyash
wasm-pack build --target web --out-dir projects/nyash-wasm/pkg

# Start local server (example)
cd projects/nyash-wasm
python3 -m http.server 8000

# Open browser
# Navigate to: http://localhost:8000/nyash_playground.html
```

## 🎯 Features (historical)

- **🐱 Full Nyash Language** - Complete interpreter running in browser
- **📦 ConsoleBox** - Browser console integration  
- **🔍 DebugBox** - Real-time debugging in browser
- **⚡ All Operators** - NOT/AND/OR/Division fully supported
- **🎮 Interactive Playground** - Code editor with examples

## 📁 File Structure

```
projects/nyash-wasm/
├── README.md                 # This file
├── nyash_playground.html     # Interactive playground
├── build.sh                  # Build script
└── pkg/                      # Generated WASM files (after build)
    ├── nyash_rust.js
    ├── nyash_rust_bg.wasm
    └── ...
```

## 🎨 Example Code (historical)

```nyash
// Browser console output
console = new ConsoleBox()
console.log("Hello from Nyash in Browser!")

// Math with new operators
x = 10
y = 3
console.log("Division: " + (x / y))          // 3.333...
console.log("Logic: " + (x > 5 and y < 5))  // true

// Debugging
debug = new DebugBox()
debug.startTracking()
debug.trackBox(x, "my_number")
console.log(debug.memoryReport())
```

## 🔧 Development

### Build Process
1. Rust code compiled to WebAssembly using wasm-bindgen
2. NyashWasm struct exported with eval() method  
3. ConsoleBox uses web-sys for browser console access
4. HTML playground provides interactive interface

### Architecture
```
Browser JavaScript
    ↓
NyashWasm.eval(code)
    ↓ 
NyashInterpreter (Rust)
    ↓
ConsoleBox → web_sys::console
```

## 🎉 Coming Soon

- **DOMBox** - DOM manipulation from Nyash
- **CanvasBox** - Graphics and games
- **EventBox** - Mouse/keyboard event handling
- **HTTPBox** - Network requests
- **Sample Apps** - Snake game, Calculator, etc.

---

**Everything is Box, even in the browser! 🐱**
