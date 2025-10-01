Calls Handler Split (Phase 1)

Scope
- Non‑breaking structural split to prepare extraction of `calls.rs` into:
  - function.rs (global functions)
  - method.rs (instance/static methods)
  - extern_call.rs (extern glue/adapters)
  - box_call.rs (builtin fast‑paths)
  - legacy.rs (original implementation; unchanged semantics)

Policy
- Behavior unchanged; tests should remain green.
- Moves will be done in small steps; legacy stays until parity is verified.
- Rollback is trivial: remove new files and keep legacy.rs only.

