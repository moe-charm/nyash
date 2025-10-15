# Phase 20.6 — Execution Plan (VM Core Complete + Dispatch Unification)

**Status**: Planning
**Duration**: 12 weeks (2026-03-01 → 2026-05-24)
**Prerequisite**: Phase 20.5 complete (VM Foundations PoC + op_eq Migration)

---

## Executive Summary (5 lines)

1. **Goal**: Complete Hakorune VM Core (16 MIR instructions) + Unify Dispatch (single Resolver path).
2. **Strategy**: Phase B (Week 1-6) = All instructions; Phase C (Week 7-12) = Resolver integration.
3. **Verification**: Golden Tests (100+ cases) ensure Rust-VM vs Hako-VM 100% output parity.
4. **Performance**: Target ≥ 50% of Rust-VM speed (acceptable for PoC, optimize later).
5. **Policy**: Fail-Fast (unknown methods → RuntimeError), no special-case dispatch, Everything is Box.

---

## Phase Breakdown

### Phase B Complete: VM Core (Week 1-6)

**Objective**: Implement all 16 MIR instructions in Hakorune.

#### Week 1-2: Memory Operations (2026-03-01 - 03-14)

**Instructions**:
- `load`: Load value from memory
- `store`: Store value to memory
- `copy`: Copy value between registers

**Deliverables**:
- [ ] `load_handler.hako` implementation
- [ ] `store_handler.hako` implementation
- [ ] `copy_handler.hako` implementation
- [ ] 10 test cases (memory operations)

**Testing**:
```bash
# Memory operation tests
tests/golden/phase-b/memory/
├── test_load_store.hako
├── test_copy_register.hako
├── test_memory_aliasing.hako
└── ... (7 more)
```

---

#### Week 3-4: Method Call Instructions (2026-03-15 - 03-28)

**Instructions**:
- `call`: Function call (Global/Module/Closure)
- `boxcall`: Box method call
- `externcall`: External function call (e.g., nyrt.time)

**Deliverables**:
- [ ] `call_handler.hako` implementation (Unified MirCall)
- [ ] `boxcall_handler.hako` implementation
- [ ] `externcall_handler.hako` implementation
- [ ] 15 test cases (call operations)

**Testing**:
```bash
# Method call tests
tests/golden/phase-b/calls/
├── test_global_function.hako
├── test_module_function.hako
├── test_closure_call.hako
├── test_box_method.hako
├── test_extern_call.hako
└── ... (10 more)
```

**Key Design**:
```hakorune
// Unified MirCall handler (call instruction)
static box MirCallHandlerBox {
    handle(inst_json, regs, mem) {
        local callee_type = extract_callee_type(inst_json)

        return match callee_type {
            "Global" => me._handle_global(inst_json, regs)
            "ModuleFunction" => me._handle_module(inst_json, regs)
            "Closure" => me._handle_closure(inst_json, regs, mem)
            _ => Result.Err("Unknown callee type")
        }
    }
}
```

---

#### Week 5: Type Operations (2026-03-29 - 04-04)

**Instructions**:
- `typeop`: Type conversion/checking
- `newbox`: Box creation

**Deliverables**:
- [ ] `typeop_handler.hako` implementation
- [ ] `newbox_handler.hako` implementation
- [ ] 10 test cases (type operations)

**Testing**:
```bash
# Type operation tests
tests/golden/phase-b/types/
├── test_type_check.hako
├── test_type_cast.hako
├── test_newbox_simple.hako
├── test_newbox_with_fields.hako
└── ... (6 more)
```

---

#### Week 6: Control Flow Optimizations + Golden Tests (2026-04-05 - 04-11)

**Instructions**:
- `barrier`: GC barrier
- `safepoint`: GC safepoint
- `loopform`: Loop detection hint
- `unaryop`: Unary operations

**Golden Test Suite** (100+ tests):
```bash
tests/golden/hakorune-vm/
├── arithmetic/       # 10 tests (add, mul, div, mod, ...)
├── control_flow/     # 10 tests (if, loop, branch, ...)
├── collections/      # 10 tests (array, map operations)
├── recursion/        # 5 tests (factorial, fibonacci, ...)
├── closures/         # 5 tests (capture, nested, ...)
├── strings/          # 10 tests (concat, substring, ...)
├── types/            # 10 tests (type checks, casts)
├── memory/           # 10 tests (load, store, aliasing)
└── complex/          # 30 tests (integration scenarios)
```

