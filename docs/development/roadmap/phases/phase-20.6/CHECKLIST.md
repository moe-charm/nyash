# Phase 20.6 — Checklist

**Status**: Planning
**Duration**: 12 weeks (2026-03-01 → 2026-05-24)

---

## Phase B Complete: VM Core (Week 1-6)

### Week 1-2: Memory Operations (2026-03-01 - 03-14)

#### Implementation

- [ ] Design memory operation handlers
- [ ] Implement `load_handler.hako`
  - [ ] Load value from memory
  - [ ] Handle invalid memory addresses
  - [ ] Result-based error handling
- [ ] Implement `store_handler.hako`
  - [ ] Store value to memory
  - [ ] Handle memory overflow
  - [ ] Result-based error handling
- [ ] Implement `copy_handler.hako`
  - [ ] Copy value between registers
  - [ ] Handle invalid register IDs
  - [ ] Result-based error handling

#### Testing

- [ ] Create test suite: `tests/golden/phase-b/memory/`
- [ ] `test_load_basic.hako` - Basic load operation
- [ ] `test_store_basic.hako` - Basic store operation
- [ ] `test_copy_basic.hako` - Basic copy operation
- [ ] `test_load_invalid_address.hako` - Error handling
- [ ] `test_store_overflow.hako` - Error handling
- [ ] `test_memory_aliasing.hako` - Multiple references
- [ ] `test_load_store_cycle.hako` - Load after store
- [ ] `test_copy_chain.hako` - Multiple copies
- [ ] `test_memory_edge_cases.hako` - Edge cases
- [ ] `test_memory_stress.hako` - Stress test

#### Integration

- [ ] Update instruction dispatcher to handle new ops
- [ ] Run Golden Tests for memory operations
- [ ] Measure performance vs Rust-VM
- [ ] Document memory operation design

---

### Week 3-4: Method Call Instructions (2026-03-15 - 03-28)

#### Implementation

- [ ] Design unified MirCall handler
- [ ] Implement `call_handler.hako` (Unified MirCall)
  - [ ] Handle Global function calls
  - [ ] Handle Module function calls
  - [ ] Handle Closure calls
  - [ ] Result-based error handling
- [ ] Implement `boxcall_handler.hako`
  - [ ] Handle Box method calls
  - [ ] Method lookup by name
  - [ ] Arity validation
  - [ ] Result-based error handling
- [ ] Implement `externcall_handler.hako`
  - [ ] Handle external function calls (nyrt.time, etc.)
  - [ ] Interface/method resolution
  - [ ] Effect tracking
  - [ ] Result-based error handling

#### Testing

- [ ] Create test suite: `tests/golden/phase-b/calls/`
- [ ] `test_global_function_call.hako` - Global function
- [ ] `test_module_function_call.hako` - Module function
- [ ] `test_closure_call_basic.hako` - Basic closure
- [ ] `test_closure_call_capture.hako` - Closure with capture
- [ ] `test_boxcall_method.hako` - Box method call
- [ ] `test_boxcall_arity_check.hako` - Arity validation
- [ ] `test_externcall_time.hako` - External call (time)
- [ ] `test_externcall_array_size.hako` - External call (array.size)
- [ ] `test_call_recursion.hako` - Recursive call
- [ ] `test_call_nested.hako` - Nested calls
- [ ] `test_call_with_complex_args.hako` - Complex arguments
- [ ] `test_call_error_handling.hako` - Error propagation
- [ ] `test_mircall_all_variants.hako` - All MirCall types
- [ ] `test_boxcall_inheritance.hako` - Method inheritance
- [ ] `test_externcall_unknown.hako` - Unknown extern (Fail-Fast)

#### Integration

- [ ] Update instruction dispatcher to handle call ops
- [ ] Run Golden Tests for call operations
- [ ] Measure call performance vs Rust-VM
- [ ] Document MirCall unification design

---

### Week 5: Type Operations (2026-03-29 - 04-04)

#### Implementation

- [ ] Design type operation handlers
- [ ] Implement `typeop_handler.hako`
  - [ ] Type checking (is_a?)
  - [ ] Type casting (as)
  - [ ] Type comparison
  - [ ] Result-based error handling
