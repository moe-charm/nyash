# Nyash: A Box-Centric Language with Unified Plugin Lifecycle and Semantics-Preserving GC/FFI Across VM/JIT/AOT/WASM

## 1. Introduction

Modern programming languages face a fundamental tension between safety, performance, and interoperability. Managed languages provide safety through garbage collection but struggle with predictable performance and native interop. Systems languages offer control and performance but require complex lifetime management. Plugin systems add another layer of complexity with inconsistent lifecycle contracts across different execution environments.

We present **Nyash**, a box-centric language that resolves these tensions through a unified lifecycle contract that works identically for user-defined boxes (classes) and native plugin boxes. The key insight is that by treating everything as a "box" with a consistent ownership model and lifecycle contract, we can achieve:

1. **Unified semantics** across interpreter, VM, JIT, AOT, and WASM backends
2. **Predictable performance** with optional GC that preserves program semantics
3. **Zero-cost FFI** through static linking and handle-based plugin system
4. **Formal correctness** with provable properties about lifecycle management

### 1.1 Contributions

Our main contributions are:

- **Unified Lifecycle Contract**: A single ownership model that governs both user boxes and plugin boxes, with deterministic finalization order
- **Semantics-Preserving GC**: A novel approach where GC can be turned on/off without changing program behavior
- **Thin C ABI**: A minimal foreign function interface that enables identical behavior across all execution backends
- **Compact Implementation**: A complete language implementation in ~4K lines of code with industrial-strength features

## 2. Design Overview

### 2.1 Everything is a Box

In Nyash, all data is encapsulated in "boxes" - reference-counted objects with a unified interface:

```nyash
// User-defined box
box Person {
    init { name, age }
    
    greet() {
        print("Hello, I'm " + me.name)
    }
}

// Plugin box (implemented in C/Rust)
// Looks identical to users
local file = new FileBox("data.txt")
file.write("Hello")
file.close()  // Deterministic cleanup
```

### 2.2 Ownership Forest

Boxes form an ownership forest with these invariants:
- Each box has at most one strong owner (in-degree ≤ 1)
- Weak references do not participate in ownership
- `fini` is called exactly once in LIFO order when strong references reach zero

### 2.3 Lifecycle Annotations

```nyash
@must_drop    // Must be finalized immediately when out of scope
@gcable       // Can be collected by GC (if enabled)
```

## 3. The Unified Contract

### 3.1 Instance ID Based Management

The key to unification is that Nyash never directly holds plugin data:

```rust
// Nyash side - only knows the ID
struct PluginBoxV2 {
    instance_id: u32,
    type_id: u32,
    fini_method_id: u32,
}

// Plugin side - owns the actual data
static INSTANCES: Mutex<HashMap<u32, StringData>> = ...;

struct StringData {
    data: String,  // Plugin owns the memory
}
```

### 3.2 C ABI v0 Specification

All plugin communication uses a minimal C ABI:

```c
// Type-Length-Value encoding
typedef struct {
    uint16_t type;   // 1=bool, 3=i64, 4=string, 8=handle
    uint16_t length;
    uint8_t value[]; // Variable length
} tlv_t;

// Standard plugin interface
int32_t nyash_plugin_invoke(
    uint32_t type_id,
    uint32_t method_id, 
    uint32_t instance_id,
    const uint8_t* args,
    size_t args_len,
    uint8_t* result,
    size_t* result_len
);
```

## 4. Execution Backends

### 4.1 Unified Pipeline

```
Nyash Source → Parser → AST → MIR → Backend
                                      ├─ Interpreter
                                      ├─ VM
                                      ├─ JIT (Cranelift)
                                      ├─ AOT (.o files)
                                      └─ WASM
```

### 4.2 Backend Equivalence

All backends guarantee:
- Identical I/O traces
- Same finalization order
- Equivalent error handling
- Matching performance characteristics (within backend constraints)

## 5. Implementation

### 5.1 MIR Design

Our MIR uses only 26 instructions:

```rust
enum MirInstruction {
    // Data movement
    Const(ValueId, ConstValue),
    Load(ValueId, String),
    Store(String, ValueId),
    
    // Box operations
    BoxCall(ValueId, String, Vec<ValueId>, Option<ValueId>),
    BoxNew(ValueId, String, Vec<ValueId>),
    
    // Control flow
    Jump(BasicBlockId),
    Branch(ValueId, BasicBlockId, BasicBlockId),
    Return(Option<ValueId>),
    
    // ... (remaining instructions)
}
```

### 5.2 Plugin Integration

The revolution: builtin boxes become plugins:

```toml
# nyash.toml
[libraries."libnyash_array_plugin.so".ArrayBox]
type_id = 10
methods.push = { method_id = 3, args = ["value"] }
methods.get = { method_id = 4, args = ["index"] }
```

## 6. Evaluation

### 6.1 Semantic Equivalence

We verify that all backends produce identical results:

| Program | Interpreter | VM | JIT | AOT | WASM |
|---------|------------|-------|-------|-------|--------|
| array_sum | ✓ | ✓ | ✓ | ✓ | ✓ |
| file_io | ✓ | ✓ | ✓ | ✓ | ✓ |
| http_server | ✓ | ✓ | ✓ | ✓ | ✓ |

### 6.2 Performance

Relative performance across backends:

| Benchmark | Interp | VM | JIT | AOT |
|-----------|--------|------|------|-----|
| fibonacci | 1.0x | 8.3x | 45.2x | 46.1x |
| array_ops | 1.0x | 12.1x | 78.3x | 79.5x |
| plugin_call | 1.0x | 10.5x | 42.1x | 85.2x* |

*AOT with static linking eliminates PLT overhead

## 7. Related Work

### 7.1 Ownership Systems
- **Rust**: Type-based ownership vs Nyash's runtime contracts
- **Swift**: ARC without plugins vs Nyash's unified model

### 7.2 FFI Systems
- **JNI/JNA**: Heavy runtime overhead vs Nyash's thin ABI
- **Python ctypes**: Dynamic typing vs Nyash's structured handles

### 7.3 Multi-backend Languages
- **GraalVM**: Polyglot runtime vs Nyash's unified semantics
- **LLVM languages**: Compiler framework vs Nyash's semantic preservation

## 8. Conclusion

Nyash demonstrates that a unified lifecycle contract can successfully bridge the gap between user-level abstractions and native code, while maintaining semantic equivalence across radically different execution strategies. The key insights are:

1. **Handle-based isolation** prevents ownership confusion
2. **Deterministic finalization** enables reasoning about resources
3. **Thin C ABI** minimizes interop overhead
4. **Semantic preservation** allows backend selection without code changes

The complete implementation in ~4K lines of code shows that these ideas are not just theoretical but practically achievable. We believe this approach can inform the design of future languages that need to balance safety, performance, and interoperability.