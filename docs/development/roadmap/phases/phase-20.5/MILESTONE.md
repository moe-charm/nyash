# Phase 20.5 — Milestone (Escape from Rust / Pure Hakorune Strategy)

Status: Planning (Pure Hakorune Strategy Adopted; v1.1‑frozen reminted)
Start: 2025-12-21
End: 2026-02-28 (10 weeks)
Strategy Change: 2025-10-14 (C Code Generator → Pure Hakorune VM)
Dependency: Phase 15.77/15.78 complete; Frozen v1.1 (Linux/MSVC/MinGW) available

---

## 🎯 Objectives

**"Rust=floor, Hakorune=house" - Implement VM itself in Hakorune**

1. **HostBridge API Complete**: Minimal C-ABI boundary (Rust↔Hakorune)
2. **op_eq Migration**: Move equality logic from Rust to Hakorune (NoOperatorGuard)
3. **VM Foundations PoC**: Instruction dispatch proof-of-concept (5 instructions)
4. **Pure Hakorune Roadmap**: Detailed plan for Phase 20.6-15.82

**Note**: C Code Generator implementation is **CANCELLED** (Pure Hakorune strategy adopted)

---

## ✅ Deliverables (DoD)

### 1️⃣ HostBridge API Complete

**C-ABI Functions Working**:
```bash
./test_hostbridge_abi        # Ubuntu
./test_hostbridge_abi.exe    # Windows
```

**5 Core Functions**:
- `Hako_RunScriptUtf8` - Execute Hakorune script
- `Hako_Retain / Hako_Release` - Handle reference counting
- `Hako_ToUtf8` - Get string view
- `Hako_LastError` - Get error message (TLS)

**Checklist**:
- [ ] 5 core functions implemented
- [ ] Ubuntu ABI tests PASS
- [ ] Windows ABI tests PASS
- [ ] Error handling (TLS) working

---

### 2️⃣ op_eq Migration Complete

**Hakorune-side op_eq Working**:
```bash
HAKO_USE_PURE_EQ=1 ./hako test.hako

# Golden Test: Rust vs Hako
./test_op_eq_golden.sh  # Expected: 100% parity
```

**Checklist**:
- [ ] NoOperatorGuard implemented (prevents infinite recursion)
- [ ] 8 comparison types (int/bool/null/string/array/map/enum/user)
- [ ] Golden tests: 20/20 PASS
- [ ] Performance: Hako-VM ≥ 70% of Rust-VM

---

### 3️⃣ VM Foundations PoC

**5 Instructions Working**:
```bash
./hako --backend vm-hako simple_program.hako
# Instructions: const, binop, compare, jump, ret
```

**Checklist**:
- [ ] 5 instructions implemented
- [ ] MIR execution loop (Hakorune implementation)
- [ ] Integration test: Run simple programs
- [ ] Performance measured

---

### 4️⃣ Documentation Complete

**Checklist**:
- [ ] STRATEGY_RECONCILIATION.md (why Pure Hakorune?)
- [ ] HOSTBRIDGE_API_DESIGN.md (C-ABI spec)
- [ ] OP_EQ_MIGRATION.md (equality implementation guide)
- [ ] PURE_HAKORUNE_ROADMAP.md (Phase 20.5-15.82 plan)
- [ ] Phase 20.6 planning document

---

## 📊 Weekly Milestones

### Week 1-2: HostBridge API Design & Implementation

**Goal**: Establish C-ABI boundary

**Tasks**:
- [ ] HostBridge API design (5 core functions)
- [ ] HandleRegistry implementation (Rust side)
- [ ] C header generation (hakorune_hostbridge.h)
- [ ] Basic implementation (RunScriptUtf8, Retain/Release)

**Deliverables**:
```
src/hostbridge/
├── mod.rs                 # C-ABI exports
├── handle_registry.rs     # Handle management
└── tests.rs               # Unit tests

include/
└── hakorune_hostbridge.h  # C header
```

---

### Week 3-4: HostBridge API Complete & Testing

**Goal**: Ubuntu/Windows ABI tests PASS

**Tasks**:
- [ ] ToUtf8, LastError implementation
- [ ] Error handling (TLS)
- [ ] ABI test creation (C language)
- [ ] Ubuntu/Windows verification

**Deliverables**:
```
tests/
├── hostbridge_abi_test.c  # C ABI tests
└── hostbridge_integration/
    └── test_hostbridge_call.hako

tools/
└── test_hostbridge_abi.sh
```

---

### Week 5-6: op_eq Migration

**Goal**: Move equality logic from Rust to Hakorune

**Tasks**:
- [ ] NoOperatorGuard implementation (recursion prevention)
- [ ] 8 comparison types (int/bool/null/string/array/map/enum/user)
- [ ] Golden test creation (20 cases)
- [ ] Rust-VM vs Hako-VM parity verification

**Deliverables**:
```
apps/hakorune-vm/
├── op_eq_box.hako
├── no_operator_guard_box.hako
└── tests/

tests/golden/op_eq/
├── primitives.hako
├── arrays.hako
├── maps.hako
├── recursion.hako
└── user_defined.hako
```

---

### Week 7-8: VM Foundations PoC

**Goal**: Instruction dispatch proof-of-concept (5 instructions)

**Tasks**:
- [ ] MIR execution loop (Hakorune implementation)
- [ ] 5 instructions (const, binop, compare, jump, ret)
- [ ] Integration test (run simple programs)
- [ ] Performance measurement

**Deliverables**:
```
apps/hakorune-vm/
├── mini_vm_box.hako
├── instruction_dispatch.hako
└── tests/

tests/vm_poc/
├── hello.hako
├── arithmetic.hako
└── control_flow.hako
```

---

### Week 9: Integration Testing & Performance

