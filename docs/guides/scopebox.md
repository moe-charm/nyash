ScopeBox and MIR Scope Hints (Dev/CI option)

Overview
- ScopeBox is an optional, compile-time-only wrapper that makes lexical scopes explicit in the AST for diagnostics and macro visibility. It is a no-op for execution: MIR lowering treats ScopeBox like a normal block and semantics are unchanged.

How to enable
- Inject ScopeBox wrappers during core normalization by setting:
  - `NYASH_SCOPEBOX_ENABLE=1`
- Injection points:
  - If.then / If.else bodies
  - Loop.body
  - Bare blocks are represented by `Program { statements }` and already get ScopeEnter/ScopeLeave hints.

MIR Scope Hints (unified env)
- Configure hint output with a single env using a pipe-style syntax:
  - `NYASH_MIR_HINTS="<target>|<filters>..."`
- Targets:
  - `trace` or `stderr`: print human-friendly hints to stderr
  - `jsonl=<path>` or a file path: append one JSON object per line
- Filters:
  - `all` (default), `scope`, `join`, `loop`, `phi`
- Examples:
  - `NYASH_MIR_HINTS="trace|all"`
  - `NYASH_MIR_HINTS="jsonl=tmp/hints.jsonl|scope|join"`
  - `NYASH_MIR_HINTS="tmp/hints.jsonl|loop"`
- Back-compat:
  - `NYASH_MIR_TRACE_HINTS=1` is still accepted (equivalent to `trace|all`).

Zero-cost policy
- ScopeBox is removed implicitly during MIR lowering (treated as a block). ScopeEnter/ScopeLeave hints are observational only. Execution and IR are unchanged.

