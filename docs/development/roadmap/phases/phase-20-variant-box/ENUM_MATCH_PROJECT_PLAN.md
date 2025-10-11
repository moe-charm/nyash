# @enum/@match Macro Implementation - Complete Project Plan

**✅ Status**: **Phase 19 完了（2025-10-08）** - 実装完了記録

**Created**: 2025-10-08
**Strategy**: ~~Choice A'' (Macro-Only Approach)~~ **実施: 正規構文として実装**
**Timeline**: ~~12-17 days~~ **✅ 完了（2025-10-08）**
**Quality Target**: ~~"ガチガチ大作戦"~~ **✅ 達成**

---

## 🎉 実装完了状況（2025-10-08）

**Phase 19として実装完了**:
- ✅ **@enum macro**: `src/parser/declarations/enum_parser.rs` (147行)
- ✅ **Macro expansion**: `src/macro/engine.rs` (expand_enum_to_boxes関数)
- ✅ **match式**: `src/parser/expr/match_expr.rs` (392行)
- ✅ **Literal/Type patterns + Guards**: 完全対応
- ⚠️ **実装方式**: @matchマクロではなく、正規match**式**として実装

**実装ファイル**:
- Parser: `src/parser/declarations/enum_parser.rs` (147行)
- Parser: `src/parser/expr/match_expr.rs` (392行)
- Macro: `src/macro/engine.rs`
- AST: `ASTNode::Match` variant

---

## 📋 Executive Summary

This document provides a comprehensive, executable project plan for implementing @enum/@match macros in Hakorune. The implementation follows the "macro-only" approach (Choice A''), expanding to existing MIR16 instructions without modifying the instruction set.

**✅ 実際の成果（Phase 19完了）**:
- ~~Days 1-5: @enum macro implementation~~ ✅ **完了（2025-10-08）**
- ~~Days 6-11: @match macro implementation~~ ✅ **完了（正規match式として）**
- ~~Days 12-14: Selfhost application~~ ⚠️ **部分完了（ResultBox Phase 1）**
- ~~Days 15-17: Buffer~~ **不要（順調に完了）**

**Success Criteria**: ~~All tests PASS~~ ✅ **達成**

---

## 🗓️ 1. Detailed Daily Task Breakdown (14-17 Days)

### **Week 1: @enum Macro Implementation (Days 1-5)**

#### **Day 1: Parser @enum Syntax + AST Nodes**
**Estimated Time**: 6-8 hours
**Objective**: Add @enum syntax recognition to parser

**Tasks**:
1. **Add @enum token** (30 min)
   - File: `src/parser/mod.rs`
   - Add `@` prefix recognition for macro syntax
   - Test: `@enum` is recognized as valid token

2. **Parse enum definition structure** (3 hours)
   - Grammar: `@enum Name { Variant1(field1, field2) Variant2 ... }`
   - Extract: enum name, variant names, field names
   - Handle: zero-field variants (e.g., `None`)

3. **Create EnumDefNode AST** (2 hours)
   - New AST type: `ASTNode::EnumDeclaration { name, variants, span }`
   - Variant structure: `{ variant_name, field_names }`
   - Ensure span information for error messages

4. **Unit tests for parser** (2 hours)
   - Test: Parse basic `@enum Option { Some(value) None }`
   - Test: Parse multi-field variant `@enum Result { Ok(value) Err(error) }`
   - Test: Syntax error detection (missing braces, duplicate variants)

**Success Criteria**:
- ✅ Parser recognizes `@enum` syntax without errors
- ✅ AST correctly represents 3+ different enum definitions
- ✅ Syntax errors produce helpful error messages
- ✅ All unit tests PASS

**Deliverables**:
- `src/parser/mod.rs` changes (~100-150 lines)
- Unit test file: `src/parser/tests/enum_syntax.rs` (~80 lines)

---

#### **Day 2: Macro Engine Integration**
**Estimated Time**: 6-8 hours
**Objective**: Hook @enum into macro expansion system

**Tasks**:
1. **Register @enum handler** (2 hours)
   - File: `src/macro/engine.rs`
   - Add `expand_enum()` function
   - Hook into `MacroEngine::expand()`

2. **Generate BoxDef for VariantBox** (2 hours)
   - Generate: `static box EnumName { ... }`
   - Add: `_schema: EnumSchemaBox` field
   - Add: `_init_schema()` method

3. **Generate birth() method** (1 hour)
   - Generate schema registration code
   - Call `register_variant()` for each variant

4. **Basic expansion test** (2 hours)
   - Test: `@enum Option { Some(value) None }` → static box structure
   - Verify: Generated AST is valid Hakorune code
   - Test: Macro expansion trace (NYASH_MACRO_TRACE=1)

**Success Criteria**:
- ✅ @enum expands to valid static box structure
- ✅ Generated code compiles without errors
- ✅ Macro trace shows correct expansion

**Deliverables**:
- `src/macro/engine.rs` changes (~120 lines)
- `apps/macros/enum/enum_macro.hako` (50 lines, scaffolding)

---

#### **Day 3: Constructor + Helper Generation**
**Estimated Time**: 6-8 hours
**Objective**: Auto-generate variant constructors and helpers

**Tasks**:
1. **Generate static variant constructors** (3 hours)
   - For each variant, generate: `VariantName(field1, field2) { ... }`
   - Code: `local v = new VariantBox("VariantName"); v.fields.push(field1); ...`
   - Handle: Zero-field variants (e.g., `None() { return new VariantBox("None") }`)

2. **Generate is_VariantName() helpers** (2 hours)
   - For each variant, generate: `is_variant(v: VariantBox) { return v.is_tag("Variant") }`
   - Return: BoolBox (1/0)

3. **Generate as_VariantName() helpers** (2 hours)
   - For each variant, generate extraction method
   - Code: `if !me.is_variant(v) { panic("Expected Variant, got " + v.tag) }; return v.field(0)`
   - Multi-field: Return ArrayBox with all fields