**Deliverables**:
- [ ] Remaining instruction implementations
- [ ] Golden Test Suite complete (100+ tests)
- [ ] Performance measurement: Hako-VM ≥ 50% of Rust-VM

**Golden Test Execution**:
```bash
# Run Golden Tests
bash tools/golden_test_hakorune_vm.sh

# Expected output:
# ✅ arithmetic: 10/10 PASS
# ✅ control_flow: 10/10 PASS
# ✅ collections: 10/10 PASS
# ✅ recursion: 5/5 PASS
# ✅ closures: 5/5 PASS
# ✅ strings: 10/10 PASS
# ✅ types: 10/10 PASS
# ✅ memory: 10/10 PASS
# ✅ complex: 30/30 PASS
# ────────────────────────────
# Total: 100/100 PASS (100%)
```

---

### Phase C: Dispatch Unification (Week 7-12)

**Objective**: Unify all method calls through single Resolver path.

#### Week 7-8: Resolver Integration (2026-04-12 - 04-25)

**Goal**: Implement `Resolver.lookup(type_id, method, arity) -> MethodHandle`

**Design**:
```hakorune
// Resolver: Single resolution path for all method calls
static box ResolverBox {
    _method_registry: MapBox  // type_id -> method_name -> MethodHandle

    // Main entry point
    lookup(type_id: IntegerBox,
           method: StringBox,
           arity: IntegerBox) {
        // 1. Search type registry
        local type_methods = me._method_registry.get(type_id)
        if type_methods.is_null() {
            return Result.Err("Unknown type: " + type_id)
        }

        // 2. Lookup method by name + arity
        local method_key = method + "/" + arity
        local handle = type_methods.get(method_key)
        if handle.is_null() {
            // Fail-Fast: No fallback
            return Result.Err("Method not found: " + method_key)
        }

        return Result.Ok(handle)
    }

    // Registration (called at startup)
    register_type(type_id: IntegerBox,
                  methods: MapBox) {
        me._method_registry.set(type_id, methods)
    }
}
```

**Deliverables**:
- [ ] `resolver_box.hako` implementation
- [ ] `method_handle_box.hako` implementation
- [ ] Type registry initialization
- [ ] 20 integration tests

**Testing**:
```bash
tests/golden/phase-c/resolver/
├── test_lookup_success.hako
├── test_lookup_failure.hako (Fail-Fast)
├── test_arity_mismatch.hako
├── test_type_not_found.hako
└── ... (16 more)
```

---

#### Week 9-10: CallableBox Refactoring (2026-04-26 - 05-09)

**Goal**: Single entry point for all invocations.

**Design**:
```hakorune
// ExecBox: Unified invocation
static box ExecBox {
    call_by_handle(handle: MethodHandleBox,
                   args: ArrayBox,
                   guard: NoOperatorGuard) {
        // 1. Extract implementation from handle
        local impl_func = handle.get_implementation()

        // 2. Validate arguments
        local expected_arity = handle.get_arity()
        if args.size() != expected_arity {
            return Result.Err("Arity mismatch")
        }

        // 3. NoOperatorGuard prevents recursion (e.g., in op_eq)
        if guard.is_active() {
            // Prevent re-entry
            return Result.Err("Recursive operator call")
        }

        // 4. Invoke
        guard.activate()
        local result = impl_func.call(args)
        guard.deactivate()

        return result
    }
}
```

**Macro Desugaring**:
```hakorune
// Before (user code):
local result = arr.push(value)

// After (desugared by compiler):
local method_ref = Callable.ref_method(arr, :push, 1)
local result = method_ref.call([value])

// Which internally calls:
// Resolver.lookup(arr.type_id, "push", 1) -> handle
// ExecBox.call_by_handle(handle, [value], NoOperatorGuard)
```

**Deliverables**:
- [ ] `exec_box.hako` implementation
- [ ] `no_operator_guard_box.hako` implementation
- [ ] Macro desugaring implementation (compiler side)
- [ ] 25 test cases

**Testing**:
```bash
tests/golden/phase-c/callable/
├── test_call_by_handle.hako
├── test_no_operator_guard.hako
├── test_macro_desugar_method.hako
├── test_macro_desugar_operator.hako
└── ... (21 more)
```

