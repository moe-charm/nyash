## Concurrency Semantics (docs-only; to guide MVP)

Status: reference for userland Phase‑0. No runtime changes during the feature‑pause.

### Blocking & Non-Blocking
- `send(v)` blocks when the buffer is full (capacity reached). Unblocks when a receiver takes an item.
- `receive()` blocks when buffer is empty. Unblocks when a sender provides an item.
- `try_send(v) -> Bool`: returns immediately; false if full or closed.
- `try_receive() -> (Bool, Any?)`: returns immediately; false if empty and not closed.
- `receive_timeout(ms) -> (Bool, Any?)`: returns false on timeout.

Implementation note (Phase‑0): no busy loop. Use cooperative queues; later replaced by OS-efficient wait.

### Close Semantics
- `close()` marks the channel closed. Further `send` is an error.
- After close, `receive` continues to drain buffered items; once empty, returns a closed indicator (shape per API, e.g., `(false, End)`).
- Double close is an error.

### Select Semantics
- `when(channel, handler)` registers selectable cases.
- `await()` examines cases:
  - If one or more ready: choose one (random/round‑robin) and execute its handler atomically.
  - If none ready: Phase‑0 may block via a multi-wait helper or return false if non-blocking policy requested.
- Fairness: no starvation requirement; Phase‑0 uses simple fairness; Phase‑2 integrates runtime help.

### Structured Concurrency
- `RoutineScopeBox` owns spawned routines.
- On `fini()`: cancel pending children, bounded join, and ensure resource release.
- Cancellation should unblock channel waits promptly.

### Types & Safety
- Phase‑0: runtime tag checks on `ChannelBox` send/receive are optional; document expected element type.
- Future: `TypedChannelBox<T>` with static verification; falls back to runtime tags when needed.

### Observability
- Enable `NYASH_CONC_TRACE=1` to emit JSONL events.
- Event schema (recommendation):
  - Common: `ts` (ms), `op` (string), `rid` (routine id), `cid` (channel id, optional), `ok` (bool), `scope` (scope id, optional)
  - spawn/start/join/cancel: `{op:"spawn", rid, ts}`, `{op:"join", rid, ok, ts}`
  - send/recv: `{op:"send", cid, size, cap, ts}`, `{op:"recv", cid, size, cap, ts}`
  - park/unpark: `{op:"park", rid, reason, ts}`, `{op:"unpark", rid, reason, ts}`
  - select: `{op:"select", cases:n, chosen:k, ts}`
  - close: `{op:"close", cid, ts}`
- Causality: producers must emit before consumers observe; timestamps monotonic per process.

### Test Plan (smokes)
- ping_pong.nyash: two routines exchange N messages; assert order and count.
- bounded_pc.nyash: producer/consumer with capacity=1..N; ensure no busy-wait and correct totals.
- select_two.nyash: two channels; verify first-ready choice and distribution.
- close_semantics.nyash: send after close -> error; drain -> End; double close -> error.
- scope_cancel.nyash: RoutineScopeBox cancels children; parked receivers unblocked.

### Migration Path
- Phase‑0 userland boxes are kept while Phase‑2 runtime grows; API stable.
- Replace cooperative wait with WaiterBox (Phase‑1) and runtime park/unpark later.
