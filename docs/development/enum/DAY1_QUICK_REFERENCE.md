# Day 1 Quick Reference Card

## 🚀 Quick Start (Copy-Paste Commands)

### Run All Tests
```bash
# Full test suite (automated)
bash tools/test_enum_day1.sh

# Just unit tests (fast)
cargo test --lib enum_parser::tests

# Single test (debug)
cargo test --lib enum_parser::tests::test_basic_two_variant_enum -- --nocapture
```

### Development Loop (TDD)
```bash
# 1. Write test
# 2. Run test (expect RED)
cargo test --lib enum_parser::tests::test_name_here

# 3. Implement feature
# 4. Run test (expect GREEN)
cargo test --lib enum_parser::tests::test_name_here

# 5. Run all tests
cargo test --lib enum_parser::tests
```

### Integration Tests
```bash
# Build project
cargo build --release

# Run single integration test
NYASH_DISABLE_PLUGINS=1 ./target/release/hako apps/tests/enum/test_enum_parse_basic.hako

# Expected output: [ENUM_BASIC] Parser accepted @enum syntax!
```

---

## 📋 Test Checklist

### Unit Tests (10)
- [ ] test_basic_two_variant_enum
- [ ] test_zero_field_variant
- [ ] test_multi_field_variant
- [ ] test_three_plus_variants
- [ ] test_single_variant_enum
- [ ] test_missing_enum_name
- [ ] test_missing_open_brace
- [ ] test_missing_close_brace
- [ ] test_empty_enum
- [ ] test_missing_comma_between_fields

### Integration Tests (3)
- [ ] test_enum_parse_basic.hako
- [ ] test_enum_parse_option.hako
- [ ] test_enum_parse_multi.hako

### E2E Tests (1)
- [ ] test_enum_e2e_minimal.hako

---

## 🎯 Expected Test Output

### Unit Tests (Success)
```
running 10 tests
test enum_parser::tests::test_basic_two_variant_enum ... ok
test enum_parser::tests::test_zero_field_variant ... ok
test enum_parser::tests::test_multi_field_variant ... ok
test enum_parser::tests::test_three_plus_variants ... ok
test enum_parser::tests::test_single_variant_enum ... ok
test enum_parser::tests::test_missing_enum_name ... ok
test enum_parser::tests::test_missing_open_brace ... ok
test enum_parser::tests::test_missing_close_brace ... ok
test enum_parser::tests::test_empty_enum ... ok
test enum_parser::tests::test_missing_comma_between_fields ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Integration Tests (Success)
```bash
$ NYASH_DISABLE_PLUGINS=1 ./target/release/hako apps/tests/enum/test_enum_parse_basic.hako
[ENUM_BASIC] Parser accepted @enum syntax!

$ NYASH_DISABLE_PLUGINS=1 ./target/release/hako apps/tests/enum/test_enum_parse_option.hako
[ENUM_OPTION] Unit variant 'None' parsed successfully!

$ NYASH_DISABLE_PLUGINS=1 ./target/release/hako apps/tests/enum/test_enum_parse_multi.hako
[ENUM_MULTI] Complex enum parsed successfully!
[ENUM_MULTI] 4 variants: Idle, Running(1), Paused(2), Stopped
```

### Full Test Suite (Success)
```
========================================
Phase 1: Unit Tests (Rust)
========================================
Building project...
✓ PASS: Cargo build

Running Rust unit tests...
✓ PASS: Unit tests (10 tests)

========================================
Phase 2: Integration Tests (Hakorune)
========================================
Test 1: Basic @enum parsing...
✓ PASS: Integration Test 1: Basic parsing
Test 2: Option-like @enum (unit variant)...
✓ PASS: Integration Test 2: Option-like enum
Test 3: Multi-variant @enum...
✓ PASS: Integration Test 3: Multi-variant enum

========================================
Phase 3: End-to-End Test
========================================
E2E Test: Full pipeline...
✓ PASS: E2E Test: Full pipeline

========================================
Test Summary
========================================
Total tests: 14
Passed: 14
Failed: 0

