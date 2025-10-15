# Phase 20.5 — Index

**Escape from Rust: Bootstrap Compiler Implementation**

Status: Planning (Gate plan active)
Duration: 10 weeks (2025-12-21 - 2026-02-28)

---

## 📚 Documentation Structure

### Core Documents

1. **[README.md](README.md)** ⭐ START HERE
   - Phase overview and goals
   - Weekly breakdown (Week 1-10)
   - Implementation strategy decision (Option B)
   - Success criteria and DoD

2. **[MILESTONE.md](MILESTONE.md)**
   - Objectives and deliverables
   - Weekly milestones
   - Success criteria
   - Risk analysis

3. **[BOOTSTRAP_CHAIN_ANALYSIS.md](BOOTSTRAP_CHAIN_ANALYSIS.md)**
   - 3-stage bootstrap chain detailed design
   - Stage 1 (Rust) → Stage 2 (Hako v1) → Stage 3 (Hako v2)
   - Data flow analysis
   - Verification strategy

4. **[C_CODE_GENERATOR_DESIGN.md](C_CODE_GENERATOR_DESIGN.md)**
   - MIR → C conversion design
   - 16-instruction mapping table
   - PHI resolution strategy
   - Test strategy (43 test cases)

5. **[PLAN.md](PLAN.md)** ✅ Gate-based execution plan（短縮版）
   - 5行サマリ / 最小命令セット / DoD
   - Gate A〜E（Parser→MIR→VM PoC→op_eq→統合）
   - テスト/CI/リスク/次ステップ

---

## 🎯 Quick Reference

### What is Phase 20.5? (5-line summary)

1) Goal: 脱Rust。凍結EXEを土台に自己ホストへ前進。
2) Strategy: Gate方式（Parser→MIR→VM PoC→op_eq→統合）。
3) Boundary: C‑ABI/HostBridgeのみ外部境界。中はEverything is Box。
4) Proof: 決定性（JSON正規化）、Golden＆固定点で検証。
5) Policy: 小さく、順序よく、SKIPはWARN、回帰のみFAIL。

### Key Deliverables

1. **Bootstrap Compiler** (`apps/bootstrap-compiler/`)
   - Written in Hakorune
   - Runs on frozen EXE
   - Outputs C code

2. **C Code Generator**
   - MIR JSON → C source
   - 16 instructions fully supported
   - NyRT function calls

3. **Verification**
   - v1 == v2 (compiler parity)
   - v2 == v3 (fixed point)
   - 10 test programs PASS

---

## 📊 Timeline at a Glance

| Weeks | Focus | Deliverable |
|-------|-------|-------------|
| 1-2 | Design & Analysis | Design docs complete |
| 3-4 | Parser Adaptation | Parser works on frozen EXE |
| 5-6 | MIR Builder Migration | MIR JSON generation works |
| 7-8 | C Code Generator | C code emission works |
| 9 | Bootstrap Integration | v1 == v2 verified |
| 10 | Documentation & Review | Phase complete |

---

## 🔄 Bootstrap Chain Overview

```
      ┌─────────────────┐
      │ hako-frozen-v1  │
      │   (Rust, 724KB) │
      └────────┬────────┘
               │
               │ compiles
               v
      ┌─────────────────┐
      │ bootstrap_v1    │
      │ (Hakorune code) │
      │ runs on frozen  │
      └────────┬────────┘
               │
               │ compiles itself
               v
      ┌─────────────────┐
      │ bootstrap_v2.c  │
      │ (C source)      │
      └────────┬────────┘
               │
               │ clang + NyRT
               v
      ┌─────────────────┐
      │ bootstrap_v2    │
      │ (native binary) │
      └─────────────────┘

      Verify: v1 output == v2 output ✅
```

---

## 💡 Implementation Strategy

### Decision: Option B (Reuse apps/selfhost-compiler/)

**Rationale**:
- 2500 lines of existing, tested code
- 170 smoke tests already PASS
- 90%+ reusability
- Proven architecture

**Code Reuse Breakdown**:
```
✅ 90% Reusable: Parser, Emitter, MIR Builder (2250 lines)
⚠️ 10% Adaptation: using paths, Box constraints (250 lines)
❌ New Code: C Code Generator (500 lines)

Total Effort: ~750 lines (vs 2500+ for Option A)
```

---

## 🧪 Test Strategy

### Test Pyramid

```
         /\
        /  \    Level 3: Self-Compilation (1 test, slow)
       /____\   - v1 compiles itself → v2
      /      \  - v2 compiles itself → v3
     /        \ - Verify: v2 == v3
    /__________\
   /            \ Level 2: Comprehensive (10 tests, medium)
  /              \ - If/else, loops, functions, boxes, arrays
 /________________\
/                  \ Level 1: Smoke Tests (43 tests, fast)
                     - Each MIR instruction
                     - Basic parsing
                     - C generation
```

---

## 📦 Directory Structure

```
Phase 20.5 Planning:
docs/development/roadmap/phases/phase-20.5/
├── INDEX.md                          # ← You are here
├── README.md                         # Phase overview
├── MILESTONE.md                      # Milestones & DoD
├── BOOTSTRAP_CHAIN_ANALYSIS.md       # 3-stage bootstrap
├── C_CODE_GENERATOR_DESIGN.md        # C codegen design
├── REUSABILITY_ANALYSIS.md           # (Week 1-2 deliverable)
├── RISK_ASSESSMENT.md                # (Week 1-2 deliverable)
├── IMPLEMENTATION_GUIDE.md           # (Week 3+ deliverable)
└── COMPLETION_REPORT.md              # (Week 10 deliverable)

Implementation:
apps/bootstrap-compiler/
├── parser/                           # Week 3-4
│   ├── parser_box.hako
│   └── lexer_box.hako
├── mir_builder/                      # Week 5-6
│   └── builder_box.hako
├── codegen/                          # Week 7-8
│   ├── c_emitter_box.hako
│   └── c_runtime_box.hako
├── tests/                            # Week 3-9
│   ├── smoke/                        # 43 tests
│   ├── integration/                  # 10 tests
│   └── bootstrap/                    # 1 test (v1==v2==v3)
├── main.hako                         # Entry point
└── README.md                         # User guide
```

