Calls Handler Split (Phase 1)

Scope
- Non‑breaking structural split to prepare extraction of `calls.rs` into:
  - function.rs (global functions)
  - method.rs (instance/static methods)
  - extern_call.rs (extern glue/adapters)
  - box_call.rs (builtin fast‑paths) — removed in Phase 15.7; routing unified via User/Plugin paths
  - legacy.rs (original implementation; unchanged semantics)

Policy
- VM convenience handlers and fast‑paths are removed (Phase 15.7). Behavior now follows vtable/extern routing.
- Parity is guarded by plugin‑on smokes; quick remains green with plugins enabled.