4. **Integration test** (1 hour)
   - Test: Create instance via `Option.Some(42)`
   - Test: Check `Option.is_some(inst)` returns 1
   - Test: Extract value via `Option.as_some(inst)` returns 42
   - Test: Wrong variant access panics correctly

**Success Criteria**:
- ✅ All helper methods generate correctly
- ✅ Constructors create valid VariantBox instances
- ✅ Type mismatch panics with helpful messages
- ✅ Integration test PASS

**Deliverables**:
- `apps/macros/enum/enum_macro.hako` completion (~150 lines total)
- Test: `apps/tests/enum_basic.hako` (~40 lines)

---

#### **Day 4: Test Suite (10 Patterns)**
**Estimated Time**: 6-8 hours
**Objective**: Comprehensive test coverage

**Test Patterns** (all must PASS):
1. **Basic Option**: `Some(42)` + `None`
2. **Basic Result**: `Ok("success")` + `Err("error")`
3. **Multi-field variant**: `Variant(a, b, c)` with 3 fields
4. **Zero-field variant**: `EmptyVariant` (no fields)
5. **Nested enum**: `Option(Result(42))`
6. **Type mismatch panic**: `as_some()` on `None` → panic
7. **Schema validation**: Unknown variant → error
8. **Field access bounds**: `field(99)` → panic
9. **Debug string**: `Some(42).debug()` → `"Some(42)"`
10. **Round-trip**: Create → extract → compare

**Tasks**:
1. **Write test file** (4 hours)
   - File: `apps/tests/enum_comprehensive.hako`
   - Each pattern as separate `test_*()` function
   - Use test harness (NYASH_TEST_RUN=1)

2. **Debug failures** (3 hours)
   - Run: `NYASH_TEST_RUN=1 ./target/release/hako apps/tests/enum_comprehensive.hako`
   - Fix: Code generation issues
   - Fix: Edge cases (empty fields, etc.)

3. **Document behavior** (1 hour)
   - File: `apps/macros/enum/README.md`
   - Document: Syntax, generated code, limitations

**Success Criteria**:
- ✅ 10/10 test patterns PASS
- ✅ Zero false positives (no spurious failures)
- ✅ Error messages are helpful

**Deliverables**:
- `apps/tests/enum_comprehensive.hako` (~200 lines)
- `apps/macros/enum/README.md` (~100 lines)

---

#### **Day 5: Smoke Tests + Integration**
**Estimated Time**: 4-6 hours
**Objective**: Add to smoke test suite and verify no regressions

**Tasks**:
1. **Add smoke test script** (2 hours)
   - File: `tools/smokes/v2/profiles/quick/core/enum_basic_vm.sh`
   - Test: Run enum_comprehensive.hako via VM backend
   - Expected: All tests PASS, exit code 0

2. **Test with existing Option/Result** (2 hours)
   - File: `apps/lib/boxes/option.hako` (existing)
   - Verify: Can coexist with manual OptionBox
   - Test: No naming conflicts

3. **Performance validation** (1 hour)
   - Benchmark: Constructor overhead vs manual `new VariantBox()`
   - Acceptable: < 5% slowdown
   - If slower: Profile and optimize

4. **Run full smoke test suite** (1 hour)
   - Run: `tools/smokes/v2/run.sh --profile quick`
   - Verify: No regressions (all existing tests still PASS)

**Success Criteria**:
- ✅ Smoke tests PASS (enum_basic_vm.sh)
- ✅ No regressions in existing tests
- ✅ Performance < 5% overhead

**Deliverables**:
- `tools/smokes/v2/profiles/quick/core/enum_basic_vm.sh`
- Performance report (inline in commit message)

---

### **Week 2: @match Macro Implementation (Days 6-11)**

#### **Day 6: Parser @match Syntax**
**Estimated Time**: 6-8 hours
**Objective**: Parse pattern matching syntax

**Tasks**:
1. **Add @match token** (30 min)
   - File: `src/parser/mod.rs`
   - Recognize `@match expr { ... }`

2. **Parse pattern syntax** (3 hours)
   - Grammar: `VariantName(var1, var2) => { body }`
   - Extract: variant name, variable bindings, body statements
   - Handle: Single-statement vs multi-statement bodies

3. **Parse match arms** (2 hours)
   - Structure: `{ arm1 arm2 ... }`
   - Separate: Pattern from arrow `=>` from body
   - Error: Duplicate patterns

4. **AST node creation** (2 hours)
   - New type: `ASTNode::MatchExpression { scrutinee, arms, span }`
   - Arm structure: `{ pattern: Pattern, body: Vec<ASTNode> }`
   - Pattern structure: `{ variant_name, bindings: Vec<String> }`

**Success Criteria**:
- ✅ Parser recognizes `@match` syntax
- ✅ Patterns parsed correctly (variant + bindings)
- ✅ Multi-arm match parses correctly

**Deliverables**:
- `src/parser/mod.rs` changes (~80-120 lines)
- Unit tests: `src/parser/tests/match_syntax.rs` (~60 lines)

---

#### **Day 7: Pattern Analysis**
**Estimated Time**: 6-8 hours
**Objective**: Extract variant names and variable bindings

**Tasks**:
1. **Extract variant names** (2 hours)
   - From: `Ok(v) => ...`
   - Extract: `"Ok"` as tag name
   - Validate: Variant name is identifier

2. **Extract variable bindings** (3 hours)
   - From: `Ok(v)` → binding: `v`
   - From: `Add(lhs, rhs)` → bindings: `["lhs", "rhs"]`
   - Handle: Zero bindings (e.g., `None => ...`)

3. **Validate pattern structure** (2 hours)
   - Check: Binding count matches variant arity (deferred to runtime for now)
   - Check: No duplicate variable names in same pattern
   - Error: Invalid pattern syntax

4. **Test pattern → AST conversion** (1 hour)
   - Test: `Ok(v)` → `{ variant: "Ok", bindings: ["v"] }`
   - Test: `None` → `{ variant: "None", bindings: [] }`

