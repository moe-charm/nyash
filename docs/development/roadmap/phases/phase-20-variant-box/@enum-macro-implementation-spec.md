# @enum Macro Implementation Specification (Choice A'': Macro-Only)

**Version**: 1.0
**Date**: 2025-10-08
**Status**: Implementation-Ready
**Approach**: Macro-only desugaring (no VariantBox Core implementation)

---

## Table of Contents

1. [Overview](#1-overview)
2. [Parser Changes Specification](#2-parser-changes-specification)
3. [Macro Desugaring Rules](#3-macro-desugaring-rules)
4. [Edge Cases](#4-edge-cases)
5. [Test Suite Design](#5-test-suite-design)
6. [Implementation Task Breakdown](#6-implementation-task-breakdown)
7. [Integration with Existing Code](#7-integration-with-existing-code)
8. [Risk Analysis](#8-risk-analysis)

---

## 1. Overview

### 1.1 Goal
Implement `@enum` macro that desugars into:
- **1 box** (data container with `_tag` field)
- **1 static box** (constructors + helper methods)
- **All Box-typed fields** (Everything is Box principle)
- **MIR16 Frozen**: No IR changes, pure desugaring

### 1.2 Design Philosophy
- **Box-First**: All fields are Box-typed (no primitives)
- **Tag-Based**: Use `_tag: StringBox` for variant discrimination
- **Fail-Fast**: Type mismatches panic with clear error messages
- **Zero IR Impact**: Everything compiles to existing MIR16 instructions

### 1.3 Existing Foundation
- **Option/Result** already implemented (apps/lib/boxes/option.hako, result.hako)
- **Current pattern**: Uses `IntegerBox` for flags (`_is_some`, `_ok`)
- **New pattern**: Switch to `StringBox` `_tag` field for consistency

---

## 2. Parser Changes Specification

### 2.1 Token Recognition

**File**: `src/parser/mod.rs`

**New Token Pattern**:
```rust
// In tokenizer or parser
TokenType::At  // '@' character
TokenType::Identifier("enum")
```

**Parsing Trigger**:
```rust
// In parse_statement() or top-level parser
if current_token == TokenType::At && peek() == TokenType::Identifier("enum") {
    return parse_enum_declaration();
}
```

### 2.2 AST Node Structure

**Option A: Reuse BoxDeclaration with flag**
```rust
// In src/ast.rs ASTNode enum
BoxDeclaration {
    name: String,
    // ... existing fields ...
    is_enum: bool,              // NEW: Flag for enum boxes
    enum_variants: Vec<EnumVariant>, // NEW: Variant definitions
    span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,           // "Ok", "Err", "None", etc.
    pub fields: Vec<EnumField>, // Field list for this variant
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumField {
    pub name: String,           // Field name (or auto-generated)
    pub span: Span,
}
```

**Option B: New EnumDeclaration variant** (recommended for clarity)
```rust
// In src/ast.rs ASTNode enum
EnumDeclaration {
    name: String,               // "Result", "Option", etc.
    variants: Vec<EnumVariant>, // List of variants
    span: Span,
}
```

### 2.3 Parser Function

**Location**: `src/parser/declarations/mod.rs` or new `src/parser/declarations/enum_parser.rs`

```rust
/// Parse @enum declaration
///
/// Grammar:
///   @enum EnumName {
///     Variant1
///     Variant2(field)
///     Variant3(field1, field2)
///   }
fn parse_enum_declaration(&mut self) -> Result<ASTNode, ParseError> {
    // 1. Consume '@enum' tokens
    self.expect(TokenType::At)?;
    self.expect_keyword("enum")?;

    // 2. Parse enum name
    let name = self.expect_identifier()?;
    let enum_name = format!("{}Box", name); // Result -> ResultBox

    // 3. Parse variants block
    self.expect(TokenType::LeftBrace)?;

    let mut variants = Vec::new();
    while !self.check(TokenType::RightBrace) {
        let variant = self.parse_enum_variant()?;
        variants.push(variant);

        // Optional newline/semicolon between variants
        self.consume_if(TokenType::Newline);
        self.consume_if(TokenType::Semicolon);
    }

    self.expect(TokenType::RightBrace)?;

    // 4. Build AST node
    Ok(ASTNode::EnumDeclaration {
        name,
        variants,
        span: self.span_from(start),
    })
}

/// Parse single enum variant
///
/// Grammar:
///   Variant
///   Variant(field)
///   Variant(field1, field2)
fn parse_enum_variant(&mut self) -> Result<EnumVariant, ParseError> {
    let name = self.expect_identifier()?;
    let mut fields = Vec::new();

    // Check for field list
    if self.consume_if(TokenType::LeftParen) {
        while !self.check(TokenType::RightParen) {
            let field_name = self.expect_identifier()?;
            fields.push(EnumField {
                name: field_name,
                span: self.current_span(),
            });

            if !self.check(TokenType::RightParen) {
                self.expect(TokenType::Comma)?;
            }
        }
        self.expect(TokenType::RightParen)?;
    }

    Ok(EnumVariant {
        name,
        fields,
        span: self.span_from(start),
    })
}
```

### 2.4 Error Handling

**Parse Errors**:
```rust
#[derive(Error, Debug)]
pub enum ParseError {
    // ... existing errors ...

    #[error("Invalid enum syntax at line {line}: {message}")]
    InvalidEnumSyntax { message: String, line: usize },

    #[error("Duplicate variant name '{name}' in enum at line {line}")]
    DuplicateVariant { name: String, line: usize },

    #[error("Empty enum declaration at line {line}")]
    EmptyEnum { line: usize },
}
```

**Validation**:
1. Enum must have at least 1 variant
2. Variant names must be unique
3. Variant names must be valid identifiers
4. Field names within a variant must be unique

---

## 3. Macro Desugaring Rules

### 3.1 Overall Transformation

**Input**:
```hakorune
@enum Result {
  Ok(value)
  Err(error)
}
```

**Output**:
```hakorune
box ResultBox {
  _tag: StringBox
  _value: Box
  _error: Box

  birth() {
    me._tag = ""
    me._value = null
    me._error = null
  }
}

static box Result {
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

  is_Ok(variant) { return variant._tag == "Ok" }
  is_Err(variant) { return variant._tag == "Err" }

  as_Ok(variant) {
    if variant._tag != "Ok" {
      print("[PANIC] Result.as_Ok: called on " + variant._tag)
      return null
    }
    return variant._value
  }

  as_Err(variant) {
    if variant._tag != "Err" {
      print("[PANIC] Result.as_Err: called on " + variant._tag)
      return null
    }
    return variant._error
  }
}
```

### 3.2 Naming Conventions

| Element | Convention | Example |
|---------|-----------|---------|
| **Enum Name** | Input name | `Result`, `Option` |
| **Box Name** | `{Name}Box` | `ResultBox`, `OptionBox` |
| **Tag Field** | `_tag` (always) | `_tag: StringBox` |
| **Variant Field (single)** | `_value` | For `Ok(value)` → `_value` |
| **Variant Field (named)** | `_{field}` | For `Err(error)` → `_error` |
| **Variant Field (multi)** | `_{field1}`, `_{field2}` | For `Pair(first, second)` → `_first`, `_second` |
| **Constructor** | Variant name | `Ok()`, `Err()` |
| **Is-checker** | `is_{Variant}(v)` | `is_Ok(v)`, `is_Err(v)` |
| **Getter** | `as_{Variant}(v)` | `as_Ok(v)`, `as_Err(v)` |

### 3.3 Field Generation Rules

**Rule 1: Collect all fields from all variants**
```
@enum Result {
  Ok(value)      → _value: Box
  Err(error)     → _error: Box
}
→ Fields: [_tag, _value, _error]
```

**Rule 2: All fields are Box-typed**
```hakorune
_tag: StringBox    // Always StringBox
_value: Box        // Generic Box (runtime polymorphism)
_error: Box
```

**Rule 3: Single unnamed field → `_value`**
```
Some(value) → _value: Box
```

**Rule 4: Multiple/named fields → prefix with `_`**
```
Pair(first, second) → _first: Box, _second: Box
```

### 3.4 Birth Method Generation

**Template**:
```hakorune
birth() {
  me._tag = ""
  {for each field}
    me.{field_name} = null
  {end for}
}
```

**Example (Result)**:
```hakorune
birth() {
  me._tag = ""
  me._value = null
  me._error = null
}
```

### 3.5 Constructor Generation

**Template for each variant**:
```hakorune
{VariantName}({param1}, {param2}, ...) {
  local inst = new {EnumName}Box()
  inst._tag = "{VariantName}"
  {for each field in variant}
    inst._{field_name} = {param_name}
  {end for}
  return inst
}
```

**Example (Ok variant)**:
```hakorune
Ok(v) {
  local r = new ResultBox()
  r._tag = "Ok"
  r._value = v
  return r
}
```

**Example (0-field variant)**:
```hakorune
None() {
  local opt = new OptionBox()
  opt._tag = "None"
  return opt
}
```

### 3.6 Helper Method Generation

#### 3.6.1 is_{Variant} Methods

**Template**:
```hakorune
is_{VariantName}(variant) {
  return variant._tag == "{VariantName}"
}
```

**Example**:
```hakorune
is_Ok(variant) { return variant._tag == "Ok" }
is_Err(variant) { return variant._tag == "Err" }
```

#### 3.6.2 as_{Variant} Methods

**Template (single field)**:
```hakorune
as_{VariantName}(variant) {
  if variant._tag != "{VariantName}" {
    print("[PANIC] {EnumName}.as_{VariantName}: called on " + variant._tag)
    return null
  }
  return variant._{field_name}
}
```

**Template (multi-field) - returns ArrayBox**:
```hakorune
as_{VariantName}(variant) {
  if variant._tag != "{VariantName}" {
    print("[PANIC] {EnumName}.as_{VariantName}: called on " + variant._tag)
    return null
  }
  local result = new ArrayBox()
  result.push(variant._{field1})
  result.push(variant._{field2})
  return result
}
```

**Example (single field)**:
```hakorune
as_Ok(variant) {
  if variant._tag != "Ok" {
    print("[PANIC] Result.as_Ok: called on " + variant._tag)
    return null
  }
  return variant._value
}
```

**Example (multi-field)**:
```hakorune
as_Pair(variant) {
  if variant._tag != "Pair" {
    print("[PANIC] Triple.as_Pair: called on " + variant._tag)
    return null
  }
  local result = new ArrayBox()
  result.push(variant._first)
  result.push(variant._second)
  return result
}
```

#### 3.6.3 0-field variant getter

For variants with no fields, `as_*` still exists but returns null:

```hakorune
as_None(variant) {
  if variant._tag != "None" {
    print("[PANIC] Option.as_None: called on " + variant._tag)
    return null
  }
  return null  // No fields to return
}
```

### 3.7 Macro Implementation Location

**File**: `src/macro/enum_macro.rs` (new file)

```rust
use crate::ast::{ASTNode, EnumVariant, EnumField};

/// Expand @enum declaration into box + static box
pub fn expand_enum(name: &str, variants: &[EnumVariant]) -> Vec<ASTNode> {
    let box_name = format!("{}Box", name);

    // 1. Generate data box
    let data_box = generate_data_box(&box_name, variants);

    // 2. Generate static box
    let static_box = generate_static_box(name, &box_name, variants);

    vec![data_box, static_box]
}

fn generate_data_box(box_name: &str, variants: &[EnumVariant]) -> ASTNode {
    // Collect all unique fields from all variants
    let mut all_fields = vec!["_tag".to_string()];

    for variant in variants {
        for field in &variant.fields {
            let field_name = if variant.fields.len() == 1 && field.name == "value" {
                "_value".to_string()
            } else {
                format!("_{}", field.name)
            };

            if !all_fields.contains(&field_name) {
                all_fields.push(field_name);
            }
        }
    }

    // Generate birth method
    let birth_body = generate_birth_method(&all_fields);

    // Build BoxDeclaration AST node
    ASTNode::BoxDeclaration {
        name: box_name.to_string(),
        fields: all_fields,
        methods: hashmap! {
            "birth".to_string() => birth_body,
        },
        // ... other fields with defaults ...
    }
}

fn generate_static_box(enum_name: &str, box_name: &str, variants: &[EnumVariant]) -> ASTNode {
    let mut methods = HashMap::new();

    for variant in variants {
        // Generate constructor
        let constructor = generate_constructor(box_name, &variant.name, &variant.fields);
        methods.insert(variant.name.clone(), constructor);

        // Generate is_* helper
        let is_helper = generate_is_helper(&variant.name);
        methods.insert(format!("is_{}", variant.name), is_helper);

        // Generate as_* helper
        let as_helper = generate_as_helper(enum_name, &variant.name, &variant.fields);
        methods.insert(format!("as_{}", variant.name), as_helper);
    }

    ASTNode::BoxDeclaration {
        name: enum_name.to_string(),
        is_static: true,
        methods,
        // ... other fields with defaults ...
    }
}
```

### 3.8 Integration Point

**File**: `src/macro/mod.rs`

```rust
mod enum_macro;

pub fn expand_macros(ast: ASTNode) -> ASTNode {
    // ... existing macro expansion ...

    // Apply enum expansion
    ast = enum_macro::expand_all_enums(ast);

    // ... continue with other macros ...
}
```

---

## 4. Edge Cases

### 4.1 Single Variant Enum

**Input**:
```hakorune
@enum Always {
  Value(data)
}
```

**Output**:
```hakorune
box AlwaysBox {
  _tag: StringBox
  _data: Box

  birth() {
    me._tag = ""
    me._data = null
  }
}

static box Always {
  Value(d) {
    local inst = new AlwaysBox()
    inst._tag = "Value"
    inst._data = d
    return inst
  }

  is_Value(variant) { return variant._tag == "Value" }

  as_Value(variant) {
    if variant._tag != "Value" {
      print("[PANIC] Always.as_Value: called on " + variant._tag)
      return null
    }
    return variant._data
  }
}
```

**Note**: Still generates all helpers for consistency.

### 4.2 Zero-Field Variant

**Input**:
```hakorune
@enum Option {
  Some(value)
  None
}
```

**Output**:
```hakorune
box OptionBox {
  _tag: StringBox
  _value: Box

  birth() {
    me._tag = ""
    me._value = null
  }
}

static box Option {
  Some(v) {
    local opt = new OptionBox()
    opt._tag = "Some"
    opt._value = v
    return opt
  }

  None() {
    local opt = new OptionBox()
    opt._tag = "None"
    return opt
  }

  is_Some(variant) { return variant._tag == "Some" }
  is_None(variant) { return variant._tag == "None" }

  as_Some(variant) {
    if variant._tag != "Some" {
      print("[PANIC] Option.as_Some: called on " + variant._tag)
      return null
    }
    return variant._value
  }

  as_None(variant) {
    if variant._tag != "None" {
      print("[PANIC] Option.as_None: called on " + variant._tag)
      return null
    }
    return null
  }
}
```

### 4.3 Multi-Field Variant

**Input**:
```hakorune
@enum Triple {
  One(a)
  Two(a, b)
  Three(a, b, c)
}
```

**Output**:
```hakorune
box TripleBox {
  _tag: StringBox
  _a: Box
  _b: Box
  _c: Box

  birth() {
    me._tag = ""
    me._a = null
    me._b = null
    me._c = null
  }
}

static box Triple {
  One(a) {
    local inst = new TripleBox()
    inst._tag = "One"
    inst._a = a
    return inst
  }

  Two(a, b) {
    local inst = new TripleBox()
    inst._tag = "Two"
    inst._a = a
    inst._b = b
    return inst
  }

  Three(a, b, c) {
    local inst = new TripleBox()
    inst._tag = "Three"
    inst._a = a
    inst._b = b
    inst._c = c
    return inst
  }

  is_One(variant) { return variant._tag == "One" }
  is_Two(variant) { return variant._tag == "Two" }
  is_Three(variant) { return variant._tag == "Three" }

  as_One(variant) {
    if variant._tag != "One" {
      print("[PANIC] Triple.as_One: called on " + variant._tag)
      return null
    }
    return variant._a
  }

  as_Two(variant) {
    if variant._tag != "Two" {
      print("[PANIC] Triple.as_Two: called on " + variant._tag)
      return null
    }
    local result = new ArrayBox()
    result.push(variant._a)
    result.push(variant._b)
    return result
  }

  as_Three(variant) {
    if variant._tag != "Three" {
      print("[PANIC] Triple.as_Three: called on " + variant._tag)
      return null
    }
    local result = new ArrayBox()
    result.push(variant._a)
    result.push(variant._b)
    result.push(variant._c)
    return result
  }
}
```

### 4.4 Field Name Conflicts

**Problem**: Different variants use same field name with different semantics.

**Input**:
```hakorune
@enum Conflict {
  TypeA(value)
  TypeB(value)
}
```

**Resolution**: Both variants use `_value` field (shared storage).

**Output**:
```hakorune
box ConflictBox {
  _tag: StringBox
  _value: Box

  birth() {
    me._tag = ""
    me._value = null
  }
}

static box Conflict {
  TypeA(v) {
    local inst = new ConflictBox()
    inst._tag = "TypeA"
    inst._value = v
    return inst
  }

  TypeB(v) {
    local inst = new ConflictBox()
    inst._tag = "TypeB"
    inst._value = v
    return inst
  }

  is_TypeA(variant) { return variant._tag == "TypeA" }
  is_TypeB(variant) { return variant._tag == "TypeB" }

  as_TypeA(variant) {
    if variant._tag != "TypeA" {
      print("[PANIC] Conflict.as_TypeA: called on " + variant._tag)
      return null
    }
    return variant._value
  }

  as_TypeB(variant) {
    if variant._tag != "TypeB" {
      print("[PANIC] Conflict.as_TypeB: called on " + variant._tag)
      return null
    }
    return variant._value
  }
}
```

**Note**: This is intentional - field reuse is OK because tag discriminates.

### 4.5 Nested Enum (Not Supported in Phase 1)

**Input**:
```hakorune
@enum Outer {
  @enum Inner {  // ERROR
    A
    B
  }
  C
}
```

**Error**: "Nested @enum declarations are not supported"

**Workaround**: Declare enums separately.

### 4.6 Reserved Names

**Problem**: Variant name conflicts with built-in methods.

**Prohibited variant names**:
- `birth`
- `fini`
- `new` (reserved keyword)
- `me` (reserved keyword)
- `this` (reserved keyword)

**Error**: "Variant name '{name}' is reserved"

---

## 5. Test Suite Design

### 5.1 Test File Structure

```
apps/lib/boxes/tests/
├── enum_basic_test.hako              # Test 1-3
├── enum_multi_variant_test.hako      # Test 4
├── enum_zero_field_test.hako         # Test 5
├── enum_multi_field_test.hako        # Test 6
├── enum_helpers_test.hako            # Test 7-8
├── enum_pattern_match_test.hako      # Test 9
├── enum_integration_test.hako        # Test 10
└── enum_real_world_ast_test.hako     # Test 11
```

### 5.2 Test Cases

#### Test 1: Basic 2-Variant Enum (Result-like)

**File**: `enum_basic_test.hako`

```hakorune
@enum Result {
  Ok(value)
  Err(error)
}

static box Main {
  main() {
    local r1 = Result.Ok(42)
    print(r1._tag)  // "Ok"

    local r2 = Result.Err("failed")
    print(r2._tag)  // "Err"

    return 0
  }
}
```

**Expected Output**:
```
Ok
Err
```

#### Test 2: Basic 2-Variant Enum (Option-like)

**File**: `enum_basic_test.hako` (additional test)

```hakorune
@enum Option {
  Some(value)
  None
}

static box Main {
  main() {
    local opt1 = Option.Some(100)
    print(opt1._tag)  // "Some"
    print(opt1._value)  // "100"

    local opt2 = Option.None()
    print(opt2._tag)  // "None"

    return 0
  }
}
```

**Expected Output**:
```
Some
100
None
```

#### Test 3: Constructor Field Assignment

**File**: `enum_basic_test.hako` (additional test)

```hakorune
@enum Result {
  Ok(value)
  Err(error)
}

static box Main {
  main() {
    local r = Result.Ok("success")

    if r._tag == "Ok" {
      print(r._value)  // "success"
    }

    if r._tag != "Err" {
      print("not error")  // "not error"
    }

    return 0
  }
}
```

**Expected Output**:
```
success
not error
```

#### Test 4: 3+ Variant Enum

**File**: `enum_multi_variant_test.hako`

```hakorune
@enum Status {
  Pending
  Running(task_id)
  Success(result)
  Failed(error)
}

static box Main {
  main() {
    local s1 = Status.Pending()
    local s2 = Status.Running(123)
    local s3 = Status.Success("done")
    local s4 = Status.Failed("timeout")

    print(s1._tag)  // "Pending"
    print(s2._tag)  // "Running"
    print(s2._task_id)  // "123"
    print(s3._tag)  // "Success"
    print(s3._result)  // "done"
    print(s4._tag)  // "Failed"
    print(s4._error)  // "timeout"

    return 0
  }
}
```

**Expected Output**:
```
Pending
Running
123
Success
done
Failed
timeout
```

#### Test 5: Variant with 0 Fields

**File**: `enum_zero_field_test.hako`

```hakorune
@enum Flag {
  On
  Off
}

static box Main {
  main() {
    local f1 = Flag.On()
    local f2 = Flag.Off()

    print(f1._tag)  // "On"
    print(f2._tag)  // "Off"

    return 0
  }
}
```

**Expected Output**:
```
On
Off
```

#### Test 6: Variant with Multiple Fields

**File**: `enum_multi_field_test.hako`

```hakorune
@enum Point {
  Point2D(x, y)
  Point3D(x, y, z)
}

static box Main {
  main() {
    local p2 = Point.Point2D(10, 20)
    local p3 = Point.Point3D(1, 2, 3)

    print(p2._tag)  // "Point2D"
    print(p2._x)    // "10"
    print(p2._y)    // "20"

    print(p3._tag)  // "Point3D"
    print(p3._x)    // "1"
    print(p3._y)    // "2"
    print(p3._z)    // "3"

    return 0
  }
}
```

**Expected Output**:
```
Point2D
10
20
Point3D
1
2
3
```

#### Test 7: is_* Helper Usage

**File**: `enum_helpers_test.hako`

```hakorune
@enum Result {
  Ok(value)
  Err(error)
}

static box Main {
  main() {
    local r1 = Result.Ok(42)
    local r2 = Result.Err("fail")

    if Result.is_Ok(r1) {
      print("r1 is Ok")  // "r1 is Ok"
    }

    if Result.is_Err(r2) {
      print("r2 is Err")  // "r2 is Err"
    }

    if Result.is_Ok(r2) {
      print("r2 is Ok")  // (not printed)
    }

    return 0
  }
}
```

**Expected Output**:
```
r1 is Ok
r2 is Err
```

#### Test 8: as_* Helper Success

**File**: `enum_helpers_test.hako` (additional test)

```hakorune
@enum Result {
  Ok(value)
  Err(error)
}

static box Main {
  main() {
    local r = Result.Ok(100)

    local val = Result.as_Ok(r)
    print(val)  // "100"

    return 0
  }
}
```

**Expected Output**:
```
100
```

#### Test 9: as_* Helper Panic (Wrong Variant)

**File**: `enum_helpers_test.hako` (additional test)

```hakorune
@enum Result {
  Ok(value)
  Err(error)
}

static box Main {
  main() {
    local r = Result.Err("failed")

    local val = Result.as_Ok(r)
    // Expected: "[PANIC] Result.as_Ok: called on Err"
    // Returns: null

    if val == null {
      print("got null after panic")
    }

    return 0
  }
}
```

**Expected Output**:
```
[PANIC] Result.as_Ok: called on Err
got null after panic
```

#### Test 10: Pattern Matching Preparation

**File**: `enum_pattern_match_test.hako`

```hakorune
@enum Option {
  Some(value)
  None
}

static box Main {
  process_option(opt) {
    if opt._tag == "Some" {
      print("Has value: " + opt._value)
    }

    if opt._tag == "None" {
      print("No value")
    }
  }

  main() {
    local opt1 = Option.Some(42)
    local opt2 = Option.None()

    me.process_option(opt1)  // "Has value: 42"
    me.process_option(opt2)  // "No value"

    return 0
  }
}
```

**Expected Output**:
```
Has value: 42
No value
```

#### Test 11: Integration with Existing Option/Result

**File**: `enum_integration_test.hako`

**Goal**: Test @enum-generated Result alongside old ResultBox.

```hakorune
using std.result as OldResult

@enum Result {
  Ok(value)
  Err(error)
}

static box Main {
  main() {
    local old_r = OldResult.ok(100)
    local new_r = Result.Ok(200)

    print(old_r.value())  // "100"
    print(Result.as_Ok(new_r))  // "200"

    return 0
  }
}
```

**Expected Output**:
```
100
200
```

#### Test 12: Complex Real-World Case (AST Node Example)

**File**: `enum_real_world_ast_test.hako`

```hakorune
@enum ASTNode {
  Literal(value)
  Variable(name)
  BinaryOp(op, left, right)
  UnaryOp(op, operand)
}

static box Main {
  main() {
    local lit = ASTNode.Literal(42)
    local var = ASTNode.Variable("x")
    local binop = ASTNode.BinaryOp("+", lit, var)

    print(ASTNode.is_Literal(lit))  // "1" (true)
    print(ASTNode.is_BinaryOp(binop))  // "1" (true)

    if ASTNode.is_BinaryOp(binop) {
      local parts = ASTNode.as_BinaryOp(binop)
      // parts is ArrayBox: ["+", lit, var]
      print(parts.get(0))  // "+"
    }

    return 0
  }
}
```

**Expected Output**:
```
1
1
+
```

### 5.3 Negative Tests

#### Test N1: Duplicate Variant Names

**File**: `enum_error_duplicate_variant.hako`

```hakorune
@enum Bad {
  Ok(value)
  Ok(other)  // ERROR: Duplicate variant
}
```

**Expected Error**: "Duplicate variant name 'Ok' in enum at line X"

#### Test N2: Reserved Variant Name

**File**: `enum_error_reserved_name.hako`

```hakorune
@enum Bad {
  birth(value)  // ERROR: Reserved name
}
```

**Expected Error**: "Variant name 'birth' is reserved"

#### Test N3: Empty Enum

**File**: `enum_error_empty.hako`

```hakorune
@enum Empty {
  // ERROR: No variants
}
```

**Expected Error**: "Empty enum declaration at line X"

### 5.4 Test Runner Script

**File**: `tools/run_enum_tests.sh`

```bash
#!/bin/bash
set -e

HAKO="./target/release/hako"

echo "=== @enum Macro Test Suite ==="

# Basic tests
echo "[1/12] Basic 2-variant enum (Result)..."
$HAKO apps/lib/boxes/tests/enum_basic_test.hako

echo "[2/12] Basic 2-variant enum (Option)..."
$HAKO apps/lib/boxes/tests/enum_basic_test.hako

echo "[3/12] Constructor field assignment..."
$HAKO apps/lib/boxes/tests/enum_basic_test.hako

echo "[4/12] Multi-variant enum (4 variants)..."
$HAKO apps/lib/boxes/tests/enum_multi_variant_test.hako

echo "[5/12] Zero-field variant..."
$HAKO apps/lib/boxes/tests/enum_zero_field_test.hako

echo "[6/12] Multi-field variant..."
$HAKO apps/lib/boxes/tests/enum_multi_field_test.hako

echo "[7/12] is_* helper usage..."
$HAKO apps/lib/boxes/tests/enum_helpers_test.hako

echo "[8/12] as_* helper success..."
$HAKO apps/lib/boxes/tests/enum_helpers_test.hako

echo "[9/12] as_* helper panic (wrong variant)..."
$HAKO apps/lib/boxes/tests/enum_helpers_test.hako

echo "[10/12] Pattern matching preparation..."
$HAKO apps/lib/boxes/tests/enum_pattern_match_test.hako

echo "[11/12] Integration with existing Option/Result..."
$HAKO apps/lib/boxes/tests/enum_integration_test.hako

echo "[12/12] Real-world AST node example..."
$HAKO apps/lib/boxes/tests/enum_real_world_ast_test.hako

# Negative tests
echo "[N1/3] Duplicate variant names..."
$HAKO apps/lib/boxes/tests/enum_error_duplicate_variant.hako 2>&1 | grep -q "Duplicate variant" && echo "  ✓ Error caught" || (echo "  ✗ Error not caught"; exit 1)

echo "[N2/3] Reserved variant name..."
$HAKO apps/lib/boxes/tests/enum_error_reserved_name.hako 2>&1 | grep -q "reserved" && echo "  ✓ Error caught" || (echo "  ✗ Error not caught"; exit 1)

echo "[N3/3] Empty enum..."
$HAKO apps/lib/boxes/tests/enum_error_empty.hako 2>&1 | grep -q "Empty enum" && echo "  ✓ Error caught" || (echo "  ✗ Error not caught"; exit 1)

echo "=== All tests passed ==="
```

---

## 6. Implementation Task Breakdown

### Day 1: Parser Changes + AST Nodes (6-8 hours)

**Morning (3-4h)**:
- [ ] Add `TokenType::At` if not exists
- [ ] Implement `parse_enum_declaration()` in `src/parser/declarations/enum_parser.rs`
- [ ] Implement `parse_enum_variant()` helper
- [ ] Add `EnumDeclaration`, `EnumVariant`, `EnumField` to `src/ast.rs`

**Afternoon (3-4h)**:
- [ ] Add validation (duplicate variants, empty enum, reserved names)
- [ ] Add error types to `ParseError` enum
- [ ] Write parser unit tests
- [ ] Test parser with minimal inputs (no macro expansion yet)

**Success Criteria**:
- Parser can parse `@enum` syntax without panicking
- AST nodes are created correctly
- Validation errors are caught

### Day 2: Macro Engine Integration + Code Generation (8-10 hours)

**Morning (4-5h)**:
- [ ] Create `src/macro/enum_macro.rs`
- [ ] Implement `expand_enum()` function
- [ ] Implement `generate_data_box()` - create box with fields
- [ ] Implement `generate_birth_method()` - initialize all fields to null

**Afternoon (4-5h)**:
- [ ] Implement `generate_static_box()` - create static box
- [ ] Implement `generate_constructor()` - one per variant
- [ ] Add macro invocation to `src/macro/mod.rs`
- [ ] Test expansion with debug prints (dump generated AST)

**Success Criteria**:
- `@enum Result { Ok(value) Err(error) }` expands to correct AST
- Generated AST can be printed back as code
- No panics during expansion

### Day 3: Helper Method Generation + Edge Cases (8-10 hours)

**Morning (4-5h)**:
- [ ] Implement `generate_is_helper()` - is_* methods
- [ ] Implement `generate_as_helper()` - as_* methods (single field)
- [ ] Implement multi-field `as_*` (returns ArrayBox)
- [ ] Handle 0-field variant `as_*` (returns null)

**Afternoon (4-5h)**:
- [ ] Test edge cases (single variant, zero fields, multi-field)
- [ ] Test field name conflicts (same name across variants)
- [ ] Add diagnostics/trace output (NYASH_MACRO_TRACE=1)
- [ ] Verify generated code compiles to MIR

**Success Criteria**:
- All helper methods generate correctly
- Edge cases handled without errors
- Generated code compiles and runs

### Day 4: Test Suite (6-8 hours)

**Morning (3-4h)**:
- [ ] Write tests 1-6 (basic cases)
- [ ] Write tests 7-9 (helper methods)
- [ ] Verify all tests run and produce expected output

**Afternoon (3-4h)**:
- [ ] Write tests 10-12 (pattern matching, integration, real-world)
- [ ] Write negative tests (N1-N3)
- [ ] Create `tools/run_enum_tests.sh` runner script
- [ ] Run full test suite

**Success Criteria**:
- All 12 positive tests pass
- All 3 negative tests catch errors correctly
- Test runner script exits with 0

### Day 5: Smoke Tests + Integration (6-8 hours)

**Morning (3-4h)**:
- [ ] Add @enum tests to smoke test suite
- [ ] Run `tools/smokes/v2/run.sh --profile quick`
- [ ] Fix any integration issues
- [ ] Document known limitations

**Afternoon (3-4h)**:
- [ ] Update `CURRENT_TASK.md` with @enum status
- [ ] Update `CLAUDE.md` development log
- [ ] Create migration guide for Option/Result
- [ ] Review and commit

**Success Criteria**:
- Smoke tests pass
- Documentation updated
- Ready for production use

---

## 7. Integration with Existing Code

### 7.1 Migration Strategy for Option/Result

**Current State**:
- `apps/lib/boxes/option.hako` - uses `_is_some: IntegerBox`
- `apps/lib/boxes/result.hako` - uses `_ok: IntegerBox`
- 5+ files use these boxes

**Migration Plan**:

#### Phase 1: Parallel Existence (Week 1)
1. Keep existing `option.hako` and `result.hako`
2. Create new `@enum Option` and `@enum Result` in separate files:
   - `apps/lib/boxes/option_v2.hako`
   - `apps/lib/boxes/result_v2.hako`
3. Update `hako.toml`:
```toml
[modules.overrides]
std.option = "apps/lib/boxes/option.hako"      # Old version
std.option_v2 = "apps/lib/boxes/option_v2.hako" # New @enum version
std.result = "apps/lib/boxes/result.hako"
std.result_v2 = "apps/lib/boxes/result_v2.hako"
```

#### Phase 2: Gradual Migration (Week 2-3)
1. Migrate test files first
2. Migrate non-critical tools
3. Verify behavior matches old version

#### Phase 3: Deprecation (Week 4)
1. Rename old files:
   - `option.hako` → `option_deprecated.hako`
   - `result.hako` → `result_deprecated.hako`
2. Rename new files:
   - `option_v2.hako` → `option.hako`
   - `result_v2.hako` → `result.hako`
3. Update all import sites

#### Phase 4: Cleanup (Week 5)
1. Delete deprecated files
2. Remove old module aliases
3. Update documentation

### 7.2 API Compatibility Matrix

| Method | Old Option | @enum Option | Compatible? |
|--------|-----------|-------------|-------------|
| `Option.some(v)` | ✓ | `Option.Some(v)` | **Different name** |
| `Option.none()` | ✓ | `Option.None()` | **Different name** |
| `opt.is_some()` | ✓ | `Option.is_Some(opt)` | **Different signature** |
| `opt.unwrap()` | ✓ | `Option.as_Some(opt)` | **Different name** |
| `opt._value` | ✓ | ✓ | ✓ Compatible |
| `opt._is_some` | ✓ | ✗ (uses `_tag`) | **Breaking** |

**Conclusion**: **NOT backward compatible**. Requires explicit migration.

### 7.3 Wrapper Strategy for Compatibility

**Option**: Create compatibility wrapper

**File**: `apps/lib/boxes/option_compat.hako`

```hakorune
@enum OptionInternal {
  Some(value)
  None
}

static box Option {
  // Old API
  some(v) { return OptionInternal.Some(v) }
  none() { return OptionInternal.None() }

  // Adapter for instance methods
  is_some(opt) { return opt._tag == "Some" }
  is_none(opt) { return opt._tag == "None" }
  unwrap(opt) { return OptionInternal.as_Some(opt) }
  unwrap_or(opt, def) {
    if opt._tag == "Some" {
      return opt._value
    }
    return def
  }
}

// Create facade OptionBox (for `new OptionBox()` compatibility)
box OptionBox {
  _internal: Box

  birth() {
    me._internal = OptionInternal.None()
  }

  is_some() { return me._internal._tag == "Some" }
  is_none() { return me._internal._tag == "None" }
  unwrap() { return OptionInternal.as_Some(me._internal) }
}
```

**Trade-off**: Adds complexity but maintains backward compatibility.

### 7.4 Migration Guide Document

**File**: `docs/guides/enum-migration-guide.md`

**Contents**:
1. Why migrate to @enum?
2. API differences table
3. Step-by-step migration process
4. Common pitfalls
5. Examples (before/after)

---

## 8. Risk Analysis

### 8.1 Parser Conflicts

**Risk**: `@` token conflicts with other syntax (e.g., `@local` sugar)

**Mitigation**:
- Check existing `@` usage with grep
- Use lookahead: `@enum` specifically (not just `@`)
- Add parser tests for ambiguous cases

**Action Items**:
- [ ] Grep codebase for existing `@` usage
- [ ] Test parser with `@local` and `@enum` in same file
- [ ] Document precedence rules

### 8.2 Macro Expansion Order

**Risk**: @enum expansion happens before/after other macros, causing issues

**Current Macro Order** (from CLAUDE.md):
```rust
// src/macro/mod.rs
1. @local expansion
2. Map literal sugar
3. (add @enum here?)
```

**Mitigation**:
- Run @enum expansion **before** other macros (early in pipeline)
- @enum generates pure box definitions (no further macro dependencies)

**Action Items**:
- [ ] Verify macro execution order in `src/macro/mod.rs`
- [ ] Test interaction with map literal sugar
- [ ] Test interaction with @local sugar

### 8.3 Field Naming Conflicts

**Risk**: Generated `_tag`, `_value` conflict with user-defined fields

**Current Design**: Fields always prefixed with `_` (e.g., `_tag`, `_value`)

**Mitigation**:
- Document that `_`-prefixed fields are reserved for enum internals
- Add validation: reject variant field names starting with `_`
- Generate error: "Variant field names cannot start with '_' (reserved)"

**Action Items**:
- [ ] Add validation in `parse_enum_variant()`
- [ ] Test case: `Variant(_tag)` → error
- [ ] Document in language guide

### 8.4 Performance Considerations

**Risk**: String tag comparison slower than integer flag comparison

**Analysis**:
- Old: `if opt._is_some == 1` (integer compare, ~1 cycle)
- New: `if opt._tag == "Some"` (string compare, ~N cycles where N = length)

**Mitigation**:
- **VM**: String comparison already optimized in StringBox
- **LLVM**: Compiler can optimize string literals to pointer comparison
- **Measurement**: Add benchmark comparing old vs new

**Action Items**:
- [ ] Benchmark: Old Result vs @enum Result (1M operations)
- [ ] Profile: Measure hotspot in string comparison
- [ ] Document performance characteristics

**Expected Result**: <10% slowdown (acceptable for MVP)

### 8.5 Debugging Challenges

**Risk**: Expanded code hard to debug (macro generates verbose output)

**Mitigation**:
- Add `NYASH_MACRO_TRACE=1` to dump expansion
- Add `--dump-mir` flag support (show post-expansion code)
- Preserve source spans in generated AST
- Add comment markers in generated code:
  ```hakorune
  // BEGIN @enum Result expansion
  box ResultBox { ... }
  // END @enum Result expansion
  ```

**Action Items**:
- [ ] Implement trace output in `enum_macro.rs`
- [ ] Test `--dump-mir` with @enum code
- [ ] Add source span preservation
- [ ] Add debug comments to generated AST

### 8.6 Error Message Quality

**Risk**: Users get confusing errors from generated code (not original @enum)

**Example Bad Error**:
```
Error at line 523: Field '_tag' not found in ResultBox
  (User wrote @enum at line 10, error points to generated code at line 523)
```

**Mitigation**:
- Preserve original span in AST nodes
- Error formatter shows original @enum location
- Add note: "in expansion of @enum Result at line 10"

**Action Items**:
- [ ] Test error reporting with intentionally broken @enum
- [ ] Verify span preservation in macro expansion
- [ ] Update error formatter to show macro context

### 8.7 Macro-Generated Code Stability

**Risk**: Generated code doesn't compile to MIR (syntax errors, etc.)

**Mitigation**:
- Generate only well-tested AST patterns (box, static box, simple statements)
- Add roundtrip test: expand → parse → expand (should be stable)
- Add integration test: @enum → MIR → VM execution

**Action Items**:
- [ ] Test: Expand @enum Result → dump AST → parse again
- [ ] Test: @enum → MIR JSON → verify structure
- [ ] Test: @enum → VM execution → verify behavior

---

## 9. Success Criteria

**Phase 1 Complete When**:
1. All 12 positive tests pass ✓
2. All 3 negative tests catch errors ✓
3. Smoke tests pass ✓
4. Documentation complete ✓
5. Performance benchmark shows <10% regression ✓
6. Integration guide written ✓

**Phase 2 (Future)**:
1. Pattern matching syntax (`match` expressions)
2. Exhaustiveness checking
3. Performance optimization (intern tag strings)

---

## 10. Future Enhancements (Post-MVP)

### 10.1 Pattern Matching Integration

**Syntax** (future):
```hakorune
match result {
  Ok(value) => print("Success: " + value)
  Err(error) => print("Error: " + error)
}
```

**Desugaring** (future):
```hakorune
if result._tag == "Ok" {
  local value = result._value
  print("Success: " + value)
}
if result._tag == "Err" {
  local error = result._error
  print("Error: " + error)
}
```

### 10.2 Exhaustiveness Checking

**Goal**: Compiler ensures all variants handled.

**Example**:
```hakorune
match option {
  Some(v) => print(v)
  // WARNING: Missing case for 'None'
}
```

### 10.3 String Interning for Tags

**Optimization**: Intern tag strings to enable pointer comparison.

**Implementation**:
- Tag strings stored in intern table
- Comparison becomes pointer equality (1 cycle)
- Backward compatible (still strings at API level)

---

## 11. References

### 11.1 Existing Code
- `apps/lib/boxes/option.hako` - Current Option implementation
- `apps/lib/boxes/result.hako` - Current Result implementation
- `src/parser/mod.rs` - Parser entry point
- `src/macro/macro_box.rs` - Macro system infrastructure

### 11.2 Documentation
- `CLAUDE.md` - Development log and context
- `docs/reference/language/quick-reference.md` - Language syntax
- `docs/development/roadmap/phases/phase-20-variant-box/` - Variant box proposals

### 11.3 Test Examples
- `apps/selfhost/test_*.hako` - Existing test patterns
- `tools/smokes/v2/` - Smoke test infrastructure

---

## Appendix A: Complete Example Expansion

**Input**:
```hakorune
@enum Option {
  Some(value)
  None
}
```

**Complete Expansion** (with all helpers):
```hakorune
// Data box
box OptionBox {
  _tag: StringBox
  _value: Box

  birth() {
    me._tag = ""
    me._value = null
  }
}

// Static box with constructors and helpers
static box Option {
  // Constructors
  Some(v) {
    local opt = new OptionBox()
    opt._tag = "Some"
    opt._value = v
    return opt
  }

  None() {
    local opt = new OptionBox()
    opt._tag = "None"
    return opt
  }

  // is_* helpers
  is_Some(variant) {
    return variant._tag == "Some"
  }

  is_None(variant) {
    return variant._tag == "None"
  }

  // as_* helpers
  as_Some(variant) {
    if variant._tag != "Some" {
      print("[PANIC] Option.as_Some: called on " + variant._tag)
      return null
    }
    return variant._value
  }

  as_None(variant) {
    if variant._tag != "None" {
      print("[PANIC] Option.as_None: called on " + variant._tag)
      return null
    }
    return null
  }
}
```

**Total Lines**: 47 (for 2-variant enum)

---

## Appendix B: Implementation Checklist

### Parser (src/parser/)
- [ ] `TokenType::At` exists or added
- [ ] `parse_enum_declaration()` implemented
- [ ] `parse_enum_variant()` implemented
- [ ] Validation: duplicate variants
- [ ] Validation: empty enum
- [ ] Validation: reserved names
- [ ] Validation: field names (no `_` prefix)
- [ ] Error types added to `ParseError`
- [ ] Parser unit tests

### AST (src/ast.rs)
- [ ] `EnumDeclaration` variant added
- [ ] `EnumVariant` struct defined
- [ ] `EnumField` struct defined
- [ ] Span preservation

### Macro (src/macro/)
- [ ] `enum_macro.rs` created
- [ ] `expand_enum()` function
- [ ] `generate_data_box()` function
- [ ] `generate_birth_method()` function
- [ ] `generate_static_box()` function
- [ ] `generate_constructor()` function
- [ ] `generate_is_helper()` function
- [ ] `generate_as_helper()` (single field)
- [ ] `generate_as_helper()` (multi-field)
- [ ] Integration with `src/macro/mod.rs`
- [ ] Trace output (NYASH_MACRO_TRACE=1)

### Tests (apps/lib/boxes/tests/)
- [ ] Test 1: Basic Result
- [ ] Test 2: Basic Option
- [ ] Test 3: Constructor field assignment
- [ ] Test 4: 3+ variants
- [ ] Test 5: Zero-field variant
- [ ] Test 6: Multi-field variant
- [ ] Test 7: is_* helpers
- [ ] Test 8: as_* success
- [ ] Test 9: as_* panic
- [ ] Test 10: Pattern matching prep
- [ ] Test 11: Integration
- [ ] Test 12: Real-world AST
- [ ] Test N1: Duplicate variants
- [ ] Test N2: Reserved names
- [ ] Test N3: Empty enum
- [ ] Test runner script

### Documentation
- [ ] Migration guide
- [ ] Language reference update
- [ ] CURRENT_TASK.md update
- [ ] CLAUDE.md update
- [ ] Performance benchmark results

### Integration
- [ ] Smoke tests pass
- [ ] hako.toml updated (if needed)
- [ ] No regressions in existing tests
- [ ] Performance <10% regression

---

**End of Specification**
