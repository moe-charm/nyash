Equality Semantics (Boxes and Primitives)

**Last Updated**: 2025-10-08
**Status**: ✅ Fully Implemented

Scope
- Define how `==` / `!=` are interpreted for primitives and user boxes across VM/JIT/LLVM.

Principles
- Primitives (int/float/bool/string): lowered as MIR `Compare(Eq/Ne)` and compiled directly.
- Boxes (user-defined, plugin, runtime boxes): do not emit MIR `Compare(Eq/Ne)` on two Box values.
  - Normalize to a call at MIR to keep VM/LLVM parity.

Normalization (MIR)
- ✅ **Current Implementation** (as of 2025-10-08): `CallTarget::Extern("nyrt.ops.op_eq")`
  - MIR Builder transforms `==` / `!=` → `nyrt.ops.op_eq` extern call
  - Recursion guard: `in_equals_method` flag prevents transformation inside `.equals/1` methods
  - Implementation: `src/mir/builder/ops.rs:169-194`

- Alternative (future): `lhs.equals/1(rhs)` (method on the receiver box).
  - Not yet implemented - requires additional method resolution logic
  - Builders may also verify that `equals/1` exists and otherwise emit a diagnostic.

VM Implementation (2025-10-08)
- ✅ **Runtime Handler**: `handle_op_eq()` with full user-defined equals() support
  - Location: `src/backend/mir_interpreter/handlers/externals.rs:148-183`
  - Delegates to: `src/backend/mir_interpreter/handlers/op_handlers.rs`

- **Equality Semantics**:
  1. **Fast path**: Primitive equality (Integer, Float, Bool, String, Void)
  2. **Pointer check**: Box pointer equality (Arc::ptr_eq)
  3. **User-defined**: Call `Class.equals/1` with `CallMode::NoOperatorGuard`
  4. **Fallback**: Return false for cross-type or no equals() method

- **Recursion Prevention**:
  - MIR Builder: `in_equals_method` flag skips transformation inside equals() bodies
  - VM Runtime: `CallMode::NoOperatorGuard` prevents operator guard interception

Verification
- A MIR verify pass should forbid `Compare(Eq/Ne)` where both operands have `MirType::Box`.
  - The builder is responsible for producing call-based forms instead.

Backend Support (2025-10-08)
- ✅ **VM**: Full implementation with user-defined equals() support
- ✅ **LLVM Python**: `nyrt.ops.op_eq` signature registered (`src/llvm_py/instructions/externcall.py:103`)
- ✅ **Normalize Pass**: Already uses `Callee::Extern` (verified - no changes needed)
- ⏳ **LLVM AOT**: Requires C runtime implementation (separate task - Phase 3.2+)

Notes
- This policy ensures identical behavior for VM and LLVM backends by fixing semantics at the MIR boundary.
- **Test Coverage**: `tools/smokes/v2/profiles/quick/core/equality_box_vm.sh` (3/3 tests PASS)

Related Documentation
- Issue Resolution: `docs/development/issues/equals-stack-overflow.md`
- Phase 19 Day 4: Box Equality Fix (`CURRENT_TASK.md`)
- Op Handlers Module: `src/backend/mir_interpreter/handlers/op_handlers.rs`
