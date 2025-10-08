# Phase 19: @enum/@match Macro Implementation (Choice A'')

**Status**: CURRENT (2025-10-08)
**Timeline**: 9-14 days (2-3 weeks)
**Approach**: Choice A'' (Macro-Only) - NO VariantBox Core
**Progress**: Day 4/14 (29% complete) ✅ Week 1 blocked on VM bug

---

## 🎯 Overview

Implement pattern matching for Hakorune selfhost compiler using **macro-only approach**.

**Strategic Context**:
- **Choice A'' (Macro-Only)**: Best of both worlds
- **Timeline**: 9-14 days (vs 28-42 days for full implementation)
- **Quality Target**: "ガチガチ大作戦" (rock-solid)
- **User's Intent**: Avoid "中途半端" (half-baked implementation)

**Current Status** (2025-10-08):
- ✅ Day 1: Parser extension (COMPLETED)
- ✅ Day 2: Macro expansion (COMPLETED)
- ✅ Day 3: Test coverage expansion (COMPLETED - 10/10 tests PASS)
- ✅ Day 4: Investigation (COMPLETED - VM bug identified, NOT macro bug)
- ⏸️ Day 5: BLOCKED on VM equals() bug fix

---

## 📋 Scope

### ✅ In Scope

1. **@enum マクロ** (Week 1)
   - Auto-generate constructors + helper functions
   - Syntax: `@enum Result { Ok(value) Err(error) }`
   - Output: Static box with constructors + `is_*()` + `unwrap_*()` methods
   - Internal: Use existing Box types + `_tag` field

2. **@match マクロ** (Week 2-3)
   - Pattern matching syntax
   - Syntax: `@match result { Ok(v) => ... Err(e) => ... }`
   - Desugar to: if-else chains
   - Runtime exhaustiveness check (panic on unknown tag)

3. **Option/Result Rewrite**
   - Rewrite existing Option/Result with @enum
   - Backward compatibility maintained
   - Migration guide for selfhost code

4. **Mini-VM Integration**
   - Apply @match to 3-5 Mini-VM files
   - Replace null checks with @match Option
   - Replace error codes with @match Result

### ❌ Out of Scope (Phase 20+)

- VariantBox Core implementation
- EnumSchemaBox
- SymbolBox (tag optimization)
- Compile-time exhaustiveness checking
- Advanced patterns (guards, literals, nested patterns)

---

## 🏗️ Implementation Plan

### Week 1: @enum Macro (4-5 days)

#### Day 1: Parser Extension ✅ COMPLETED (2025-10-08)

**Goal**: @enum syntax parsing works
**Status**: ✅ All tests passing

**Deliverables**:
- ✅ TokenType::AT added to tokenizer
- ✅ EnumVariant struct + ASTNode::EnumDeclaration
- ✅ enum_parser.rs (150 lines, clean modular design)
- ✅ Integration with parse_declaration_statement()
- ✅ Test execution successful (@enum Result/Option)

**Files Modified**:
- `src/tokenizer/kinds.rs` - AT token
- `src/tokenizer/engine.rs` - @ character recognition
- `src/ast.rs` - EnumVariant struct + EnumDeclaration variant
- `src/ast/utils.rs` - Pattern matching (4 locations)
- `src/parser/declarations/enum_parser.rs` - NEW (150 lines)
- `src/parser/declarations/mod.rs` - Module export
- `src/parser/statements/declarations.rs` - @ dispatch
- `src/parser/statements/mod.rs` - TokenType::AT recognition

**Test Results**:
- ✅ cargo check: PASS
- ✅ cargo build --release: PASS
- ✅ Runtime test: @enum Result/Option parses correctly

**AST Structure Implemented**:
```rust
pub enum ASTNode {
    // ...
    EnumDeclaration {
        name: String,
        variants: Vec<EnumVariant>,
    }
}

pub struct EnumVariant {
    name: String,
    fields: Vec<String>,  // field names
}
```

**Example Syntax Working**:
```hakorune
@enum Result {
    Ok(value)
    Err(error)
}

@enum Option {
    Some(value)
    None
}
```

#### Day 2: Macro Expansion ✅ COMPLETED (2025-10-08)

**Goal**: EnumDeclaration → Box + Static Box generation
**Status**: ✅ All tests passing

**File**: `src/macro/engine.rs` (+323 lines)

