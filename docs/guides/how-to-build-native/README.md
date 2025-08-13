# Nyash Native Build Guide

## Quick Start

### 🚀 Build CLI Only (Default)
```bash
# Minimal CLI build - fastest and cleanest
cargo build --bin nyash

# Run a simple program
cargo run --bin nyash -- local_tests/simple_hello.nyash
```

### 🎨 Build with GUI Features (Optional)
```bash
# Build with GUI support
cargo build --features gui

# Build GUI examples
cargo build --features gui-examples --example gui_simple_notepad
```

## Features Overview

Nyash uses Cargo features to separate functionality:

- **Default**: `cli` - Core CLI interpreter only
- **gui**: Adds EguiBox for desktop GUI applications
- **gui-examples**: Includes GUI example applications

### Available Build Targets

#### Core Binary
- `cargo build --bin nyash` - Main CLI interpreter

#### GUI Examples (with `--features gui-examples`)
- `gui_simple_notepad` - Text editor example
- `gui_nyash_explorer` - File manager example  
- `gui_debug_notepad` - Debug-enabled text editor

## Platform Support

### Linux/WSL
```bash
# Standard build
cargo build --release --bin nyash

# Output: target/release/nyash
```

### Windows Cross-Compilation
```bash
# Install cross-compilation tools
cargo install cargo-xwin

# Build Windows executable
cargo xwin build --target x86_64-pc-windows-msvc --release --bin nyash

# Output: target/x86_64-pc-windows-msvc/release/nyash.exe (~916KB)
```

## CLI Options

- `--dump-mir` - Output MIR (Middle Intermediate Representation)
- `--verify` - Verify program structure
- `--debug-fuel N` - Limit parser iterations for debugging

Example:
```bash
cargo run --bin nyash -- --debug-fuel 1000 program.nyash
```

## Testing

### Local Test Files
Simple working examples are provided in `local_tests/`:

- `simple_hello.nyash` - Basic "Hello World" with variables
- `basic_math.nyash` - Arithmetic and boolean operations
- `static_main.nyash` - Static box Main pattern

```bash
# Run all test files
for file in local_tests/*.nyash; do
    echo "Testing $file"
    cargo run --bin nyash -- "$file"
done
```

### Build Verification
```bash
# Verify CLI builds cleanly
cargo build --bin nyash

# Verify GUI features work when enabled
cargo build --features gui

# Verify examples compile
cargo build --features gui-examples --examples
```

## Architecture Benefits

The feature-based architecture provides:

1. **Fast Development**: Default CLI build is minimal and fast
2. **Optional GUI**: Heavy GUI dependencies only when needed
3. **Clean Separation**: Core language separate from UI examples
4. **Deployment Flexibility**: Choose minimal or full-featured builds

## Troubleshooting

### Build Issues
- **GUI dependency errors**: Make sure to use `--features gui` when building GUI components
- **Parser errors**: Use `--debug-fuel` to limit parser iterations and diagnose infinite loops
- **Missing dependencies**: Run `cargo update` to refresh dependencies

### Common Commands
```bash
# Clean build
cargo clean && cargo build --bin nyash

# Check for warnings
cargo check --bin nyash

# Run with verbose output
RUST_LOG=debug cargo run --bin nyash -- program.nyash
```

This guide ensures you can quickly build and run Nyash for development and testing of the upcoming MIR/VM/JIT features.