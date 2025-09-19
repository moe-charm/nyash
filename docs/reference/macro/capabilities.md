# Macro Capabilities (Sandbox Policy)

Status: Design/PoC. This document defines the capability model for user macros executed via the PyVM sandbox.

## Goals
- Determinism by default: macro expansion should be reproducible and isolated.
- Principle of least privilege: nothing is allowed unless explicitly requested.
- Simple, declarative switches per macro file (future: nyash.toml), with safe defaults.

## Capability Set (v0)
- io: allow file I/O (read‑only at first; write is out of scope for Phase 2)
- net: allow network access (HTTP/Socket); default OFF
- env: allow environment variable reads via a MacroCtx helper (getEnv); default OFF

Default: all OFF (io=false, net=false, env=false)

## Behavior per Capability
- io=false
  - Disable FileBox and other I/O boxes in the macro sandbox.
  - No reads from the filesystem; macros operate purely on the AST JSON.
- net=false
  - Disable Http/Socket boxes.
  - No external calls; prevents non‑deterministic expansion due to remote content.
- env=false
  - MacroCtx.getEnv is unavailable (returns an error / empty); child inherits a minimal, scrubbed environment.

## Configuration (planned)
nyash.toml example (subject to refinement):
```
[macros]
paths = [
  "apps/macros/examples/echo_macro.nyash",
  "apps/macros/examples/upper_string_macro.nyash",
]

[macro_caps."apps/macros/examples/echo_macro.nyash"]
io = false
net = false
env = false

[macro_caps."apps/macros/examples/upper_string_macro.nyash"]
io = false
net = false
env = false
```

Phase‑2 PoC maps these to the child process environment/sandbox:
- Always sets: `NYASH_VM_USE_PY=1`, `NYASH_DISABLE_PLUGINS=1`
- Timeouts: `NYASH_NY_COMPILER_TIMEOUT_MS` (default 2000ms)
- Strict execution: `NYASH_MACRO_STRICT=1` (default) → child failure/timeout aborts the build
- Future: map `io/net/env` to enabling specific safe Boxes inside the PyVM macro runtime

## Execution Flow (recap)
1) Parse → AST
2) Expansion pass (fixed‑point with cycle/depth guards):
   - Built‑in (Rust) macros first (e.g., derive Equals/ToString, public‑only)
   - User macros (Nyash) via Proxy → PyVM child
3) Using/resolve → MIR → Backend (VM/LLVM/WASM/AOT)

## Diagnostics/Observability
- Trace JSONL: `NYASH_MACRO_TRACE_JSONL=<file>` produces one JSON record per pass with size/time/change flags.
- Dump expanded AST: `--dump-expanded-ast-json <file.nyash>` prints AST JSON v0 post‑expansion for golden diffs.
- Strict mode failures are surfaced with non‑zero exit codes (2/124).

## Recommendations
- Keep macros pure (operate only on AST JSON v0) unless there is a strong case for capabilities.
- Treat `net=true` as exceptional and subject to policy review, due to determinism concerns.
- Prefer deterministic inputs (versioned data files) if `io=true` is deemed necessary in future.