========================================
ALL TESTS PASSED! 🎉
Day 1 parser implementation complete!
========================================
```

---

## 🐛 Common Errors & Fixes

### Error: "expected enum name"
**Cause**: Missing identifier after `@enum`
**Fix**: Ensure `parse_enum_declaration` expects identifier token

### Error: "expected '{'"
**Cause**: Missing opening brace
**Fix**: Check `expect_token(TokenType::LBrace)` call

### Error: "enum must have at least one variant"
**Cause**: Empty enum `@enum X {}`
**Fix**: This is correct behavior! Enums need ≥1 variant

### Error: Test compiles but always fails
**Cause**: Parser state issue (cursor position)
**Fix**: Ensure tests call `parser.cursor.advance()` to skip `@enum` token

### Error: Integration test crashes
**Cause**: `@enum` token not recognized by main parser
**Fix**: Wire `parse_enum_declaration` into main parser loop

---

## 📊 Coverage Targets

| Component | Target | Critical |
|-----------|--------|----------|
| parse_enum_declaration | 100% | All branches |
| parse_enum_variant | 100% | Unit & tuple variants |
| parse_enum_field_list | 90% | Edge cases ok |
| Error handling | 100% | All errors tested |
| **Overall** | **>90%** | **Pass gate** |

---

## 🔧 Debug Commands

### Show parser state
```rust
// Add to parser code temporarily
eprintln!("Current token: {:?}", parser.cursor.peek_token());
eprintln!("Position: {}", parser.cursor.position());
```

### Dump AST structure
```rust
// In test
let ast = parse_enum_declaration(&mut parser).unwrap();
eprintln!("{:#?}", ast);
```

### Run with verbose cargo
```bash
cargo test --lib enum_parser::tests -- --nocapture --test-threads=1
```

---

## ⏱️ Time Estimates

| Task | Optimistic | Realistic | Pessimistic |
|------|------------|-----------|-------------|
| Setup & first test | 15 min | 30 min | 45 min |
| Core parsing (3 tests) | 30 min | 60 min | 90 min |
| Error handling (5 tests) | 30 min | 60 min | 90 min |
| Integration tests | 30 min | 75 min | 120 min |
| E2E test | 15 min | 30 min | 60 min |
| Test automation | 30 min | 60 min | 90 min |
| Coverage & polish | 30 min | 60 min | 90 min |
| Docs & commit | 30 min | 60 min | 90 min |
| **Total** | **3.5 hours** | **8 hours** | **12 hours** |

**Plan for**: 8 hours (realistic)
**Buffer**: 2 hours for unexpected issues

---

## ✅ Success Gates

### Morning Gate (by lunch)
- [ ] 5+ unit tests passing
- [ ] Core parsing logic works
- [ ] No compiler errors

**If not met**: Skip coverage, focus on core functionality

### Afternoon Gate (by end of day)
- [ ] All 10 unit tests passing
- [ ] 3 integration tests working
- [ ] E2E test working

**If not met**: Defer documentation, ship working parser

### Final Gate (commit ready)
- [ ] All tests GREEN
- [ ] No warnings
- [ ] Coverage >85% (>90% ideal)

**If not met**: Don't commit! Debug until passing.

---

## 📞 Help Resources

### Stuck on parsing?
- Check: `src/parser/declarations/box_def/mod.rs` (similar structure)
- Read: Parser cursor API docs

### Stuck on tests?
- Check: Existing test patterns in `src/parser/`
- Read: Rust testing guide

### Stuck on errors?
- Add: Debug prints
- Run: Single test with `--nocapture`
- Check: Parser error message format

### Completely blocked?
1. Document the blocker
2. Create minimal reproduction
3. Ask for help (don't waste >30 min stuck)

---

## 🎯 Day 1 Definition of Done

**@enum syntax parsing works** means:

1. ✅ Parser recognizes `@enum` keyword
2. ✅ Parser builds `EnumDeclaration` AST
3. ✅ Handles all syntax variants (unit, tuple, multi-field)
4. ✅ Reports clear errors for invalid syntax
5. ✅ >90% test coverage
6. ✅ No compiler warnings
7. ✅ Automated tests pass

**Does NOT include** (Day 2+):
- ❌ Code generation (box expansion)
- ❌ Runtime behavior
- ❌ Integration with match expressions

---

**Ready to start?** → [Day 1 TDD Workflow](DAY1_TDD_WORKFLOW.md)
