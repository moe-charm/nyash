# C Code Generator Design — Phase 15.79

Purpose: Convert MIR JSON to executable C code that links with NyRT runtime

---

## 🎯 Design Goals

1. **Correctness First**: Readable output is secondary to correct execution
2. **NyRT Dependency**: All Box operations via NyRT function calls
3. **16-Instruction Complete**: Support entire MIR frozen instruction set
4. **Test-Driven**: Each instruction has dedicated test cases

---

## 📋 MIR Instruction Set → C Mapping

### 1. Const

**MIR**:
```json
{
  "op": "const",
  "dst": 0,
  "value": {"type": "Int", "value": 42}
}
```

**C Output**:
```c
int64_t v0 = 42;
```

**String Constant**:
```json
{
  "op": "const",
  "dst": 1,
  "value": {"type": "String", "value": "Hello"}
}
```

**C Output**:
```c
int64_t v1 = nyash_box_from_i8_string("Hello");
```

---

### 2. BinOp

**MIR**:
```json
{
  "op": "binop",
  "dst": 2,
  "kind": "Add",
  "lhs": 0,
  "rhs": 1
}
```

**C Output**:
```c
int64_t v2 = v0 + v1;  // For integers
```

**Box BinOp** (via NyRT):
```c
int64_t v2 = nyrt_int_add(v0, v1);
```

**All BinOp Kinds**:
```c
Add  → +  (or nyrt_int_add)
Sub  → -  (or nyrt_int_sub)
Mul  → *  (or nyrt_int_mul)
Div  → /  (or nyrt_int_div)
Mod  → %  (or nyrt_int_mod)
```

---

### 3. Compare

**MIR**:
```json
{
  "op": "compare",
  "dst": 3,
  "kind": "Gt",
  "lhs": 0,
  "rhs": 1
}
```

**C Output**:
```c
int64_t v3 = (v0 > v1) ? 1 : 0;
```

**All Compare Kinds**:
```c
Eq  → ==
Ne  → !=
Lt  → <
Le  → <=
Gt  → >
Ge  → >=
```

---

### 4. Jump

**MIR**:
```json
{
  "op": "jump",
  "target": "bb1"
}
```

**C Output**:
```c
goto bb1;
```

---

### 5. Branch

**MIR**:
```json
{
  "op": "branch",
  "cond": 3,
  "then_block": "bb_then",
  "else_block": "bb_else"
}
```

**C Output**:
```c
if (v3) {
  goto bb_then;
} else {
  goto bb_else;
}
```

---

### 6. Phi

**MIR**:
```json
{
  "op": "phi",
  "dst": 5,
  "inputs": [
    {"block": "bb_then", "value": 1},
    {"block": "bb_else", "value": 2}
  ]
}
```

**C Output** (pre-computed):
```c
// At bb_then:
phi_v5 = v1;
goto bb_merge;

// At bb_else:
phi_v5 = v2;
goto bb_merge;

// At bb_merge:
int64_t v5 = phi_v5;
```

**Note**: PHI requires preprocessing to convert to explicit assignments before merge blocks.

---

### 7. Return

**MIR**:
```json
{
  "op": "ret",
  "value": 3
}
```

**C Output**:
```c
return v3;
```

---

### 8. Call (Global Function)

**MIR**:
```json
{
  "op": "call",
  "dst": 4,
  "callee": "print",
  "args": [0, 1]
}
```

**C Output**:
```c
int64_t v4 = ny_print(v0, v1);
```

---

### 9. BoxCall (Method Call)

**MIR**:
```json
{
  "op": "boxcall",
  "dst": 5,
  "receiver": 0,
  "method": "concat",
  "args": [1]
}
```

**C Output** (via NyRT):
```c
int64_t args[] = {v1};
int64_t v5 = nyash_boxcall(v0, "concat", args, 1);
```

**Common Methods**:
```c
String.concat    → nyash_string_concat_hh
String.len       → nyash_string_len_h
String.substring → nyash_string_substring_hii
Array.size       → nyash_array_size_h
Array.get        → nyash_array_get_hi
Map.set          → nyash_map_set_hhh
Map.get          → nyash_map_get_hh
```

---

### 10. ExternCall

**MIR**:
```json
{
  "op": "externcall",
  "dst": 6,
  "interface": "env.console",
  "method": "log",
  "args": [0]
}
```

**C Output**:
```c
int64_t v6 = nyrt_externcall("env.console.log", v0);
```

---

### 11. Load

**MIR**:
```json
{
  "op": "load",
  "dst": 7,
  "addr": 6
}
```

**C Output**:
```c
int64_t v7 = *(int64_t*)v6;
```

---

### 12. Store

**MIR**:
```json
{
  "op": "store",
  "addr": 6,
  "value": 7
}
```

**C Output**:
```c
*(int64_t*)v6 = v7;
```

