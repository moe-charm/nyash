# @enum Implementation Module Architecture

**Goal**: Production-ready, clean, maintainable architecture following Hakorune conventions

**Date**: 2025-10-08
**Status**: Design Phase - Ready for Implementation

---

## Executive Summary

This document defines the complete module placement strategy for @enum implementation (Day 1 scope). The design follows **Separation of Concerns** principle, integrates naturally with existing codebase patterns, and provides clear boundaries for testing and future expansion.

**Key Decisions**:
- Parser: `src/parser/declarations/enum_parser.rs` (NEW)
- AST: Add 3 structs to `src/ast.rs` (existing monolithic file)
- Macro Expander: `src/macro/enum_expander.rs` (NEW - Rust for Day 1)
- Tests: `apps/tests/enum/` (NEW directory for integration tests)
- Docs: `docs/reference/language/enum-syntax.md` (NEW user guide)

---

## 1. Current Project Structure Analysis

### src/ Structure (Rust Implementation)
```
src/
├── lib.rs                      [exports all modules]
├── main.rs                     [CLI entry point]
├── ast.rs                      [monolithic AST definitions - 16,868 bytes]
├── ast/                        [utility modules]
│   ├── nodes.rs               [additional AST utilities]
│   ├── span.rs                [Span implementation]
│   └── utils.rs               [AST helpers]
├── parser/
│   ├── mod.rs                 [main parser - 15,287 bytes]
│   ├── common.rs              [ParserUtils trait]
│   ├── cursor.rs              [token cursor]
│   ├── declarations/          [DECLARATION PARSERS - TARGET LOCATION]
│   │   ├── mod.rs            [exports: box_def, box_definition, static_box, etc.]
│   │   ├── box_definition.rs [box/static box parser]
│   │   ├── box_def/          [box parser internals]
│   │   ├── static_box.rs     [static box parser]
│   │   ├── static_def/       [static box internals]
│   │   ├── dependency_helpers.rs  [circular dependency detection]
│   │   └── flow.rs           [control flow parsing]
│   ├── expr/
│   ├── expressions.rs
│   ├── items/
│   └── statements/
├── macro/                      [MACRO SYSTEM - TARGET LOCATION]
│   ├── mod.rs                 [macro engine entry]
│   ├── engine.rs              [macro expansion engine]
│   ├── macro_box.rs           [built-in macro boxes]
│   ├── macro_box_ny.rs        [Hakorune macro boxes]
│   ├── ast_json.rs            [AST↔JSON conversion]
│   ├── pattern.rs             [pattern matching]
│   └── ctx.rs                 [macro context]
├── tokenizer/
├── mir/
├── backend/
└── ...
```

**Pattern Observed**:
- **AST**: Monolithic `ast.rs` (16KB) with some utilities in `ast/` subdirectory
- **Parser**: Modular by concern (`declarations/`, `expressions/`, `statements/`)
- **Macro**: Flat structure in `macro/` directory (6-7 files)

### apps/ Structure (Hakorune Programs)
```
apps/
├── lib/
│   └── boxes/                  [standard library boxes]
│       ├── array_std.hako
│       ├── console_std.hako
│       ├── map_std.hako
│       ├── string_std.hako
│       ├── option.hako        [❗ 2-variant enum-like, 1755 bytes]
│       └── result.hako        [❗ 2-variant enum-like, 1874 bytes]
├── tests/                      [integration test programs]
│   ├── (48 subdirectories)
│   ├── core/                  [core language tests]
│   ├── array_literal_basic.nyash
│   └── ...
├── macros/                     [macro implementations]
│   ├── examples/
│   └── selfhost_min/
├── selfhost/                   [self-hosting compiler]
├── selfhost-compiler/
├── benchmarks/
└── ...
```

**Pattern Observed**:
- **Boxes**: Standard library in `apps/lib/boxes/` (`.hako` extension)
- **Tests**: Flat + categorized subdirectories in `apps/tests/`
- **Macros**: Some exist in `apps/macros/` but primary impl is Rust (`src/macro/`)

---

## 2. Architecture Decision: Hybrid Modular Approach

### Rationale
Based on existing patterns:
1. **Parser follows declarations/ pattern** - Add `enum_parser.rs` to existing `src/parser/declarations/`
2. **AST stays monolithic** - Add structs to `src/ast.rs` (consistent with current 16KB file)
3. **Macro in Rust (Day 1)** - Fast prototyping, type-safe, easier debugging
4. **Tests in dedicated subdirectory** - `apps/tests/enum/` for clear organization

### Benefits
- ✅ **Zero architectural surprises** - follows existing patterns exactly
- ✅ **Minimal file changes** - only 4 new files + modifications
- ✅ **Easy to find** - predictable locations for future developers
- ✅ **Testable** - clear separation between parser/expander/integration tests

---

## 3. Module Placement Design

