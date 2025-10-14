# Strategy Reconciliation — Phase 20.5 "Escape from Rust"

**Date**: 2025-10-14
**Status**: APPROVED (Pure Hakorune Strategy)
**Decision**: User preference: "純 Hakorune 大作戦" ✅

---

## 🎯 Executive Summary

We had TWO competing strategies for Phase 20.5:

1. **Original Plan (Task Agent)**: C Code Generator approach - 10 weeks to bootstrap
2. **ChatGPT Pro Proposal**: Pure Hakorune VM - 30+ weeks for complete realization

**Decision**: Adopt **Pure Hakorune Strategy** with phased implementation.

**Phase 20.5 (10 weeks)** will complete:
- ✅ Phase A: HostBridge API (C-ABI boundary)
- ✅ Phase B (start): VM foundations in Hakorune (`op_eq` migration, instruction dispatch PoC)

**Deferred to Phase 20.6+**:
- ⏸️ Full VM core in Hakorune (Phase B complete)
- ⏸️ Dispatch unification (Phase C)
- ⏸️ Collections in Hakorune (Phase D)
- ⏸️ GC v0 (Phase E)
- ⏸️ Rust VM compat mode (Phase F)

**C Code Generator approach**: ❌ Discarded (not implemented)

---

## 📊 Side-by-Side Comparison

| Aspect | Original Plan (C Generator) | Pure Hakorune Strategy |
|--------|----------------------------|------------------------|
| **Timeline** | 10 weeks (Phase 20.5) | 30+ weeks (Phase 20.5-15.82) |
| **Core Approach** | Generate C code from MIR | Implement VM itself in Hakorune |
| **Rust Role** | Minimal (~200 lines) | Ultra-minimal (C-ABI bridge only) |
| **Output** | `.c` files → EXE | VM runs MIR natively |
| **Bootstrap Method** | v1 compiles v2 (via C code) | v1 VM runs v2 VM code |
| **Complexity** | Medium (500 lines C emitter) | High (full VM in Hakorune) |
| **Long-term Vision** | "Hakorune compiles Hakorune" | "Hakorune IS Hakorune" |
| **Architecture** | Compiler-focused | VM-focused |
| **Maintenance** | Two paths (Rust VM + C output) | Single path (Hakorune VM) |

---

## 💡 Why Pure Hakorune is Preferred

### Architectural Elegance

**C Generator Approach**:
```
Hakorune Source → [Rust Parser] → MIR JSON → [Hako C Gen] → C Code → [clang] → EXE
                     ↑                                                   ↑
                   Rust VM                                          C compiler
```

**Pure Hakorune Approach**:
```
Hakorune Source → [Hako Parser] → MIR JSON → [Hako VM] → Execution
                     ↑                          ↑
                Everything in Hakorune    Everything in Hakorune

Rust = Thin floor (C-ABI bridge only)
```

**Result**: "Rust=floor, Hakorune=house" - Ultimate Box Theory realization

### Long-term Maintainability

**C Generator Issues**:
- ❌ Two execution paths (Rust VM + compiled C)
- ❌ C code generation adds complexity
- ❌ C compiler dependency (clang/gcc)
- ❌ Debugging split between Hakorune → C → EXE

**Pure Hakorune Benefits**:
- ✅ Single execution path (Hakorune VM)
- ✅ VM semantics defined once (in Hakorune)
- ✅ No external compiler dependency (except Rust→Hakorune bridge)
- ✅ Debugging entirely in Hakorune

### Reflects Past Learnings

**ChatGPT Pro's guidance incorporates fixes for**:
1. **Equals/== recursion bug** (2025 issue):
   - Solution: `NoOperatorGuard` in `call_by_handle`
   - Pure Hakorune enforces this structurally

2. **Handle lifecycle complexity**:
   - Solution: Minimal C-ABI with `Retain/Release`
   - Pure Hakorune minimizes Rust↔Hakorune boundary

3. **Dispatch fragmentation**:
   - Solution: Single resolution path (`Resolver.lookup`)
   - Pure Hakorune unifies all dispatch

### Ultimate Box Theory

**C Generator**: Hakorune is a compiler *for* programs
**Pure Hakorune**: Hakorune *is* the program (self-contained)

**Box Philosophy**:
> Everything is Box, including the VM that runs Boxes.

Pure Hakorune realizes this fully.

---

## 🚧 What Gets Deferred to Phase 20.6+

### Phase 20.5 (10 weeks) Scope