- [ ] Implement `newbox_handler.hako`
  - [ ] Box creation
  - [ ] Field initialization
  - [ ] Birth lifecycle integration
  - [ ] Result-based error handling

#### Testing

- [ ] Create test suite: `tests/golden/phase-b/types/`
- [ ] `test_typeop_check.hako` - Type checking
- [ ] `test_typeop_cast.hako` - Type casting
- [ ] `test_typeop_compare.hako` - Type comparison
- [ ] `test_typeop_invalid.hako` - Invalid type operation
- [ ] `test_newbox_simple.hako` - Simple box creation
- [ ] `test_newbox_with_fields.hako` - Box with fields
- [ ] `test_newbox_birth_lifecycle.hako` - Birth method call
- [ ] `test_newbox_inheritance.hako` - Box inheritance
- [ ] `test_newbox_error_handling.hako` - Creation errors
- [ ] `test_type_operations_stress.hako` - Stress test

#### Integration

- [ ] Update instruction dispatcher to handle type ops
- [ ] Run Golden Tests for type operations
- [ ] Measure type operation performance
- [ ] Document type system design

---

### Week 6: Control Optimizations + Golden Tests (2026-04-05 - 04-11)

#### Implementation

- [ ] Implement `barrier_handler.hako`
  - [ ] GC barrier semantics
  - [ ] Memory ordering
- [ ] Implement `safepoint_handler.hako`
  - [ ] GC safepoint semantics
  - [ ] Stack scanning preparation
- [ ] Implement `loopform_handler.hako`
  - [ ] Loop detection hint
  - [ ] Optimization metadata
- [ ] Implement `unaryop_handler.hako`
  - [ ] Unary negation
  - [ ] Logical NOT
  - [ ] Result-based error handling

#### Golden Test Suite (100+ tests)

##### Arithmetic (10 tests)
- [ ] `test_arithmetic_add.hako`
- [ ] `test_arithmetic_sub.hako`
- [ ] `test_arithmetic_mul.hako`
- [ ] `test_arithmetic_div.hako`
- [ ] `test_arithmetic_mod.hako`
- [ ] `test_arithmetic_overflow.hako`
- [ ] `test_arithmetic_underflow.hako`
- [ ] `test_arithmetic_divide_by_zero.hako`
- [ ] `test_arithmetic_complex.hako`
- [ ] `test_arithmetic_stress.hako`

##### Control Flow (10 tests)
- [ ] `test_control_if_else.hako`
- [ ] `test_control_loop.hako`
- [ ] `test_control_nested_loop.hako`
- [ ] `test_control_branch.hako`
- [ ] `test_control_jump.hako`
- [ ] `test_control_phi_simple.hako`
- [ ] `test_control_phi_complex.hako`
- [ ] `test_control_early_return.hako`
- [ ] `test_control_break_continue.hako`
- [ ] `test_control_stress.hako`

##### Collections (10 tests)
- [ ] `test_collections_array_ops.hako`
- [ ] `test_collections_map_ops.hako`
- [ ] `test_collections_array_iteration.hako`
- [ ] `test_collections_map_iteration.hako`
- [ ] `test_collections_nested_arrays.hako`
- [ ] `test_collections_nested_maps.hako`
- [ ] `test_collections_mixed.hako`
- [ ] `test_collections_large_array.hako`
- [ ] `test_collections_large_map.hako`
- [ ] `test_collections_stress.hako`

##### Recursion (5 tests)
- [ ] `test_recursion_factorial.hako`
- [ ] `test_recursion_fibonacci.hako`
- [ ] `test_recursion_ackermann.hako`
- [ ] `test_recursion_mutual.hako`
- [ ] `test_recursion_deep.hako`

##### Closures (5 tests)
- [ ] `test_closures_capture.hako`
- [ ] `test_closures_nested.hako`
- [ ] `test_closures_multiple_capture.hako`
- [ ] `test_closures_return_closure.hako`
- [ ] `test_closures_closure_chain.hako`

