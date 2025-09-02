# 🐱 Nyash Programming Language
**A Seriously-Crafted Hobby Language**  
**From Zero to Native Binary in 20 Days - The AI-Powered Language Revolution**

*[🇯🇵 日本語版はこちら / Japanese Version](README.ja.md)*

[![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen.svg)](#)
[![Everything is Box](https://img.shields.io/badge/Philosophy-Everything%20is%20Box-blue.svg)](#philosophy)
[![Performance](https://img.shields.io/badge/Performance-13.5x%20Faster-ff6b6b.svg)](#performance)
[![JIT Ready](https://img.shields.io/badge/JIT-Cranelift%20Powered-orange.svg)](#execution-modes)
[![Try in Browser](https://img.shields.io/badge/Try%20Now-Browser%20Playground-ff6b6b.svg)](projects/nyash-wasm/nyash_playground.html)
[![MIT License](https://img.shields.io/badge/License-MIT-green.svg)](#license)

---

## 🎮 **Try Nyash in Your Browser Right Now!**

👉 **[Launch Browser Playground](projects/nyash-wasm/nyash_playground.html)** 👈

No installation needed - experience Nyash instantly in your web browser!

---

## 🚀 **Breaking News: Self-Hosting Revolution!**

**September 2, 2025** - 🔥 **ABI as a Box!** Nyash ABI itself implemented as TypeBox (C language) - path to self-hosting clear!
**September 1, 2025** - Revolutionary TypeBox ABI unification achieved! C ABI + Nyash ABI seamlessly integrated.
**August 29, 2025** - Just 20 days after inception, Nyash can now compile to native executables!

```bash
# From Nyash source to native binary (Cranelift required)
cargo build --release --features cranelift-jit
./tools/build_aot.sh program.nyash -o app         # Native EXE
./app                                             # Standalone execution!
```

**What we achieved in 24 days:**
- ✅ Full programming language with interpreter
- ✅ VM with 13.5x performance boost  
- ✅ JIT compiler (Cranelift integration)
- ✅ WebAssembly support
- ✅ Plugin system (C ABI + Nyash ABI unified)
- ✅ Native binary generation
- ✅ Python integration via plugins
- ✅ TypeBox ABI bridge (revolutionary plugin unification)
- ✅ **Self-hosting path clear** (Nyash ABI in C, no Rust dependency!)

---

## ✨ **Why Nyash?**

### 🎯 **Everything is Box Philosophy**
```nyash
// Traditional languages have complex type systems
// Nyash: One concept rules them all - Box

static box Main {
    main() {
        // Every value is a Box - unified, safe, simple
        local name = new StringBox("Nyash")
        local count = new IntegerBox(42)
        local data = new MapBox()
        
        // Even Python objects are Boxes!
        local py = new PyRuntimeBox()
        local math = py.import("math")
        print("sqrt(9) = " + math.getattr("sqrt").call(9).str())
        
        return 0
    }
}
```

### ⚡ **Unprecedented Development Speed**
- **Day 1**: Basic interpreter working
- **Day 4**: Already planning JIT
- **Day 13**: VM achieving 13.5x speedup
- **Day 20**: Native executable generation!

### 🔌 **Plugin-First Architecture**
```nyash
// Any functionality can be a plugin Box
local file = new FileBox()          // File I/O plugin
local http = new HttpClientBox()    // Network plugin  
local py = new PyRuntimeBox()       // Python plugin

// Plugins compile to native code too!
```

---

## 🏗️ **Multiple Execution Modes**

### 1. **Interpreter Mode** (Development)
```bash
./target/release/nyash program.nyash
```
- Instant execution
- Full debug information
- Perfect for development

### 2. **VM Mode** (Production) 
```bash
./target/release/nyash --backend vm program.nyash
```
- 13.5x faster than interpreter
- Optimized bytecode execution
- Production-ready performance

### 3. **JIT Mode** (High Performance)
```bash
NYASH_JIT_EXEC=1 ./target/release/nyash --backend vm program.nyash
```
- Cranelift-powered JIT compilation
- Near-native performance
- Hot function optimization

### 4. **Native Binary** (Distribution)
```bash
# Build once (Cranelift)
cargo build --release --features cranelift-jit

./tools/build_aot.sh program.nyash -o myapp
./myapp  # Standalone executable!
```
- Zero dependencies
- Maximum performance
- Easy distribution

Quick smoke test (VM vs EXE):
```bash
tools/smoke_aot_vs_vm.sh examples/aot_min_string_len.nyash
```

### LLVM Backend Notes
- `NYASH_LLVM_OBJ_OUT`: Path to emit `.o` when running `--backend llvm`.
  - Example: `NYASH_LLVM_OBJ_OUT=$PWD/nyash_llvm_temp.o ./target/release/nyash --backend llvm apps/ny-llvm-smoke/main.nyash`
- `NYASH_LLVM_ALLOW_BY_NAME=1`: Debug-only fallback for plugin calls by name when by-id isn’t available.
  - Emits calls to `nyash.plugin.invoke_by_name_i64` for development.
  - Do not enable in production.


### 5. **WebAssembly** (Browser)
```bash
cargo build --release --features wasm-backend
./target/release/nyash --compile-wasm program.nyash
```
- Run in browsers
- Cross-platform by default
- Web-first development

---

## 📊 **Performance Benchmarks**

Real-world benchmark results (ny_bench.nyash):

```
Mode           | Time      | Relative Speed
---------------|-----------|---------------
Interpreter    | 110.10ms  | 1.0x (baseline)
VM             | 8.14ms    | 13.5x faster
VM + JIT       | 5.8ms     | 19.0x faster  
Native Binary  | ~4ms      | ~27x faster
```

---

## 🎮 **Language Features**

### Clean Syntax
```nyash
box GameCharacter {
    private { name, health, skills }
    
    // Birth constructor - giving life to Boxes!
    birth(characterName) {
        me.name = characterName
        me.health = 100
        me.skills = new ArrayBox()
        print("🌟 " + characterName + " has been born!")
    }
    
    learnSkill(skill) {
        me.skills.push(skill)
        return me  // Method chaining
    }
}

// Usage
local hero = new GameCharacter("Neko")
hero.learnSkill("Fire Magic").learnSkill("Healing")
```

### Modern Async/Await
```nyash
// Concurrent operations made simple
nowait task1 = fetchDataFromAPI()
nowait task2 = processLocalFiles()

// Do other work while waiting
updateUI()

// Collect results
local apiData = await task1
local files = await task2
```

### Delegation Pattern
```nyash
// Composition over inheritance
box EnhancedArray from ArrayBox {
    private { logger }
    
    override push(item) {
        me.logger.log("Adding: " + item)
        from ArrayBox.push(item)  // Delegate to parent
    }
}
```

---

## 🔌 **Revolutionary Plugin System (TypeBox Architecture)**

### TypeBox: The Universal Plugin Bridge (September 2025)
**"Everything is Box" Philosophy - Even ABI is a Box!**

```c
// TypeBox - Type information as a Box (enables cross-plugin creation)
typedef struct {
    uint32_t abi_tag;           // 'TYBX'
    const char* name;           // "ArrayBox"
    void* (*create)(void);      // Box creation function
} NyrtTypeBox;

// NEW: Nyash ABI itself as a TypeBox! (C implementation, no Rust)
typedef struct {
    uint32_t abi_tag;           // 'NABI'
    const char* name;           // "NyashABIProvider"
    void* (*create)(void);      // ABI provider creation
    // ... Nyash operations (call, retain, release)
} NyashABITypeBox;
```

**Revolutionary Achievement**: ABI implementation in pure C enables self-hosting!

### Plugin Configuration
```toml
# nyash.toml v3.0 - Unified plugin support
[plugins.map]
path = "plugins/map.so"
abi = "c"              # Traditional C ABI

[plugins.advanced_map]
path = "plugins/adv_map.so"
abi = "nyash"          # Type-safe Nyash ABI

[plugins.hybrid]
path = "plugins/hybrid.so"
abi = "unified"        # Both ABIs supported!
```

**Key Innovation**: TypeBox enables cross-plugin Box creation without circular dependencies. MapBox can now return ArrayBox seamlessly!

📚 **[Full TypeBox Documentation](docs/development/roadmap/phases/phase-12/)**

---

## 🛠️ **Getting Started**

### Quick Install (Linux/Mac/WSL)
```bash
# Clone and build
git clone https://github.com/moe-charm/nyash.git
cd nyash
cargo build --release --features cranelift-jit

# Run your first program
echo 'print("Hello Nyash!")' > hello.nyash
./target/release/nyash hello.nyash
```

### Windows
```bash
# Cross-compile for Windows
cargo install cargo-xwin
cargo xwin build --target x86_64-pc-windows-msvc --release
# Use target/x86_64-pc-windows-msvc/release/nyash.exe

# Native EXE (AOT) on Windows (requires Cranelift and MSYS2/WSL toolchain for linking)
cargo build --release --features cranelift-jit
powershell -ExecutionPolicy Bypass -File tools\build_aot.ps1 -Input examples\aot_min_string_len.nyash -Out app.exe
./app.exe
```

---

## 🌟 **Unique Innovations**

### 1. **AI-Driven Development**
- Developed with Claude, ChatGPT, and Codex collaboration
- 20-day journey from concept to native compilation
- Proves AI can accelerate language development by 30x

### 2. **Box-First Architecture**
- Every optimization preserves the Box abstraction
- Plugins are Boxes, JIT preserves Boxes, even native code respects Boxes
- TypeBox: Even type information is a Box!
- Unprecedented consistency across all execution modes

### 3. **Observable by Design**
- Built-in debugging and profiling
- JSON event streams for JIT compilation
- DOT graph visualization of optimizations

---

## 📚 **Examples**

### Python Integration
```nyash
// Use Python libraries from Nyash!
local py = new PyRuntimeBox()
local np = py.import("numpy")
local array = np.getattr("array").call([1, 2, 3])
print("NumPy array: " + array.str())
```

### Web Server
```nyash
local server = new HttpServerBox()
server.start(8080)

loop(true) {
    local request = server.accept()
    local response = new HttpResponseBox()
    response.setStatus(200)
    response.write("Hello from Nyash!")
    request.respond(response)
}
```

### Game Development
```nyash
box GameObject {
    public { x, y, sprite }
    
    update(deltaTime) {
        // Physics simulation
        me.y = me.y + gravity * deltaTime
    }
    
    render(canvas) {
        canvas.drawImage(me.sprite, me.x, me.y)
    }
}
```

---

## 🤝 **Contributing**

Join the revolution! We welcome:
- 🐛 Bug reports and fixes
- ✨ New Box types via plugins
- 📚 Documentation improvements
- 🎮 Cool example programs

See also: Contributor guide in `AGENTS.md` (Repository Guidelines) for project layout, build/test commands, and PR expectations.

## 📄 **License**

MIT License - Use freely in your projects!

---

## 👨‍💻 **Creator**

**charmpic** - Hobby Language Developer
- 🐱 GitHub: [@moe-charm](https://github.com/moe-charm)
- 🌟 Created with: Claude, ChatGPT, Codex collaboration

---

## 🎉 **Historical Timeline**

- **August 9, 2025**: First commit - "Hello Nyash!"
- **August 13**: JIT planning begins (day 4!)
- **August 20**: VM achieves 13.5x performance
- **August 29**: Native EXE compilation achieved!
- **September 1**: TypeBox ABI unification - C ABI + Nyash ABI seamless integration
- **September 2**: 🔥 Self-hosting path clear - Nyash ABI in C (no Rust dependency!)

*24 days from zero to self-hosting capability - a new record in language development!*

---

**🚀 Nyash - Where Everything is a Box, and Boxes Compile to Native Code!**

*Built with ❤️, 🤖 AI collaboration, and the belief that programming languages can be created at the speed of thought*