### 3.1 Parser Module (Token → AST)

**Location**: `src/parser/declarations/enum_parser.rs` (NEW)

**Responsibility**: Parse `@enum Name { ... }` syntax into AST nodes

**Exports**:
```rust
// src/parser/declarations/enum_parser.rs

use crate::ast::{ASTNode, EnumDeclaration, EnumVariant, EnumField, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;

impl NyashParser {
    /// Parse @enum declaration
    ///
    /// Syntax:
    ///   @enum EnumName {
    ///     VariantA(field1: TypeBox, field2: TypeBox)
    ///     VariantB()
    ///     VariantC(value: IntegerBox)
    ///   }
    pub(crate) fn parse_enum_declaration(&mut self) -> Result<ASTNode, ParseError> {
        // Implementation here
    }
}

#[cfg(test)]
mod tests {
    // Unit tests for enum parsing only
    // Test AST node construction, error cases, edge cases
}
```

**Integration**:
```rust
// src/parser/declarations/mod.rs
pub mod enum_parser;  // ADD THIS LINE

// src/parser/mod.rs (or statements.rs)
// In parse_statement() or parse_top_level():
if self.match_token(&TokenType::MacroKeyword) &&
   self.peek_ahead_matches("enum") {
    return self.parse_enum_declaration();
}
```

**Dependencies**:
- `crate::ast::{EnumDeclaration, EnumVariant, EnumField}` (new AST nodes)
- `crate::tokenizer::TokenType::MacroKeyword` (existing or new)
- `crate::parser::common::ParserUtils` (existing trait)

---

### 3.2 AST Module (Data Structures)

**Location**: `src/ast.rs` (MODIFY - add 3 structs)

**Responsibility**: Define AST node structures for enum declarations

**New Structures**:
```rust
// src/ast.rs

/// Enum declaration AST node (parsed from @enum syntax)
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDeclaration {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

/// Enum variant (e.g., "Some(value: T)" or "None()")
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<EnumField>,
    pub span: Span,
}

/// Enum variant field (e.g., "value: IntegerBox")
#[derive(Debug, Clone, PartialEq)]
pub struct EnumField {
    pub name: String,
    pub type_name: String,  // "IntegerBox", "StringBox", "T", etc.
    pub span: Span,
}

// Add to ASTNode enum:
pub enum ASTNode {
    // ... existing variants ...

    /// Enum declaration (macro syntax: @enum Name { ... })
    EnumDeclaration {
        name: String,
        variants: Vec<EnumVariant>,
        span: Span,
    },

    // ... rest of variants ...
}
```

**Rationale for Monolithic**:
- Current `ast.rs` is 16KB - still manageable size
- No `src/ast/declarations.rs` exists yet
- Adding 50-80 lines doesn't justify new file
- **Later** (Phase 19.2+): Consider `src/ast/declarations/` if file grows beyond 25KB

**Alternative (Future Consideration)**:
```
src/ast/
├── mod.rs              [main ASTNode enum + re-exports]
├── declarations.rs     [EnumDeclaration, BoxDeclaration, etc.]
├── expressions.rs      [ExpressionNode variants]
└── statements.rs       [StatementNode variants]
```

---

### 3.3 Tokenizer Integration

**Location**: `src/tokenizer/mod.rs` or equivalent (MODIFY)

**Responsibility**: Recognize `@enum` keyword

**Changes Required**:
```rust
// Option A: Reuse existing MacroKeyword token
// (if @enum, @macro, etc. all use same token)

// Option B: Add specific EnumKeyword token
pub enum TokenType {
    // ... existing tokens ...
    MacroKeyword,  // for @ prefix
    // ... or ...
    EnumKeyword,   // specifically for @enum
}

// In tokenizer loop:
'@' => {
    // Check if followed by 'enum'
    if self.peek_word() == "enum" {
        tokens.push(Token::new(TokenType::MacroKeyword, "@", line));
        tokens.push(Token::new(TokenType::Identifier, "enum", line));
    }
}
```

**Decision**: Use **Option A** (reuse `MacroKeyword`) for consistency with macro system.

---

### 3.4 Macro Expander Module (AST → Box AST)

**Location**: `src/macro/enum_expander.rs` (NEW)

**Responsibility**: Transform `EnumDeclaration` AST into `BoxDeclaration` + `StaticBoxDeclaration` AST

**Architecture** (Two-Layer):
```
Parser Layer:
  parse_enum_declaration()
    ↓ produces
  EnumDeclaration AST

Macro Expander Layer:
  expand_enum(EnumDeclaration)
    ↓ produces
  Vec<ASTNode> [BoxDeclaration, StaticBoxDeclaration]
```