##### Strings (10 tests)
- [ ] `test_strings_concat.hako`
- [ ] `test_strings_substring.hako`
- [ ] `test_strings_length.hako`
- [ ] `test_strings_compare.hako`
- [ ] `test_strings_empty.hako`
- [ ] `test_strings_unicode.hako`
- [ ] `test_strings_escape.hako`
- [ ] `test_strings_long.hako`
- [ ] `test_strings_interpolation.hako`
- [ ] `test_strings_stress.hako`

##### Types (10 tests)
- [ ] `test_types_check.hako`
- [ ] `test_types_cast.hako`
- [ ] `test_types_newbox.hako`
- [ ] `test_types_inheritance.hako`
- [ ] `test_types_polymorphism.hako`
- [ ] `test_types_interface.hako`
- [ ] `test_types_null_handling.hako`
- [ ] `test_types_error_propagation.hako`
- [ ] `test_types_complex.hako`
- [ ] `test_types_stress.hako`

##### Memory (10 tests)
- [ ] `test_memory_load.hako`
- [ ] `test_memory_store.hako`
- [ ] `test_memory_copy.hako`
- [ ] `test_memory_aliasing.hako`
- [ ] `test_memory_stack.hako`
- [ ] `test_memory_heap.hako`
- [ ] `test_memory_gc_barrier.hako`
- [ ] `test_memory_gc_safepoint.hako`
- [ ] `test_memory_leak_check.hako`
- [ ] `test_memory_stress.hako`

##### Complex (30 tests)
- [ ] `test_complex_mini_compiler.hako`
- [ ] `test_complex_json_parser.hako`
- [ ] `test_complex_expression_evaluator.hako`
- [ ] `test_complex_state_machine.hako`
- [ ] `test_complex_tree_traversal.hako`
- [ ] `test_complex_graph_algorithm.hako`
- [ ] `test_complex_sorting.hako`
- [ ] `test_complex_searching.hako`
- [ ] `test_complex_pattern_matching.hako`
- [ ] `test_complex_regex.hako`
- [ ] `test_complex_http_parser.hako`
- [ ] `test_complex_csv_parser.hako`
- [ ] `test_complex_xml_parser.hako`
- [ ] `test_complex_markdown_parser.hako`
- [ ] `test_complex_code_formatter.hako`
- [ ] `test_complex_calculator.hako`
- [ ] `test_complex_interpreter.hako`
- [ ] `test_complex_vm_simulator.hako`
- [ ] `test_complex_assembler.hako`
- [ ] `test_complex_disassembler.hako`
- [ ] `test_complex_profiler.hako`
- [ ] `test_complex_debugger.hako`
- [ ] `test_complex_optimizer.hako`
- [ ] `test_complex_code_generator.hako`
- [ ] `test_complex_type_checker.hako`
- [ ] `test_complex_linter.hako`
- [ ] `test_complex_test_framework.hako`
- [ ] `test_complex_build_system.hako`
- [ ] `test_complex_package_manager.hako`
- [ ] `test_complex_stress_all.hako`

#### Performance Measurement

- [ ] Run all Golden Tests (100+)
- [ ] Measure Hako-VM vs Rust-VM performance
- [ ] Generate performance report
- [ ] Verify: Hako-VM ≥ 50% of Rust-VM

#### Week 6 Deliverables

- [ ] All 16 MIR instructions implemented
- [ ] 100+ Golden Tests ALL PASS
- [ ] Performance report complete
- [ ] Phase B completion report

---

## Phase C: Dispatch Unification (Week 7-12)

### Week 7-8: Resolver Integration (2026-04-12 - 04-25)

#### Design

- [ ] Finalize Resolver architecture
- [ ] Design MethodHandle structure
- [ ] Design type registry structure
- [ ] Review with ChatGPT/Claude

#### Implementation

- [ ] Implement `resolver_box.hako`
  - [ ] `lookup(type_id, method, arity) -> MethodHandle`
  - [ ] Type registry initialization
  - [ ] Method registration API
  - [ ] Fail-Fast for unknown methods
