# Box/ExternCall Architecture Design Decision

## Date: 2025-09-11

### Background
During LLVM backend development, confusion arose about the proper boundary between:
- ExternCall (core runtime functions)
- BoxCall (unified Box method dispatch)
- nyrt (Nyash runtime library)

### Design Decision

#### 1. **nyrt Built-in Core Boxes**
The following boxes are built into nyrt for self-hosting stability:

```rust
// crates/nyrt/src/core_boxes/
├── integer.rs  // IntegerBox（arithmetic, comparison）
├── string.rs   // StringBox（string operations）
├── array.rs    // ArrayBox（array operations）
├── map.rs      // MapBox（key-value storage）
└── bool.rs     // BoolBox（logical operations）
```

**Rationale**:
- Essential for self-hosting (compiler needs these)
- Available at boot time (no plugin loader dependency)
- High performance (no FFI overhead)
- Environment independent

#### 2. **Plugin Boxes**
All other boxes are implemented as plugins:
```
plugins/
├── file/      // FileBox
├── net/       // NetBox
└── custom/    // User-defined boxes
```

#### 3. **Minimal ExternCall**
ExternCall is limited to truly external operations:

```rust
// Only these 5 functions!
extern nyash.io.print(handle: i64)
extern nyash.io.error(handle: i64)  
extern nyash.runtime.panic(handle: i64)
extern nyash.runtime.exit(code: i64)
extern nyash.time.now() -> i64
```

### Key Principle: Everything Goes Through BoxCall

```nyash
local s = new StringBox("Hello")  // BoxCall → nyrt built-in
local f = new FileBox()          // BoxCall → plugin
s.concat(" World")               // BoxCall (unified interface)
```

Even core boxes use the same BoxCall mechanism - no special fast paths!

### Trade-offs Considered

**Option 1: Everything as plugins**
- ✅ Beautiful uniformity
- ❌ Complex bootstrap
- ❌ Performance overhead
- ❌ Environment dependencies

**Option 2: Core boxes in nyrt (chosen)**
- ✅ Simple, stable bootstrap
- ✅ Self-hosting friendly
- ✅ High performance for basics
- ❌ Slightly larger core

### Conclusion
This design prioritizes self-hosting stability while maintaining the "Everything is Box" philosophy through unified BoxCall interface.