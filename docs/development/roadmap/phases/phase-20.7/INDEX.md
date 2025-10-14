# Phase 20.7: Collections in Hakorune - Document Index

**Phase Duration**: 8 weeks (2026-05-25 → 2026-07-19)
**Status**: PLANNED

---

## 📚 Document Structure

### Core Documents

1. **[README.md](README.md)** (日本語)
   - Phase 20.7 概要
   - 実装範囲 (MapBox/ArrayBox)
   - キー比較順序 (Symbol < Int < String)
   - ValueBox/DataBox境界
   - 成功基準

2. **[PLAN.md](PLAN.md)** (English)
   - Executive summary
   - Week 1-4: MapBox implementation plan
   - Week 5-8: ArrayBox implementation plan
   - Key design decisions
   - Success criteria & risk mitigation

3. **[CHECKLIST.md](CHECKLIST.md)**
   - Week 1-4: MapBox implementation checkboxes
   - Week 5-8: ArrayBox implementation checkboxes
   - Deterministic behavior verification
   - Golden testing validation

4. **[INDEX.md](INDEX.md)** (this file)
   - Document structure overview
   - Quick reference guide
   - Related phases & resources

---

## 🎯 Quick Reference

### MapBox API (Week 1-4)

```hakorune
box MapBox {
    set(key: DataBox, value: DataBox) -> NullBox
    get(key: DataBox) -> DataBox | NullBox
    has(key: DataBox) -> BoolBox
    remove(key: DataBox) -> DataBox | NullBox
    size() -> IntegerBox
    keys() -> ArrayBox      // Deterministic order
    values() -> ArrayBox    // Deterministic order
}
```

### ArrayBox API (Week 5-8)

```hakorune
box ArrayBox {
    push(value: DataBox) -> NullBox
    pop() -> DataBox | NullBox
    get(index: IntegerBox) -> DataBox | NullBox
    set(index: IntegerBox, value: DataBox) -> NullBox
    size() -> IntegerBox
    slice(start: IntegerBox, end: IntegerBox) -> ArrayBox
    concat(other: ArrayBox) -> ArrayBox
}
```

### Key Comparison Order

```
Symbol < Int < String
```

**Example**:
```hakorune
local map = new MapBox()
map.set(:foo, 1)      // Symbol
map.set(42, 2)        // Int
map.set("bar", 3)     // String

local keys = map.keys()
// Output: [:foo, 42, "bar"]  (deterministic order)
```

---

## 🔗 Related Phases

### Prerequisite

- **[Phase 20.6](../phase-20.6/)**: VM Core Complete + Dispatch Unification
  - Required: All 16 MIR instructions working
  - Required: Resolver-based dispatch
  - Required: Golden test framework

### Provides to

- **[Phase 20.8](../phase-20.8/)**: GC v0 + Rust Deprecation
  - Delivers: Collections for GC root scanning
  - Delivers: Reference tracking infrastructure
  - Delivers: Deterministic iteration for mark phase

### Related Phases

- **[Phase 20.5](../phase-20.5/)**: Foundation (HostBridge + op_eq + VM PoC)
  - Establishes: ValueBox/DataBox boundaries
  - Establishes: Fail-Fast philosophy
  - Establishes: Golden testing strategy

---

## 📖 External Resources

### Hakorune Core Documentation

1. **[CLAUDE.md](../../../../CLAUDE.md#🧱-先頭原則-箱理論box-first)**
   - Box-First principle
   - Everything is Box philosophy
   - Fail-Fast error handling

2. **[Pure Hakorune Roadmap](../phase-20.5/PURE_HAKORUNE_ROADMAP.md)**
   - Overall strategy (Phase A → F)
   - Golden testing strategy
   - Implementation principles

3. **[Language Reference](../../../../reference/language/LANGUAGE_REFERENCE_2025.md)**
   - Box syntax
   - Method definitions
   - Type system

### Phase 20.7 Specific

1. **[Box System Reference](../../../../reference/boxes-system/)**
   - DataBox vs ValueBox
   - Box lifecycle
   - Memory management

2. **[Golden Test Framework](../phase-20.5/PURE_HAKORUNE_ROADMAP.md#🧪-golden-testing-strategy)**
   - Rust-VM vs Hako-VM comparison
   - Test case structure
   - CI integration

---

## 🧪 Testing Strategy

### Golden Tests

**Goal**: Prove Hako-Collections produces identical output to Rust-Collections

**Test Suite**:
```
tests/golden/collections/
├── mapbox_basic.hako           # Basic MapBox operations
├── mapbox_deterministic.hako   # Key order verification
├── mapbox_stress.hako          # 10,000 elements
├── arraybox_basic.hako         # Basic ArrayBox operations
├── arraybox_bounds.hako        # Bounds checking
└── arraybox_stress.hako        # 10,000 elements
```

**Verification**:
```bash
# Run same test on both implementations
./hako --backend vm-rust test.hako > rust_output.txt
./hako --backend vm test.hako > hako_output.txt

# Compare outputs
diff rust_output.txt hako_output.txt
# Expected: No differences
```

### Performance Benchmarks

**Target**: Hako-Collections ≥ 70% of Rust-Collections speed

**Metrics**:
- MapBox: set/get/remove operations per second
- ArrayBox: push/pop/get/set operations per second
- Large-scale: 10,000 element operations

---

## 🎯 Success Criteria Summary

### Week 4 (MapBox Complete)

- [ ] All MapBox methods implemented in Pure Hakorune
- [ ] Golden tests: 50+ test cases PASS
- [ ] Deterministic key order verified (Symbol < Int < String)
- [ ] Performance ≥ 70% of Rust-MapBox

### Week 8 (ArrayBox Complete)

- [ ] All ArrayBox methods implemented in Pure Hakorune
- [ ] Golden tests: 50+ test cases PASS
- [ ] Bounds checking Fail-Fast verified
- [ ] Performance ≥ 70% of Rust-ArrayBox

### Phase 20.7 Complete

- [ ] MapBox/ArrayBox fully in Hakorune
- [ ] 100+ golden tests PASS (combined)
- [ ] Deterministic behavior verified
- [ ] ValueBox/DataBox boundaries enforced
- [ ] Ready for Phase 20.8 (GC v0)

---

## 📝 Document Maintenance

### Update Frequency

- **Weekly**: Update CHECKLIST.md with completed tasks
- **Milestone**: Update README.md with week summaries
- **Phase End**: Update PLAN.md with lessons learned

### Review Process

1. Week 4 Review: MapBox implementation complete
2. Week 8 Review: ArrayBox implementation complete
3. Phase End Review: Overall Phase 20.7 retrospective

---

**Status**: PLANNED
**Last Updated**: 2025-10-14
**Next Review**: Week 4 (MapBox complete) - 2026-06-22
