# Phase 20.7: Collections in Hakorune - Implementation Checklist

**Duration**: 8 weeks (2026-05-25 → 2026-07-19)
**Status**: PLANNED

---

## Week 1: MapBox Foundation

### Data Structures
- [ ] Design bucket storage (chaining or open addressing)
- [ ] Implement hash function for Symbol type
- [ ] Implement hash function for Int type
- [ ] Implement hash function for String type (FNV-1a or similar)
- [ ] Design collision resolution strategy

### Core Operations
- [ ] Implement `MapBox._hash(key)` internal method
- [ ] Implement `MapBox.set(key, value)` method
- [ ] Implement `MapBox.get(key)` method
- [ ] Implement basic resizing logic (load factor > 0.75)
- [ ] Implement bucket allocation/deallocation

### Tests
- [ ] Test basic set operation (10 test cases)
- [ ] Test basic get operation (10 test cases)
- [ ] Test hash collision handling (5 test cases)
- [ ] Test resizing behavior (3 test cases)
- [ ] Test empty map edge cases (3 test cases)

### Documentation
- [ ] Document bucket storage design
- [ ] Document hash functions (Symbol/Int/String)
- [ ] Document collision resolution strategy
- [ ] Document resizing algorithm

---

## Week 2: Deterministic Hashing & Key Order

### Hash Strategy
- [ ] Implement deterministic Symbol hash (use symbol ID)
- [ ] Implement deterministic Int hash (use integer value)
- [ ] Implement deterministic String hash (stable UTF-8 algorithm)
- [ ] Test hash stability (same key → same bucket, 100 runs)

### Key Normalization
- [ ] Implement type detection (Symbol vs Int vs String)
- [ ] Implement unified comparison function (`compare_keys`)
- [ ] Implement stable sort for mixed-type keys
- [ ] Verify key comparison order: Symbol < Int < String

### Deterministic Iteration
- [ ] Implement `MapBox.keys()` method (returns sorted keys)
- [ ] Implement `MapBox.values()` method (returns values in key order)
- [ ] Implement provenance tracking (plugin_id, version)
- [ ] Verify iteration order stability (same input → same output, 10 runs)

### Tests
- [ ] Test mixed-type key insertion (Symbol, Int, String) (10 test cases)
- [ ] Test deterministic iteration order (20 test cases)
- [ ] Test key order: Symbol < Int < String (5 test cases)
- [ ] Test hash stability across runs (10 test cases)
- [ ] Test provenance tracking (3 test cases)

### Documentation
- [ ] Document deterministic hash strategy
- [ ] Document key comparison order (Symbol < Int < String)
- [ ] Document provenance tracking design

---

## Week 3: Advanced MapBox Operations

### Remove Operation
- [ ] Implement `MapBox.remove(key)` method
- [ ] Implement bucket chain repair (if using chaining)
- [ ] Implement size decrement logic
- [ ] Handle remove of non-existent key (return NullBox)

### Query Operations
- [ ] Implement `MapBox.has(key)` method
- [ ] Implement `MapBox.size()` method
- [ ] Handle edge cases (empty map, single element)
- [ ] Verify O(1) average time complexity

### Boundary Checks (Fail-Fast)
- [ ] Reject ValueBox keys (panic with clear message)
- [ ] Reject ValueBox values (panic with clear message)
- [ ] Ensure DataBox-only inputs
- [ ] Implement entry point unpack logic

### Tests
- [ ] Test remove operation (15 test cases)
- [ ] Test remove non-existent key (5 test cases)
- [ ] Test has operation (10 test cases)
- [ ] Test size operation (10 test cases)
- [ ] Test edge cases (empty map, single element) (5 test cases)
- [ ] Test ValueBox rejection (Fail-Fast) (5 test cases)

### Documentation
- [ ] Document remove operation design
- [ ] Document ValueBox/DataBox boundary enforcement
- [ ] Document Fail-Fast error messages

---

## Week 4: MapBox Golden Tests & Performance

### Golden Test Suite
- [ ] Create 50+ test cases for MapBox
- [ ] Compare Rust-MapBox vs Hako-MapBox output
- [ ] Verify identical output for all test cases
- [ ] Test deterministic behavior (same input → same output, 10 runs per test)
- [ ] Test key order stability (Symbol < Int < String)

