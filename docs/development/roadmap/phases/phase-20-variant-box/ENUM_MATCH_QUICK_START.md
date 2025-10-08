# @enum/@match Quick Start Guide

**For**: Developers implementing Choice A'' (Macro-Only Approach)
**Full Plan**: [ENUM_MATCH_PROJECT_PLAN.md](./ENUM_MATCH_PROJECT_PLAN.md)

---

## ⚡ TL;DR

**Timeline**: 12-17 days (2-4 weeks)
**Approach**: Macro expansion only, no MIR changes
**Risk Level**: Medium (60% chance of parser conflicts, 55% chance of binding issues)

---

## 📅 3-Week Schedule

### **Week 1: @enum Macro (Days 1-5)**
- Day 1: Parser + AST (6-8h)
- Day 2: Macro engine integration (6-8h)
- Day 3: Constructor generation (6-8h)
- Day 4: Test suite - 10 patterns (6-8h)
- Day 5: Smoke tests (4-6h)

**Gate 1**: 10/10 tests PASS or rollback

---

### **Week 2: @match Macro (Days 6-11)**
- Day 6: Parser @match (6-8h)
- Day 7: Pattern analysis (6-8h)
- Day 8: If-else desugaring (6-8h)
- Day 9: Exhaustiveness check (4-6h)
- Day 10: Test suite - 15 patterns (8h)
- Day 11: Edge cases (4-6h)

**Gate 2**: 15/15 tests PASS or rollback to @enum-only

---

### **Week 3: Selfhost (Days 12-14)**
- Day 12: Option/Result v2 (4h)
- Day 13: Mini-VM integration (6-8h)
- Day 14: Integration tests + docs (4-6h)

**Gate 3**: ALL smoke tests PASS or full rollback

---

### **Buffer (Days 15-17)**
- Debug, optimize, polish

---

## 🚨 Critical Risks

### **Risk #1: Parser Conflicts (60%)**
- **Symptom**: `@enum` conflicts with existing syntax
- **Mitigation**: Use `enum!` syntax instead
- **Cost**: +1 day

### **Risk #2: Variable Bindings (55%)**
- **Symptom**: Field extraction fails in multi-field variants
- **Mitigation**: MVP supports single-field only
- **Cost**: +2 days

### **Risk #3: False Positives (50%)**
- **Symptom**: Valid code triggers exhaustiveness panic
- **Mitigation**: Add `NYASH_MATCH_ALLOW_PARTIAL=1` flag
- **Cost**: +1 day

---

## ✅ Success Criteria

### **Must Have (Go/No-Go)**
- ✅ @enum: 10/10 tests PASS
- ✅ @match: 15/15 tests PASS
- ✅ Smoke tests: ALL PASS
- ✅ Performance: < 5% slowdown

### **Nice to Have**
- 100% pattern consistency
- Comprehensive docs
- Zero regressions

---

## 🔄 Rollback Options

### **Option 1: Keep @enum Only**
- **Trigger**: @match fails (< 12/15 tests)
- **Time Saved**: 4-6 days
- **Quality Loss**: Medium (still get type-safe constructors)

### **Option 2: Full Rollback**
- **Trigger**: Both fail or block critical work
- **Time Saved**: 9-14 days
- **Quality Loss**: High (but recoverable in Phase 20)

---

## 📊 Daily Checklist

### **Every Day**
- [ ] Run relevant tests (unit + integration)
- [ ] Update CURRENT_TASK.md with progress
- [ ] Document issues/blockers
- [ ] Check timeline (on track for gate?)

### **Gate Days (5, 11, 14)**
- [ ] Run full test suite
- [ ] Performance check
- [ ] Go/No-Go decision
- [ ] Update stakeholders

---

## 🎯 Key Files to Track

### **Implementation**
- `src/parser/mod.rs` (~230 lines)
- `apps/macros/enum/enum_macro.hako` (~150 lines)
- `apps/macros/match/match_macro.hako` (~200 lines)

### **Tests**
- `apps/tests/enum_comprehensive.hako` (~200 lines)
- `apps/tests/match_comprehensive.hako` (~300 lines)
- `tools/smokes/v2/profiles/quick/core/enum_basic_vm.sh`
- `tools/smokes/v2/profiles/quick/core/match_basic_vm.sh`

### **Docs**
- `apps/macros/enum/README.md`
- `apps/macros/match/README.md`
- `docs/guides/enum-match-migration.md`

---

## 🚀 Quick Commands

### **Run Tests**
```bash
# @enum tests
NYASH_TEST_RUN=1 ./target/release/hako apps/tests/enum_comprehensive.hako

# @match tests
NYASH_TEST_RUN=1 ./target/release/hako apps/tests/match_comprehensive.hako

# Smoke tests
tools/smokes/v2/run.sh --profile quick

# Specific smoke
bash tools/smokes/v2/profiles/quick/core/enum_basic_vm.sh
```

### **Debug**
```bash
# Macro trace
NYASH_MACRO_TRACE=1 ./target/release/hako test.hako

# Verbose expansion
NYASH_MACRO_VERBOSE=1 ./target/release/hako test.hako

# MIR dump
./target/release/hako --dump-mir test.hako
```

---

## 📈 Progress Tracking

### **Week 1 Progress** (Days 1-5)
- [ ] Day 1: Parser + AST ✅ or ❌
- [ ] Day 2: Macro integration ✅ or ❌
- [ ] Day 3: Constructors ✅ or ❌
- [ ] Day 4: 10 tests PASS ✅ or ❌
- [ ] Day 5: Smoke tests ✅ or ❌
- [ ] **Gate 1**: Go ✅ or No-Go ❌

### **Week 2 Progress** (Days 6-11)
- [ ] Day 6: Parser @match ✅ or ❌
- [ ] Day 7: Pattern analysis ✅ or ❌
- [ ] Day 8: Desugaring ✅ or ❌
- [ ] Day 9: Exhaustiveness ✅ or ❌
- [ ] Day 10: 15 tests PASS ✅ or ❌
- [ ] Day 11: Edge cases ✅ or ❌
- [ ] **Gate 2**: Go ✅ or No-Go ❌

### **Week 3 Progress** (Days 12-14)
- [ ] Day 12: Option/Result v2 ✅ or ❌
- [ ] Day 13: Mini-VM ✅ or ❌
- [ ] Day 14: Integration ✅ or ❌
- [ ] **Gate 3**: Go ✅ or No-Go ❌

---

## 💡 Tips for Success

### **Time Management**
- Start each day with clear goal (what must work by EOD?)
- Take breaks every 2 hours (avoid burnout)
- If stuck > 1 hour, consult design docs or escalate

### **Quality First**
- Don't skip tests to "catch up" (technical debt compounds)
- Write tests BEFORE implementation when possible
- Document limitations immediately (don't hide issues)

### **Communication**
- Daily updates in CURRENT_TASK.md (2 minutes)
- Gate decisions documented (5 minutes)
- Ask for help early (don't wait until rollback threshold)

---

## 🎓 When to Escalate

### **Immediate Escalation**
- MIR16 cannot express required pattern (Day 2)
- Fundamental design flaw discovered (any day)
- Security issue found (any day)

### **Gate Escalation**
- Gate 1: < 8/10 tests PASS
- Gate 2: < 12/15 tests PASS
- Gate 3: > 2 smoke test failures

### **Timeline Escalation**
- Day 17: Still not done → Full rollback

---

**Last Updated**: 2025-10-08
**Status**: READY TO START (after Phase 15.7)
**Owner**: (Your name here)
