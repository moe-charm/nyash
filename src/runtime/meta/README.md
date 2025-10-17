# Runtime Meta Layer (Callable/Future)

Responsibility
- Host-owned meta boxes that support language execution:
  - `CallableBox`: method references with arity and optional receiver
  - `FutureBox`: minimal async result container for callAsync/external tasks

Contract
- No external I/O in this layer
- No direct plugin dependency; only VM/GC/Scheduler allowed
- Fail-Fast: invalid usage must surface as errors; no silent fallbacks

Public Surface (minimal)
- CallableBox: `new(receiver, method, arity)`, `arity()`, `to_string_box()`, receiver share
- FutureBox: `new()`, `set_result(box)`, `get()`, `ready()`, `downgrade()`

Notes
- When the `legacy-boxes` feature is enabled, legacy implementations are reused.
- In plugin-only builds, thin host implementations are provided with the same API.

Migration
- Re-exports `runtime::{callable_box,future_box}` have been removed (Phase‑31 cleanup).
- Always import from `runtime::meta::{callable,future}` directly.