**Implementation**:
```rust
// src/macro/enum_expander.rs

use crate::ast::{ASTNode, EnumDeclaration, EnumVariant, Span};

/// Expand @enum declaration into Box + StaticBox AST nodes
///
/// Transformation:
///   @enum Result { Ok(value: T), Err(error: E) }
///
/// Generates:
///   box ResultBox<T, E> {
///     variant: StringBox
///     ok_value: T
///     err_error: E
///
///     birth(variant, ...) { ... }
///     is_ok() { ... }
///     is_err() { ... }
///     unwrap() { ... }
///   }
///
///   static box Result {
///     Ok(value) { return new ResultBox("Ok", value, null) }
///     Err(error) { return new ResultBox("Err", null, error) }
///   }
pub fn expand_enum(decl: EnumDeclaration) -> Result<Vec<ASTNode>, EnumExpansionError> {
    let box_node = generate_box_declaration(&decl)?;
    let static_node = generate_static_box(&decl)?;

    Ok(vec![box_node, static_node])
}

fn generate_box_declaration(decl: &EnumDeclaration) -> Result<ASTNode, EnumExpansionError> {
    // Generate BoxDeclaration with:
    // - variant: StringBox field
    // - All variant fields (with nullable types for variants that don't use them)
    // - birth(variant, ...) constructor
    // - is_* query methods
    // - unwrap/unwrap_or/map/... utility methods
}

fn generate_static_box(decl: &EnumDeclaration) -> Result<ASTNode, EnumExpansionError> {
    // Generate StaticBoxDeclaration with:
    // - Constructor methods for each variant (Ok, Err, Some, None, etc.)
}

#[derive(Debug)]
pub enum EnumExpansionError {
    DuplicateVariant(String),
    DuplicateField(String),
    InvalidTypeName(String),
}

#[cfg(test)]
mod tests {
    // Unit tests for enum expansion logic
    // Test generated AST structure
    // Test error cases (duplicate variants, invalid names, etc.)
}
```

**Integration**:
```rust
// src/macro/engine.rs (MODIFY)
use crate::macro::enum_expander;

impl MacroEngine {
    pub fn expand(&mut self, ast: &ASTNode) -> (ASTNode, Vec<Patch>) {
        match ast {
            ASTNode::EnumDeclaration { .. } => {
                // Expand enum and replace node
                let expanded = enum_expander::expand_enum(ast)?;
                // Replace single EnumDeclaration with multiple nodes
            }
            // ... other macro expansions ...
        }
    }
}
```

**Rationale for Rust Implementation (Day 1-2)**:
- ✅ **Faster prototyping** - Rich type system, IDE support, compiler checks
- ✅ **Easier debugging** - Standard Rust tooling
- ✅ **Type-safe codegen** - Constructing AST nodes with compiler verification
- ❌ **Not dogfooding** - But acceptable for MVP

**Future Migration** (Phase 19.3+):
- Rewrite in Hakorune (`apps/macros/enum_macro.hako`)
- Demonstrate self-hosting capability
- String-based codegen (less type-safe but proves language capability)

---

### 3.5 Test Organization

**Structure**:
```
apps/tests/
└── enum/                          [NEW - dedicated enum tests]
    ├── test_enum_basic.hako       [2-variant Result-like]
    ├── test_enum_option.hako      [2-variant Option-like]
    ├── test_enum_multi.hako       [3+ variants]
    ├── test_enum_zero_field.hako  [unit variants only]
    ├── test_enum_is_methods.hako  [is_ok, is_err queries]
    ├── test_enum_unwrap.hako      [unwrap, unwrap_or]
    └── test_enum_error_cases.hako [duplicate variants, etc.]

src/parser/declarations/enum_parser.rs
  └── #[cfg(test)] mod tests { ... }  [parser unit tests]

src/macro/enum_expander.rs
  └── #[cfg(test)] mod tests { ... }  [expander unit tests]
```

**Test Separation**:

| Test Type | Location | What to Test |
|-----------|----------|--------------|
| **Parser Unit** | `src/parser/declarations/enum_parser.rs::tests` | AST construction, syntax errors, edge cases |
| **Expander Unit** | `src/macro/enum_expander.rs::tests` | Generated AST structure, expansion errors |
| **Integration** | `apps/tests/enum/*.hako` | Full pipeline: parse → expand → execute |

**Example Integration Test**:
```hakorune
// apps/tests/enum/test_enum_basic.hako

@enum Result {
    Ok(value: IntegerBox)
    Err(error: StringBox)
}

static box Main {
    main() {
        local r1
        r1 = Result.Ok(42)

        local r2
        r2 = Result.Err("oops")

        if r1.is_ok() {
            print("r1 is Ok")
        }

        if r2.is_err() {
            print("r2 is Err")
        }

        local val
        val = r1.unwrap()
        print(val)  // Should print 42

        return true
    }
}
```

**Test Execution**:
```bash
# Run parser unit tests
cargo test --lib enum_parser::tests

# Run expander unit tests
cargo test --lib enum_expander::tests

# Run integration tests
./target/release/hako apps/tests/enum/test_enum_basic.hako
```

