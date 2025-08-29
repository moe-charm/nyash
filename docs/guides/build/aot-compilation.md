# 🚀 AOT (Ahead-of-Time) Compilation Guide

Transform your Nyash programs into standalone native executables!

## 📌 What is AOT?

AOT compilation converts Nyash programs directly to native machine code, producing standalone executables that:
- Run without the Nyash runtime
- Start instantly (no JIT compilation)
- Achieve maximum performance
- Can be distributed as single files

## 🔧 Prerequisites

- Nyash built with Cranelift support: `--features cranelift-jit`
- C compiler (gcc/clang) for linking
- nyrt runtime library (automatically built)

## 🏗️ Building AOT Support

```bash
# Enable Cranelift JIT (required for AOT)
cargo build --release --features cranelift-jit

# Build the runtime library
cd crates/nyrt
cargo build --release
```

## 📦 Creating Native Executables

### Linux/WSL

Use the provided build script:

```bash
# Basic usage
./tools/build_aot.sh program.nyash -o myapp

# Run the native executable
./myapp
```

**What happens behind the scenes:**
1. Nyash compiles your program to object file (.o)
2. Links with libnyrt.a (runtime support)
3. Produces standalone executable

### Windows

From PowerShell:

```powershell
# Build native executable
powershell -ExecutionPolicy Bypass -File tools\build_aot.ps1 -Input program.nyash -Out myapp.exe

# Run it
.\myapp.exe
```

From WSL (cross-compile):

```bash
# Use the bash script even for Windows targets
NYASH_TARGET=windows ./tools/build_aot.sh program.nyash -o myapp.exe
```

## 🎯 AOT Compilation Pipeline

```
program.nyash
    ↓
[Nyash Parser]
    ↓
   AST
    ↓
[MIR Builder]
    ↓
   MIR
    ↓
[JIT Compiler - Strict Mode]
    ↓
  main.o (Object File)
    ↓
[C Linker + libnyrt.a]
    ↓
Native Executable
```

## ⚡ Supported Features

### Currently Supported (Phase 10.10)
- ✅ Basic arithmetic operations
- ✅ Function calls (limited)
- ✅ Integer operations
- ✅ String operations (read-only)
- ✅ Control flow (if/else, loops)

### Not Yet Supported
- ❌ Full plugin system
- ❌ Dynamic code loading
- ❌ Some built-in Box methods
- ❌ Async/await
- ❌ Exception handling

## 📝 Writing AOT-Compatible Code

### Example: Simple Calculation

```nyash
// aot_example.nyash
static box Main {
    main() {
        local x = 10
        local y = 20
        local result = x + y
        return result  // Exit code: 30
    }
}
```

### Example: String Length (Plugin-Based)

```nyash
// Requires NYASH_USE_PLUGIN_BUILTINS=1
static box Main {
    main() {
        local s = "Hello"
        return s.length()  // Exit code: 5
    }
}
```

## 🔍 Debugging AOT Compilation

### Enable Verbose Output

```bash
# See what's happening
NYASH_CLI_VERBOSE=1 ./tools/build_aot.sh program.nyash
```

### Check JIT Coverage

```bash
# See which operations are supported
NYASH_JIT_DUMP=1 ./target/release/nyash --backend vm program.nyash
```

### Common Issues

#### "Object not generated"
```
error: object not generated: target/aot_objects/main.o
hint: Strict mode forbids fallback. Ensure main() is lowerable under current JIT coverage.
```

**Solution**: Simplify your program or wait for more JIT coverage.

#### "Unsupported lowering ops"
```
[JIT][strict] unsupported lowering ops for main: 3 — failing compile
```

**Solution**: Check which operations are causing issues:
- Complex Box method calls
- Dynamic features
- Plugin-dependent code

## 🏃 Performance Considerations

### Benchmark Results

| Execution Mode | Time (ny_bench.nyash) | Relative |
|----------------|----------------------|----------|
| Interpreter | 110.10ms | 1.0x |
| VM | 8.14ms | 13.5x |
| VM + JIT | 5.8ms | 19.0x |
| **AOT Native** | ~4ms | **~27x** |

### Optimization Flags

```bash
# Maximum performance
RUSTFLAGS="-C target-cpu=native" cargo build --release --features cranelift-jit

# Then compile your program
./tools/build_aot.sh program.nyash -o optimized_app
```

## 🔌 Plugin Support in AOT

### Static Linking (Future)

The goal is to statically link plugins:

```bash
# Future syntax (not yet implemented)
./tools/build_aot.sh program.nyash \
    --static-plugins filebox,stringbox \
    -o standalone_app
```

### Dynamic Plugins (Current)

AOT executables can load plugins if:
- nyash.toml is present
- Plugin .so/.dll files are available
- Paths are correctly configured

## 🛠️ Advanced Usage

### Custom Linker Flags

```bash
# Modify build_aot.sh or use directly:
cc main.o \
   -L crates/nyrt/target/release \
   -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive \
   -lpthread -ldl -lm \
   -O3 -flto \  # Extra optimizations
   -o myapp
```

### Embedding Resources

Future feature to embed files in the executable:

```nyash
// Future syntax
#[embed("data.json")]
static DATA: &str

static box Main {
    main() {
        local config = JSON.parse(DATA)
        // ...
    }
}
```

## 📊 AOT vs Other Modes

| Feature | Interpreter | VM | JIT | AOT |
|---------|------------|----|----|-----|
| Startup Time | Fast | Fast | Medium | **Instant** |
| Peak Performance | Slow | Good | Better | **Best** |
| Memory Usage | Low | Medium | High | **Lowest** |
| Distribution | Needs Nyash | Needs Nyash | Needs Nyash | **Standalone** |
| Plugin Support | Full | Full | Full | Limited |
| Debug Info | Full | Good | Limited | Minimal |

## 🎯 When to Use AOT

**Use AOT when:**
- Distributing standalone applications
- Maximum performance is critical
- Startup time matters
- Deployment simplicity is important

**Avoid AOT when:**
- Using dynamic features heavily
- Requiring full plugin ecosystem
- Rapid development/debugging
- Code uses unsupported operations

## 🔮 Future Roadmap

### Phase 10.11+ Plans
- Full plugin static linking
- Complete Box method coverage
- Cross-compilation support
- Size optimization options
- Debug symbol control

## 📚 See Also

- [Build Overview](README.md)
- [JIT Architecture](../../reference/jit/)
- [Performance Guide](../../reference/performance/)