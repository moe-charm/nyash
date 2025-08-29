# 🌍 Cross-Platform Development Guide

How to build and distribute Nyash applications across Windows, Linux, and macOS.

## 🎯 Philosophy: Write Once, Run Everywhere

Nyash achieves true cross-platform compatibility through:
- **Platform-agnostic configuration** (nyash.toml)
- **Automatic library resolution** (plugin loader)
- **Unified plugin system** (C ABI)

## 🔄 Cross-Platform Plugin Loading

### The Magic Behind It

ChatGPT5's `resolve_library_path` implementation in `src/runtime/plugin_loader_v2.rs`:

```rust
// Automatic extension mapping
let cur_ext: &str = if cfg!(target_os = "windows") { "dll" } 
                   else if cfg!(target_os = "macos") { "dylib" } 
                   else { "so" };

// Windows: try without 'lib' prefix
if cfg!(target_os = "windows") {
    if let Some(stripped) = stem.strip_prefix("lib") {
        // Try nyash_plugin.dll instead of libnyash_plugin.dll
    }
}
```

### Configuration That Works Everywhere

```toml
# nyash.toml - Same file works on all platforms!
[libraries."libnyash_filebox_plugin.so"]
boxes = ["FileBox"]
path = "libnyash_filebox_plugin.so"  # .so works everywhere!
```

**Runtime Resolution:**
- Linux: `libnyash_filebox_plugin.so` ✅
- Windows: `nyash_filebox_plugin.dll` ✅ (lib prefix removed)
- macOS: `libnyash_filebox_plugin.dylib` ✅

## 🏗️ Building for Multiple Platforms

### From Linux/WSL (Recommended Hub)

```bash
# For Linux (native)
cargo build --release

# For Windows
cargo xwin build --target x86_64-pc-windows-msvc --release

# For macOS (requires macOS SDK)
# cargo build --target x86_64-apple-darwin --release
```

### Build Matrix

| Host OS | Target | Tool | Command |
|---------|--------|------|---------|
| Linux/WSL | Linux | cargo | `cargo build --release` |
| Linux/WSL | Windows | cargo-xwin | `cargo xwin build --target x86_64-pc-windows-msvc` |
| Windows | Windows | cargo | `cargo build --release` |
| Windows | Linux | cross | `cross build --target x86_64-unknown-linux-gnu` |
| macOS | macOS | cargo | `cargo build --release` |
| macOS | Linux | cross | `cross build --target x86_64-unknown-linux-gnu` |

## 📦 Creating Universal Distributions

### Directory Structure
```
nyash-universal/
├── bin/
│   ├── linux/
│   │   └── nyash
│   ├── windows/
│   │   └── nyash.exe
│   └── macos/
│       └── nyash
├── plugins/
│   ├── linux/
│   │   ├── libnyash_filebox_plugin.so
│   │   └── ...
│   ├── windows/
│   │   ├── nyash_filebox_plugin.dll
│   │   └── ...
│   └── macos/
│       ├── libnyash_filebox_plugin.dylib
│       └── ...
├── nyash.toml          # Universal config
├── examples/
└── README.md
```

### Universal Launcher Script

Create `run.sh` (Linux/macOS) and `run.bat` (Windows):

**run.sh:**
```bash
#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
OS="$(uname -s)"

case "$OS" in
    Linux*)  BIN="linux/nyash" ;;
    Darwin*) BIN="macos/nyash" ;;
    *)       echo "Unsupported OS: $OS"; exit 1 ;;
esac

export NYASH_PLUGIN_PATH="$SCRIPT_DIR/plugins/$(echo $OS | tr '[:upper:]' '[:lower:]')"
"$SCRIPT_DIR/bin/$BIN" "$@"
```

**run.bat:**
```batch
@echo off
set SCRIPT_DIR=%~dp0
set NYASH_PLUGIN_PATH=%SCRIPT_DIR%plugins\windows
"%SCRIPT_DIR%bin\windows\nyash.exe" %*
```

## 🔌 Plugin Compatibility

### Building Plugins for All Platforms

Automated build script (`build_all_plugins.sh`):

