ScopeBox and MIR Scope Hints (Dev/CI option)

Overview
- ScopeBox is an optional, compile-time-only wrapper that makes lexical scopes explicit in the AST for diagnostics and macro visibility. It is a no-op for execution: MIR lowering treats ScopeBox like a normal block and semantics are unchanged.

How to enable
- Inject ScopeBox wrappers during core normalization by setting:
  - `NYASH_SCOPEBOX_ENABLE=1`
  - Selfhost compiler path: Runner maps `NYASH_SCOPEBOX_ENABLE=1` to child arg `--scopebox` and applies a JSON prepass via `apps/lib/scopebox_inject.nyash`（現状は恒等: 構文確認のみ）。
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

Notes (Selfhost path)
- 当面は JSON v0 に `ScopeBox` 型は導入しない（互換維持）。前処理は恒等（identity）から開始し、将来は安全な包み込み/ヒント付与を検討する。