---

### 3.6 Documentation Placement

**User-Facing Documentation**:
```
docs/
└── reference/
    └── language/
        └── enum-syntax.md         [NEW - user guide for @enum]
```

**Developer Documentation**:
```
docs/
└── development/
    └── roadmap/phases/phase-19-enum-match/
        ├── README.md              [EXISTS - project overview]
        ├── enum-module-architecture.md  [THIS FILE]
        └── enum-implementation-notes.md [NEW - implementation details]
```

**Generated Code Documentation**:
```
apps/lib/boxes/
├── option_generated.hako          [OPTION: Generated from @enum Option]
└── result_generated.hako          [OPTION: Generated from @enum Result]

# OR: Overwrite existing files
apps/lib/boxes/
├── option.hako                    [REPLACE: Generated by @enum]
└── result.hako                    [REPLACE: Generated by @enum]
```

**Decision on Generated Files**:
- **Day 1-2**: Generate to `apps/lib/boxes/option_generated.hako` (don't overwrite existing)
- **Day 3+**: Add config to allow overwriting or in-place generation

**User Guide Contents** (`enum-syntax.md`):
- Syntax reference
- Examples (Option, Result, HttpStatus)
- Generated methods documentation
- Best practices
- Comparison with manual Box implementation

---

### 3.7 Configuration Updates

**hako.toml** (MODIFY if needed):
```toml
[modules.overrides]
# If using generated files in standard library:
std.option = "apps/lib/boxes/option_generated.hako"
std.result = "apps/lib/boxes/result_generated.hako"

# Or if overwriting:
std.option = "apps/lib/boxes/option.hako"
std.result = "apps/lib/boxes/result.hako"
```

**Cargo.toml** (No changes needed):
- Pure code feature, no new dependencies
- Uses existing AST/parser/macro infrastructure

**Environment Variables** (for testing/debugging):
```bash
# Enable enum macro system
export NYASH_ENUM_ENABLE=1

# Trace enum expansion
export NYASH_ENUM_TRACE=1

# Dump generated AST
export NYASH_ENUM_DUMP_AST=1
```

---

## 4. Dependency Management

### 4.1 Dependency Graph

```
Tokenizer (existing)
    ↓ Token stream
enum_parser.rs (NEW)
    ↓ EnumDeclaration AST
ast.rs (MODIFY - add structs)
    ↓ AST nodes
enum_expander.rs (NEW)
    ↓ BoxDeclaration + StaticBoxDeclaration
macro/engine.rs (MODIFY - call expander)
    ↓ Expanded AST
parser/mod.rs (integration)
    ↓ Complete program AST
MIR builder (existing)
    ↓ MIR
Backend (existing)
```

### 4.2 Module Dependencies

```rust
// src/parser/declarations/enum_parser.rs
use crate::ast::{ASTNode, EnumDeclaration, EnumVariant, EnumField, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;

// src/macro/enum_expander.rs
use crate::ast::{ASTNode, EnumDeclaration, EnumVariant, Span};
use crate::ast::{BoxDeclaration, StaticBoxDeclaration}; // existing
use std::collections::HashMap;

// src/macro/engine.rs (modification)
use crate::macro::enum_expander;
```

**Circular Dependency Check**: ✅ None
- Tokenizer → Parser → AST → Expander → Engine (linear dependency chain)
- No cycles detected

---

## 5. Naming Conventions

### 5.1 File Names
| Type | Convention | Examples |
|------|------------|----------|
| Rust modules | `snake_case.rs` | `enum_parser.rs`, `enum_expander.rs` |
| Hakorune tests | `test_*.hako` | `test_enum_basic.hako`, `test_enum_option.hako` |
| Generated boxes | `*_generated.hako` or `*.hako` | `option_generated.hako` or `option.hako` |
| Documentation | `kebab-case.md` | `enum-syntax.md`, `enum-implementation-notes.md` |

### 5.2 Function Names
| Type | Convention | Examples |
|------|------------|----------|
| Parser | `parse_*` | `parse_enum_declaration()`, `parse_enum_variant()` |
| Expander | `expand_*`, `generate_*` | `expand_enum()`, `generate_box_declaration()` |
| Helpers | `verb_noun` | `validate_variant_name()`, `collect_fields()` |

### 5.3 Type Names
| Type | Convention | Examples |
|------|------------|----------|
| AST structs | `CamelCase` | `EnumDeclaration`, `EnumVariant`, `EnumField` |
| Error types | `*Error` | `EnumExpansionError`, `ParseError` |
| Config types | `*Config` | `EnumGeneratorConfig` (future) |

### 5.4 Generated Code
| Item | Naming Pattern | Examples |
|------|----------------|----------|
| Input syntax | `@enum Name { ... }` | `@enum Result`, `@enum Option` |
| Generated box | `{Name}Box<T>` | `ResultBox<T, E>`, `OptionBox<T>` |
| Static box | `{Name}` | `Result`, `Option` |
| Variant field | `{variant}_{field}` | `ok_value`, `err_error`, `some_value` |
| Query method | `is_{variant}()` | `is_ok()`, `is_err()`, `is_some()`, `is_none()` |

---

## 6. File Creation Checklist

### 6.1 Files to CREATE

**Parser**:
- [ ] `src/parser/declarations/enum_parser.rs` (NEW)
  - `parse_enum_declaration()` - main parser function
  - `parse_enum_variant()` - variant parser
  - `parse_enum_field()` - field parser
  - Unit tests in `#[cfg(test)] mod tests`

**Macro Expander**:
- [ ] `src/macro/enum_expander.rs` (NEW)
  - `expand_enum()` - main expander function
  - `generate_box_declaration()` - box AST generator
  - `generate_static_box()` - static box AST generator
  - `EnumExpansionError` enum
  - Unit tests in `#[cfg(test)] mod tests`

**Tests**:
- [ ] `apps/tests/enum/test_enum_basic.hako` (NEW) - 2-variant Result-like
- [ ] `apps/tests/enum/test_enum_option.hako` (NEW) - 2-variant Option-like
- [ ] `apps/tests/enum/test_enum_multi.hako` (NEW) - 3+ variants

**Documentation**:
- [ ] `docs/reference/language/enum-syntax.md` (NEW) - user guide
- [ ] `docs/development/roadmap/phases/phase-19-enum-match/enum-implementation-notes.md` (NEW)

### 6.2 Files to MODIFY

**AST**:
- [ ] `src/ast.rs` - Add 3 new structs:
  - `EnumDeclaration` struct
  - `EnumVariant` struct
  - `EnumField` struct
  - Add `ASTNode::EnumDeclaration` variant

**Parser Integration**:
- [ ] `src/parser/declarations/mod.rs` - Export `enum_parser` module
- [ ] `src/parser/mod.rs` - Integrate enum parsing in `parse_statement()` or equivalent

**Tokenizer** (if needed):
- [ ] `src/tokenizer/mod.rs` - Add `MacroKeyword` token (or reuse existing)

**Macro Engine**:
- [ ] `src/macro/mod.rs` - Export `enum_expander` module
- [ ] `src/macro/engine.rs` - Call `enum_expander::expand_enum()` for `EnumDeclaration` nodes

**Configuration** (optional):
- [ ] `hako.toml` - Add generated module overrides (if using generated stdlib)

### 6.3 Files to READ (Understand Before Coding)

**Required Reading**:
- [ ] `src/parser/mod.rs` - Understand current parser structure
- [ ] `src/parser/declarations/box_definition.rs` - Learn box parsing patterns
- [ ] `src/ast.rs` - Understand AST node patterns
- [ ] `src/macro/engine.rs` - Understand macro expansion flow

**Reference Reading**:
- [ ] `apps/lib/boxes/option.hako` - Current manual Option implementation
- [ ] `apps/lib/boxes/result.hako` - Current manual Result implementation
- [ ] `docs/development/roadmap/phases/phase-19-enum-match/README.md` - Project overview

---

## 7. Directory Tree (Before/After)

### 7.1 BEFORE (Current State)

```
hakorune-selfhost/
├── src/
│   ├── ast.rs                                [16,868 bytes]
│   ├── parser/
│   │   ├── mod.rs                           [15,287 bytes]
│   │   ├── declarations/
│   │   │   ├── mod.rs
│   │   │   ├── box_definition.rs
│   │   │   ├── static_box.rs
│   │   │   └── ... (6 files total)
│   │   └── ... (11 modules)
│   ├── macro/
│   │   ├── mod.rs
│   │   ├── engine.rs
│   │   ├── macro_box.rs
│   │   └── ... (6 files total)
│   └── ... (32 modules)
├── apps/
│   ├── lib/boxes/
│   │   ├── option.hako                      [1,755 bytes - manual impl]
│   │   ├── result.hako                      [1,874 bytes - manual impl]
│   │   └── ... (6 files)
│   ├── tests/
│   │   ├── (48 subdirectories)
│   │   └── ... (many test files)
│   └── ... (38 directories)
├── docs/
│   ├── reference/language/
│   │   ├── quick-reference.md
│   │   ├── LANGUAGE_REFERENCE_2025.md
│   │   └── ... (other docs)
│   └── development/roadmap/phases/phase-19-enum-match/
│       └── README.md                        [13,478 bytes]
└── ...
```

### 7.2 AFTER (With @enum Implementation)

```
hakorune-selfhost/
├── src/
│   ├── ast.rs                                [+80 lines - 3 new structs]
│   ├── parser/
│   │   ├── mod.rs                           [+10 lines - integration]
│   │   ├── declarations/
│   │   │   ├── mod.rs                       [+1 line - export]
│   │   │   ├── enum_parser.rs               [NEW ~200 lines]  ⭐
│   │   │   └── ... (existing files)
│   │   └── ... (existing modules)
│   ├── macro/
│   │   ├── mod.rs                           [+1 line - export]
│   │   ├── engine.rs                        [+15 lines - enum expansion]
│   │   ├── enum_expander.rs                 [NEW ~300 lines]  ⭐
│   │   └── ... (existing files)
│   └── ... (existing modules)
├── apps/
│   ├── lib/boxes/
│   │   ├── option.hako                      [unchanged OR regenerated]
│   │   ├── result.hako                      [unchanged OR regenerated]
│   │   ├── option_generated.hako            [OPTION: NEW ~150 lines]
│   │   ├── result_generated.hako            [OPTION: NEW ~180 lines]
│   │   └── ... (existing files)
│   ├── tests/
│   │   ├── enum/                            [NEW directory]  ⭐
│   │   │   ├── test_enum_basic.hako         [NEW ~40 lines]
│   │   │   ├── test_enum_option.hako        [NEW ~30 lines]
│   │   │   └── test_enum_multi.hako         [NEW ~50 lines]
│   │   └── ... (existing subdirectories)
│   └── ... (existing directories)
├── docs/
│   ├── reference/language/
│   │   ├── enum-syntax.md                   [NEW ~300 lines]  ⭐
│   │   └── ... (existing docs)
│   └── development/roadmap/phases/phase-19-enum-match/
│       ├── README.md                        [existing]
│       ├── enum-module-architecture.md      [THIS FILE]  ⭐
│       └── enum-implementation-notes.md     [NEW ~200 lines]  ⭐
└── ...
```

**Summary of Changes**:
- **NEW files**: 8 files (⭐ marked above)
- **Modified files**: 5 files (`ast.rs`, `parser/mod.rs`, `declarations/mod.rs`, `macro/mod.rs`, `macro/engine.rs`)
- **Total new code**: ~1,500 lines (parser 200 + expander 300 + tests 120 + docs 500 + AST 80 + integration 50 + notes 200)

---

## 8. Implementation Phases (Day-by-Day Breakdown)

### Day 1: Parser + AST (Foundation)

**Goal**: Parse `@enum` syntax into AST nodes

**Tasks**:
1. ✅ Read reference files (parser/mod.rs, box_definition.rs, ast.rs)
2. ✅ Add AST structs to `src/ast.rs` (EnumDeclaration, EnumVariant, EnumField)
3. ✅ Create `src/parser/declarations/enum_parser.rs`
4. ✅ Implement `parse_enum_declaration()`
5. ✅ Add tokenizer support for `@enum` (if needed)
6. ✅ Write parser unit tests
7. ✅ Integration: Export from `declarations/mod.rs`, integrate in `parser/mod.rs`

**Success Criteria**:
- Parser can parse: `@enum Result { Ok(value: T), Err(error: E) }`
- Parser unit tests pass
- AST nodes correctly constructed

**Estimated Lines**: ~300 lines (200 parser + 80 AST + 20 integration)

### Day 2: Macro Expander (Code Generation)

**Goal**: Transform `EnumDeclaration` AST into `BoxDeclaration` + `StaticBoxDeclaration`

**Tasks**:
1. ✅ Create `src/macro/enum_expander.rs`
2. ✅ Implement `expand_enum()` - main expander
3. ✅ Implement `generate_box_declaration()` - box with variant field
4. ✅ Implement `generate_static_box()` - constructor methods
5. ✅ Write expander unit tests (check generated AST structure)
6. ✅ Integration: Call from `macro/engine.rs`

**Success Criteria**:
- Expander generates correct Box AST (variant field + all variant fields)
- Expander generates correct StaticBox AST (constructor methods)
- Expander unit tests pass
- Generated AST can be converted to MIR (existing pipeline)

**Estimated Lines**: ~350 lines (300 expander + 50 integration/tests)

### Day 3: Integration Tests + Documentation

**Goal**: Full pipeline working, documented, tested

**Tasks**:
1. ✅ Create `apps/tests/enum/` directory
2. ✅ Write `test_enum_basic.hako` (Result-like)
3. ✅ Write `test_enum_option.hako` (Option-like)
4. ✅ Write `test_enum_multi.hako` (3+ variants)
5. ✅ Run integration tests (parse → expand → execute)
6. ✅ Write `docs/reference/language/enum-syntax.md` (user guide)
7. ✅ Write `enum-implementation-notes.md` (developer notes)

**Success Criteria**:
- All integration tests pass
- Documentation complete and accurate
- Generated code executes correctly

**Estimated Lines**: ~620 lines (120 tests + 500 docs)

### Day 4: Polish + Edge Cases (Buffer)

**Goal**: Handle edge cases, improve error messages, optimize

**Tasks**:
- Error handling: duplicate variants, invalid names, circular dependencies
- Error messages: clear, actionable, with suggestions
- Edge cases: zero-field variants, single variant, generic types
- Performance: AST generation optimization
- Code review: refactor, simplify, document

**Success Criteria**:
- All error cases handled gracefully
- Error messages are helpful
- Code is clean and maintainable

---

## 9. Testing Strategy

### 9.1 Unit Tests (Rust)

**Parser Unit Tests** (`src/parser/declarations/enum_parser.rs::tests`):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_two_variant_enum() {
        // Parse: @enum Result { Ok(value: T), Err(error: E) }
        // Assert: Correct AST structure
    }

    #[test]
    fn test_parse_zero_field_variant() {
        // Parse: @enum Option { Some(value: T), None() }
        // Assert: None has empty fields vec
    }

    #[test]
    fn test_duplicate_variant_error() {
        // Parse: @enum Foo { A(), A() }
        // Assert: ParseError with duplicate message
    }
}
```

**Expander Unit Tests** (`src/macro/enum_expander.rs::tests`):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_result_enum() {
        // Input: EnumDeclaration for Result
        // Assert: Generated BoxDeclaration has variant field
        // Assert: Generated StaticBoxDeclaration has Ok/Err methods
    }

    #[test]
    fn test_generated_box_fields() {
        // Assert: ok_value and err_error fields exist
        // Assert: Fields are nullable for variants not using them
    }
}
```

