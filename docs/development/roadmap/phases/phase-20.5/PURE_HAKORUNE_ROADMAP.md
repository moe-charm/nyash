# Pure Hakorune Roadmap — Phase 20.5 to 15.82

**Vision**: "Rust=floor, Hakorune=house" - Implement VM itself in Hakorune
**Timeline**: 36 weeks (2025-12-21 → 2026-09-30)
**Status**: APPROVED (User preference: "純 Hakorune 大作戦")

---

## 🎯 Overview: 6 Phases of Pure Hakorune

ChatGPT Pro's strategy divides implementation into 6 logical phases:

```
Phase A: HostBridge        (Week 1-4)    ← Phase 20.5
Phase B: VM Core           (Week 5-12)   ← Phase 20.5 (start) → 15.80 (complete)
Phase C: Dispatch Unity    (Week 13-18)  ← Phase 20.6
Phase D: Collections       (Week 19-26)  ← Phase 20.7
Phase E: GC v0             (Week 27-32)  ← Phase 20.8
Phase F: Rust Compat Mode  (Week 33-36)  ← Phase 20.8
```

**Key Insight**: Phase 20.5 (10 weeks) completes Phase A + starts Phase B.

---

## 📊 Phase Breakdown

### Phase 20.5 (10 weeks): Foundation

**Dates**: 2025-12-21 → 2026-02-28

**Completes**:
- ✅ **Phase A: HostBridge** (Week 1-4)
- ✅ **Phase B (start): VM Foundations** (Week 5-8)

**Deliverables**:
1. **HostBridge API** (C-ABI boundary):
   - `Hako_RunScriptUtf8`, `Retain/Release`, `ToUtf8`, `LastError`
   - Ubuntu/Windows ABI tests
   - Error handling (TLS)

2. **`op_eq` Migration** (Week 5-6):
   - Equality logic moved from Rust → Hakorune
   - `NoOperatorGuard` prevents recursion
   - Golden tests: Rust-VM vs Hako-VM parity

3. **VM Foundations PoC** (Week 7-8):
   - Instruction dispatch skeleton (5 instructions)
   - MIR execution loop in Hakorune
   - Integration test: Run simple programs

4. **Documentation** (Week 9-10):
   - HostBridge API spec
   - Op_eq migration guide
   - Pure Hakorune roadmap (this doc)
   - Phase 20.6 planning

**Success Criteria**:
- ✅ HostBridge API stable (Ubuntu/Windows)
- ✅ `op_eq` in Hakorune (NoOperatorGuard working)
- ✅ VM PoC runs 5 instructions (const, binop, compare, jump, ret)
- ⚠️ NOT: Full VM (deferred to Phase 20.6)
- ⚠️ NOT: Bootstrap compiler (deferred to Phase 20.8+)

---

### Phase 20.6 (12 weeks): VM Core Complete

**Dates**: 2026-03-01 → 2026-05-24

**Completes**:
- ✅ **Phase B (complete): VM Core in Hakorune**
- ✅ **Phase C: Dispatch Unification**

**Deliverables**:

#### Phase B Complete (Week 1-6):
1. **All 16 MIR Instructions** in Hakorune:
   - const, binop, compare, jump, branch, phi, ret ✅ (from 15.79)
   - call, boxcall, externcall
   - load, store, copy, typeop
   - barrier, safepoint, loopform

2. **Control Flow**:
   - Basic blocks
   - Branch/jump handling
   - PHI node resolution
   - Loop detection

3. **Golden Tests**:
   - 100+ test cases: Rust-VM vs Hako-VM parity
   - All outputs must match exactly
   - Performance: Hako-VM ≥ 50% of Rust-VM speed

#### Phase C: Dispatch Unification (Week 7-12):
1. **Resolver Integration**:
   - `Resolver.lookup(type_id, method, arity) -> MethodHandle`
   - All method calls go through Resolver
   - No special-case dispatch

