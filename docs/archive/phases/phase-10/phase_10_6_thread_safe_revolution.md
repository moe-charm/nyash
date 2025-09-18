# Phase 10.6 — Thread-Safe Revolution (Design Prep)

Status: preparation

Principles
- Opt-in concurrency: default runtime remains single-threaded for simplicity.
- All extension points intended for cross-thread use must be `Send + Sync`.

What’s prepared
- `GcHooks: Send + Sync` to allow multi-threaded collectors later.
- `NyashRuntime` holds `Arc<dyn GcHooks>` for safe sharing across threads.

Planned tasks
- Audit `NyashBox` implementations for `Send + Sync` (introduce marker traits or wrappers).
- Introduce scheduler abstraction for futures/actors (no global state).
- Introduce interior mutability strategy `RwLock` on shared mutable state, with minimal contention.

Notes
- Until the audit, VM enforces single-threaded access; sharing across threads is unsupported by default.

