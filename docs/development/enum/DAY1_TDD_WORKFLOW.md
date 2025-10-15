# Day 1 TDD Workflow Timeline

## 🎯 Goal
**@enum syntax parsing works** - Complete parser implementation using Test-Driven Development

## ⏱️ Time Budget
**8 hours** (Morning: 4h, Afternoon: 4h)

---

## 🌅 Morning Session (09:00 - 13:00)

### 09:00 - 09:30: Setup & First Test (30 min)

**Objective**: Get RED test running

```bash
# 1. Create enum_parser.rs skeleton (5 min)
# Already done! See src/parser/declarations/enum_parser.rs

# 2. Write first test (5 min)
# test_basic_two_variant_enum

# 3. Run test - expect RED (5 min)
cargo test enum_parser::tests::test_basic_two_variant_enum
# Expected: Compilation error (function doesn't exist)

# 4. Implement minimal parse_enum_declaration (10 min)
# - Parse name only
# - Return dummy EnumDeclaration

# 5. Run test - expect partial GREEN (5 min)
cargo test enum_parser::tests::test_basic_two_variant_enum
```

**Success Criteria**:
- [ ] Test compiles
- [ ] Test runs (even if fails)
- [ ] Clear error message shows what's missing

---

### 09:30 - 10:30: Core Parsing Logic (60 min)

**Objective**: Parse variants

**TDD Cycle** (repeat 3 times, 20 min each):

```bash
# Cycle 1: Zero-field variant (20 min)
# 1. Write test_zero_field_variant (3 min)
# 2. Run → RED (2 min)
# 3. Implement parse_enum_variant for unit variants (10 min)
# 4. Run → GREEN (2 min)
# 5. Refactor (3 min)

# Cycle 2: Single-field variant (20 min)
# 1. Write test_multi_field_variant (3 min)
# 2. Run → RED (2 min)
# 3. Implement parse_enum_field_list (10 min)
# 4. Run → GREEN (2 min)
# 5. Refactor (3 min)

# Cycle 3: Three+ variants (20 min)
# 1. Write test_three_plus_variants (3 min)
# 2. Run → RED (2 min)
# 3. Fix loop logic in parse_enum_declaration (10 min)
# 4. Run → GREEN (2 min)
# 5. Refactor (3 min)
```

**Success Criteria**:
- [ ] 4 positive tests passing
- [ ] No compiler warnings
- [ ] Code coverage >60%

---

### 10:30 - 10:45: Coffee Break ☕

**Mental Reset**: Step away from code, review progress

---

### 10:45 - 11:45: Error Handling (60 min)

**Objective**: Write negative tests (errors)

**TDD Cycle** (5 tests × 12 min each):

```bash
# Test 1: Missing enum name (12 min)
# 1. Write test_missing_enum_name (2 min)
# 2. Run → Might pass already or fail (2 min)
# 3. Fix error handling if needed (5 min)
# 4. Verify error message quality (3 min)

# Test 2: Missing open brace (12 min)
# Same pattern...

# Test 3: Missing close brace (12 min)
# Same pattern...

# Test 4: Empty enum (12 min)
# Add validation: must have ≥1 variant

# Test 5: Missing comma (12 min)
# Field list parsing error
```

**Success Criteria**:
- [ ] 5 negative tests passing
- [ ] All error messages clear & user-friendly
- [ ] Code coverage >80%

---

### 11:45 - 13:00: Integration Tests (75 min)

**Objective**: Verify full program parsing

```bash
# 1. Review integration test files (10 min)
cat apps/tests/enum/test_enum_parse_basic.hako
cat apps/tests/enum/test_enum_parse_option.hako
cat apps/tests/enum/test_enum_parse_multi.hako

# 2. Wire @enum into main parser (30 min)
# Edit src/parser/mod.rs to recognize @enum token
# Add call to parse_enum_declaration

# 3. Build project (5 min)
cargo build --release

# 4. Run integration tests manually (15 min)
NYASH_DISABLE_PLUGINS=1 ./target/release/hako apps/tests/enum/test_enum_parse_basic.hako
NYASH_DISABLE_PLUGINS=1 ./target/release/hako apps/tests/enum/test_enum_parse_option.hako
NYASH_DISABLE_PLUGINS=1 ./target/release/hako apps/tests/enum/test_enum_parse_multi.hako

# 5. Debug any issues (15 min)
# Use --dump-ast if available
# Check parser error messages
```

**Success Criteria**:
- [ ] All 3 integration tests parse without errors
- [ ] Programs print expected output
- [ ] No crashes or panics

---

## 🌆 Afternoon Session (14:00 - 18:00)

### 14:00 - 14:30: E2E Test (30 min)

**Objective**: Verify full pipeline

```bash
# 1. Review E2E test (5 min)
cat apps/tests/enum/test_enum_e2e_minimal.hako

# 2. Run E2E test (5 min)
NYASH_DISABLE_PLUGINS=1 ./target/release/hako apps/tests/enum/test_enum_e2e_minimal.hako

# 3. Debug if needed (10 min)

# 4. Verify output (5 min)
# Should print: [ENUM_E2E] Parser didn't crash!

# 5. Document expected behavior (5 min)
# Day 1: Parsing only
# Day 2+: Enum expansion
```