---

#### Week 11: Universal Route Minimization (2026-05-10 - 05-16)

**Goal**: Remove all special-case dispatch, delegate everything to Resolver.

**Before (multiple dispatch paths)**:
```rust
// Rust VM (old approach)
match call_type {
    Global => dispatch_global(...),
    BoxMethod => dispatch_box_method(...),
    Closure => dispatch_closure(...),
    Constructor => dispatch_constructor(...),
    ModuleFunction => dispatch_module_function(...),
    // 5 different code paths!
}
```

**After (single dispatch path)**:
```hakorune
// Hakorune VM (unified approach)
// All calls:
local handle = Resolver.lookup(type_id, method, arity)
local result = ExecBox.call_by_handle(handle, args, NoOperatorGuard)

// No special cases!
```

**Removal List**:
- ❌ Remove: Special-case dispatch (Global/Box/Closure/Constructor/Module)
- ❌ Remove: Hardcoded method tables
- ❌ Remove: Fallback implementations (use Fail-Fast instead)

**Deliverables**:
- [ ] Special-case dispatch removal complete
- [ ] Codebase cleanup (reduce complexity)
- [ ] 30 edge case tests (ensure Fail-Fast works)

**Testing**:
```bash
tests/golden/phase-c/unification/
├── test_no_global_special_case.hako
├── test_no_box_special_case.hako
├── test_fail_fast_unknown_method.hako
├── test_fail_fast_wrong_arity.hako
└── ... (26 more)
```

---

#### Week 12: Integration + Documentation (2026-05-17 - 05-24)

**Goal**: Phase 20.6 completion report.

**Integration Testing**:
- [ ] Re-run all Golden Tests (100+ tests ALL PASS)
- [ ] Performance testing (≥ 50% of Rust-VM)
- [ ] Stress testing (large programs)
- [ ] Regression testing (no breakage)

**Performance Benchmark**:
```bash
# Benchmark script
bash tools/benchmark_hakorune_vm.sh

# Expected output:
# Rust-VM:     10.2s
# Hako-VM:     18.5s (55% of Rust-VM) ✅
# Memory:      180MB (< 200MB target) ✅
```

**Documentation**:
- [ ] VM Core Complete Reference
- [ ] Dispatch Unification Design Doc
- [ ] Performance Report
- [ ] Phase 20.6 Completion Report
- [ ] Phase 20.7 Planning Doc

---

## Success Criteria (DoD)

### Technical

1. **VM Core Complete**:
   - [ ] All 16 MIR instructions working
   - [ ] Control flow complete (branch, phi, loopform)
   - [ ] Memory operations complete (load, store, copy)

2. **Golden Tests**:
   - [ ] 100+ test cases ALL PASS
   - [ ] Rust-VM vs Hako-VM: 100% output match
   - [ ] Edge cases covered

3. **Dispatch Unified**:
   - [ ] All method calls go through Resolver
   - [ ] Special-case dispatch completely removed
   - [ ] Fail-Fast verified (unknown methods → immediate error)

### Performance

- [ ] **Hako-VM ≥ 50% of Rust-VM** (execution speed)
- [ ] **Memory usage**: < 200MB (typical programs)
- [ ] **Compile time**: < 10s (small programs)

### Quality

- [ ] **Test coverage**: All instructions and control paths covered
- [ ] **Documentation**: Architecture, design, migration guide complete
- [ ] **Code review**: Approved by ChatGPT + Claude

---

## Risk Mitigation

### Risk 1: Implementation Complexity

**Issue**: Implementing all 16 instructions is complex.

**Impact**: HIGH

**Mitigation**:
- Use Rust VM as reference implementation
- Incremental implementation (2-4 instructions/week)
- Dedicated test suite per instruction
- Golden Tests catch bugs early

### Risk 2: Dispatch Unification Difficulty

**Issue**: Removing special cases has wide impact.

**Impact**: MEDIUM

**Mitigation**:
- Gradual migration: Add new code → Test → Remove old code
- Implement and test Resolver first
- Feature flag for toggle
- Strengthen regression tests

### Risk 3: Performance Degradation

**Issue**: Hako-VM may be slower than Rust-VM.

**Impact**: MEDIUM

**Mitigation**:
- Initial target: 50% (acceptable)
- Weekly benchmark measurements
- Early bottleneck identification
- Optimization deferred to Phase 20.7

