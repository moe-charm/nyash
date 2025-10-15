# Phase 20.7: Collections in Hakorune - Implementation Plan

**Duration**: 8 weeks (2026-05-25 → 2026-07-19)
**Prerequisite**: Phase 20.6 (VM Core Complete + Dispatch Unification)
**Completes**: Phase D (Collections - MapBox/ArrayBox in Pure Hakorune)

---

## Executive Summary

Phase 20.7 implements Hakorune's core collection libraries (MapBox and ArrayBox) in **Pure Hakorune**, completing the "Everything is Box" vision. This phase eliminates Rust dependencies for collection operations, enabling full self-hosting of data structures.

**Key Achievements**:
- MapBox (hash map) with deterministic key ordering
- ArrayBox (dynamic array) with Fail-Fast boundary checks
- Golden tests proving Rust-Collections vs Hako-Collections parity
- Performance ≥ 70% of Rust-Collections

**Critical Principles**:
- **Deterministic Behavior**: Key order is predictable (Symbol < Int < String)
- **ValueBox/DataBox Boundaries**: ValueBox is temporary (pipeline only), DataBox is persistent
- **Fail-Fast**: Unknown methods → RuntimeError (no silent fallbacks)

---

## Week 1-4: MapBox Implementation

### Overview

MapBox is a hash map storing key-value pairs with deterministic iteration order.

### Core Methods

```hakorune
box MapBox {
    // Storage
    _buckets: ArrayBox      // Internal bucket storage
    _size: IntegerBox       // Element count
    _load_factor: FloatBox  // For resizing

    // Public API
    set(key: DataBox, value: DataBox) -> NullBox
    get(key: DataBox) -> DataBox | NullBox
    has(key: DataBox) -> BoolBox
    remove(key: DataBox) -> DataBox | NullBox
    size() -> IntegerBox
    keys() -> ArrayBox      // Returns keys in deterministic order
    values() -> ArrayBox    // Returns values in deterministic order
}
```

### Week 1: Foundation

**Deliverables**:
1. **Data Structures**:
   - Bucket storage design (chaining or open addressing)
   - Hash function for Symbol/Int/String
   - Collision resolution strategy

2. **Core Operations**:
   - `set(key, value)` implementation
   - `get(key)` implementation
   - Basic resizing logic (load factor > 0.75 → double capacity)

3. **Tests**:
   - Basic set/get operations (10 test cases)
   - Hash collision handling (5 test cases)
   - Resizing behavior (3 test cases)

### Week 2: Deterministic Hashing

**Deliverables**:
1. **Hash Strategy**:
   - Symbol hash: Use symbol ID (deterministic)
   - Int hash: Use integer value (deterministic)
   - String hash: Use UTF-8 bytes with stable algorithm (e.g., FNV-1a)

2. **Key Normalization**:
   - Type detection (Symbol vs Int vs String)
   - Unified comparison function
   - Stable sort implementation

3. **Deterministic Iteration**:
   - `keys()` returns sorted keys (Symbol < Int < String)
   - `values()` returns values in key order
   - Provenance tracking (plugin_id, version)

4. **Tests**:
   - Mixed-type key insertion (Symbol, Int, String)
   - Deterministic iteration order (20 test cases)
   - Hash stability (same key → same bucket)

### Week 3: Advanced Operations

**Deliverables**:
1. **Remove Operation**:
   - `remove(key)` implementation
   - Bucket chain repair (if using chaining)
   - Size decrement

2. **Query Operations**:
   - `has(key)` implementation
   - `size()` implementation
   - Edge cases (empty map, single element)

3. **Boundary Checks**:
   - Reject ValueBox keys (Fail-Fast)
   - Ensure DataBox inputs only
   - Clear error messages

4. **Tests**:
   - Remove operations (15 test cases)
   - Edge cases (empty map, remove non-existent key)
   - ValueBox rejection (Fail-Fast verification)

### Week 4: MapBox Golden Tests

**Deliverables**:
1. **Golden Test Suite**:
   - 50+ test cases comparing Rust-MapBox vs Hako-MapBox
   - Identical output verification (keys, values, size)
   - Performance benchmarks (set/get/remove)

2. **Deterministic Verification**:
   - Same input → same output (10 runs per test)
   - Key order stability (Symbol < Int < String)
   - Provenance tracking validation

