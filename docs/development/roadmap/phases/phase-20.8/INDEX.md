# Phase 20.8: GC + Rust Deprecation - Document Index

**Phase**: 20.8
**Duration**: 6 weeks (2026-07-20 → 2026-08-30)
**Status**: Not Started

---

## 📚 Document Structure

### Core Documents

1. **[README.md](README.md)** (日本語)
   - Phase 20.8 概要
   - Phase E: GC v0 (Week 1-4)
   - Phase F: Rust VM Deprecation (Week 5-6)
   - 成功基準
   - マイルストーン

2. **[PLAN.md](PLAN.md)** (English)
   - Executive summary
   - Week-by-week implementation plan
   - GC implementation details
   - Rust VM deprecation strategy
   - Bit-identical verification
   - Success criteria

3. **[CHECKLIST.md](CHECKLIST.md)** (タスクリスト)
   - Week 1-4: GC v0 implementation
   - Week 5-6: Rust VM deprecation
   - Self-compilation verification
   - Rust layer audit

4. **[INDEX.md](INDEX.md)** (このファイル)
   - Document structure
   - Quick reference
   - Related phases

---

## 🎯 Quick Reference

### Phase E: GC v0 (Week 1-4)

**Goal**: Implement Mark & Sweep garbage collection in Hakorune

**Key Features**:
- Stop-the-world GC
- Mark phase: Trace reachable objects from roots
- Sweep phase: Free unreachable objects
- GC Roots: Stack frames, global static boxes, HandleRegistry
- Metrics: Allocation count, survivor count, sweep time
- Observability: `HAKO_GC_TRACE=1`

**Deliverables**:
- GcBox implementation (Mark & Sweep)
- GcMetricsBox (metrics collection)
- VM integration (allocation hook, GC trigger)
- Tests: Golden tests, smoke tests

### Phase F: Rust VM Deprecation (Week 5-6)

**Goal**: Deprecate Rust VM and achieve true self-hosting

**Key Features**:
- Hakorune-VM is default (`--backend vm`)
- Rust-VM opt-in (`--backend vm-rust`, with warning)
- Bit-identical self-compilation (Hako₁ → Hako₂ → Hako₃)
- Rust layer ≤ 100 lines (HostBridge API only)

**Deliverables**:
- CLI updated (Hakorune-VM default)
- Deprecation warning added
- Self-compilation verification script
- CI integration (daily verification)
- Rust layer audit (≤ 100 lines)

---

## ✅ Success Criteria Summary

### Phase E (GC v0)

- [ ] GC v0 works (no memory leaks)
- [ ] Mark & Sweep correct
- [ ] GC roots detected (stack, global, handles)
- [ ] Metrics collection working
- [ ] `HAKO_GC_TRACE=1` provides detailed logs

### Phase F (Rust VM Deprecation)

- [ ] Hakorune-VM is default backend
- [ ] Rust-VM deprecated (warning displayed)
- [ ] Bit-identical self-compilation verified
- [ ] CI daily verification passes
- [ ] Rust layer ≤ 100 lines

### Overall

- [ ] **True Self-Hosting**: Hakorune IS Hakorune
- [ ] **Rust=floor, Hakorune=house**: Architecture realized
- [ ] All smoke tests pass with Hakorune-VM

---

## 🔗 Related Phases

### Prerequisites

- **Phase 20.7 (Collections)**: [../phase-20.7/README.md](../phase-20.7/README.md)
  - MapBox in Hakorune
  - ArrayBox in Hakorune
  - Deterministic behavior

- **Phase 15.80 (VM Core)**: [../phase-15.80/README.md](../phase-15.80/README.md)
  - 16 MIR instructions
  - Control flow
  - Dispatch unification

- **Phase 20.5 (HostBridge)**: [../phase-20.5/README.md](../phase-20.5/README.md)
  - HostBridge API (C-ABI)
  - Op_eq migration
  - VM foundations PoC

### Follow-up Phases

- **Phase 15.82 (Advanced GC & Optimization)**: TBD
  - Generational GC
  - Incremental GC
  - Performance optimization
  - JIT compilation

---

## 📖 Additional Resources

### Pure Hakorune Roadmap

- **[PURE_HAKORUNE_ROADMAP.md](../phase-20.5/PURE_HAKORUNE_ROADMAP.md)**
  - 36-week timeline (2025-12-21 → 2026-09-30)
  - 6 phases: A → B → C → D → E → F
  - "Rust=floor, Hakorune=house" vision

