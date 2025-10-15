# Extern Adapter — Boxed Registrations

Purpose
- Keep the extern adapter (`extern_adapter.rs`) as a small hub.
- Actual handlers live in thin modules under this folder and expose `register(&mut map)`.

Modules
- `extern_string.rs`: nyrt.string.* (length/indexOf/lastIndexOf/substring/charAt/replace)
- `extern_array.rs`:  nyrt.array.*  (size/length)
- `extern_map.rs`:    nyrt.map.*    (size/keys/values)
- `extern_set.rs`:    nyrt.set.*    (Map-backed Set: add/remove/has/size/clear/toArray)
- `extern_env.rs`:    env.local.get / nyash.json.canonicalize_h
- time.now_ms: registered inline in the adapter hub (tiny handler; no dedicated module)
- `extern_future_legacy.rs`: env.future.* (legacy-boxes gated)
- `extern_file_dev.rs`: nyrt.file.* (dev convenience)
- `extern_rune_dev.rs`: nyrt.rune.eval (dev mock)
- `extern_nykernel_stub.rs`: nykernel.* (opt-in dev stub)

Gates & Policy
- Legacy-only pieces are guarded with `#[cfg(feature="legacy-boxes")]` in module implementations.
- These dev modules are non-essential and may be removed or replaced later; keeping them boxed simplifies rollback.
