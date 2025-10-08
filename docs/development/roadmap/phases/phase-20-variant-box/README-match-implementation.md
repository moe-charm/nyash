# @match Macro Implementation - Complete Package

**Created**: 2025-10-08
**Status**: Ready for Implementation
**Estimated Effort**: 6 person-days (48 hours)
**Approach**: Choice A'' (Macro-Only)

---

## 📦 Documentation Package

This directory contains everything needed to implement the @match macro for Hakorune:

### 1. **Implementation Specification** (Complete)
[**@match-macro-implementation-spec.md**](./@match-macro-implementation-spec.md)

**Contents**:
- ✅ Complete parser changes specification
- ✅ 5 detailed desugaring examples
- ✅ 15 test case specifications with expected output
- ✅ Desugaring algorithm pseudo-code
- ✅ Error handling design
- ✅ Risk analysis
- ✅ Integration with @enum (future)

**Length**: ~1000 lines of detailed technical specification

**Use**: Reference document for implementation details

---

### 2. **Quick Start Guide** (Practical)
[**@match-quick-start.md**](./@match-quick-start.md)

**Contents**:
- ✅ 6-day implementation plan with daily tasks
- ✅ Key code snippets for parser/desugaring
- ✅ Testing strategy and smoke test templates
- ✅ Common pitfalls and solutions
- ✅ Progress checklist
- ✅ Debug commands

**Length**: ~300 lines of practical guidance

**Use**: Day-to-day implementation guide

---

### 3. **VariantBox Design** (Context)
[**DESIGN.md**](./DESIGN.md)

**Contents**:
- ✅ Overall VariantBox philosophy
- ✅ @enum desugaring specification
- ✅ Long-term Phase 20 roadmap

**Use**: Understand the bigger picture

---

## 🎯 Implementation Summary

### What is @match?

**Macro-based pattern matching** that desugars to if/else chains:

```hakorune
@match result {
  Ok(v) => print("Success: " + v)
  Err(e) => print("Error: " + e)
}
```

**Becomes**:
```hakorune
if result._ok == 1 {
  local v = result._val
  print("Success: " + v)
} else if result._ok == 0 {
  local e = result._err
  print("Error: " + e)
} else {
  print("[PANIC] non-exhaustive match on Result")
}
```

---

### Why Macro-Only (Choice A'')?

**Philosophy**:
- ✅ **No runtime overhead**: Pure AST transformation
- ✅ **MIR16 frozen**: Uses existing if/else/compare instructions
- ✅ **Fast delivery**: 6 days vs 4-6 weeks for full VariantBox
- ✅ **Complete experience**: Pattern matching works immediately
- ✅ **Future-proof**: Can add VariantBox Core later (transparent upgrade)

**Tradeoffs**:
- ❌ No compile-time exhaustiveness check (Phase 2)
- ❌ Hardcoded field mappings (Result/Option only in Phase 1)
- ❌ No SymbolBox optimization (strings for tags)

**Acceptable because**:
- Runtime panic catches non-exhaustive matches (safe)
- Result/Option are 90% of use cases
- String tags are fast enough for selfhost compiler

---

### Phase 1 Scope (MVP)

**Included**:
- ✅ Variant patterns: `Ok(v)`, `Err(e)`, `Some(x)`, `None`
- ✅ Multi-field patterns: `Cartesian(x, y)`
- ✅ Wildcard pattern: `_`
- ✅ Variable binding
- ✅ Match as expression (returns value)
- ✅ Runtime exhaustiveness check (panic)
- ✅ Result/Option support (existing boxes)

**Excluded (Future)**:
- ❌ Literal patterns (`42 => ...`)
- ❌ Guard clauses (`Some(x) if x > 10`)
- ❌ Nested patterns (`Some(Ok(v))`)
- ❌ Compile-time exhaustiveness check
- ❌ Generic VariantBox (without @enum)

---

## 📋 Quick Reference

### Files to Modify

| File | Changes | Lines |
|------|---------|-------|
| `src/ast.rs` | Add `MatchPattern` AST node | +30 |
| `src/tokenizer.rs` | Add `FatArrow`, `Underscore` tokens | +10 |
| `src/parser/statements/mod.rs` | Add `parse_match_pattern()` | +150 |
| `src/macro/match_desugar.rs` | **New**: Desugaring logic | +200 |

**Total new code**: ~400 lines

---

### Test Files to Create

