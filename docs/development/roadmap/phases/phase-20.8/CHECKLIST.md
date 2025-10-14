# Phase 20.8: GC + Rust Deprecation - Implementation Checklist

**Duration**: 6 weeks (2026-07-20 → 2026-08-30)
**Status**: Not Started

---

## 📋 Week 1-2: GC Mark Phase

### Task 1.1: GC Roots Detection

- [ ] Create `GcBox` in Hakorune
- [ ] Implement `stack_roots` (ArrayBox)
- [ ] Implement `global_roots` (ArrayBox)
- [ ] Implement `handle_roots` (ArrayBox)
- [ ] Implement `collect_roots()` method
- [ ] Implement `scan_stack_frames()` method
- [ ] Implement `scan_global_boxes()` method
- [ ] Implement `scan_handle_registry()` method
- [ ] Test: Verify all roots found
- [ ] Test: Verify no duplicates in root set

### Task 1.2: Mark Algorithm

- [ ] Implement `marked` (MapBox: object_id → true)
- [ ] Implement `mark()` method
- [ ] Implement `mark_object(obj)` method
- [ ] Implement recursive marking of children
- [ ] Implement cycle detection (avoid infinite loops)
- [ ] Test: Verify mark correctness (all reachable objects marked)
- [ ] Test: Verify cycle handling (no infinite loops)
- [ ] Test: Verify unreachable objects not marked

### Code Review Checkpoint

- [ ] Code review: GC roots detection
- [ ] Code review: Mark algorithm
- [ ] Performance measurement: Mark phase timing
- [ ] Documentation: GC roots design doc

---

## 📋 Week 3-4: GC Sweep Phase & Metrics

### Task 3.1: Sweep Algorithm

- [ ] Implement `all_objects` (ArrayBox)
- [ ] Implement `sweep()` method
- [ ] Implement object destruction (finalization)
- [ ] Implement survivor list update
- [ ] Implement freed object counter
- [ ] Implement sweep timing measurement
- [ ] Test: Verify sweep correctness (all unmarked objects freed)
- [ ] Test: Verify survivors remain intact
- [ ] Test: Verify object finalization called

### Task 3.2: GC Metrics Collection

- [ ] Create `GcMetricsBox` in Hakorune
- [ ] Implement `total_allocations` counter
- [ ] Implement `total_collections` counter
- [ ] Implement `total_freed` counter
- [ ] Implement `peak_handles` counter
- [ ] Implement `increment_allocations()` method
- [ ] Implement `increment_collections()` method
- [ ] Implement `record_collection()` method
- [ ] Implement `print_stats()` method
- [ ] Implement `HAKO_GC_TRACE=1` logging
- [ ] Test: Verify metrics accuracy
- [ ] Test: Verify `HAKO_GC_TRACE=1` output format

### Task 3.3: Integration & Testing

- [ ] Integrate GcBox with MiniVmBox
- [ ] Implement `allocate_object()` hook
- [ ] Implement GC trigger policy
- [ ] Implement stats printing at exit
- [ ] Test: End-to-end GC validation
- [ ] Test: Zero memory leaks (Valgrind)
- [ ] Test: Stress test (1M allocations)
- [ ] Test: Cycle test (cyclic references)
- [ ] Test: Root test (all roots traced)

### Code Review Checkpoint

- [ ] Code review: Sweep algorithm
- [ ] Code review: Metrics collection
- [ ] Code review: VM integration
- [ ] Performance measurement: GC overhead (≤ 10%)
- [ ] Documentation: GC implementation guide

---

## 📋 Week 5: Backend Switching & Deprecation

### Task 5.1: Make Hakorune-VM Default

- [ ] Update CLI argument parsing (Rust)
- [ ] Set `--backend vm` as default
- [ ] Add `--backend vm-rust` (deprecated)
- [ ] Implement deprecation warning
- [ ] Update help text
- [ ] Update documentation (README.md)
- [ ] Update documentation (CLAUDE.md)
- [ ] Update documentation (execution guides)
- [ ] Test: Verify default backend is Hakorune-VM
- [ ] Test: Verify deprecation warning displayed