2. **CallableBox Refactoring**:
   - `ExecBox.call_by_handle(handle, args, NoOperatorGuard)`
   - Single entry point for all invocations
   - Macro desugaring: `arr.methodRef("push",1)` → `Callable.ref_method(arr, :push, 1)`

3. **Universal Route Minimization**:
   - Remove pseudo-method implementations
   - Everything delegates to Resolver
   - Fail-Fast: Unknown methods → RuntimeError

**Success Criteria**:
- ✅ Hako-VM runs all 16 instructions
- ✅ Golden tests: 100% Rust-VM parity
- ✅ Dispatch unified (single Resolver path)
- ✅ Performance: Hako-VM ≥ 50% of Rust-VM
- ⚠️ NOT: Collections in Hakorune (deferred to Phase 20.7)

---

### Phase 20.7 (8 weeks): Collections in Hakorune

**Dates**: 2026-05-25 → 2026-07-19

**Completes**:
- ✅ **Phase D: Collections (MapBox/ArrayBox)**

**Deliverables**:

1. **MapBox in Hakorune**:
   - Hash map implementation
   - Deterministic hash/eq (decision mode)
   - Key normalization (Symbol/Int/String)
   - Methods: `set`, `get`, `has`, `remove`, `size`, `keys`, `values`

2. **ArrayBox in Hakorune**:
   - Dynamic array implementation
   - Methods: `push`, `pop`, `get`, `set`, `size`, `slice`, `concat`

3. **Key Comparison Order**:
   - Symbol < Int < String (stable sort)
   - Deterministic iteration order
   - Provenance tracking (plugin_id, version)

4. **ValueBox/DataBox Boundaries**:
   - ValueBox: Temporary (pipeline boundaries only)
   - DataBox: Persistent (long-lived)
   - Fail-Fast: Unpack at entry/exit points

**Success Criteria**:
- ✅ MapBox/ArrayBox fully in Hakorune
- ✅ Golden tests: Rust-Collections vs Hako-Collections parity
- ✅ Deterministic behavior (decision mode)
- ✅ Performance: ≥ 70% of Rust-Collections
- ⚠️ NOT: GC (still using Rust-side GC)

---

### Phase 20.8 (6 weeks): GC + Rust Deprecation

**Dates**: 2026-07-20 → 2026-08-30

**Completes**:
- ✅ **Phase E: GC v0 (Mark & Sweep)**
- ✅ **Phase F: Rust VM Compat Mode**

**Deliverables**:

#### Phase E: GC v0 (Week 1-4):
1. **Stop-the-world Mark & Sweep**:
   - Mark: Trace reachable objects from roots
   - Sweep: Free unreachable objects
   - Minimal implementation (no generational, no incremental)

2. **Roots**:
   - Stack frames (local variables)
   - Global static boxes
   - HandleRegistry (C-ABI handles)

3. **Metrics**:
   - Allocation count
   - Survivor count
   - Sweep time
   - Handle count

4. **Observability**:
   - `HAKO_GC_TRACE=1` → detailed GC log
   - GC stats at program exit

#### Phase F: Rust VM Compat Mode (Week 5-6):
1. **Deprecate Rust VM**:
   - Hakorune-VM is default (`--backend vm`)
   - Rust-VM becomes opt-in (`--backend vm-rust`)
   - Warning: "Rust-VM is deprecated, use Hakorune-VM"

2. **Bit-Identical Verification**:
   - Hako₁ → Hako₂ → Hako₃ (self-compilation)
   - Verify: Hako₁ == Hako₂ == Hako₃ (byte-by-byte)
   - CI runs daily verification

3. **Rust Layer Minimization**:
   - Rust code: ~100 lines (HostBridge only)
   - Everything else in Hakorune
   - "Rust=floor, Hakorune=house" ✅

**Success Criteria**:
- ✅ GC v0 works (no memory leaks)
- ✅ Hakorune-VM is default backend
- ✅ Rust-VM deprecated (compat mode)
- ✅ Bit-identical self-compilation
- ✅ Rust layer ≤ 100 lines

