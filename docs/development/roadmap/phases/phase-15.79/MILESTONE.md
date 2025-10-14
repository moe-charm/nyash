# Phase 15.79 — Milestone (Escape from Rust / Bootstrap Compiler)

Status: Planning
Start: 2025-12-21
End: 2026-02-28 (10 weeks)

---

## 🎯 Objectives

**Turn the frozen EXE into a true bootstrap compiler that can compile itself**

1. **Bootstrap Compiler Implementation**: Hakorune-written compiler running on frozen EXE
2. **3-Stage Bootstrap Chain**: Rust → Hakorune(v1) → Hakorune(v2)
3. **C Code Generator**: MIR → C code emission
4. **Compiler Parity**: v1 == v2 verification (identical output)

---

## ✅ Deliverables (DoD)

### 1️⃣ Bootstrap Chain Operational

```bash
# Stage 1: Rust compiler (frozen)
./hako-frozen-v1 program.hako --emit-mir program.mir.json

# Stage 2: Hakorune compiler v1
./hako-frozen-v1 apps/bootstrap-compiler/main.hako \
  --input program.hako \
  --output program_v1.c

# Stage 3: v1 compiles v2
./hako-frozen-v1 apps/bootstrap-compiler/main.hako \
  --input apps/bootstrap-compiler/main.hako \
  --output bootstrap_v2.c

# Verification: v1 == v2
diff program_v1.c program_v2.c  # Expected: identical
```

### 2️⃣ C Code Generator Complete

- [ ] MIR JSON → C code conversion
- [ ] All 16 instructions supported
- [ ] NyRT function calls generated
- [ ] 10 smoke tests PASS

### 3️⃣ Compiler Parity Verified

- [ ] 10 test programs: v1 == v2 output
- [ ] MIR output identical
- [ ] Execution results identical
- [ ] Performance: v2 ≥ 80% of v1

### 4️⃣ Documentation Complete

- [ ] Bootstrap procedure guide
- [ ] C Code Generator design doc
- [ ] Troubleshooting FAQ
- [ ] Completion report

---

## 📊 Weekly Milestones

### Week 1-2: Design & Investigation
- Complete apps/selfhost-compiler/ analysis
- Frozen EXE constraints analysis
- C Code Generator design document
- Bootstrap Chain detailed design

### Week 3-4: Parser Adaptation
- ParserBox dependency cleanup
- Frozen EXE constraints adaptation
- 10 test cases created
- AST JSON output verified

### Week 5-6: MIR Builder Migration
- MIR Builder core structure migrated
- All 16 instructions supported
- CFG construction (Branch/Jump/Phi)
- MIR JSON output smoke tests

### Week 7-8: C Code Generator Implementation
- C Code Emitter Box implemented
- 16 instructions → C conversion
- NyRT function call generation
- C → EXE → execution verified

### Week 9: Bootstrap Chain Integration
- Stage 1 → Stage 2 verified
- Stage 2 → Stage 3 verified
- v1 == v2 parity (10 cases)
- Performance measurements

### Week 10: Documentation & Review
- Bootstrap procedure guide written
- Troubleshooting FAQ created
- Completion report drafted
- ChatGPT/Claude review completed

---

## 🎯 Success Criteria

### Technical

1. **Bootstrap Chain Works**:
   - Rust frozen EXE compiles v1
   - v1 compiles v2
   - v2 compiles v3
   - v1 == v2 == v3 (identical output)

2. **C Code Generator Works**:
   - All 16 MIR instructions → C
   - NyRT function calls correct
   - Generated C compiles and runs
   - 10/10 smoke tests PASS

3. **Performance Acceptable**:
   - v1 compile time < 5 seconds for simple programs
   - v2 ≥ 80% of v1 performance
   - Memory usage reasonable (< 100MB)

### Process

1. **Documentation Complete**:
   - User can follow bootstrap guide
   - Troubleshooting covers common issues
   - Design rationale documented

2. **Testing Comprehensive**:
   - 10+ test cases for each component
   - Golden tests for v1 == v2
   - Edge cases covered

3. **Review Approved**:
   - ChatGPT review complete
   - Claude review complete
   - No blocking issues

---

## 🚨 Risks & Mitigations

### Risk 1: Frozen EXE Constraints

**Problem**: Limited Box set in frozen EXE
**Impact**: HIGH
**Mitigation**:
- Pre-survey required Boxes
- Implement workarounds in Hakorune
- Test each Box function

### Risk 2: C Code Generator Complexity

**Problem**: 16 instructions → C is non-trivial
**Impact**: MEDIUM
**Mitigation**:
- Incremental implementation (basic → advanced)
- Test-driven development
- Reference LLVM Backend implementation

### Risk 3: Bootstrap Chain Verification