### Task 5.2: Golden Tests Verification

- [ ] Run golden test suite
- [ ] Verify `arithmetic.hako` (Rust-VM == Hakorune-VM)
- [ ] Verify `control_flow.hako` (Rust-VM == Hakorune-VM)
- [ ] Verify `collections.hako` (Rust-VM == Hakorune-VM)
- [ ] Verify `recursion.hako` (Rust-VM == Hakorune-VM)
- [ ] Verify `strings.hako` (Rust-VM == Hakorune-VM)
- [ ] Verify `enums.hako` (Rust-VM == Hakorune-VM)
- [ ] Verify `closures.hako` (Rust-VM == Hakorune-VM)
- [ ] Verify `selfhost_mini.hako` (Rust-VM == Hakorune-VM)
- [ ] Fix any divergence (if found)
- [ ] Test: All golden tests PASSED (8/8)

### Code Review Checkpoint

- [ ] Code review: CLI changes
- [ ] Code review: Deprecation strategy
- [ ] Golden tests: 100% parity
- [ ] Documentation: Migration guide (Rust-VM → Hakorune-VM)

---

## 📋 Week 6: Bit-Identical Verification & Audit

### Task 6.1: Bit-Identical Self-Compilation

- [ ] Create `verify_self_compilation.sh` script
- [ ] Implement Hako₁ build (Rust-based)
- [ ] Implement Hako₂ build (via Hako₁)
- [ ] Implement Hako₃ build (via Hako₂)
- [ ] Implement Hako₂ == Hako₃ verification
- [ ] Implement Hako₁ == Hako₂ verification (optional)
- [ ] Test: Verify Hako₂ == Hako₃ (bit-identical)
- [ ] Test: Verify script error handling

### Task 6.2: CI Integration

- [ ] Create `.github/workflows/self_compilation.yml`
- [ ] Add push trigger
- [ ] Add pull_request trigger
- [ ] Add daily schedule trigger (midnight)
- [ ] Implement self-compilation chain steps
- [ ] Implement artifact upload (on failure)
- [ ] Test: CI workflow runs successfully
- [ ] Test: CI catches divergence (manual test)

### Task 6.3: Rust Layer Audit

- [ ] Create `audit_rust_layer.sh` script
- [ ] Implement line count check (≤ 100 lines)
- [ ] Implement Rust file listing
- [ ] Verify `host_bridge.rs` line count
- [ ] Identify non-HostBridge Rust files
- [ ] Create migration plan (if needed)
- [ ] Test: Verify Rust layer ≤ 100 lines
- [ ] Test: Verify audit script accuracy

### Task 6.4: Final Documentation

- [ ] Update CURRENT_TASK.md (Phase 20.8 complete)
- [ ] Update README.md (True Self-Hosting achieved)
- [ ] Update CLAUDE.md (development status)
- [ ] Create Phase 20.8 completion report
- [ ] Create Phase 15.82 planning document
- [ ] Update smoke test documentation
- [ ] Update execution mode guide
- [ ] Review all related documentation for accuracy

### Code Review Checkpoint

- [ ] Code review: Self-compilation script
- [ ] Code review: CI workflow
- [ ] Code review: Rust layer audit
- [ ] Final verification: All success criteria met
- [ ] Final verification: All tests pass
- [ ] Final verification: Documentation complete

---

## ✅ Phase E Success Criteria

### GC Implementation

- [ ] GC v0 implemented and functional
- [ ] Mark phase: Trace reachable objects from roots
- [ ] Sweep phase: Free unreachable objects
- [ ] GC roots detected (stack, global, handles)
- [ ] Metrics collection working
- [ ] `HAKO_GC_TRACE=1` provides detailed logs

### Testing & Validation

- [ ] Zero memory leaks in smoke tests
- [ ] Golden tests: GC correctness validated
- [ ] Stress test: 1M allocations handled
- [ ] Cycle test: Cyclic references collected
- [ ] Root test: All roots traced

### Performance