### 9.2 Integration Tests (Hakorune)

**Basic Functionality**:
```hakorune
// apps/tests/enum/test_enum_basic.hako

@enum Result {
    Ok(value: IntegerBox)
    Err(error: StringBox)
}

static box Main {
    main() {
        local r1
        r1 = Result.Ok(42)

        if r1.is_ok() {
            print("PASS: is_ok works")
        } else {
            print("FAIL: is_ok broken")
            return false
        }

        local val
        val = r1.unwrap()
        if val == 42 {
            print("PASS: unwrap works")
        } else {
            print("FAIL: unwrap broken")
            return false
        }

        return true
    }
}
```

**Test Execution**:
```bash
# Run all enum tests
for test in apps/tests/enum/*.hako; do
    echo "Running $test..."
    ./target/release/hako "$test" || echo "FAILED: $test"
done
```

### 9.3 Smoke Tests

**Add to Existing Smoke Test Suite**:
```bash
# tools/smokes/v2/profiles/quick/core/enum_basic_vm.sh

#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../../lib/test_runner.sh"

TEST_FILE="apps/tests/enum/test_enum_basic.hako"
EXPECTED_OUTPUT="PASS: is_ok works
PASS: unwrap works"

run_test_vm "$TEST_FILE" "$EXPECTED_OUTPUT"
```