---

### 13. Copy

**MIR**:
```json
{
  "op": "copy",
  "dst": 8,
  "src": 7
}
```

**C Output**:
```c
int64_t v8 = v7;
```

---

### 14. TypeOp

**MIR**:
```json
{
  "op": "typeop",
  "dst": 9,
  "kind": "TypeCheck",
  "value": 7,
  "target_type": "String"
}
```

**C Output**:
```c
int64_t v9 = nyrt_typecheck(v7, "String");
```

---

### 15. Barrier (GC)

**MIR**:
```json
{
  "op": "barrier",
  "kind": "write",
  "addr": 6,
  "value": 7
}
```

**C Output**:
```c
nyrt_gc_barrier_write((void*)v6, v7);
```

---

### 16. Safepoint (GC)

**MIR**:
```json
{
  "op": "safepoint"
}
```

**C Output**:
```c
nyrt_gc_safepoint();
```

---

## 🏗️ Overall C Structure

### Template

```c
// === Header ===
#include <stdint.h>
#include <stdio.h>

// NyRT Function Declarations
extern int64_t nyash_box_from_i8_string(const char*);
extern int64_t nyash_string_concat_hh(int64_t, int64_t);
extern int64_t nyash_string_len_h(int64_t);
extern int64_t nyash_array_size_h(int64_t);
extern int64_t nyash_map_set_hhh(int64_t, int64_t, int64_t);
extern int64_t nyash_boxcall(int64_t, const char*, int64_t*, int);
extern int64_t nyrt_externcall(const char*, int64_t);
extern int64_t nyrt_typecheck(int64_t, const char*);
extern void nyrt_gc_barrier_write(void*, int64_t);
extern void nyrt_gc_safepoint(void);

// === Function Definitions ===
int64_t ny_main(void) {
  // Variable declarations
  int64_t v0, v1, v2, v3, v4, v5;
  int64_t phi_v5;  // PHI variables

  // === bb0 (entry) ===
bb0:
  v0 = nyash_box_from_i8_string("Hello");
  v1 = nyash_box_from_i8_string(" World");
  v2 = nyash_string_concat_hh(v0, v1);
  v3 = nyash_string_len_h(v2);
  return v3;
}

// === Main Entry Point ===
int main(int argc, char** argv) {
  int64_t result = ny_main();
  printf("Result: %lld\n", (long long)result);
  return 0;
}
```

---

## 🧪 Test Strategy

### Test Cases per Instruction

| Instruction | Test Count | Priority |
|-------------|------------|----------|
| const       | 3 (int, string, bool) | HIGH |
| binop       | 5 (add, sub, mul, div, mod) | HIGH |
| compare     | 6 (eq, ne, lt, le, gt, ge) | HIGH |
| jump        | 2 (basic, nested) | HIGH |
| branch      | 3 (true, false, nested) | HIGH |
| phi         | 4 (if-else, loop, multiple) | HIGH |
| ret         | 2 (int, box) | HIGH |
| call        | 2 (0-arg, 2-arg) | MEDIUM |
| boxcall     | 5 (string, array, map) | HIGH |
| externcall  | 2 (console, time) | MEDIUM |
| load        | 2 (basic, nested) | LOW |
| store       | 2 (basic, nested) | LOW |
| copy        | 1 (basic) | LOW |
| typeop      | 2 (typecheck, cast) | LOW |
| barrier     | 1 (basic) | LOW |
| safepoint   | 1 (basic) | LOW |

**Total Test Cases**: 43

---

## 📦 Implementation Structure

### apps/bootstrap-compiler/codegen/

```
codegen/
├── c_emitter_box.hako          # Main C emitter
│   ├── emit_function()         # Function-level emission
│   ├── emit_block()            # Basic block emission
│   └── emit_instruction()      # Instruction-level emission
├── c_header_box.hako           # Header generation
│   ├── emit_includes()
│   └── emit_nyrt_decls()
├── c_phi_resolver_box.hako     # PHI → explicit assignments
│   └── resolve_phi()
├── c_runtime_box.hako          # NyRT call helpers
│   ├── emit_boxcall()
│   ├── emit_externcall()
│   └── emit_typeop()
└── tests/                      # 43 test cases
    ├── test_const.hako
    ├── test_binop.hako
    ├── test_compare.hako
    └── ...
```

---

## 🔄 Compilation Pipeline

```
Input: program.mir.json

Step 1: Parse MIR JSON
  ↓
Step 2: Preprocess PHI instructions
  ↓
Step 3: Emit C header
  ↓
Step 4: Emit function declarations
  ↓
Step 5: For each function:
  - Emit variable declarations
  - For each basic block:
    - Emit block label
    - For each instruction:
      - Emit C statement
  ↓
Step 6: Emit main entry point
  ↓
Output: program.c

Step 7: Compile with clang
  clang program.c -o program \
    -L /path/to/hako_kernel \
    -lhako_kernel \
    -lpthread -ldl -lm
  ↓
Output: program (executable)
```

