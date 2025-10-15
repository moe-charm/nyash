# @match Macro Quick Start Guide

**✅ Status**: **Phase 19 完了（2025-10-08）** - match式 完全実装済み（@matchマクロではなく、正規match構文として）

**For**: ~~Developers implementing @match macro~~ **実装完了記録**
**Time**: ~~6 days~~ **✅ 完了（2025-10-08）**
**Difficulty**: Medium
**Prerequisites**: Understanding of Rust parser, AST, macros

---

## 🎉 実装完了状況（2025-10-08）

- ✅ **match式 Parser**: `src/parser/expr/match_expr.rs` (392行)
- ✅ **Literal patterns**: 整数・文字列・bool対応
- ✅ **Type patterns**: Box型パターン対応
- ✅ **Guards**: `if` ガード条件対応
- ⚠️ **注意**: `@match`マクロ**ではなく**、正規`match`構文として実装

**使用例** (実装済み):
```hakorune
match result {
  Ok(value) if value > 0 => print("Positive")
  Ok(value) => print("Non-positive")
  Err(e) => print("Error: " + e)
}
```

---

## 📋 Quick Links

- **Complete Spec**: [@match-macro-implementation-spec.md](./@match-macro-implementation-spec.md)
- **VariantBox Design**: [DESIGN.md](./DESIGN.md)
- **Existing Boxes**:
  - [apps/lib/boxes/result.hako](../../../../apps/lib/boxes/result.hako)
  - [apps/lib/boxes/option.hako](../../../../apps/lib/boxes/option.hako)

---

## 🎯 What You're Building

Transform this:
```hakorune
@match result {
  Ok(v) => print("Success: " + v)
  Err(e) => print("Error: " + e)
}
```

Into this:
```hakorune
if result._ok == 1 {
  local v
  v = result._val
  print("Success: " + v)
} else if result._ok == 0 {
  local e
  e = result._err
  print("Error: " + e)
} else {
  print("[PANIC] non-exhaustive match on Result: unknown state")
}
```

---

## 🚀 6-Day Implementation Plan

### Day 1: Parser Foundation
**Goal**: Parse @match syntax without errors

**Tasks**:
1. Add `MatchPattern` to `ASTNode` enum
2. Add `Pattern` and `MatchArm` structs
3. Add `FatArrow` and `Underscore` tokens
4. Implement `parse_match_pattern()` method

**Files to Modify**:
- `src/ast.rs` - AST nodes
- `src/tokenizer.rs` - Token types
- `src/parser/statements/mod.rs` - Parser methods

**Test**:
```bash
# Should parse without error
echo '@match x { Ok(v) => v }' | ./target/release/hako --dump-ast -
```

### Day 2: Tag Comparison Generation
**Goal**: Generate if-conditions for patterns

**Tasks**:
1. Create field mapping registry
2. Implement tag comparison code
3. Generate if-condition AST nodes

**Hardcoded Mappings**:
```rust
// Result
Ok  → _ok==1, field: _val
Err → _ok==0, field: _err

// Option
Some → _is_some==1, field: _value
None → _is_some==0

// Generic VariantBox
Variant → _tag=="Variant", fields: _0, _1, ...
```

**Test**:
```hakorune
@match result { Ok(v) => v }
// Should generate: if result._ok == 1 { ... }
```

### Day 3: Variable Binding
**Goal**: Extract pattern variables to local declarations

**Tasks**:
1. Implement binding extraction
2. Generate `local` declarations
3. Generate field access code
4. Insert bindings at top of arm

**Test**:
```hakorune
@match point { Cartesian(x, y) => print(x + y) }
// Should generate:
// local x
// local y
// x = point._x
// y = point._y
```

### Day 4: Exhaustiveness Check
**Goal**: Add safety panic for unknown variants

**Tasks**:
1. Generate catch-all else clause
2. Generate panic message
3. Handle match-as-expression case

**Test**:
```hakorune
@match result { Ok(v) => v }
// Should generate final else:
// else { print("[PANIC] non-exhaustive match...") }
```

### Day 5: Basic Tests (8/15)
**Goal**: Verify core functionality

