# Nyash Concurrency — Box Model (Proposal, docs-only)

Status: design-only during the feature‑pause. No runtime or spec changes. Implement after Mini‑VM baseline is stable.

Intent
- Bring Go-like CSP (goroutine/channels/select) into Nyash via “Everything is Box”.
- Keep semantics explicit, lifecycle safe (birth/fini), and observable. Phase-in from userland → runtime.

Scope (Phase‑0: userland MVP)
- RoutineBox: lightweight task wrapper over `nowait` (state, join/cancel, status).
- ChannelBox: bounded/unbounded queue + blocking/non-blocking ops + close semantics.
- SelectBox: multi-channel wait (first-ready) with simple fairness.
- RoutineScopeBox: structured concurrency; children are canceled on scope fini.
- Observability: JSONL trace toggled by `NYASH_CONC_TRACE=1`.

Non‑Goals (Phase‑0)
- M:N scheduler, OS-level park/unpark, net poller integration (deferred to Phase‑2 runtime work).

API Sketch (userland)
- RoutineBox
  - birth(fn)
  - start(): Void
  - join(timeout_ms?: Int) -> Bool  // true if joined; false on timeout
  - cancel(): Void
  - status() -> String  // ready|running|done|canceled|error
- ChannelBox(capacity: Int=0)
  - send(v): Void  // blocks if full (Phase‑0: simulated park)
  - try_send(v) -> Bool
  - receive() -> Any  // blocks if empty (Phase‑0: simulated park)
  - try_receive() -> (Bool, Any?)
  - receive_timeout(ms: Int) -> (Bool, Any?)
  - close(): Void  // further send fails; recv drains until empty then End
- SelectBox
  - birth()
  - when(ch: ChannelBox, handler: Fn): Void
  - await() -> Bool  // returns after one handler runs; false if none ready and no wait policy
  - await_timeout(ms: Int) -> Bool
- RoutineScopeBox
  - birth()
  - spawn(fn) -> RoutineBox
  - fini()  // cancels pending routines and waits boundedly

Semantics
- Capacity:
  - 0: rendezvous channel (send/recv rendezvous).
  - N>0: bounded ring buffer.
- Close:
  - close() marks channel as closed. send() after close -> error. receive() returns buffered items; when empty -> (false, End) style result; exact return shape defined per API.
- Blocking:
  - Phase‑0 userland uses cooperative wait queues; no busy loops. try_* and timeout variants provided.
- Select fairness:
  - If multiple ready, choose random/round‑robin. Starvation avoidance is a design requirement; precise algorithm can evolve.
- Types:
  - `TypedChannelBox<T>` is a future extension; Phase‑0 uses runtime tags/guards documented in reference.
- Cancellation:
  - RoutineScopeBox cancels children on fini; Channel waits should return (canceled) promptly.

Phases
- Phase‑0 (userland MVP / PyVM first)
  - Implement the 4 boxes above with minimal queues/waits, plus trace hooks.
  - Smokes: ping‑pong, bounded producer/consumer, two‑way select, close semantics, scope cancel.
- Phase‑1 (park/unpark abstraction)
  - Introduce `WaiterBox`/`CondBox` that map to efficient OS waits where available. Keep same APIs.
- Phase‑2 (runtime integration)
  - Scheduler (M:N), GC and net poller integration, fairness and profiling. Keep Box APIs stable.

Observability
- `NYASH_CONC_TRACE=1` → JSONL events: spawn/join/cancel/send/recv/park/unpark/select/close with routine IDs, channel IDs, timestamps.

Safety & Diagnostics
- Deadlock hints: trace dependent waits; optional detector (dev only) can dump wait‑for graph.
- API contracts explicitly define error return for misuse (send on closed, double close, etc.).

Deliverables (docs‑only during the feature‑pause)
- This proposal (boxes & semantics).
- Reference page with blocking/close/select rules (see reference/concurrency/semantics.md).
- Test plan with named smokes and expected outputs.
