# @enum Architecture Summary (Quick Reference)

**For**: Developers who want the TL;DR version
**See Also**: [enum-module-architecture.md](enum-module-architecture.md) (complete design)

---

## File Map (What Goes Where)

```
┌─────────────────────────────────────────────────────────────────┐
│                       @enum Implementation                      │
│                        File Organization                        │
└─────────────────────────────────────────────────────────────────┘

📁 src/
├── 📄 ast.rs                                    [MODIFY +80 lines]
│   └── Add: EnumDeclaration, EnumVariant, EnumField structs
│
├── 📁 parser/
│   ├── 📄 mod.rs                                [MODIFY +10 lines]
│   │   └── Integrate enum parsing in parse_statement()
│   └── 📁 declarations/
│       ├── 📄 mod.rs                            [MODIFY +1 line]
│       │   └── Export enum_parser module
│       └── 📄 enum_parser.rs                    [NEW ~200 lines] ⭐
│           └── parse_enum_declaration()
│
└── 📁 macro/
    ├── 📄 mod.rs                                [MODIFY +1 line]
    │   └── Export enum_expander module
    ├── 📄 engine.rs                             [MODIFY +15 lines]
    │   └── Call enum_expander::expand_enum()
    └── 📄 enum_expander.rs                      [NEW ~300 lines] ⭐
        ├── expand_enum()
        ├── generate_box_declaration()
        └── generate_static_box()

📁 apps/
└── 📁 tests/
    └── 📁 enum/                                 [NEW directory] ⭐
        ├── 📄 test_enum_basic.hako             [NEW ~40 lines]
        ├── 📄 test_enum_option.hako            [NEW ~30 lines]
        └── 📄 test_enum_multi.hako             [NEW ~50 lines]

📁 docs/
├── 📁 reference/language/
│   └── 📄 enum-syntax.md                        [NEW ~300 lines] ⭐
│       └── User guide for @enum
└── 📁 development/roadmap/phases/phase-19-enum-match/
    ├── 📄 enum-module-architecture.md           [THIS DOC] ⭐
    ├── 📄 IMPLEMENTATION_CHECKLIST.md           [CHECKLIST] ⭐
    └── 📄 enum-implementation-notes.md          [NEW ~200 lines] ⭐
        └── Developer implementation notes
```

---

## Data Flow (Pipeline)

```
┌──────────────────────────────────────────────────────────────────────┐
│                     @enum Processing Pipeline                        │
└──────────────────────────────────────────────────────────────────────┘

Source Code:
  @enum Result { Ok(value: T), Err(error: E) }
         │
         ▼
  ┌─────────────────┐
  │   Tokenizer     │  (existing)
  └─────────────────┘
         │ Token Stream: [@, enum, Result, {, Ok, (, value, :, T, ), ...]
         ▼
  ┌─────────────────┐
  │  Parser         │  src/parser/declarations/enum_parser.rs (NEW)
  │  enum_parser.rs │  ← parse_enum_declaration()
  └─────────────────┘
         │ EnumDeclaration AST
         ▼
  ┌─────────────────┐
  │  AST Structs    │  src/ast.rs (MODIFY)
  │  ast.rs         │  ← EnumDeclaration, EnumVariant, EnumField
  └─────────────────┘
         │ EnumDeclaration AST
         ▼
  ┌─────────────────┐
  │  Macro Engine   │  src/macro/engine.rs (MODIFY)
  │  engine.rs      │  ← Call enum_expander::expand_enum()
  └─────────────────┘
         │
         ▼
  ┌─────────────────┐
  │  Enum Expander  │  src/macro/enum_expander.rs (NEW)
  │ enum_expander.rs│  ← expand_enum()
  └─────────────────┘
         │ Vec<ASTNode> [BoxDeclaration, StaticBoxDeclaration]
         ▼
  ┌─────────────────────────────────────────────────────────────────┐
  │  Generated AST:                                                 │
  │                                                                 │
  │  box ResultBox<T, E> {                                          │
  │    variant: StringBox                                           │
  │    ok_value: T                                                  │
  │    err_error: E                                                 │
  │    birth(variant, ok_value, err_error) { ... }                  │
  │    is_ok() { return me.variant == "Ok" }                        │
  │    is_err() { return me.variant == "Err" }                      │
  │  }                                                              │
  │                                                                 │
  │  static box Result {                                            │
  │    Ok(value) { return new ResultBox("Ok", value, null) }        │
  │    Err(error) { return new ResultBox("Err", null, error) }      │
  │  }                                                              │
  └─────────────────────────────────────────────────────────────────┘
         │ BoxDeclaration + StaticBoxDeclaration AST
         ▼
  ┌─────────────────┐
  │  MIR Builder    │  (existing)
  └─────────────────┘
         │ MIR
         ▼
  ┌─────────────────┐
  │  Backend        │  (existing)
  │  VM/LLVM/WASM   │
  └─────────────────┘
         │
         ▼
     Executable
```

