# @match Macro Implementation Specification

**Created**: 2025-10-08
**Target Phase**: Phase 20 (VariantBox)
**Strategy**: Choice A'' (macro-only, desugar to if/else)
**MIR Compatibility**: MIR16 Frozen (no new instructions)

---

## 📋 Table of Contents

1. [Design Goals](#design-goals)
2. [Parser Changes](#parser-changes)
3. [Macro Desugaring Rules](#macro-desugaring-rules)
4. [Pattern Types Support](#pattern-types-support)
5. [Edge Cases Handling](#edge-cases-handling)
6. [Test Suite Design](#test-suite-design)
7. [Implementation Task Breakdown](#implementation-task-breakdown)
8. [Integration with @enum](#integration-with-enum)
9. [Desugaring Algorithm](#desugaring-algorithm)
10. [Error Handling Design](#error-handling-design)
11. [Risk Analysis](#risk-analysis)

---

## 🎯 Design Goals

### Philosophy Alignment
- ✅ **Everything-is-Box**: Result/Option are existing Box types with `_tag`/`_value` fields
- ✅ **MIR16 Frozen**: Desugar to existing if/else/compare MIR instructions
- ✅ **Macro-Only**: No runtime support, pure AST transformation
- ✅ **Fail-Fast**: Runtime panic for non-exhaustive matches (compile-time check is future work)

### Technical Goals
- Pattern matching for Result/Option (existing boxes)
- Variable binding extraction (`Ok(v)` → `local v = result._value`)
- Exhaustiveness check via runtime panic
- Expression-level match (return value support)

---

## 🔧 Parser Changes

### File: `src/parser/mod.rs`

### 1. New AST Node Variant

Add to `ASTNode` enum in `src/ast.rs`:

```rust
/// Pattern match expression: @match expr { Pattern(vars) => body ... }
MatchPattern {
    scrutinee: Box<ASTNode>,
    arms: Vec<MatchArm>,
    span: Span,
}
```

### 2. New Struct Definitions

```rust
/// Match arm: Pattern(x, y) => expression/block
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<ASTNode>,  // Can be single expression or multi-statement block
    pub span: Span,
}

/// Pattern types
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Variant pattern: Ok(v), Err(e), Some(x), None
    Variant {
        name: String,           // "Ok", "Err", "Some", "None"
        bindings: Vec<String>,  // ["v"], ["e"], ["x"], []
        span: Span,
    },
    /// Wildcard pattern: _
    Wildcard {
        span: Span,
    },
}
```

### 3. Parser Method: `parse_match_pattern()`

Add to `src/parser/statements/mod.rs`:

```rust
fn parse_match_pattern(&mut self) -> Result<ASTNode, ParseError> {
    let start = self.current_token().line;

    // Consume @match
    self.advance();

    // Parse scrutinee expression
    let scrutinee = Box::new(self.parse_expression()?);

    // Expect {
    self.expect(TokenType::LeftBrace)?;

    // Parse match arms
    let mut arms = Vec::new();
    while !self.check(&TokenType::RightBrace) {
        let arm = self.parse_match_arm()?;
        arms.push(arm);
    }

    // Expect }
    self.expect(TokenType::RightBrace)?;

    // Validation: at least one arm required
    if arms.is_empty() {
        return Err(ParseError::InvalidExpression {
            line: start,
            message: "Match expression must have at least one arm".to_string()
        });
    }

    Ok(ASTNode::MatchPattern {
        scrutinee,
        arms,
        span: Span::unknown(),
    })
}

fn parse_match_arm(&mut self) -> Result<MatchArm, ParseError> {
    let start_line = self.current_token().line;

    // Parse pattern
    let pattern = self.parse_pattern()?;

    // Expect =>
    self.expect(TokenType::FatArrow)?;  // => operator

    // Parse body (can be single expression or block with {})
    let body = if self.check(&TokenType::LeftBrace) {
        // Multi-statement block
        self.advance(); // consume {
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) {
            statements.push(self.parse_statement()?);
        }
        self.expect(TokenType::RightBrace)?;
        statements
    } else {
        // Single expression (implicit return)
        vec![self.parse_expression()?]
    };

    Ok(MatchArm {
        pattern,
        body,
        span: Span::unknown(),
    })
}

fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
    let start_line = self.current_token().line;

    match &self.current_token().token_type {
        // Wildcard: _
        TokenType::Underscore => {
            self.advance();
            Ok(Pattern::Wildcard {
                span: Span::unknown(),
            })
        }
        // Variant pattern: Ok(v), Err(e), Some(x), None
        TokenType::Identifier(name) => {
            let variant_name = name.clone();
            self.advance();

            // Check for bindings: (x, y, z)
            let bindings = if self.check(&TokenType::LeftParen) {
                self.advance(); // consume (
                let mut vars = Vec::new();

                while !self.check(&TokenType::RightParen) {
                    if let TokenType::Identifier(var) = &self.current_token().token_type {
                        vars.push(var.clone());
                        self.advance();

                        // Optional comma
                        if self.check(&TokenType::Comma) {
                            self.advance();
                        }
                    } else {
                        return Err(ParseError::ExpectedIdentifier { line: start_line });
                    }
                }

                self.expect(TokenType::RightParen)?;
                vars
            } else {
                Vec::new()  // 0-field variant (e.g., None)
            };

            Ok(Pattern::Variant {
                name: variant_name,
                bindings,
                span: Span::unknown(),
            })
        }
        _ => Err(ParseError::InvalidExpression {
            line: start_line,
            message: "Expected pattern (variant or wildcard)".to_string(),
        })
    }
}
```

### 4. Token Type Addition

Add to `src/tokenizer.rs`:

```rust
pub enum TokenType {
    // ... existing tokens ...

    /// => (fat arrow for match arms)
    FatArrow,

    /// _ (wildcard pattern)
    Underscore,

    /// @match, @enum, etc.
    MacroKeyword(String),
}
```

### 5. Tokenizer Update

Modify `src/tokenizer.rs` to recognize:
- `@match` as `MacroKeyword("match")`
- `=>` as `FatArrow`
- `_` as `Underscore` (if not already present)

---

## 📝 Macro Desugaring Rules

### Core Principle
**Every @match transforms to a chain of if/else with field access and runtime exhaustiveness check.**

### Example 1: Basic Result Matching

**Input**:
```hakorune
@match result {
  Ok(v) => print("Success: " + v)
  Err(e) => print("Error: " + e)
}
```

**Desugared Output**:
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

**Implementation Notes**:
- Result uses `_ok` (1=Ok, 0=Err) instead of `_tag` string
- Field mapping: `Ok` → `_val`, `Err` → `_err`
- Exhaustiveness: catch-all else for safety

### Example 2: Option Matching with Wildcard

**Input**:
```hakorune
@match option {
  Some(x) => return x * 2
  None => return 0
}
```

**Desugared Output**:
```hakorune
if option._is_some == 1 {
  local x
  x = option._value
  return x * 2
} else if option._is_some == 0 {
  return 0
} else {
  print("[PANIC] non-exhaustive match on Option: unknown state")
}
```

**Implementation Notes**:
- Option uses `_is_some` (1=Some, 0=None) instead of `_tag`
- Field mapping: `Some` → `_value`, `None` → (no field)
- Exhaustiveness: still required for robustness

### Example 3: Generic VariantBox (Future @enum)

**Input**:
```hakorune
@enum Point {
  Cartesian(x, y)
  Polar(r, theta)
}

@match point {
  Cartesian(x, y) => print("x=" + x + " y=" + y)
  Polar(r, theta) => print("r=" + r + " theta=" + theta)
}
```

**Desugared Output**:
```hakorune
if point._tag == "Cartesian" {
  local x
  local y
  x = point._x
  y = point._y
  print("x=" + x + " y=" + y)
} else if point._tag == "Polar" {
  local r
  local theta
  r = point._r
  theta = point._theta
  print("r=" + r + " theta=" + theta)
} else {
  print("[PANIC] non-exhaustive match on Point: " + point._tag)
}
```

**Implementation Notes**:
- Generic VariantBox uses string `_tag` field
- Field mapping: `Variant(a, b)` → `_a`, `_b` (lowercase field names)
- Exhaustiveness: show actual unknown tag in panic message

### Example 4: Match as Expression (Return Value)

**Input**:
```hakorune
local result
result = @match status {
  Success(val) => val * 2
  Failure(msg) => 0
}
print(result)
```

**Desugared Output**:
```hakorune
local result
local __match_tmp
if status._tag == "Success" {
  local val
  val = status._val
  __match_tmp = val * 2
} else if status._tag == "Failure" {
  local msg
  msg = status._msg
  __match_tmp = 0
} else {
  print("[PANIC] non-exhaustive match on Status: " + status._tag)
  __match_tmp = null
}
result = __match_tmp
print(result)
```

**Implementation Notes**:
- Introduce temporary variable `__match_tmp` to hold result
- Each arm assigns to `__match_tmp` instead of returning
- Final assignment outside if/else chain
- Exhaustiveness fallback assigns `null`

### Example 5: Nested Blocks with Multiple Statements

**Input**:
```hakorune
@match result {
  Ok(v) => {
    local doubled
    doubled = v * 2
    print("Result: " + doubled)
    return doubled
  }
  Err(e) => {
    print("Error: " + e)
    return -1
  }
}
```

**Desugared Output**:
```hakorune
if result._ok == 1 {
  local v
  v = result._val
  local doubled
  doubled = v * 2
  print("Result: " + doubled)
  return doubled
} else if result._ok == 0 {
  local e
  e = result._err
  print("Error: " + e)
  return -1
} else {
  print("[PANIC] non-exhaustive match on Result: unknown state")
}
```

**Implementation Notes**:
- Multi-statement blocks preserved as-is
- Variable bindings inserted at top of block
- No wrapping needed (already in block context)

---

## 🔍 Pattern Types Support

### Phase 1 (MVP Implementation)

| Pattern Type | Syntax | Example | Status |
|--------------|--------|---------|--------|
| **Variant 0-field** | `Name` | `None` | ✅ MVP |
| **Variant 1-field** | `Name(x)` | `Ok(v)`, `Some(x)` | ✅ MVP |
| **Variant N-field** | `Name(x, y, ...)` | `Cartesian(x, y)` | ✅ MVP |
| **Wildcard** | `_` | `_ => default` | ✅ MVP |

### Phase 2 (Future Enhancements)

| Pattern Type | Syntax | Example | Status |
|--------------|--------|---------|--------|
| **Literal** | `42`, `"str"` | `42 => ...` | ❌ Future |
| **Guard Clause** | `pattern if cond` | `Some(x) if x > 10` | ❌ Future |
| **Nested Pattern** | `Outer(Inner(x))` | `Some(Ok(v))` | ❌ Future |
| **Multiple Patterns** | `A \| B` | `Ok(v) \| Some(v)` | ❌ Future |

**MVP Focus**: Variant patterns only (sufficient for Result/Option/VariantBox)

---

## ⚠️ Edge Cases Handling

### 1. Match as Expression

**Issue**: Match must produce a value for assignment

**Solution**: Introduce temporary variable pattern (see Example 4 above)

**Test Case**:
```hakorune
local result = @match opt {
  Some(x) => x * 2
  None => 0
}
assert(result == (opt._is_some == 1 ? opt._value * 2 : 0))
```

### 2. Single Pattern Match

**Issue**: Single arm still needs exhaustiveness check

**Solution**: Always add catch-all else clause

**Input**:
```hakorune
@match result {
  Ok(v) => print(v)
}
```

**Desugared**:
```hakorune
if result._ok == 1 {
  local v
  v = result._val
  print(v)
} else {
  print("[PANIC] non-exhaustive match: missing Err case")
}
```

### 3. Empty Match

**Issue**: Match with zero arms is invalid

**Solution**: Parser error at parse time

**Error Message**:
```
[ERROR] Match expression must have at least one arm at line 42
```

### 4. Duplicate Patterns

**Issue**: Two arms with same pattern

**Input**:
```hakorune
@match result {
  Ok(v) => print("first")
  Ok(x) => print("second")  // ❌ Duplicate
}
```

**Solution**: Warning (first arm wins, second unreachable)

**Warning Message**:
```
[WARN] Unreachable pattern 'Ok' at line 43 (already matched at line 42)
```

### 5. Variable Shadowing

**Issue**: Pattern variable shadows local variable

**Input**:
```hakorune
local v = 10
@match result {
  Ok(v) => print(v)  // Different 'v'
}
print(v)  // Original 'v' = 10
```

**Solution**: Each arm introduces new scope (via if block)

**Desugared**:
```hakorune
local v = 10
if result._ok == 1 {
  local v  // New scope, shadows outer v
  v = result._val
  print(v)
}
print(v)  // Original v still 10
```

### 6. Match on Non-Enum Value

**Issue**: Match on primitive type (IntegerBox, StringBox)

**Input**:
```hakorune
@match 42 {
  Ok(v) => print(v)  // ❌ 42 has no _ok field
}
```

**Solution**: Runtime error (field access fails)

**Alternative**: Static check (Phase 2, requires type system)

### 7. Early Return in Match Arm

**Issue**: Return statement inside match arm

**Input**:
```hakorune
local compute() {
  @match result {
    Ok(v) => return v
    Err(e) => return -1
  }
}
```

**Desugared**:
```hakorune
local compute() {
  if result._ok == 1 {
    local v
    v = result._val
    return v  // ✅ Works (early exit from function)
  } else if result._ok == 0 {
    local e
    e = result._err
    return -1
  } else {
    print("[PANIC] non-exhaustive match on Result")
  }
}
```

**Result**: Works correctly (return exits function)

### 8. Match Inside Loop with Break/Continue

**Input**:
```hakorune
loop(i < 10) {
  @match opt {
    Some(x) => {
      if x > 5 {
        break
      }
      continue
    }
    None => i = i + 1
  }
}
```

**Desugared**:
```hakorune
loop(i < 10) {
  if opt._is_some == 1 {
    local x
    x = opt._value
    if x > 5 {
      break  // ✅ Breaks from loop
    }
    continue  // ✅ Continues loop
  } else if opt._is_some == 0 {
    i = i + 1
  } else {
    print("[PANIC] non-exhaustive match on Option")
  }
}
```

**Result**: Works correctly (break/continue affect loop)

---

## 🧪 Test Suite Design

### Minimum 15 Test Patterns

### Basic Matching (5 tests)

#### Test 1: Result 2-Variant Match
```hakorune
// apps/tests/match/test_result_basic.hako
using "../../lib/boxes/result.hako"

static box Main {
  main() {
    local r = Result.ok(42)

    local result = @match r {
      Ok(v) => v * 2
      Err(e) => -1
    }

    assert(result == 84)
    print("PASS: Result basic match")
  }
}
```

#### Test 2: Option 2-Variant Match
```hakorune
// apps/tests/match/test_option_basic.hako
using "../../lib/boxes/option.hako"

static box Main {
  main() {
    local opt = Option.some(10)

    @match opt {
      Some(x) => print("Value: " + x)
      None => print("No value")
    }

    print("PASS: Option basic match")
  }
}
```

#### Test 3: 3+ Variant Enum Match (Future VariantBox)
```hakorune
// apps/tests/match/test_variant_3way.hako
@enum Status {
  Pending
  Success(result)
  Failure(error)
}

static box Main {
  main() {
    local s = Status.Success("Done")

    @match s {
      Pending => print("Waiting...")
      Success(r) => print("OK: " + r)
      Failure(e) => print("Error: " + e)
    }

    print("PASS: 3-way variant match")
  }
}
```

#### Test 4: Match with Return Values
```hakorune
// apps/tests/match/test_match_expression.hako
using "../../lib/boxes/result.hako"

static box Main {
  compute(r) {
    return @match r {
      Ok(v) => v * 2
      Err(e) => 0
    }
  }

  main() {
    local r1 = Result.ok(21)
    local r2 = Result.err("fail")

    assert(me.compute(r1) == 42)
    assert(me.compute(r2) == 0)

    print("PASS: Match as expression")
  }
}
```

#### Test 5: Match with Multi-Statement Blocks
```hakorune
// apps/tests/match/test_match_blocks.hako
using "../../lib/boxes/option.hako"

static box Main {
  main() {
    local opt = Option.some(5)
    local result = 0

    @match opt {
      Some(x) => {
        local doubled = x * 2
        local tripled = doubled + x
        result = tripled
        print("Computed: " + result)
      }
      None => {
        result = -1
        print("No value")
      }
    }

    assert(result == 15)
    print("PASS: Multi-statement blocks")
  }
}
```

### Variable Binding (3 tests)

#### Test 6: Single Variable Binding
```hakorune
// apps/tests/match/test_binding_single.hako
using "../../lib/boxes/result.hako"

static box Main {
  main() {
    local r = Result.ok("hello")

    @match r {
      Ok(msg) => {
        assert(msg == "hello")
        print("PASS: Single variable binding")
      }
      Err(e) => print("Error: " + e)
    }
  }
}
```

#### Test 7: Multi-Variable Binding
```hakorune
// apps/tests/match/test_binding_multi.hako
@enum Point {
  Cartesian(x, y)
  Polar(r, theta)
}

static box Main {
  main() {
    local p = Point.Cartesian(3, 4)

    @match p {
      Cartesian(x, y) => {
        assert(x == 3)
        assert(y == 4)
        print("PASS: Multi-variable binding")
      }
      Polar(r, theta) => print("Polar")
    }
  }
}
```

#### Test 8: Variable Shadowing Test
```hakorune
// apps/tests/match/test_shadowing.hako
using "../../lib/boxes/option.hako"

static box Main {
  main() {
    local x = 100
    local opt = Option.some(42)

    @match opt {
      Some(x) => {
        // Inner x = 42
        assert(x == 42)
      }
      None => print("None")
    }

    // Outer x still 100
    assert(x == 100)
    print("PASS: Variable shadowing")
  }
}
```

### Exhaustiveness (3 tests)

#### Test 9: Exhaustive Match (All Variants)
```hakorune
// apps/tests/match/test_exhaustive_ok.hako
using "../../lib/boxes/result.hako"

static box Main {
  main() {
    local r = Result.ok(42)

    local handled = 0
    @match r {
      Ok(v) => handled = 1
      Err(e) => handled = 2
    }

    assert(handled == 1)
    print("PASS: Exhaustive match")
  }
}
```

#### Test 10: Non-Exhaustive Match (Runtime Panic)
```hakorune
// apps/tests/match/test_exhaustive_fail.hako
using "../../lib/boxes/result.hako"

static box Main {
  main() {
    // Manually create invalid Result state
    local r = new ResultBox()
    r._ok = 99  // Invalid state

    // This should trigger panic
    @match r {
      Ok(v) => print("OK")
      Err(e) => print("Err")
    }
    // Expected output: [PANIC] non-exhaustive match on Result: unknown state
  }
}
```

#### Test 11: Wildcard Pattern Exhaustiveness
```hakorune
// apps/tests/match/test_wildcard.hako
using "../../lib/boxes/option.hako"

static box Main {
  main() {
    local opt = Option.none()

    local result = @match opt {
      Some(x) => x
      _ => -1  // Wildcard catches None and any invalid states
    }

    assert(result == -1)
    print("PASS: Wildcard pattern")
  }
}
```

### Edge Cases (4 tests)

#### Test 12: Match on 0-Field Variant
```hakorune
// apps/tests/match/test_0field_variant.hako
using "../../lib/boxes/option.hako"

static box Main {
  main() {
    local opt = Option.none()

    @match opt {
      Some(x) => print("Value: " + x)
      None => print("PASS: 0-field variant")
    }
  }
}
```

#### Test 13: Nested If-Else Inside Match Arm
```hakorune
// apps/tests/match/test_nested_if.hako
using "../../lib/boxes/result.hako"

static box Main {
  main() {
    local r = Result.ok(10)

    @match r {
      Ok(v) => {
        if v > 5 {
          print("Large: " + v)
        } else {
          print("Small: " + v)
        }
      }
      Err(e) => print("Error: " + e)
    }

    print("PASS: Nested if-else")
  }
}
```

#### Test 14: Match Inside Loop with Break/Continue
```hakorune
// apps/tests/match/test_loop_control.hako
using "../../lib/boxes/option.hako"

static box Main {
  main() {
    local i = 0
    local found = 0

    loop(i < 10) {
      local opt = (i == 5) ? Option.some(i) : Option.none()

      @match opt {
        Some(x) => {
          found = x
          break
        }
        None => i = i + 1
      }

      i = i + 1
    }

    assert(found == 5)
    print("PASS: Loop control with match")
  }
}
```

#### Test 15: Match with Early Return in Each Arm
```hakorune
// apps/tests/match/test_early_return.hako
using "../../lib/boxes/result.hako"

static box Main {
  compute(r) {
    @match r {
      Ok(v) => return v * 2
      Err(e) => return -1
    }
    // This line is unreachable
  }

  main() {
    local r1 = Result.ok(21)
    local r2 = Result.err("fail")

    assert(me.compute(r1) == 42)
    assert(me.compute(r2) == -1)

    print("PASS: Early return in arms")
  }
}
```

---

## 📅 Implementation Task Breakdown

### 6-Day Implementation Plan

#### Day 1: Parser Infrastructure (8 hours)
- [ ] Add `MatchPattern` AST node variant
- [ ] Add `Pattern` and `MatchArm` struct definitions
- [ ] Implement `parse_match_pattern()` method
- [ ] Implement `parse_match_arm()` method
- [ ] Implement `parse_pattern()` method
- [ ] Add `FatArrow` and `Underscore` token types
- [ ] Update tokenizer to recognize `@match`, `=>`, `_`
- [ ] **Milestone**: Parser can parse @match syntax without errors

#### Day 2: Pattern Analysis & Tag Comparison (8 hours)
- [ ] Create field mapping registry (Pattern → field names)
  - Result: `Ok` → `_ok==1, _val`, `Err` → `_ok==0, _err`
  - Option: `Some` → `_is_some==1, _value`, `None` → `_is_some==0`
  - VariantBox: `Tag` → `_tag=="Tag", _field0, _field1, ...`
- [ ] Implement tag comparison code generation
- [ ] Generate if-condition AST nodes for each pattern
- [ ] **Milestone**: Can generate if/else skeleton for patterns

#### Day 3: Variable Binding Extraction (8 hours)
- [ ] Implement binding extraction (`Variant(x, y)` → field names)
- [ ] Generate `local` declaration AST nodes
- [ ] Generate field access AST nodes (`result._val`, etc.)
- [ ] Insert binding code at top of each arm
- [ ] Handle 0-field variants (no bindings)
- [ ] **Milestone**: Variable bindings work correctly

#### Day 4: Exhaustiveness Check & Panic Generation (8 hours)
- [ ] Generate catch-all else clause AST
- [ ] Generate panic print statement
- [ ] Implement exhaustiveness check logic
- [ ] Add tag name to panic message (for VariantBox)
- [ ] Handle match-as-expression case (assign null in panic)
- [ ] **Milestone**: Exhaustiveness panic works

#### Day 5: Test Suite - Basic & Variable Binding (8 hours)
- [ ] Write Test 1-5 (Basic Matching)
- [ ] Write Test 6-8 (Variable Binding)
- [ ] Run tests, fix issues
- [ ] Verify desugared code with `--dump-mir`
- [ ] **Milestone**: 8/15 tests passing

#### Day 6: Test Suite - Exhaustiveness & Edge Cases (8 hours)
- [ ] Write Test 9-11 (Exhaustiveness)
- [ ] Write Test 12-15 (Edge Cases)
- [ ] Run full test suite
- [ ] Document known issues/limitations
- [ ] Add smoke tests to CI
- [ ] **Milestone**: 15/15 tests passing, MVP complete

---

## 🔗 Integration with @enum

### Phase 1 (MVP): Hardcoded Field Mappings

**Strategy**: Use fixed field mapping for existing boxes (Result, Option)

```rust
// In desugaring logic
fn get_field_mapping(variant: &str, index: usize) -> String {
    match variant {
        // Result
        "Ok" => if index == 0 { "_val" } else { panic!("Ok has 1 field") },
        "Err" => if index == 0 { "_err" } else { panic!("Err has 1 field") },

        // Option
        "Some" => if index == 0 { "_value" } else { panic!("Some has 1 field") },
        "None" => panic!("None has 0 fields"),

        // Unknown: assume generic VariantBox convention
        _ => format!("_{}", index),  // _0, _1, _2, ...
    }
}

fn get_tag_check(variant: &str) -> (String, String) {
    match variant {
        // Result: check _ok field
        "Ok" => ("_ok".to_string(), "1".to_string()),
        "Err" => ("_ok".to_string(), "0".to_string()),

        // Option: check _is_some field
        "Some" => ("_is_some".to_string(), "1".to_string()),
        "None" => ("_is_some".to_string(), "0".to_string()),

        // Unknown: assume generic _tag field
        _ => ("_tag".to_string(), format!("\"{}\"", variant)),
    }
}
```

### Phase 2 (Future): @enum Integration

**When @enum is implemented**:
1. @enum generates EnumSchemaBox metadata
2. @match queries EnumSchemaBox for field names
3. Compile-time exhaustiveness check possible

**Example**:
```hakorune
@enum Status {
  Pending
  Success(result)
  Failure(error, code)
}

// EnumSchemaBox generated:
// Status._schema.variants = {
//   "Pending": { arity: 0, fields: [] },
//   "Success": { arity: 1, fields: ["result"] },
//   "Failure": { arity: 2, fields: ["error", "code"] }
// }

@match status {
  Pending => ...
  Success(r) => ...
  // ❌ Missing Failure → Compile-time error (Phase 2)
}
```

**Exhaustiveness Check Algorithm (Phase 2)**:
```rust
fn check_exhaustiveness(scrutinee_type: &str, arms: &[MatchArm]) -> Result<(), String> {
    // 1. Get enum schema
    let schema = get_enum_schema(scrutinee_type)?;

    // 2. Collect covered variants
    let mut covered = HashSet::new();
    let mut has_wildcard = false;

    for arm in arms {
        match &arm.pattern {
            Pattern::Variant { name, .. } => { covered.insert(name); }
            Pattern::Wildcard { .. } => { has_wildcard = true; }
        }
    }

    // 3. Check if all variants covered
    if !has_wildcard {
        for variant in schema.variants.keys() {
            if !covered.contains(variant) {
                return Err(format!("Missing pattern: {}", variant));
            }
        }
    }

    Ok(())
}
```

---

## 🛠️ Desugaring Algorithm

### High-Level Algorithm

```
INPUT: MatchPattern { scrutinee, arms }
OUTPUT: If { condition, then_body, else_body }

ALGORITHM:
  1. Generate temporary variable (if match is expression)
  2. For each arm in arms:
     a. Extract pattern and body
     b. Generate tag comparison condition
     c. Generate variable bindings (local declarations)
     d. Prepend bindings to body
     e. If expression: wrap body with assignment to temp var
  3. Chain arms into if/else-if/else structure
  4. Add exhaustiveness panic as final else clause
  5. If expression: append temp var access after if/else
```

### Pseudo-Code Implementation

```rust
fn desugar_match(
    scrutinee: &ASTNode,
    arms: &[MatchArm],
    is_expression: bool
) -> ASTNode {
    // Step 1: Temp var for expression case
    let temp_var = if is_expression {
        Some(format!("__match_tmp_{}", unique_id()))
    } else {
        None
    };

    // Step 2: Build if/else chain
    let mut current_else: Option<Vec<ASTNode>> = Some(vec![
        // Final else: exhaustiveness panic
        ASTNode::Print {
            expression: Box::new(ASTNode::Literal {
                value: LiteralValue::String(
                    "[PANIC] non-exhaustive match".to_string()
                ),
                span: Span::unknown(),
            }),
            span: Span::unknown(),
        }
    ]);

    // Build chain in reverse order (last arm first)
    for arm in arms.iter().rev() {
        // Step 3a: Extract pattern
        let (tag_field, tag_value) = get_tag_check(&arm.pattern);

        // Step 3b: Generate condition
        let condition = ASTNode::BinaryOp {
            operator: BinaryOperator::Equals,
            left: Box::new(ASTNode::FieldAccess {
                object: Box::new(scrutinee.clone()),
                field: tag_field,
                span: Span::unknown(),
            }),
            right: Box::new(ASTNode::Literal {
                value: tag_value,
                span: Span::unknown(),
            }),
            span: Span::unknown(),
        };

        // Step 3c: Generate bindings
        let bindings = generate_bindings(
            scrutinee,
            &arm.pattern
        );

        // Step 3d: Build then_body
        let mut then_body = bindings;

        if let Some(ref temp) = temp_var {
            // Expression case: wrap body with assignment
            let value = if arm.body.len() == 1 {
                arm.body[0].clone()
            } else {
                // Multi-statement: last statement is value
                arm.body.last().unwrap().clone()
            };

            then_body.extend(arm.body[..arm.body.len()-1].to_vec());
            then_body.push(ASTNode::Assignment {
                target: Box::new(ASTNode::Variable {
                    name: temp.clone(),
                    span: Span::unknown(),
                }),
                value: Box::new(value),
                span: Span::unknown(),
            });
        } else {
            // Statement case: just append body
            then_body.extend(arm.body.clone());
        }

        // Step 3e: Create if node
        current_else = Some(vec![ASTNode::If {
            condition: Box::new(condition),
            then_body,
            else_body: current_else,
            span: Span::unknown(),
        }]);
    }

    // Step 4: Return final structure
    if let Some(temp) = temp_var {
        // Expression: declare temp, if/else, return temp
        let mut result = vec![
            ASTNode::Local {
                variables: vec![temp.clone()],
                span: Span::unknown(),
            },
        ];
        result.extend(current_else.unwrap());
        result.push(ASTNode::Variable {
            name: temp,
            span: Span::unknown(),
        });

        ASTNode::Program {
            statements: result,
            span: Span::unknown(),
        }
    } else {
        // Statement: just if/else
        current_else.unwrap()[0].clone()
    }
}

fn generate_bindings(
    scrutinee: &ASTNode,
    pattern: &Pattern
) -> Vec<ASTNode> {
    match pattern {
        Pattern::Variant { name, bindings, .. } => {
            let mut result = Vec::new();

            for (i, var) in bindings.iter().enumerate() {
                let field_name = get_field_mapping(name, i);

                // local var
                result.push(ASTNode::Local {
                    variables: vec![var.clone()],
                    span: Span::unknown(),
                });

                // var = scrutinee._field
                result.push(ASTNode::Assignment {
                    target: Box::new(ASTNode::Variable {
                        name: var.clone(),
                        span: Span::unknown(),
                    }),
                    value: Box::new(ASTNode::FieldAccess {
                        object: Box::new(scrutinee.clone()),
                        field: field_name,
                        span: Span::unknown(),
                    }),
                    span: Span::unknown(),
                });
            }

            result
        }
        Pattern::Wildcard { .. } => {
            Vec::new()  // No bindings
        }
    }
}
```

---

## ⚠️ Error Handling Design

### Parser Errors

#### 1. Pattern Syntax Error

**Input**:
```hakorune
@match result {
  Ok(v =>  // ❌ Missing )
}
```

**Error Message**:
```
[ERROR] Expected ')' at line 2, found '=>'
Context: Parsing variant pattern 'Ok'
```

#### 2. Unknown Variant (Future, with @enum)

**Input**:
```hakorune
@match result {
  Success(v) => ...  // ❌ Result has Ok/Err, not Success
}
```

**Error Message**:
```
[ERROR] Unknown variant 'Success' for type 'Result' at line 2
Available variants: Ok(value), Err(error)
Help: Did you mean 'Ok'?
```

#### 3. Duplicate Pattern

**Input**:
```hakorune
@match result {
  Ok(v) => print("first")
  Ok(x) => print("second")  // ❌ Duplicate
}
```

**Warning Message**:
```
[WARN] Unreachable pattern 'Ok' at line 3
Note: This pattern is already matched at line 2
```

#### 4. Type Mismatch (Future, with type system)

**Input**:
```hakorune
@match 42 {
  Ok(v) => ...  // ❌ 42 is IntegerBox, not Result
}
```

**Error Message**:
```
[ERROR] Type mismatch in match expression at line 1
Expected: Result, Option, or VariantBox
Found: IntegerBox
```

#### 5. Missing Fat Arrow

**Input**:
```hakorune
@match result {
  Ok(v) print(v)  // ❌ Missing =>
}
```

**Error Message**:
```
[ERROR] Expected '=>' at line 2, found 'print'
Context: After pattern 'Ok(v)'
```

#### 6. Empty Match Arms

**Input**:
```hakorune
@match result {
}
```

**Error Message**:
```
[ERROR] Match expression must have at least one arm at line 1
```

### Runtime Errors

#### 1. Non-Exhaustive Match Panic

**Trigger**: Unknown variant tag at runtime

**Output**:
```
[PANIC] non-exhaustive match on Result: unknown state
```

**For VariantBox with _tag**:
```
[PANIC] non-exhaustive match on Point: InvalidTag
```

#### 2. Field Access Error (Arity Mismatch)

**Trigger**: Pattern has wrong number of bindings

**Input** (if validation skipped):
```hakorune
@match result {
  Ok(v, x) => ...  // ❌ Ok has 1 field, not 2
}
```

**Runtime Error**:
```
[ERROR] Field access failed: result._field1 does not exist
```

**Future Prevention**: Static arity check (Phase 2)

---

## 🚨 Risk Analysis

### 1. Parser Conflicts with Existing Syntax

**Risk**: `=>` arrow operator may conflict with other syntax

**Mitigation**:
- Check existing uses of `>` and `=` tokens
- Ensure tokenizer prioritizes `=>` as single token
- Run full parser test suite

**Impact**: Medium (parser failure)

### 2. Pattern Variable Naming Conflicts

**Risk**: Pattern variable shadows important local/field

**Mitigation**:
- Each if arm creates new scope (natural shadowing)
- Warning for shadowing (optional)
- Document scoping rules

**Impact**: Low (expected behavior)

### 3. Exhaustiveness Check False Positives/Negatives

**Risk**: Runtime panic when all cases actually covered

**False Negative Example**:
```hakorune
// All cases covered, but panic still present
@match result {
  Ok(v) => ...
  Err(e) => ...
}
// Else clause still added: panic on invalid state
```

**Mitigation**:
- Phase 1: Always add panic (safe but verbose)
- Phase 2: Static analysis to remove panic if proven exhaustive

**Impact**: Low (extra safety, no correctness issue)

**False Positive Risk**: None (panic only triggers on actual invalid state)

### 4. Performance of If/Else Chain vs Jump Table

**Risk**: Long match arms → slow O(n) check instead of O(1) jump

**Benchmark**:
```hakorune
@match status {
  Case1 => ...
  Case2 => ...
  // ... 100 cases
  Case100 => ...
}
// Worst case: 100 comparisons
```

**Mitigation**:
- Phase 1: Accept O(n) for simplicity
- Phase 2: Optimize to jump table (requires MIR switch instruction)

**Impact**: Low (typical match has 2-5 arms)

### 5. Debugging Challenges (Pattern → Field Mapping)

**Risk**: Hard to trace which field maps to which variable

**Example Debug Scenario**:
```hakorune
@match point {
  Cartesian(x, y) => print(x)  // Which field is 'x'?
}
```

**Mitigation**:
- Clear documentation of field naming convention
- `--dump-mir` flag shows desugared code
- Error messages show field names

**Example Desugared (for debugging)**:
```hakorune
if point._tag == "Cartesian" {
  local x  // Maps to point._x
  local y  // Maps to point._y
  x = point._x
  y = point._y
  print(x)
}
```

**Impact**: Medium (developer experience)

### 6. Integration Risks with @enum

**Risk**: @match implemented before @enum, field mappings hardcoded

**Mitigation**:
- Phase 1: Hardcode Result/Option mappings
- Phase 1.5: Assume generic VariantBox convention (`_tag`, `_0`, `_1`, ...)
- Phase 2: Replace with EnumSchemaBox query

**Impact**: Low (phased approach, backwards compatible)

### 7. Macro Expansion Order Issues

**Risk**: @match expands before @enum, can't find schema

**Mitigation**:
- Phase 1: @match doesn't depend on @enum (hardcoded mappings)
- Phase 2: Macro dependency ordering system

**Impact**: None (Phase 1 independent)

### 8. AST Size Explosion

**Risk**: Large match → large if/else AST → slow compilation

**Example**:
```hakorune
@match value {
  Case1 => { /* 100 lines */ }
  Case2 => { /* 100 lines */ }
  // ... 50 cases
}
// Desugared AST: 5000+ nodes
```

**Mitigation**:
- Accept for Phase 1 (typical case: 2-5 arms)
- Optimize AST representation (Phase 2)

**Impact**: Low (rare in practice)

---

## 📊 Success Metrics

### MVP Completion Criteria

- ✅ All 15 test cases pass
- ✅ Result/Option matching works correctly
- ✅ Variable bindings extract fields properly
- ✅ Exhaustiveness panic triggers on invalid states
- ✅ Match-as-expression works (returns value)
- ✅ No regressions in existing tests
- ✅ Documentation complete (this spec + user guide)

### Performance Benchmarks (Optional)

- Match with 2 arms: <1ms overhead vs hand-written if/else
- Match with 10 arms: <5ms overhead
- Compilation time: <10% increase for typical codebase

### Quality Metrics

- Code coverage: >90% for match desugaring logic
- Zero parser crashes on valid input
- Clear error messages for all invalid inputs

---

## 📚 Related Documentation

- **VariantBox Design**: [DESIGN.md](./DESIGN.md)
- **@enum Macro Spec**: (To be created)
- **Result/Option API**: [apps/lib/boxes/result.hako](../../../apps/lib/boxes/result.hako), [option.hako](../../../apps/lib/boxes/option.hako)
- **Macro System**: [docs/development/roadmap/phases/phase-16-macro-revolution/](../phase-16-macro-revolution/)

---

## 🎯 Next Steps After MVP

### Phase 20.1: @enum Macro
- Implement @enum desugaring to static box + VariantBox
- Generate EnumSchemaBox metadata
- Connect @match to schema for field name resolution

### Phase 20.2: Compile-Time Exhaustiveness Check
- Implement exhaustiveness checker
- Remove runtime panic for proven-exhaustive matches
- Add warnings for unreachable patterns

### Phase 20.3: Advanced Patterns
- Literal patterns (`42 => ...`)
- Guard clauses (`Some(x) if x > 10`)
- Nested patterns (`Some(Ok(v))`)
- Multiple patterns (`Ok(v) | Some(v)`)

---

**Document Status**: ✅ Complete, ready for implementation
**Estimated Effort**: 6 person-days (48 hours)
**Dependencies**: None (self-contained)
**Blocking**: None (can start immediately)
