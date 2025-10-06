# ExternKernelBox (dev stub)

Purpose
- Centralize the nykernel.* development stub (malloc/load_i64/store_i64).
- Single policy for 8-byte alignment, optional auto-resize, and trace.

I/O and Policy
- Enabled when `NYASH_ENABLE_NYKERNEL_STUB=1`.
- 8-byte aligned addresses; negative or unaligned addrs are rejected.
- `store_i64` auto-resizes heap in dev (no panic), subject to change when hardened.

Implementation
- Rust module: `src/runtime/nykernel_stub.rs`
- Used by VM extern adapter and plugin loader externs.

Future
- Add optional tracing and bounds stats (dev-only env toggles) without changing behavior.