- [ ] GC overhead ≤ 10% of total runtime
- [ ] Mark phase: < 5ms for typical programs
- [ ] Sweep phase: < 10ms for typical programs

---

## ✅ Phase F Success Criteria

### Backend Switching

- [ ] Hakorune-VM is default backend (`--backend vm`)
- [ ] Rust-VM deprecated with clear warning
- [ ] Help text updated
- [ ] Documentation updated

### Golden Tests

- [ ] All golden tests pass (Rust-VM vs Hakorune-VM parity)
- [ ] 100% output match (8/8 tests)
- [ ] No divergence detected

### Self-Compilation

- [ ] Bit-identical self-compilation verified (Hako₂ == Hako₃)
- [ ] Self-compilation script works reliably
- [ ] CI daily verification passes

### Rust Layer

- [ ] Rust layer ≤ 100 lines (HostBridge API only)
- [ ] No non-HostBridge Rust files (or migration plan created)
- [ ] Audit script confirms minimization

---

## ✅ Overall Success Criteria

### True Self-Hosting

- [ ] **Hakorune IS Hakorune**: VM, parser, GC all in Hakorune
- [ ] **Rust=floor, Hakorune=house**: Architecture realized
- [ ] HostBridge API stable (C-ABI boundary)

### Testing

- [ ] All smoke tests pass with Hakorune-VM
- [ ] All golden tests pass (100% parity)
- [ ] Self-compilation verified (bit-identical)
- [ ] CI verification passes (daily)

### Documentation

- [ ] README.md: Self-hosting status clear
- [ ] CLAUDE.md: Development status updated
- [ ] Phase 20.8 completion report published
- [ ] Phase 15.82 planning document created

### Performance

- [ ] Hakorune-VM ≥ 50% of Rust-VM speed
- [ ] GC overhead ≤ 10% of total runtime
- [ ] Production ready (no memory leaks, stable)

---

## 🚀 Post-Phase 20.8 Tasks

### Immediate Follow-up

- [ ] Announce Phase 20.8 completion
- [ ] Update project website (if applicable)
- [ ] Share success story (blog post, social media)
- [ ] Gather feedback from community

### Phase 15.82 Planning

- [ ] Define Phase 15.82 scope (Advanced GC)
- [ ] Create Phase 15.82 planning documents
- [ ] Estimate timeline for Phase 15.82
- [ ] Identify dependencies and risks

### Continuous Improvement

- [ ] Monitor CI for divergence
- [ ] Profile performance bottlenecks
- [ ] Collect GC metrics from production
- [ ] Plan optimization priorities

---

## 📊 Progress Tracking

### Week 1-2 Progress

- [ ] Task 1.1: GC Roots Detection (0/10)
- [ ] Task 1.2: Mark Algorithm (0/8)
- [ ] Code Review Checkpoint (0/4)

### Week 3-4 Progress

- [ ] Task 3.1: Sweep Algorithm (0/9)
- [ ] Task 3.2: GC Metrics Collection (0/12)
- [ ] Task 3.3: Integration & Testing (0/9)
- [ ] Code Review Checkpoint (0/5)

### Week 5 Progress

- [ ] Task 5.1: Make Hakorune-VM Default (0/10)
- [ ] Task 5.2: Golden Tests Verification (0/12)
- [ ] Code Review Checkpoint (0/4)

### Week 6 Progress

- [ ] Task 6.1: Bit-Identical Self-Compilation (0/8)
- [ ] Task 6.2: CI Integration (0/8)
- [ ] Task 6.3: Rust Layer Audit (0/8)
- [ ] Task 6.4: Final Documentation (0/8)
- [ ] Code Review Checkpoint (0/6)

### Overall Progress

- [ ] Phase E Success Criteria (0/17)
- [ ] Phase F Success Criteria (0/13)
- [ ] Overall Success Criteria (0/13)

**Total Tasks**: 106
**Completed**: 0
**Remaining**: 106
**Progress**: 0%

---

**Status**: Not Started
**Start Date**: 2026-07-20
**Target Completion**: 2026-08-30
**Dependencies**: Phase 20.7 complete

**Last Updated**: 2025-10-14
