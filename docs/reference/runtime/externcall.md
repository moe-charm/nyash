# ExternCall — Runtime Interfaces (env.*)

Overview
- ExternCall represents calls to host-provided interfaces, addressed by an interface name and method, e.g. `env.console.log`.
- In Nyash, source-level `print/println` normalize to an ExternCall targeting the console interface.

Normalization of println/print
- Builder/Optimizer rewrites language-level printing to ExternCall:
  - `println(x)` or `ConsoleBox.println(x)` → `ExternCall("env.console", "log", [x])`
- Rationale: unifies backends and keeps ConsoleBox for other roles (vtable optimization, advanced console APIs).

Backend Behavior
- LLVM/AOT (EXE-first):
  - `env.console.log` is normalized in the LLVM builder to kernel exports and links statically.
  - Primary mapping uses pointer-API when possible to avoid handle churn:
    - `nyash.console.log(i8*) -> i64`
    - Fallback to handle-API helpers (`nyash.string.to_i8p_h`) if only a handle is available.
  - Runtime result line: the kernel prints `Result: <code>` after `ny_main()` returns. Set `NYASH_NYRT_SILENT_RESULT=1` to suppress for tests.
- PyVM:
  - Accepts `env.console.log/warn/error` and writes to stdout (MVP). Return is `0` when a destination is present.
- JIT:
  - Host-bridge directly prints the first argument to stdout for `env.console.log` minimal parity.

MIR JSON v0 Encoding
- Instruction shape:
  - `{ "op": "externcall", "func": "env.console.log", "args": [<vid>], "dst": <vid|null>, "dst_type": "i64"? }`
- Builder may also emit `"func": "nyash.console.log"` in some paths; both are accepted by backends. The LLVM builder maps `env.console.*` to `nyash.console.*` automatically.

Key Fields (JSON v0, minimal)
- `op`: literal `"externcall"`.
- `func`: fully qualified name. Preferred: `"env.console.log"`. Accepted: `"nyash.console.log"`.
- `args`: value-ids array (each is an integer referencing a producer).
- `dst`: optional value-id to store result; may be null when ignored.
- `dst_type` (optional): backend hint. For console methods, `"i64"` is used when `dst` is present.


Return Value Convention
- Console methods return `i64` status (typically 0). Most user code ignores it; when `dst` is set, backends materialize `0`.

Guidance
- Use `print/println` in Nyash source; the toolchain normalizes to ExternCall.
- Prefer exit code assertions in EXE-first tests. If you must compare stdout, set `NYASH_NYRT_SILENT_RESULT=1` to hide NyRT's `Result:` line.
