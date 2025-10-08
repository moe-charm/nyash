Equality Semantics (Boxes and Primitives)

Scope
- Define how `==` / `!=` are interpreted for primitives and user boxes across VM/JIT/LLVM.

Principles
- Primitives (int/float/bool/string): lowered as MIR `Compare(Eq/Ne)` and compiled directly.
- Boxes (user-defined, plugin, runtime boxes): do not emit MIR `Compare(Eq/Ne)` on two Box values.
  - Normalize to a call at MIR to keep VM/LLVM parity.

Normalization (MIR)
- Preferred: `lhs.equals/1(rhs)` (method on the receiver box).
  - Do not apply this rewrite inside `.equals/…` bodies to avoid recursion.
  - Builders may also verify that `equals/1` exists and otherwise emit a diagnostic.
- Alternative (universal) fallback: `MirCall::external("nyrt.ops.op_eq")`.
  - Use the unified MirCall form (`Callee::Extern`) rather than legacy `ExternCall`.

VM adapter (compatibility)
- The VM provides a guarded fallback (`eval_equals`) that calls `Class.equals/1` if present.
  - Reentrancy guard: comparisons inside `.equals/…` skip dynamic dispatch to avoid infinite recursion.
  - This is a compatibility aid only; canonical semantics are determined by the MIR shape.

Verification
- A MIR verify pass should forbid `Compare(Eq/Ne)` where both operands have `MirType::Box`.
  - The builder is responsible for producing call-based forms instead.

Notes
- This policy ensures identical behavior for VM and LLVM backends by fixing semantics at the MIR boundary.
