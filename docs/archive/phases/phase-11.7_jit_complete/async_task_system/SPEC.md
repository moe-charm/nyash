# Async Task System — SPEC

Scope: Define a structured concurrency model for Nyash with TaskGroup and Future as Boxes. Implementable across VM and JIT/EXE without adding new MIR instructions.

## User‑Facing API (Everything is Box)

Box TaskGroup
- spawn(fn: () -> T) -> Future<T>
- cancelAll() -> void
- joinAll() -> void
- fini: must run cancelAll() then joinAll() (LIFO order) to ensure structured shutdown.

Box Future<T>
- await(timeout_ms?: int) -> Result<T, Err>
  - Ok(value)
  - Err(Timeout) | Err(Cancelled) | Err(Panic)
- cancel() -> void (idempotent)

Sugar
- nowait v = expr is sugar for: let g = current_group(); v = g.spawn(lambda: expr)

Default Ownership
- An implicit TaskGroup is created per function scope and for Main. It owns tasks spawned in that scope.
- Leaving the scope triggers cancelAll→joinAll on its group (LIFO), unless tasks were moved to a longer‑lived group explicitly.

Detachment (discouraged)
- unsafe_spawn_detached(fn) only in advanced modules. Verifier should disallow use in normal code paths.

## MIR Mapping

- No new MIR instructions. Use existing BoxCall/PluginInvoke forms.
- TaskGroup.spawn → BoxCall on TaskGroup Box, returns Future Box.
- Future.await → BoxCall on Future Box with optional timeout parameter.
- Lowerer inserts safepoint around await: ExternCall env.runtime.checkpoint before and after the await call.

Example Lowering (high level)
- AST: nowait fut = arr.length()
- MIR (normalized):
  - recv = … (arr)
  - mname = Const("length")
  - fut = ExternCall iface="env.future", method="spawn_instance", args=[recv, mname]
  - v = BoxCall Future.await(fut, timeout_ms?)

Note: In the final API, TaskGroup.spawn replaces env.future.spawn_instance, but the MIR contract remains BoxCall/ExternCall‑based.

## VM Semantics

- Scheduler: SingleThreadScheduler initially; spawn enqueues closure in FIFO. safepoint_and_poll() runs due tasks.
- Future: implemented with Mutex+Condvar; set_result notifies; await waits with optional timeout; on cancel/timeout, returns Result.Err.
- CancellationToken: parent→child propagation only, idempotent cancel().
- TaskGroup: holds token and child registry; fini enforces cancelAll→joinAll (LIFO).

## JIT/EXE Semantics

- NyRT C‑ABI Shims:
  - nyash.future.spawn_method_h(type_id, method_id, argc, recv_h, vals*, tags*) -> i64 (Future handle)
  - nyash.future.spawn_instance3_i64(a0, a1, a2, argc) -> i64 (Future handle, by name/first args)
  - nyash.future.await_h(handle, timeout_ms?) -> i64/handle (Result encoding handled at Nyash layer)
- Await shim must poll safepoints and honor timeout; returns 0 or sentinel; Nyash layer maps to Result.Err.*

## Errors & Results

- Distinguish Timeout vs Cancelled vs Panic.
- Logging: concise, off by default; env flags can enable traces.
- No silent fallback: unimplemented paths error early with clear labels.

## GC & Safepoints

- Lowerer must emit env.runtime.checkpoint immediately before and after await calls.
- Scheduler.poll runs at checkpoints; long loops should explicitly insert checkpoints.

## Configuration

- NYASH_AWAIT_MAX_MS (default 5000) — global default timeout for await when not specified.
- NYASH_SCHED_POLL_BUDGET — tasks per poll, default 1.
- NYASH_SCHED_TRACE — prints poll/move/ran counts when 1.

## Security & Determinism

- Structured shutdown prevents orphan tasks after parent exit.
- LIFO joinAll reduces deadlock surfaces.
- Detached tasks are explicit and rare.

