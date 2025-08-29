# 🏗️ Nyash Build Guide - Complete Build Patterns

Everything you need to know about building Nyash for different platforms and configurations.

## 📚 Table of Contents

1. [Quick Start](#quick-start)
2. [Build Patterns Overview](#build-patterns-overview)
3. [Platform-Specific Builds](#platform-specific-builds)
4. [Plugin Development](#plugin-development)
5. [Distribution Packages](#distribution-packages)
6. [Troubleshooting](#troubleshooting)

## 🚀 Quick Start

### Basic Build (Linux/WSL)
```bash
# Standard build
cargo build --release -j32

# Run
./target/release/nyash program.nyash
```

### Windows Cross-Compile from WSL
```bash
# Install cargo-xwin
cargo install cargo-xwin

# Build for Windows
cargo xwin build --target x86_64-pc-windows-msvc --release
```

## 📊 Build Patterns Overview

| Pattern | Command | Output | Use Case |
|---------|---------|--------|----------|
| **Standard** | `cargo build --release` | Linux/macOS binary | Local development |
| **Windows Cross** | `cargo xwin build --target x86_64-pc-windows-msvc` | Windows .exe | Windows distribution |
| **WebAssembly** | `wasm-pack build --target web` | .wasm + JS | Browser deployment |
| **AOT Native** | `./tools/build_aot.sh program.nyash` | Standalone executable | High-performance deployment |
| **Plugins** | `cargo build --release` (in plugin dir) | .so/.dll/.dylib | Extending Nyash |

## 🖥️ Platform-Specific Builds

### 🐧 Linux/WSL Build

Standard Rust build process:

```bash
# Debug build
cargo build

# Release build (recommended)
cargo build --release -j32

# With specific features
cargo build --release --features cranelift-jit
```

### 🪟 Windows Build

#### Prerequisites (Native Windows)
- Rust MSVC toolchain (host = x86_64-pc-windows-msvc)
- Visual Studio Build Tools (C++ デスクトップ開発)
- LLVM/clang (AOTリンクに推奨)
- PowerShell 実行許可（`-ExecutionPolicy Bypass`で一時回避可）

#### Option 1: Cross-compile from Linux/WSL (Recommended)

```bash
# One-time setup
cargo install cargo-xwin

# Build Windows binary
cargo xwin build --target x86_64-pc-windows-msvc --release

# Output: target/x86_64-pc-windows-msvc/release/nyash.exe
```

#### Option 2: Native Windows Build

Requirements:
- Rust with MSVC toolchain
- Visual Studio Build Tools (C++ Desktop Development)
- LLVM/clang (optional, for AOT)

```bash
# On Windows
cargo build --release
```

#### AOT on Windows (Native EXE)
```powershell
# Requires Cranelift + (clang または MSYS2/WSL の bash+cc)
cargo build --release --features cranelift-jit
powershell -ExecutionPolicy Bypass -File tools\build_aot.ps1 -Input examples\aot_min_string_len.nyash -Out app.exe
./app.exe
```

Notes:
- EXEはまず実行ファイルと同じフォルダの `nyash.toml` を探します。なければカレントディレクトリを参照します。
- 追加ログは `-v` もしくは `set NYASH_CLI_VERBOSE=1` で表示。
- プラグインの依存DLLがある場合は、各プラグインDLLと同じフォルダに配置するか、`PATH` に追加してください。

### 🌐 WebAssembly Build

Two types of WASM builds:

#### 1. Nyash Interpreter in Browser
```bash
# Build Nyash itself as WASM
wasm-pack build --target web

# Files generated in pkg/
# - nyash_rust_bg.wasm
# - nyash_rust.js
# - nyash_rust.d.ts
```

#### 2. Compile Nyash Code to WASM
```bash
# Compile .nyash to .wat (WebAssembly Text)
./target/release/nyash --compile-wasm program.nyash -o output.wat

# Convert to binary WASM (requires wabt)
wat2wasm output.wat -o output.wasm
```

### 🚀 AOT (Ahead-of-Time) Native Compilation

Compile Nyash programs to standalone native executables:

#### Linux/WSL
```bash
# Build with Cranelift support
cargo build --release --features cranelift-jit

# Compile to native
./tools/build_aot.sh program.nyash -o app
./app  # Standalone executable!
```

#### Windows
```powershell
# From PowerShell
cargo build --release --features cranelift-jit
powershell -ExecutionPolicy Bypass -File tools\build_aot.ps1 -Input program.nyash -Out app.exe
.\app.exe
```

## 🔌 Plugin Development

### Building Plugins

Plugins must be built for each target platform:

#### Linux Plugin (.so)
```bash
cd plugins/nyash-example-plugin
cargo build --release
# Output: target/release/libnyash_example_plugin.so
```

#### Windows Plugin (.dll)
```bash
# From WSL
cd plugins/nyash-example-plugin
cargo xwin build --target x86_64-pc-windows-msvc --release
# Output: target/x86_64-pc-windows-msvc/release/nyash_example_plugin.dll
```

#### macOS Plugin (.dylib)
```bash
cd plugins/nyash-example-plugin
cargo build --release
# Output: target/release/libnyash_example_plugin.dylib
```

### Plugin Naming Convention

**Important**: Windows removes the `lib` prefix automatically:
- Linux/macOS: `libnyash_example_plugin.so/dylib`
- Windows: `nyash_example_plugin.dll`

The plugin loader handles this automatically with platform-agnostic configuration.

## 📦 Distribution Packages

### Creating a Windows Distribution

Perfect for sharing Nyash applications:

```bash
# 1. Build main executable
cargo xwin build --target x86_64-pc-windows-msvc --release

# 2. Build plugins
for plugin in filebox array map string integer; do
  (cd plugins/nyash-$plugin-plugin && \
   cargo xwin build --target x86_64-pc-windows-msvc --release)
done

# 3. Create distribution structure
mkdir -p dist/plugins
cp target/x86_64-pc-windows-msvc/release/nyash.exe dist/
cp nyash.toml dist/

# 4. Copy plugin DLLs
cp plugins/*/target/x86_64-pc-windows-msvc/release/*.dll dist/plugins/

# 5. Add your .nyash files
cp your_app.nyash dist/
```

**Distribution structure:**
```
dist/
├── nyash.exe
├── nyash.toml
├── your_app.nyash
└── plugins/
    ├── nyash_filebox_plugin.dll
    ├── nyash_array_plugin.dll
    └── ...
```

### Platform-Agnostic Configuration

Use `.so` extensions in `nyash.toml` - they work on all platforms:

```toml
[plugin_paths]
search_paths = ["./plugins"]

[libraries]
[libraries."libnyash_filebox_plugin.so"]
boxes = ["FileBox"]
path = "libnyash_filebox_plugin.so"  # Works on Windows/Linux/macOS!
```

The plugin loader automatically:
- Converts `.so` → `.dll` (Windows) or `.dylib` (macOS)
- Tries with/without `lib` prefix on Windows
- Searches in configured paths

## 🔧 Build Optimization

### Performance Builds
```bash
# Maximum optimization
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Link-time optimization
CARGO_PROFILE_RELEASE_LTO=true cargo build --release
```

### Size Optimization
```bash
# Smaller binary
cargo build --release --profile=min-size

# Strip symbols (Linux/macOS)
strip target/release/nyash
```

## ❓ Troubleshooting

### Common Issues

#### "LoadLibraryExW failed" on Windows
- Ensure plugins are built for Windows (`cargo xwin build`)
- Check that DLLs are in the `plugins/` directory
- Verify `nyash.toml` has correct `plugin_paths`

#### Cross-compilation fails
- Install `cargo-xwin`: `cargo install cargo-xwin`
- For mingw target: `sudo apt install mingw-w64`

#### WASM build errors
- Install wasm-pack: `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`
- Check Rust target: `rustup target add wasm32-unknown-unknown`

## 📚 Related Documentation

- [Cross-Platform Development](cross-platform.md)
- [Plugin Development Guide](../../reference/plugin-system/)
- [AOT Compilation Details](aot-compilation.md)
- [Performance Tuning](../../reference/performance/)