**Deliverables**:
- ✅ Program flat_map support for multi-node expansion
- ✅ expand_enum_to_boxes() main expansion function
- ✅ build_enum_birth_method() - null field initialization
- ✅ build_enum_is_method() - is_Ok()/is_Err() predicates
- ✅ build_enum_as_method() - as_Ok()/as_Err() extractors
- ✅ build_enum_constructor() - Ok(v)/Err(e)/None() constructors
- ✅ Integration with MacroEngine::expand_node()
- ✅ Smoke test suite (5/5 tests PASS)

**Test Results**:
- ✅ cargo build --release: PASS
- ✅ Manual test: Result.Ok/Err - PASS
- ✅ Manual test: Option.Some/None (zero-field) - PASS
- ✅ Smoke test: enum_macro_basic.sh (5/5) - PASS

**Actual Time**: ~4 hours

**Desugaring Example**:
```hakorune
// Input
@enum Result {
    Ok(value)
    Err(error)
}

// Output
static box Result {
    // Constructors
    Ok(v) {
        local r = new ResultBox()
        r._tag = "Ok"
        r._value = v
        return r
    }

    Err(e) {
        local r = new ResultBox()
        r._tag = "Err"
        r._error = e
        return r
    }

    // Helpers
    is_ok(r) { return r._tag == "Ok" }
    is_err(r) { return r._tag == "Err" }

    unwrap_ok(r) {
        if r._tag != "Ok" {
            panic("unwrap_ok called on Err")
        }
        return r._value
    }

    unwrap_err(r) {
        if r._tag != "Err" {
            panic("unwrap_err called on Ok")
        }
        return r._error
    }
}
```

#### Day 3: Test Coverage Expansion ✅ COMPLETED (2025-10-08)

**Goal**: Expand test coverage from 5 to 10 patterns
**Status**: ✅ All tests passing

**Files Modified**:
- `tools/smokes/v2/profiles/quick/selfhost/enum_macro_basic.sh` (+133 lines)

**Test Patterns Completed**:
1-5. Basic tests (from Day 2)
6. Multi-field variant (3+ fields)
7. String-heavy variants
8. Tag comparison (is_* with multiple variants)
9. toString() representation
10. Single variant edge case

**Results**:
- ✅ 10/10 tests PASS
- ✅ enum_macro_basic.sh smoke test fully functional
- ⚠️ equals() issue discovered (see Day 4)

**Actual Time**: ~1 hour

#### Day 4: Investigation - equals() Stack Overflow ✅ COMPLETED (2025-10-08)

**Goal**: Fix equals() stack overflow issue
**Status**: ✅ Root cause identified + Solution confirmed - NOT an @enum macro bug

**Investigation Process**:
1. Created minimal test case without @enum → Same crash
2. Implemented manual equals() → Method never called
3. Traced to VM layer → eq_vm() infinite recursion
4. ChatGPT Code: 3 VM-level fix attempts → All failed
5. ChatGPT Pro: Root cause analysis + MIR-level solution → Correct approach

**Key Findings**:
- **Root Cause**: `operator_guard_intercept_entry()` calls `eval_cmp()` before fn context update
- **Bug Type**: Architectural issue - operator guard intercepts ALL boxcalls
- **Scope**: ALL Box types (not @enum-specific)
- **Evidence**: Simple box without @enum crashes identically

**Test Evidence**:
```hakorune
// Test 1: Simple box (no @enum) - CRASHES
box SimpleBox { value }
local s1 = new SimpleBox()
local s2 = new SimpleBox()
s1.value = 42
s2.value = 42
if s1.equals(s2) { }  // STACK OVERFLOW

// Test 2: Manual equals() - CRASHES BEFORE METHOD ENTRY
box SimpleBox {
  value
  equals(other) {
    print("Never printed")  // Method never called
    return me.value == other.value
  }
}
```

### Resolution Path

**Three Failed Attempts** (ChatGPT Code):
1. VM-level fix in `eq_vm()` - Reference equality check
2. VM-level fix v2 - Improved dispatch logic
3. VM-level fix v3 - Method lookup optimization

**Result**: All still caused stack overflow

**Why VM fixes failed**: Operator guard is architectural - intercepts ALL boxcalls for operator checking. VM-level fix would break operator semantics or require complex recursion detection.