**Goal**: Integrated operation verification

**Tasks**:
- [ ] HostBridge + op_eq + VM PoC integration
- [ ] E2E tests (10 cases)
- [ ] Performance measurement & comparison
- [ ] Issue identification & fixes

**Verification Script**:
```bash
#!/bin/bash
set -e

echo "Test 1: HostBridge API"
./test_hostbridge_abi

echo "Test 2: op_eq Golden Tests"
./test_op_eq_golden.sh

echo "Test 3: VM PoC"
./hako --backend vm-hako tests/vm_poc/arithmetic.hako

echo "✅ PASS: Phase 20.5 Integration Tests"
```

---

### Week 10: Documentation & Phase 20.6 Planning

**Goal**: Documentation & next phase planning

**Tasks**:
- [ ] STRATEGY_RECONCILIATION.md
- [ ] HOSTBRIDGE_API_DESIGN.md
- [ ] OP_EQ_MIGRATION.md
- [ ] PURE_HAKORUNE_ROADMAP.md
- [ ] Phase 20.6 planning document
- [ ] Completion report

**Deliverables**:
```
docs/development/roadmap/phases/phase-20.5/
├── STRATEGY_RECONCILIATION.md
├── HOSTBRIDGE_API_DESIGN.md
├── OP_EQ_MIGRATION.md
├── PURE_HAKORUNE_ROADMAP.md
├── COMPLETION_REPORT.md
└── LESSONS_LEARNED.md

docs/development/roadmap/phases/phase-20.6/
└── README.md
```

---

## 🎯 Success Criteria

### Technical

1. **HostBridge API Works**:
   - C-ABI functions work on Ubuntu + Windows
   - Handle lifecycle correct (no leaks)
   - Error handling via TLS

2. **op_eq Migration Works**:
   - NoOperatorGuard prevents infinite recursion
   - Golden tests: 100% Rust-VM parity
   - Performance: ≥ 70% of Rust-VM

3. **VM PoC Works**:
   - 5 instructions execute correctly
   - Simple programs run end-to-end
   - Performance measurable

### Process

1. **Documentation Complete**:
   - User can understand Pure Hakorune strategy
   - Technical specs enable Phase 20.6 implementation
   - Lessons learned documented

2. **Testing Comprehensive**:
   - ABI tests (Ubuntu/Windows)
   - Golden tests (Rust-VM vs Hako-VM)
   - Integration tests (E2E)

3. **Review Approved**:
   - ChatGPT review complete
   - Claude review complete
   - No blocking issues

---

## 🚨 Risks & Mitigations

### Risk 1: Longer Timeline (36 weeks total)

**Impact**: HIGH (delayed complete self-hosting)
**Mitigation**:
- Phase 20.5 delivers independent value
- Can pause after each phase
- Progressive derisking

### Risk 2: Implementation Complexity

**Impact**: MEDIUM (more effort than C Generator)
**Mitigation**:
- Rust VM as reference implementation
- Golden tests catch bugs early
- Incremental approach (5→16 instructions)

### Risk 3: Dual VM Maintenance

**Impact**: MEDIUM (maintenance burden)
**Mitigation**:
- Freeze Rust VM after Phase 20.5
- All new work in Hakorune VM
- Clear deprecation timeline (Phase 20.8)

### Risk 4: C-ABI Stability

**Impact**: LOW (well-understood boundary)
**Mitigation**:
- Minimal API (5 functions)
- Proven design (Lua/Python C API)
- Comprehensive ABI tests

---

## 🎉 Success Impact

### After Phase 20.5 (10 weeks)

1. **HostBridge API**: Clean Rust↔Hakorune boundary
2. **op_eq in Hakorune**: Improved correctness (NoOperatorGuard)
3. **VM PoC**: Feasibility demonstrated
4. **Phase 20.6 Plan**: Detailed roadmap ready

### After Phase 20.8 (36 weeks, 2026-09-30)

5. **Complete Self-Hosting**: Hakorune IS Hakorune
6. **Rust Minimized**: ~100 lines (C-ABI bridge only)
7. **Ultimate Box Theory**: VM also implemented as Box
8. **Long-term Maintainability**: Single execution path

---

## 📚 Related Resources

### Phase 20.5 Documents
- [STRATEGY_RECONCILIATION.md](STRATEGY_RECONCILIATION.md) - Why Pure Hakorune?
- [HOSTBRIDGE_API_DESIGN.md](HOSTBRIDGE_API_DESIGN.md) - C-ABI specification
- [OP_EQ_MIGRATION.md](OP_EQ_MIGRATION.md) - Equality implementation guide
- [PURE_HAKORUNE_ROADMAP.md](PURE_HAKORUNE_ROADMAP.md) - Overall plan
- [CHATGPT_PURE_HAKORUNE_STRATEGY.md](CHATGPT_PURE_HAKORUNE_STRATEGY.md) - Original proposal

### Previous Phases
- [Phase 15.77 - Frozen EXE Finalization](../phase-15.77/)
- [Phase 15.76 - extern_c & Frozen Toolchain](../phase-15.76/)

### Industry Patterns
- **Lua**: Minimal C-API (5-10 functions)
- **Python**: C-API + Pure Python stdlib
- **Rust**: stage0 (frozen) → stage1 (bootstrap) → stage2 (verify)
- **Go**: Go 1.4 frozen → Go 1.5 self-hosted

---

**Created**: 2025-10-14
**Strategy Changed**: 2025-10-14 (Pure Hakorune adopted)
**Phase Start**: 2025-12-21 (after Phase 15.77 completion)
**Duration**: 10 weeks (Phase 20.5 only)
**Complete Realization**: 36 weeks (Phase 20.5-15.82)
**Strategy**: Pure Hakorune ("Rust=floor, Hakorune=house")