### Risk 4: Golden Test Mismatches

**Issue**: Rust-VM and Hako-VM outputs may differ.

**Impact**: HIGH

**Mitigation**:
- Ensure determinism (JSON normalization, sorted keys)
- Replace float operations with integers
- Eliminate randomness (timestamps, PIDs)
- Detailed diff logging

---

## Test Strategy

### Golden Test Framework

**Goal**: Prove Hakorune-VM produces identical output to Rust-VM.

**Test Suite Structure**:
```
tests/golden/
├── phase-b/               # Week 1-6 (VM Core)
│   ├── memory/            # 10 tests (load, store, copy)
│   ├── calls/             # 15 tests (call, boxcall, externcall)
│   ├── types/             # 10 tests (typeop, newbox)
│   └── control/           # 15 tests (barrier, safepoint, loopform)
├── phase-c/               # Week 7-12 (Dispatch)
│   ├── resolver/          # 20 tests (lookup, registration)
│   ├── callable/          # 25 tests (call_by_handle, guard)
│   └── unification/       # 30 tests (edge cases, Fail-Fast)
└── integration/           # Week 12
    ├── arithmetic.hako    # Basic arithmetic
    ├── control_flow.hako  # if/loop/branch
    ├── collections.hako   # Array/Map operations
    ├── recursion.hako     # Recursive functions
    ├── strings.hako       # String manipulation
    ├── enums.hako         # @enum types
    └── closures.hako      # Closure capture
```

**Verification Script**:
```bash
#!/bin/bash
# tools/golden_test_hakorune_vm.sh

for test in tests/golden/**/*.hako; do
    echo "Testing: $test"

    # Run on Rust VM
    ./hako --backend vm-rust "$test" > /tmp/rust_out.txt 2>&1
    rust_exit=$?

    # Run on Hakorune VM
    ./hako --backend vm "$test" > /tmp/hako_out.txt 2>&1
    hako_exit=$?

    # Compare outputs
    if diff /tmp/rust_out.txt /tmp/hako_out.txt && [ $rust_exit -eq $hako_exit ]; then
        echo "  ✅ PASS"
    else
        echo "  ❌ FAIL"
        exit 1
    fi
done

echo "✅ All Golden Tests PASSED!"
```

---

## CI Integration

```yaml
# .github/workflows/golden_tests.yml
name: Golden Tests (Phase 20.6)
on: [push, pull_request]

jobs:
  golden-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Build Hakorune
        run: cargo build --release

      - name: Run Golden Tests
        run: bash tools/golden_test_hakorune_vm.sh

      - name: Performance Benchmark
        run: |
          bash tools/benchmark_hakorune_vm.sh
          # Verify: Hako-VM ≥ 50% of Rust-VM
```

---

## Dependencies

### From Phase 20.5

- [x] HostBridge API (C-ABI boundary)
- [x] op_eq Migration (NoOperatorGuard)
- [x] VM Foundations PoC (5 instructions)

### For Phase 20.6

- [ ] Resolver design finalized (Week 7)
- [ ] MethodHandle design finalized (Week 7)
- [ ] Golden Test infrastructure ready (Week 1)
- [ ] Benchmark infrastructure ready (Week 1)

---

## Next Steps (Actionable)

1. **Week 1 Preparation** (before 2026-03-01):
   - [ ] Lock Golden Test list (100+ tests)
   - [ ] Set up test infrastructure
   - [ ] Prepare benchmark suite
   - [ ] Review Phase 20.5 deliverables

2. **Week 1-6 Execution**:
   - [ ] Implement instructions incrementally
   - [ ] Run Golden Tests continuously
   - [ ] Measure performance weekly

3. **Week 7-12 Execution**:
   - [ ] Implement Resolver + CallableBox
   - [ ] Remove special-case dispatch
   - [ ] Verify Fail-Fast behavior

4. **Week 12 Completion**:
   - [ ] All Golden Tests PASS
   - [ ] Performance ≥ 50% verified
   - [ ] Documentation complete
   - [ ] Phase 20.7 planning doc ready

---

**Created**: 2025-10-14
**Phase Start**: 2026-03-01
**Phase End**: 2026-05-24
**Duration**: 12 weeks
**Prerequisite**: Phase 20.5 complete
**Success**: VM Core Complete + Dispatch Unified ✅
