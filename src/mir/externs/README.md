Externs Registry (Thin Box)

Purpose
- Provide a single place to declare known extern endpoints like "env.console.log" or "nyrt.ops.op_eq".
- Keep backends (VM/LLVM/WASM) loosely coupled by sharing only names and minimal metadata.

Scope (MVP)
- Known set query: is_known("iface.method") -> bool
- Optional minimal signature info (arity only for now)

Non‑goals (MVP)
- Full type system for externs
- Backend‑specific lowering policies

Usage
- Builders emit Call with callee=Extern("iface.method").
- Backends may consult this registry for quick validation or diagnostics.

Fail‑Fast
- Unknown externs should be surfaced clearly by backends.

