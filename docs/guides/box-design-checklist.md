# Box Design Checklist (docs-only)

Use this checklist when introducing a new Box or evolving an existing one.

## Lifecycle
- Define birth parameters and side effects (allocation, registration).
- State machine: initial → running → closed → fini; reentrancy rules.
- `fini()` idempotent; post-fini methods: error vs no-op.

## Ownership & Sharing
- Who owns the resource? Is there a LeaseBox form?
- Mutation boundaries (single-thread/Actor only?).
- Cross-thread usage: mailbox-only or direct allowed?

## Concurrency
- Blocking APIs: provide `try_*` and `*_timeout(ms)`.
- Cancellation: accept CancelToken/Deadline; ensure prompt unblock.
- Busy-wait forbidden; document waiting strategy (Phase‑0 cooperative / later OS).

## Close Semantics
- What does `close()` mean? Which ops fail after close?
- Draining behavior and End marker shape.
- Double close and use-after-close handling.

## Observability
- Events to emit (op, ok, extra fields) and Box ID.
- Env toggles (e.g., `NYASH_*_TRACE=1`).
- Expected order/causality in traces.

## Capabilities & Affinity
- External authorities needed (fs/net/io/thread).
- Thread affinity constraints (creator thread / any / Actor only).

## Performance
- Hot vs cold paths; offer ThinBox where needed.
- Complexity of operations; backpressure strategy.

## Errors & Types
- Error shape (Bool/Option/Result) and consistency with ecosystem.
- Type expectations (runtime tags vs future static types).

