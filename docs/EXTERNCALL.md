# ExternCall Policy (Phase 15)

## Allowed Interfaces (minimal set)
- `env.console.{log,warn,error,readLine}`
- `env.debug.trace`
- `env.system.{exit,now}` (if present)

All other host interactions should go through BoxCall (NyRT or plugins).

## Argument‑type‑based selection
- For `env.console.{log,warn,error}` and `env.debug.trace`:
  - If the single argument is `i8*` (C string), call the C‑string variant:
    - `nyash.console.log(i8*)`, `nyash.console.warn(i8*)`, `nyash.console.error(i8*)`
    - `nyash.debug.trace(i8*)`
  - Otherwise convert to `i64` and call the handle variant:
    - `nyash.console.log_handle(i64)`, `nyash.console.warn_handle(i64)`, `nyash.console.error_handle(i64)`
    - `nyash.debug.trace_handle(i64)`

## Rationale
- Keeps the AOT string path fast and avoids accidental `inttoptr` of handles.
- Avoids adding broad implicit conversions in ExternCall; selection is local and explicit.

## Non‑LLVM Backends
- VM, Cranelift JIT/AOT, and the interpreter may not implement this policy yet (not MIR14‑ready). LLVM is authoritative; other backends will align after stabilization.

