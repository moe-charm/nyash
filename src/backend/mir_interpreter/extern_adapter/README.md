# Extern Adapter — Boxed Registrations

Purpose
- Keep the extern adapter (`extern_adapter.rs`) as a small hub.
- Actual handlers live in thin modules under this folder and expose `register(&mut map)`.

Modules
- `extern_core.rs`: time/string/map (core semantics)
- `extern_future_legacy.rs`: env.future.* (legacy-boxes gated)
- `extern_file_dev.rs`: nyrt.file.* (dev convenience)
- `extern_rune_dev.rs`: nyrt.rune.eval (dev mock)
- `extern_nykernel_stub.rs`: nykernel.* (opt-in dev stub)

Gates & Policy
- Legacy-only pieces are guarded with `#[cfg(feature="legacy-boxes")]` in module implementations.
- These dev modules are non-essential and may be removed or replaced later; keeping them boxed simplifies rollback.

