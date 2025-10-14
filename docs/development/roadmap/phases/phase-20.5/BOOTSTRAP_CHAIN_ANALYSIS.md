# Bootstrap Chain Analysis — Phase 20.5

Purpose: Detailed analysis of the 3-stage bootstrap chain for achieving true self-hosting

---

## 🎯 Overview

**Goal**: Establish a bootstrap chain where Hakorune can compile itself

```
Stage 1 (Rust Frozen)  →  Stage 2 (Hako v1)  →  Stage 3 (Hako v2)
     Trusted                 Bootstrap            Verification
```

---

## 📊 Three-Stage Bootstrap Chain

### Stage 1: Rust Compiler (Frozen Toolchain)

**Identity**:
- Binary: `hako-frozen-v1.exe` (724KB MSVC, 7.4MB MinGW)
- Language: Rust
- Status: **Frozen** (no changes after Phase 15.77)
- Git Tag: `v1.0.0-frozen`

**Capabilities**:
- Parse Hakorune source → AST
- Lower AST → MIR JSON
- Execute MIR (VM mode)
- Call NyRT functions via C ABI

**Inputs/Outputs**:
```
Input:  program.hako (Hakorune source)
Output: program.mir.json (MIR JSON)
        OR
        program.exe (via AOT: MIR → .o → EXE)
```

**Role in Bootstrap**:
- Compile the Hakorune-written compiler (Stage 2)
- Provide trusted baseline for verification
- Emergency fallback if Stage 2/3 fail

**Constraints**:
- No modifications allowed (frozen)
- Limited Box set (String, Array, Map, Console, Time, JSON, File[min])
- Must remain stable for reproducibility

---

### Stage 2: Hakorune Compiler v1 (Bootstrap)

**Identity**:
- Source: `apps/bootstrap-compiler/**/*.hako`
- Implementation: ~3000 lines Hakorune code
- Compiled by: Stage 1 (frozen EXE)
- Execution: On frozen EXE VM

**Capabilities**:
- Parse Hakorune source → AST JSON
- Lower AST → MIR JSON
- Generate C code from MIR JSON
- Output: `.c` files that link with NyRT

**Inputs/Outputs**:
```
Input:  program.hako (Hakorune source)
Output: program.c (C source code)

Execution:
  ./hako-frozen-v1 apps/bootstrap-compiler/main.hako \
    --input program.hako \
    --output program.c
```

**Role in Bootstrap**:
- **Primary compiler**: Compile arbitrary Hakorune programs
- **Self-compilation**: Compile its own source (Stage 2 → Stage 3)
- **Verification baseline**: Reference for v2 output

**Implementation Strategy**:
```
apps/bootstrap-compiler/
├── parser/              # Reuse from apps/selfhost-compiler/
│   ├── parser_box.hako  # 90% reusable
│   └── lexer_box.hako
├── mir_builder/         # Reuse from apps/selfhost-compiler/
│   └── builder_box.hako # 80% reusable
├── codegen/             # NEW - C Code Generator
│   ├── c_emitter_box.hako
│   └── c_runtime_box.hako
└── main.hako            # Entry point
```

**Key Features**:
1. **C Output**: Unlike frozen EXE (MIR JSON), outputs C code
2. **Self-Hosting**: Can compile itself
3. **NyRT Integration**: Generated C calls NyRT functions
4. **Verification**: Must match Stage 3 output

---

### Stage 3: Hakorune Compiler v2 (Verification)

**Identity**:
- Source: Same as Stage 2 (`apps/bootstrap-compiler/**/*.hako`)
- Compiled by: Stage 2 (Hakorune v1)
- Execution: As standalone EXE (or on frozen VM)

**Capabilities**:
- **Identical to Stage 2**
- Parse → MIR → C code generation

**Inputs/Outputs**:
```
Input:  program.hako
Output: program.c (must be identical to Stage 2 output)

Execution:
  # Compile v2 using v1
  ./hako-frozen-v1 apps/bootstrap-compiler/main.hako \
    --input apps/bootstrap-compiler/main.hako \
    --output bootstrap_v2.c

  # Compile bootstrap_v2.c → v2 binary
  clang bootstrap_v2.c -o bootstrap_v2 -lhako_kernel

  # Use v2 to compile a test program
  ./bootstrap_v2 --input test.hako --output test_v2.c
```