**Problem**: v1 == v2 verification difficult
**Impact**: MEDIUM
**Mitigation**:
- Diff tools for C output comparison
- Golden tests with known programs
- Incremental verification (small → large)

### Risk 4: Performance Issues

**Problem**: Hakorune compiler might be slow
**Impact**: LOW
**Mitigation**:
- Measure execution time at each stage
- Identify bottlenecks
- Acceptable if v2 ≥ 80% of v1

---

## 📋 Implementation Strategy: Option B (Recommended)

### Reuse apps/selfhost-compiler/ ⭐

**Rationale**:
- 2500 lines of existing implementation
- 170 test cases already PASS
- Proven Parser/Emitter/MIR Builder
- 90%+ code reusability

**Adaptation Required**:
```
Existing: apps/selfhost-compiler/
├── ParserBox           ✅ Reuse as-is
├── EmitterBox          ✅ Reuse as-is
├── MirEmitterBox       ⚠️ Minor adjustments
└── JsonProgramBox      ✅ Reuse as-is

New Implementation:
└── CCodeEmitterBox     ❌ New (Week 7-8)
```

**Benefits**:
- Shorter development time (10 weeks vs 15+ weeks)
- Battle-tested code
- Existing test suite
- Proven architecture

---

## 🔄 Bootstrap Chain Verification Flow

```
      ┌─────────────┐
      │ program.hako│
      └──────┬──────┘
             │
    ┌────────┴────────┐
    │                 │
┌───▼────┐      ┌────▼────┐
│Stage 1 │      │Stage 2  │
│(Rust)  │      │(Hako v1)│
└───┬────┘      └────┬────┘
    │                │
    v                v
┌────────┐      ┌────────┐
│mir.json│      │prog.c  │
└────────┘      └────────┘

        ┌────────────┐
        │bootstrap   │
        │compiler    │
        │(v1 source) │
        └──────┬─────┘
               │
          ┌────▼────┐
          │Stage 2  │
          │(Hako v1)│
          └────┬────┘
               │
               v
          ┌────────┐
          │boot_v2.c│
          └────┬───┘
               │
          ┌────▼────┐
          │Stage 3  │
          │(Hako v2)│
          └────┬────┘
               │
          Verify: v1 == v2
```

---

## 📦 Code Reuse Analysis

### apps/selfhost-compiler/ Structure

```
apps/selfhost-compiler/              2500 lines total
├── boxes/parser/                    1328 lines ✅ 90% reusable
│   ├── parser_box.hako              237 lines
│   ├── expr/                        570 lines
│   └── stmt/                        521 lines
├── boxes/emitter_box.hako           10 lines  ✅ 100% reusable
├── boxes/mir_emitter_box.hako       179 lines ⚠️ 80% reusable
├── boxes/json_program_box.hako      264 lines ✅ 100% reusable
├── builder/ssa/                     200 lines ⚠️ Optional
└── common/                          ~300 lines ✅ 95% reusable

Reusability: ~90% (2250/2500 lines)
New Implementation: ~500 lines (C Code Generator)
Total Effort: ~750 lines (500 new + 250 adaptations)
```

### Frozen EXE Available Boxes

```
Frozen v1 Includes:
├── String             ✅ Used extensively
├── Array              ✅ Used extensively
├── Map                ✅ Used for AST/MIR
├── Console (print)    ✅ Used for output
├── Time (now_ms)      ⚠️ Optional (profiling)
├── JSON (stringify)   ✅ Used for MIR JSON
└── File[min]          ✅ Used for I/O

Additional Boxes Needed:
└── None (sufficient for bootstrap)
```

---

## 🎉 Success Impact

After Phase 15.79 completion:

1. **True Self-Hosting Achieved**: Hakorune compiles Hakorune
2. **Rust Dependency Minimized**: Only VM executor remains (~200 lines)
3. **Development Velocity Increased**: Single codebase in Hakorune
4. **Extension Simplified**: Language features easier to add

---

## 📚 Related Resources

### Previous Phases
- [Phase 15.77 - Frozen EXE Finalization](../phase-15.77/)
- [Phase 15.76 - extern_c & Frozen Toolchain](../phase-15.76/)

### Reference Implementation
- [apps/selfhost-compiler/](../../../../apps/selfhost-compiler/)
- [src/llvm_py/](../../../../src/llvm_py/) - LLVM Backend reference

### Industry Patterns
- **Rust**: stage0 (frozen) → stage1 (bootstrap) → stage2 (verify)
- **Go**: Go 1.4 frozen → Go 1.5 self-hosted
- **OCaml**: ocamlc frozen → ocamlopt self-hosted

---

**Created**: 2025-10-14
**Phase Start**: 2025-12-21 (after Phase 15.77 completion)
**Duration**: 10 weeks
**Strategy**: Reuse apps/selfhost-compiler/ (Option B)