**Success Criteria**:
- [ ] E2E test runs successfully
- [ ] Clear output message
- [ ] No errors or warnings

---

### 14:30 - 15:30: Test Automation (60 min)

**Objective**: Create automated test suite

```bash
# 1. Review test script (10 min)
cat tools/test_enum_day1.sh

# 2. Run full test suite (10 min)
bash tools/test_enum_day1.sh

# 3. Fix any failures (20 min)
# Debug unit tests
# Debug integration tests

# 4. Iterate until all GREEN (15 min)

# 5. Document test execution (5 min)
# Add to README or docs
```

**Success Criteria**:
- [ ] All tests pass in automated suite
- [ ] Test execution time <5s
- [ ] Clear pass/fail reporting

---

### 15:30 - 16:00: Code Coverage (30 min)

**Objective**: Measure & improve coverage

```bash
# 1. Install coverage tool (5 min)
cargo install cargo-tarpaulin

# 2. Run coverage (10 min)
cargo tarpaulin --out Html --output-dir coverage -- enum_parser::tests

# 3. Open coverage report (5 min)
open coverage/index.html  # or xdg-open on Linux

# 4. Identify uncovered lines (5 min)

# 5. Add tests for gaps (5 min)
# Focus on edge cases
```

**Success Criteria**:
- [ ] Coverage >90%
- [ ] All critical branches tested
- [ ] No untested error paths

---

### 16:00 - 16:15: Coffee Break ☕

---

### 16:15 - 17:00: Refinement (45 min)

**Objective**: Polish & cleanup

```bash
# 1. Code review (15 min)
# - Check for dead code
# - Verify naming conventions
# - Ensure consistent style

# 2. Add documentation (15 min)
# - Function doc comments
# - Module-level docs
# - Examples in comments

# 3. Run rustfmt & clippy (10 min)
cargo fmt
cargo clippy -- -D warnings

# 4. Final test run (5 min)
bash tools/test_enum_day1.sh
```

**Success Criteria**:
- [ ] No clippy warnings
- [ ] All code formatted
- [ ] Doc comments on public items

---

### 17:00 - 18:00: Documentation & Commit (60 min)

**Objective**: Finalize Day 1 work

```bash
# 1. Write Day 1 report (20 min)
# Document:
# - What was implemented
# - Test results
# - Known limitations
# - Next steps (Day 2)

# 2. Create commit (15 min)
git add src/parser/declarations/enum_parser.rs
git add apps/tests/enum/*.hako
git add tools/test_enum_day1.sh
git commit -m "feat(parser): Day 1 @enum parsing implementation

- Add EnumDeclaration/EnumVariant/EnumField AST structures
- Implement parse_enum_declaration with full error handling
- Add 10 unit tests (positive + negative)
- Add 3 integration tests + 1 E2E test
- Test coverage: >90%

Day 1 Goal: @enum syntax parsing works ✓"

# 3. Run final verification (10 min)
bash tools/test_enum_day1.sh
cargo test

# 4. Update project docs (15 min)
# - Update CURRENT_TASK.md
# - Update roadmap
# - Log in CLAUDE.md
```

**Success Criteria**:
- [ ] Clean git commit
- [ ] All tests passing
- [ ] Documentation updated
- [ ] Ready for Day 2

---

## ✅ Day 1 Success Checklist

### Morning Success (by 13:00)
- [ ] 5/10 unit tests passing
- [ ] Core parsing logic implemented
- [ ] 3/10 tests GREEN

### Afternoon Success (by 18:00)
- [ ] 10/10 unit tests passing
- [ ] 3/3 integration tests working
- [ ] 1/1 E2E test working
- [ ] Code coverage >90%
- [ ] No compiler warnings
- [ ] Clean commit created

### Overall Day 1 Complete
- [ ] All tests GREEN
- [ ] Automated test suite working
- [ ] Documentation complete
- [ ] Code reviewed
- [ ] Ready to start Day 2 (enum expansion)

---

## 🚨 Troubleshooting

### If tests fail:
1. **Read error message carefully**
2. **Isolate the failure** (run single test)
3. **Add debug prints** if needed
4. **Check assumptions** (token types, parser state)
5. **Ask for help** if stuck >20 min

### If behind schedule:
1. **Skip coverage step** (do it later)
2. **Reduce integration tests** (1 instead of 3)
3. **Focus on positive tests** (skip some negative tests)
4. **Defer documentation** (minimal commit message ok)

### If ahead of schedule:
1. **Add more edge case tests**
2. **Improve error messages**
3. **Add property-based tests** (quickcheck)
4. **Start Day 2 planning**

---

## 📊 Expected Timeline Reality

**Optimistic** (everything works first try): 6 hours
**Realistic** (normal debugging): 8 hours
**Pessimistic** (major issues): 10 hours

**Buffer**: 2 hours for unexpected problems

---

## 🎓 Learning Notes

### What went well:
- (Fill in after Day 1)

### What was difficult:
- (Fill in after Day 1)

### What to improve for Day 2:
- (Fill in after Day 1)

---

**Next**: [Day 2: Enum Expansion](DAY2_EXPANSION_PLAN.md)