```bash
#!/bin/bash
PLUGINS="filebox array map string integer net"

for plugin in $PLUGINS; do
    echo "Building $plugin plugin..."
    cd "plugins/nyash-$plugin-plugin" || exit 1
    
    # Linux
    cargo build --release
    cp target/release/lib*.so ../../dist/plugins/linux/
    
    # Windows (from WSL)
    cargo xwin build --target x86_64-pc-windows-msvc --release
    cp target/x86_64-pc-windows-msvc/release/*.dll ../../dist/plugins/windows/
    
    cd ../..
done
```

### Plugin ABI Stability

The C ABI ensures plugins work across platforms:

```c
// Standard C ABI functions (same on all platforms)
extern "C" {
    fn nyash_plugin_abi_version() -> u32;
    fn nyash_plugin_init(host_fns: *const HostFunctions) -> i32;
    fn nyash_plugin_invoke(params: *const InvokeParams) -> i32;
    fn nyash_plugin_shutdown();
}
```

## 🎯 Testing Cross-Platform Builds

### Test Matrix Script

```bash
#!/bin/bash
# test_all_platforms.sh

echo "=== Testing Linux Build ==="
./target/release/nyash test/cross_platform.nyash

echo "=== Testing Windows Build (Wine) ==="
wine ./target/x86_64-pc-windows-msvc/release/nyash.exe test/cross_platform.nyash

echo "=== Testing WASM Build ==="
wasmtime ./target/wasm32-wasi/release/nyash.wasm test/cross_platform.nyash
```

### Cross-Platform Test Program

```nyash
// test/cross_platform.nyash
print("Platform Test Starting...")

// Test basic plugins
local arr = new ArrayBox()
arr.push("Cross")
arr.push("Platform")
arr.push("Success!")
print("Array test: " + arr.get(0) + "-" + arr.get(1) + " " + arr.get(2))

// Test file operations (platform-specific paths)
local file = new FileBox()
if file {
    print("FileBox available on this platform")
}

print("✅ All tests passed!")
```

## 🚀 CI/CD for Cross-Platform

### GitHub Actions Example

```yaml
name: Cross-Platform Build

on: [push, pull_request]

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: macos-latest
            target: x86_64-apple-darwin
    
    runs-on: ${{ matrix.os }}
    
    steps:
    - uses: actions/checkout@v2
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: ${{ matrix.target }}
    
    - name: Build
      run: cargo build --release --target ${{ matrix.target }}
    
    - name: Test
      run: cargo test --release --target ${{ matrix.target }}
    
    - name: Upload artifacts
      uses: actions/upload-artifact@v2
      with:
        name: nyash-${{ matrix.target }}
        path: target/${{ matrix.target }}/release/nyash*
```

## 💡 Best Practices

### 1. Always Test on Target Platform
```bash
# Quick test after cross-compile
wine ./nyash.exe --version  # Windows binary on Linux
```

### 2. Use Platform-Agnostic Paths
```nyash
// Bad
local file = new FileBox()
file.open("C:\\data\\file.txt", "r")  // Windows-specific

// Good
local file = new FileBox()
file.open("./data/file.txt", "r")  // Works everywhere
```

### 3. Handle Platform Differences Gracefully
```nyash
box PlatformHelper {
    static getPlatformName() {
        // Future: return actual platform
        return "universal"
    }
    
    static getPathSeparator() {
        // Future: return platform-specific separator
        return "/"
    }
}
```

## 🔧 Troubleshooting

### "Can't find plugin" after cross-compile
- Check file naming (lib prefix on Windows)
- Verify correct architecture (x86_64 vs ARM)
- Use `--verbose` flag for detailed logs

### Different behavior across platforms
- File path separators (`/` vs `\`)
- Case sensitivity (Linux/macOS vs Windows)
- Line endings (LF vs CRLF)

### Performance variations
- Debug vs Release builds
- Native CPU optimizations
- Plugin loading overhead

## 📚 References

- [Plugin System Architecture](../../reference/plugin-system/)
- [Build Configuration](README.md)
- [Platform-Specific Notes](../../reference/platform-notes.md)