### Deterministic Verification
- [ ] Run each test 10 times, verify identical output
- [ ] Test with 1,000+ keys (mixed types)
- [ ] Verify provenance tracking in golden tests
- [ ] Verify iteration order matches Rust-MapBox

### Performance Baseline
- [ ] Measure set operation time (per 1,000 operations)
- [ ] Measure get operation time (per 1,000 operations)
- [ ] Measure remove operation time (per 1,000 operations)
- [ ] Compare against Rust-MapBox
- [ ] Verify performance ≥ 70% of Rust-MapBox
- [ ] Profile hot paths (hash function, collision resolution)

### Documentation
- [ ] Document MapBox API reference
- [ ] Document usage examples (basic, advanced)
- [ ] Document performance characteristics (O(1) average, O(n) worst case)
- [ ] Document golden test results
- [ ] Document performance benchmarks

### Week 4 Milestone Review
- [ ] All MapBox methods implemented in Pure Hakorune
- [ ] Golden tests: 50+ test cases PASS
- [ ] Deterministic key order verified
- [ ] Performance ≥ 70% of Rust-MapBox
- [ ] Ready to proceed to ArrayBox implementation

---

## Week 5: ArrayBox Foundation

### Data Structures
- [ ] Design RawBufferBox for storage
- [ ] Implement capacity expansion strategy (2x growth)
- [ ] Implement bounds checking logic
- [ ] Design element addressing (index → buffer offset)

### Core Operations
- [ ] Implement `ArrayBox.push(value)` method
- [ ] Implement `ArrayBox.pop()` method
- [ ] Implement capacity doubling logic
- [ ] Handle empty array edge cases

### Tests
- [ ] Test basic push operation (10 test cases)
- [ ] Test basic pop operation (10 test cases)
- [ ] Test capacity expansion (grow from 0 → 1 → 2 → 4 → 8) (5 test cases)
- [ ] Test empty array edge cases (5 test cases)
- [ ] Test push/pop sequence (10 test cases)

### Documentation
- [ ] Document RawBufferBox design
- [ ] Document capacity expansion strategy
- [ ] Document push/pop algorithms

---

## Week 6: Index Operations & Fail-Fast

### Index Access
- [ ] Implement `ArrayBox.get(index)` method
- [ ] Implement `ArrayBox.set(index, value)` method
- [ ] Implement bounds checking (index < 0 or index ≥ size)
- [ ] Verify O(1) time complexity

### Fail-Fast Boundaries
- [ ] Out-of-bounds access → RuntimeError (not panic)
- [ ] Negative index → RuntimeError with clear message
- [ ] Reject ValueBox elements (panic with clear message)
- [ ] Ensure DataBox-only inputs

### Tests
- [ ] Test valid index access (20 test cases)
- [ ] Test out-of-bounds access (10 test cases)
- [ ] Test negative index handling (5 test cases)
- [ ] Test get/set sequence (10 test cases)
- [ ] Test ValueBox rejection (Fail-Fast) (5 test cases)

### Documentation
- [ ] Document index access design
- [ ] Document bounds checking strategy
- [ ] Document Fail-Fast error messages

---

## Week 7: Advanced ArrayBox Operations

### Slice Operation
- [ ] Implement `ArrayBox.slice(start, end)` method
- [ ] Return new ArrayBox with elements [start, end)
- [ ] Handle edge cases (start > end, negative indices)
- [ ] Handle out-of-bounds start/end (clamp or error)

### Concat Operation
- [ ] Implement `ArrayBox.concat(other)` method
- [ ] Create new ArrayBox with combined elements
- [ ] Efficient memory allocation (pre-allocate total size)
- [ ] Handle empty array concatenation

### ValueBox/DataBox Boundaries
- [ ] Reject ValueBox elements in push/set (Fail-Fast)
- [ ] Ensure DataBox-only inputs
- [ ] Implement entry/exit point unpack logic
- [ ] Clear error messages for boundary violations

### Tests
- [ ] Test slice operation (15 test cases)
- [ ] Test slice edge cases (start > end, out-of-bounds) (5 test cases)
- [ ] Test concat operation (10 test cases)
- [ ] Test concat empty arrays (3 test cases)
- [ ] Test ValueBox/DataBox boundaries (5 test cases)

### Documentation
- [ ] Document slice operation design
- [ ] Document concat operation design
- [ ] Document ValueBox/DataBox boundary enforcement

---

## Week 8: ArrayBox Golden Tests & Performance