3. **Performance Baseline**:
   - Measure operation times (set/get/remove)
   - Compare against Rust-MapBox
   - Target: ≥ 70% of Rust performance

4. **Documentation**:
   - MapBox API reference
   - Usage examples
   - Performance characteristics (O(1) average, O(n) worst case)

---

## Week 5-8: ArrayBox Implementation

### Overview

ArrayBox is a dynamic array (list) with automatic capacity expansion.

### Core Methods

```hakorune
box ArrayBox {
    // Storage
    _data: RawBufferBox     // Internal buffer
    _size: IntegerBox       // Current element count
    _capacity: IntegerBox   // Allocated capacity

    // Public API
    push(value: DataBox) -> NullBox
    pop() -> DataBox | NullBox
    get(index: IntegerBox) -> DataBox | NullBox
    set(index: IntegerBox, value: DataBox) -> NullBox
    size() -> IntegerBox
    slice(start: IntegerBox, end: IntegerBox) -> ArrayBox
    concat(other: ArrayBox) -> ArrayBox
}
```

### Week 5: Foundation

**Deliverables**:
1. **Data Structures**:
   - RawBufferBox for storage
   - Capacity expansion strategy (2x growth)
   - Bounds checking

2. **Core Operations**:
   - `push(value)` implementation
   - `pop()` implementation
   - Capacity doubling logic

3. **Tests**:
   - Basic push/pop operations (10 test cases)
   - Capacity expansion (grow from 0 → 1 → 2 → 4 → 8)
   - Empty array edge cases

### Week 6: Index Operations

**Deliverables**:
1. **Index Access**:
   - `get(index)` implementation
   - `set(index, value)` implementation
   - Bounds checking (index < 0 or index ≥ size → error)

2. **Fail-Fast Boundaries**:
   - Out-of-bounds access → RuntimeError
   - Negative index → RuntimeError
   - Clear error messages with context

3. **Tests**:
   - Valid index access (20 test cases)
   - Out-of-bounds access (10 test cases)
   - Negative index handling (5 test cases)

### Week 7: Advanced Operations

**Deliverables**:
1. **Slice Operation**:
   - `slice(start, end)` implementation
   - Return new ArrayBox with elements [start, end)
   - Handle edge cases (start > end, negative indices)

2. **Concat Operation**:
   - `concat(other)` implementation
   - Create new ArrayBox with combined elements
   - Efficient memory allocation

3. **ValueBox/DataBox Boundaries**:
   - Reject ValueBox elements (Fail-Fast)
   - Ensure DataBox inputs only
   - Unpack at entry/exit points

4. **Tests**:
   - Slice operations (15 test cases)
   - Concat operations (10 test cases)
   - ValueBox rejection (5 test cases)

### Week 8: ArrayBox Golden Tests

**Deliverables**:
1. **Golden Test Suite**:
   - 50+ test cases comparing Rust-ArrayBox vs Hako-ArrayBox
   - Identical output verification (size, elements, order)
   - Performance benchmarks (push/pop/get/set)

2. **Performance Baseline**:
   - Measure operation times (all methods)
   - Compare against Rust-ArrayBox
   - Target: ≥ 70% of Rust performance

3. **Large-Scale Tests**:
   - 10,000 element arrays
   - Stress test capacity expansion
   - Memory usage validation

4. **Documentation**:
   - ArrayBox API reference
   - Usage examples
   - Performance characteristics (O(1) amortized, O(n) worst case)

---

## Key Design Decisions

### 1. Deterministic Key Order (Symbol < Int < String)

**Rationale**:
- Predictable iteration order for debugging
- Reproducible behavior across runs
- Stable sorting for golden tests

**Implementation**:
```hakorune
// Key comparison function
method compare_keys(key1: DataBox, key2: DataBox) -> IntegerBox {
    local type1 = key1.type_id()
    local type2 = key2.type_id()

    // Type priority: Symbol < Int < String
    if (type1 != type2) {
        if (type1 == :Symbol) { return -1 }
        if (type2 == :Symbol) { return 1 }
        if (type1 == :Int) { return -1 }
        if (type2 == :Int) { return 1 }
    }

    // Same type: natural order
    return key1.compare(key2)
}
```

### 2. ValueBox/DataBox Boundaries

**Rationale**:
- ValueBox is ephemeral (pipeline boundaries only)
- DataBox is persistent (long-lived storage)
- Fail-Fast prevents confusion

