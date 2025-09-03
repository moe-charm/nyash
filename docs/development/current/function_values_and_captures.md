# Function Values, Captures, and Events

Summary of current behavior and guidelines.

- Function values: created via `function(...) { ... }` produce a `FunctionBox` with
  captured environment (by-ref for locals via `RefCellBox`, by-value for globals/statics) and optional weak `me`.
- Assignment cell reflection: assigning to a variable or field bound to `RefCellBox` updates the inner value instead of replacing the cell.
- `this → me`: inside methods, `this` is bound as `me` for field/method access. External code should prefer `me`.
- Parent:: syntax: parser groundwork exists for `Parent::method` references; calling from child uses from-call lowering.
- `?` (propagate): `expr ?` lowers to `isOk`/`getValue` branching and returns early with the Result on error.
- `peek`: tokenized and parsed; desugars to if-else chain in VM.

Event APIs

- P2PBox.on/onOnce/off: handlers now accept both `MethodBox` and `FunctionBox`.
  - `MethodBox` handlers invoke the bound method on receive with arguments `(intent, from)`.
  - `FunctionBox` handlers execute the function body with params bound from `(intent, from)` (excess args ignored).

Notes

- RefCell-backed locals captured by closures will reflect assignments (`x = ...`) in the outer scope.
- For plugin-backed boxes, assignment and argument passing uses share semantics to preserve identity.

MIR/VM call unification (Phase 12)

- MIR `Call`: accepts either a function name (String) or a `FunctionBox` value.
  - If the callee is a String, VM performs a named-function dispatch (existing path).
  - If the callee is a `FunctionBox` (BoxRef), VM runs it via the interpreter helper with captures/`me` injected and proper return propagation.
- Lambda immediate calls are still directly lowered inline for P1 compatibility.
- Lambda→FunctionBox: Lambda expressions now lower to a `FunctionNew` MIR instruction that constructs a `FunctionBox` value (minimal: captures currently omitted). This enables MIR-only pipelines to construct and call function values.
