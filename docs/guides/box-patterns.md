# Box Patterns (dev guide; docs-only)

Status: design notes to standardize reusable Box idioms. No runtime/spec change during the feature‑pause.

## OwnershipBox / LeaseBox
- Intent: make ownership vs borrowing explicit for resource Boxes.
- Rules
  - OwnershipBox owns the underlying handle; calls `fini()` on drop.
  - LeaseBox references an owned Box without taking responsibility to `fini()`.
  - Upgrading a lease to ownership must be explicit (and validated).
- Smells avoided: double-close, leaked handles, implicit transfer.

## CancelTokenBox / DeadlineBox
- Intent: structured cancellation and time limits across waiting APIs.
- Rules
  - Token can be passed to blocking APIs (Channel/Select/Waiter) to abort waits.
  - DeadlineBox wraps an absolute time; APIs accept either `token` or `deadline`.
  - Cancellation is idempotent; multiple cancel calls are safe.
- Effects
  - Waiting ops return promptly with a well-typed cancel/timeout indicator.

## CapabilityBox
- Intent: define minimal authority (I/O, net, thread, fs) explicitly.
- Rules
  - Boxes that access external capabilities must declare the capability dependency.
  - Tests can substitute Noop/Mock capability Boxes.
- Effects
  - Principle of least privilege; easier sandboxing and auditing.

## AffinityBox
- Intent: encode thread/runtime affinity constraints.
- Rules
  - Annotate Boxes that must be used on their creator thread or via Actor mailbox.
  - Violations produce early, explicit errors (not UB).
- Effects
  - Predictable behavior across concurrency boundaries.

## ObservableBox
- Intent: unify tracing/metrics hooks.
- Rules
  - Emit JSONL events with `ts`, `box_id`, `op`, `ok`, and domain-specific fields.
  - Allow opt-in (`NYASH_*_TRACE=1`) and keep logs stable for tooling.
- Effects
  - Cross-cutting visibility with one schema; simpler troubleshooting.

## Composition Tips
- Separate hot/cold paths: ThinBox (hot) vs RichBox (full) to avoid overhead on the critical path.
- Prefer immutable handles + message passing across threads (Actor) to avoid races.
- Keep `birth/fini` idempotent; document post-fini behavior (no-op vs error).