### Golden Test Suite
- [ ] Create 50+ test cases for ArrayBox
- [ ] Compare Rust-ArrayBox vs Hako-ArrayBox output
- [ ] Verify identical output for all test cases
- [ ] Test bounds checking Fail-Fast behavior
- [ ] Test slice/concat operations

### Performance Baseline
- [ ] Measure push operation time (per 1,000 operations)
- [ ] Measure pop operation time (per 1,000 operations)
- [ ] Measure get operation time (per 1,000 operations)
- [ ] Measure set operation time (per 1,000 operations)
- [ ] Compare against Rust-ArrayBox
- [ ] Verify performance ≥ 70% of Rust-ArrayBox
- [ ] Profile hot paths (bounds checks, capacity expansion)

### Large-Scale Tests
- [ ] Test 10,000 element array (push all, get all)
- [ ] Stress test capacity expansion (0 → 10,000)
- [ ] Measure memory usage (verify no leaks)
- [ ] Test slice on large array (10,000 elements)
- [ ] Test concat on large arrays (5,000 + 5,000)

### Documentation
- [ ] Document ArrayBox API reference
- [ ] Document usage examples (basic, advanced)
- [ ] Document performance characteristics (O(1) amortized, O(n) worst case)
- [ ] Document golden test results
- [ ] Document performance benchmarks

### Week 8 Milestone Review
- [ ] All ArrayBox methods implemented in Pure Hakorune
- [ ] Golden tests: 50+ test cases PASS
- [ ] Bounds checking Fail-Fast verified
- [ ] Performance ≥ 70% of Rust-ArrayBox
- [ ] Large-scale tests (10,000 elements) PASS

---

## Phase 20.7 Final Verification

### Combined Golden Tests
- [ ] Run all MapBox golden tests (50+ cases)
- [ ] Run all ArrayBox golden tests (50+ cases)
- [ ] Verify 100+ total test cases PASS
- [ ] Verify deterministic behavior across all tests
- [ ] Verify ValueBox/DataBox boundaries enforced

### Performance Validation
- [ ] MapBox performance ≥ 70% of Rust-MapBox
- [ ] ArrayBox performance ≥ 70% of Rust-ArrayBox
- [ ] Large-scale tests (10,000 elements) practical speed
- [ ] No memory leaks detected

### Integration Tests
- [ ] Test MapBox with ArrayBox values
- [ ] Test ArrayBox with MapBox elements
- [ ] Test nested collections (Map of Arrays, Array of Maps)
- [ ] Test with selfhost compiler (use collections in compiler code)

### Documentation Complete
- [ ] README.md finalized (Phase 20.7 overview)
- [ ] PLAN.md finalized (implementation details)
- [ ] INDEX.md updated (all references correct)
- [ ] CHECKLIST.md completed (this file)
- [ ] API reference complete (MapBox, ArrayBox)
- [ ] Performance benchmarks documented

### Readiness for Phase 20.8
- [ ] Collections ready for GC root scanning
- [ ] Reference tracking infrastructure in place
- [ ] Deterministic iteration verified (for mark phase)
- [ ] No Rust dependencies (except HostBridge)

---

## Phase 20.7 Success Criteria (Final)

### Mandatory (✅ All must be checked)
- [ ] MapBox/ArrayBox fully in Pure Hakorune
- [ ] No Rust dependencies (except HostBridge)
- [ ] Golden tests: 100+ test cases PASS
- [ ] Rust-Collections vs Hako-Collections: 100% parity
- [ ] Deterministic behavior verified (Symbol < Int < String)
- [ ] Iteration order stable
- [ ] Provenance tracking works
- [ ] Performance ≥ 70% of Rust-Collections
- [ ] Basic operations (set/get/push/pop) O(1) average time
- [ ] Large-scale data (10,000 elements) practical speed

### Excluded (⚠️ NOT in Phase 20.7)
- [ ] Confirmed: GC not implemented (deferred to Phase 20.8)
- [ ] Confirmed: No optimization beyond basic implementation
- [ ] Confirmed: Single-threaded only (no concurrency)
- [ ] Confirmed: In-memory only (no persistence)

---

**Status**: PLANNED
**All Checkboxes**: Uncompleted ([ ])
**Review Date**: Week 4 (2026-06-22), Week 8 (2026-07-19)
**Phase End Date**: 2026-07-19