---

## 10. Error Handling Strategy

### 10.1 Parser Errors

**Error Types**:
```rust
pub enum ParseError {
    // Existing errors...

    // Enum-specific errors
    DuplicateEnumVariant {
        enum_name: String,
        variant_name: String,
        line: usize,
    },

    DuplicateEnumField {
        variant_name: String,
        field_name: String,
        line: usize,
    },

    InvalidEnumName {
        name: String,
        reason: String,
        line: usize,
    },

    InvalidVariantName {
        name: String,
        reason: String,
        line: usize,
    },
}
```

**Error Messages** (User-Friendly):
```
Error: Duplicate variant 'Ok' in enum 'Result' at line 3
  |
3 |     Ok(value: T)
  |     ^^ variant already declared at line 2
  |
Help: Each variant in an enum must have a unique name
```

### 10.2 Expander Errors

**Error Types**:
```rust
pub enum EnumExpansionError {
    DuplicateVariant(String),
    DuplicateField(String),
    InvalidTypeName(String),
    CircularDependency(String),
    ReservedMethodName { variant: String, method: String },
}
```

**Example**:
```
Error: Reserved method name 'is_ok' conflicts with generated query method
  |
  | @enum Result { Ok(is_ok: IntegerBox) }
  |                   ^^^^^ field name conflicts with auto-generated method
  |
Help: Rename field to avoid 'is_*' prefix (e.g., 'ok_status', 'ok_flag')
```