**Correct Solution** (ChatGPT Pro): MIR-level transformation

**Approach**: Lower `equals()` calls to `op_eq()` runtime function
```
// Before (high-level MIR)
boxcall recv=v%1 method="equals" args=[v%2] dst=v%3

// After (lowered MIR)
externcall interface="nyrt.ops" method="op_eq" args=[v%1, v%2] dst=v%3
```

**Why this is correct**:
- Separates comparison semantics from method dispatch
- Works for VM, LLVM, and WASM backends
- No VM changes needed (operator guard stays intact)
- Follows existing pattern (`op_to_string`, `op_hash`)

**Implementation Plan** (4 phases, 8-12 hours):
1. Runtime function (1-2h): Add `op_eq()` to extern registry
2. MIR lowering (2-3h): Transform `boxcall equals` → `externcall op_eq`
3. LLVM/WASM support (3-4h): Implement in all backends
4. Integration testing (2-3h): Full @enum test suite

**Conclusion**:
- ✅ @enum macro implementation is CORRECT
- ✅ Bug is in VM operator guard architecture
- ✅ Solution identified: MIR-level lowering (correct architectural fix)
- 📋 Detailed issue doc: `docs/development/issues/equals-stack-overflow.md`

**Timeline Update**:
- Day 4: Investigation complete (2 hours)
- Day 4-5: Implement fix (8-12 hours estimated)
- Day 6: Integration testing (originally Day 5)

**Actual Time**: ~2 hours investigation + 8-12 hours implementation (in progress)

#### Day 5: Selfhost Integration ⏸️ BLOCKED

**Blocking Issue**: VM equals() bug must be fixed first
**Status**: Waiting for VM fix

**Planned Tasks** (when unblocked):
- [ ] Option/Result defined with @enum
- [ ] Full integration test suite
- [ ] Backward compatibility verification

---

### Week 2-3: @match Macro (5-9 days)

#### Day 1-3: Parser Extension (Rust)

**File**: `src/parser/mod.rs`

**Tasks**:
- [ ] Add `@match` syntax parsing
- [ ] Parse pattern syntax: `Tag(bindings)`
- [ ] Create AST node: `ASTNode::MatchExpression`

**AST Structure**:
```rust
pub enum ASTNode {
    // ...
    MatchExpression {
        scrutinee: Box<ASTNode>,
        arms: Vec<MatchArm>,
    }
}

pub struct MatchArm {
    pattern: Pattern,
    body: Box<ASTNode>,
}

pub enum Pattern {
    Variant {
        tag: String,
        bindings: Vec<String>,
    }
}
```

**Example**:
```hakorune
@match result {
    Ok(value) => console.log(value)
    Err(error) => console.error(error)
}
```

#### Day 4-7: Macro Engine (Hakorune)

**File**: `apps/macros/match/match_macro.hako`

**Tasks**:
- [ ] Implement @match desugaring
- [ ] Generate if-else chains
- [ ] Add runtime exhaustiveness check

**Desugaring Example**:
```hakorune
// Input
@match result {
    Ok(value) => {
        console.log("Success: " + value)
        return value
    }
    Err(error) => {
        console.error("Error: " + error)
        return null
    }
}

// Output
local __match_scrutinee = result
if __match_scrutinee._tag == "Ok" {
    local value = __match_scrutinee._value
    console.log("Success: " + value)
    return value
} else if __match_scrutinee._tag == "Err" {
    local error = __match_scrutinee._error
    console.error("Error: " + error)
    return null
} else {
    panic("Non-exhaustive match: unknown tag '" + __match_scrutinee._tag + "'")
}
```

#### Day 8-9: Integration Testing

**Files**:
- `apps/tests/match_patterns.hako` (15 test patterns)
- `apps/selfhost/vm/boxes/*_with_match.hako` (3-5 files)

**Test Patterns**:
1. Simple 2-arm match
2. Multi-arm match (3+ arms)
3. Match with bindings
4. Match with multiple bindings
5. Match with zero-field variant
6. Match with return in arms
7. Match with early return
8. Nested match expressions
9. Match in loop
10. Match in if condition
11. Option pattern matching
12. Result pattern matching
13. Custom enum pattern matching
14. Non-exhaustive match (should panic)
15. All arms covered (no panic)