**Success Criteria**:
- ✅ Pattern → AST conversion works for 5+ patterns
- ✅ Validation catches errors (duplicates, invalid syntax)

**Deliverables**:
- `src/macro/pattern_match.rs` (new file, ~150 lines)
- Unit tests embedded in file

---

#### **Day 8: If-Else Desugaring**
**Estimated Time**: 6-8 hours
**Objective**: Generate if-else-if chain from match arms

**Tasks**:
1. **Generate tag comparison chain** (3 hours)
   - First arm: `if scrutinee.is_tag("Ok") { ... }`
   - Middle arms: `else if scrutinee.is_tag("Err") { ... }`
   - Last arm: `else { panic("Non-exhaustive match") }`

2. **Generate local variable bindings** (3 hours)
   - Code: `local v = scrutinee.field(0)`
   - Multi-field: `local lhs = scrutinee.field(0); local rhs = scrutinee.field(1)`
   - Insert: At start of each arm body

3. **Handle multi-statement blocks** (1 hour)
   - Preserve: Return values, side effects
   - Test: Match as expression (returns value)

4. **Basic desugaring test** (1 hour)
   - Input: `@match r { Ok(v) => v  Err(e) => null }`
   - Output: if-else-if chain (verify manually)
   - Test: Execution returns correct value

**Success Criteria**:
- ✅ Desugaring generates valid Hakorune code
- ✅ Variable bindings work correctly
- ✅ Multi-statement blocks preserved

**Deliverables**:
- `apps/macros/match/match_macro.hako` (~100 lines)
- Test: `apps/tests/match_desugar.hako` (~50 lines)

---

#### **Day 9: Exhaustiveness Check**
**Estimated Time**: 4-6 hours
**Objective**: Add runtime exhaustiveness checking

**Tasks**:
1. **Add else clause with panic** (2 hours)
   - Generate: `else { panic("Non-exhaustive match on EnumName: unexpected variant " + scrutinee.tag) }`
   - Include: Enum name and actual tag in error message

2. **Generate proper error messages** (2 hours)
   - Format: `"Non-exhaustive match on Result: expected Ok|Err, got UnknownVariant"`
   - Test: Error message clarity

3. **Test exhaustiveness** (1 hour)
   - Test: Missing variant → panic at runtime
   - Test: Error message shows expected variants

**Success Criteria**:
- ✅ Missing patterns trigger panic
- ✅ Error messages are helpful (show expected variants)

**Deliverables**:
- `apps/macros/match/match_macro.hako` updates (~50 lines)
- Test: `apps/tests/match_exhaustiveness.hako` (~30 lines)

---

#### **Day 10: Test Suite (15 Patterns)**
**Estimated Time**: 8 hours
**Objective**: Comprehensive pattern matching test coverage

**Test Patterns** (all must PASS):
1. **Basic match**: `Ok(v) => v`
2. **Multi-arm**: `Ok(_) / Err(_)`
3. **Variable binding**: Extract and use value
4. **Zero-field pattern**: `None => 0`
5. **Multi-field pattern**: `Add(lhs, rhs) => lhs + rhs`
6. **Match as expression**: Return value from match
7. **Match in function**: Use as return statement
8. **Nested match**: Match inside match arm
9. **Side effects**: Print in match arm
10. **Non-exhaustive panic**: Missing variant triggers error
11. **Wrong variant**: Verify tag check works
12. **Multi-statement arm**: Multiple lines in arm body
13. **Return from arm**: Early return works correctly
14. **Wildcard future**: Prepare for `_ => ...` (optional)
15. **Integration**: Option + Result together

**Tasks**:
1. **Write test file** (4 hours)
   - File: `apps/tests/match_comprehensive.hako`
   - Each pattern as `test_*()` function

2. **Debug failures** (3 hours)
   - Run tests, fix code generation issues
   - Fix: Edge cases (zero fields, nested match)

3. **Document usage** (1 hour)
   - File: `apps/macros/match/README.md`
   - Examples, syntax, limitations

**Success Criteria**:
- ✅ 15/15 test patterns PASS
- ✅ Exhaustiveness check works correctly

**Deliverables**:
- `apps/tests/match_comprehensive.hako` (~300 lines)
- `apps/macros/match/README.md` (~150 lines)

---

#### **Day 11: Edge Cases + Smoke Tests**
**Estimated Time**: 4-6 hours
**Objective**: Handle edge cases and add smoke tests

**Tasks**:
1. **Match as expression** (2 hours)
   - Test: `local x = @match r { Ok(v) => v  Err(_) => 0 }`
   - Verify: Return value propagates correctly

2. **Return values** (1 hour)
   - Test: `return @match r { ... }`
   - Verify: Early return from function works

3. **Nested patterns** (1 hour)
   - Test: `@match outer { Some(inner) => @match inner { ... } }`
   - Verify: Bindings don't conflict

4. **Add smoke test** (1 hour)
   - File: `tools/smokes/v2/profiles/quick/core/match_basic_vm.sh`
   - Run: `match_comprehensive.hako`
   - Expected: All tests PASS

**Success Criteria**:
- ✅ All edge cases PASS
- ✅ Smoke tests PASS
- ✅ No regressions

**Deliverables**:
- `tools/smokes/v2/profiles/quick/core/match_basic_vm.sh`
- Edge case tests in `match_comprehensive.hako`

---

### **Week 3: Selfhost Application (Days 12-14)**

#### **Day 12: Rewrite Option/Result with @enum**
**Estimated Time**: 4 hours
**Objective**: Convert manual OptionBox/ResultBox to @enum definitions

**Tasks**:
1. **Rewrite Option** (1 hour)
   - File: `apps/lib/boxes/option_v2.hako` (new file, preserve old)
   - Code: `@enum Option { Some(value) None }`
   - Test: Backward compatibility with existing API

2. **Rewrite Result** (1 hour)
   - File: `apps/lib/boxes/result_v2.hako` (new file, preserve old)
   - Code: `@enum Result { Ok(value) Err(error) }`
   - Test: Backward compatibility