---

## 11. Performance Considerations

### 11.1 Parser Performance
- **Minimal lookahead**: Use `peek()` instead of backtracking
- **Single-pass parsing**: No multiple traversals of token stream
- **Reuse existing patterns**: Follow `box_definition.rs` structure

### 11.2 Expander Performance
- **Lazy AST construction**: Build nodes only when needed
- **Avoid cloning**: Use references where possible, clone only final AST
- **Inline small functions**: Mark helpers as `#[inline]`

### 11.3 Generated Code Performance
- **Minimal runtime overhead**: Generated boxes behave like hand-written boxes
- **No macro runtime**: All expansion at compile-time (parse-time)
- **Efficient method calls**: `is_ok()` checks string comparison (variant == "Ok")

**Future Optimizations** (Post-Day 1):
- Enum variant as integer tag (faster than string comparison)
- Packed field storage (avoid nullable fields for unused variants)
- Static dispatch for known variant types

---

## 12. Future Expansion Points

### 12.1 Match Expression Integration (Phase 19.2)

**Current**: `if result.is_ok() { ... }`
**Future**: `match result { Ok(v) => ..., Err(e) => ... }`

**Architecture Hook**:
```rust
// src/parser/expressions.rs (future)
fn parse_match_expression(&mut self) -> Result<ASTNode, ParseError> {
    // match <scrutinee> { <pattern> => <expr>, ... }
}

// src/macro/enum_expander.rs (future)
fn expand_match_expression(expr: MatchExpression, enum_decl: &EnumDeclaration)
    -> Result<ASTNode, EnumExpansionError> {
    // Desugar to if-else tree with is_* checks
}
```