- [ ] Implement `method_handle_box.hako`
  - [ ] Handle storage (implementation + metadata)
  - [ ] Arity information
  - [ ] Type information
- [ ] Implement `type_registry_box.hako`
  - [ ] Type registration
  - [ ] Method registry per type
  - [ ] Lookup optimization

#### Testing

- [ ] Create test suite: `tests/golden/phase-c/resolver/`
- [ ] `test_resolver_lookup_success.hako` - Successful lookup
- [ ] `test_resolver_lookup_failure.hako` - Fail-Fast behavior
- [ ] `test_resolver_arity_mismatch.hako` - Arity check
- [ ] `test_resolver_type_not_found.hako` - Unknown type
- [ ] `test_resolver_method_not_found.hako` - Unknown method
- [ ] `test_resolver_registration.hako` - Type registration
- [ ] `test_resolver_override.hako` - Method override
- [ ] `test_resolver_inheritance.hako` - Method inheritance
- [ ] `test_resolver_multiple_types.hako` - Multiple types
- [ ] `test_resolver_performance.hako` - Lookup performance
- [ ] `test_resolver_edge_cases.hako` (10 tests) - Edge cases

#### Integration

- [ ] Integrate Resolver with call_handler
- [ ] Run integration tests (20 tests)
- [ ] Measure lookup performance
- [ ] Document Resolver design

---

### Week 9-10: CallableBox Refactoring (2026-04-26 - 05-09)

#### Design

- [ ] Finalize CallableBox architecture
- [ ] Design NoOperatorGuard mechanism
- [ ] Design macro desugaring strategy
- [ ] Review with ChatGPT/Claude

#### Implementation

- [ ] Implement `exec_box.hako`
  - [ ] `call_by_handle(handle, args, guard)`
  - [ ] Argument validation
  - [ ] NoOperatorGuard integration
  - [ ] Error handling
- [ ] Implement `no_operator_guard_box.hako`
  - [ ] Guard activation/deactivation
  - [ ] Recursion prevention (for op_eq, etc.)
  - [ ] Thread-local state (if needed)
- [ ] Implement macro desugaring (compiler side)
  - [ ] `arr.push(value)` → `Callable.ref_method(arr, :push, 1).call([value])`
  - [ ] Operator desugaring (e.g., `a + b` → `Callable.ref_operator(a, :+, 1).call([b])`)

#### Testing

- [ ] Create test suite: `tests/golden/phase-c/callable/`
- [ ] `test_callable_call_by_handle.hako` - Basic call
- [ ] `test_callable_no_operator_guard.hako` - Guard mechanism
- [ ] `test_callable_recursion_prevention.hako` - Prevent re-entry
- [ ] `test_callable_arity_validation.hako` - Arity check
- [ ] `test_callable_error_propagation.hako` - Error handling
- [ ] `test_macro_desugar_method.hako` - Method desugaring
- [ ] `test_macro_desugar_operator.hako` - Operator desugaring
- [ ] `test_macro_desugar_chain.hako` - Method chaining
- [ ] `test_macro_desugar_nested.hako` - Nested calls
- [ ] `test_callable_performance.hako` - Call performance
- [ ] `test_callable_edge_cases.hako` (15 tests) - Edge cases

#### Integration

- [ ] Integrate ExecBox with Resolver
- [ ] Update compiler for macro desugaring
- [ ] Run integration tests (25 tests)
- [ ] Measure call overhead
- [ ] Document CallableBox design

---

### Week 11: Universal Route Minimization (2026-05-10 - 05-16)

#### Removal Tasks

- [ ] Identify all special-case dispatch code
- [ ] Remove Global function special case
- [ ] Remove Box method special case
- [ ] Remove Closure special case
- [ ] Remove Constructor special case
- [ ] Remove Module function special case
- [ ] Remove hardcoded method tables
- [ ] Remove fallback implementations

#### Unification Tasks

- [ ] All calls go through Resolver.lookup
- [ ] All invocations go through ExecBox.call_by_handle
- [ ] Verify Fail-Fast for unknown methods
- [ ] Verify no silent fallbacks

#### Testing

