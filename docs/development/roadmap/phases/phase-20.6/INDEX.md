# Phase 20.6 — Index

**VM Core Complete + Dispatch Unification**

Status: Planning
Duration: 12 weeks (2026-03-01 - 2026-05-24)
Prerequisite: Phase 20.5 complete

---

## 📚 Documentation Structure

### Core Documents

1. **[README.md](README.md)** ⭐ START HERE (日本語)
   - Phase overview and goals
   - Architecture overview (16 instructions + Dispatch)
   - Weekly breakdown (Week 1-12)
   - Success criteria and DoD
   - Risk mitigation

2. **[PLAN.md](PLAN.md)** (English)
   - Executive summary (5 lines)
   - Phase breakdown (Phase B + Phase C)
   - Week-by-week implementation plan
   - Golden Test strategy
   - CI integration

3. **[CHECKLIST.md](CHECKLIST.md)**
   - Week 1-6: VM Core Complete tasks
   - Week 7-12: Dispatch Unification tasks
   - Golden Test checklist (100+ tests)
   - Documentation checklist

4. **[COMPLETION_REPORT.md](COMPLETION_REPORT.md)** (Week 12 deliverable)
   - Phase 20.6 results
   - Performance measurements
   - Lessons learned
   - Phase 20.7 recommendations

---

## 🎯 Quick Reference

### What is Phase 20.6? (5-line summary)

1. **Goal**: Complete Hakorune VM Core (16 MIR instructions) + Unify Dispatch (single Resolver path).
2. **Strategy**: Phase B (Week 1-6) = All instructions; Phase C (Week 7-12) = Resolver integration.
3. **Verification**: Golden Tests (100+ cases) ensure Rust-VM vs Hako-VM 100% output parity.
4. **Performance**: Target ≥ 50% of Rust-VM speed (acceptable for PoC, optimize later).
5. **Policy**: Fail-Fast (unknown methods → RuntimeError), no special-case dispatch, Everything is Box.

### Key Deliverables

**Phase B Complete (Week 1-6)**:
1. **All 16 MIR Instructions** in Hakorune:
   - const, binop, compare, jump, branch, phi, ret ✅ (from 20.5)
   - call, boxcall, externcall ⬜ (Week 3-4)
   - load, store, copy, typeop ⬜ (Week 1-2, 5)
   - barrier, safepoint, loopform, unaryop ⬜ (Week 6)

2. **Control Flow**:
   - Basic blocks
   - Branch/jump handling
   - PHI node resolution
   - Loop detection

3. **Golden Tests**:
   - 100+ test cases: Rust-VM vs Hako-VM parity
   - All outputs must match exactly
   - Performance: Hako-VM ≥ 50% of Rust-VM speed

**Phase C: Dispatch Unification (Week 7-12)**:
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

---

## 📊 Timeline at a Glance

| Weeks | Focus | Deliverable |
|-------|-------|-------------|
| 1-2 | Memory Operations | load, store, copy working |
| 3-4 | Method Calls | call, boxcall, externcall working |
| 5 | Type Operations | typeop, newbox working |
| 6 | Control + Golden Tests | All 16 instructions + 100 tests PASS |
| 7-8 | Resolver Integration | Resolver.lookup working |
| 9-10 | CallableBox Refactoring | ExecBox.call_by_handle working |
| 11 | Route Minimization | Special-case dispatch removed |
| 12 | Integration + Docs | Phase complete, Phase 20.7 planned |

---

## 🏗️ Architecture Overview

### Phase B: 16 MIR Instructions

```
┌─────────────────────────────────────┐
│ Phase 20.5 Complete (5 instructions)│
│ ✅ const, ret, jump, branch, phi    │
│ ✅ binop (Add/Sub/Mul/Div/Mod)      │
│ ✅ compare (Eq/Ne/Lt/Le/Gt/Ge)      │
├─────────────────────────────────────┤
│ Phase 20.6 Additions (11 instructions)
│ ⬜ load, store, copy (Week 1-2)     │
│ ⬜ call, boxcall, externcall (Week 3-4)
│ ⬜ typeop, newbox (Week 5)          │
│ ⬜ barrier, safepoint, loopform,    │
│    unaryop (Week 6)                 │
└─────────────────────────────────────┘
```

### Phase C: Dispatch Unification

```
Before (multiple dispatch paths):
┌────────────────────────────────────┐
│ Method Call Dispatch               │
│  ├─ Global Function (special)      │
│  ├─ Box Method (special)           │
│  ├─ Closure (special)              │
│  ├─ Constructor (special)          │
│  └─ Module Function (special)      │
└────────────────────────────────────┘

After (single dispatch path):
┌────────────────────────────────────┐
│ Unified Resolver Path              │
│                                    │
│  Resolver.lookup(type_id, method, arity)
│           ↓                        │
│      MethodHandle                  │
│           ↓                        │
│  ExecBox.call_by_handle(           │
│      handle, args, NoOperatorGuard)│
└────────────────────────────────────┘
```