**Test Files**:
- Test 1-5: Basic matching
- Test 6-8: Variable binding

**Run**:
```bash
./target/release/hako apps/tests/match/test_result_basic.hako
./target/release/hako apps/tests/match/test_option_basic.hako
# ... etc
```

### Day 6: Edge Cases (7/15)
**Goal**: Handle corner cases

**Test Files**:
- Test 9-11: Exhaustiveness
- Test 12-15: Edge cases (loops, early return, etc.)

**Final Verification**:
```bash
# Run all match tests
bash tools/test_match_patterns.sh

# Run smoke tests
tools/smokes/v2/run.sh --profile quick
```

---

## 🔑 Key Implementation Points

### 1. AST Node Structure

```rust
// src/ast.rs
pub enum ASTNode {
    // ... existing variants ...

    MatchPattern {
        scrutinee: Box<ASTNode>,
        arms: Vec<MatchArm>,
        span: Span,
    },
}

pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<ASTNode>,
    pub span: Span,
}

pub enum Pattern {
    Variant {
        name: String,
        bindings: Vec<String>,
        span: Span,
    },
    Wildcard {
        span: Span,
    },
}
```

### 2. Desugaring Algorithm (Simplified)

```rust
fn desugar_match(scrutinee: &ASTNode, arms: &[MatchArm]) -> ASTNode {
    let mut if_chain = None;

    // Build if/else chain (reverse order)
    for arm in arms.iter().rev() {
        let condition = generate_tag_check(scrutinee, &arm.pattern);
        let bindings = generate_bindings(scrutinee, &arm.pattern);

        let mut then_body = bindings;
        then_body.extend(arm.body.clone());

        if_chain = Some(ASTNode::If {
            condition: Box::new(condition),
            then_body,
            else_body: if_chain.map(|node| vec![node]),
            span: Span::unknown(),
        });
    }

    // Add exhaustiveness panic
    let panic = ASTNode::Print {
        expression: Box::new(ASTNode::Literal {
            value: LiteralValue::String("[PANIC] non-exhaustive match".to_string()),
            span: Span::unknown(),
        }),
        span: Span::unknown(),
    };

    if let Some(ASTNode::If { else_body, .. }) = &mut if_chain {
        *else_body = Some(vec![panic]);
    }

    if_chain.unwrap()
}
```

### 3. Field Mapping Function

```rust
fn get_field_mapping(variant: &str, binding_index: usize) -> String {
    match variant {
        "Ok" => "_val".to_string(),
        "Err" => "_err".to_string(),
        "Some" => "_value".to_string(),
        "None" => panic!("None has no fields"),
        _ => format!("_{}", binding_index),  // Generic: _0, _1, ...
    }
}

fn get_tag_check(variant: &str) -> (&str, &str) {
    match variant {
        "Ok" => ("_ok", "1"),
        "Err" => ("_ok", "0"),
        "Some" => ("_is_some", "1"),
        "None" => ("_is_some", "0"),
        _ => ("_tag", variant),  // Generic: _tag == "Variant"
    }
}
```

---

## 🧪 Testing Strategy

### Smoke Test Template

```hakorune
// apps/tests/match/test_NAME.hako
using "../../lib/boxes/result.hako"

static box Main {
  main() {
    // Setup
    local r = Result.ok(42)

    // Test @match
    local result = @match r {
      Ok(v) => v * 2
      Err(e) => -1
    }

    // Assert
    if result != 84 {
      print("FAIL: Expected 84, got " + result)
      return 1
    }

    print("PASS: test_NAME")
    return 0
  }
}
```

### Running Tests

```bash
# Single test
./target/release/hako apps/tests/match/test_result_basic.hako

# With MIR dump (verify desugaring)
./target/release/hako --dump-mir apps/tests/match/test_result_basic.hako

# All tests
for f in apps/tests/match/*.hako; do
  echo "Testing $f..."
  ./target/release/hako "$f" || echo "FAILED: $f"
done
```

---

## 🚨 Common Pitfalls

