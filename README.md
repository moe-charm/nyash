# 🐱 Nyash Programming Language
**A Seriously-Crafted Hobby Language**  
**From Zero to Native Binary in 20 Days - The AI-Powered Language Revolution**

*[🇯🇵 日本語版はこちら / Japanese Version](README.ja.md)*

[![Selfhost Minimal](https://github.com/moe-charm/nyash/actions/workflows/selfhost-minimal.yml/badge.svg?branch=selfhosting-dev)](https://github.com/moe-charm/nyash/actions/workflows/selfhost-minimal.yml)
[![Core Smoke](https://github.com/moe-charm/nyash/actions/workflows/smoke.yml/badge.svg)](https://github.com/moe-charm/nyash/actions/workflows/smoke.yml)
[![Everything is Box](https://img.shields.io/badge/Philosophy-Everything%20is%20Box-blue.svg)](#philosophy)
[![Performance](https://img.shields.io/badge/Performance-13.5x%20Faster-ff6b6b.svg)](#performance)
[![JIT Ready](https://img.shields.io/badge/JIT-Cranelift%20Powered%20(runtime%20disabled)-orange.svg)](#execution-modes)
[![Try in Browser](https://img.shields.io/badge/Try%20Now-Browser%20Playground-ff6b6b.svg)](projects/nyash-wasm/nyash_playground.html)
[![MIT License](https://img.shields.io/badge/License-MIT-green.svg)](#license)

---

Execution Status (Feature Additions Pause)
- Active
  - `--backend llvm` (Python/llvmlite harness; AOT object emit)
  - `--backend vm` (PyVM harness)
- Inactive/Sealed
  - `--backend cranelift`, `--jit-direct` (sealed; use LLVM harness)
  - Rust VM (legacy opt‑in via features)

Quick pointers
- Emit object with harness: set `NYASH_LLVM_USE_HARNESS=1` and `NYASH_LLVM_OBJ_OUT=<path>` (defaults in tools use `tmp/`).
- Run PyVM: `NYASH_VM_USE_PY=1 ./target/release/nyash --backend vm apps/APP/main.nyash`.

Developer quickstart: see `docs/DEV_QUICKSTART.md`. Changelog highlights: `CHANGELOG.md`.
User Macros (Phase 2): `docs/guides/user-macros.md`
Exceptions (postfix catch/cleanup): `docs/guides/exception-handling.md`
ScopeBox & MIR hints: `docs/guides/scopebox.md`
AST JSON v0 (macro/bridge): `docs/reference/ir/ast-json-v0.md`
MIR mode note: default is MIR13 (PHI-off). See `docs/development/mir/MIR13_MODE.md`.
Self‑hosting one‑pager: `docs/how-to/self-hosting.md`.
ExternCall (env.*) and println normalization: `docs/reference/runtime/externcall.md`.

Profiles (quick)
- `--profile dev` → Macros ON (strict), PyVM dev向け設定を適用（必要に応じて環境で上書き可）
- `--profile lite` → Macros OFF の軽量実行
  - 例: `./target/release/nyash --profile dev --backend vm apps/tests/ternary_basic.nyash`

Specs & Constraints
- Invariants (must-hold): `docs/reference/invariants.md`
- Constraints (known/temporary/resolved): `docs/reference/constraints.md`
- PHI & SSA design: `docs/architecture/phi-and-ssa.md`
- Testing matrix (spec → tests): `docs/guides/testing-matrix.md`
- Comparison with other languages: `docs/comparison/nyash-vs-others.md`

## Table of Contents
- [Self‑Hosting (Dev Focus)](#self-hosting)
- [Try in Browser](#-try-nyash-in-your-browser-right-now)
- [🌟 Property System Revolution](#-property-system-revolution-september-18-2025)
- [Language Features](#-language-features)
- [Plugin System](#-revolutionary-plugin-system-typebox-architecture)

<a id="self-hosting"></a>
## 🧪 Self‑Hosting (Dev Focus)
- Guide: `docs/how-to/self-hosting.md`
- Minimal E2E: `NYASH_DISABLE_PLUGINS=1 ./target/release/nyash --backend vm apps/selfhost-minimal/main.nyash`
- Smokes: `bash tools/jit_smoke.sh` / `bash tools/selfhost_vm_smoke.sh`
- Makefile: `make run-minimal`, `make smoke-selfhost`

MIR note: Core‑13 minimal kernel is enforced by default (NYASH_MIR_CORE13=1). Legacy ops are normalized (Array/Ref→BoxCall; TypeCheck/Cast/Barrier/WeakRef unified).

Pure mode: set `NYASH_MIR_CORE13_PURE=1` to enable strict Core‑13. The optimizer rewrites a few ops (Load/Store/NewBox/Unary) to Core‑13 forms, and the compiler rejects any remaining non‑Core‑13 ops. This may break execution temporarily by design to surface MIR violations early.

Note: JIT runtime execution is currently disabled to reduce debugging overhead. Use Interpreter/VM for running and AOT (Cranelift/LLVM) for distribution.

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

Important: JIT runtime execution is sealed for now. Use PyVM/VM for running, and Cranelift AOT/LLVM AOT for native executables.

Phase‑15 (Self‑Hosting): Legacy VM/Interpreter are feature‑gated
- Default build runs PyVM for `--backend vm` (python3 + `tools/pyvm_runner.py` required)
- To enable legacy Rust VM/Interpreter, build with:
  ```bash
  cargo build --release --features vm-legacy,interpreter-legacy
  ```
  Then `--backend vm`/`--backend interpreter` use the legacy paths.
 - Note: `--benchmark` requires the legacy VM. Build with `--features vm-legacy` before running benchmarks.

### 1. **Interpreter Mode** (Development)
```bash
./target/release/nyash program.nyash
```
- Instant execution
- Full debug information
- Perfect for development

### 2. **VM Mode (PyVM default / Legacy optional)**
```bash
# Default: PyVM harness (requires python3)
./target/release/nyash --backend vm program.nyash

# Enable legacy Rust VM if needed
cargo build --release --features vm-legacy
./target/release/nyash --backend vm program.nyash
```
- Default (vm-legacy OFF): PyVM executes MIR(JSON) via `tools/pyvm_runner.py`
- Legacy VM: 13.5x over interpreter (historical); kept for comparison and plugin tests

### 3. **Native Binary (Cranelift AOT)** (Distribution)
```bash
# Build once (Cranelift)
cargo build --release --features cranelift-jit

./tools/build_aot.sh program.nyash -o myapp
./myapp  # Standalone executable!
```
- Zero dependencies
- Maximum performance
- Easy distribution

### 4. **Native Binary (LLVM AOT)**
```bash
LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) \
  cargo build --release --features llvm
NYASH_LLVM_OBJ_OUT=$PWD/nyash_llvm_temp.o \
  ./target/release/nyash --backend llvm program.nyash
# Link and run
cc nyash_llvm_temp.o -L crates/nyrt/target/release -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive -lpthread -ldl -lm -o myapp
./myapp
```

Quick smoke test (VM vs EXE):
```bash
tools/smoke_aot_vs_vm.sh examples/aot_min_string_len.nyash
```

### LLVM Backend Notes
- `NYASH_LLVM_OBJ_OUT`: Path to emit `.o` when running `--backend llvm`.
  - Example: `NYASH_LLVM_OBJ_OUT=$PWD/nyash_llvm_temp.o ./target/release/nyash --backend llvm apps/ny-llvm-smoke/main.nyash`
- Previously available `NYASH_LLVM_ALLOW_BY_NAME=1`: Removed - all plugin calls now use method_id by default.
  - The LLVM backend only supports method_id-based plugin calls for better performance and type safety.


### 5. **WebAssembly** (Browser)
```bash
cargo build --release --features wasm-backend
./target/release/nyash --compile-wasm program.nyash
```
- Run in browsers
- Cross-platform by default
- Web-first development

---

## 🧰 One‑Command Build (MVP): `nyash --build`

Reads `nyash.toml`, builds plugins → core → emits AOT object → links an executable in one shot.

Basic (Cranelift AOT)
```bash
./target/release/nyash --build nyash.toml \
  --app apps/egui-hello-plugin/main.nyash \
  --out app_egui
```

Key options (minimal)
- `--build <path>`: path to nyash.toml
- `--app <file>`: entry `.nyash`
- `--out <name>`: output executable (default: `app`/`app.exe`)
- `--build-aot cranelift|llvm` (default: cranelift)
- `--profile release|debug` (default: release)
- `--target <triple>` (only when needed)

Notes
- LLVM AOT requires LLVM 18 (`LLVM_SYS_180_PREFIX`).
- Apps that open a GUI may show a window during AOT emission; close it to continue.
- On WSL if the window doesn’t show, see `docs/guides/cranelift_aot_egui_hello.md` (Wayland→X11).


## 📊 **Performance Benchmarks**

Real-world benchmark results (ny_bench.nyash):

```
Mode           | Time      | Relative Speed
---------------|-----------|---------------
Interpreter    | 110.10ms  | 1.0x (baseline)
VM             | 8.14ms    | 13.5x faster
Cranelift AOT  | ~4–6ms    | ~20–27x faster  
Native (LLVM)  | ~4ms      | ~27x faster
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

## 🌟 **Property System Revolution (September 18, 2025)**

### The 4-Category Property Breakthrough
**Just completed: Revolutionary unification of all property types into one elegant syntax!**

```nyash
box RevolutionaryBox {
    // 🔵 stored: Traditional field storage
    name: StringBox
    
    // 🟢 computed: Calculated every access  
    size: IntegerBox { me.items.count() }
    
    // 🟡 once: Lazy evaluation with caching
    once cache: CacheBox { buildExpensiveCache() }
    
    // 🔴 birth_once: Eager evaluation at object creation
    birth_once config: ConfigBox { loadConfiguration() }
    
    birth() {
        me.name = "Example"
        // birth_once properties already initialized!
    }
}
```

### Python Integration Breakthrough  
**The Property System enables revolutionary Python → Nyash transpilation:**

```python
# Python side
class DataProcessor:
    @property
    def computed_result(self):
        return self.value * 2
    
    @functools.cached_property
    def expensive_data(self):
        return heavy_computation()
```

```nyash
// Auto-generated Nyash (1:1 mapping!)
box DataProcessor {
    computed_result: IntegerBox { me.value * 2 }      // computed
    once expensive_data: ResultBox { heavy_computation() }  // once
}
```

**Result**: Python code runs 10-50x faster as native Nyash binaries!

### Documentation
- **[Property System Specification](docs/proposals/unified-members.md)** - Complete syntax reference
- **[Python Integration Guide](docs/development/roadmap/phases/phase-10.7/)** - Python → Nyash transpilation
- **[Implementation Strategy](docs/private/papers/paper-m-method-postfix-catch/implementation-strategy.md)** - Technical details

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
- **September 4**: 🪟 Windows GUI displayed via JIT/native EXE (OS-native window)

*24 days from zero to self-hosting capability - a new record in language development!*

---

**🚀 Nyash - Where Everything is a Box, and Boxes Compile to Native Code!**

*Built with ❤️, 🤖 AI collaboration, and the belief that programming languages can be created at the speed of thought*