---

## 🚀 Quick Start (After Phase Complete)

### Compile a Hakorune Program

```bash
# Stage 1: Use frozen EXE to run bootstrap compiler
./hako-frozen-v1 apps/bootstrap-compiler/main.hako \
  --input my_program.hako \
  --output my_program.c

# Stage 2: Compile C to native binary
clang my_program.c -o my_program \
  -L /path/to/hako_kernel \
  -lhako_kernel \
  -lpthread -ldl -lm

# Stage 3: Run
./my_program
```

### Verify Bootstrap Chain

```bash
# Run verification script
bash tools/verify_bootstrap_chain.sh

# Expected output:
# ✅ Stage 1 → Stage 2: OK
# ✅ Stage 2 → Stage 3: OK
# ✅ v1 == v2: VERIFIED
# ✅ v2 == v3: VERIFIED (fixed point)
# ✅ Bootstrap chain complete!
```

---

## ⚠️ Prerequisites

### From Phase 15.77

- [x] Frozen EXE built and tested (`hako-frozen-v1.exe`)
- [x] NyRT function calls working (Result: 6 test PASS)
- [x] MIR JSON → .o → EXE pipeline verified
- [x] Windows (MSVC/MinGW) + Linux support

### For Phase 20.5

- [ ] apps/selfhost-compiler/ analysis complete (Week 1-2)
- [ ] Frozen EXE constraints documented (Week 1-2)
- [ ] C Code Generator design approved (Week 1-2)
- [ ] Test infrastructure ready (Week 3+)

---

## 🎯 Success Criteria Summary

### Technical

- [ ] Bootstrap chain works: Stage 1 → 2 → 3
- [ ] C Code Generator: 16/16 instructions supported
- [ ] Verification: v1 == v2 == v3 (identical output)
- [ ] Tests: 43 smoke + 10 integration + 1 bootstrap PASS

### Performance

- [ ] Stage 2 compile time: < 30s for self-compilation
- [ ] Stage 3 compile time: < 5s for self-compilation
- [ ] Memory usage: < 100MB

### Quality

- [ ] Documentation complete (user guide + design docs)
- [ ] Code is modular (Box-based)
- [ ] Edge cases covered
- [ ] Review approved (ChatGPT + Claude)

---

## 📚 Related Phases

### Previous

- [Phase 15.77 - Frozen EXE Finalization](../phase-15.77/)
- [Phase 15.76 - extern_c & Frozen Toolchain](../phase-15.76/)
- [Phase 15.75 - Escape from Rust Planning](../phase-15.75/)

### Next

- **Phase 20.6 - Complete Rust Removal**
  - VM executor → Hakorune implementation
  - Rust codebase → 0 lines
  - Pure Hakorune self-hosting

### Parallel

- [Phase 15.78 - Frozen UX Polish](../phase-15.78/)
  - Distribution packaging
  - Doctor improvements
  - Windows polish

---

## 💬 Communication

### Weekly Sync Points

- **Monday**: Week start, goal setting
- **Wednesday**: Mid-week progress check
- **Friday**: Week review, next week planning

### Issue Tracking

- Use GitHub issues with label `phase-20.5`
- Prefix: `[20.5]` in commit messages
- Milestone: `Phase 20.5 - Bootstrap Compiler`

### Review Process

- Each week: Self-review + smoke tests
- Week 5, 10: Full review with ChatGPT/Claude
- Blocking issues: Immediate escalation

---

## 🔗 External Resources

### Industry Examples

- **Rust Bootstrap**: [Rust stage0 documentation](https://rustc-dev-guide.rust-lang.org/building/bootstrapping.html)
- **Go Bootstrap**: [Go 1.5 Bootstrap Process](https://go.dev/doc/go1.5#bootstrap)
- **OCaml Bootstrap**: [OCaml self-hosting](https://ocaml.org/docs/compiling-ocaml-projects)

### Papers

- [Rapid Self-Hosting Paper](../../../../private/papers-active/rapid-selfhost-ai-collaboration/)
- [Reflections on Trusting Trust](https://www.cs.cmu.edu/~rdriley/487/papers/Thompson_1984_ReflectionsonTrustingTrust.pdf) (Ken Thompson, 1984)

---

## 📝 Notes

### Naming Conventions

- **Stage 1**: Rust compiler, frozen EXE (`hako-frozen-v1`)
- **Stage 2**: Hakorune compiler v1 (`bootstrap_v1`)
- **Stage 3**: Hakorune compiler v2 (`bootstrap_v2`)
- **Fixed Point**: v2 == v3 (self-consistency)

### File Naming

- Test outputs: `test_v1.c`, `test_v2.c`
- Bootstrap outputs: `bootstrap_v2.c`, `bootstrap_v3.c`
- Always include version suffix for clarity

### Verification

- Use `diff` for exact comparison
- Use `md5sum` for quick hash checks
- Use `tools/verify_bootstrap_chain.sh` for automation

---

**Created**: 2025-10-14
**Last Updated**: 2025-10-14
**Status**: Planning (Phase not yet started)
**Next Review**: 2025-12-21 (Phase start)