### Technical Specifications

- **[HOSTBRIDGE_API_DESIGN.md](../phase-20.5/HOSTBRIDGE_API_DESIGN.md)**
  - C-ABI boundary
  - HostBridge API spec
  - Error handling (TLS)

- **[OP_EQ_MIGRATION.md](../phase-20.5/OP_EQ_MIGRATION.md)**
  - Equality logic migration
  - NoOperatorGuard pattern
  - Golden tests

### Architecture Documents

- **[Box/ExternCall Design](../../../../architecture/box-externcall-design.md)**
  - Everything is Box
  - BoxCall vs ExternCall
  - Unified invocation

- **[MIR Instruction Set](../../../../../reference/mir/INSTRUCTION_SET.md)**
  - 16 frozen instructions
  - MIR semantics
  - VM execution model

---

## 🧪 Testing Strategy

### Golden Tests

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

### Self-Compilation Verification

**Goal**: Prove Hakorune compiler is bit-identical across generations

**Verification Chain**:
```bash
# Hako₁: Rust-based compiler
cargo build --release
cp target/release/hako hako_1

# Hako₂: Compiled by Hako₁
./hako_1 apps/selfhost-compiler/main.hako -o hako_2

# Hako₃: Compiled by Hako₂
./hako_2 apps/selfhost-compiler/main.hako -o hako_3

# Verify bit-identical
diff hako_2 hako_3  # Expected: No differences
```

### GC Tests

**Goal**: Verify GC correctness and performance

**Test Categories**:
1. **Leak Tests**: Ensure no memory leaks
2. **Stress Tests**: Allocate/free large number of objects
3. **Cycle Tests**: Verify cyclic references are collected
4. **Root Tests**: Verify all roots are traced
5. **Metrics Tests**: Verify metrics accuracy

**Tools**:
- `HAKO_GC_TRACE=1`: Detailed GC logs
- Valgrind: Memory leak detection
- Custom GC metrics

---

## 📊 Milestones

### Week 1-2: GC Mark Phase

- [ ] GC roots detection
- [ ] Mark algorithm
- [ ] Cycle detection
- [ ] Tests: Mark correctness

### Week 3-4: GC Sweep Phase & Metrics

- [ ] Sweep algorithm
- [ ] Metrics collection
- [ ] `HAKO_GC_TRACE=1`
- [ ] Integration & testing

### Week 5: Backend Switching & Deprecation

- [ ] Hakorune-VM default
- [ ] Rust-VM deprecation warning
- [ ] Golden tests verification
- [ ] Documentation updates

### Week 6: Bit-Identical Verification & Audit

- [ ] Self-compilation chain
- [ ] CI integration
- [ ] Rust layer audit (≤ 100 lines)
- [ ] Final documentation

---

## 🎉 Final Success Impact

### Before Phase 20.8

```
Rust Layer:
├── VM implementation (~5000 lines)
├── Collections (MapBox, ArrayBox) (~2000 lines)
├── GC (reference counting) (~500 lines)
└── HostBridge API (~100 lines)

Hakorune Layer:
├── Parser (selfhost-compiler)
└── Standard library
```

### After Phase 20.8

```
Rust Layer (~100 lines):
└── HostBridge API (C-ABI only)

Hakorune Layer (everything else):
├── VM (MiniVmBox)
├── Parser (MirJsonBuilderBox)
├── Collections (MapBox, ArrayBox)
├── GC (GcBox)
└── Standard library (StringBox, IntegerBox, etc.)
```

**Achievement**: "Rust=floor, Hakorune=house" ✅

**True Self-Hosting**: Hakorune IS Hakorune

---

## 🚀 Next Steps

After Phase 20.8 completion:

1. **Phase 15.82 (Advanced GC)**:
   - Generational GC
   - Incremental GC
   - Write barriers

2. **Performance Optimization**:
   - JIT compilation
   - Inline caching
   - Type specialization

3. **Language Features**:
   - Async/await
   - Generators
   - Pattern matching

4. **Tooling**:
   - Debugger
   - Profiler
   - Language server

---

**Status**: Not Started
**Start Date**: 2026-07-20
**Target Completion**: 2026-08-30
**Dependencies**: Phase 20.7 complete

**Last Updated**: 2025-10-14