### 12.2 Hakorune Self-Hosting (Phase 19.3)

**Current**: Rust implementation (`src/macro/enum_expander.rs`)
**Future**: Hakorune implementation (`apps/macros/enum_macro.hako`)

**Migration Path**:
1. Keep Rust implementation as reference
2. Implement in Hakorune (string-based codegen)
3. Test both implementations in parallel
4. Switch default to Hakorune version
5. Archive Rust version for fallback

### 12.3 Advanced Features (Phase 19.4+)

- **Enum with methods**: `@enum Result { ... } impl { map() { ... } }`
- **Enum with associated values**: `@enum Color { Red = 1, Green = 2 }`
- **Enum exhaustiveness checking**: Compile-time warning for missing match arms
- **Enum pattern matching destructuring**: `match r { Ok(value) if value > 0 => ... }`

---

## 13. Summary Table

| Aspect | Decision | File Location |
|--------|----------|---------------|
| **Parser** | New module in existing directory | `src/parser/declarations/enum_parser.rs` |
| **AST** | Add to existing monolithic file | `src/ast.rs` (3 new structs) |
| **Expander** | New module in macro system | `src/macro/enum_expander.rs` |
| **Tests** | Dedicated subdirectory | `apps/tests/enum/` |
| **Docs** | User guide + developer notes | `docs/reference/language/enum-syntax.md` + `docs/development/.../enum-implementation-notes.md` |
| **Implementation** | Rust (Day 1-2) → Hakorune (Future) | Rust for MVP, migrate later |
| **Generated Code** | Option/Result standard library | `apps/lib/boxes/option.hako`, `result.hako` |
| **Integration** | Minimal modifications | 5 files modified |
| **Timeline** | 3 days core + 1 day buffer | Day 1: Parser, Day 2: Expander, Day 3: Tests+Docs |

---

## 14. Rationale Summary

**Why this architecture?**

1. **Follows Existing Patterns**:
   - Parser in `declarations/` (like `box_definition.rs`)
   - AST in monolithic `ast.rs` (current practice)
   - Macro in `macro/` directory (existing system)

2. **Separation of Concerns**:
   - Parser: Token → AST (single responsibility)
   - Expander: AST → AST (single responsibility)
   - Engine: Orchestration (existing)

3. **Testability**:
   - Parser unit tests (AST correctness)
   - Expander unit tests (generation correctness)
   - Integration tests (full pipeline)

4. **Maintainability**:
   - Clear file structure (predictable locations)
   - Minimal dependencies (linear flow)
   - Comprehensive documentation

5. **Future-Proof**:
   - Easy to extend (match expressions, advanced features)
   - Easy to migrate (Rust → Hakorune)
   - Easy to optimize (clear boundaries)

---

## 15. Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| **Parser complexity** | Follow existing `box_definition.rs` patterns |
| **AST size growth** | Monitor file size, split if exceeds 25KB |
| **Expander bugs** | Comprehensive unit tests for each generated AST node |
| **Performance regression** | Benchmark generated code vs hand-written |
| **Breaking changes** | Keep manual implementations working alongside generated |
| **Documentation drift** | Update docs in same PR as implementation |

---

## 16. Conclusion

This architecture provides a **clean, maintainable, production-ready** foundation for @enum implementation. It follows Hakorune conventions, integrates naturally with existing code, and provides clear extension points for future features.

**Next Steps**:
1. ✅ Review this document with team
2. ✅ Create implementation checklist from "File Creation Checklist"
3. ✅ Begin Day 1 implementation (Parser + AST)
4. ✅ Iterate based on feedback

**Questions or Concerns?**
- Open issue in Phase 19 tracker
- Discuss in development channel
- Update this document as architecture evolves

---

**Document Version**: 1.0
**Last Updated**: 2025-10-08
**Author**: Claude Code (Architecture Analysis Agent)
**Status**: Ready for Implementation ✅