- [ ] Create test suite: `tests/golden/phase-c/unification/`
- [ ] `test_unification_no_global_special_case.hako` - No special case
- [ ] `test_unification_no_box_special_case.hako` - No special case
- [ ] `test_unification_no_closure_special_case.hako` - No special case
- [ ] `test_unification_no_constructor_special_case.hako` - No special case
- [ ] `test_unification_no_module_special_case.hako` - No special case
- [ ] `test_unification_fail_fast_unknown_method.hako` - Fail-Fast
- [ ] `test_unification_fail_fast_wrong_arity.hako` - Fail-Fast
- [ ] `test_unification_no_fallback.hako` - No fallback
- [ ] `test_unification_all_paths.hako` (22 tests) - All code paths

#### Codebase Cleanup

- [ ] Remove unused code
- [ ] Simplify control flow
- [ ] Consolidate error handling
- [ ] Update documentation

#### Integration

- [ ] Run all Golden Tests again (100+)
- [ ] Verify no regressions
- [ ] Measure performance (should be same or better)
- [ ] Document route minimization

---

### Week 12: Integration + Documentation (2026-05-17 - 05-24)

#### Integration Testing

- [ ] Re-run all Golden Tests (155+ tests)
  - [ ] Phase B tests: 50 tests
  - [ ] Phase C tests: 75 tests
  - [ ] Integration tests: 30 tests
- [ ] Verify 100% PASS rate
- [ ] Run stress tests
- [ ] Run regression tests

#### Performance Testing

- [ ] Run benchmark suite
- [ ] Measure Hako-VM vs Rust-VM
- [ ] Verify: Hako-VM ≥ 50% of Rust-VM
- [ ] Generate performance report
- [ ] Identify bottlenecks (for Phase 20.7)

#### Documentation

- [ ] **VM Core Complete Reference**
  - [ ] All 16 instructions documented
  - [ ] Control flow patterns
  - [ ] Memory model
  - [ ] Error handling
- [ ] **Dispatch Unification Design Doc**
  - [ ] Resolver architecture
  - [ ] CallableBox design
  - [ ] NoOperatorGuard mechanism
  - [ ] Macro desugaring strategy
- [ ] **Performance Report**
  - [ ] Benchmark results
  - [ ] Performance comparison
  - [ ] Bottleneck analysis
  - [ ] Optimization recommendations
- [ ] **Phase 20.6 Completion Report**
  - [ ] Summary of achievements
  - [ ] Lessons learned
  - [ ] Known issues
  - [ ] Recommendations for Phase 20.7
- [ ] **Phase 20.7 Planning Doc**
  - [ ] Collections in Hakorune
  - [ ] MapBox implementation plan
  - [ ] ArrayBox implementation plan
  - [ ] Deterministic behavior design

#### Code Review

- [ ] Self-review complete
- [ ] ChatGPT review complete
- [ ] Claude review complete
- [ ] Address all review comments

#### CI Integration

- [ ] All Golden Tests passing in CI
- [ ] Performance benchmarks in CI
- [ ] Documentation builds in CI
- [ ] Release notes prepared

---

## Summary Checklist

### Phase B Complete (Week 1-6)

- [ ] All 16 MIR instructions implemented
- [ ] 50+ Golden Tests for Phase B
- [ ] Performance: Hako-VM ≥ 50% of Rust-VM
- [ ] Phase B completion report

### Phase C Complete (Week 7-12)

- [ ] Resolver.lookup implemented
- [ ] ExecBox.call_by_handle implemented
- [ ] Special-case dispatch removed
- [ ] 75+ Golden Tests for Phase C
- [ ] Phase C completion report

### Overall Phase 20.6

- [ ] 155+ Golden Tests ALL PASS
- [ ] Dispatch unified (single path)
- [ ] Performance verified (≥ 50%)
- [ ] Documentation complete
- [ ] Phase 20.7 planned
- [ ] Phase 20.6 completion report submitted

---

**Created**: 2025-10-14
**Last Updated**: 2025-10-14
**Status**: Planning
**Total Tasks**: 200+ checkboxes
**All Uncompleted**: [ ] (ready for execution)