**Role in Bootstrap**:
- **Verification**: Prove v1 == v2 (identical output)
- **Self-Consistency**: v2 can compile v3, v3 == v2
- **Confidence**: If v1 == v2 == v3, bootstrap is successful

**Verification Process**:
```bash
# Step 1: v1 compiles test.hako
./hako-frozen-v1 apps/bootstrap-compiler/main.hako \
  --input test.hako --output test_v1.c

# Step 2: v1 compiles itself → v2
./hako-frozen-v1 apps/bootstrap-compiler/main.hako \
  --input apps/bootstrap-compiler/main.hako \
  --output bootstrap_v2.c

# Step 3: Build v2 binary
clang bootstrap_v2.c -o bootstrap_v2 -lhako_kernel

# Step 4: v2 compiles test.hako
./bootstrap_v2 --input test.hako --output test_v2.c

# Step 5: Verify v1 == v2
diff test_v1.c test_v2.c
# Expected: No differences

# Step 6 (optional): v2 compiles itself → v3
./bootstrap_v2 --input apps/bootstrap-compiler/main.hako \
  --output bootstrap_v3.c

# Step 7: Verify v2 == v3
diff bootstrap_v2.c bootstrap_v3.c
# Expected: No differences
```

---

## 🔄 Data Flow Analysis

### Stage 1 → Stage 2

**Input**: Hakorune compiler source (`apps/bootstrap-compiler/`)

**Process**:
```
[Hakorune Source]
        ↓
   Stage 1: hako-frozen-v1.exe
   - Parser (Rust)
   - MIR Builder (Rust)
   - VM Executor (Rust)
        ↓
[Hakorune Compiler v1 Running on VM]
   - Capabilities: Parse, MIR Build, C Gen
```

**Output**: Running Hakorune compiler (v1)

**Key Points**:
- v1 runs **on** the frozen EXE VM
- v1 is **interpreted**, not compiled to native
- v1 has access to frozen EXE's Box set (String, Array, Map, etc.)

---

### Stage 2 → Stage 3

**Input**: Hakorune compiler source (same as Stage 1 input)

**Process**:
```
[Hakorune Compiler Source]
        ↓
   Stage 2: Hakorune Compiler v1
   - Parser (Hakorune)
   - MIR Builder (Hakorune)
   - C Generator (Hakorune)
        ↓
[bootstrap_v2.c]
        ↓
   clang + NyRT
        ↓
[bootstrap_v2 EXE]
```

**Output**: Standalone Hakorune compiler binary (v2)