---

## 🧪 Test Strategy Overview

### Golden Test Pyramid

```
         /\
        /  \    Integration (30 tests)
       /____\   Complex scenarios
      /      \
     /        \ Functional (70 tests)
    /__________\ Individual instruction tests
   /            \
  /              \ Unit (per-handler)
 /________________\ Each handler has dedicated tests
```

### Test Categories

1. **Memory Operations** (10 tests):
   - load, store, copy
   - Memory aliasing
   - Edge cases

2. **Method Calls** (15 tests):
   - call (Global/Module/Closure)
   - boxcall
   - externcall

3. **Type Operations** (10 tests):
   - typeop
   - newbox

4. **Control Flow** (15 tests):
   - barrier, safepoint, loopform
   - Branch/jump combinations

5. **Resolver** (20 tests):
   - lookup success/failure
   - Arity mismatches
   - Type not found

6. **Callable** (25 tests):
   - call_by_handle
   - NoOperatorGuard
   - Macro desugaring

7. **Integration** (30 tests):
   - Arithmetic
   - Control flow
   - Collections
   - Recursion
   - Closures
   - Strings

**Total**: 100+ Golden Tests

---

## 📦 Implementation Structure

```
Phase 20.6 Planning:
docs/development/roadmap/phases/phase-20.6/
├── INDEX.md                          # ← You are here
├── README.md                         # Phase overview (日本語)
├── PLAN.md                           # Execution plan (English)
├── CHECKLIST.md                      # Week-by-week checklist
└── COMPLETION_REPORT.md              # (Week 12 deliverable)

Hakorune VM Implementation:
selfhost/hakorune-vm/
├── handlers/                         # Week 1-6 deliverables
│   ├── load_handler.hako             # Week 1-2
│   ├── store_handler.hako            # Week 1-2
│   ├── copy_handler.hako             # Week 1-2
│   ├── call_handler.hako             # Week 3-4 (Unified MirCall)
│   ├── boxcall_handler.hako          # Week 3-4
│   ├── externcall_handler.hako       # Week 3-4
│   ├── typeop_handler.hako           # Week 5
│   ├── newbox_handler.hako           # Week 5
│   ├── barrier_handler.hako          # Week 6
│   ├── safepoint_handler.hako        # Week 6
│   ├── loopform_handler.hako         # Week 6
│   └── unaryop_handler.hako          # Week 6
├── resolver/                         # Week 7-8 deliverables
│   ├── resolver_box.hako
│   ├── method_handle_box.hako
│   └── type_registry_box.hako
├── callable/                         # Week 9-10 deliverables
│   ├── exec_box.hako
│   ├── no_operator_guard_box.hako
│   └── macro_desugar.hako
└── tests/                            # Week 1-12
    ├── golden/                       # 100+ tests
    │   ├── phase-b/                  # Week 1-6
    │   │   ├── memory/               # 10 tests
    │   │   ├── calls/                # 15 tests
    │   │   ├── types/                # 10 tests
    │   │   └── control/              # 15 tests
    │   ├── phase-c/                  # Week 7-12
    │   │   ├── resolver/             # 20 tests
    │   │   ├── callable/             # 25 tests
    │   │   └── unification/          # 30 tests
    │   └── integration/              # Week 12
    │       ├── arithmetic.hako
    │       ├── control_flow.hako
    │       ├── collections.hako
    │       ├── recursion.hako
    │       ├── strings.hako
    │       └── closures.hako
    └── benchmark/                    # Performance tests
        ├── benchmark_suite.sh
        └── performance_report.md
```

---

## 🚀 Quick Start (After Phase Complete)

### Run Golden Tests

```bash
# Run all Golden Tests
bash tools/golden_test_hakorune_vm.sh

# Expected output:
# ✅ phase-b/memory: 10/10 PASS
# ✅ phase-b/calls: 15/15 PASS
# ✅ phase-b/types: 10/10 PASS
# ✅ phase-b/control: 15/15 PASS
# ✅ phase-c/resolver: 20/20 PASS
# ✅ phase-c/callable: 25/25 PASS
# ✅ phase-c/unification: 30/30 PASS
# ✅ integration: 30/30 PASS
# ────────────────────────────────────
# Total: 155/155 PASS (100%)
```

### Run Performance Benchmark