---

## 🔄 Dependencies Between Phases

```
Phase A (HostBridge)
    ↓
Phase B (VM Core)  ← Needs HostBridge for C-ABI
    ↓
Phase C (Dispatch) ← Needs VM Core for execution
    ↓
Phase D (Collections) ← Needs Dispatch for method calls
    ↓
Phase E (GC)       ← Needs Collections for memory management
    ↓
Phase F (Deprecate Rust) ← Needs GC for full autonomy
```

**Critical Path**: A → B → C → D → E → F (sequential)

**Parallelization Opportunities**:
- Phase C (Dispatch) can start during Phase B (VM Core) Week 5-6
- Phase D (Collections) design can happen during Phase C
- Phase E (GC) design can happen during Phase D

---

## ⚙️ Implementation Principles

### 1. Box-First ("箱理論")

**Every component is a Box**:
```hakorune
// HostBridge
box HostBridgeBox { /* C-ABI calls */ }

// VM
box MiniVmBox { /* Instruction dispatch */ }

// Collections
box MapBox { /* Hash map */ }
box ArrayBox { /* Dynamic array */ }

// GC
box GcBox { /* Mark & sweep */ }
```

### 2. Boundary = C-ABI ("境界")

**Single boundary**:
```
Rust (floor)                 Hakorune (house)
     |                              |
     |--- HostBridge (C-ABI) -------|
     |                              |
   ~100 lines               Everything else
```

### 3. Invocation = Handle ("呼び出し")

**Single invocation path**:
```hakorune
// All calls go through:
ExecBox.call_by_handle(handle, args, NoOperatorGuard)

// handle from:
Resolver.lookup(type_id, method, arity)
```

### 4. Resolution = Resolver ("解決")

**Single resolution path**:
```hakorune
// No special-case dispatch
Resolver.lookup(type_id, method, arity) -> MethodHandle

// Example:
local handle = Resolver.lookup(arr_type_id, :push, 1)
ExecBox.call_by_handle(handle, [arr, value], NoOperatorGuard)
```

### 5. Fail-Fast ("即失敗")

**No silent fallbacks**:
```hakorune
// ❌ BAD: Silent fallback
if (lookup_fails) { return default_value }

// ✅ GOOD: Explicit error
if (lookup_fails) { panic("Method not found: " + method) }
```

---

## 🧪 Golden Testing Strategy

### Golden Test = Rust-VM vs Hako-VM Parity

**Goal**: Prove Hakorune-VM produces identical output to Rust-VM

**Test Suite**:
```
tests/golden/
├── arithmetic.hako          # Basic arithmetic
├── control_flow.hako        # if/loop/branch
├── collections.hako         # Array/Map operations
├── recursion.hako           # Recursive functions
├── strings.hako             # String manipulation
├── enums.hako               # @enum types
├── closures.hako            # Closure capture
└── selfhost_mini.hako       # Mini compiler
```

**Verification**:
```bash
# Run same program on both VMs
./hako --backend vm-rust test.hako > rust_output.txt
./hako --backend vm test.hako > hako_output.txt

# Compare outputs
diff rust_output.txt hako_output.txt
# Expected: No differences
```

**CI Integration**:
```yaml
# .github/workflows/golden_tests.yml
name: Golden Tests
on: [push, pull_request]
jobs:
  golden:
    runs-on: ubuntu-latest
    steps:
      - name: Run golden tests
        run: |
          for test in tests/golden/*.hako; do
            ./hako --backend vm-rust "$test" > rust.txt
            ./hako --backend vm "$test" > hako.txt
            diff rust.txt hako.txt || exit 1
          done
```

---

## 🎯 Risk Mitigation

### Risk 1: Longer Timeline (36 weeks)

**Mitigation**:
- Each phase delivers independent value
- Can pause after Phase 20.5 (HostBridge + op_eq + VM PoC)
- Can pause after Phase 20.6 (Full VM Core)
- Progressive derisking