**Implementation**:
```hakorune
// MapBox.set with boundary check
method set(key: DataBox, value: DataBox) -> NullBox {
    if (key.is_valuebox()) {
        panic("MapBox.set: key must be DataBox, not ValueBox")
    }
    if (value.is_valuebox()) {
        panic("MapBox.set: value must be DataBox, not ValueBox")
    }

    // Proceed with actual set operation
    local hash = me._hash(key)
    local bucket = me._buckets.get(hash)
    bucket.insert(key, value)
}
```

### 3. Fail-Fast Error Handling

**Rationale**:
- No silent fallbacks (prevents bugs)
- Clear error messages (aids debugging)
- Early detection (stops at source)

**Implementation**:
```hakorune
// ArrayBox.get with Fail-Fast
method get(index: IntegerBox) -> DataBox | NullBox {
    if (index < 0) {
        panic("ArrayBox.get: negative index: " + index)
    }
    if (index >= me._size) {
        panic("ArrayBox.get: index out of bounds: " + index + " >= " + me._size)
    }

    return me._data.get(index)
}
```

---

## Success Criteria

### Mandatory (✅ All must pass)

1. **MapBox/ArrayBox Fully Implemented**:
   - All methods work in Pure Hakorune
   - No Rust dependencies (except HostBridge)

2. **Golden Tests Pass**:
   - Rust-Collections vs Hako-Collections: 100% parity
   - 100+ test cases with identical output
   - Deterministic behavior verified

3. **Deterministic Behavior**:
   - Key order is predictable (Symbol < Int < String)
   - Iteration order is stable
   - Provenance tracking works

4. **Performance**:
   - Hako-Collections ≥ 70% of Rust-Collections speed
   - Basic operations (set/get/push/pop) are O(1) average
   - Large-scale data (10,000 elements) is practical

### Excluded (⚠️ NOT in Phase 20.7)

- **GC (Garbage Collection)**: Deferred to Phase 20.8
- **Optimization**: Basic implementation only (no profiling yet)
- **Concurrency**: Single-threaded only
- **Persistence**: In-memory only (no disk storage)

---

## Dependencies

### Prerequisite: Phase 20.6

Phase 20.7 requires:
1. **VM Core Complete**: All 16 MIR instructions working
2. **Dispatch Unification**: Single Resolver path for all method calls
3. **Golden Test Framework**: Rust-VM vs Hako-VM comparison working

### Provides to: Phase 20.8

Phase 20.7 delivers:
1. **Collections in Hakorune**: MapBox/ArrayBox fully self-hosted
2. **Deterministic Iteration**: Stable key order for GC root scanning
3. **Reference Tracking**: Preparation for GC mark phase

---

## Risk Mitigation

### Risk 1: Performance Below 70%

**Mitigation**:
- Measure at each week (not just Week 4/8)
- Profile hot paths early (hash function, bounds checks)
- Accept slower performance initially (can optimize later)

### Risk 2: Deterministic Hashing Complexity

**Mitigation**:
- Use simple, proven hash functions (FNV-1a for strings)
- Stable sort implementation (merge sort, not quicksort)
- Test hash stability with 1,000+ keys

### Risk 3: ValueBox/DataBox Confusion

**Mitigation**:
- Clear error messages ("must be DataBox, not ValueBox")
- Fail-Fast at entry points (set/push methods)
- Golden tests verify boundary enforcement

---

## Related Documents

- **Phase 20.5**: [Pure Hakorune Roadmap](../phase-20.5/PURE_HAKORUNE_ROADMAP.md)
- **Phase 20.6**: [VM Core Complete](../phase-20.6/)
- **Phase 20.8**: [GC v0 + Rust Deprecation](../phase-20.8/)
- **Box-First Principle**: [CLAUDE.md](../../../../CLAUDE.md#🧱-先頭原則-箱理論box-first)
- **Golden Testing Strategy**: [PURE_HAKORUNE_ROADMAP.md#🧪-golden-testing-strategy](../phase-20.5/PURE_HAKORUNE_ROADMAP.md#-golden-testing-strategy)

---

**Status**: PLANNED
**Dependencies**: Phase 20.6 (VM Core + Dispatch)
**Delivers**: Phase D (Collections in Pure Hakorune)
**Timeline**: 8 weeks (2026-05-25 → 2026-07-19)
