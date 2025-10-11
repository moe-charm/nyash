# @enum Macro Implementation - Quick Start Guide

**✅ Status**: **Phase 19 完了（2025-10-08）** - @enum macro 完全実装済み

**Read this first**, then dive into the full spec: [@enum-macro-implementation-spec.md](./@enum-macro-implementation-spec.md)

---

## 🎉 実装状況（2025-10-08完了）

- ✅ **@enum macro**: 完全実装（enum_parser.rs, macro/engine.rs）
- ✅ **match式**: 完全実装（match_expr.rs、literal/type patterns + guards対応）
- ✅ **ResultBox Phase 1**: 実装完了（apps/lib/boxes/result.hako, 103行）
- ⚠️ **API競合**: 小文字版(Result.ok/err) vs @enum版(Result.Ok/Err) 共存中

**実装場所**:
- Parser: `src/parser/declarations/enum_parser.rs` (147行)
- Macro: `src/macro/engine.rs` (expand_enum_to_boxes関数)
- Match: `src/parser/expr/match_expr.rs` (392行)

---

## 30-Second Overview

**What**: Implement `@enum` macro that desugars to Box + Static Box
**How**: Macro-only (no IR changes, no VariantBox core)
**When**: ~~3-5 days~~ **✅ 完了（2025-10-08）**

---

## Core Design Pattern

### Input
```hakorune
@enum Result {
  Ok(value)
  Err(error)
}
```

### Output (Desugared)
```hakorune
box ResultBox {
  _tag: StringBox      // Discriminator
  _value: Box          // Ok's field
  _error: Box          // Err's field

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

---

## Key Rules

1. **Tag Field**: Always `_tag: StringBox` (discriminator)
2. **Field Naming**:
   - Single field: `_value`
   - Named fields: `_{field_name}` (e.g., `error` → `_error`)
   - Multi-field: `_field1`, `_field2`, etc.
3. **All fields are Box-typed**: `Box` (runtime polymorphism)
4. **Constructors**: Variant name becomes static method
5. **Helpers**: `is_{Variant}(v)` and `as_{Variant}(v)` auto-generated
6. **Fail-Fast**: `as_*` panics if wrong variant

---

## Implementation Roadmap (5 Days)

### Day 1: Parser (6-8h)
- Add `@enum` token recognition
- Parse `@enum Name { Variant1(field) Variant2 }`
- Create AST nodes: `EnumDeclaration`, `EnumVariant`, `EnumField`
- Validation: duplicates, empty enum, reserved names

### Day 2: Macro Expansion (8-10h)
- Create `src/macro/enum_macro.rs`
- Implement `expand_enum()` → generates AST
- Generate data box (`{Name}Box` with `_tag` + all fields)
- Generate `birth()` method

### Day 3: Helpers (8-10h)
- Generate static box with constructors
- Generate `is_{Variant}()` methods
- Generate `as_{Variant}()` methods (single/multi-field)
- Handle edge cases (0-field variants, conflicts)

### Day 4: Tests (6-8h)
- 12 positive tests (basic, multi-variant, helpers, integration)
- 3 negative tests (errors)
- Test runner script
- Verify all tests pass

### Day 5: Integration (6-8h)
- Smoke tests
- Performance benchmark (vs old Option/Result)
- Migration guide
- Documentation update

---

## Files to Create/Modify

### New Files
- `src/parser/declarations/enum_parser.rs` - Parser logic
- `src/macro/enum_macro.rs` - Macro expansion
- `apps/lib/boxes/tests/enum_*.hako` - Test suite (8 files)
- `tools/run_enum_tests.sh` - Test runner
- `docs/guides/enum-migration-guide.md` - Migration guide

### Modified Files
- `src/parser/mod.rs` - Add enum parser invocation
- `src/parser/declarations/mod.rs` - Export enum parser
- `src/ast.rs` - Add `EnumDeclaration`, `EnumVariant`, `EnumField`
- `src/macro/mod.rs` - Integrate enum expansion
- `hako.toml` - Add v2 module aliases (optional)

---

## Critical Edge Cases

1. **Zero-field variant**: `None` → constructor takes no args
2. **Multi-field variant**: `as_*` returns `ArrayBox` with all fields
3. **Field conflicts**: Same field name across variants → shared storage (OK)
4. **Reserved names**: `birth`, `fini`, `new`, `me`, `this` → error
5. **Empty enum**: No variants → parse error

---

## Test Examples (Must Pass)

### Basic Usage
```hakorune
@enum Option { Some(value) None }

local opt = Option.Some(42)
print(opt._tag)        // "Some"
print(opt._value)      // "42"

if Option.is_Some(opt) {
  local v = Option.as_Some(opt)
  print(v)             // "42"
}
```

### Panic Behavior
```hakorune
local err = Result.Err("failed")
local v = Result.as_Ok(err)  // Panics!
// Output: [PANIC] Result.as_Ok: called on Err
// Returns: null
```

### Pattern Matching (Manual)
```hakorune
if result._tag == "Ok" {
  print("Success: " + result._value)
}
if result._tag == "Err" {
  print("Error: " + result._error)
}
```

---

## Migration Strategy (for existing Option/Result)

### Phase 1: Parallel Existence
- Keep old `option.hako` / `result.hako`
- Create new `option_v2.hako` / `result_v2.hako` with `@enum`
- Add module aliases in `hako.toml`

### Phase 2: Gradual Migration
- Migrate test files first
- Migrate non-critical tools
- Verify behavior parity

### Phase 3: Deprecation
- Rename old → `*_deprecated.hako`
- Rename v2 → main files
- Update all import sites

### Phase 4: Cleanup
- Delete deprecated files
- Update documentation

**Important**: NOT backward compatible. API changes:
- `Option.some(v)` → `Option.Some(v)` (capital S)
- `opt.is_some()` → `Option.is_Some(opt)` (static method)
- `opt._is_some` → `opt._tag == "Some"` (tag-based)

---

## Performance Expectations

- **String comparison**: Slightly slower than integer flags
- **Expected regression**: <10% (acceptable for MVP)
- **Future optimization**: String interning → pointer comparison

---

## Success Criteria

- [ ] All 12 positive tests pass
- [ ] All 3 negative tests catch errors
- [ ] Smoke tests pass
- [ ] Performance <10% regression
- [ ] Documentation complete
- [ ] Migration guide written

---

## Risk Mitigation

1. **Parser conflicts**: Test with `@local` in same file
2. **Macro order**: Run @enum expansion early (before other macros)
3. **Field naming**: Reserve `_` prefix, validate in parser
4. **Error messages**: Preserve source spans, show original @enum location
5. **Debugging**: Add `NYASH_MACRO_TRACE=1` output

---

## Next Steps

1. Read full spec: [@enum-macro-implementation-spec.md](./@enum-macro-implementation-spec.md)
2. Start with Day 1 (Parser changes)
3. Follow daily checklist
4. Ask questions early (don't guess)

---

**Full Specification**: [@enum-macro-implementation-spec.md](./@enum-macro-implementation-spec.md) (13,000+ words, implementation-ready)