---

## Module Dependencies (No Cycles)

```
┌─────────────────────────────────────────────────────────────┐
│                  Dependency Graph                           │
│                  (Linear Flow)                              │
└─────────────────────────────────────────────────────────────┘

tokenizer.rs
    ↓
enum_parser.rs ──depends on──> ast.rs (EnumDeclaration structs)
    ↓
ast.rs
    ↓
enum_expander.rs ──depends on──> ast.rs (BoxDeclaration structs)
    ↓
engine.rs ──depends on──> enum_expander.rs
    ↓
parser/mod.rs (integration)
    ↓
MIR builder (existing)

✅ No circular dependencies
✅ Clear data flow
✅ Easy to test each layer independently
```

---

## Naming Conventions (Quick Reference)

| Item | Pattern | Example |
|------|---------|---------|
| **Files** | `snake_case.rs` | `enum_parser.rs`, `enum_expander.rs` |
| **AST structs** | `CamelCase` | `EnumDeclaration`, `EnumVariant`, `EnumField` |
| **Functions** | `verb_noun()` | `parse_enum_declaration()`, `expand_enum()` |
| **Tests** | `test_*.hako` | `test_enum_basic.hako` |
| **Input syntax** | `@enum Name` | `@enum Result`, `@enum Option` |
| **Generated box** | `{Name}Box<T>` | `ResultBox<T, E>`, `OptionBox<T>` |
| **Generated static** | `{Name}` | `Result`, `Option` |
| **Variant fields** | `{variant}_{field}` | `ok_value`, `err_error` |
| **Query methods** | `is_{variant}()` | `is_ok()`, `is_err()` |

---

## Test Organization (3 Layers)

```
┌─────────────────────────────────────────────────────────────────┐
│                     Test Strategy                               │
└─────────────────────────────────────────────────────────────────┘

Layer 1: Parser Unit Tests (Rust)
  📁 src/parser/declarations/enum_parser.rs
  └── #[cfg(test)] mod tests {
      ✓ test_parse_two_variant_enum()
      ✓ test_parse_zero_field_variant()
      ✓ test_duplicate_variant_error()
  }

  Run: cargo test --lib enum_parser::tests

Layer 2: Expander Unit Tests (Rust)
  📁 src/macro/enum_expander.rs
  └── #[cfg(test)] mod tests {
      ✓ test_expand_result_enum()
      ✓ test_generated_box_fields()
      ✓ test_generated_is_methods()
  }

  Run: cargo test --lib enum_expander::tests

Layer 3: Integration Tests (Hakorune)
  📁 apps/tests/enum/
  ├── test_enum_basic.hako       ← Result-like (Ok/Err)
  ├── test_enum_option.hako      ← Option-like (Some/None)
  └── test_enum_multi.hako       ← 3+ variants

  Run: ./target/release/hako apps/tests/enum/*.hako
```

---

## Timeline (3 Days + 1 Buffer)

