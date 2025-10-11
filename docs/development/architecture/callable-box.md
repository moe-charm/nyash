# CallableBox — Function/Method Reference (Phase 15.7)

Purpose
- Treat function/method references as first-class Boxes.
- Unify sync and async calls via the same surface (call/callAsync) and route through MirCall.

Highlights
- Single Route: Router normalizes to `route(recv, method, argv)`; the same for builtin, plugin, user boxes.
- SSOT: TypeRegistry holds slots for `CallableBox` methods (arity/call/callAsync/toString).
- Interop: Works with Map/Array/String; can be stored in `MapBox` like a function pointer map.

API (Box surface)
- `arity() -> IntegerBox`
- `call(args: ArrayBox) -> Box|Void`
- `callAsync(args: ArrayBox) -> FutureBox`
- `toString() -> StringBox`

Creation
- Instance method reference: `a.methodRef(name: string, arity: int) -> CallableBox` (ArrayBox slot 113)
- External helper: `env.callable.{make|from|from_instance}(recv, method, arity) -> CallableBox`

Slots (SSOT)
- CallableBox (500+):
  - 500: `arity()`
  - 501: `call(argsArray)`
  - 502: `callAsync(argsArray)`
  - 503: `toString()`
- ArrayBox additions:
  - 113: `methodRef(name, arity)` → `CallableBox`

Sync/Async
- `call` runs synchronously and returns a Box/Value.
- `callAsync` returns a `FutureBox`.
  - Plugin-backed receivers: when `HAKO_CALLABLE_ASYNC=1`, tasks are spawned via `global_hooks::spawn_task` and resolved asynchronously.
  - Builtin receivers (Array/Map/String): when `HAKO_CALLABLE_ASYNC=1`, calls are scheduled onto the single-thread scheduler and resolved on VM polling (job-queue model).
  - Fallback: when async is OFF, `callAsync` executes synchronously and returns a ready `FutureBox`.
  - Await via `env.future.await` (alias: `env.future.wait`).

Notes
- Arity guard uses TypeRegistry; incorrect arity fails fast.
- Until parser support for `ref` sugar is added, use `methodRef(...)` or `env.callable.make(...)`.
- VM polling: the interpreter performs `global_hooks::safepoint_and_poll()` at instruction boundaries when `HAKO_CALLABLE_ASYNC=1` (or `*_VM_POLL_SCHED=1`), ensuring scheduled builtin calls make progress.
 - VM-internal only: Callable/Future/Result are not transported across Hako ABI. Create and use them inside the VM. When persistence/transport is truly needed, prefer handle-based representations and keep semantics in core.