**Key Points**:
- v2 is **native binary** (compiled C → EXE)
- v2 is **independent** (doesn't need frozen EXE to run)
- v2 must produce **identical output** to v1

---

### Stage 3 → Verification

**Process**:
```
Test Program (test.hako)
        │
  ┌─────┴─────┐
  │           │
  v           v
Stage 2      Stage 3
(v1)         (v2)
  │           │
  v           v
test_v1.c   test_v2.c
  │           │
  └─────┬─────┘
        │
     diff
        │
        v
   Identical? ✅
```

**Verification Criteria**:
1. **Bytecode Level**: test_v1.c == test_v2.c (character-by-character)
2. **Semantic Level**: Compiled EXEs produce same output
3. **Recursive**: v2 → v3, v3 == v2 (fixed point)

---

## ⚙️ Technical Constraints

### Stage 1 Constraints (Frozen EXE)

**Available Boxes**:
```
✅ String              - Full support
✅ Array               - Full support
✅ Map                 - Full support
✅ Console (print)     - Output only
✅ Time (now_ms)       - Timing
✅ JSON (stringify)    - JSON generation
✅ File[min]           - Read/write (minimal)
```

**NOT Available**:
```
❌ Regex              - Too heavy for frozen
❌ Network            - Security concern
❌ OS/Path (extended) - Environment-specific
❌ Crypto             - Not needed for compiler
```

**Implications**:
- Hakorune compiler must work with limited Box set
- No regex for parsing (use manual string ops)
- No network I/O for compiler
- File I/O limited to read source, write output

### Stage 2 Constraints (Hakorune v1)

**Execution Environment**:
- Runs **on** frozen EXE VM (interpreted)
- No native compilation until Stage 3
- Performance: ~10x slower than native (acceptable)

**Implementation Constraints**:
- Must use only frozen EXE Box set
- Cannot rely on Rust-specific features
- Must be pure Hakorune code

**Memory Constraints**:
- VM register limit: 256 per function (typical)
- Stack depth: Limited by VM (avoid deep recursion)
- Heap: Managed by frozen EXE GC

### Stage 3 Constraints (Hakorune v2)

**Binary Constraints**:
- Must link with NyRT (`libhako_kernel.a`)
- C code must be valid C11
- No undefined behavior

**Verification Constraints**:
- Output must be **deterministic**
- No timestamps, PIDs, or non-deterministic data in output
- Identical AST/MIR JSON for same input

---

## 🎯 Success Criteria

### Functional Success

1. **Stage 1 → 2 Works**:
   ```bash
   ./hako-frozen-v1 apps/bootstrap-compiler/main.hako \
     --input hello.hako --output hello_v1.c
   # ✅ Compiles successfully
   ```

2. **Stage 2 → 3 Works**:
   ```bash
   ./bootstrap_v1 --input apps/bootstrap-compiler/main.hako \
     --output bootstrap_v2.c
   clang bootstrap_v2.c -o bootstrap_v2 -lhako_kernel
   # ✅ Builds successfully
   ```

3. **v1 == v2 Verification**:
   ```bash
   diff <(./bootstrap_v1 --input test.hako) \
        <(./bootstrap_v2 --input test.hako)
   # ✅ No differences
   ```

4. **v2 == v3 Fixed Point**:
   ```bash
   ./bootstrap_v2 --input apps/bootstrap-compiler/main.hako \
     --output bootstrap_v3.c
   diff bootstrap_v2.c bootstrap_v3.c
   # ✅ No differences (fixed point reached)
   ```

### Performance Success

1. **Stage 2 Compile Time**:
   - Simple program (< 100 lines): < 2 seconds
   - Medium program (< 1000 lines): < 10 seconds
   - Compiler itself (3000 lines): < 30 seconds

2. **Stage 3 Compile Time**:
   - Should be ~10x faster than Stage 2 (native vs interpreted)
   - Simple program: < 0.5 seconds
   - Medium program: < 2 seconds
   - Compiler itself: < 5 seconds

3. **Memory Usage**:
   - Stage 2: < 100MB
   - Stage 3: < 50MB

### Quality Success

1. **Test Coverage**:
   - 10+ test programs compile correctly
   - All 16 MIR instructions covered
   - Edge cases tested (recursion, loops, etc.)

2. **Error Handling**:
   - Parse errors: Clear messages
   - MIR errors: Diagnostic output
   - C generation errors: Fail-fast with context

3. **Maintainability**:
   - Code is modular (Box-based)
   - Each component has tests
   - Documentation for each Box

---

## 🔍 Verification Strategy

### Level 1: Smoke Tests (Fast)

**Goal**: Quick sanity check

```bash
# Test 1: Hello World
echo 'static box Main { main() { return 42 } }' > hello.hako
./bootstrap_v1 --input hello.hako --output hello_v1.c
./bootstrap_v2 --input hello.hako --output hello_v2.c
diff hello_v1.c hello_v2.c  # ✅

# Test 2: Arithmetic
cat > arith.hako << 'EOF'
static box Main {
  main() {
    local x = 10
    local y = 20
    return x + y
  }
}
EOF
./bootstrap_v1 --input arith.hako --output arith_v1.c
./bootstrap_v2 --input arith.hako --output arith_v2.c
diff arith_v1.c arith_v2.c  # ✅
```

### Level 2: Comprehensive Tests (Medium)

**Goal**: Test all language features

```bash
# Test Suite: 10 programs covering:
# - If/else
# - Loops
# - Functions
# - Boxes
# - Arrays
# - Strings
# - Recursion
# - etc.

for test in tests/*.hako; do
  name=$(basename "$test" .hako)
  ./bootstrap_v1 --input "$test" --output "${name}_v1.c"
  ./bootstrap_v2 --input "$test" --output "${name}_v2.c"
  diff "${name}_v1.c" "${name}_v2.c" || exit 1
done
echo "✅ All tests passed"
```

### Level 3: Self-Compilation (Slow)

**Goal**: Verify fixed point (v2 == v3)

```bash
# Compile v2
./bootstrap_v1 --input apps/bootstrap-compiler/main.hako \
  --output bootstrap_v2.c
clang bootstrap_v2.c -o bootstrap_v2 -lhako_kernel

# Compile v3 using v2
./bootstrap_v2 --input apps/bootstrap-compiler/main.hako \
  --output bootstrap_v3.c

# Verify v2 == v3
diff bootstrap_v2.c bootstrap_v3.c
echo "✅ Fixed point reached: v2 == v3"

# (Optional) Compile v4 using v3, verify v3 == v4
clang bootstrap_v3.c -o bootstrap_v3 -lhako_kernel
./bootstrap_v3 --input apps/bootstrap-compiler/main.hako \
  --output bootstrap_v4.c
diff bootstrap_v3.c bootstrap_v4.c
echo "✅ Fixed point stable: v3 == v4"
```

---

## 📊 Bootstrap Timeline Estimate

### Week 3-4: Parser Adaptation (Stage 1 → 2 foundation)
- Migrate apps/selfhost-compiler/parser/ → apps/bootstrap-compiler/
- Adapt to frozen EXE constraints
- Test: 10 parsing tests PASS

**Output**: Stage 2 can parse Hakorune → AST JSON

### Week 5-6: MIR Builder (Stage 1 → 2 complete)
- Migrate MIR Builder
- Support 16 instructions
- Test: 10 MIR generation tests PASS

**Output**: Stage 2 can parse → MIR JSON

### Week 7-8: C Code Generator (Stage 2 → 3 foundation)
- Implement C emitter
- 16 instructions → C
- Test: 43 C generation tests PASS

**Output**: Stage 2 can parse → MIR → C

### Week 9: Bootstrap Integration (Stage 2 ↔ 3)
- Compile v2 using v1
- Verify v1 == v2 (10 tests)
- Verify v2 == v3 (fixed point)

**Output**: Bootstrap chain complete, verified

---

## ⚠️ Risk Analysis

### Risk 1: v1 != v2 (Output Mismatch)

**Probability**: MEDIUM
**Impact**: HIGH

**Causes**:
- Non-deterministic output (timestamps, PIDs)
- Floating-point precision differences
- Hash map iteration order
- Different AST/MIR construction

**Mitigation**:
- Enforce deterministic output
- Canonical JSON formatting (sorted keys)
- Test incrementally (Stage 1 → 2 first)
- Golden tests with known outputs

### Risk 2: Performance Too Slow

**Probability**: LOW
**Impact**: MEDIUM

**Causes**:
- Stage 2 is interpreted (10x slower)
- Inefficient algorithms
- Excessive memory allocation

**Mitigation**:
- Profile Stage 2 execution
- Optimize hot paths
- Acceptable threshold: < 30s for self-compilation

### Risk 3: Frozen EXE Constraints Too Limiting

**Probability**: LOW
**Impact**: MEDIUM

**Causes**:
- Missing Box functionality
- File I/O limitations
- Memory constraints

**Mitigation**:
- Pre-survey required Boxes (done)
- Workarounds in Hakorune code
- Minimal compiler design (no advanced features)

---

## 🎉 Success Impact

After Bootstrap Chain is verified:

1. **True Self-Hosting**: Hakorune compiles Hakorune
2. **Reproducibility**: v2 == v3 proves determinism
3. **Independence**: No Rust needed for new features
4. **Foundation**: Ready for Phase 20.6 (complete Rust removal)

---

## 📚 Industry Examples

### Rust Bootstrap

```
stage0 (frozen)  →  stage1 (bootstrap)  →  stage2 (verify)
     |                    |                       |
   rustc           rustc (built by stage0)   rustc (built by stage1)
  (frozen)
                        Verify: stage1 == stage2
```

### Go Bootstrap

```
Go 1.4 (C)  →  Go 1.5 (built by Go 1.4)  →  Go 1.6 (built by Go 1.5)
   |                   |                           |
Frozen            Bootstrap                   Verification
```

### Hakorune Bootstrap (Our Plan)

```
hako-frozen-v1.exe  →  bootstrap_v1 (Hako)  →  bootstrap_v2 (Hako)
      |                        |                       |
    Rust                  Interpreted              Native Binary
   (frozen)              (on frozen VM)           (standalone)

                        Verify: v1 == v2 == v3 (fixed point)
```

---

**Created**: 2025-10-14
**Phase**: 20.5
**Component**: Bootstrap Chain Analysis