**✅ CAN COMPLETE**:
1. **HostBridge API** (Week 1-4):
   - Minimal C-ABI surface (`Hako_RunScriptUtf8`, `Retain/Release`, etc.)
   - Ubuntu/Windows ABI tests
   - Error handling (TLS `Hako_LastError`)

2. **`op_eq` Migration** (Week 5-6):
   - Move equality logic from Rust to Hakorune
   - Implement `NoOperatorGuard` (prevent recursion)
   - Golden tests: Rust-VM vs Hako-VM parity

3. **VM Foundations PoC** (Week 7-8):
   - Instruction dispatch skeleton (5 instructions: const, binop, compare, jump, ret)
   - MIR execution loop in Hakorune
   - Integration test: Run simple programs

4. **Documentation** (Week 9-10):
   - HostBridge API spec
   - Op_eq migration guide
   - Pure Hakorune roadmap
   - Phase 20.6 planning

**❌ CANNOT COMPLETE** (deferred):
- Full VM core (all 16 instructions) → **Phase 20.6**
- Dispatch unification (Resolver-based) → **Phase 20.6**
- Collections in Hakorune (MapBox/ArrayBox) → **Phase 20.7**
- GC v0 (mark&amp;sweep) → **Phase 20.8**
- Rust VM deprecation → **Phase 20.8**

### Phase 20.6-15.82 Roadmap

**Phase 20.6** (12 weeks): VM Core Complete
- Phase B: All 16 MIR instructions in Hakorune
- Phase C: Dispatch unification (`Resolver.lookup` everywhere)
- Golden tests: 100% Rust-VM parity

**Phase 20.7** (8 weeks): Collections in Hakorune
- Phase D: MapBox/ArrayBox implementation
- Deterministic hash/eq
- Key normalization (Symbol/Int/String)

**Phase 20.8** (6 weeks): GC + Rust Deprecation
- Phase E: GC v0 (Stop-the-world mark&amp;sweep)
- Phase F: Rust VM becomes `--backend vm-rust` (compat mode)
- Hakorune-VM is default

**Total**: 36 weeks (Phase 20.5 → 15.82)

---

## ⚠️ Risk Assessment

### Longer Timeline Risk

**Risk**: 36 weeks vs original 10 weeks
**Impact**: HIGH (delayed complete self-hosting)
**Mitigation**:
- Phase 20.5 delivers **tangible value** (HostBridge, `op_eq`, VM PoC)
- Each phase is independently useful
- Can pause/reprioritize after each phase
- Progressive derisking (start with easy parts)

### Higher Complexity Risk

**Risk**: Implementing VM in Hakorune is harder than C generator
**Impact**: MEDIUM (more implementation effort)
**Mitigation**:
- Existing Rust VM as reference implementation
- Golden tests ensure parity (catch bugs early)
- Incremental approach (5 instructions → 16 instructions)
- ChatGPT/Claude collaboration for complex parts

### Rust VM Maintenance Risk

**Risk**: Must maintain both Rust VM and Hakorune VM during transition
**Impact**: MEDIUM (dual maintenance burden)
**Mitigation**:
- Rust VM frozen after Phase 20.5 (no new features)
- Hakorune VM is where new work happens
- Clear deprecation timeline (Phase 20.8)
- Golden tests prevent divergence

### C-ABI Stability Risk

**Risk**: HostBridge API might need changes during implementation
**Impact**: LOW (well-understood boundary)
**Mitigation**:
- Minimal API surface (5 functions)
- Inspired by proven designs (Lua, Python C API)
- Version function (`Hako_ApiVersion()`)
- Comprehensive ABI tests (Ubuntu/Windows)

---

## 🎯 Fallback Strategies

### If Pure Hakorune Takes Too Long

**Option 1**: Pause and ship Phase 20.5 deliverables
- HostBridge API is valuable independently
- `op_eq` in Hakorune improves correctness
- VM PoC demonstrates feasibility
- Resume Phase 20.6 later

**Option 2**: Hybrid approach (short-term)
- Keep Rust VM as primary
- Hakorune VM as experimental (`--backend vm-hako`)
- Gradually expand Hakorune VM coverage
- No hard deadline for full transition

**Option 3**: Revert to C Generator (if absolutely necessary)
- Original plan documents preserved
- C Generator design doc complete
- Can pivot in Phase 20.6 if needed

### If HostBridge API Has Issues

**Problem**: C-ABI doesn't work on Windows/Ubuntu
**Fallback**:
- Simplify API (fewer functions)
- Use more conservative types (no TLS, explicit context)
- Add compatibility shims per platform