**Mini-VM Integration**:
- [ ] Rewrite `mini_vm_core.hako` error handling with @match
- [ ] Rewrite `instruction_scanner.hako` null checks with @match
- [ ] Rewrite `op_handlers.hako` result handling with @match

**Success Criteria**:
- [ ] 15/15 tests PASS
- [ ] 3-5 Mini-VM files successfully migrated
- [ ] No manual `if box.is_tag()` in migrated files
- [ ] No null checks in migrated files
- [ ] No error codes (-1/-2/0) in migrated files

---

## ✅ Success Criteria

### Phase 19 Complete = ALL of the following

1. **@enum Macro Functional**
   - [ ] 10/10 @enum tests PASS
   - [ ] Option/Result defined with @enum
   - [ ] Constructors auto-generated
   - [ ] Helpers auto-generated

2. **@match Macro Functional**
   - [ ] 15/15 @match tests PASS
   - [ ] Correct desugaring to if-else
   - [ ] Runtime exhaustiveness check works
   - [ ] Panic on non-exhaustive match

3. **Selfhost Code Application**
   - [ ] 3-5 Mini-VM files migrated to @match
   - [ ] null checks → @match Option
   - [ ] error codes → @match Result
   - [ ] NO manual tag checking (`if box.is_tag()`)

4. **Smoke Tests**
   - [ ] Quick profile ALL PASS
   - [ ] Integration profile ALL PASS
   - [ ] No performance regression

5. **Documentation**
   - [ ] @enum/@match user guide
   - [ ] Migration guide (null → Option, -1/-2 → Result)
   - [ ] Phase 20 migration plan (VariantBox Core)

---

## 🚨 Risks and Mitigation

### 🔴 Critical Risks

1. **@syntax Parser Extension Complexity**
   - Impact: High (Rust parser extension could be difficult)
   - Probability: Medium (Phase 16 has @derive experience)
   - Mitigation:
     - Analyze Phase 16 @derive implementation in detail
     - Start with simple syntax, iterate
     - Fallback: Use `enum!()` macro syntax instead of `@enum`

2. **Macro Expansion Complexity**
   - Impact: High (bugs in desugared code)
   - Probability: Medium (200-350 lines of macro code)
   - Mitigation:
     - Comprehensive test suite (25 patterns)
     - Reference: loop_normalize_macro.nyash (393 lines)
     - ChatGPT Pro code review

### 🟡 Medium Risks

3. **Compatibility with Existing Option/Result**
   - Impact: Medium (migration period conflicts)
   - Probability: Low (can use separate names)
   - Mitigation:
     - Implement as `option_enum.hako` / `result_enum.hako`
     - Gradual migration (3-5 files at a time)
     - Compatibility layer if needed

4. **Performance Degradation**
   - Impact: Medium (desugared code efficiency)
   - Probability: Low (if-else is VM-optimized)
   - Mitigation:
     - Benchmark measurements
     - Optimize desugared code if needed

### 🟢 Minor Risks

5. **No Exhaustiveness Checking**
   - Impact: Low (runtime errors catch issues)
   - Probability: Certain (static checking is Phase 25)
   - Mitigation:
     - Document clearly
     - Tests cover all patterns
     - Add static checking in Phase 25

---

## 🔄 Rollback Plan

### If Phase 19 Fails

**Criteria for Failure**:
- Week 3 end: 50%+ of 25 tests FAIL
- Performance degradation >2x
- Parser extension technically impossible

**Rollback Options**:

1. **Option A: Revert to Strategy C** (Recommended)
   - Implement enum MVP (basic implementation only)
   - Defer @match to Phase 20
   - Duration: +2 weeks (total 5-7 weeks)
   - Rationale: "Half-baked" is bad, but basic enum is better than complete failure

2. **Option B: Revert to Strategy B**
   - Abandon enum completely
   - Prioritize Mini-VM implementation
   - Accept technical debt (quality compromise)

**Recommended**: Option A (Strategy C Rollback)

---

## 📊 Dependencies

### Prerequisites

- ✅ Phase 15.7 completion (selfhost achieved)
- ✅ StringBox/ArrayBox/MapBox stable
- ✅ Macro system basics (Phase 16 provisional)

### Recommended

- StringBox comparison efficiency (SymbolBox deferred to Phase 20+)
- Parser extension experience

### Blocks

- Mini-VM migration (starts Week 4, Step 2)

---

## 📚 References

