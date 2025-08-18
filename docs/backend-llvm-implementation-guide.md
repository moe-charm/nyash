# LLVM Backend Implementation Guide

## Overview

The LLVM backend provides native code compilation for Nyash programs through the inkwell LLVM wrapper. This document covers the implementation completed in Phase 9.78 Week 1.

## Current Implementation Status

### ✅ Completed (Phase 9.78 Week 1)

- **Infrastructure Setup**: Complete LLVM backend directory structure
- **Mock Implementation**: Fully functional mock that demonstrates all integration points
- **CLI Integration**: `--backend llvm` option support
- **MIR Integration**: Connection between MIR compiler and LLVM backend
- **Error Handling**: Proper error messages and user feedback

### 🔄 In Progress/Planned

- **Real LLVM Integration**: Requires LLVM development libraries
- **Advanced Code Generation**: Full MIR instruction set support
- **Optimization Passes**: LLVM optimization pipeline integration

## Architecture

### Directory Structure

```
src/backend/llvm/
├── mod.rs          # Main module interface and exports
├── context.rs      # LLVM context, module, and target management  
└── compiler.rs     # MIR to LLVM IR compilation logic
```

### Key Components

1. **LLVMCompiler**: Main compilation orchestrator
2. **CodegenContext**: LLVM context and target machine management
3. **Integration Layer**: Connects with existing MIR → Backend pipeline

## Usage

### Mock Implementation (Current)

```bash
# Run with mock LLVM backend
cargo run -- --backend llvm test_program.nyash

# This will:
# 1. Parse Nyash source to AST
# 2. Compile AST to MIR
# 3. Analyze MIR structure (mock)
# 4. Display mock compilation results
# 5. Return appropriate exit code
```

### Real Implementation (Future)

```bash
# Install LLVM development libraries first
sudo apt install llvm-17-dev clang-17

# Enable LLVM feature and build
cargo build --features llvm --release

# Run with real LLVM backend  
cargo run --features llvm -- --backend llvm test_program.nyash

# This will:
# 1. Parse Nyash source to AST
# 2. Compile AST to MIR
# 3. Generate LLVM IR from MIR
# 4. Compile to object file
# 5. Link with system linker
# 6. Execute native binary
```

## Test Cases

### Basic Return Test

**File**: `local_tests/test_return_42.nyash`
```nyash
static box Main {
    main() {
        return 42
    }
}
```

**Expected Behavior**:
- Mock: Analyzes MIR and returns appropriate exit code
- Real: Compiles to native code that returns exit code 42

### Running Tests

```bash
# Test mock implementation
cargo run -- --backend llvm local_tests/test_return_42.nyash
echo "Exit code: $?"

# Should show mock execution and exit code 0 (42 when real implementation is complete)
```

## Implementation Details

### Mock vs Real Implementation

The current implementation uses conditional compilation to provide both mock and real implementations:

```rust
#[cfg(feature = "llvm")]
// Real inkwell-based implementation
use inkwell::context::Context;

#[cfg(not(feature = "llvm"))]  
// Mock implementation for demonstration
pub struct CodegenContext { /* ... */ }
```

### MIR Integration

The LLVM backend integrates with the existing MIR compilation pipeline:

1. **AST → MIR**: Uses existing `MirCompiler`
2. **MIR → LLVM**: New `LLVMCompiler` handles MIR instruction translation
3. **LLVM → Native**: Uses LLVM's code generation and system linker

### Error Handling

The implementation provides comprehensive error handling:

- **Environment Errors**: Missing LLVM libraries, compilation failures
- **User Feedback**: Clear messages about requirements and next steps
- **Graceful Degradation**: Mock implementation when LLVM unavailable

## Dependencies

### System Requirements

- **LLVM 17+**: Development libraries and headers
- **Clang**: System C compiler for linking
- **Target Support**: Native target architecture support

### Rust Dependencies

```toml
[dependencies]
inkwell = { version = "0.5", features = ["target-x86"], optional = true }

[features]
llvm = ["dep:inkwell"]
```

## Future Enhancements

### Week 2+ Roadmap

1. **Real LLVM Integration**: Replace mock with inkwell implementation
2. **Instruction Support**: Complete MIR instruction set coverage
3. **Type System**: Proper LLVM type mapping for Nyash Box types
4. **Optimization**: LLVM optimization pass integration
5. **Debug Info**: Debug symbol generation for debugging support

### Performance Goals

- **Compilation Speed**: Sub-second compilation for small programs
- **Runtime Performance**: 2x+ improvement over VM backend
- **Binary Size**: Reasonable binary sizes with optional optimization

## Contributing

### Development Setup

1. Install LLVM development libraries
2. Enable LLVM feature flag
3. Run tests with `cargo test --features llvm`
4. Implement missing MIR instruction handlers

### Code Style

- Follow existing Nyash code patterns
- Use conditional compilation for feature gates
- Provide comprehensive error messages
- Include tests for new functionality

## References

- [inkwell Documentation](https://thedan64.github.io/inkwell/)
- [LLVM Language Reference](https://llvm.org/docs/LangRef.html)
- [Nyash MIR Specification](../mir/README.md)
- [Phase 9.78 Implementation Plan](../../予定/native-plan/llvm/issue/001-setup-inkwell-hello-world.md)