### If `op_eq` Migration Fails

**Problem**: Hakorune-side equality doesn't match Rust behavior
**Fallback**:
- Keep Rust `op_eq` as reference
- Hakorune `op_eq` as opt-in (`HAKO_USE_PURE_EQ=1`)
- Golden tests identify discrepancies
- Fix incrementally

---

## 📋 Confidence Assessment

### High Confidence (80%+)

- ✅ **HostBridge API**: Well-understood, similar to Lua/Python C API
- ✅ **`op_eq` Migration**: Clear algorithm, golden tests available
- ✅ **VM PoC**: Rust VM as reference, only 5 instructions needed

### Medium Confidence (60-80%)

- ⚠️ **Full VM Core** (Phase 20.6): 16 instructions is more work
- ⚠️ **Dispatch Unification**: Requires careful refactoring
- ⚠️ **Collections**: MapBox/ArrayBox have subtle semantics

### Lower Confidence (40-60%)

- ⚠️ **GC v0**: Garbage collection is inherently complex
- ⚠️ **Rust VM Deprecation**: Requires full parity, no gaps

### Overall Confidence: **70%** (MEDIUM-HIGH)

**Reasoning**:
- Phase 20.5 deliverables are achievable (high confidence)
- Phase 20.6+ depends on learnings from 15.79 (medium confidence)
- Fallback options exist if needed (risk mitigation)

---

## 🚀 Why This is Worth It

### Short-term (Phase 20.5)

1. **HostBridge API**: Clean boundary between Rust and Hakorune
2. **`op_eq` in Hakorune**: Fixes recursion bugs, improves correctness
3. **VM PoC**: Demonstrates feasibility of Pure Hakorune vision

### Medium-term (Phase 20.6-15.81)

4. **VM Core Complete**: Hakorune can run itself
5. **Collections in Hakorune**: Everything is Box (truly)
6. **Single Codebase**: All semantics defined once (in Hakorune)

### Long-term (Phase 20.8+)

7. **Rust Minimal**: Only C-ABI bridge (~100 lines)
8. **True Self-Hosting**: Hakorune IS Hakorune (not just compiles)
9. **Ultimate Box Theory**: Realized in its purest form

---

## 📚 Document Evolution

### Original Documents (Preserved)

- `/phase-20.5/README.md` (original plan) → **UPDATED**
- `/phase-20.5/MILESTONE.md` (original milestones) → **UPDATED**
- `/phase-20.5/C_CODE_GENERATOR_DESIGN.md` → **ARCHIVED** (not implemented)
- `/phase-20.5/BOOTSTRAP_CHAIN_ANALYSIS.md` → **UPDATED** (Pure Hakorune approach added)

### New Documents (This Reconciliation)

- `/phase-20.5/STRATEGY_RECONCILIATION.md` → **THIS FILE**
- `/phase-20.5/HOSTBRIDGE_API_DESIGN.md` → **NEW** (C-ABI spec)
- `/phase-20.5/PURE_HAKORUNE_ROADMAP.md` → **NEW** (Phase 20.5-15.82 plan)
- `/phase-20.5/OP_EQ_MIGRATION.md` → **NEW** (equality implementation guide)

### Updated Documents

- `/phase-20.5/README.md`: Removed C Generator, added HostBridge/op_eq
- `/phase-20.5/MILESTONE.md`: Week-by-week plan updated
- `/phase-20.5/BOOTSTRAP_CHAIN_ANALYSIS.md`: Pure Hakorune approach added

---

## 🎤 Final Recommendation

**Adopt Pure Hakorune Strategy** for the following reasons:

1. **Architectural elegance**: "Rust=floor, Hakorune=house"
2. **Long-term maintainability**: Single execution path
3. **Reflects learnings**: Incorporates past bug fixes
4. **Ultimate Box Theory**: Realizes core philosophy
5. **User preference**: tomoaki chose "純 Hakorune 大作戦" ✅

**Accept longer timeline** (36 weeks) for:
- Clean architecture
- Minimal technical debt
- Future-proof foundation

**Phase 20.5 (10 weeks)** delivers:
- HostBridge API (C-ABI boundary)
- `op_eq` in Hakorune (correctness improvement)
- VM PoC (feasibility demonstration)

**Next phases** (15.80-15.82) complete the vision.

---

**Status**: APPROVED
**Decision Maker**: tomoaki
**Implementation**: ChatGPT (lead), Claude (review)
**Timeline**: 2025-12-21 → 2026-09-30 (Phase 20.5-15.82)