### Existing Implementations

- **Phase 20 VariantBox Design**: [DESIGN.md](../phase-20-variant-box/DESIGN.md)
- **Result Box Design**: [RESULT_BOX_COMPLETE_DESIGN.md](../phase-20-variant-box/RESULT_BOX_COMPLETE_DESIGN.md)
- **Phase 16 Macro Revolution**: [README.md](../phase-16-macro-revolution/README.md)

### Reference Code

**Existing Macros**:
- `apps/macros/loop_normalize_macro.nyash` (393 lines) - Complex desugaring example
- `apps/macros/if_match_normalize_macro.nyash` (404 lines) - Pattern-matching-like syntax
- `src/macro/pattern.rs` (252 lines) - Rust pattern matching implementation

**Existing Option/Result**:
- `apps/lib/boxes/option.hako` - Existing implementation (commit: e441b2ba)
- `apps/lib/boxes/result.hako` - Existing implementation (commit: e441b2ba)
- `apps/selfhost/vm/boxes/result_box.hako` (34 lines) - Mini-VM implementation

### Strategic Context

- **Strategy Decision**: [STRATEGY_DECISION.md](../../current/main/STRATEGY_DECISION.md)
- **Mini-VM Plan**: [mini_vm_migration_plan.md](../../current/main/mini_vm_migration_plan.md)
- **Choice A'' Analysis**: [CHOICE_A_DOUBLE_PRIME.md](../../current/main/CHOICE_A_DOUBLE_PRIME.md) (to be created)

---

## 🎯 Next Actions

### Day 0: Preparation (0.5 days)

**Environment Check**:
- [ ] Hakorune build verification
- [ ] Baseline smoke tests
- [ ] Phase 16 macro system verification

**Design Review**:
- [ ] Read Phase 20 VariantBox design
- [ ] Study @derive implementation (`src/macro/`)
- [ ] Study loop_normalize_macro.nyash (desugaring patterns)

**Task Preparation**:
- [ ] Create progress tracking file
- [ ] Create lessons learned file
- [ ] Prepare Week 1 task list

### Day 1: Start @enum Implementation

**Task**: Parser extension for @enum
- [ ] Analyze `src/parser/mod.rs`
- [ ] Design `@enum` syntax
- [ ] Add AST node `EnumDeclaration`
- [ ] Create minimal test case

---

## 📝 Deliverables

### Code

**Macros**:
- `apps/macros/enum/enum_macro.hako` (100-150 lines)
- `apps/macros/match/match_macro.hako` (150-200 lines)

**Libraries**:
- `apps/lib/boxes/option_enum.hako` (60-80 lines)
- `apps/lib/boxes/result_enum.hako` (60-80 lines)

**Tests**:
- `apps/tests/enum_basic.hako` (10 patterns)
- `apps/tests/match_patterns.hako` (15 patterns)

**Integration**:
- 3-5 Mini-VM files rewritten with @match

### Documentation

**Guides**:
- `docs/guides/enum-match-guide.md` - @enum/@match user guide
- `docs/guides/pattern-matching-migration.md` - Migration from null/error codes

**Design**:
- `docs/development/roadmap/phases/phase-19-enum-match/README.md` - This file
- `docs/development/roadmap/phases/phase-19-enum-match/LESSONS.md` - Lessons learned

**Planning**:
- `docs/development/roadmap/phases/phase-20-variant-box/README.md` - Updated with migration plan

---

## 🎓 Expected Outcomes

### Technical

- ✅ Complete pattern matching capability
- ✅ Type-safe error handling
- ✅ No "half-baked" period
- ✅ Foundation for Phase 20 VariantBox Core

### Quality

- ✅ Selfhost code: 100% @match (no manual tag checking)
- ✅ Error handling: 100% @match Result (no -1/-2/0)
- ✅ Null handling: 100% @match Option (no null checks)
- ✅ Technical debt: Minimal (predictable desugared code)

### Timeline

- ✅ Pattern matching in 2-3 weeks (vs 4-6 weeks for full implementation)
- ✅ Half the time of Choice A (Full enum)
- ✅ Same quality as Choice A for selfhost purposes

---

**Created**: 2025-10-08
**Status**: ACTIVE
**Owner**: Phase 19 Implementation Team
**User Intent**: "ガチガチ大作戦" (Rock-Solid Selfhosting)