3. **Update module aliases** (1 hour)
   - File: `hako.toml`
   - Add: `option_v2 = "apps/lib/boxes/option_v2.hako"`
   - Add: `result_v2 = "apps/lib/boxes/result_v2.hako"`

4. **Test existing usages** (1 hour)
   - Verify: 5 existing ResultBox usages still work
   - Test: Can switch by changing import

**Success Criteria**:
- ✅ @enum versions functionally equivalent
- ✅ All existing usages still work
- ✅ Can toggle between old/new via imports

**Deliverables**:
- `apps/lib/boxes/option_v2.hako` (~30 lines)
- `apps/lib/boxes/result_v2.hako` (~30 lines)
- `hako.toml` updates

---

#### **Day 13: Apply @match to Mini-VM**
**Estimated Time**: 6-8 hours
**Objective**: Replace if-else chains with @match in Mini-VM

**Tasks**:
1. **Identify error handling locations** (2 hours)
   - File: `selfhost/vm/boxes/result_box.hako` usages
   - Find: if-else chains checking `.is_ok()`
   - Candidates: 3-5 locations in Mini-VM

2. **Rewrite with @match** (3 hours)
   - Before: `if result.is_ok() { ... } else { ... }`
   - After: `@match result { Ok(v) => ... Err(e) => ... }`
   - Preserve: Exact same logic

3. **Test Mini-VM functionality** (2 hours)
   - Run: `apps/selfhost-runtime/runner.nyash` (Mini-VM test)
   - Verify: All existing functionality works
   - Verify: No behavioral changes

4. **Code review** (1 hour)
   - Check: No regressions
   - Check: Code is more readable
   - Document: Locations changed

**Success Criteria**:
- ✅ Mini-VM tests PASS (no behavioral changes)
- ✅ Code is more readable
- ✅ At least 1 function uses @match

**Deliverables**:
- Mini-VM file updates (3-5 files, ~50 line changes total)
- Test run output (paste in commit message)

---

#### **Day 14: Integration Testing + Documentation**
**Estimated Time**: 4-6 hours
**Objective**: Verify full integration and document changes

**Tasks**:
1. **Run full smoke test suite** (2 hours)
   - Run: `tools/smokes/v2/run.sh --profile quick`
   - Expected: ALL tests PASS
   - Fix: Any regressions found

2. **Update documentation** (2 hours)
   - File: `docs/reference/language/quick-reference.md`
   - Add: @enum/@match syntax section
   - Add: Examples, best practices

3. **Verify no regressions** (1 hour)
   - Run: Integration smoke tests
   - Compare: Before/after performance
   - Verify: < 5% performance impact

4. **Write migration guide** (1 hour)
   - File: `docs/guides/enum-match-migration.md`
   - Content: How to migrate from manual if-else to @match
   - Examples: Option, Result, custom enums

**Success Criteria**:
- ✅ ALL smoke tests PASS (quick + integration)
- ✅ No performance regressions (< 5% slowdown)
- ✅ Documentation complete and accurate

**Deliverables**:
- Smoke test run output (all PASS)
- `docs/reference/language/quick-reference.md` updates
- `docs/guides/enum-match-migration.md` (new file, ~200 lines)

---

### **Buffer Days (Days 15-17): Debugging & Performance Tuning**

**Purpose**: Handle unexpected issues, optimize hot paths, polish documentation

**Typical Activities**:
- Debug edge cases discovered during integration testing
- Performance profiling and optimization
- Additional test coverage
- Documentation polish
- Code review and cleanup

---

## 🚨 2. Risk Analysis with Mitigation

### **High Risk (Probability > 50%)**

#### **Risk #1: Parser Conflicts with Existing Syntax**
**Description**: `@enum/@match` tokens may conflict with identifiers or existing `@` usage
**Impact**: ❌ Critical - Cannot parse, implementation blocked
**Probability**: 60%

**Mitigation**:
1. **Pre-check**: Search codebase for existing `@` usage (Day 1, 30 min)
2. **Fallback syntax**: If conflicts, use `enum!` macro syntax instead of `@enum`
3. **Testing**: Parse test files early to detect conflicts

**Contingency Plan**:
- If `@` prefix conflicts → Use `!` suffix (Rust-style): `enum!`, `match!`
- Time impact: +1 day (update parser, tests)

---

#### **Risk #2: Variable Binding Complexity**
**Description**: Field name resolution from pattern may fail in edge cases
**Impact**: ❌ Critical - Incorrect code generation, wrong variable values
**Probability**: 55%

**Mitigation**:
1. **Extensive unit tests**: Cover all binding patterns (Day 7, 3 hours)
2. **Verbose error messages**: Show what went wrong in code generation
3. **Fallback**: Restrict to single-field variants in MVP if multi-field fails

**Contingency Plan**:
- If multi-field bindings fail → MVP supports only single-field variants
- Time impact: +2 days (simplify implementation, update docs)

---

#### **Risk #3: Exhaustiveness Check False Positives**
**Description**: Valid code may trigger "non-exhaustive" panic incorrectly
**Impact**: ⚠️ High - Cannot use @match reliably
**Probability**: 50%

**Mitigation**:
1. **Conservative checking**: Only panic if truly non-exhaustive
2. **Wildcard pattern**: Support `_ => ...` for catch-all (optional Day 10)
3. **Env variable**: `NYASH_MATCH_ALLOW_PARTIAL=1` disables exhaustiveness check

**Contingency Plan**:
- If false positives occur → Make exhaustiveness check optional (env var)
- Time impact: +1 day (add env check, update docs)

---

### **Medium Risk (Probability 20-50%)**

#### **Risk #4: Performance Degradation (If-Else Chain)**
**Description**: Tag comparison chain may be slower than expected
**Impact**: ⚠️ Medium - Mini-VM slowdown
**Probability**: 40%

**Mitigation**:
1. **Benchmark early**: Compare before/after on Day 5
2. **Optimize hot paths**: Use direct field access instead of helpers
3. **Fallback**: Keep critical paths as manual if-else