### 1. Parser Order
**Problem**: `=>` tokenized as `>` + `=`
**Solution**: Tokenize `=>` as single `FatArrow` token first

### 2. Variable Scope
**Problem**: Pattern variables leak to outer scope
**Solution**: Each if-arm creates new scope (via if block)

### 3. Expression vs Statement
**Problem**: Match used as expression (returns value)
**Solution**: Introduce temporary variable, assign in each arm

### 4. Exhaustiveness False Positives
**Problem**: All cases covered, but panic still added
**Solution**: Phase 1: Always add panic (safe). Phase 2: Static analysis

### 5. Field Name Conflicts
**Problem**: Multiple variants with same field name
**Solution**: Use variant-specific field names (`_val` vs `_err`)

---

## 📊 Progress Checklist

### Day 1: Parser
- [ ] `MatchPattern` AST node added
- [ ] `Pattern` enum added
- [ ] `FatArrow` token added
- [ ] `parse_match_pattern()` implemented
- [ ] Can parse basic @match without error

### Day 2: Tag Comparison
- [ ] Field mapping registry created
- [ ] Tag check generation works
- [ ] If-condition AST generation works
- [ ] Can generate if/else skeleton

### Day 3: Variable Binding
- [ ] Binding extraction works
- [ ] `local` declarations generated
- [ ] Field access generated
- [ ] Bindings inserted correctly

### Day 4: Exhaustiveness
- [ ] Catch-all else clause works
- [ ] Panic message generated
- [ ] Match-as-expression works
- [ ] Null assignment in panic case

### Day 5: Basic Tests
- [ ] Test 1: Result basic - PASS
- [ ] Test 2: Option basic - PASS
- [ ] Test 3: 3-way variant - PASS
- [ ] Test 4: Expression - PASS
- [ ] Test 5: Multi-statement - PASS
- [ ] Test 6: Single binding - PASS
- [ ] Test 7: Multi binding - PASS
- [ ] Test 8: Shadowing - PASS

### Day 6: Edge Cases
- [ ] Test 9: Exhaustive - PASS
- [ ] Test 10: Non-exhaustive - PASS
- [ ] Test 11: Wildcard - PASS
- [ ] Test 12: 0-field variant - PASS
- [ ] Test 13: Nested if - PASS
- [ ] Test 14: Loop control - PASS
- [ ] Test 15: Early return - PASS

### Final
- [ ] All 15 tests PASS
- [ ] Smoke tests PASS
- [ ] Documentation complete
- [ ] Ready for production

---

## 🎓 Learning Resources

### Rust Parser References
- `src/parser/expressions.rs` - Expression parsing patterns
- `src/parser/statements/mod.rs` - Statement parsing
- `src/ast.rs` - AST node definitions

### Existing Macro Examples
- `src/macro/macro_box.rs` - MacroBox trait
- `apps/macros/loop_normalize_macro.nyash` - Complex desugaring

### Hakorune Box Examples
- `apps/lib/boxes/result.hako` - Result implementation
- `apps/lib/boxes/option.hako` - Option implementation

---

## 💡 Tips for Success

1. **Start Simple**: Test with 2-arm match first (Ok/Err)
2. **Use --dump-mir**: Verify desugared code constantly
3. **Incremental Testing**: Test each day's work immediately
4. **Reference Phase 16**: @derive implementation is similar pattern
5. **Ask Questions**: Consult complete spec for details

---

## 🆘 When Stuck

### Debug Commands
```bash
# See parsed AST
./target/release/hako --dump-ast test.hako

# See desugared code
./target/release/hako --dump-mir test.hako

# Verbose diagnostics
NYASH_CLI_VERBOSE=1 ./target/release/hako test.hako
```

### Common Issues
- **Parse Error**: Check tokenizer output
- **Wrong Desugaring**: Check field mapping registry
- **Test Failure**: Use --dump-mir to compare expected vs actual

---

**Ready to Start?** Begin with Day 1 tasks and refer to the complete spec for detailed algorithms!

**Questions?** Check [@match-macro-implementation-spec.md](./@match-macro-implementation-spec.md) for comprehensive details.