```bash
# Benchmark Hako-VM vs Rust-VM
bash tools/benchmark_hakorune_vm.sh

# Expected output:
# Test: arithmetic.hako
#   Rust-VM:  1.2s
#   Hako-VM:  2.1s (57% speed) ✅
#
# Test: control_flow.hako
#   Rust-VM:  0.8s
#   Hako-VM:  1.5s (53% speed) ✅
#
# Average: Hako-VM = 55% of Rust-VM ✅
# Memory: 180MB (< 200MB target) ✅
```

### Use Unified Dispatch

```bash
# Run program with unified dispatch
./target/release/hako --backend vm program.hako

# Debug Resolver
HAKO_RESOLVER_TRACE=1 ./target/release/hako --backend vm program.hako

# Output:
# [resolver] lookup type_id=42 method="push" arity=1 → handle=0x123
# [exec] call_by_handle handle=0x123 args=[value] → OK
```

---

## ⚠️ Prerequisites

### From Phase 20.5

- [x] HostBridge API complete
- [x] op_eq Migration complete (NoOperatorGuard)
- [x] VM Foundations PoC complete (5 instructions)
- [x] Golden Test infrastructure ready

### For Phase 20.6

- [ ] Week 1: Memory operation design approved
- [ ] Week 3: Method call design approved (Unified MirCall)
- [ ] Week 7: Resolver design approved
- [ ] Week 9: CallableBox design approved

---

## 🎯 Success Criteria Summary

### Technical

- [ ] All 16 MIR instructions working
- [ ] Control flow complete (branch, phi, loopform)
- [ ] Golden Tests: 100+ cases ALL PASS
- [ ] Dispatch unified (single Resolver path)
- [ ] Special-case dispatch removed

### Performance

- [ ] Hako-VM ≥ 50% of Rust-VM speed ✅
- [ ] Memory usage < 200MB ✅
- [ ] Compile time < 10s (small programs) ✅

### Quality

- [ ] Test coverage: All instructions covered
- [ ] Documentation: Complete architecture docs
- [ ] Code review: ChatGPT + Claude approved
- [ ] CI: All tests passing

---

## 📚 Related Phases

### Previous

- [Phase 20.5 - VM Foundations](../phase-20.5/)
  - HostBridge API
  - op_eq Migration
  - VM Foundations PoC (5 instructions)

### Next

- **Phase 20.7 - Collections in Hakorune** (8 weeks, 2026-05-25 → 2026-07-19)
  - MapBox in Hakorune
  - ArrayBox in Hakorune
  - Deterministic iteration order
  - Performance: ≥ 70% of Rust-Collections

### Parallel

- [Phase 15.78 - Frozen UX Polish](../phase-15.78/)
  - Distribution packaging
  - Doctor improvements

---

## 💬 Communication

### Weekly Sync Points

- **Monday**: Week start, goal setting
- **Wednesday**: Mid-week progress check
- **Friday**: Week review, performance measurement

### Issue Tracking

- Use GitHub issues with label `phase-20.6`
- Prefix: `[20.6]` in commit messages
- Milestone: `Phase 20.6 - VM Core Complete`

### Review Process

- Each week: Self-review + Golden Tests
- Week 6, 12: Full review with ChatGPT/Claude
- Blocking issues: Immediate escalation

---

## 🔗 External Resources

### Reference Implementations

- **Rust VM**: `/src/backend/mir_interpreter/` (reference implementation)
- **MIR Spec**: [INSTRUCTION_SET.md](../../../../reference/mir/INSTRUCTION_SET.md)
- **Phase 20.5 Roadmap**: [PURE_HAKORUNE_ROADMAP.md](../phase-20.5/PURE_HAKORUNE_ROADMAP.md)

### Related Documents

- [Hakorune VM Discovery](../phase-20.5/HAKORUNE_VM_DISCOVERY.md) - VM implementation analysis
- [HostBridge API Design](../phase-20.5/HOSTBRIDGE_API_DESIGN.md) - C-ABI boundary
- [Op_eq Migration](../phase-20.5/OP_EQ_MIGRATION.md) - NoOperatorGuard pattern

---

## 📝 Notes

### Naming Conventions

- **Phase B**: VM Core Complete (Week 1-6)
- **Phase C**: Dispatch Unification (Week 7-12)
- **Golden Tests**: Rust-VM vs Hako-VM parity tests
- **Resolver**: Single resolution path for all method calls

### File Naming

- Handlers: `{instruction}_handler.hako`
- Tests: `test_{feature}_{case}.hako`
- Golden outputs: `{test}_rust.txt`, `{test}_hako.txt`

### Verification

- Use `diff` for exact output comparison
- Use Golden Test script for automation
- Use benchmark script for performance measurement

---

**Created**: 2025-10-14
**Last Updated**: 2025-10-14
**Status**: Planning (Phase not yet started)
**Phase Start**: 2026-03-01
**Phase End**: 2026-05-24
**Next Review**: 2026-03-01 (Phase start)