```
┌─────────────────────────────────────────────────────────────────┐
│                     Implementation Timeline                     │
└─────────────────────────────────────────────────────────────────┘

Day 1: Parser + AST (Foundation)                    ⏰ 8 hours
  ✓ Add AST structs (EnumDeclaration, EnumVariant, EnumField)
  ✓ Create enum_parser.rs
  ✓ Implement parse_enum_declaration()
  ✓ Add tokenizer support for @enum
  ✓ Write parser unit tests
  ✓ Integration: Export from mod.rs

  Output: Can parse @enum syntax into AST ✅

Day 2: Macro Expander (Code Generation)             ⏰ 8 hours
  ✓ Create enum_expander.rs
  ✓ Implement expand_enum()
  ✓ Implement generate_box_declaration()
  ✓ Implement generate_static_box()
  ✓ Write expander unit tests
  ✓ Integration: Call from engine.rs

  Output: Can transform enum AST → box AST ✅

Day 3: Integration Tests + Documentation            ⏰ 8 hours
  ✓ Create apps/tests/enum/ directory
  ✓ Write test_enum_basic.hako (Result-like)
  ✓ Write test_enum_option.hako (Option-like)
  ✓ Write test_enum_multi.hako (3+ variants)
  ✓ Write enum-syntax.md (user guide)
  ✓ Write enum-implementation-notes.md (dev notes)

  Output: Full pipeline working + documented ✅

Day 4: Polish + Edge Cases (Buffer)                 ⏰ 4-8 hours
  ✓ Error handling (duplicate variants, invalid names)
  ✓ Edge cases (zero-field variants, single variant)
  ✓ Code review and refactoring
  ✓ Clippy and fmt fixes

  Output: Production-ready code ✅
```

---

## Success Criteria (Checklist)

- [ ] ✅ **Parser**: Can parse `@enum Result { Ok(value: T), Err(error: E) }`
- [ ] ✅ **AST**: EnumDeclaration nodes correctly constructed
- [ ] ✅ **Expander**: Generates BoxDeclaration + StaticBoxDeclaration
- [ ] ✅ **Generated Code**: Valid Hakorune syntax
- [ ] ✅ **Tests**: All integration tests pass
- [ ] ✅ **Docs**: User guide and developer notes complete
- [ ] ✅ **Errors**: Helpful error messages for common mistakes
- [ ] ✅ **Edge Cases**: Zero-field variants, single variant, many variants
- [ ] ✅ **No Regressions**: Existing tests still pass

---

## Quick Commands (Copy-Paste Ready)

```bash
# Day 1: Build and test parser
cargo build
cargo test --lib enum_parser::tests

# Day 2: Build and test expander
cargo build
cargo test --lib enum_expander::tests

# Day 3: Run integration tests
./target/release/hako apps/tests/enum/test_enum_basic.hako
./target/release/hako apps/tests/enum/test_enum_option.hako
./target/release/hako apps/tests/enum/test_enum_multi.hako

# Day 4: Code quality checks
cargo clippy
cargo fmt
cargo test

# All enum tests at once
for test in apps/tests/enum/*.hako; do
    echo "Running $test..."
    ./target/release/hako "$test" || echo "FAILED: $test"
done
```

---

## Key Design Principles

1. **Separation of Concerns**: Parser → AST → Expander → Engine (clear boundaries)
2. **Testability**: 3-layer testing (parser unit, expander unit, integration)
3. **Follows Existing Patterns**: Consistent with box_definition.rs style
4. **Minimal Changes**: Only 5 files modified + 8 files created
5. **Future-Proof**: Easy to extend for match expressions (Phase 19.2)

---

## FAQ (Quick Answers)

**Q: Where do I start?**
A: Read [IMPLEMENTATION_CHECKLIST.md](IMPLEMENTATION_CHECKLIST.md) and start Day 1

**Q: What if I get stuck?**
A: Check [enum-module-architecture.md](enum-module-architecture.md) for detailed design

**Q: How do I run tests?**
A: See "Quick Commands" section above

**Q: What about match expressions?**
A: Phase 19.2 (future work) - hooks are in place for easy integration

**Q: Will this break existing code?**
A: No - all changes are additive, no existing functionality modified

**Q: How do I generate documentation?**
A: `cargo doc --open` for Rust docs, markdown files for user docs

---

## Next Steps

1. ✅ Review this summary
2. ✅ Read [IMPLEMENTATION_CHECKLIST.md](IMPLEMENTATION_CHECKLIST.md)
3. ✅ Start Day 1: Parser + AST
4. ✅ Follow checklist day by day
5. ✅ Celebrate when done! 🎉

---

**Document Version**: 1.0
**Last Updated**: 2025-10-08
**Status**: Ready for Implementation ✅