### Risk 2: Implementation Complexity

**Mitigation**:
- Rust VM as reference implementation
- Golden tests catch bugs early
- Incremental approach (5 instructions → 16 instructions)
- ChatGPT/Claude collaboration

### Risk 3: Performance

**Mitigation**:
- Measure at each phase
- Accept slower performance initially (50% of Rust-VM is OK)
- Optimize hot paths after correctness is proven
- Profile-guided optimization in later phases

### Risk 4: Maintenance Burden (Dual VMs)

**Mitigation**:
- Freeze Rust VM after Phase 20.5 (no new features)
- All new work happens in Hakorune-VM
- Clear deprecation timeline (Phase 20.8)
- Golden tests prevent divergence

---

## 📊 Progress Tracking

### Phase 20.5 (Current)

**Week 1-4**: HostBridge API
- [ ] C-ABI design finalized
- [ ] Rust implementation (5 functions)
- [ ] Ubuntu ABI tests PASS
- [ ] Windows ABI tests PASS

**Week 5-6**: Op_eq Migration
- [ ] Equality logic in Hakorune
- [ ] NoOperatorGuard implementation
- [ ] Golden tests: Rust vs Hako equality

**Week 7-8**: VM Foundations PoC
- [ ] 5 instructions implemented (const, binop, compare, jump, ret)
- [ ] MIR execution loop
- [ ] Integration test: Run simple programs

**Week 9-10**: Documentation
- [ ] HostBridge API spec
- [ ] Op_eq migration guide
- [ ] Pure Hakorune roadmap
- [ ] Phase 20.6 planning

### Phase 20.6 (Future)

**Week 1-6**: VM Core Complete
- [ ] 16 instructions implemented
- [ ] Control flow (branch, phi, loop)
- [ ] Golden tests: 100% parity

**Week 7-12**: Dispatch Unification
- [ ] Resolver integration
- [ ] CallableBox refactoring
- [ ] Universal route minimization

### Phase 20.7 (Future)

**Week 1-4**: MapBox in Hakorune
**Week 5-8**: ArrayBox in Hakorune

### Phase 20.8 (Future)

**Week 1-4**: GC v0
**Week 5-6**: Rust VM deprecation

---

## 🎉 Success Impact

### After Phase 20.5 (10 weeks)

1. **HostBridge API**: Clean C-ABI boundary
2. **Op_eq in Hakorune**: Improved correctness (NoOperatorGuard)
3. **VM PoC**: Feasibility demonstrated

### After Phase 20.6 (22 weeks total)

4. **VM Core Complete**: Hakorune can run itself
5. **Dispatch Unified**: Single resolution path (Resolver)
6. **Performance**: ≥ 50% of Rust-VM

### After Phase 20.7 (30 weeks total)

7. **Collections in Hakorune**: MapBox/ArrayBox
8. **Deterministic**: Stable iteration order
9. **Everything is Box**: True realization

### After Phase 20.8 (36 weeks total)

10. **GC v0**: No memory leaks
11. **Rust Minimal**: ≤ 100 lines (HostBridge only)
12. **True Self-Hosting**: Hakorune IS Hakorune ✅

---

## 📚 Related Documents

- [STRATEGY_RECONCILIATION.md](STRATEGY_RECONCILIATION.md) - Why Pure Hakorune?
- [HOSTBRIDGE_API_DESIGN.md](HOSTBRIDGE_API_DESIGN.md) - C-ABI spec
- [OP_EQ_MIGRATION.md](OP_EQ_MIGRATION.md) - Equality implementation
- [CHATGPT_PURE_HAKORUNE_STRATEGY.md](CHATGPT_PURE_HAKORUNE_STRATEGY.md) - Original proposal

---

**Status**: APPROVED
**Timeline**: 2025-12-21 → 2026-09-30 (36 weeks)
**Current Phase**: 15.79 (Week 1-10)
**Next Phase**: 15.80 (Week 11-22)
