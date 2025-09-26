# Phase 17 — Minimal, Clean, Small (Blueprint)

Purpose: ship a tiny, beautiful vertical slice that runs Core‑13 IR end‑to‑end, is observable, and easy to extend. Keep everything additive and simple.

1) Scope (MVP)
- IR: Core‑13 only (Loop IR placeholder only).
- Exec: Wrap existing VM with a tiny `ExecEngine` adapter.
- Remote: One transport only — NDJSON over stdio (`nyash-engine-core13`).
- CLI: `run`, `ir-emit`, `ir-run`. Trace: Enter/Exit only.

2) ExecEngine (tiny)
- Value: `Int(i64) | Bool(bool) | Str(String) | Box(u64) | None`.
- Trait:
  - `load(&Core13Module) -> ModuleHandle`
  - `get(&ModuleHandle, func: &str) -> FuncHandle`
  - `call(&FuncHandle, args: &[Value]) -> Result<Value>`
- Adapter: `VMEngine` delegates to existing VM (`execute_module`), converts Value <-> NyashBox.

3) Core‑13 IR JSON (minimal)
- module: `{ schema:1, name:"m", funcs:[ { name, params:[], ret:"i64|bool|string|box|void", blocks:[...] } ] }`
- block: `{ id:0, inst:[...], term:{...} }`
- inst (13 only): `Const, BinOp, Compare, Jump, Branch, Return, Phi, Call, BoxCall, ExternCall, TypeOp, Safepoint, Barrier`.
- example Const: `{ op:"Const", dst:1, ty:"i64", value:42 }`
- example Return: `{ term:"Return", value:1 }`

4) NDJSON protocol (stdio)
- Common: every request has `op,id,schema(=1)`; every response `{ok:true|false,id,...}`; unknown keys ignored.
- Ops (MVP): `load_module`, `call`, `ping`, `trace_sub` (Enter/Exit only).
- Requests:
  - load_module: `{op:"load_module",id:1,ir:"core13",format:"json",bytes:"<base64>"}`
  - call: `{op:"call",id:2,module_id:1,func:"main",args:[]}`
  - ping: `{op:"ping",id:3}`
  - trace_sub: `{op:"trace_sub",id:4,mask:["EnterFunc","ExitFunc"],flush_ms:50?}`
- Responses:
  - `{ok:true,id:1,module_id:1,features:["interp","trace"],ir:"core13"}`
  - `{ok:true,id:2,value:42,events_dropped:0}`
  - `{ok:true,id:3,now:1725600000}`
  - `{ok:true,id:4}` (events then stream as separate lines)
- Events (lines): `{"event":"EnterFunc","func":"main"}` / `{"event":"ExitFunc","func":"main"}`.

5) CLI UX
- `nyash run --engine=vm apps/hello.nyash`
- `nyash run --engine=remote --exe ./nyash-engine-core13 apps/hello.nyash`
- `nyash ir-emit --ir=core13 --format=json apps/hello.nyash > out.json`
- `nyash ir-run --engine=vm < out.json`

6) Milestones
- M1: ExecEngine + VMAdapter + `run --engine=vm` (tests: add/if/string.length)
- M2: Core‑13 serde/verify + `ir-emit`/`ir-run` (round‑trip + 1 exec test)
- M3: `nyash-engine-core13` NDJSON (load/call/ping/trace_sub) + `run --engine=remote` parity

7) Design rules (beauty & simplicity)
- Additive evolution only: version fields present (`schema`), unknown keys ignored.
- Deterministic JSON: stable key order where practical to make diffs readable.
- Naming: short, explicit; avoid redundancy; keep method ids optional.
- Observability first: ship with Enter/Exit trace; branch/extern later.

Appendix: Two request/response examples
1) Load then call
REQ: `{ "op":"load_module", "id":1, "schema":1, "ir":"core13", "format":"json", "bytes":"<base64>" }`
RESP:`{ "ok":true, "id":1, "module_id":1, "features":["interp","trace"], "ir":"core13" }`
REQ: `{ "op":"call", "id":2, "schema":1, "module_id":1, "func":"main", "args":[] }`
RESP:`{ "ok":true, "id":2, "value":42, "events_dropped":0 }`

2) Trace subscribe and ping
REQ: `{ "op":"trace_sub", "id":10, "schema":1, "mask":["EnterFunc","ExitFunc"] }`
RESP:`{ "ok":true, "id":10 }`
EVT: `{ "event":"EnterFunc", "func":"main" }`
EVT: `{ "event":"ExitFunc", "func":"main" }`
REQ: `{ "op":"ping", "id":11, "schema":1 }`
RESP:`{ "ok":true, "id":11, "now":1725600000 }`