| File | Description | Lines |
|------|-------------|-------|
| `apps/tests/match/test_result_basic.hako` | Result 2-variant | 20 |
| `apps/tests/match/test_option_basic.hako` | Option 2-variant | 20 |
| `apps/tests/match/test_variant_3way.hako` | 3+ variants | 30 |
| `apps/tests/match/test_match_expression.hako` | Match as expr | 25 |
| `apps/tests/match/test_match_blocks.hako` | Multi-statement | 30 |
| `apps/tests/match/test_binding_single.hako` | Single binding | 20 |
| `apps/tests/match/test_binding_multi.hako` | Multi binding | 30 |
| `apps/tests/match/test_shadowing.hako` | Variable shadow | 25 |
| `apps/tests/match/test_exhaustive_ok.hako` | Exhaustive | 20 |
| `apps/tests/match/test_exhaustive_fail.hako` | Non-exhaustive | 15 |
| `apps/tests/match/test_wildcard.hako` | Wildcard | 20 |
| `apps/tests/match/test_0field_variant.hako` | None variant | 15 |
| `apps/tests/match/test_nested_if.hako` | Nested if | 25 |
| `apps/tests/match/test_loop_control.hako` | Break/continue | 30 |
| `apps/tests/match/test_early_return.hako` | Early return | 25 |

**Total test code**: ~350 lines

---

### Implementation Milestones

| Day | Milestone | Verification |
|-----|-----------|--------------|
| 1 | Parser accepts `@match` syntax | No parse errors |
| 2 | Tag comparison generation | `--dump-mir` shows if-conditions |
| 3 | Variable binding works | `--dump-mir` shows `local v = ...` |
| 4 | Exhaustiveness panic works | Invalid state triggers panic |
| 5 | 8/15 tests pass | Basic + binding tests |
| 6 | 15/15 tests pass | All tests + smoke tests |

---

## 🚀 Getting Started

### Step 1: Read Quick Start
```bash
cat docs/development/roadmap/phases/phase-20-variant-box/@match-quick-start.md
```

### Step 2: Day 1 Tasks
1. Add `MatchPattern` to `src/ast.rs`
2. Add `FatArrow` token to `src/tokenizer.rs`
3. Implement `parse_match_pattern()` in parser
4. Test parsing basic @match

### Step 3: Verify
```bash
# Should parse without error
echo '@match x { Ok(v) => v }' | ./target/release/hako --dump-ast -
```

### Step 4: Continue Daily Plan
Follow [Quick Start Guide](./@match-quick-start.md) for Days 2-6

---

## 📖 Documentation Hierarchy

```
README-match-implementation.md (this file)
  ├─ Overview and quick reference
  └─ Points to:
      ├─ @match-quick-start.md
      │   └─ Daily implementation guide
      └─ @match-macro-implementation-spec.md
          └─ Complete technical specification
```

**Recommendation**:
1. **Start here** for overview
2. **Use Quick Start** for day-to-day work
3. **Reference Spec** for detailed questions

---

## ✅ Success Criteria

### MVP Complete When:
- ✅ All 15 test cases pass
- ✅ Result/Option matching works perfectly
- ✅ Variable bindings extract fields correctly
- ✅ Exhaustiveness panic triggers on invalid states
- ✅ Match-as-expression works (returns value)
- ✅ No regressions in existing smoke tests
- ✅ Documentation complete

### Performance Targets:
- Match overhead: <1ms for 2 arms
- Compilation time increase: <10%
- Zero runtime crashes on valid input

---

## 🎯 Next Steps After MVP

### Phase 20.1: @enum Integration
- Implement @enum macro
- Generate EnumSchemaBox metadata
- Connect @match to schema for field names

### Phase 20.2: Static Analysis
- Compile-time exhaustiveness check
- Remove runtime panic for proven-exhaustive matches
- Unreachable pattern warnings

### Phase 20.3: Advanced Patterns
- Literal patterns, guards, nested patterns

---

## 📚 Related Resources

### Internal Documentation
- [Phase 20 VariantBox Design](./DESIGN.md)
- [Result Box Design](./RESULT_BOX_COMPLETE_DESIGN.md)
- [Strategy Decision](../../../current/main/STRATEGY_DECISION.md)

### Existing Implementations
- Result: `apps/lib/boxes/result.hako`
- Option: `apps/lib/boxes/option.hako`
- Loop Macro: `apps/macros/loop_normalize_macro.nyash` (reference)

### Rust Code References
- Parser: `src/parser/statements/mod.rs`
- AST: `src/ast.rs`
- Macros: `src/macro/macro_box.rs`

---

## 🆘 Support

### When Stuck:
1. Check [Quick Start Guide](./@match-quick-start.md) common pitfalls
2. Use `--dump-mir` to verify desugaring
3. Reference [Complete Spec](./@match-macro-implementation-spec.md) algorithms
4. Search existing macro implementations for patterns

### Debug Commands:
```bash
# See parsed AST
./target/release/hako --dump-ast test.hako

# See desugared MIR
./target/release/hako --dump-mir test.hako

# Verbose diagnostics
NYASH_CLI_VERBOSE=1 ./target/release/hako test.hako
```

---

**Status**: ✅ Ready for Implementation
**Estimated Time**: 6 person-days (48 hours)
**Difficulty**: Medium (requires Rust parser + AST knowledge)

**Good Luck! 🚀**