---

## 💡 PHI Resolution Strategy

### Problem

PHI nodes in SSA form don't directly translate to C:

```
MIR:
  bb_merge:
    v5 = phi [bb_then: v1, bb_else: v2]
```

C doesn't have PHI!

### Solution: Pre-Merge Assignment

```c
// bb_then:
phi_v5 = v1;  // ← Explicit assignment before jump
goto bb_merge;

// bb_else:
phi_v5 = v2;  // ← Explicit assignment before jump
goto bb_merge;

// bb_merge:
int64_t v5 = phi_v5;  // ← Load from PHI variable
```

### Algorithm

1. **Identify PHI nodes**: Scan all blocks for PHI instructions
2. **Create PHI variables**: `phi_vN` for each PHI destination
3. **Insert assignments**: Before each predecessor jump, assign `phi_vN = vX`
4. **Replace PHI**: PHI node becomes `vN = phi_vN`

---

## ⚠️ Edge Cases

### 1. String Escaping

**Problem**: C string literals need escaping

**Example**:
```
MIR: const v0 = "Hello \"World\"\n"
C:   v0 = nyash_box_from_i8_string("Hello \\\"World\\\"\\n");
```

**Escaping Rules**:
- `"` → `\"`
- `\` → `\\`
- `\n` → `\\n`
- `\t` → `\\t`

### 2. Null Values

**Problem**: How to represent null?

**Solution**:
```c
#define NYRT_NULL ((int64_t)0)
int64_t v0 = NYRT_NULL;
```

### 3. Large Constants

**Problem**: String/Array constants in MIR JSON

**Solution**:
```c
// Option A: Inline (simple)
v0 = nyash_box_from_i8_string("very long string...");

// Option B: Static data (future optimization)
static const char str_0[] = "very long string...";
v0 = nyash_box_from_i8_string(str_0);
```

### 4. Recursive Functions

**Problem**: C requires forward declarations

**Solution**:
```c
// Forward declarations
int64_t ny_func_a(int64_t);
int64_t ny_func_b(int64_t);

// Definitions
int64_t ny_func_a(int64_t arg) {
  return ny_func_b(arg + 1);
}

int64_t ny_func_b(int64_t arg) {
  if (arg > 10) return arg;
  return ny_func_a(arg * 2);
}
```

---

## 🎯 Success Criteria

### Correctness

- [ ] All 16 instructions → C correctly
- [ ] 43/43 test cases PASS
- [ ] No segfaults or memory errors
- [ ] Output matches VM execution

### Performance

- [ ] Compilation time < 1 second for 100-line programs
- [ ] Generated C compiles without warnings
- [ ] Execution time comparable to Rust VM

### Maintainability

- [ ] Each instruction has dedicated test
- [ ] Code is modular (Box-based)
- [ ] Edge cases documented
- [ ] Examples for common patterns

---

## 📚 NyRT Function Reference

### String Operations

```c
int64_t nyash_box_from_i8_string(const char* str);
int64_t nyash_string_concat_hh(int64_t lhs, int64_t rhs);
int64_t nyash_string_len_h(int64_t str);
int64_t nyash_string_substring_hii(int64_t str, int64_t start, int64_t end);
```

### Array Operations

```c
int64_t nyash_array_size_h(int64_t arr);
int64_t nyash_array_get_hi(int64_t arr, int64_t index);
int64_t nyash_array_set_hih(int64_t arr, int64_t index, int64_t value);
```

### Map Operations

```c
int64_t nyash_map_set_hhh(int64_t map, int64_t key, int64_t value);
int64_t nyash_map_get_hh(int64_t map, int64_t key);
int64_t nyash_map_size_h(int64_t map);
```

### Generic Operations

```c
int64_t nyash_boxcall(int64_t receiver, const char* method, int64_t* args, int arg_count);
int64_t nyrt_externcall(const char* interface_method, int64_t arg);
int64_t nyrt_typecheck(int64_t value, const char* type_name);
```

### GC Operations

```c
void nyrt_gc_barrier_write(void* addr, int64_t value);
void nyrt_gc_safepoint(void);
```

---

## 🔧 Implementation Plan

### Week 7: Basic Instructions

**Day 1-2**: const, copy, ret
**Day 3-4**: binop, compare
**Day 5-7**: jump, branch, phi (basic)

### Week 8: Advanced Instructions

**Day 1-2**: call, boxcall
**Day 3-4**: externcall, typeop
**Day 5-6**: load, store, barrier, safepoint
**Day 7**: Integration testing

---

**Created**: 2025-10-14
**Phase**: 15.79 (Week 7-8)
**Component**: C Code Generator