**Contingency Plan**:
- If > 10% slowdown → Keep hot paths as manual if-else, use @match elsewhere
- Time impact: +1 day (identify hot paths, selective application)

---

#### **Risk #5: Debugging Difficulty (Expanded Code Invisible)**
**Description**: Errors in macro-expanded code hard to trace to source
**Impact**: ⚠️ Medium - Development slowdown
**Probability**: 35%

**Mitigation**:
1. **Verbose expansion mode**: `NYASH_MACRO_VERBOSE=1` shows expanded code
2. **Preserve spans**: Include source location in generated AST
3. **Manual expansion**: Document how to manually expand for debugging

**Contingency Plan**:
- If debugging is too hard → Provide macro expansion tool (separate script)
- Time impact: +2 days (implement expansion tool)

---

#### **Risk #6: Integration with Existing Option/Result**
**Description**: Breaking changes to 5 existing usages
**Impact**: ⚠️ Medium - More rewrite work than expected
**Probability**: 30%

**Mitigation**:
1. **Backward compatibility layer**: Keep old OptionBox/ResultBox as-is
2. **Gradual migration**: Only new code uses @enum versions
3. **Parallel imports**: `option.hako` (old) + `option_v2.hako` (new)

**Contingency Plan**:
- If migration is too complex → Keep old implementation indefinitely
- Time impact: +0 days (just don't migrate old code)

---

### **Low Risk (Probability < 20%)**

#### **Risk #7: Macro Expansion Order Issues**
**Description**: `@match` expands before `@enum`, unknown variant errors
**Impact**: ⚠️ Low - Compilation errors
**Probability**: 15%

**Mitigation**:
1. **Document expansion order**: @enum first, @match second
2. **Test inter-macro usage**: Day 10, pattern #8 (nested match)
3. **Explicit order control**: If needed, add phase hints to macro engine

**Contingency Plan**:
- If order issues occur → Document "define @enum before using @match"
- Time impact: +0.5 days (document workaround)

---

#### **Risk #8: MIR16 Limitations**
**Description**: Cannot express pattern matching in MIR16 (very unlikely)
**Impact**: ❌ Critical - Would require MIR changes (violates Frozen principle)
**Probability**: 5%

**Mitigation**:
1. **Proof of concept**: All patterns desugar to if/else (already proven in design)
2. **Early test**: Day 2, verify basic desugaring generates valid MIR

**Contingency Plan**:
- If MIR16 insufficient → Escalate to architecture review
- Time impact: +5-10 days (would require MIR changes, unlikely)

---

## ✅ 3. Success Criteria (Acceptance Tests)

### **@enum Macro Success Criteria**

#### **Functional Requirements**
- ✅ **CR-E1**: Parse at least 3 different enum definitions without error
  - Test: `@enum Option { Some(value) None }`
  - Test: `@enum Result { Ok(value) Err(error) }`
  - Test: `@enum Shape { Circle(radius) Rectangle(width, height) }`

- ✅ **CR-E2**: Generate correct box structure with _tag field
  - Verify: `static box EnumName { _schema: EnumSchemaBox ... }`
  - Verify: Schema registration code present

- ✅ **CR-E3**: Generate working constructors (can create instances)
  - Test: `local opt = Option.Some(42)`
  - Test: `opt.tag == "Some"`
  - Test: `opt.fields.len() == 1`

- ✅ **CR-E4**: Generate working is_* helpers (correct true/false)
  - Test: `Option.is_some(opt) == 1`
  - Test: `Option.is_none(opt) == 0`

- ✅ **CR-E5**: Generate working as_* helpers (correct panic on mismatch)
  - Test: `Option.as_some(opt) == 42`
  - Test: `Option.as_none(opt)` → panic with helpful message

#### **Quality Requirements**
- ✅ **CR-E6**: Pass 10/10 test patterns (Day 4)
- ✅ **CR-E7**: Pass smoke tests with 0 regressions (Day 5)
- ✅ **CR-E8**: Performance < 5% overhead vs manual VariantBox (Day 5)

---

### **@match Macro Success Criteria**

#### **Functional Requirements**
- ✅ **CR-M1**: Parse at least 5 different match expressions without error
  - Test: Basic match (Ok/Err)
  - Test: Multi-arm match (3+ arms)
  - Test: Zero-field pattern (None)
  - Test: Multi-field pattern (Add(lhs, rhs))
  - Test: Nested match

- ✅ **CR-M2**: Desugar to correct if-else chain
  - Verify: Generated code structure matches expected pattern
  - Verify: Tag comparisons use `is_tag()` method

- ✅ **CR-M3**: Extract and bind variables correctly
  - Test: `Ok(v) => print(v)` → `v` contains field(0)
  - Test: `Add(lhs, rhs)` → `lhs` = field(0), `rhs` = field(1)

- ✅ **CR-M4**: Exhaustiveness check catches non-exhaustive matches
  - Test: Match with missing variant → panic at runtime
  - Test: Error message shows expected variants

- ✅ **CR-M5**: Return values work (match as expression)
  - Test: `local x = @match r { Ok(v) => v Err(_) => 0 }`
  - Verify: `x` contains correct value

#### **Quality Requirements**
- ✅ **CR-M6**: Pass 15/15 test patterns (Day 10)
- ✅ **CR-M7**: Pass smoke tests with 0 regressions (Day 11)
- ✅ **CR-M8**: Edge cases handled (nested, return, expression) (Day 11)

---

### **Selfhost Application Success Criteria**

#### **Functional Requirements**
- ✅ **CR-S1**: Option/Result rewritten with @enum
  - Verify: `option_v2.hako` and `result_v2.hako` exist
  - Verify: Functionally equivalent to old versions

- ✅ **CR-S2**: All 5 existing usages still work
  - Test: Can toggle between old/new via imports
  - Test: No breaking changes

- ✅ **CR-S3**: At least 1 Mini-VM function uses @match
  - Verify: Code is more readable than if-else
  - Verify: Behavior unchanged

- ✅ **CR-S4**: Full smoke test suite PASS
  - Run: `tools/smokes/v2/run.sh --profile quick`
  - Expected: ALL tests PASS (0 failures)

- ✅ **CR-S5**: No performance regression (< 5% slowdown)
  - Benchmark: Before/after comparison
  - Verify: Mini-VM execution time < 5% increase

---

### **"ガチガチ" Quality Criteria (Rock-Solid)**

- ✅ **CR-Q1**: 100% pattern consistency
  - All error handling uses @match (no mixing of paradigms)
  - Exception: Hot paths can remain manual if needed

- ✅ **CR-Q2**: Zero "中途半端" (No half-measures)
  - Either full @match support or explicit documentation of limitations
  - No hidden edge cases

- ✅ **CR-Q3**: Clear documentation
  - When to use @match vs if-else (guidelines)
  - Migration guide for existing code
  - Examples for all common patterns

- ✅ **CR-Q4**: Comprehensive error messages
  - Panic messages show: expected variants, actual tag, enum name
  - Syntax errors show: line/column, helpful suggestions

---

## 🔄 4. Rollback Plan

### **Option 1: Minimal Rollback (If @match Fails)**

**Scenario**: @enum macro works, but @match macro fails or takes > 11 days

**Action**:
- Keep @enum macro (already working)
- Skip @match macro implementation
- Use `is_*`/`as_*` helpers + manual if-else

**Example**:
```hakorune
// Before (planned):
@match result {
    Ok(v) => print(v)
    Err(e) => print("Error: " + e)
}

// After rollback:
if Result.is_ok(result) {
    local v = Result.as_ok(result)
    print(v)
} else if Result.is_err(result) {
    local e = Result.as_err(result)
    print("Error: " + e)
}
```

**Benefits**:
- ✅ Still better than plain if-else (type-safe constructors)
- ✅ Still get enum-style definitions
- ✅ Can add @match later (non-blocking)

**Drawbacks**:
- ⚠️ No pattern matching sugar
- ⚠️ More verbose code

**Time Saved**: 4-6 days (Days 6-11 of @match implementation)
**Quality Loss**: Medium (syntax less clean, but still functional)

**Decision Trigger**:
- If Day 11 arrives and < 12/15 test patterns PASS → Rollback to Option 1

---

### **Option 2: Full Rollback to Choice B (If Both Fail)**

**Scenario**: Both @enum and @match fail, or block Mini-VM migration

**Action**:
- Keep existing `OptionBox`/`ResultBox` (34 lines total)
- Postpone enum/match to Phase 20 (post-selfhost)
- Focus on Mini-VM migration (current priority)

**Benefits**:
- ✅ No risk to current development
- ✅ Can revisit with more time/resources
- ✅ Mini-VM migration not blocked

**Drawbacks**:
- ❌ No enum/match benefits
- ❌ Lost development time (Days 1-14)

**Time Saved**: 9-14 days (full implementation time)
**Quality Loss**: High (but recoverable later, not permanent)

**Decision Trigger**:
- If both macros fail (< 8/10 @enum tests or < 12/15 @match tests)
- OR if blocking critical Mini-VM work
- OR if takes > 17 days total

---

### **Rollback Decision Matrix**

| Scenario | Action | Time Lost | Quality Impact | Reversible? |
|----------|--------|-----------|----------------|-------------|
| @enum works, @match fails | Option 1: Keep @enum only | 4-6 days | Medium | ✅ Yes (add @match later) |
| Both fail or block critical work | Option 2: Full rollback | 9-14 days | High | ✅ Yes (retry in Phase 20) |
| Performance unacceptable | Option 1: Keep @enum, selective @match | 2-3 days | Low | ✅ Yes (optimize later) |

---

## 📦 5. Dependencies and Blockers

### **Prerequisites (Must Complete First)**

✅ **DONE: Option/Result Boxes Implemented**
- Status: ✅ Complete (commit `e441b2ba`)
- Files: `apps/lib/boxes/option.hako`, `apps/lib/boxes/result.hako`
- Impact: Can start immediately, no blockers

✅ **DONE: Module Alias System Working**
- Status: ✅ Complete (hako.toml v2)
- Files: `hako.toml`, module resolution system
- Impact: Can use `using option_v2 as Option` seamlessly

✅ **DONE: Smoke Test Infrastructure Ready**
- Status: ✅ Complete
- Files: `tools/smokes/v2/` framework
- Impact: Can add new tests immediately

---

### **Parallel Work (Can Do Simultaneously)**

**Documentation Updates** (not blocking):
- Update language reference as features complete
- Can be done in parallel with implementation
- Deadline: Day 14 (before final integration)

**Mini-VM Planning** (separate track):
- Identify @match application points
- Can be done during Week 2 (Days 6-11)
- Feeds into Day 13 (Mini-VM integration)

---

### **Blockers (Cannot Start Until)**

**@match Cannot Start Until @enum Complete**:
- Reason: Need enum definitions to exist for pattern matching
- Blocker: Day 5 completion (Day 6 cannot start before Day 5)
- Mitigation: Ensure Day 1-5 schedule is strict

**Selfhost Application Cannot Start Until Both Macros Work**:
- Reason: Need both @enum and @match for full integration
- Blocker: Day 11 completion (Day 12 cannot start before Day 11)
- Mitigation: Use buffer days 15-17 if Week 2 runs over

---

### **External Dependencies (None Identified)**

No external dependencies or library upgrades required.

---

## 💼 6. Resource Requirements

### **Human Time**

**Daily Focused Work**: 6-8 hours/day
**Total Time**: 84-136 hours (12-17 days × 8 hours)

**Breakdown**:
- Week 1 (@enum): 30-40 hours
- Week 2 (@match): 38-50 hours
- Week 3 (Selfhost): 16-22 hours
- Buffer: 24 hours (3 days × 8 hours)

**Recommended Schedule**:
- Days 1-5: Focus on @enum (no distractions)
- Days 6-11: Focus on @match (parallel doc updates OK)
- Days 12-14: Integration and testing (requires full attention)
- Days 15-17: Buffer (flexible, can pause if ahead of schedule)

---

### **Testing Infrastructure**

✅ **Available**:
- Unit test framework (Rust + Hakorune test harness)
- Smoke test suite (`tools/smokes/v2/`)
- Test runner with filter support (`NYASH_TEST_FILTER`)

⚠️ **Need to Add**:
- Benchmark tools for performance validation (Day 5, 1 hour)
- Macro expansion trace tool (NYASH_MACRO_VERBOSE already exists)

---

### **Documentation**

**Required Documents**:
1. **Implementation Specs** (in progress, reference: phase-20-variant-box/DESIGN.md)
2. **User Guide** (@enum/@match syntax) - Create on Days 4, 10, 14
3. **Migration Guide** (from old Option/Result) - Create on Day 14

**Documentation Time Budget**:
- Day 4: 1 hour (@enum README)
- Day 10: 1 hour (@match README)
- Day 14: 2 hours (migration guide + language reference update)
- Total: 4 hours (included in daily estimates)

---

## 🚦 7. Quality Gates (Go/No-Go Decisions)

### **Gate 1: End of Day 5 (After @enum)**

**Criteria**:
- ✅ **Go**: 10/10 tests PASS, smoke tests clean
- ❌ **No-Go**: < 8/10 tests PASS

**Actions if No-Go**:
1. **Assess failure type**:
   - Parser issues → +1 day debugging, continue
   - Code generation issues → +1 day debugging, continue
   - Fundamental design flaw → Escalate to architecture review

2. **Decision Point**:
   - If fixable in +1-2 days → Continue with extended timeline
   - If requires > 3 days → Rollback to Option 2 (Full Rollback)

**Stakeholder Notification**:
- Update CURRENT_TASK.md with status
- If No-Go, document issues and mitigation plan

---

### **Gate 2: End of Day 11 (After @match)**

**Criteria**:
- ✅ **Go**: 15/15 tests PASS, exhaustiveness works
- ⚠️ **Caution**: 12-14/15 tests PASS (minor issues)
- ❌ **No-Go**: < 12/15 tests PASS

**Actions if Caution**:
1. **Assess missing functionality**:
   - If edge cases only → Document limitations, proceed
   - If core functionality broken → +1-2 days debugging

2. **Decision Point**:
   - Proceed to Day 12 with documented limitations
   - Add buffer days for polish

**Actions if No-Go**:
1. **Fallback to Option 1** (Keep @enum, skip @match)
2. **Pivot to manual if-else** (use is_*/as_* helpers)
3. **Document decision** in CURRENT_TASK.md

**Stakeholder Notification**:
- If Caution: Note limitations in commit message
- If No-Go: Announce rollback to Option 1

---

### **Gate 3: End of Day 14 (After Selfhost Integration)**

**Criteria**:
- ✅ **Go**: ALL smoke tests PASS, no regressions
- ⚠️ **Caution**: 1-2 test failures (non-critical)
- ❌ **No-Go**: > 2 test failures or critical regression

**Actions if Caution**:
1. **Use buffer days 15-17** to fix failures
2. **If performance regression > 5%**:
   - Profile hot paths
   - Optimize or revert problematic changes

**Actions if No-Go**:
1. **Rollback to Option 2** (Full Rollback)
2. **Preserve @enum/@match in feature branch**
3. **Return to Phase 15.7 work**

**Stakeholder Notification**:
- If Go: Announce successful completion, prepare for commit
- If Caution: Note issues, extend timeline to Day 17
- If No-Go: Announce rollback, document lessons learned

---

### **Final Gate: End of Day 17 (After Buffer)**

**Criteria**:
- ✅ **Ship**: All tests PASS, documentation complete, performance OK
- ❌ **Abort**: Still failing tests or unresolved issues

**Actions if Ship**:
1. **Create commit** (follow git safety protocol)
2. **Update CURRENT_TASK.md** with completion
3. **Announce success** in CLAUDE.md

**Actions if Abort**:
1. **Full Rollback** to Option 2
2. **Post-mortem analysis** (what went wrong?)
3. **Document for Phase 20** (retry later)

---

## 📢 8. Communication Plan

### **Daily Updates**

**End of Each Day**:
1. **Progress Report** (1-2 sentences)
   - Tasks completed today
   - Issues found (if any)
   - Tomorrow's plan

2. **Update CURRENT_TASK.md**:
   - Move completed tasks to "RECENTLY DONE"
   - Update "NOW" section with current status
   - Add any blockers to "Risks / Blockers"

**Format**:
```markdown
### 2025-10-XX: @enum/@match Day X

**Completed**:
- ✅ Task 1
- ✅ Task 2

**Issues**:
- ⚠️ Issue description (mitigation: ...)

**Tomorrow**:
- [ ] Task 3
- [ ] Task 4
```

---

### **Milestone Reports**

**End of Week 1 (Day 5): @enum Completion Report**

**Content**:
- Test results (10/10 PASS or X/10 PASS with explanation)
- Code statistics (lines added, files changed)
- Performance impact (< 5% or actual %)
- Known limitations (if any)
- Next steps (proceed to @match or rollback)

**Location**: CURRENT_TASK.md + commit message

---

**End of Week 2 (Day 11): @match Completion Report**

**Content**:
- Test results (15/15 PASS or X/15 PASS with explanation)
- Code statistics
- Exhaustiveness check behavior
- Known limitations
- Next steps (proceed to selfhost integration or rollback)

**Location**: CURRENT_TASK.md + commit message

---

**End of Week 3 (Day 14): Final Integration Report**

**Content**:
- Smoke test results (ALL PASS or X failures)
- Performance comparison (before/after)
- Files migrated (list of changed files)
- Documentation updates
- Success criteria met (checklist)
- Final decision (ship, extend, or rollback)

**Location**: CURRENT_TASK.md + CLAUDE.md + commit message

---

### **Issue Escalation**

**Blocker Found**:
1. **Immediate report** in CURRENT_TASK.md (add to "Risks / Blockers")
2. **Mitigation plan** (what are we trying?)
3. **Timeline impact** (how many days delay?)
4. **Decision deadline** (when do we escalate to rollback?)

**Risk Materializes**:
1. **Update risk section** with actual impact
2. **Reassess timeline** (can we still meet Day 17?)
3. **Consider fallback options** (Option 1 or Option 2?)
4. **Document lessons learned** (for future phases)

---

## 📈 9. Success Metrics (Quantitative)

### **Code Metrics**

**Lines of Code**:
- @enum macro implementation: ~150 lines (apps/macros/enum/enum_macro.hako)
- @match macro implementation: ~200 lines (apps/macros/match/match_macro.hako)
- Parser changes: ~230 lines (src/parser/mod.rs)
- Tests: ~530 lines (apps/tests/enum_comprehensive.hako + match_comprehensive.hako)
- Total new code: ~1,110 lines

**Lines Deleted** (eventually):
- Manual if-else chains in Mini-VM: ~50 lines (replaced by @match)
- Net change: +1,060 lines

**Files Changed**:
- New files: 8 (macro implementations, tests, docs)
- Modified files: 10-15 (parser, Mini-VM, hako.toml, docs)

---

### **Test Coverage**

**Test Patterns**:
- @enum: 10 patterns (Day 4)
- @match: 15 patterns (Day 10)
- Integration: 5 scenarios (Day 14)
- Total: 30 test patterns

**Smoke Tests**:
- New smoke tests: 2 (enum_basic_vm.sh, match_basic_vm.sh)
- Existing smoke tests: ~60 (must all still PASS)
- Total smoke coverage: 62 tests

---

### **Performance Metrics**

**Acceptable Targets**:
- @enum constructor overhead: < 5% vs manual `new VariantBox()`
- @match vs if-else: < 5% slower
- Overall Mini-VM: < 5% slowdown
- Smoke test suite: < 10% longer runtime

**Measurement Method**:
- Day 5: Benchmark @enum constructors (10,000 iterations)
- Day 11: Benchmark @match vs if-else (10,000 iterations)
- Day 14: Full smoke test suite runtime comparison

---

### **Quality Metrics**

**Completeness**:
- All planned features implemented: 100%
- Known limitations documented: 100%
- Edge cases tested: 100%

**Reliability**:
- Test success rate: 100% (all tests PASS)
- Regression rate: 0% (no existing tests break)
- Panic rate: 0% (no unexpected panics)

**Maintainability**:
- Documentation coverage: 100% (README for each macro)
- Code comments: All public APIs documented
- Migration guide: Complete with examples

---

## 📝 10. Post-Implementation Checklist

### **Before Commit**

- [ ] All tests PASS (10/10 @enum, 15/15 @match, ALL smoke tests)
- [ ] No regressions (existing tests still PASS)
- [ ] Performance < 5% impact
- [ ] Documentation complete (READMEs, migration guide, language reference)
- [ ] Code review (self-review for clarity and consistency)
- [ ] Update CURRENT_TASK.md (move to "RECENTLY DONE")
- [ ] Update CLAUDE.md (add to "Recent Accomplishments")

---

### **Commit Message Template**

```
feat(macro): Add @enum/@match macros (Phase 20.7-20.8)

Implements enum-style sum types via VariantBox with pattern matching sugar.

**@enum Macro**:
- Syntax: @enum Name { Variant1(field) Variant2 }
- Generates: static box with constructors, is_*/as_* helpers
- Tests: 10/10 PASS

**@match Macro**:
- Syntax: @match expr { Pattern(var) => body }
- Desugars to: if-else chain with tag checks
- Exhaustiveness: Runtime panic on non-exhaustive match
- Tests: 15/15 PASS

**Selfhost Application**:
- Option/Result rewritten with @enum (option_v2.hako, result_v2.hako)
- Mini-VM: 1 function migrated to @match
- Backward compatibility: Old implementations preserved

**Test Results**:
- New tests: 30 patterns (all PASS)
- Smoke tests: 62 total (all PASS)
- Performance: +X.X% (< 5% target met)

**Files Changed**:
- Parser: src/parser/mod.rs (+230 lines)
- Macros: apps/macros/enum/*, apps/macros/match/* (+350 lines)
- Tests: apps/tests/enum_*, apps/tests/match_* (+530 lines)
- Docs: 3 new files (+450 lines)
- Total: +1,560 lines

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

---

### **After Commit**

- [ ] Update docs/development/roadmap/phases/phase-20-variant-box/README.md (mark as COMPLETE)
- [ ] Create follow-up tasks (if any limitations identified)
- [ ] Archive project plan (move to docs/development/archive/)
- [ ] Celebrate! 🎉

---

## 🎓 11. Lessons Learned (Post-Mortem Template)

**To be filled after completion (Day 17)**

### **What Went Well**

- (To be filled)

### **What Could Be Improved**

- (To be filled)

### **Unexpected Challenges**

- (To be filled)

### **Recommendations for Future Phases**

- (To be filled)

---

## 📚 12. References

### **Design Documents**
- [DESIGN.md](./DESIGN.md) - VariantBox complete design
- [RESULT_BOX_COMPLETE_DESIGN.md](./RESULT_BOX_COMPLETE_DESIGN.md) - Result<T,E> design
- [README.md](./README.md) - Phase 20 overview

### **Implementation References**
- Existing macros: `apps/macros/loop_normalize_macro.nyash` (393 lines)
- Pattern matching: `src/macro/pattern.rs` (252 lines)
- Option/Result: `apps/lib/boxes/option.hako`, `apps/lib/boxes/result.hako`

### **Related Phases**
- Phase 16: Macro Revolution (@derive implementation)
- Phase 20.0-20.5: Macro Full Features (foundation)
- Phase 25: Type System (static exhaustiveness checks)

---

**Document Status**: READY FOR EXECUTION
**Approval**: Pending Phase 15.7 completion
**Next Action**: Wait for Phase 15.7, then begin Day 1 (@enum parser)
